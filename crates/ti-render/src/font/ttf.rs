//! TTF 软件光栅化（`ab_glyph`，无 Freetype 依赖）。
//!
//! 输出与位图字体同构的 `Raster` 位图，可直接 blit 到帧缓冲。
//! 实例绑定单一字号，同字节多档 = 多实例。

use ab_glyph::{Font as AbFont, FontVec, PxScale, ScaleFont};

use crate::font::bitmap::Raster;

/// TTF/OTF 软件光栅化字体。
///
/// 字符级缓存：同字号同字符只光栅一次，之后返回克隆。
/// `Arc` 内层 = 实例廉价可克隆。
#[derive(Debug, Clone)]
pub struct TtfFont {
    inner: std::sync::Arc<TtfFontInner>,
}

#[derive(Debug)]
struct TtfFontInner {
    font: FontVec,
    size_px: f32,
    /// 字符 → 该字号位图缓存（`BTreeMap` 确定性遍历）。
    cache: std::sync::Mutex<std::collections::BTreeMap<char, Raster>>,
}

impl TtfFont {
    /// 从字节加载字体（TTF/OTF），绑定呈现字号 `size_px`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8], size_px: f32) -> Option<Self> {
        let font = FontVec::try_from_vec(bytes.to_vec()).ok()?;
        Some(Self {
            inner: std::sync::Arc::new(TtfFontInner {
                font,
                size_px,
                cache: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            }),
        })
    }

    /// 绑定字号（px）。
    #[must_use]
    pub fn size_px(&self) -> f32 {
        self.inner.size_px
    }

    /// 内部字体引用。
    #[must_use]
    pub fn font(&self) -> &FontVec {
        &self.inner.font
    }

    /// 光栅化字符为位图（命中缓存后为克隆，非重算）。
    #[must_use]
    pub fn rasterize(&self, ch: char) -> Raster {
        if let Some(r) = self.cache_get(ch) {
            return r;
        }
        let raster = self.rasterize_fresh(ch);
        self.cache_fill(ch, raster.clone());
        raster
    }

    fn cache_get(&self, ch: char) -> Option<Raster> {
        let guard = self.inner.cache.lock().ok()?;
        guard.get(&ch).cloned()
    }

    fn cache_fill(&self, ch: char, raster: Raster) {
        if let Ok(mut guard) = self.inner.cache.lock() {
            guard.insert(ch, raster);
        }
    }

    fn rasterize_fresh(&self, ch: char) -> Raster {
        if ch.is_whitespace() {
            return Raster::blank();
        }
        let scale = PxScale::from(self.inner.size_px);
        let scaled = self.inner.font.as_scaled(scale);
        if self.inner.font.glyph_id(ch) == ab_glyph::GlyphId(0) {
            return Self::tofu_at(self.inner.size_px);
        }
        let Some(outlined) = scaled.outline_glyph(scaled.scaled_glyph(ch)) else {
            return Self::tofu_at(self.inner.size_px);
        };
        let bounds = outlined.px_bounds();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            let cols = bounds.width().ceil().max(1.0) as usize;
            let rows = bounds.height().ceil().max(1.0) as usize;
            let mut ink = vec![false; cols * rows].into_boxed_slice();
            outlined.draw(|x, y, v| {
                let px = x as usize;
                let py = y as usize;
                if px < cols && py < rows && v > 0.3 {
                    ink[py * cols + px] = true;
                }
            });
            Raster::from_ink(cols, rows, ink)
        }
    }

    /// 缺字豆腐块边长（`size_px` 取整，下限 12）。
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn tofu_size(size_px: f32) -> usize {
        size_px.round().max(12.0) as usize
    }

    fn tofu_at(size_px: f32) -> Raster {
        let s = Self::tofu_size(size_px);
        Raster::from_fn_dims(s, s, |x, y| x == 0 || y == 0 || x + 1 == s || y + 1 == s)
    }

    /// 字符推进宽度（像素，本实例绑定字号；文本布局用）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn advance(&self, ch: char) -> f32 {
        let scale = PxScale::from(self.inner.size_px);
        let scaled = self.inner.font.as_scaled(scale);
        let glyph_id = self.inner.font.glyph_id(ch);
        if glyph_id == ab_glyph::GlyphId(0) {
            return Self::tofu_size(self.inner.size_px) as f32;
        }
        scaled.h_advance(glyph_id)
    }

    /// 行高（像素，本实例绑定字号）。
    #[must_use]
    pub fn line_height(&self) -> f32 {
        let scale = PxScale::from(self.inner.size_px);
        let scaled = self.inner.font.as_scaled(scale);
        scaled.line_gap() + scaled.height()
    }
}
