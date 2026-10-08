//! `.Map` 场景文件（S1.4）。
//!
//! 实测结构（`docs/reverse-facts.md` F-301/F-302）：
//! - 头部 16 字节：`width=0xB0(176)`、`height=0xDC(220)`、`flags`、`reserved`；
//! - 其后为 u32 字流（控件/场景记录），**文件内无任何字符串**（F-302）；
//! - 文件长度必为 `16 + 4k`，因此 roundtrip 可逐字节还原（0 未知字节）。
//!
//! 记录级布局已逆出（S1.11 / F-306：`words.len() == 5 + 6*count`，`count = words[4]`）；
//! 控件类型、事件 id 等字段语义待 S1.4b 结合 exe 事件表解析。

use crate::define_format;
use crate::error::{Error, ParseKind, Result};

define_format! {
    /// `.Map` 固定头部（16 字节）。
    pub struct MapHeader {
        /// 场景宽（UI 场景恒为 176，地形图为 20 —— F-301，故此处不写死）。
        pub width: u32,
        /// 场景高（UI 场景恒为 220，地形图为 20）。
        pub height: u32,
        /// 标志位（实测 0 或 1，语义待定）。
        pub flags: u32,
        /// 保留/变体字段（实测 0 或 0xFFFFFFFF，语义待定）。
        pub reserved: u32,
    }
}

/// 一个 `.Map` 场景文件：头部 + u32 字流。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapScene {
    /// 固定头部。
    pub header: MapHeader,
    /// 头部之后的 u32 字流（控件/场景记录，语义待逆）。
    pub words: Vec<u32>,
}

impl MapScene {
    /// 解析 `.Map`。
    ///
    /// # Errors
    ///
    /// 长度不足 16 字节、头部不匹配（宽高非 176/220）或剩余长度不是 4 的倍数时返回错误。
    pub fn parse(path: &str, buf: &[u8]) -> Result<Self> {
        let header = MapHeader::read(
            path,
            buf.get(..16).ok_or_else(|| {
                Error::parse(path, 0, ParseKind::UnexpectedEof { field: "map header", needed: 16 })
            })?,
        )?;
        let body = buf.get(16..).ok_or_else(|| {
            Error::parse(path, 16, ParseKind::UnexpectedEof { field: "map body", needed: 1 })
        })?;
        if body.len() % 4 != 0 {
            return Err(Error::parse(
                path,
                16 + body.len() - body.len() % 4,
                ParseKind::TrailingBytes { count: body.len() % 4 },
            ));
        }
        let mut words = Vec::with_capacity(body.len() / 4);
        for chunk in body.chunks_exact(4) {
            words.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Ok(Self { header, words })
    }

    /// 逐字节还原为原始文件内容（roundtrip 保证 0 未知字节）。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.words.len() * 4);
        out.extend_from_slice(&self.header.width.to_le_bytes());
        out.extend_from_slice(&self.header.height.to_le_bytes());
        out.extend_from_slice(&self.header.flags.to_le_bytes());
        out.extend_from_slice(&self.header.reserved.to_le_bytes());
        for w in &self.words {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }

    /// 导出为 JSON（纯 ASCII，无第三方依赖）。
    #[must_use]
    pub fn to_json(&self, name: &str) -> String {
        let mut s = String::with_capacity(64 + self.words.len() * 12);
        s.push_str("{\n  \"name\": \"");
        s.push_str(name);
        s.push_str("\",\n  \"width\": ");
        s.push_str(&self.header.width.to_string());
        s.push_str(",\n  \"height\": ");
        s.push_str(&self.header.height.to_string());
        s.push_str(",\n  \"flags\": ");
        s.push_str(&self.header.flags.to_string());
        s.push_str(",\n  \"reserved\": ");
        s.push_str(&self.header.reserved.to_string());
        s.push_str(",\n  \"words\": [");
        for (i, w) in self.words.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push(' ');
            s.push_str(&w.to_string());
        }
        s.push_str(" ]\n}\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&176u32.to_le_bytes());
        v.extend_from_slice(&220u32.to_le_bytes());
        v.extend_from_slice(&1u32.to_le_bytes());
        v.extend_from_slice(&114u32.to_le_bytes());
        for w in [31u32, 1, 105, 6, 12, 1] {
            v.extend_from_slice(&w.to_le_bytes());
        }
        v
    }

    #[test]
    fn parses_header_and_words() {
        let s = MapScene::parse("t.Map", &sample());
        assert!(s.is_ok());
        let m = s.ok();
        if let Some(m) = m {
            assert_eq!(m.header.width, 176);
            assert_eq!(m.header.height, 220);
            assert_eq!(m.header.flags, 1);
            assert_eq!(m.header.reserved, 114);
            assert_eq!(m.words.len(), 6);
            assert_eq!(m.words[0], 31);
        }
    }

    #[test]
    fn roundtrip_is_byte_identical() {
        let raw = sample();
        let m = MapScene::parse("t.Map", &raw);
        let m = m.ok();
        if let Some(m) = m {
            assert_eq!(m.to_bytes(), raw);
        }
    }

    #[test]
    fn preserves_non_ui_dimensions() {
        // 地形图 20×20（F-301）必须原样透传，不被 UI 场景约束拦截
        let mut raw = sample();
        raw[0..4].copy_from_slice(&20u32.to_le_bytes());
        raw[4..8].copy_from_slice(&20u32.to_le_bytes());
        let m = MapScene::parse("t.Map", &raw);
        assert!(m.is_ok());
        let m = m.ok();
        if let Some(m) = m {
            assert_eq!((m.header.width, m.header.height), (20, 20));
            assert_eq!(m.to_bytes(), raw);
        }
    }

    #[test]
    fn rejects_ragged_body() {
        let mut raw = sample();
        raw.push(0);
        assert!(MapScene::parse("t.Map", &raw).is_err());
    }

    #[test]
    fn json_contains_header_fields() {
        let m = MapScene::parse("t.Map", &sample());
        let m = m.ok();
        if let Some(m) = m {
            let j = m.to_json("t");
            assert!(j.contains("\"width\": 176"), "{j}");
            assert!(j.contains("\"height\": 220"), "{j}");
            assert!(j.contains("\"words\": [ 31, 1, 105"), "{j}");
        }
    }
}
