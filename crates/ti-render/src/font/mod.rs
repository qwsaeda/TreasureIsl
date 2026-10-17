//! 系统：`Font` trait + `FontSet` 类型别名。
//!
//! 字体系统分两层：
//! - **TTF 光栅化**（[`ttf`]）：矢量字体 → 位图（`ab_glyph`）
//! - **位图回退**（[`bitmap`]）：内置点阵字体解析与光栅

pub mod bitmap;
pub mod ttf;

pub use ttf::TtfFont;

/// 已加载的「字体 + 字符集」对。
pub type FontSet = (ti_formats::font::Font, ti_formats::font::Charset);
