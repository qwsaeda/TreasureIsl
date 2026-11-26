//! S87.2 世界层 schema 渲染器：把 `world_components.json` 的静态元素画到 1080 世界层。
//!
//! ## 边界（零游戏语义）
//!
//! 本模块只认 [`WorldComponent`]（`type`/`role`/`binding`/`geometry`/`world_rect`/`z`/`pick`）、
//! [`AssetRoles`]（`role` → 资产路径）、`&dyn SceneHost`（白名单访问器）与
//! [`ti_render::canvas::Canvas`]（绘制原语）。**不出现组合根 / 角色 / 地图槽 / 战斗快照
//! 等业务类型**：业务取数全经 `SceneHost` 白名单 `path`，字符串 → 语义的映射留壳侧。
//!
//! 不产生任何 `bevy_ui` Node：所有绘制经 [`Canvas`] 原语 ⇒ CPU `MenuShell` 与
//! Bevy `DrawList` 两端自动同画面（`bevy-cpu-parity` 口径不变，UI Node 计数为 0）。
//! 世界层**不挂** `chrome_in_bevy_ui` 守卫（`vs`/`main_map` 现无此守卫）。
//!
//! ## 影子期（S87.2 不切默认路径）
//!
//! [`enabled`] 读 `TI_WORLD_SCHEMA=1`，默认 `false`；本模块**不修改** `main_map`/`vs` 的
//! `render()` 调用（S87.3/4 才接最薄一行分支）。默认路径 0 行为变化，gate 零空窗。
//!
//! ## 两阶段结构
//!
//! 1. [`resolve_world_draw_cmds`] = 后端无关中间层：z 升序 + state/可见性过滤 +
//!    `geometry`/`world_rect` 解析 + `binding`/`role`/`action` 解析 → [`ResolvedDrawCmd`]；
//! 2. [`dispatch_draw_cmds`] = 薄 Canvas 适配：只做 `color token → Theme` 与
//!    `kind → Canvas` 原语分派，语义不重复。
//!
//! 颜色一律 Theme token（`engine-no-color-literal`）；坐标只从 `geometry` 引用
//! 或 `world_rect` 来（`no-handwritten-grid`），模块内不散落裸像素。

use std::sync::OnceLock;

use ti_core::data::V6_LAYOUT_JSON_LANDSCAPE;
use ti_core::data::scenes as scene_table;
use ti_core::data::ui_schema::{
    AssetRoles, BindingSpec, VisibleWhen, WorldComponent, WorldRect, asset_roles, world_components,
};
use ti_core::domain::ui_action::UiAction;
use ti_render::Theme;
use ti_render::canvas::Canvas;

use super::{RenderHost, SceneHost, chrome};

/// 恒等 tint（RGB565 白 = `u16::MAX`）：`blit_image_tint` 乘法的单位元，
/// 与 `menu_shell` 的 `blit_image` 口径一致（S21.14）。用 `u16::MAX` 表达
/// 而非写颜色字面量，避免色值散落。
const TINT_IDENTITY: u16 = u16::MAX;

/// 文本档位（渲染器内部语义；与 `DrawOp::Tier` 一一对应，零游戏语义）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTier {
    /// 标题档。
    Title,
    /// 正文档。
    Body,
    /// 提示档。
    Hint,
}

/// 绘制种类（CPU `DrawOp` 三型；与 schema `type` 闭集对应）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawKind {
    /// 实心矩形（主题色 token）。
    Rect,
    /// 文本（档位 + 已解析文案）。
    Text(TextTier),
    /// 贴图（role → 资产路径）。
    Image,
}

/// 渲染器内部矩形值（后端无关；与 schema `WorldRect` 互转，自带 `Eq` 供测试对账）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DrawRect {
    /// 左上 x。
    pub x: i32,
    /// 左上 y。
    pub y: i32,
    /// 宽。
    pub w: i32,
    /// 高。
    pub h: i32,
}

impl From<WorldRect> for DrawRect {
    fn from(value: WorldRect) -> Self {
        Self { x: value.x, y: value.y, w: value.w, h: value.h }
    }
}

/// 后端无关绘制命令：中间层产物，bevy / CPU 两端各自薄适配。
///
/// 字段只含字符串 / 整数 / 简单枚举 / [`DrawRect`]，不携带任何业务类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDrawCmd {
    /// 元素 id（诊断 / 测试用）。
    pub id: String,
    /// 绘制种类。
    pub kind: DrawKind,
    /// 1080 世界坐标目标矩形。
    pub rect: DrawRect,
    /// 主题令牌名（Rect/Text 用；Image = 空串表示恒等 tint）。
    pub color_token: String,
    /// 已解析文案（Text 用；Image 的 binding 派生标签亦走此字段）。
    pub text: String,
    /// 资产路径（Image 用）。
    pub asset: Option<String>,
    /// `pick.action` 派生的 opaque key（UiAction 闭集；None = 不可点）。
    pub action_key: Option<u32>,
}

/// 白名单运行值：只表达「数 / 文本 / 无」，不携带业务结构。
#[derive(Debug, Clone, PartialEq, Eq)]
enum HostValue {
    /// 数值。
    Number(u64),
    /// 文本。
    Text(String),
    /// 无值（`Option::None` / 未知 path）。
    None,
}

impl HostValue {
    /// 是否无值（`visible_when.is_none` / `is_some` 判据）。
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// 展示文本（binding 无 fmt 时 / `eq`/`ne` 比较用）。
    fn text(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::Text(s) => s.clone(),
            Self::None => String::new(),
        }
    }
}

/// 影子期开关判定（纯函数便于无副作用单测）。
///
/// 仅 `"1"` 开启；其余（含未设置 / `"0"` / 任意字符串）= 关闭。
#[must_use]
pub fn env_enabled(raw: Option<&str>) -> bool {
    raw == Some("1")
}

/// 本进程是否启用世界层 schema 渲染器（`TI_WORLD_SCHEMA=1`）。
///
/// 默认 `false`；S87.3/4 在 `main_map`/`vs` 的 `render` 分支调用本函数，
/// 本阶段只提供函数，不切换默认路径。
#[must_use]
pub fn enabled() -> bool {
    // S87.5：schema 世界层是**默认渲染路径**（编辑器写盘即生效，无需重编译/无需开开关）。
    // 旧实现的「默认关闭、TI_WORLD_SCHEMA=1 才启用」是「改了数据还得记着开环境变量」的 SB 设计。
    // 保留 env 仅作**回退开关**：显式 `TI_WORLD_SCHEMA=0` 才退回旧过程式路径（用于 A/B 对比/问题排查）。
    std::env::var("TI_WORLD_SCHEMA").ok().as_deref() != Some("0")
}

/// 单元素可见状态判定（与 `ui_component::component_visible` 同口径）。
fn component_visible(component: &WorldComponent, selected: i64) -> bool {
    if component.state.is_empty() {
        return true;
    }
    let has_normal = component.state.iter().any(|s| s == "normal");
    let has_selected = component.state.iter().any(|s| s == "selected");
    match (has_normal, has_selected) {
        (true, true) | (false, false) => true,
        (true, false) => selected < 0,
        (false, true) => selected >= 0,
    }
}

/// 运行期可见性条件求值（白名单 path + `is_none`/`is_some`/`eq`/`ne`）。
fn visible_when_matches(condition: &VisibleWhen, host: &dyn SceneHost) -> bool {
    let value = host_value(host, &condition.path);
    let wanted = condition.value.as_deref().unwrap_or_default();
    match condition.op.as_str() {
        "is_none" => value.is_none(),
        "is_some" => !value.is_none(),
        "eq" => value.text() == wanted,
        "ne" => value.text() != wanted,
        _ => true,
    }
}

/// 当前面板语义名（`MapHost.panel` 运行值；按数据声明顺序取首个打开面板）。
fn open_panel_name(host: &dyn SceneHost) -> &'static str {
    if host.build_panel_open() {
        "build"
    } else if host.trade_panel_open() {
        "trade"
    } else if host.merchant_panel_open() {
        "merchant"
    } else if host.event_panel_open() {
        "event"
    } else if host.ship_panel_open() {
        "ship"
    } else if host.garrison_panel_open() {
        "garrison"
    } else if host.upgrade_panel_open() {
        "upgrade"
    } else if host.dungeon_panel_open() {
        "dungeon"
    } else if host.card_panel_open() {
        "card"
    } else if host.achievement_panel_open() {
        "achievement"
    } else {
        ""
    }
}

/// `SceneHost` 白名单 path → 运行值（S87.1 `world_components.schema.json` 同表）。
///
/// 仅列 14 条白名单；未知 path = [`HostValue::None`]（不 panic、不反射调用任意方法）。
fn host_value(host: &dyn SceneHost, path: &str) -> HostValue {
    match path {
        "EconomyHost.gold" => HostValue::Number(u64::from(host.gold())),
        "EconomyHost.cargo" => HostValue::Number(u64::from(host.boat_cap())),
        "MapHost.dice" => HostValue::Number(u64::from(host.board_last_roll())),
        "MapHost.seat" => HostValue::Number(u64::from(host.turn_seat())),
        "MapHost.panel" => HostValue::Text(open_panel_name(host).to_owned()),
        "MapHost.handoff_pending" => match host.handoff_pending() {
            Some(seat) => HostValue::Number(u64::from(seat)),
            None => HostValue::None,
        },
        "BattleHost.hp_player" => host
            .battle_snapshot()
            .map_or(HostValue::None, |s| HostValue::Number(u64::from(s.attacker_crew))),
        "BattleHost.hp_enemy" => host
            .battle_snapshot()
            .map_or(HostValue::None, |s| HostValue::Number(u64::from(s.defender_crew))),
        "BattleHost.crew" => host
            .battle_snapshot()
            .map_or(HostValue::None, |s| HostValue::Number(u64::from(s.attacker_crew))),
        "BattleHost.report" => host.siege_snapshot().map_or(HostValue::None, |s| {
            HostValue::Text(format!("{} / {}", s.attacker_crew, s.defender_crew))
        }),
        "ToastHost.text" => match host.last_ai_toast().or_else(|| host.board_toast()) {
            Some(text) => HostValue::Text(text),
            None => HostValue::None,
        },
        "OutcomeHost.text" => match host.match_outcome() {
            Some(_) => HostValue::Text(crate::i18n::sget("scene.vs.title").to_owned()),
            None => HostValue::None,
        },
        // `BattleHost.round` / `BattleHost.phase` 与未知 path 一样无可用标量 ⇒ 空值。
        _ => HostValue::None,
    }
}

/// binding → 文案：i18n key 走 `sget`，host path 走白名单 + 可选 `sgetf` 格式化。
fn binding_text(binding: Option<&BindingSpec>, host: &dyn SceneHost) -> String {
    match binding {
        None => String::new(),
        Some(BindingSpec::I18n(key)) => crate::i18n::sget(key).to_owned(),
        Some(BindingSpec::Host { path, fmt, .. }) => {
            let value = host_value(host, path).text();
            if fmt.is_empty() { value } else { crate::i18n::sgetf(fmt, &[&value]) }
        }
    }
}

/// v6 布局 JSON 的轻量 ref 解析（dot 路径 + `[n]` / `[*]` 下标）。
///
/// 与 `world_schema_check.py` / doccheck2 `world_ref_exists` 同口径；
/// `[*]` 取首项作代表（schema 模板元素 `tiles[*]` 用）。
fn ref_value<'a>(root: &'a serde_json::Value, reference: &str) -> Option<&'a serde_json::Value> {
    let mut node = root;
    for segment in reference.split('.') {
        let (key, indices) = match segment.split_once('[') {
            Some((k, rest)) => (k, format!("[{rest}")),
            None => (segment, String::new()),
        };
        node = node.get(key)?;
        for index in indices.trim_matches(['[', ']']).split(']').filter(|s| !s.is_empty()) {
            if index == "*" {
                node = node.as_array()?.first()?;
            } else {
                node = node.as_array()?.get(index.parse::<usize>().ok()?)?;
            }
        }
    }
    Some(node)
}

/// v6 布局节点 → 世界矩形（中心 + 宽度；`tiles[n]` 无 `w` 时用根 `tile_r`）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn v6_node_rect(root: &serde_json::Value, node: &serde_json::Value) -> Option<WorldRect> {
    let cx = node.get("cx")?.as_f64()?;
    let cy = node.get("cy")?.as_f64()?;
    let side = node.get("w").and_then(serde_json::Value::as_f64).or_else(|| {
        root.get("tile_r").and_then(serde_json::Value::as_f64).map(|radius| radius * 2.0)
    })?;
    let half = side / 2.0;
    Some(WorldRect {
        x: (cx - half).round() as i32,
        y: (cy - half).round() as i32,
        w: side.round() as i32,
        h: side.round() as i32,
    })
}

/// `main_map_v6_layout_landscape.json` 几何引用 → 矩形（编译期内嵌，每进程解析一次）。
fn v6_geometry(reference: &str) -> Option<WorldRect> {
    static V6: OnceLock<serde_json::Value> = OnceLock::new();
    let root = V6.get_or_init(|| {
        serde_json::from_str(V6_LAYOUT_JSON_LANDSCAPE).unwrap_or(serde_json::Value::Null)
    });
    v6_node_rect(root, ref_value(root, reference)?)
}

/// `scenes.json` 几何引用（`main_map.hud_plaque_l`）→ 矩形（横屏优先）。
fn scenes_geometry(reference: &str) -> Option<WorldRect> {
    let (scene_id, key) = reference.split_once('.')?;
    let scenes = scene_table().ok()?;
    let scene = scenes.get(scene_id)?;
    let rect = scene.extra_for().get(key)?;
    Some(WorldRect { x: rect.x, y: rect.y, w: rect.w, h: rect.h })
}

/// 元素目标矩形：`world_rect` 直接用；`geometry` 走两源白名单，不认第三源。
fn element_rect(component: &WorldComponent) -> Option<WorldRect> {
    if let Some(rect) = component.world_rect {
        return Some(rect);
    }
    let geometry = component.geometry.as_ref()?;
    match geometry.source.as_str() {
        "main_map_v6_layout_landscape.json" => v6_geometry(&geometry.reference),
        "scenes.json" => scenes_geometry(&geometry.reference),
        _ => None,
    }
}

/// 文本档位 → 主题色令牌（S38 palette 迁移；三档 token 均由 `Theme::by_name` 识别）。
fn tier_color_token(tier: &str) -> &'static str {
    match tier {
        "title" => "ui_gold_text",
        "hint" => "ui_muted_gold",
        _ => "ui_cream_text",
    }
}

/// 文本档位 → 渲染档位。
fn tier_kind(tier: &str) -> DrawKind {
    match tier {
        "title" => DrawKind::Text(TextTier::Title),
        "hint" => DrawKind::Text(TextTier::Hint),
        _ => DrawKind::Text(TextTier::Body),
    }
}

/// 中间层：`z` 升序 + `state`/`visible_when` 过滤 + `geometry`/`binding`/`role`/`action` 解析。
///
/// 结果可直接交 [`dispatch_draw_cmds`]（CPU / 绘制列表两端共用），也可由其它
/// 后端自行消费；函数签名只含 schema 类型、`&dyn SceneHost`、`&AssetRoles`、`&Theme`，
/// 零业务类型。
#[must_use]
pub fn resolve_world_draw_cmds(
    components: &[WorldComponent],
    roles: &AssetRoles,
    host: &dyn SceneHost,
    theme: &Theme,
    selected: i64,
) -> Vec<ResolvedDrawCmd> {
    let mut ordered: Vec<&WorldComponent> = components.iter().collect();
    ordered.sort_by_key(|component| component.z); // 稳定排序：同 z 保持声明顺序
    let mut commands = Vec::with_capacity(ordered.len());
    for component in ordered {
        if !component_visible(component, selected) {
            continue;
        }
        if component
            .visible_when
            .as_ref()
            .is_some_and(|condition| !visible_when_matches(condition, host))
        {
            continue;
        }
        let Some(rect) = element_rect(component).map(DrawRect::from) else {
            continue;
        };
        let action_key = component
            .pick
            .as_ref()
            .and_then(|pick| UiAction::from_schema_str(&pick.action))
            .map(UiAction::key);
        match component.kind.as_str() {
            "Rect" => {
                let token = if theme.by_name(&component.role).is_some() {
                    component.role.clone()
                } else {
                    "panel_wood".to_owned()
                };
                commands.push(ResolvedDrawCmd {
                    id: component.id.clone(),
                    kind: DrawKind::Rect,
                    rect,
                    color_token: token,
                    text: String::new(),
                    asset: None,
                    action_key,
                });
            }
            "Text" => {
                let text = binding_text(component.binding.as_ref(), host);
                if text.is_empty() {
                    continue;
                }
                commands.push(ResolvedDrawCmd {
                    id: component.id.clone(),
                    kind: tier_kind(&component.tier),
                    rect,
                    color_token: tier_color_token(&component.tier).to_owned(),
                    text,
                    asset: None,
                    action_key,
                });
            }
            "Image" => {
                let Some(asset) = roles.path(&component.role).map(str::to_owned) else {
                    continue;
                };
                commands.push(ResolvedDrawCmd {
                    id: component.id.clone(),
                    kind: DrawKind::Image,
                    rect,
                    color_token: String::new(),
                    text: String::new(),
                    asset: Some(asset),
                    action_key,
                });
                // 按钮等 Image 元素可声明 i18n binding 作文字标签：同 z 紧接着画一行提示档文本。
                let label = binding_text(component.binding.as_ref(), host);
                if !label.is_empty() {
                    commands.push(ResolvedDrawCmd {
                        id: format!("{}_label", component.id),
                        kind: DrawKind::Text(TextTier::Hint),
                        rect,
                        color_token: "ui_muted_gold".to_owned(),
                        text: label,
                        asset: None,
                        action_key: None,
                    });
                }
            }
            _ => {} // 未知 type = 跳过（不 panic、不画占位）
        }
    }
    commands
}

/// 薄 Canvas 适配：token → Theme 色值 + `kind → Canvas` 原语分派。
///
/// 只依赖后端无关类型；调用方传入的 `Canvas` 可以是 `DrawList`（Bevy）或
/// CPU 帧缓冲，命令序列逐 op 等价。
pub fn dispatch_draw_cmds(canvas: &mut dyn Canvas, commands: &[ResolvedDrawCmd], theme: &Theme) {
    for command in commands {
        let DrawRect { x, y, w, h } = command.rect;
        match command.kind {
            DrawKind::Rect | DrawKind::Text(_) => {
                let Some(color) = theme.by_name(&command.color_token) else {
                    continue;
                };
                match command.kind {
                    DrawKind::Rect => {
                        canvas.fill_rect(x, y, w, h, color);
                    }
                    DrawKind::Text(tier) => {
                        let line_height = match tier {
                            TextTier::Title => canvas.title_glyph_height(),
                            TextTier::Body => canvas.glyph_height(),
                            TextTier::Hint => canvas.hint_glyph_height(),
                        };
                        let text_y = y + (h - line_height).max(0) / 2;
                        match tier {
                            TextTier::Title => {
                                canvas.draw_title_text(&command.text, x, text_y, color);
                            }
                            TextTier::Body => {
                                canvas.draw_text(&command.text, x, text_y, color);
                            }
                            TextTier::Hint => {
                                canvas.draw_hint_text(&command.text, x, text_y, color);
                            }
                        }
                    }
                    DrawKind::Image => {}
                }
            }
            DrawKind::Image => {
                let Some(asset) = command.asset.as_deref() else {
                    continue;
                };
                if !chrome::asset_present(asset) {
                    continue;
                }
                canvas.blit_image_tint(asset, x, y, w, h, TINT_IDENTITY);
            }
        }
    }
}

/// `dyn SceneHost` → `dyn Canvas` 桥接体（镜像 `chrome::HostCanvas`）。
///
/// 每个方法**必须转发 `RenderHost` 方法**（而非直接写帧缓冲）：记录型宿主
/// （`record_draw=true` → `DrawList`）才能逐 op 收到调用，CPU / Bevy 两端等价。
struct SceneHostCanvas<'a> {
    host: &'a mut dyn SceneHost,
}

impl Canvas for SceneHostCanvas<'_> {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        <dyn SceneHost as RenderHost>::fill_rect(self.host, x, y, w, h, color);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        <dyn SceneHost as RenderHost>::draw_text_colored(self.host, text, x, y, color);
    }

    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        <dyn SceneHost as RenderHost>::draw_title_text(self.host, text, x, y, color);
    }

    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        <dyn SceneHost as RenderHost>::draw_hint_text(self.host, text, x, y, color);
    }

    fn text_width(&self, text: &str) -> i32 {
        self.host.shell().text_width_px(text)
    }

    fn title_text_width(&self, text: &str) -> i32 {
        self.host.shell().title_text_width_px(text)
    }

    fn hint_text_width(&self, text: &str) -> i32 {
        self.host.shell().hint_text_width_px(text)
    }

    fn glyph_height(&self) -> i32 {
        self.host.shell().glyph_px()
    }

    fn title_glyph_height(&self) -> i32 {
        self.host.shell().title_glyph_px()
    }

    fn hint_glyph_height(&self) -> i32 {
        self.host.shell().hint_glyph_px()
    }

    fn clear_frame(&mut self, color: u16) {
        <dyn SceneHost as RenderHost>::clear_frame(self.host, color);
    }

    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        <dyn SceneHost as RenderHost>::pixel_at(self.host, x, y)
    }

    fn blit_image_tint(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32, tint: u16) {
        <dyn SceneHost as RenderHost>::blit_image_tint(self.host, asset, x, y, w, h, tint);
    }
}

/// 世界层 schema 渲染入口（S87.3/4 在 env 分支里调用本函数）。
///
/// 返回 `false` = 场景未声明 / 元素为空 / 数据表加载失败 ⇒ 调用方**回退旧路径**；
/// 返回 `true` = 已消费该场景全部静态元素（过程式动画仍由 Rust 画在其后）。
///
/// 零游戏语义：只经 [`Canvas`] 原语绘制，不挂 `chrome_in_bevy_ui` 守卫。
pub fn render_world_components(host: &mut dyn SceneHost, scene_id: &str, selected: i64) -> bool {
    let Ok(scenes) = world_components() else {
        return false;
    };
    let Some(scene) = scenes.get(scene_id) else {
        return false;
    };
    if scene.elements.is_empty() {
        return false;
    }
    let Ok(roles) = asset_roles() else {
        return false;
    };
    // 先克隆主题，避免与后续 `&mut host` 借用重叠。
    let theme = host.theme().clone();
    let commands = resolve_world_draw_cmds(&scene.elements, roles, host, &theme, selected);
    let mut canvas = SceneHostCanvas { host };
    dispatch_draw_cmds(&mut canvas, &commands, &theme);
    true
}
