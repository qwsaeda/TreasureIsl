//! 背景组合函数 `parchment_bg`（底色 + 垂直渐变行 + 四边暗角）。
//!
//! 绘制原语是 [`crate::canvas::Canvas`] 的 trait 方法（见 `docs/process/08_render_arch.md`
//! §2）；本模块只提供建立在原语之上的**组合函数**。
//!
//! 高频渲染路径优化：避免逐像素 `fill_rect(1,1)` 造成百万次调用，
//! 改为整行/整条批量填充（S15.10：`gradient_row`/`vignette` 为本组合函数私有辅助，
//! 原语 1 的自由函数转发壳已删除 —— 零外部调用且与 trait 方法重复）。

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

/// 垂直渐变行（整行填充，从 `top_color` 渐变到 `bottom_color`；`parchment_bg` 内部用，
/// S15.10 收可见性 —— 对外原语是 `Canvas` trait 方法，不是本自由函数）。
///
/// 逐行计算混合色后整行填充，避免逐像素调用。
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn gradient_row(host: &mut dyn Canvas, theme: &Theme, y: i32, strength: f32) {
    let t = y as f32 / LOGICAL_H_I32 as f32;
    let color = blend_color(theme.parch_light, theme.parch_dark, strength * t);
    host.fill_rect(0, y, LOGICAL_W_I32, 1, color);
}

/// 四边暗角（整条填充，压深边缘；`parchment_bg` 内部用，S15.10 收可见性）。
fn vignette(host: &mut dyn Canvas, theme: &Theme, vig: i32) {
    host.fill_rect(0, 0, LOGICAL_W_I32, vig, theme.parch_dark);
    host.fill_rect(0, LOGICAL_H_I32 - vig, LOGICAL_W_I32, vig, theme.parch_dark);
}

/// 背景底：整帧纯色 + 垂直渐变 + 四边暗角。
///
/// 1080×1920 下**逐像素 `fill_rect(1,1)` 会卡死**（旧棋盘 4 层 = 828 万次/帧）。
/// 改：底色整帧 1 次 + 渐变逐行整行（~1920 次）+ 暗角整条。
/// 视觉调参值命名化（飞轮 R1 ①-30：02_conventions「数值进数据或命名常量」底线口径）。
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn parchment_bg(host: &mut dyn Canvas, theme: &Theme) {
    /// 垂直渐变强度（顶→底混入 `parch_light` 的比例，0.18；同 ①-30）。
    const PARCHMENT_GRAD_STRENGTH: f32 = 0.18;
    /// 暗角条宽（px，上下整条压深；同 ①-30）。
    const PARCHMENT_VIGNETTE_PX: i32 = 36;
    host.fill_rect(0, 0, LOGICAL_W_I32, LOGICAL_H_I32, theme.parch_dark);
    for py in 0..LOGICAL_H_I32 {
        gradient_row(host, theme, py, PARCHMENT_GRAD_STRENGTH);
    }
    vignette(host, theme, PARCHMENT_VIGNETTE_PX);
}
