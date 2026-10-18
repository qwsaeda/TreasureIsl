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

use crate::scenes::{RenderHost, SceneHost};
use ti_core::data::{IconDef, ui_icons};

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
    let _ = PRESS_FLASH_MS_LEFT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(v.saturating_sub(delta_ms)));
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

/// 等距投影：`(row, col)` → 呈现层坐标。
#[must_use]
pub fn iso_cell(row: i32, col: i32, layout: &data::SceneLayout) -> (i32, i32) {
    let s = &layout.extra_scalar;
    let ox = s.get("grid.x0").copied().unwrap_or(0);
    let oy = s.get("grid.y0").copied().unwrap_or(0);
    let tw = s.get("grid.tile_w").copied().unwrap_or(0);
    let th = s.get("grid.tile_h").copied().unwrap_or(0);
    (ox + (col - row) * tw, oy + (row + col) * th)
}

/// 等距 tile 半宽/半高。
#[must_use]
pub fn iso_tile(layout: &data::SceneLayout) -> (i32, i32) {
    let s = &layout.extra_scalar;
    (s.get("grid.tile_w").copied().unwrap_or(0), s.get("grid.tile_h").copied().unwrap_or(0))
}

/// **场景外壳**：深色底 + 航海网格装饰 + 卡片边框 + 标题条 + 标题文字。
pub fn scene_shell(host: &mut dyn SceneHost, scene_id: &str, title_key: &str) {
    let layout = scene_layout(scene_id);
    parchment_bg(host);
    nautical_grid(host);
    wood_frame(host);
    title_bar(host, layout.title.h);
    let title = crate::i18n::sget(title_key);
    let cx = centered_title_x(host, title);
    let color = host.theme().text_on_wood;
    draw_title_text(host, title, cx, layout.title.y, color);
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
    let _ = PAGE_MS_LEFT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(v.saturating_sub(delta_ms)));
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
    let Some(art) = layout.extra.get("ambient_art") else {
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
    let Some(art) = layout.extra.get("title_art") else {
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
    let Some(art) = layout.extra.get("battle_arena") else {
        return;
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::battle_arena(c, &theme, art.x, art.y, art.w, art.h));
}
