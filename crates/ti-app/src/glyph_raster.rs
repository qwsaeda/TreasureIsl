//! 字形光栅化：真实 `GoldLand.Fnt` 字形 → `GLYPH_WIDTH × GLYPH_ROWS` 墨点位图。
//!
//! 从 `treasure_wasm` 壳里抽到 lib 层（S2.5 补实）的原因：
//! 1. `tests/` 必须能对**渲染路径本身**做逐位断言 —— 证明画出来的是真字形，
//!    而非早期占位的伪随机位图（已废弃，仅 `tests/glyph_raster.rs::legacy_noise`
//!    保留一段历史复刻用于"噪点已被清除"的对照断言，主渲染路径不再使用）；
//! 2. 光栅逻辑与 macroquad 解耦：纯函数、无 IO、无窗口，三端共用同一份实现。
//!
//! 实测依据：`docs/facts/index.md` F-303；几何断言先例见
//! `crates/ti-formats/tests/font_parity.rs`（S1.5b）。

use ti_core::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use ti_formats::font::{Charset, Font, GLYPH_ROWS, GLYPH_WIDTH, Glyph};
use ti_platform::gfx::Framebuf;

/// 已加载的「字体 + 字符集」对（`GoldLand.Fnt` 与 `font.dat` 下标 1:1，F-303）。
pub type FontSet = (Font, Charset);

/// 12×12 墨点位图（行主序：`ink[y * cols + x]`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    /// 列数（恒为 `GLYPH_WIDTH`）。
    cols: usize,
    /// 行数（恒为 `GLYPH_ROWS`）。
    rows: usize,
    /// 墨点（`true` = 上墨）。
    ink: Box<[bool]>,
}

impl Raster {
    /// 全空白位图（空白字符用：占位推进但不着墨）。
    #[must_use]
    pub fn blank() -> Self {
        Self::from_fn(|_, _| false)
    }

    /// 真实字形位图 —— 与 [`Glyph::pixel`] **逐位一致**（验收断言点）。
    #[must_use]
    pub fn from_glyph(glyph: &Glyph) -> Self {
        Self::from_fn(|x, y| glyph.pixel(x, y))
    }

    /// 豆腐块（空心方框）：字符不在字符表时的**可见**回退，禁止静默空白。
    #[must_use]
    pub fn tofu() -> Self {
        let cols = GLYPH_WIDTH;
        let rows = usize::from(GLYPH_ROWS);
        Self::from_fn(|x, y| x == 0 || y == 0 || x + 1 == cols || y + 1 == rows)
    }

    /// 字符 → 位图：空白字符留空；命中字表用真字形；否则豆腐块。
    #[must_use]
    pub fn for_char(font: Option<&FontSet>, ch: char) -> Self {
        if ch.is_whitespace() {
            return Self::blank();
        }
        match font {
            Some((parsed, charset)) => parsed
                .glyph_for(charset, ch)
                .map_or_else(Self::tofu, |glyph| Self::from_glyph(&glyph)),
            None => Self::tofu(),
        }
    }

    fn from_fn<F: Fn(usize, usize) -> bool>(pixel_at: F) -> Self {
        let cols = GLYPH_WIDTH;
        let rows = usize::from(GLYPH_ROWS);
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

    /// 像素 `(x, y)` 是否着墨（越界为 `false`，不 panic）。
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

    /// 行主序墨点切片（长度 = `cols × rows`）。
    #[must_use]
    pub fn as_slice(&self) -> &[bool] {
        &self.ink
    }

    /// 把位图上墨到帧缓冲（整数逻辑坐标，越界部分逐像素裁剪，不整体丢弃）。
    ///
    /// 着墨完成后按实际写入的行区间调用 [`Framebuf::mark_dirty`]（S4.3 脏行带）；
    /// 全空白字形（无墨点）不置脏。
    pub fn blit(&self, fb: &mut Framebuf, x: i32, y: i32, color: u16) {
        let screen_w = i32::try_from(LOGICAL_WIDTH).unwrap_or(i32::MAX);
        let screen_h = i32::try_from(LOGICAL_HEIGHT).unwrap_or(i32::MAX);
        let stride = usize::try_from(LOGICAL_WIDTH).unwrap_or(0);
        let mut first_written: Option<i32> = None;
        let mut last_written: Option<i32> = None;
        for row in 0..self.rows {
            let Ok(offset_y) = i32::try_from(row) else {
                break;
            };
            let dst_y = y.saturating_add(offset_y);
            if !(0..screen_h).contains(&dst_y) {
                continue;
            }
            let mut row_has_ink = false;
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
                let (Ok(px), Ok(py)) = (usize::try_from(dst_x), usize::try_from(dst_y)) else {
                    continue;
                };
                let index = py.saturating_mul(stride).saturating_add(px);
                if let Some(slot) = fb.as_mut_slice().get_mut(index) {
                    *slot = color;
                    row_has_ink = true;
                }
            }
            if row_has_ink {
                if first_written.is_none() {
                    first_written = Some(dst_y);
                }
                last_written = Some(dst_y);
            }
        }
        if let (Some(fy), Some(lt)) = (first_written, last_written) {
            fb.mark_dirty(x, fy, i32::try_from(self.cols).unwrap_or(0), lt - fy + 1);
        }
    }

    /// S7.5（feature `hd-8bit`）：把位图 RGBA8 上墨到 HD 帧缓冲（与 [`Self::blit`] 同几何）。
    #[cfg(feature = "hd-8bit")]
    pub fn blit_rgba(&self, fb: &mut ti_platform::gfx::FramebufRgba, x: i32, y: i32, color: u32) {
        use ti_core::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
        let screen_w = i32::try_from(LOGICAL_WIDTH).unwrap_or(i32::MAX);
        let screen_h = i32::try_from(LOGICAL_HEIGHT).unwrap_or(i32::MAX);
        let stride = usize::try_from(LOGICAL_WIDTH).unwrap_or(0);
        let mut first_written: Option<i32> = None;
        let mut last_written: Option<i32> = None;
        for row in 0..self.rows {
            let Ok(offset_y) = i32::try_from(row) else {
                break;
            };
            let dst_y = y.saturating_add(offset_y);
            if !(0..screen_h).contains(&dst_y) {
                continue;
            }
            let mut row_has_ink = false;
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
                let (Ok(px), Ok(py)) = (usize::try_from(dst_x), usize::try_from(dst_y)) else {
                    continue;
                };
                let index = py.saturating_mul(stride).saturating_add(px);
                if let Some(slot) = fb.as_mut_slice().get_mut(index) {
                    *slot = color;
                    row_has_ink = true;
                }
            }
            if row_has_ink {
                if first_written.is_none() {
                    first_written = Some(dst_y);
                }
                last_written = Some(dst_y);
            }
        }
        if let (Some(fy), Some(lt)) = (first_written, last_written) {
            fb.mark_dirty(x, fy, i32::try_from(self.cols).unwrap_or(0), lt - fy + 1);
        }
    }
}
