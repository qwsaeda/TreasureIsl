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

// === v3 场景叙事装饰层（UI 重设计 v3.1 index 试点，docs/ui-redesign-v3/03 §3.1） ===
//
// 零游戏语义 + 零 RNG：星位表/几何全为常量（快照可复现），颜色全走 theme 令牌
// （check_hardcode 门禁：chrome/ti-render 禁裸色值）。矩形非法（w/h ≤ 0）= 不画，
// 数据驱动由调用方（`chrome::star_sky` / `chrome::title_art_block`）判键。

/// v3 星空常量星位表：`(百分数 x, 百分数 y, 边长 px, 是否亮星)`。
///
/// 确定性（无随机数，渲染快照可复现）；26 颗星覆盖全画布，屏心净空
/// （屏心 50,52 半径 12% 内无星，视觉重心区不被星点干扰）。
const STARS: &[(u8, u8, u8, bool)] = &[
    (8, 9, 1, false),
    (22, 5, 2, true),
    (36, 14, 1, false),
    (52, 7, 1, false),
    (71, 11, 2, true),
    (88, 6, 1, false),
    (15, 21, 1, false),
    (64, 19, 1, false),
    (90, 23, 2, true),
    (6, 33, 2, true),
    (30, 29, 1, false),
    (79, 31, 1, false),
    (94, 40, 1, false),
    (10, 47, 1, false),
    (86, 51, 2, true),
    (18, 60, 2, true),
    (92, 63, 1, false),
    (7, 72, 1, false),
    (38, 68, 1, false),
    (84, 75, 2, true),
    (14, 84, 1, false),
    (62, 81, 1, false),
    (91, 88, 1, false),
    (25, 91, 2, true),
    (48, 94, 1, false),
    (74, 92, 1, false),
];

/// 星域内屏心排除半径（百分数，重心区净空）。
const STAR_CENTER_EXCLUDE: u8 = 12;

/// v3 氛围星空：`rect` 内确定性星点（暗星 1px `parch_light` / 亮星 2px
/// `gold_glow` + 四向 1px 光晕 `parch_light`）。
///
/// 绘制区夹到 12px 外框内（`wood_frame` 之上层不得覆盖框线）；z-order = 调用序
/// （`08_render_arch.md` 契约：背景层 → 装饰层 → 内容层）。
pub fn star_sky(host: &mut dyn Canvas, theme: &Theme, x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 || h <= 0 {
        return;
    }
    // 夹到整屏外框内（wood_frame 12px 边）
    let x0 = x.max(12);
    let y0 = y.max(12);
    let x1 = (x + w).min(LOGICAL_W_I32 - 12);
    let y1 = (y + h).min(LOGICAL_H_I32 - 12);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    for &(fx, fy, size, bright) in STARS {
        // 屏心净空（P-SoT-2）：跳过排除半径内的星
        if fx.abs_diff(50) <= STAR_CENTER_EXCLUDE && fy.abs_diff(52) <= STAR_CENTER_EXCLUDE {
            continue;
        }
        let px = x0 + i32::from(fx) * (x1 - x0) / 100;
        let py = y0 + i32::from(fy) * (y1 - y0) / 100;
        let side = i32::from(size);
        let col = if bright { theme.gold_glow } else { theme.parch_light };
        if bright {
            // 四向光晕（1px）
            host.fill_rect(px - 1, py, 1, 1, theme.parch_light);
            host.fill_rect(px + side, py, 1, 1, theme.parch_light);
            host.fill_rect(px, py - 1, 1, 1, theme.parch_light);
            host.fill_rect(px, py + side, 1, 1, theme.parch_light);
        }
        host.fill_rect(px, py, side, side, col);
    }
}

/// v3 仪式感标题艺术面板：卡底 + 木纹双描边 + `bevel` 凸凹 + 中央罗盘饰（横/竖针 +
/// 中心点）+ 上下金色饰线。文字由调用方叠画（原语零语义）。
///
/// 几何：内容区 = 矩形内缩 12px；罗盘 = 56×3 / 3×56 针 + 8×8 中心点
/// （`title_bar` 右侧罗盘 28×3 语言的 2 倍放大）；饰线 = 内区上下 8px 处 1px。
pub fn title_art_block(host: &mut dyn Canvas, theme: &Theme, x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 || h <= 0 {
        return;
    }
    let ix = x + 12;
    let iy = y + 12;
    let iw = w - 24;
    let ih = h - 24;
    if iw <= 0 || ih <= 0 {
        return;
    }
    // 卡底 + 木纹外描边（2px）
    host.fill_rect(ix, iy, iw, ih, theme.menu_bg);
    host.fill_rect(ix, iy, iw, 2, theme.wood);
    host.fill_rect(ix, iy + ih - 2, iw, 2, theme.wood);
    host.fill_rect(ix, iy, 2, ih, theme.wood);
    host.fill_rect(ix + iw - 2, iy, 2, ih, theme.wood);
    // bevel（顶/左凸起，底凹陷）
    host.fill_rect(ix + 2, iy + 2, iw - 4, 1, theme.bevel_hi);
    host.fill_rect(ix + 2, iy + 2, 1, ih - 4, theme.bevel_hi);
    host.fill_rect(ix + 2, iy + ih - 3, iw - 4, 1, theme.bevel_shadow);
    // 上下金色饰线
    host.fill_rect(ix + 8, iy + 8, iw - 16, 1, theme.gold_bright);
    host.fill_rect(ix + 8, iy + ih - 9, iw - 16, 1, theme.gold_bright);
    // 中央罗盘饰（罗盘十字 + 中心点；内区过矮时只画饰线不画罗盘）
    if ih >= 96 {
        let cx = ix + iw / 2;
        let cy = iy + ih / 2;
        host.fill_rect(cx - 28, cy - 1, 56, 3, theme.gold_bright);
        host.fill_rect(cx - 1, cy - 28, 3, 56, theme.gold_bright);
        host.fill_rect(cx - 4, cy - 4, 8, 8, theme.gold);
    }
}
