//! 文本排版 API：宽度/高度测量 + 居中定位 + 多行折行。
//!
//! 纯函数排版逻辑，不直接操作像素——宽度/高度测量供调用方
//! 决定绘制位置，居中公式保证文字在指定区域内水平/垂直居中。

/// 文本排版度量结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextMetrics {
    /// 文本像素宽。
    pub width: i32,
    /// 字形垂直高（px）。
    pub height: i32,
    /// 行高（多行排版用）。
    pub line_height: i32,
}

/// 计算文本在指定区域内的水平居中左缘 x。
///
/// `area_width` 为区域宽度，`text_width` 为文本实际宽度。
#[must_use]
pub fn center_x(area_width: i32, text_width: i32) -> i32 {
    (area_width - text_width) / 2
}

/// 计算文本在指定区域内的垂直居中顶缘 y。
///
/// `area_height` 为区域高度，`glyph_height` 为字形高度。
#[must_use]
pub fn center_y(area_height: i32, glyph_height: i32) -> i32 {
    (area_height - glyph_height) / 2
}

/// 多行文本折行：按最大宽度把长文本拆成多行。
///
/// 按字符逐个累加宽度，超过 `max_width` 时折行。
/// 返回折行后的文本行列表（每行宽度 ≤ `max_width`）。
#[must_use]
pub fn wrap_text(text: &str, char_width: i32, max_width: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_w = 0i32;

    for ch in text.chars() {
        let ch_w = if ch.is_whitespace() { char_width / 2 } else { char_width };
        if current_w + ch_w > max_width && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
            current_w = 0;
        }
        current.push(ch);
        current_w += ch_w;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// 多行排版：计算总行数与总高度。
#[must_use]
pub fn multi_line_metrics(line_count: usize, line_height: i32) -> TextMetrics {
    let height = i32::try_from(line_count).unwrap_or(i32::MAX).saturating_mul(line_height);
    TextMetrics { width: 0, height, line_height }
}
