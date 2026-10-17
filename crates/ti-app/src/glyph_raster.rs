//! 字形光栅化：真实 `GoldLand.Fnt` 字形 → 墨点位图 + TTF 光栅化。
//!
//! 实现已抽取到 [`ti_render::font`]，本模块保留重导出用于兼容旧导入路径。

pub use ti_render::font::FontSet;
pub use ti_render::font::bitmap::Raster;
pub use ti_render::font::ttf::TtfFont;
