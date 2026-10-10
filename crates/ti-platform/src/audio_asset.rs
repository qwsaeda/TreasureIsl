//! 音频素材解析（S1.7）。
//!
//! 实测（`data/music.lpk` 6 条）：
//! - 5 条 `RIFF/WAVE`（8000Hz/8bit 单声道等）；
//! - 1 条 **MOD 音乐**（`glmusic.mod`，key `0x8046_15C2`，S6.5 业务名），头部为 **`INHL.MOD`** + 标题（偏移 `0x14`）
//!   `"I Need Her Love"`，**非标准 `ProTracker` 20 字节签名布局**，需按原版模块播放器解析。
//!
//! 交付：素材识别 + WAV 抽取 + MOD 头部规格 + PCM 归一化转储。
//! OGG/Vorbis 编码需外部编码器（S1.7b），本模块不做有损转码。

use crate::error::{Error, Result};

/// MOD 文件头签名（实测 `INHL.MOD`）。
pub const MOD_MAGIC: &[u8; 8] = b"INHL.MOD";

/// 音频素材类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// RIFF/WAVE。
    Wav,
    /// 原版模块音乐（`INHL.MOD`）。
    Mod,
    /// 其他二进制（音效表等）。
    Binary,
}

/// 素材识别结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// 素材类型。
    pub kind: Kind,
    /// MOD 标题（仅 `Kind::Mod`）。
    pub title: String,
    /// 原始字节。
    pub bytes: Vec<u8>,
}

impl Asset {
    /// 识别素材。
    #[must_use]
    pub fn detect(bytes: &[u8]) -> Self {
        if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
            return Self { kind: Kind::Wav, title: String::new(), bytes: bytes.to_vec() };
        }
        if bytes.len() >= 8 && &bytes[0..8] == MOD_MAGIC {
            // 标题为第一个 NUL 结尾的 ASCII 串（实测偏移 0x10）
            let start = 0x14.min(bytes.len());
            let end =
                bytes[start..].iter().position(|b| *b == 0).map_or(bytes.len(), |p| start + p);
            let title = String::from_utf8_lossy(&bytes[start..end]).trim().to_string();
            return Self { kind: Kind::Mod, title, bytes: bytes.to_vec() };
        }
        Self { kind: Kind::Binary, title: String::new(), bytes: bytes.to_vec() }
    }

    /// 按 WAV 解析（仅 `Kind::Wav`）。
    ///
    /// # Errors
    ///
    /// 非 WAV 或头非法时返回错误。
    pub fn as_wav(&self) -> Result<crate::audio::Clip> {
        if self.kind != Kind::Wav {
            return Err(Error::NotReady("not a RIFF/WAVE asset"));
        }
        crate::audio::parse_wav(&self.bytes)
    }

    /// 转储为 8bit 单声道 PCM WAV（统一采样率），供后续编码器消费。
    ///
    /// # Errors
    ///
    /// 非 WAV 或转换失败时返回错误。
    pub fn to_pcm_wav(&self, target_rate: u32) -> Result<Vec<u8>> {
        let clip = self.as_wav()?;
        if target_rate == 0 {
            return Err(Error::NotReady("target rate must be > 0"));
        }
        // 最近邻重采样（素材本就是低采样率，此处只做整数倍/整数除）
        let frames = clip.frames();
        let step = clip.sample_rate.max(1) as usize;
        let out_frames = if target_rate == clip.sample_rate {
            frames
        } else {
            frames * target_rate as usize / step
        };
        let mut data = Vec::with_capacity(out_frames * 2);
        for i in 0..out_frames {
            let src = if out_frames == 0 { 0 } else { i * step / target_rate as usize };
            let s = clip.sample_at(src.min(frames.saturating_sub(1)), 0);
            // 8bit WAV 为无符号：i16 [-32768,32767] → u8 [0,255]，偏移 +128
            let v = u8::try_from(i32::from(s) / 256 + 128).unwrap_or(128).clamp(0, 255);
            data.push(v);
        }
        Ok(build_wav_8bit(target_rate, &data))
    }
}

/// 组装 8bit 单声道 PCM WAV。
#[must_use]
pub fn build_wav_8bit(sample_rate: u32, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(44 + data.len());
    let riff_len = u32::try_from(36 + data.len()).unwrap_or(u32::MAX);
    let data_len = u32::try_from(data.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&8u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(data);
    out
}
