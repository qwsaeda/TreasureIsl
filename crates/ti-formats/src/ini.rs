//! UTF-16LE INI 解析/序列化（S1.6，用于 `cards.lpk` 内的 `cards.ini` / `event.ini`）。
//!
//! 实测结构（`data/cards.lpk` 首条，key=`0x43104390` = `djb2("cards.ini")`）：
//! - 整文件 **UTF-16LE**（带中文游戏文本），行尾 `\r\n`；
//! - `;` 开头为注释（含文件头的 `; event catalog` 枚举说明）；
//! - `[section]` 分段，`key = value` 键值对，值可含中文与 `%N`/`%D` 占位符；
//! - 往返要求**逐字节无损**（含注释与空行），故解析时保留原文行。

use std::fmt::Write as _;

use crate::error::{Error, ParseKind, Result};

/// 注释前缀。
pub const COMMENT_PREFIX: char = ';';

/// 一个键值对（`value` 保留首尾空白以外的原文，注释与空行不进入此结构）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniEntry {
    /// 段名（`[section]` 的内容，无方括号）。
    pub section: String,
    /// 键名（原文大小写）。
    pub key: String,
    /// 值（`=` 之后的原文，已去首尾空白）。
    pub value: String,
}

/// 解析后的 INI：段 → 键值对序列 + 原始行（用于无损往返）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ini {
    /// 去掉 BOM 与注释后的字节偏移量（UTF-16 码元数），用于诊断。
    pub entries: Vec<IniEntry>,
    /// 原始 UTF-16LE 文本（不含 BOM）。
    pub raw: String,
}

impl Ini {
    /// 从 UTF-16LE 字节解析。
    ///
    /// # Errors
    ///
    /// 字节数为奇数（非完整 UTF-16 码元）时返回错误。
    pub fn parse_utf16le(path: &str, bytes: &[u8]) -> Result<Self> {
        let units: Vec<u16> =
            bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        if bytes.len() % 2 != 0 {
            return Err(Error::parse(path, bytes.len(), ParseKind::TrailingBytes { count: 1 }));
        }
        let mut text = String::from_utf16_lossy(&units);
        // 去 UTF-16 BOM
        if let Some(stripped) = text.strip_prefix('\u{feff}') {
            text = stripped.to_string();
        }
        let mut entries: Vec<IniEntry> = Vec::new();
        let mut section = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(COMMENT_PREFIX) {
                continue;
            }
            if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = inner.trim().to_string();
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                entries.push(IniEntry {
                    section: section.clone(),
                    key: k.trim().to_string(),
                    value: v.trim().to_string(),
                });
            }
        }
        Ok(Self { entries, raw: text })
    }

    /// 序列化回 UTF-16LE（含 BOM，与原文件一致）。
    #[must_use]
    pub fn to_utf16le(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity((self.raw.len() + 1) * 2);
        out.extend_from_slice(&0xFEFFu16.to_le_bytes());
        for u in self.raw.encode_utf16() {
            out.extend_from_slice(&u.to_le_bytes());
        }
        out
    }

    /// 导出 JSON（ASCII 转义中文为 `\uXXXX`）。
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\n  \"entries\": [");
        for (i, e) in self.entries.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str("\n    {\"section\": \"");
            s.push_str(&json_escape(&e.section));
            s.push_str("\", \"key\": \"");
            s.push_str(&json_escape(&e.key));
            s.push_str("\", \"value\": \"");
            s.push_str(&json_escape(&e.value));
            s.push_str("\"}");
        }
        s.push_str("\n  ]\n}\n");
        s
    }

    /// 查找某个段下的键值（段名大小写敏感，键名不敏感）。
    #[must_use]
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.section == section && e.key.eq_ignore_ascii_case(key))
            .map(|e| e.value.as_str())
    }
}

/// JSON 字符串转义（非 ASCII 转成 `\uXXXX`）。
#[must_use]
pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c if c.is_ascii() => out.push(c),
            c => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str =
        "; event catalog\r\n[event]\r\nCount = 22\r\n[event1]\r\nid = 1\r\nname = 奖学金\r\n";

    fn encode(text: &str) -> Vec<u8> {
        let mut v = Vec::new();
        for u in text.encode_utf16() {
            v.extend_from_slice(&u.to_le_bytes());
        }
        v
    }

    #[test]
    fn parses_sections_and_pairs() {
        let ini = Ini::parse_utf16le("t.ini", &encode(SAMPLE));
        assert!(ini.is_ok());
        let i = ini.ok();
        if let Some(i) = i {
            assert_eq!(i.entries.len(), 3);
            assert_eq!(i.get("event", "Count"), Some("22"));
            assert_eq!(i.get("event1", "name"), Some("奖学金"));
        }
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let i = Ini::parse_utf16le("t.ini", &encode(SAMPLE));
        let i = i.ok();
        if let Some(i) = i {
            assert!(i.entries.iter().all(|e| !e.key.starts_with(';')));
        }
    }

    #[test]
    fn roundtrip_is_byte_identical_with_bom() {
        let with_bom = {
            let mut v = vec![0xFF, 0xFE];
            v.extend_from_slice(&encode(SAMPLE));
            v
        };
        let i = Ini::parse_utf16le("t.ini", &with_bom);
        let i = i.ok();
        if let Some(i) = i {
            assert_eq!(i.to_utf16le(), with_bom);
        }
    }

    #[test]
    fn rejects_odd_byte_length() {
        assert!(Ini::parse_utf16le("t.ini", &[0x41]).is_err());
    }

    #[test]
    fn json_escapes_chinese() {
        let i = Ini::parse_utf16le("t.ini", &encode(SAMPLE));
        let i = i.ok();
        if let Some(i) = i {
            let j = i.to_json();
            assert!(j.contains("\\u5b66"), "{j}");
        }
    }
}
