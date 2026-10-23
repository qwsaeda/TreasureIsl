#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
//! 通用原语集（S18.7）—— **后端无关的界面描述**的可执行语义。
//!
//! 界面 = `scenes.json` 的 `primitives` 数组（有序原语表）；本模块是 **CPU 执行器**
//! （`Bevy` 执行器 = `ti-engine::scene_render`，两侧消费同一份描述 → 同构）。
//!
//! **通用原语只有 6 种**（渲染层不认识"菜单""木框"等场景语义）：
//! | kind | 语义 |
//! |---|---|
//! | `rect` | 实心矩形 |
//! | `text` | 文本（`tier` = title/body/hint；`rect` 内居中） |
//! | `icon` | 图标（`params.icon` = 图标名；`params.size` = 边长） |
//! | `grad` | 垂直渐变（`params.from_token`/`to_token`/`steps`） |
//! | `glow` | 径向光晕（圆心 + 半径） |
//! | `disc` | 实心圆盘（圆心 + 半径） |
//!
//! 场景语义 = 数据（`scenes.json`）；本模块只认 6 个 kind + theme 令牌名。

use crate::canvas::Canvas;
use crate::framebuffer::{LOGICAL_H_I32, LOGICAL_W_I32};
use crate::theme::Theme;

/// RGB565 → RGB8（5/6/5 位展 8 位）。
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn rgb565_to_rgb8(c: u16) -> (u8, u8, u8) {
    let r5 = (c >> 11) & 0x1F;
    let g6 = (c >> 5) & 0x3F;
    let b5 = c & 0x1F;
    (
        (u32::from(r5) << 3 | u32::from(r5) >> 2) as u8,
        (u32::from(g6) << 2 | u32::from(g6) >> 4) as u8,
        (u32::from(b5) << 3 | u32::from(b5) >> 2) as u8,
    )
}

/// RGB565 线性插值（`t` = 0..=1）。
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn lerp_rgb565(from: u16, to: u16, t: f32) -> u16 {
    let t = t.clamp(0.0, 1.0);
    let f = u32::from(from);
    let g = u32::from(to);
    let mix = |a: u32, b: u32| -> u32 {
        let d = i64::from(b) - i64::from(a);
        #[allow(clippy::cast_possible_truncation)]
        let v = i64::from(a) + (d * (t * 1000.0) as i64) / 1000;
        u32::try_from(v.max(0)).unwrap_or(0)
    };
    let r = mix((f >> 11) & 0x1F, (g >> 11) & 0x1F).min(31);
    let gg = mix((f >> 5) & 0x3F, (g >> 5) & 0x3F).min(63);
    let b = mix(f & 0x1F, g & 0x1F).min(31);
    u16::try_from((r << 11) | (gg << 5) | b).unwrap_or(0)
}

/// RGB565 通道缩放（千分比 0..=1000）。
#[must_use]
pub fn scale_rgb565(color: u16, scale_permille: u32) -> u16 {
    let v = u32::from(color);
    let r = (((v >> 11) & 0x1F) * scale_permille / 1000).min(31);
    let g = (((v >> 5) & 0x3F) * scale_permille / 1000).min(63);
    let b = ((v & 0x1F) * scale_permille / 1000).min(31);
    u16::try_from((r << 11) | (g << 5) | b).unwrap_or(0)
}

/// 原语 `rect`：实心矩形。
pub fn rect(c: &mut dyn Canvas, left: i32, top: i32, width: i32, height: i32, color: u16) {
    c.fill_rect(left, top, width, height, color);
}

/// 原语 `text`：文本（`rect` 内水平居中；`tier` 决定字号档）。
#[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
pub fn text(
    c: &mut dyn Canvas,
    content: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    tier: &str,
    color: u16,
) {
    if content.is_empty() {
        return;
    }
    let (tw, th) = match tier {
        "title" => (c.title_text_width(content), c.title_glyph_height()),
        "hint" => (c.hint_text_width(content), c.hint_glyph_height()),
        _ => (c.text_width(content), c.glyph_height()),
    };
    let tx = x + (w - tw) / 2;
    let ty = y + (h - th) / 2;
    match tier {
        "title" => c.draw_title_text(content, tx, ty, color),
        "hint" => c.draw_hint_text(content, tx, ty, color),
        _ => c.draw_text(content, tx, ty, color),
    }
}

/// 原语 `icon`：图标（`ti_render::icons` 的图标语言）。
#[allow(clippy::many_single_char_names)]
pub fn icon(c: &mut dyn Canvas, kind: u32, x: i32, y: i32, size: i32, color: u16) {
    let _ = size;
    crate::icons::icon(c, &Theme::default(), kind, x, y, color);
}

/// 原语 `grad`：垂直渐变（`steps` 条色带）。
#[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
pub fn grad(c: &mut dyn Canvas, x: i32, y: i32, w: i32, h: i32, from: u16, to: u16, steps: i32) {
    if w <= 0 || h <= 0 {
        return;
    }
    let steps = clamp_grad_steps(steps);
    let step_h = h / steps;
    if step_h <= 0 {
        return;
    }
    for i in 0..steps {
        #[allow(clippy::cast_precision_loss)]
        let t = i as f32 / (steps - 1) as f32;
        c.fill_rect(x, y + i * step_h, w, step_h + 1, lerp_rgb565(from, to, t));
    }
}

/// 渐变色带级数钳制（2..=256；**双执行器共享**：CPU 端 [`grad`] 与 Bevy 端
/// `ti_engine::scene_render` 同读，飞轮 R1 ⑥-12 双端不漂移）。
///
/// 普通 `fn`（当前工具链 `Ord::max/min` 未 const 稳定，`const fn` 受限）。
#[must_use]
pub const fn clamp_grad_steps(steps: i32) -> i32 {
    if steps < 2 {
        2
    } else if steps > 256 {
        256
    } else {
        steps
    }
}

/// 原语 `glow`：径向光晕（逐行扫描线；中心亮 → 边缘暗）。
#[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
pub fn glow(c: &mut dyn Canvas, cx: i32, cy: i32, radius: i32, color: u16) {
    /// 光晕亮度基线（千分比：边缘 35%；飞轮 R1 ①-28 命名化）。
    const GLOW_BASE_PERMILLE: u32 = 350;
    /// 光晕亮度斜坡（千分比：`t=1` 时中心 +650 → 100%；同上）。
    const GLOW_SWING_PERMILLE: u32 = 650;
    if radius <= 0 {
        return;
    }
    let (r0, g0, b0) = rgb565_to_rgb8(color);
    for dy in -radius..=radius {
        let inside = radius * radius - dy * dy;
        if inside < 0 {
            continue;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let half_w = (inside as f32).sqrt() as i32;
        #[allow(clippy::cast_precision_loss)]
        let t = 1.0 - dy.abs() as f32 / radius.max(1) as f32;
        // 亮度衰减（按 RGB565 通道缩放，避免整体乘法溢出）。
        #[allow(clippy::cast_possible_truncation)]
        let level = scale_rgb565(
            color,
            GLOW_BASE_PERMILLE
                + u32::try_from((t * GLOW_SWING_PERMILLE as f32) as i64).unwrap_or(0),
        );
        let _ = (r0, g0, b0);
        c.fill_rect(cx - half_w, cy + dy, half_w * 2, 1, level);
    }
}

/// 原语 `disc`：实心圆盘。
#[allow(clippy::many_single_char_names, clippy::too_many_arguments)]
pub fn disc(c: &mut dyn Canvas, cx: i32, cy: i32, radius: i32, color: u16) {
    if radius <= 0 {
        return;
    }
    for dy in -radius..=radius {
        let inside = radius * radius - dy * dy;
        if inside < 0 {
            continue;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let half_w = (inside as f32).sqrt() as i32;
        c.fill_rect(cx - half_w, cy + dy, half_w * 2, 1, color);
    }
}

/// 逻辑画布常量（供边界计算）。
pub const LOGICAL_W: i32 = LOGICAL_W_I32;
/// 逻辑画布常量。
pub const LOGICAL_H: i32 = LOGICAL_H_I32;
