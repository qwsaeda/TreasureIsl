//! S15 UI 重设计：羊皮纸/木纹/金色系**共享视觉外壳**（7 场景复用）。
//!
//! 绘制函数已抽取到 [`ti_render::shell`] / [`ti_render::primitives`] / [`ti_render::iso`]，
//! 本模块保留：
//! - 状态管理（翻页/按下闪烁动画帧计数）
//! - 场景外壳组合函数（`scene_shell` / `centered_title_x` 等）
//! - 颜色/字号常量重导出（从 [`ti_render::Theme`] 默认值，兼容旧导入路径）
//!
//! 坐标系：1080×1920 呈现层像素（`menu_shell::LOGICAL_W_I32`/`LOGICAL_H_I32`）。

use std::sync::atomic::{AtomicU32, Ordering};
use ti_core::data;

use ti_render::framebuffer::LOGICAL_W_I32;
use ti_render::theme::Theme;
// Canvas trait：文本宽度度量（`title_text_width`/`text_width`）在 trait 上，需在作用域内。

use crate::scenes::{RenderHost, SceneHost};
use ti_core::data::{IconDef, Primitive, icon_ids, ui_icons};

// === 颜色常量重导出（从 Theme 默认值，兼容旧导入路径） ===

/// 深底上的**白文字**色。
pub const COLOR_TEXT_ON_PARCH: u16 = Theme::DEFAULT.text_on_parch;
/// 标题条上的**亮文字**色（象牙金）。
pub const COLOR_TEXT_ON_WOOD: u16 = Theme::DEFAULT.text_on_wood;
/// 危险/BOSS 标记色。
pub const COLOR_DANGER: u16 = Theme::DEFAULT.danger;
/// 玩家棋子/单位色。
pub const COLOR_PLAYER: u16 = Theme::DEFAULT.player;
/// 据点标记色。
pub const COLOR_LANDMARK: u16 = Theme::DEFAULT.landmark;
/// 背景填充色。
pub const COLOR_BG: u16 = Theme::DEFAULT.bg;
/// 标题条主色。
pub const COLOR_TITLE_BAR: u16 = Theme::DEFAULT.title_bar;
/// 选中高亮色。
pub const COLOR_SELECTED: u16 = Theme::DEFAULT.selected;
/// 次文字/提示行色。
pub const COLOR_SECONDARY: u16 = Theme::DEFAULT.secondary;
/// 卡片亮（bevel 凸起）。
pub const COLOR_BEVEL_HI: u16 = Theme::DEFAULT.bevel_hi;
/// 暗角（bevel 凹陷）。
pub const COLOR_BEVEL_SHADOW: u16 = Theme::DEFAULT.bevel_shadow;
/// 卡片底（菜单项色块）。
pub const COLOR_MENU_BG: u16 = Theme::DEFAULT.menu_bg;
/// 卡片描边（菜单项边框）。
pub const COLOR_MENU_BORDER: u16 = Theme::DEFAULT.menu_border;
/// 进度条底。
pub const COLOR_HILITE: u16 = Theme::DEFAULT.hilite;
/// 深底。
pub const COLOR_PARCH_DARK: u16 = Theme::DEFAULT.parch_dark;
/// 深底亮。
pub const COLOR_PARCH_LIGHT: u16 = Theme::DEFAULT.parch_light;
/// 卡片描边/标题条。
pub const COLOR_WOOD: u16 = Theme::DEFAULT.wood;
/// 金高光。
pub const COLOR_GOLD: u16 = Theme::DEFAULT.gold;
/// 航海蓝亮。
pub const COLOR_GOLD_GLOW: u16 = Theme::DEFAULT.gold_glow;
/// 航海蓝。
pub const COLOR_GOLD_BRIGHT: u16 = Theme::DEFAULT.gold_bright;
/// 清屏黑。
pub const COLOR_BLACK: u16 = Theme::DEFAULT.black;
/// S18.6 暖木色（Agnes 风格木框；暖色系令牌）。
pub const COLOR_WOOD_WARM: u16 = Theme::DEFAULT.wood_warm;
/// S18.6 暖金（Agnes 风格标题金；暖色系令牌）。
pub const COLOR_GOLD_WARM: u16 = Theme::DEFAULT.gold_warm;
/// S18.6 夜空深蓝（Agnes 风格夜空渐变底色）。
pub const COLOR_SKY_NAVY: u16 = Theme::DEFAULT.sky_navy;

// === 动画状态管理 ===
//
// S17.4 动效参数表（数值源 = `docs/process/10_ui_redesign.md` §7；doccheck2
// `check_ui_redesign_tokens` 断言本表常量 == 文档表）：全部**时间制（ms）**，
// 帧率无关（旧实现按帧计数，30fps 下动效拖长一倍）。

/// 场景切换交叉淡化时长（ms，ease-in-out）。
pub const MOTION_TRANSITION_MS: u32 = 240;
/// 按压闪光时长（ms，线性衰减）。
pub const MOTION_PRESS_FLASH_MS: u32 = 120;
/// 血条数值插值时长（ms，线性趋近目标）。
pub const MOTION_HP_LERP_MS: u32 = 160;
/// 翻页亮度渐入时长（ms，线性）。
pub const MOTION_PAGE_MS: u32 = 120;

static PAGE_MS_LEFT: AtomicU32 = AtomicU32::new(0);
static PRESS_FLASH_MS_LEFT: AtomicU32 = AtomicU32::new(0);

/// 触发按下闪烁（持续 [`MOTION_PRESS_FLASH_MS`]）。
pub fn flash_press() {
    PRESS_FLASH_MS_LEFT.store(MOTION_PRESS_FLASH_MS, Ordering::Relaxed);
}

/// 每帧推进按下闪烁倒计时（`delta_ms` = 本帧经过毫秒）。
pub fn tick_press_flash(delta_ms: u32) {
    let mut cur = PRESS_FLASH_MS_LEFT.load(Ordering::Relaxed);
    while let Err(got) = PRESS_FLASH_MS_LEFT.compare_exchange_weak(
        cur,
        cur.saturating_sub(delta_ms),
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        cur = got;
    }
}

/// 按下闪烁是否处于可见期。
#[must_use]
pub fn press_flash_active() -> bool {
    PRESS_FLASH_MS_LEFT.load(Ordering::Relaxed) > 0
}

/// 重置翻页动画倒计时（起始为最暗）。
pub fn reset_page_frame() {
    PAGE_MS_LEFT.store(MOTION_PAGE_MS, Ordering::Relaxed);
}

/// 重置全部动画状态。
pub fn reset_animation_state() {
    PAGE_MS_LEFT.store(0, Ordering::Relaxed);
    PRESS_FLASH_MS_LEFT.store(0, Ordering::Relaxed);
}

/// 读某场景的版面数据。
#[must_use]
pub fn scene_layout(scene_id: &str) -> data::SceneLayout {
    data::scenes().ok().and_then(|s| s.get(scene_id)).cloned().unwrap_or_default()
}

/// scenes.json ↔ 代码的**版面键契约**（`main_map` 棋盘 / v3 世界画布簇）。
///
/// 单一事实源：`extra`（矩形键）与 `extra_scalar`（点号标量键，见
/// [`ti_core::data`] 的 `format!("{k}.{fk}")` 扁平化）读站点一律引用本模块 const，
/// **勿再写裸字面量**；改 scenes.json 键名只改这里一处。
pub mod keys {
    /// v2 静态棋盘原点 x（`extra_scalar` 点号键 = scenes.json `main_map.grid.x0`）。
    pub const GRID_X0: &str = "grid.x0";
    /// v2 静态棋盘原点 y（= `main_map.grid.y0`）。
    pub const GRID_Y0: &str = "grid.y0";
    /// v2 静态棋盘 tile 半宽（= `main_map.grid.tile_w`）。
    pub const GRID_TILE_W: &str = "grid.tile_w";
    /// v2 静态棋盘 tile 半高（= `main_map.grid.tile_h`）。
    pub const GRID_TILE_H: &str = "grid.tile_h";
    /// v3 世界画布矩形键（extra = `main_map.world_canvas`）。
    pub const WORLD_CANVAS: &str = "world_canvas";
    /// v3 HUD 顶左金币角矩形键（extra = `main_map.hud_gold`）。
    pub const HUD_GOLD: &str = "hud_gold";
    /// v3 HUD 顶右持有量角矩形键（extra = `main_map.hud_player`）。
    pub const HUD_PLAYER: &str = "hud_player";
    /// v3 底部命令盘「贸易」键（extra = `main_map.deck_trade`）。
    pub const DECK_TRADE: &str = "deck_trade";
    /// v3 底部命令盘「出售」键（extra = `main_map.deck_sell`）。
    pub const DECK_SELL: &str = "deck_sell";
    /// v3 底部命令盘「建造」键（extra = `main_map.deck_build`）。
    pub const DECK_BUILD: &str = "deck_build";
    /// v2 HUD 经济按钮「贸易」矩形键（extra = `main_map.hud_trade`）。
    pub const HUD_TRADE: &str = "hud_trade";
    /// v2 HUD 经济按钮「出售」矩形键（extra = `main_map.hud_sell`）。
    pub const HUD_SELL: &str = "hud_sell";
    /// v2 HUD 经济按钮「建造」矩形键（extra = `main_map.hud_build`）。
    pub const HUD_BUILD: &str = "hud_build";
    /// v3 实体百分比簇前缀（`extra_scalar` = `main_map.canvas_entities.<key>`）。
    pub const CANVAS_ENTITIES: &str = "canvas_entities";
    /// 实体 v2 size 回退键后缀（`extra_scalar` = `<key>.size`）。
    pub const SIZE_SUFFIX: &str = "size";
    // ── v3 场景叙事装饰层（index/vs；各场景一段，键缺失/非正 = 不画） ──
    /// v3 氛围星空层矩形键（extra = 场景 `ambient_art`）。
    pub const AMBIENT_ART: &str = "ambient_art";
    /// v3 仪式感标题艺术面板矩形键（extra = 场景 `title_art`）。
    pub const TITLE_ART: &str = "title_art";
    /// v3 战斗竞技区装饰矩形键（extra = `vs.battle_arena`）。
    pub const BATTLE_ARENA: &str = "battle_arena";
}

/// 等距投影：`(row, col)` → 呈现层坐标。
#[must_use]
pub fn iso_cell(row: i32, col: i32, layout: &data::SceneLayout) -> (i32, i32) {
    let s = &layout.extra_scalar;
    let ox = s.get(keys::GRID_X0).copied().unwrap_or(0);
    let oy = s.get(keys::GRID_Y0).copied().unwrap_or(0);
    let tw = s.get(keys::GRID_TILE_W).copied().unwrap_or(0);
    let th = s.get(keys::GRID_TILE_H).copied().unwrap_or(0);
    (ox + (col - row) * tw, oy + (row + col) * th)
}

/// 等距 tile 半宽/半高。
#[must_use]
pub fn iso_tile(layout: &data::SceneLayout) -> (i32, i32) {
    let s = &layout.extra_scalar;
    (s.get(keys::GRID_TILE_W).copied().unwrap_or(0), s.get(keys::GRID_TILE_H).copied().unwrap_or(0))
}

// ── v3 场景叙事 · 世界画布棋盘几何（main_map S17.8；G 依赖运行时拟合） ──────

/// v3 世界画布棋盘几何：`world_canvas` 键在位 → 把 `g×g` 等距棋盘（2:1 菱形）
/// **拟合进画布**（宽 `2*g*tw ≤ 画布宽`、`tw` 8pt 向下取整、画布中心对齐）；
/// 缺失/非正 → 回退 v2 静态 `grid.*`（双写：v3 键缺席即 v2 版面不变）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardGeom {
    /// 棋盘菱形左上参考（`iso_cell_fit(0,0)` 原点）。
    pub x0: i32,
    /// 棋盘顶行 y。
    pub y0: i32,
    /// tile 半宽（v3 拟合 / v2 静态）。
    pub tw: i32,
    /// tile 半高（v3 = tw/2 / v2 静态）。
    pub th: i32,
}

/// 场景是否启用 v3 世界画布（`world_canvas` 键 + 正尺寸）。
#[must_use]
pub fn has_world_canvas(layout: &data::SceneLayout) -> bool {
    layout.extra.get(keys::WORLD_CANVAS).is_some_and(|r| r.w > 0 && r.h > 0)
}

/// 棋盘几何（v3 拟合 / v2 回退，见 [`BoardGeom`] 文档）。
///
/// v3：`tw = (画布宽 / (2*g)) / 8 * 8`（≥8，否则回退 v2），`th = tw/2`，
/// 画布中心 `cx/cy` → `x0 = cx - tw`、`y0 = cy - g*th`（菱形中心对齐画布中心）。
#[must_use]
pub fn board_fit(layout: &data::SceneLayout, g: u32) -> BoardGeom {
    let gi = i32::try_from(g).unwrap_or(0);
    if let Some(c) = layout.extra.get(keys::WORLD_CANVAS) {
        if c.w > 0 && c.h > 0 && gi > 0 {
            let tw = c.w / (2 * gi);
            let tw = (tw / 8) * 8;
            if tw >= 8 {
                let th = tw / 2;
                let cx = c.x + c.w / 2;
                let cy = c.y + c.h / 2;
                return BoardGeom { x0: cx - tw, y0: cy - gi * th, tw, th };
            }
        }
    }
    let s = &layout.extra_scalar;
    BoardGeom {
        x0: s.get(keys::GRID_X0).copied().unwrap_or(0),
        y0: s.get(keys::GRID_Y0).copied().unwrap_or(0),
        tw: s.get(keys::GRID_TILE_W).copied().unwrap_or(0),
        th: s.get(keys::GRID_TILE_H).copied().unwrap_or(0),
    }
}

/// v3 等距投影（G 依赖 `board_fit`）：`(row, col)` → 呈现层坐标。
#[must_use]
pub fn iso_cell_fit(row: i32, col: i32, layout: &data::SceneLayout, g: u32) -> (i32, i32) {
    let bg = board_fit(layout, g);
    (bg.x0 + (col - row) * bg.tw, bg.y0 + (row + col) * bg.th)
}

/// v3 tile 半宽/半高（G 依赖 `board_fit`）。
#[must_use]
pub fn iso_tile_fit(layout: &data::SceneLayout, g: u32) -> (i32, i32) {
    let bg = board_fit(layout, g);
    (bg.tw, bg.th)
}

/// v3 实体边长：`canvas_entities.<key>`（`board_fit.tw` 的百分之几）× tw；
/// 缺失/非正 → 回退 v2 `<key>.size`（QA106 语义，`tw/th` 与实体同数据源不越界）。
#[must_use]
pub fn entity_size_fit(layout: &data::SceneLayout, key: &str, g: u32) -> i32 {
    let s = &layout.extra_scalar;
    let ratio = s.get(&format!("{}.{}", keys::CANVAS_ENTITIES, key)).copied().unwrap_or(0);
    if ratio > 0 {
        let bg = board_fit(layout, g);
        return (bg.tw * ratio / 100).max(4);
    }
    s.get(&format!("{key}.{}", keys::SIZE_SUFFIX)).copied().unwrap_or(0)
}

/// v3 世界画布装饰框（`main_map`）：读 `world_canvas` 矩形，缺失/非正 = 不画。
///
/// 调用序（z-order）：`nautical_grid` 之后、等距瓦片之前（垫底框 + 四角罗盘方位块）。
pub fn world_canvas_frame(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(rc) = layout.extra.get(keys::WORLD_CANVAS) else {
        return;
    };
    if rc.w <= 0 || rc.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::world_canvas_frame(c, &theme, rc.x, rc.y, rc.w, rc.h));
}

/// v3 底部命令盘按钮（`main_map`）：读 `<key>` 矩形（`deck_trade`/`deck_sell`/`deck_build`），
/// 面板 + 图标（经 `icon_key` 查 `ui_icons` 注册表，如 `hud_trade`）+ `28px` 标签；缺失/非正 = 不画。
///
/// 图标宽度不足时自动退回纯文字居中（与 `render_hud_button` 同判据）。
pub fn deck_button(
    host: &mut dyn SceneHost,
    scene_id: &str,
    key: &str,
    icon_key: &str,
    label_key: &str,
) {
    let layout = scene_layout(scene_id);
    let Some(r) = layout.extra.get(key) else {
        return;
    };
    if r.w <= 0 || r.h <= 0 {
        return;
    }
    panel(host, r.x, r.y, r.w, r.h);
    let label = crate::i18n::sget(label_key);
    let icon = hud_icon_def(icon_key);
    let (label_x, icon_x) = match icon.as_ref() {
        Some(def) if r.w >= def.size + 8 => {
            let ix = r.x + 8;
            (ix + def.size + 8, Some((def.id, def.size, ix)))
        }
        _ => (r.x + (r.w - host.shell().hint_text_width_px(label)) / 2, None),
    };
    if let Some((id, size, ix)) = icon_x {
        ui_icon(host, id, ix, r.y + (r.h - size) / 2, false);
    }
    let ly = r.y + (r.h - host.shell().hint_glyph_px()) / 2;
    host.draw_hint_text(label, label_x, ly, COLOR_TEXT_ON_PARCH);
}

/// **场景外壳**：深色底 + 航海网格装饰 + 卡片边框 + 标题条 + 标题文字。
pub fn scene_shell(host: &mut dyn SceneHost, scene_id: &str, title_key: &str) {
    let layout = scene_layout(scene_id);
    scene_base(host);
    scene_frame(host);
    title_bar(host, layout.title.h);
    let title = crate::i18n::sget(title_key);
    let cx = centered_title_x(host, title);
    let color = host.theme().text_on_wood;
    draw_title_text(host, title, cx, layout.title.y, color);
}

/// 场景外壳**底色层**（纯色 + 垂直渐变 + 暗角）——可被背景类原语覆盖
/// （S18.6 声明式原语：背景类原语在底色之上、外框之下）。
pub fn scene_base(host: &mut dyn SceneHost) {
    parchment_bg(host);
}

/// 场景外壳**外框层**（航海网格 + 木外框）——在背景类原语之上。
pub fn scene_frame(host: &mut dyn SceneHost) {
    nautical_grid(host);
    wood_frame(host);
}

/// 96px 标题屏心居中的左缘 x。
#[must_use]
pub fn centered_title_x(host: &mut dyn SceneHost, title: &str) -> i32 {
    (LOGICAL_W_I32 - host.shell().title_text_width_px(title)) / 2
}

/// 44px 正文屏心居中的左缘 x。
#[must_use]
pub fn centered_body_x(host: &mut dyn SceneHost, text: &str) -> i32 {
    (LOGICAL_W_I32 - host.shell().text_width_px(text)) / 2
}

/// **场景提示行**。
pub fn scene_hint(host: &mut dyn SceneHost, scene_id: &str, hint_key: &str) {
    let layout = scene_layout(scene_id);
    let text = crate::i18n::sget(hint_key);
    hint_row(host, text, LOGICAL_W_I32 / 2, layout.hint.y);
}

/// 推进翻页动画倒计时（`delta_ms` = 本帧经过毫秒）。
pub fn tick_page_frame(delta_ms: u32) {
    let mut cur = PAGE_MS_LEFT.load(Ordering::Relaxed);
    while let Err(got) = PAGE_MS_LEFT.compare_exchange_weak(
        cur,
        cur.saturating_sub(delta_ms),
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        cur = got;
    }
}

/// 获取当前翻页动画亮度因子（0.6 → 1.0 线性，时长 [`MOTION_PAGE_MS`]；
/// 倒计时耗尽 = 1.0 稳态）。
#[must_use]
pub fn page_brightness_factor() -> f32 {
    let left = PAGE_MS_LEFT.load(Ordering::Relaxed);
    if left >= MOTION_PAGE_MS {
        return 0.6;
    }
    let full = f32::from(u16::try_from(MOTION_PAGE_MS).unwrap_or(0));
    let done = f32::from(u16::try_from(MOTION_PAGE_MS - left).unwrap_or(0)) / full;
    0.6 + done * 0.4
}

// === 兼容包装函数（旧签名，内部调用 ti-render） ===
//
// S15.7：绘制类 wrapper 一律只收 `&mut MenuShell`（它已 `impl Canvas`）+ 从
// `shell.theme` 内部取主题。这样做有 3 个好处：
//   1. 渲染框架只见 `Canvas`，不见 `SceneHost`/场景/宿主概念 → 框架可独立复用；
//   2. 调用点写 `host.shell_mut()`，避免「`host` 已可变借用 + `host.theme()`
//      还要不可变借用」的 E0502 冲突；
//   3. 无需 trait-object 上转（不依赖 1.86 trait upcasting）。

/// S15.7：把宿主的 [`RenderHost`] 像素面桥接为渲染框架的 [`ti_render::canvas::Canvas`]。
///
/// `ti_render::shell` / `primitives` 只认 `&mut dyn Canvas`，游戏侧统一持 `&mut dyn
/// SceneHost`。本适配器实现 `Canvas` 并把**像素操作**转发到宿主的 `RenderHost` 同名方法
/// （`fill_rect`/`draw_*`/`clear_frame`/`pixel_at`）——**不经由具体 `MenuShell`**——使
/// 测试宿主（`render_intent::VsHost`）能拦截并记录绘制；宽度/字形高是只读几何，走
/// `shell()` 读。渲染框架因此依旧不感知游戏场景概念，且与具体帧缓冲解耦（`PixelSink`）。
struct HostCanvas<'a> {
    host: &'a mut dyn SceneHost,
}

impl ti_render::canvas::Canvas for HostCanvas<'_> {
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
}

/// 在 `host` 上构造本地 [`HostCanvas`] 并交给 `f`（消「返回指向临时值的引用」——
/// 桥接体作为函数局部持有，`&mut` 借用随 `f` 作用域结束，不跨函数返回）。
fn with_canvas(host: &mut dyn SceneHost, f: impl FnOnce(&mut HostCanvas<'_>)) {
    let mut bridge = HostCanvas { host };
    f(&mut bridge);
}

/// 背景底：整帧纯色 + 垂直渐变 + 四边暗角。
pub fn parchment_bg(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::primitives::parchment_bg(c, &theme));
}

/// 整屏外框。
pub fn wood_frame(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::wood_frame(c, &theme));
}

/// 标题条。
pub fn title_bar(host: &mut dyn SceneHost, title_h: i32) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::title_bar(c, &theme, title_h));
}

/// 航海图装饰层。
pub fn nautical_grid(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::nautical_grid(c, &theme));
}

/// S18.7 声明式原语 dispatch（**CPU 执行器**）：`scenes.json` 的 `primitives` 表 → 画布绘制。
///
/// 与 Bevy 执行器（`ti_engine::scene_render`）消费**同一份描述** → 双后端同构
/// （同一 `scenes.json` 出同一画面）。**通用原语只有 6 种**（`rect`/`text`/`icon`/
/// `grad`/`glow`/`disc`），渲染层**不认识**"菜单""木框"等场景语义 —— 场景语义 = 数据。
pub fn render_primitives(host: &mut dyn SceneHost, scene_id: &str, selected: i64) {
    let layout = scene_layout(scene_id);
    let theme = host.theme().clone();
    // 层序：z 升序 = 越大越靠上（调用序即层序，`08_render_arch.md` 契约 1）。
    let mut prims: Vec<&Primitive> = layout.primitives.iter().collect();
    prims.sort_by_key(|p| p.z);
    for p in prims {
        if !primitive_visible(p, selected) {
            continue;
        }
        let color = theme.by_name(&p.color).unwrap_or(theme.text_on_parch);
        let r = p.rect;
        with_canvas(host, |c| match p.kind.as_str() {
            "rect" => ti_render::generic::rect(c, r.x, r.y, r.w, r.h, color),
            "text" => {
                let content = if p.text_key.is_empty() {
                    p.text.clone()
                } else {
                    crate::i18n::sget(&p.text_key).to_owned()
                };
                ti_render::generic::text(c, &content, r.x, r.y, r.w, r.h, &p.tier, color);
            }
            "icon" => {
                let kind = param_str(p, "icon")
                    .and_then(|name| icon_ids().get(name.as_str()).copied())
                    .unwrap_or(0);
                let size = i32::try_from(param_i64(p, "size").unwrap_or(48)).unwrap_or(48);
                ti_render::generic::icon(c, kind, r.x, r.y, size, color);
            }
            "grad" => {
                let from =
                    param_str(p, "from_token").and_then(|n| theme.by_name(&n)).unwrap_or(theme.bg);
                let to =
                    param_str(p, "to_token").and_then(|n| theme.by_name(&n)).unwrap_or(theme.bg);
                let steps = i32::try_from(param_i64(p, "steps").unwrap_or(32)).unwrap_or(32);
                ti_render::generic::grad(c, r.x, r.y, r.w, r.h, from, to, steps);
            }
            "glow" => ti_render::generic::glow(c, r.x, r.y, r.w, color),
            "disc" => ti_render::generic::disc(c, r.x, r.y, r.w, color),
            _ => {}
        });
    }
}

/// 原语可见性（`when` = `selected`/`normal`；`item` = 菜单项序号）。
fn primitive_visible(p: &Primitive, selected: i64) -> bool {
    match p.when.as_str() {
        "selected" => p.item == selected,
        "normal" => p.item != selected,
        _ => true,
    }
}

/// 原语参数（字符串）。
fn param_str(p: &Primitive, key: &str) -> Option<String> {
    p.params.get(key).and_then(|v| v.as_str()).map(str::to_owned)
}

/// 原语参数（数值）。
fn param_i64(p: &Primitive, key: &str) -> Option<i64> {
    p.params.get(key).and_then(serde_json::Value::as_i64)
}

/// 菜单 icon（兼容出口：委托 `ti_render::icons`，kind 即图标 id）。
pub fn menu_icon(host: &mut dyn SceneHost, kind: usize, x: i32, y: i32) {
    let theme = host.theme().clone();
    let kind_u32 = u32::try_from(kind).unwrap_or(0);
    with_canvas(host, |c| ti_render::shell::menu_icon(c, &theme, kind_u32, x, y));
}

/// S17.3 图标（注册表驱动）：`id` = `assets/data/ui_icons.json` 的 `icons[].id`，
/// `selected` 决定 tint（默认 `theme.gold` / 选中 `theme.text_on_wood`——
/// 选中底是航海蓝，白墨对比更高）。
pub fn ui_icon(host: &mut dyn SceneHost, id: u32, x: i32, y: i32, selected: bool) {
    let theme = host.theme().clone();
    let tint = if selected { theme.text_on_wood } else { theme.gold };
    with_canvas(host, |c| ti_render::icons::icon(c, &theme, id, x, y, tint));
}

/// 图标元数据取用（`ui_icons.json` 解析失败 = 不画图标，不 panic）。
#[must_use]
pub fn icon_def(name: &str) -> Option<IconDef> {
    ui_icons().ok().and_then(|t| t.icon(name)).cloned()
}

/// 注册表按 id 取用（主菜单等按序号取图标的场景）。
#[must_use]
pub fn index_menu_icon_def(menu_index: usize) -> Option<IconDef> {
    ui_icons().ok().and_then(|t| t.index_menu_icon(menu_index)).cloned()
}

/// 注册表按 HUD 键取用。
#[must_use]
pub fn hud_icon_def(hud_key: &str) -> Option<IconDef> {
    ui_icons().ok().and_then(|t| t.hud_icon(hud_key)).cloned()
}

/// 菜单项。
pub fn menu_item(
    host: &mut dyn SceneHost,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    chosen: bool,
    pressed: bool,
) {
    let theme = host.theme().clone();
    with_canvas(host, |c| {
        ti_render::shell::menu_item(c, &theme, left, top, width, height, chosen, pressed);
    });
}

/// 血条槽位。
pub fn bar_slot(host: &mut dyn SceneHost, x: i32, y: i32, w: i32, h: i32) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::bar_slot(c, &theme, x, y, w, h));
}

/// 内容面板。
pub fn panel(host: &mut dyn SceneHost, x: i32, y: i32, w: i32, h: i32) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::panel(c, &theme, x, y, w, h));
}

/// 标题条内文字。
pub fn draw_title_text(host: &mut dyn SceneHost, text: &str, x: i32, y: i32, color: u16) {
    <dyn SceneHost as RenderHost>::draw_title_text(host, text, x, y, color);
}

/// 正文档文字。
pub fn draw_body_text(host: &mut dyn SceneHost, text: &str, x: i32, y: i32) {
    host.draw_text_colored(text, x, y, COLOR_TEXT_ON_PARCH);
}

/// 次级文字。
pub fn draw_body_text_secondary(host: &mut dyn SceneHost, text: &str, x: i32, y: i32) {
    host.draw_text_colored(text, x, y, COLOR_SECONDARY);
}

/// 底部提示行。QB123：条**屏心居中**（忽略传入 `x`，与老 chrome 一致；`x` 仅为兼容
/// 旧调用签名保留）。
pub fn hint_row(host: &mut dyn SceneHost, text: &str, _x: i32, y: i32) {
    let theme = host.theme().clone();
    let cx = LOGICAL_W_I32 / 2;
    with_canvas(host, |c| ti_render::shell::hint_row(c, &theme, text, cx, y));
}

// === v3 场景叙事装饰层包装（UI 重设计 v3.1，docs/ui-redesign-v3/02 §8） ===
//
// 数据驱动（契约 4）：矩形全读 `scenes.json` 的 `extra` 键；键缺失 / 非正 = 不画
// （不 panic、不占位色块，与 icon 列 / 实体 size 同判据）。

/// v3 氛围星空层：读 `scenes.json` 该场景的 `ambient_art` 矩形，缺失/非正 = 不画。
///
/// z-order（调用序）：`scene_shell`（底/网格/框/标题条）之后、内容（菜单/血条）之前。
pub fn star_sky(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(art) = layout.extra.get(keys::AMBIENT_ART) else {
        return;
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::star_sky(c, &theme, art.x, art.y, art.w, art.h));
}

/// v3 仪式感标题艺术面板：读 `scenes.json` 该场景的 `title_art` 矩形，缺失/非正 = 不画。
///
/// 面板文字（如巨字标题）由调用方按版面叠画（原语零语义）；z-order 在星空层之后。
pub fn title_art_block(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(art) = layout.extra.get(keys::TITLE_ART) else {
        return;
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::title_art_block(c, &theme, art.x, art.y, art.w, art.h));
}

/// v3 战斗竞技区装饰（vs）：读 `scenes.json` 该场景的 `battle_arena` 矩形，缺失/非正 = 不画。
///
/// z-order（调用序）：`scene_shell` 之后、血条/回合文本/提示行之前（装饰垫底，内容叠上）。
pub fn battle_arena(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(art) = layout.extra.get(keys::BATTLE_ARENA) else {
        return;
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::battle_arena(c, &theme, art.x, art.y, art.w, art.h));
}
