//! 字体系统分两层：
//! - **TTF 光栅化**（[`ttf`]）：矢量字体 → 位图（`ab_glyph`）
//! - **位图回退**（[`bitmap`]）：内置点阵字体 → 墨点位图
//!
//! S15.10（可复用性收口）：本模块**不依赖 `ti-formats`**。字体的**载体格式**
//! （原版 `GoldLand.Fnt` + `font.dat`）由 `ti-formats` 解析，宿主（`ti-app`）在
//! 加载期把解析结果转换成本模块的**中性表示** [`FontSet`]（行位图而非原始字节）。
//! 换游戏 / 换字体格式时只需换「解析 + 转换」那一层，渲染框架不动。

pub mod bitmap;
pub mod ttf;

pub use ttf::TtfFont;

/// 点阵字形行数（中性表示的固定行数，与宿主字体载体无关）。
pub const GLYPH_ROWS: u16 = 12;

/// 点阵字形每行有效位数（中性表示的固定字宽）。
pub const GLYPH_WIDTH: usize = 12;

/// 一个点阵字形（行主序：`rows[y]` 的 bit `GLYPH_WIDTH-1-x` 为像素 `(x, y)`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    /// 各行像素位（每行低 [`GLYPH_WIDTH`] 位有效）。
    pub rows: [u16; GLYPH_ROWS as usize],
}

impl Glyph {
    /// 像素 `(x, y)` 是否着墨（坐标越界返回 `false`，不 panic）。
    #[must_use]
    pub fn pixel(&self, x: usize, y: usize) -> bool {
        if x >= GLYPH_WIDTH {
            return false;
        }
        self.rows.get(y).is_some_and(|r| (r >> (GLYPH_WIDTH - 1 - x)) & 1 == 1)
    }
}

/// 已加载的「点阵字体 + 字符表」中性表示（`FontSet`）。
///
/// 与宿主字体载体（`GoldLand.Fnt` 等二进制格式）**解耦**：宿主解析后转换而来，
/// 渲染框架只认本类型 —— 这是框架可复用（换游戏不改编框架）的关键边界。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontSet {
    /// 字形表（与 `chars` 一一对应）。
    pub glyphs: Vec<Glyph>,
    /// 字符表（按字形表顺序；含重复时以首个出现位为准）。
    pub chars: Vec<char>,
}

impl FontSet {
    /// 字符表首个出现位（与 `ti_formats::font::Charset::index_of` 同口径：1 起原版码位）。
    #[must_use]
    pub fn index_of(&self, ch: char) -> Option<usize> {
        self.chars.iter().position(|&c| c == ch)
    }

    /// 字符 → 字形（缺字 = `None`，调用方画豆腐块）。
    #[must_use]
    pub fn glyph_for(&self, ch: char) -> Option<Glyph> {
        self.index_of(ch).and_then(|i| self.glyphs.get(i).copied())
    }
}
