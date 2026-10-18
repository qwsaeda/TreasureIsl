//! 位图字体回退：12×12 墨点位图光栅化。
//!
//! 从 `GoldLand.Fnt` 解析的字形 → 位图，缺字画豆腐块。
//! 作为 TTF 不可用时的回退路径。

use crate::font::{FontSet, GLYPH_ROWS, GLYPH_WIDTH, Glyph};
use crate::framebuffer::{CANVAS_H, CANVAS_W};

/// 12×12 墨点位图（行主序：`ink[y * cols + x]`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    cols: usize,
    rows: usize,
    ink: Box<[bool]>,
}

impl Raster {
    /// 全空白位图（空白字符用）。
    #[must_use]
    pub fn blank() -> Self {
        Self::from_fn(|_, _| false)
    }

    /// 真实字形位图。
    #[must_use]
    pub fn from_glyph(glyph: &Glyph) -> Self {
        Self::from_fn(|x, y| glyph.pixel(x, y))
    }

    /// 豆腐块（空心方框）：字符不在字符表时的可见回退。
    #[must_use]
    pub fn tofu() -> Self {
        let cols = GLYPH_WIDTH;
        let rows = usize::from(GLYPH_ROWS);
        Self::from_fn(|x, y| x == 0 || y == 0 || x + 1 == cols || y + 1 == rows)
    }

    /// 字符 → 位图：空白留空；命中字表用真字形；否则豆腐块。
    #[must_use]
    pub fn for_char(font: Option<&FontSet>, ch: char) -> Self {
        if ch.is_whitespace() {
            return Self::blank();
        }
        match font {
            Some(set) => {
                set.glyph_for(ch).map_or_else(Self::tofu, |glyph| Self::from_glyph(&glyph))
            }
            None => Self::tofu(),
        }
    }

    fn from_fn<F: Fn(usize, usize) -> bool>(pixel_at: F) -> Self {
        Self::from_fn_dims(GLYPH_WIDTH, usize::from(GLYPH_ROWS), pixel_at)
    }

    pub(crate) fn from_fn_dims<F: Fn(usize, usize) -> bool>(
        cols: usize,
        rows: usize,
        pixel_at: F,
    ) -> Self {
        let mut ink = vec![false; cols * rows].into_boxed_slice();
        for row in 0..rows {
            for col in 0..cols {
                if let Some(slot) = ink.get_mut(row * cols + col) {
                    *slot = pixel_at(col, row);
                }
            }
        }
        Self { cols, rows, ink }
    }

    /// 从外部墨点构造（尺寸可变）。
    #[must_use]
    pub fn from_ink(cols: usize, rows: usize, ink: Box<[bool]>) -> Self {
        Self { cols, rows, ink }
    }

    /// 列数。
    #[must_use]
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// 行数。
    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// 像素 `(x, y)` 是否着墨（越界为 `false`）。
    #[must_use]
    pub fn at(&self, x: usize, y: usize) -> bool {
        if x >= self.cols {
            return false;
        }
        self.ink.get(y * self.cols + x).is_some_and(|&on| on)
    }

    /// 墨点总数。
    #[must_use]
    pub fn ink_count(&self) -> usize {
        self.ink.iter().filter(|&&on| on).count()
    }

    /// 行主序墨点切片。
    #[must_use]
    pub fn as_slice(&self) -> &[bool] {
        &self.ink
    }

    /// 把位图上墨到帧缓冲（整数逻辑坐标，越界部分逐像素裁剪）。
    /// `fb` = 任意实现 [`crate::framebuffer::PixelSink`] 的像素目标（平台无关）。
    pub fn blit(&self, fb: &mut dyn crate::framebuffer::PixelSink, x: i32, y: i32, color: u16) {
        let screen_w = i32::try_from(CANVAS_W).unwrap_or(i32::MAX);
        let screen_h = i32::try_from(CANVAS_H).unwrap_or(i32::MAX);
        for row in 0..self.rows {
            let Ok(offset_y) = i32::try_from(row) else {
                break;
            };
            let dst_y = y.saturating_add(offset_y);
            if !(0..screen_h).contains(&dst_y) {
                continue;
            }
            for col in 0..self.cols {
                if !self.at(col, row) {
                    continue;
                }
                let Ok(offset_x) = i32::try_from(col) else {
                    break;
                };
                let dst_x = x.saturating_add(offset_x);
                if !(0..screen_w).contains(&dst_x) {
                    continue;
                }
                fb.set_pixel(dst_x, dst_y, color);
            }
        }
    }
}
