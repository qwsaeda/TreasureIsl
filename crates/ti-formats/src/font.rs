//! 字体 `GoldLand.Fnt` + 字符集 `font.dat` 解析与位图解码（S1.5 / S1.5b）。
//!
//! 实测结构（F-303，S1.5b 修正 —— ⚠️ 旧记录「头部 24B 是字宽表」是错的）：
//! - `GoldLand.Fnt` 共 24384B = **1016 个字形 × 24B，无头部**（24384/24 = 1016 精确整除）；
//! - 每字形 = **12 行 × 2 字节**，像素 **MSB-first 在左**：`x in 0..8` 取字节 0 的
//!   `bit(7-x)`，`x in 8..12` 取字节 1 的 `bit(15-x)`；**字节 1 低 4 位恒为 0**
//!   （全文件 12192 行 0 违例，是填充位）→ 每行 12 像素；
//! - 行序**自上而下**（`A` 尖顶在第 1 行）、水平方向 **MSB 在左**（`b` 竖杆在左、
//!   `L` 脚衬线向右）—— 由 `tests/font_parity.rs` 几何断言实测验证；
//! - `font.dat` = UTF-16LE + BOM 的**字符集清单**（非字宽表），1016 个字符与字形
//!   按下标 **1:1** 对应；唯一字符 1015 个（`记` 出现两次 @981/@996，两处字形字节全同）。

use std::fmt::Write as _;

use crate::error::{Error, ParseKind, Result};

/// 每字形行数。
pub const GLYPH_ROWS: u16 = 12;

/// 每行字节数。
pub const ROW_BYTES: usize = 2;

/// 每字形字节数。
pub const GLYPH_BYTES: usize = GLYPH_ROWS as usize * ROW_BYTES;

/// 每行像素数（前 12 位有效，字节 1 低 4 位为填充）。
pub const GLYPH_WIDTH: usize = 12;

/// 解析后的字体文件（全部字形，无头部）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Font {
    /// 字形位图区（原始字节，逐字形等分）。
    pub glyphs: Vec<Vec<u8>>,
}

impl Font {
    /// 解析 `GoldLand.Fnt`。
    ///
    /// # Errors
    ///
    /// 文件长度不是 `GLYPH_BYTES` 整数倍时返回错误。
    pub fn parse(path: &str, buf: &[u8]) -> Result<Self> {
        if buf.len() % GLYPH_BYTES != 0 {
            return Err(Error::parse(
                path,
                buf.len() - buf.len() % GLYPH_BYTES,
                ParseKind::TrailingBytes { count: buf.len() % GLYPH_BYTES },
            ));
        }
        let glyphs = buf.chunks_exact(GLYPH_BYTES).map(<[u8]>::to_vec).collect();
        Ok(Self { glyphs })
    }

    /// 字形数量（应与 [`Charset::len`] 相等，实测 1016）。
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    /// 解码第 `index` 个字形为 12×12 像素行（bit `11-x` = 列 `x`，行序自上而下）。
    #[must_use]
    pub fn glyph(&self, index: usize) -> Option<Glyph> {
        let raw = self.glyphs.get(index)?;
        let mut rows = [0u16; GLYPH_ROWS as usize];
        for (r, pair) in raw.chunks_exact(ROW_BYTES).enumerate() {
            let Some(slot) = rows.get_mut(r) else {
                break;
            };
            *slot = (u16::from(pair[0]) << 4) | (u16::from(pair[1]) >> 4);
        }
        Some(Glyph { rows })
    }

    /// 按字符集查字符对应的字形（下标 1:1，取字符集首个出现位）。
    #[must_use]
    pub fn glyph_for(&self, charset: &Charset, ch: char) -> Option<Glyph> {
        charset.index_of(ch).and_then(|i| self.glyph(i))
    }

    /// 逐字节还原为原始文件内容（roundtrip 保证 0 未知字节）。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.glyphs.len() * GLYPH_BYTES);
        for g in &self.glyphs {
            out.extend_from_slice(g);
        }
        out
    }

    /// 导出 JSON（位图以十六进制串，便于逐位比对）。
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\n  \"glyph_width\": ");
        s.push_str(&GLYPH_WIDTH.to_string());
        s.push_str(",\n  \"glyph_rows\": ");
        s.push_str(&GLYPH_ROWS.to_string());
        s.push_str(",\n  \"glyph_count\": ");
        s.push_str(&self.glyph_count().to_string());
        s.push_str(",\n  \"glyphs\": [");
        for (i, g) in self.glyphs.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str("\n    \"");
            for b in g {
                let _ = write!(s, "{b:02x}");
            }
            s.push('"');
        }
        s.push_str("\n  ]\n}\n");
        s
    }
}

/// `font.dat` 字符集清单（UTF-16LE + BOM，与字形下标 1:1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charset {
    /// 清单字符（按出现顺序，含 1 处重复）。
    pub chars: Vec<char>,
}

impl Charset {
    /// 解析 `font.dat`（UTF-16LE，容忍 BOM）。
    ///
    /// # Errors
    ///
    /// 字节数为奇数（非完整 UTF-16 码元）或包含非法码元时返回错误。
    pub fn parse(path: &str, buf: &[u8]) -> Result<Self> {
        if buf.len() % 2 != 0 {
            return Err(Error::parse(path, buf.len(), ParseKind::TrailingBytes { count: 1 }));
        }
        let units: Vec<u16> =
            buf.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        let text = String::from_utf16(&units).map_err(|_| {
            Error::parse(path, 0, ParseKind::InvalidValue { field: "font.dat utf-16 units" })
        })?;
        let mut chars: Vec<char> = text.chars().collect();
        if chars.first() == Some(&'\u{feff}') {
            chars.remove(0);
        }
        Ok(Self { chars })
    }

    /// 清单长度（应与 `Font::glyph_count` 相等，实测 1016）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// 清单是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// 字符在清单中的下标（取首个出现位；不在清单中返回 `None`）。
    #[must_use]
    pub fn index_of(&self, ch: char) -> Option<usize> {
        self.chars.iter().position(|&c| c == ch)
    }
}

/// 解码后的字形位图：`rows[y]` 的 bit `11-x` 为像素 `(x, y)`，行序自上而下。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    /// 12 行像素行（每行低 12 位有效）。
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

    /// 水平步进（右缘墨迹 + 1；**推测**运行时按此前进，无字宽表，见 F-303）。
    #[must_use]
    pub fn width(&self) -> usize {
        (0..GLYPH_WIDTH)
            .rev()
            .find(|&x| (0..GLYPH_ROWS as usize).any(|y| self.pixel(x, y)))
            .map_or(0, |x| x + 1)
    }
}
