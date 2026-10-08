//! `LucencyPackage`（`*.pak` / `*.lpk`）资源容器格式（S1.1 实测规格）。
//!
//! 完整规格见 `docs/formats/lucencypackage.md`。要点：
//! - 头部 0x36 字节固定，之后是索引流、数据区；
//! - 头部字段**非 4 字节对齐**：条目数 `@0x26`、索引长度 `@0x2E`、数据区起点 `@0x32`；
//! - 版本号 `@0x1A`：`<= 2004`（`.pak`）索引为 zlib 流，`>= 2005`（`.lpk`）为原始记录；
//! - 索引记录 16 字节 `{key, off, usize, csize}`，`off` 相对数据区起点；
//! - `key = name_key(name)` = **小写化后的 djb2（种子 0，`h = h*33 + c`）**。

use std::io::Read;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::error::{Error, ParseKind, Result};

/// 文件魔数（16 字节）。
pub const MAGIC: &[u8; 16] = b"LucencyPackage\0\0";

/// 头部长度：索引流固定从偏移 `0x36` 开始。
pub const HEADER_LEN: usize = 0x36;

/// 版本号 >= 该值的容器**通常**为 `.lpk`（但与索引压缩无关，实测 `glpics.pak` 亦为 2005）。
pub const VERSION_LPK: u16 = 2005;

/// 判断一段字节是否为合法 zlib 流头（CM=deflate 且 FCHECK 正确）。
#[must_use]
pub fn is_zlib(bytes: &[u8]) -> bool {
    let Some(head) = bytes.get(..2) else {
        return false;
    };
    if head[0] & 0x0F != 8 {
        return false;
    }
    let be = (u16::from(head[0]) << 8) | u16::from(head[1]);
    be % 31 == 0
}

/// 索引记录长度（字节）。
pub const RECORD_LEN: usize = 16;

/// 资源名 → 容器内 key：`h = 0; for c in name.to_ascii_lowercase() { h = h*33 + c }`。
///
/// 这是原版唯一的资源查找键函数（实测 7 个容器 100% 命中，见 S1.1）。
#[must_use]
pub fn name_key(name: &str) -> u32 {
    let mut h: u32 = 0;
    for byte in name.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(byte.to_ascii_lowercase()));
    }
    h
}

/// 索引里的一条资源记录。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// 资源名哈希（见 [`name_key`]）。
    pub key: u32,
    /// 数据区内的偏移（相对 `data_start`）。
    pub off: u32,
    /// 解压后长度。
    pub len: u32,
    /// 压缩后长度。
    pub clen: u32,
}

/// 已解析的容器（仅头部 + 索引；数据区按需解压）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// 格式版本（实测 2004 / 2005，与索引压缩无关）。
    pub version: u16,
    /// 索引是否为 zlib 压缩（`.pak`=true，`.lpk`=false，由 zlib 魔数判定）。
    pub index_compressed: bool,
    /// 数据区起点（文件绝对偏移）。
    pub data_start: usize,
    /// 索引记录。
    pub entries: Vec<Entry>,
}

impl Package {
    /// 按 key 查找资源下标。
    #[must_use]
    pub fn find(&self, name: &str) -> Option<usize> {
        let key = name_key(name);
        self.entries.iter().position(|e| e.key == key)
    }

    /// 解压第 `index` 条资源的内容。
    ///
    /// # Errors
    ///
    /// 下标越界、压缩流越界或 zlib 解压失败时返回错误（偏移已换算成文件绝对位置）。
    pub fn read_entry(&self, buf: &[u8], index: usize) -> Result<Vec<u8>> {
        let entry = self.entries.get(index).copied().ok_or_else(|| {
            Error::parse("package", index, ParseKind::InvalidValue { field: "entry index" })
        })?;
        let start = self.data_start.checked_add(entry.off as usize).ok_or_else(|| {
            Error::parse(
                "package",
                entry.off as usize,
                ParseKind::UnexpectedEof { field: "data", needed: 1 },
            )
        })?;
        let end = start.checked_add(entry.clen as usize).ok_or_else(|| {
            Error::parse("package", start, ParseKind::UnexpectedEof { field: "data", needed: 1 })
        })?;
        let slice = buf.get(start..end).ok_or_else(|| {
            Error::parse(
                "package",
                start,
                ParseKind::UnexpectedEof { field: "data", needed: end.saturating_sub(buf.len()) },
            )
        })?;
        let mut out = Vec::with_capacity(entry.len as usize);
        ZlibDecoder::new(slice).read_to_end(&mut out).map_err(|_| {
            Error::parse("package", start, ParseKind::TrailingBytes { count: entry.clen as usize })
        })?;
        Ok(out)
    }

    /// 由 [`Package::entries`] 重新编码索引流（原始记录或 zlib，按 `index_compressed`）。
    ///
    /// 对 `.lpk`（原始索引）实测与原文件索引**逐字节一致**，因此本方法同时充当
    /// 「索引记录布局正确」的证明；对 `.pak` 因原版 zlib 版本/参数未知，压缩流无法复现。
    ///
    /// # Errors
    ///
    /// zlib 写入/收尾失败时返回错误。
    pub fn encode_index(&self) -> Result<Vec<u8>> {
        let mut index = Vec::with_capacity(self.entries.len() * RECORD_LEN);
        for e in &self.entries {
            index.extend_from_slice(&e.key.to_le_bytes());
            index.extend_from_slice(&e.off.to_le_bytes());
            index.extend_from_slice(&e.len.to_le_bytes());
            index.extend_from_slice(&e.clen.to_le_bytes());
        }
        if !self.index_compressed {
            return Ok(index);
        }
        let mut enc = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        {
            use std::io::Write as _;
            enc.write_all(&index).map_err(|_| {
                Error::parse("package", HEADER_LEN, ParseKind::TrailingBytes { count: 0 })
            })?;
        }
        enc.finish()
            .map_err(|_| Error::parse("package", HEADER_LEN, ParseKind::TrailingBytes { count: 0 }))
    }

    /// 重建容器字节：头部 + 索引 + 数据区。
    ///
    /// 头部的 `条目数 / 索引长度 / 数据区起点` 三个字段由已解析的 [`Package`]
    /// **重新计算写回**（证明偏移理解正确）；索引与数据区（各条目的 zlib 压缩流）
    /// 逐字节原样复制 —— 原版压缩器参数未知，重新压缩无法保证字节一致。
    ///
    /// # Errors
    ///
    /// 源缓冲区越界（头部/索引/数据区）时返回错误。
    pub fn repack(&self, src: &[u8]) -> Result<Vec<u8>> {
        let header = src.get(..HEADER_LEN).ok_or_else(|| {
            Error::parse(
                "package",
                0,
                ParseKind::UnexpectedEof { field: "header", needed: HEADER_LEN },
            )
        })?;
        let index_src = src.get(HEADER_LEN..self.data_start).ok_or_else(|| {
            Error::parse(
                "package",
                HEADER_LEN,
                ParseKind::UnexpectedEof { field: "index", needed: 1 },
            )
        })?;
        let data = src.get(self.data_start..).ok_or_else(|| {
            Error::parse(
                "package",
                self.data_start,
                ParseKind::UnexpectedEof { field: "data", needed: 1 },
            )
        })?;

        let mut out = Vec::with_capacity(src.len());
        out.extend_from_slice(header);
        let count = u32::try_from(self.entries.len()).unwrap_or(u32::MAX);
        let idx_len = u32::try_from(index_src.len()).unwrap_or(u32::MAX);
        let data_start = u32::try_from(self.data_start).unwrap_or(u32::MAX);
        out[0x26..0x2A].copy_from_slice(&count.to_le_bytes());
        out[0x2E..0x32].copy_from_slice(&idx_len.to_le_bytes());
        out[0x32..0x36].copy_from_slice(&data_start.to_le_bytes());
        out.extend_from_slice(index_src);
        out.extend_from_slice(data);
        Ok(out)
    }

    /// 解析容器（头部 + 索引）。
    ///
    /// # Errors
    ///
    /// 魔数不符、字段越界、版本不支持、索引解压失败或索引长度与条目数不符时返回错误。
    pub fn parse(path: &str, buf: &[u8]) -> Result<Self> {
        let bad = |offset: usize, kind: ParseKind| Error::parse(path, offset, kind);

        let magic = buf.get(..MAGIC.len()).ok_or_else(|| {
            bad(0, ParseKind::UnexpectedEof { field: "magic", needed: MAGIC.len() })
        })?;
        if magic != MAGIC {
            return Err(bad(0, ParseKind::BadMagic { expected: MAGIC }));
        }
        let word = |off: usize| -> Result<u32> {
            let raw = buf
                .get(off..off + 4)
                .ok_or_else(|| bad(off, ParseKind::UnexpectedEof { field: "header", needed: 4 }))?;
            let mut b = [0u8; 4];
            b.copy_from_slice(raw);
            Ok(u32::from_le_bytes(b))
        };

        let version = u16::from_le_bytes([buf[0x1A], buf[0x1B]]);
        let entry_count = word(0x26)? as usize;
        let index_len = word(0x2E)? as usize;
        let data_start = word(0x32)? as usize;

        let raw_index = buf.get(HEADER_LEN..HEADER_LEN + index_len).ok_or_else(|| {
            bad(
                HEADER_LEN,
                ParseKind::UnexpectedEof {
                    field: "index",
                    needed: index_len.saturating_sub(buf.len().saturating_sub(HEADER_LEN)),
                },
            )
        })?;

        let decoded: Vec<u8> = if is_zlib(raw_index) {
            let mut out = Vec::new();
            ZlibDecoder::new(raw_index)
                .read_to_end(&mut out)
                .map_err(|_| bad(HEADER_LEN, ParseKind::TrailingBytes { count: index_len }))?;
            out
        } else {
            raw_index.to_vec()
        };

        if decoded.len() != entry_count * RECORD_LEN {
            return Err(bad(HEADER_LEN, ParseKind::TrailingBytes { count: decoded.len() }));
        }

        let mut entries = Vec::with_capacity(entry_count);
        for chunk in decoded.chunks_exact(RECORD_LEN) {
            let mut b = [0u8; RECORD_LEN];
            b.copy_from_slice(chunk);
            let key = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            let off = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
            let len = u32::from_le_bytes([b[8], b[9], b[10], b[11]]);
            let clen = u32::from_le_bytes([b[12], b[13], b[14], b[15]]);
            entries.push(Entry { key, off, len, clen });
        }

        Ok(Self { version, index_compressed: is_zlib(raw_index), data_start, entries })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::*;

    #[test]
    fn name_key_is_lowercase_djb2_seed_zero() {
        assert_eq!(name_key(""), 0);
        assert_eq!(name_key("a"), 97);
        assert_eq!(name_key("b"), 98);
        // 'A' 与 'a' 同 key
        assert_eq!(name_key("Dialogue.pcx"), name_key("dialogue.pcx"));
        // 大小写无关但区分不同名字
        assert_ne!(name_key("number.pcx"), name_key("target_area.pcx"));
    }

    #[test]
    fn name_key_matches_measured_keys() {
        // 实测样本（ui.pak / glpics.pak，S1.1）
        assert_eq!(name_key("landinfo.pcx"), 0x5C5A_F184);
        assert_eq!(name_key("number.pcx"), 0xFCFF_FB62);
        assert_eq!(name_key("target_area.pcx"), 0xE56B_BE98);
        assert_eq!(name_key("dialogue.pcx"), 0xFFFB_9A43);
        assert_eq!(name_key("logo.pcx"), 0x047D_426A);
    }

    #[allow(clippy::cast_possible_truncation)]
    fn build_pkg(version: u16, keys: &[u32]) -> Vec<u8> {
        let mut idx = Vec::new();
        let mut off = 0u32;
        for k in keys {
            idx.extend_from_slice(&k.to_le_bytes());
            idx.extend_from_slice(&off.to_le_bytes());
            idx.extend_from_slice(&10u32.to_le_bytes());
            idx.extend_from_slice(&5u32.to_le_bytes());
            off += 5;
        }
        let body: Vec<u8> = if version >= VERSION_LPK {
            idx
        } else {
            let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            let _ = e.write_all(&idx);
            e.finish().unwrap_or_default()
        };
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&[0u8; 0x36 - 16]);
        out[0x1A..0x1C].copy_from_slice(&version.to_le_bytes());
        out[0x26..0x2A].copy_from_slice(&(keys.len() as u32).to_le_bytes());
        out[0x2E..0x32].copy_from_slice(&(body.len() as u32).to_le_bytes());
        out[0x32..0x36].copy_from_slice(&((0x36 + body.len()) as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    #[test]
    fn parse_compressed_index_v2004() {
        let pkg = build_pkg(2004, &[0x2F5C_FA6A, 0x2F6F_12EB]);
        let parsed = Package::parse("t.pak", &pkg);
        assert!(parsed.is_ok(), "{parsed:?}");
        let p = parsed.ok();
        if let Some(p) = p {
            assert_eq!(p.version, 2004);
            assert_eq!(p.entries.len(), 2);
            assert_eq!(p.entries[0].key, 0x2F5C_FA6A);
            assert_eq!(p.entries[1].off, 5);
        }
    }

    #[test]
    fn parse_raw_index_v2005() {
        let pkg = build_pkg(2005, &[1, 2, 3]);
        let parsed = Package::parse("t.lpk", &pkg);
        assert!(parsed.is_ok());
        let p = parsed.ok();
        if let Some(p) = p {
            assert_eq!(p.version, 2005);
            assert_eq!(p.entries.len(), 3);
            assert_eq!(p.data_start, HEADER_LEN + 48);
        }
    }

    #[test]
    fn find_uses_name_key() {
        let pkg = build_pkg(2004, &[name_key("number.pcx")]);
        let parsed = Package::parse("t.pak", &pkg);
        let p = parsed.ok();
        if let Some(p) = p {
            assert_eq!(p.find("number.pcx"), Some(0));
            assert_eq!(p.find("Number.PCX"), Some(0));
            assert_eq!(p.find("missing.pcx"), None);
        }
    }

    #[test]
    fn bad_magic_is_rejected() {
        let mut pkg = build_pkg(2004, &[1]);
        pkg[0] = b'X';
        let err = Package::parse("t.pak", &pkg).err();
        let e = err;
        if let Some(e) = e {
            assert!(e.to_string().contains("bad magic"), "{e}");
        }
    }
}
