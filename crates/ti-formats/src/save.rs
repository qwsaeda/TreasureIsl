//! 存档/设置格式（S1.9）。
//!
//! 实测（F-304 + S1.9 复核）：
//! - `settings.dat` 固定 **8 字节**：`5D 4A 71 4A 2E 01 01 01`（磁盘存在，可往返）；
//! - `gl0~4.sav` / `gl.rec` 在原始资源中**不存在**，是运行时写出的存档，
//!   其字节布局须从 S3 存档逻辑 + oracle 反推，字段表待 S3.10 落地。
//!
//! 本模块交付：可往返的设置记录结构 + 版本化存档封套（SaveCodec）骨架。

use crate::error::{Error, ParseKind, Result};

/// `settings.dat` 魔数（实测 `5D 4A 71 4A`）。
pub const SETTINGS_MAGIC: [u8; 4] = [0x5D, 0x4A, 0x71, 0x4A];

/// `settings.dat` 总长度（实测 8）。
pub const SETTINGS_LEN: usize = 8;

/// 设置记录（对齐实测 8 字节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// 4 字节魔数（实测 `5D 4A 71 4A`）。
    pub magic: [u8; 4],
    /// 字节 4：实测 `0x2E`（语义待 S3.10）。
    pub flag: u8,
    /// 字节 5：实测 `0x01`（版本/开关）。
    pub version: u8,
    /// 字节 6：实测 `0x01`（音量/画质开关）。
    pub options: u8,
    /// 字节 7：实测 `0x01`（校验/保留）。
    pub checksum: u8,
}

impl Settings {
    /// 从 8 字节解析。
    ///
    /// # Errors
    ///
    /// 长度不等于 8 或魔数不符时返回错误。
    pub fn parse(path: &str, buf: &[u8]) -> Result<Self> {
        if buf.len() != SETTINGS_LEN {
            return Err(Error::parse(
                path,
                0,
                ParseKind::UnexpectedEof { field: "settings", needed: SETTINGS_LEN },
            ));
        }
        if buf[0..4] != SETTINGS_MAGIC {
            return Err(Error::parse(path, 0, ParseKind::BadMagic { expected: &SETTINGS_MAGIC }));
        }
        Ok(Self {
            magic: [buf[0], buf[1], buf[2], buf[3]],
            flag: buf[4],
            version: buf[5],
            options: buf[6],
            checksum: buf[7],
        })
    }

    /// 逐字节还原。
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref, clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.magic[0],
            self.magic[1],
            self.magic[2],
            self.magic[3],
            self.flag,
            self.version,
            self.options,
            self.checksum,
        ]
    }
}

/// 存档封套：版本化 + 负载长度 + 校验（S3.10 落地时补 v1 兼容读写）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveEnvelope {
    /// 存档版本（v1=原版，v2=新格式，S4.2）。
    pub version: u16,
    /// 负载字节。
    pub payload: Vec<u8>,
}

impl SaveEnvelope {
    /// 新建封套（默认 v1）。
    #[must_use]
    pub fn new(version: u16, payload: Vec<u8>) -> Self {
        Self { version, payload }
    }

    /// 序列化为 `magic(2) + version(2) + len(4) + payload`。
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(8 + self.payload.len());
        v.extend_from_slice(b"TS");
        v.extend_from_slice(&self.version.to_le_bytes());
        let len = u32::try_from(self.payload.len()).unwrap_or(u32::MAX);
        v.extend_from_slice(&len.to_le_bytes());
        v.extend_from_slice(&self.payload);
        v
    }

    /// 解析封套。
    ///
    /// # Errors
    ///
    /// 魔数不符、长度字段与实际不符时返回错误。
    pub fn decode(path: &str, buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 || &buf[0..2] != b"TS" {
            return Err(Error::parse(path, 0, ParseKind::BadMagic { expected: b"TS" }));
        }
        let version = u16::from_le_bytes([buf[2], buf[3]]);
        let len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
        let payload = buf.get(8..8 + len).ok_or_else(|| {
            Error::parse(path, 8, ParseKind::UnexpectedEof { field: "save payload", needed: len })
        })?;
        Ok(Self { version, payload: payload.to_vec() })
    }
}
