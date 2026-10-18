//! 场景外壳：可复用的 UI 装饰组件（木纹框/标题条/菜单项/提示行/航海网格/图标）。
//!
//! 零游戏语义——所有组件只负责绘制装饰性 UI 元素，
//! 业务内容（菜单文字/图标含义）由调用方提供。

use crate::canvas::Canvas;
use crate::framebuffer::{LOGICAL_H_I32, LOGICAL_W_I32};
use crate::theme::Theme;

/// 整屏外框（12px 卡片描边）。
pub fn wood_frame(host: &mut dyn Canvas, theme: &Theme) {
    let f = 12i32;
    host.fill_rect(0, 0, LOGICAL_W_I32, f, theme.wood);
    host.fill_rect(0, LOGICAL_H_I32 - f, LOGICAL_W_I32, f, theme.wood);
    host.fill_rect(0, 0, f, LOGICAL_H_I32, theme.wood);
    host.fill_rect(LOGICAL_W_I32 - f, 0, f, LOGICAL_H_I32, theme.wood);
}

/// 标题条：卡片描边条 + 深底内块 + 左侧 accent 竖条 + 底部细线 + 右侧罗盘装饰。
///
/// `title_h` = 标题条高（逻辑 px）。
pub fn title_bar(host: &mut dyn Canvas, theme: &Theme, title_h: i32) {
    let h = title_h.max(1);
    host.fill_rect(12, 12, LOGICAL_W_I32 - 24, h, theme.wood);
    host.fill_rect(16, 16, LOGICAL_W_I32 - 32, h - 8, theme.parch_dark);
    // 左侧 accent 竖条
    host.fill_rect(20, 20, 8, h - 16, theme.gold_bright);
    // 底部细线
    host.fill_rect(16, h - 8, LOGICAL_W_I32 - 32, 3, theme.gold_bright);
    // 右侧罗盘装饰：横针 28×3 + 竖针 3×28 + 中心点 4×4
    let cx = LOGICAL_W_I32 - 48;
    let mid = 16 + (h - 8) / 2;
    host.fill_rect(cx - 14, mid - 1, 28, 3, theme.gold_bright);
    host.fill_rect(cx - 1, mid - 14, 3, 28, theme.gold_bright);
    host.fill_rect(cx - 2, mid - 2, 4, 4, theme.gold);
}

/// 航海图装饰层——低对比海网格（水平虚线短 rect 阵列）。
///
/// z-order：`BgBoard` 层内最低——`parchment_bg` 之后、`wood_frame`/卡片/文字之前。
pub fn nautical_grid(host: &mut dyn Canvas, theme: &Theme) {
    for row in 0..13 {
        let y = 1040 + row * 60;
        for col in 0..30 {
            host.fill_rect(60 + col * 32, y, 20, 2, theme.parch_light);
        }
    }
}

/// 菜单 icon（兼容出口）：委托 14 枚图标集（`icons::icon`），kind = 图标 id。
///
/// S17.3 起本函数**不含绘制逻辑**（零重复实现）：几何全在 `crate::icons`，
/// tint 由调用方（`chrome::ui_icon`）按选中态传入；此处保留旧签名做薄转发。
pub fn menu_icon(host: &mut dyn Canvas, theme: &Theme, kind: u32, x: i32, y: i32) {
    crate::icons::icon(host, theme, kind, x, y, theme.gold);
}

/// 菜单项：卡片按钮 + 描边 + bevel 凸起/凹陷 + 选中金底铆钉。
///
/// - `chosen` = true：金底 + 铆钉
/// - `pressed` = true：bevel 反转（凹陷实体感）
#[allow(clippy::too_many_arguments)]
pub fn menu_item(
    host: &mut dyn Canvas,
    theme: &Theme,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    chosen: bool,
    pressed: bool,
) {
    let base_color = if chosen { theme.gold_bright } else { theme.menu_bg };
    host.fill_rect(left, top, width, height, base_color);
    // 卡片描边（8px）
    host.fill_rect(left, top, width, 8, theme.menu_border);
    host.fill_rect(left, top + height - 8, width, 8, theme.menu_border);
    host.fill_rect(left, top + 8, 8, height - 16, theme.menu_border);
    host.fill_rect(left + width - 8, top + 8, 8, height - 16, theme.menu_border);
    // bevel：pressed 时反转（凹陷），否则凸起
    let (hi, shadow) = if pressed {
        (theme.bevel_shadow, theme.bevel_hi)
    } else {
        (theme.bevel_hi, theme.bevel_shadow)
    };
    host.fill_rect(left + 8, top + 8, width - 16, 2, hi);
    host.fill_rect(left + 8, top + 8, 2, height - 16, hi);
    host.fill_rect(left + 8, top + height - 10, width - 16, 2, shadow);
    host.fill_rect(left + width - 10, top + 8, 2, height - 16, shadow);
    if chosen {
        // 铆钉（左右各 1 个 8×8）
        host.fill_rect(left + 24, top + height / 2 - 4, 8, 8, theme.gold_bright);
        host.fill_rect(left + width - 32, top + height / 2 - 4, 8, 8, theme.gold_bright);
    }
}

/// 血条槽位：木纹槽底 + 内阴影（前景血量由调用方叠在槽上）。
pub fn bar_slot(host: &mut dyn Canvas, theme: &Theme, x: i32, y: i32, w: i32, h: i32) {
    host.fill_rect(x, y, w, h, theme.hilite);
    host.fill_rect(x + 4, y + 4, w - 8, 2, theme.bevel_shadow);
}

/// 内容面板：羊皮纸浅块 + 木纹细边 + 内 bevel。
pub fn panel(host: &mut dyn Canvas, theme: &Theme, x: i32, y: i32, w: i32, h: i32) {
    host.fill_rect(x, y, w, h, theme.menu_bg);
    host.fill_rect(x, y, w, 2, theme.menu_border);
    host.fill_rect(x, y + h - 2, w, 2, theme.menu_border);
    host.fill_rect(x, y + 2, 2, h - 4, theme.menu_border);
    host.fill_rect(x + w - 2, y + 2, 2, h - 4, theme.menu_border);
    host.fill_rect(x + 2, y + 2, w - 4, 1, theme.bevel_hi);
    host.fill_rect(x + 2, y + h - 3, w - 4, 1, theme.bevel_shadow);
}

/// 底部提示行：卡色底条 + 白墨字 + 水平居中。
///
/// 底条宽 = 提示档文本宽 + 8px 衬边；底条高 = 提示档字形高 + 4px。
pub fn hint_row(host: &mut dyn Canvas, theme: &Theme, text: &str, screen_cx: i32, y: i32) {
    let w = host.hint_text_width(text) + 8;
    let h = host.hint_glyph_height() + 4;
    let cx = (2 * screen_cx - w) / 2;
    host.fill_rect(cx, y - 2, w, h, theme.menu_bg);
    host.fill_rect(cx, y - 2, w, 1, theme.bevel_hi);
    host.fill_rect(cx, y - 2 + h - 1, w, 1, theme.bevel_shadow);
    host.draw_hint_text(text, cx + 4, y, theme.text_on_parch);
}

/// 标题条内文字（标题档字号）。
pub fn draw_title_text(host: &mut dyn Canvas, text: &str, x: i32, y: i32, color: u16) {
    host.draw_title_text(text, x, y, color);
}

/// 正文档文字（正文字号）。
pub fn draw_body_text(host: &mut dyn Canvas, text: &str, x: i32, y: i32, color: u16) {
    host.draw_text(text, x, y, color);
}

/// 次级文字（正文档字号，次要灰墨）。
pub fn draw_body_text_secondary(host: &mut dyn Canvas, theme: &Theme, text: &str, x: i32, y: i32) {
    host.draw_text(text, x, y, theme.secondary);
}
