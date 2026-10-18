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

static PAGE_FRAME: AtomicU32 = AtomicU32::new(0);
static PRESS_FLASH: AtomicU32 = AtomicU32::new(0);
const PRESS_FLASH_FRAMES: u32 = 4;

/// 触发按下闪烁。
pub fn flash_press() {
    PRESS_FLASH.store(PRESS_FLASH_FRAMES, Ordering::Relaxed);
}

/// 每帧推进按下闪烁倒计时。
pub fn tick_press_flash() {
    let _ = PRESS_FLASH
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(v.saturating_sub(1)));
}

/// 按下闪烁是否处于可见期。
#[must_use]
pub fn press_flash_active() -> bool {
    PRESS_FLASH.load(Ordering::Relaxed) > 0
}

/// 重置翻页动画帧计数。
pub fn reset_page_frame() {
    PAGE_FRAME.store(0, Ordering::Relaxed);
}

/// 重置全部动画帧计数。
pub fn reset_animation_state() {
    PAGE_FRAME.store(0, Ordering::Relaxed);
    PRESS_FLASH.store(0, Ordering::Relaxed);
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

/// 推进翻页动画帧计数。
pub fn tick_page_frame() {
    PAGE_FRAME.fetch_add(1, Ordering::Relaxed);
}

/// 获取当前翻页动画亮度因子。
#[must_use]
pub fn page_brightness_factor() -> f32 {
    let f = PAGE_FRAME.load(Ordering::Relaxed);
    match f {
        0..=4 => 0.6 + f32::from(f.min(4) as u16) * 0.1,
        _ => 1.0,
    }
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

/// 菜单 icon。
pub fn menu_icon(host: &mut dyn SceneHost, kind: usize, x: i32, y: i32) {
    let theme = host.theme().clone();
    let kind_u32 = u32::try_from(kind).unwrap_or(0);
    with_canvas(host, |c| ti_render::shell::menu_icon(c, &theme, kind_u32, x, y));
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
