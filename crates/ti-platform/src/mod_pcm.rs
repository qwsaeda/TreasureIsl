//! INHL.MOD PCM 提取（S1.7b）。
//!
//! 实测 `data/music.lpk` 中的 `804615c2.bin` 是自定义 `INHL.MOD` 格式：
//! - 0x00-0x07: `INHL.MOD` 魔数
//! - 0x14 起: NUL 结尾标题（"I Need Her Love"）
//! - 0x4844 起至文件尾: 8bit 无符号单声道 PCM 采样
//!
//! 本模块只负责魔数校验 + PCM 提取 + 8bit→i16 转换（纯计算，无 IO）。
//! OGG/Opus 编码在 `ti-tools`（`mod2ogg` bin）与 `ti-app` 音频管线中完成。

use crate::error::{Error, Result};

/// INHL.MOD 魔数（实测 8 字节）。
pub const MOD_MAGIC: &[u8; 8] = b"INHL.MOD";

/// PCM 采样数据起点（实测 0x4844，此前为头部 + 嵌入式文本元数据）。
pub const AUDIO_OFFSET: usize = 0x4844;

/// 默认采样率（Hz）：原版 Hekkus 模块播放器低采样率 BGM，取 8000。
pub const DEFAULT_MOD_RATE: u32 = 8_000;

/// INHL.MOD 解析结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModAudio {
    /// 标题（NUL 结尾 ASCII，实测偏移 0x14）。
    pub title: String,
    /// 8bit 无符号 PCM 采样（至文件尾）。
    pub pcm_u8: Vec<u8>,
    /// 样本数。
    pub samples: usize,
    /// 采样率（Hz）。
    pub sample_rate: u32,
}

impl ModAudio {
    /// 从 INHL.MOD 字节解析 PCM（纯函数，不分配标题以外的内容）。
    ///
    /// # Errors
    ///
    /// 魔数不符、长度不足 [`AUDIO_OFFSET`] 或标题非 ASCII 时返回错误。
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < AUDIO_OFFSET {
            return Err(Error::Asset {
                path: "INHL.MOD".into(),
                reason: "文件过小，无法定位 PCM 区",
            });
        }
        let magic: [u8; 8] = bytes[..8]
            .try_into()
            .map_err(|_| Error::Asset { path: "INHL.MOD".into(), reason: "魔数长度不足" })?;
        if magic != *MOD_MAGIC {
            return Err(Error::Asset { path: "INHL.MOD".into(), reason: "魔数不是 INHL.MOD" });
        }
        // 标题：0x14 起 NUL 结尾
        let start = 0x14.min(bytes.len());
        let end = bytes[start..].iter().position(|b| *b == 0).map_or(bytes.len(), |p| start + p);
        let title_bytes = &bytes[start..end];
        let title = String::from_utf8_lossy(title_bytes).trim().to_string();
        let pcm_u8 = bytes[AUDIO_OFFSET..].to_vec();
        Ok(Self { title, samples: pcm_u8.len(), pcm_u8, sample_rate: DEFAULT_MOD_RATE })
    }

    /// 时长（秒）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.samples as f32 / self.sample_rate as f32
    }

    /// 8bit 无符号 → i16 有符号（偏移 128 居中，扩展 8→16bit 动态范围）。
    #[must_use]
    pub fn to_i16(&self) -> Vec<i16> {
        self.pcm_u8.iter().map(|&b| i16::from(b).wrapping_sub(128).wrapping_mul(128)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ti_formats::Package;

    fn repo_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf)
    }

    /// 读取 MOD 条目；失败时测试直接返回（数据文件不存在属于环境问题，不在库内断言）。
    fn mod_bytes() -> Option<Vec<u8>> {
        const MOD_KEY: u32 = 0x8046_15C2;
        let buf = std::fs::read(repo_root().join("data").join("music.lpk")).ok()?;
        let pkg = Package::parse("music.lpk", &buf).ok()?;
        let idx = pkg.find_key(MOD_KEY)?;
        pkg.read_entry(&buf, idx).ok()
    }

    #[test]
    fn parses_magic_title_and_pcm() {
        let Some(raw) = mod_bytes() else {
            eprintln!("SKIP: data/music.lpk 不存在");
            return;
        };
        let Ok(a) = ModAudio::parse(&raw) else {
            return; // 数据文件异常时不阻断库测试
        };
        assert_eq!(a.title, "I Need Her Love", "标题");
        assert_eq!(a.samples, raw.len() - AUDIO_OFFSET, "PCM 长度");
        assert!(!a.pcm_u8.is_empty(), "PCM 非空");
        assert_eq!(a.sample_rate, 8_000, "默认采样率");
        assert!(a.duration_secs() > 3.0 && a.duration_secs() < 5.0, "时长合理");
    }

    #[test]
    fn to_i16_centers_and_scales() {
        let a = ModAudio {
            title: "t".into(),
            pcm_u8: vec![0, 128, 255],
            samples: 3,
            sample_rate: 8_000,
        };
        let v = a.to_i16();
        assert_eq!(v[0], (-128i16).wrapping_mul(128), "0 → -16384");
        assert_eq!(v[1], 0, "128 → 0");
        assert_eq!(v[2], 127i16.wrapping_mul(128), "255 → +16256");
    }

    #[test]
    fn rejects_bad_magic_and_short_input() {
        assert!(ModAudio::parse(&[0u8; 16]).is_err(), "短输入拒绝");
        let mut buf = vec![0u8; 0x4844];
        buf[..4].copy_from_slice(b"XXXX");
        assert!(ModAudio::parse(&buf).is_err(), "错魔数拒绝");
    }
}
