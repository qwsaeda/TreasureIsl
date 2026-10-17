//! 3 个绘制原语：纯色矩形 / 垂直渐变行 / 四边暗角。
//!
//! 高频渲染路径优化：避免逐像素 `fill_rect(1,1)` 造成百万次调用，
//! 改为整行/整条批量填充。

use crate::canvas::Canvas;
use crate::framebuffer::{LOGICAL_H_I32, LOGICAL_W_I32};
use crate::theme::Theme;

/// RGB565 颜色混合（`fg` 按 `alpha` 叠到 `bg` 上，百分比量化避免浮点 cast）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn blend_color(fg: u16, bg: u16, alpha: f32) -> u16 {
    let percent = (alpha.clamp(0.0, 1.0) * 100.0).round() as u8;
    let inv_percent = 100 - u32::from(percent);
    let mix = |ch_fg: u32, ch_bg: u32, shift: u32, mask: u16| -> u16 {
        let mixed = (ch_fg * u32::from(percent) + ch_bg * inv_percent + 50) / 100;
        ((mixed & u32::from(mask)) << shift) as u16
    };
    let r = mix(u32::from((fg >> 11) & 0x1F), u32::from((bg >> 11) & 0x1F), 11, 0x1F);
    let g = mix(u32::from((fg >> 5) & 0x3F), u32::from((bg >> 5) & 0x3F), 5, 0x3F);
    let b = mix(u32::from(fg & 0x1F), u32::from(bg & 0x1F), 0, 0x1F);
    r | g | b
}

/// 原语 1：纯色矩形填充。
pub fn fill_rect(host: &mut dyn Canvas, x: i32, y: i32, w: i32, h: i32, color: u16) {
    host.fill_rect(x, y, w, h, color);
}

/// 原语 2：垂直渐变行（整行填充，从 `top_color` 渐变到 `bottom_color`）。
///
/// 逐行计算混合色后整行填充，避免逐像素调用。
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn gradient_row(host: &mut dyn Canvas, theme: &Theme, y: i32, strength: f32) {
    let t = y as f32 / LOGICAL_H_I32 as f32;
    let color = blend_color(theme.parch_light, theme.parch_dark, strength * t);
    host.fill_rect(0, y, LOGICAL_W_I32, 1, color);
}

/// 原语 3：四边暗角（整条填充，压深边缘）。
pub fn vignette(host: &mut dyn Canvas, theme: &Theme, vig: i32) {
    host.fill_rect(0, 0, LOGICAL_W_I32, vig, theme.parch_dark);
    host.fill_rect(0, LOGICAL_H_I32 - vig, LOGICAL_W_I32, vig, theme.parch_dark);
}

/// 背景底：整帧纯色 + 垂直渐变 + 四边暗角。
///
/// 1080×1920 下**逐像素 `fill_rect(1,1)` 会卡死**（旧棋盘 4 层 = 828 万次/帧）。
/// 改：底色整帧 1 次 + 渐变逐行整行（~1920 次）+ 暗角整条。
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn parchment_bg(host: &mut dyn Canvas, theme: &Theme) {
    host.fill_rect(0, 0, LOGICAL_W_I32, LOGICAL_H_I32, theme.parch_dark);
    for py in 0..LOGICAL_H_I32 {
        gradient_row(host, theme, py, 0.18);
    }
    vignette(host, theme, 36);
}
