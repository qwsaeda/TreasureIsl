//! 音频：通道式混音、BGM 循环、音量（WAV 头解析 + 可测的混音内核）。
//!
//! 原版用 Hekkus Sound System（F-104），也是通道模型，故直接对齐 `Audio` trait。
//! 本模块是**纯计算内核**（不依赖声卡，CI 可跑）；S18.8b 起 macroquad 输出后端退役，
//! 音频输出由 Bevy 壳（`bevy_audio`）承担（S18.8 口径 = 发货无声，双渲染期关）。

use crate::error::{Error, Result};

/// 采样率上限（防止畸形 WAV 撑爆内存）。
pub const MAX_SAMPLE_RATE: u32 = 192_000;

/// 单个声音素材（PCM 单声道/立体声，i16 采样）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    /// 采样率（Hz）。
    pub sample_rate: u32,
    /// 声道数（1 或 2）。
    pub channels: u16,
    /// 位深（8 或 16）。
    pub bits: u8,
    /// 交错 PCM 采样（统一为 i16；8bit 无符号输入已按 -128 居中）。
    pub samples: Vec<i16>,
}

impl Clip {
    /// 帧数（每声道的采样数）。
    #[must_use]
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels.max(1))
    }

    /// 时长（秒）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.frames() as f32 / self.sample_rate as f32
    }

    /// 取第 `frame` 帧、第 `ch` 声道的采样（越界返回 0）。
    #[must_use]
    pub fn sample_at(&self, frame: usize, ch: u16) -> i16 {
        let idx = frame * usize::from(self.channels) + usize::from(ch);
        self.samples.get(idx).copied().unwrap_or(0)
    }
}

/// 通道当前播放状态。
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelState {
    /// 通道是否活跃。
    pub active: bool,
    /// 音量（0..=255，255 为最大）。
    pub volume: u8,
    /// 当前播放位置（帧）。
    pub position: usize,
    /// 是否循环。
    pub looping: bool,
}

/// 混音器：管理多个通道并按帧输出单声道混合样本。
#[derive(Debug, Default)]
pub struct Mixer {
    channels: Vec<ChannelState>,
    clips: Vec<Option<Clip>>,
}

impl Mixer {
    /// 新建最多 `n` 个通道的混音器。
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            channels: vec![
                ChannelState { active: false, volume: 0, position: 0, looping: false };
                n
            ],
            clips: (0..n).map(|_| None).collect(),
        }
    }

    /// 通道总数。
    #[must_use]
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// 活跃通道数。
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.channels.iter().filter(|c| c.active).count()
    }

    /// 读取通道状态。
    #[must_use]
    pub fn state(&self, ch: usize) -> Option<&ChannelState> {
        self.channels.get(ch)
    }

    /// 在 `ch` 通道播放 `clip`；通道越界返回错误。
    ///
    /// # Errors
    ///
    /// 通道下标越界或 clip 无采样时返回错误。
    pub fn play(&mut self, ch: usize, clip: Clip, volume: u8, looping: bool) -> Result<()> {
        let (state, slot) = self.slots(ch)?;
        if clip.samples.is_empty() {
            return Err(Error::NotReady("audio clip is empty"));
        }
        *state = ChannelState { active: true, volume, position: 0, looping };
        *slot = Some(clip);
        Ok(())
    }

    /// 停止通道。
    ///
    /// # Errors
    ///
    /// 通道下标越界时返回错误。
    pub fn stop(&mut self, ch: usize) -> Result<()> {
        let (state, slot) = self.slots(ch)?;
        *state = ChannelState { active: false, volume: 0, position: 0, looping: false };
        *slot = None;
        Ok(())
    }

    /// 设置通道音量（0..=255）。
    ///
    /// # Errors
    ///
    /// 通道下标越界时返回错误。
    pub fn set_volume(&mut self, ch: usize, volume: u8) -> Result<()> {
        self.slots(ch)?.0.volume = volume;
        Ok(())
    }

    /// 推进 `frames` 帧并输出混合后的 i32 累加值（调用方按需裁剪/缩放）。
    ///
    /// 通道到末尾：非循环则自动停止；循环则回到开头。
    ///
    /// # Errors
    ///
    /// 通道下标越界时返回错误。
    pub fn render(&mut self, frames: usize, out: &mut Vec<i32>) -> Result<()> {
        if self.channels.is_empty() {
            return Err(Error::NotReady("no channels"));
        }
        out.clear();
        out.resize(frames, 0);
        for f in 0..frames {
            let mut acc = 0i32;
            for ch in 0..self.channels.len() {
                let Some(state) = self.channels.get(ch) else { continue };
                if !state.active {
                    continue;
                }
                let Some(clip) = self.clips.get(ch).and_then(|c| c.as_ref()) else {
                    continue;
                };
                let total = clip.frames();
                if total == 0 {
                    continue;
                }
                let pos = state.position.min(total.saturating_sub(1));
                let s = clip.sample_at(pos, 0);
                acc += i32::from(s) * i32::from(state.volume) / 255;
            }
            if let Some(v) = out.get_mut(f) {
                *v = acc;
            }
            self.advance(frames, f);
        }
        Ok(())
    }

    /// 每渲染一帧统一推进各通道游标。
    fn advance(&mut self, _frames: usize, _f: usize) {
        let mut finished: Vec<usize> = Vec::new();
        for ch in 0..self.channels.len() {
            let Some(state) = self.channels.get_mut(ch) else { continue };
            if !state.active {
                continue;
            }
            let total = self.clips.get(ch).and_then(|c| c.as_ref()).map_or(0, Clip::frames);
            if total == 0 {
                continue;
            }
            state.position += 1;
            if state.position >= total {
                if state.looping {
                    state.position = 0;
                } else {
                    finished.push(ch);
                }
            }
        }
        for ch in finished {
            let _ = self.stop(ch);
        }
    }

    fn slots(&mut self, ch: usize) -> Result<(&mut ChannelState, &mut Option<Clip>)> {
        let state =
            self.channels.get_mut(ch).ok_or(Error::NotReady("audio channel out of range"))?;
        let slot = self.clips.get_mut(ch).ok_or(Error::NotReady("audio channel out of range"))?;
        Ok((state, slot))
    }
}

/// 解析 RIFF/WAVE 头，返回 `(sample_rate, channels, bits_per_sample, data_len)`。
///
/// # Errors
///
/// 非 RIFF/WAVE、`fmt `/`data` 块缺失或字段非法时返回错误。
pub fn parse_wav_header(buf: &[u8]) -> Result<(u32, u16, u16, u32)> {
    let bad = |msg: &'static str| Error::Asset { path: "wav".into(), reason: msg };
    if buf.len() < 12 || &buf[0..4] != b"RIFF" || &buf[8..12] != b"WAVE" {
        return Err(bad("不是 RIFF/WAVE"));
    }
    let mut pos = 12usize;
    let mut fmt: Option<(u32, u16, u16)> = None;
    let mut data_len = 0u32;
    while pos + 8 <= buf.len() {
        let id = &buf[pos..pos + 4];
        let size = u32::from_le_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
        let body = pos + 8;
        if id == b"fmt " && body + 16 <= buf.len() {
            let channels = u16::from_le_bytes([buf[body + 2], buf[body + 3]]);
            let rate =
                u32::from_le_bytes([buf[body + 4], buf[body + 5], buf[body + 6], buf[body + 7]]);
            let bits = u16::from_le_bytes([buf[body + 14], buf[body + 15]]);
            fmt = Some((rate, channels, bits));
        } else if id == b"data" {
            data_len = size;
        }
        pos = body + (size as usize) + usize::try_from(size % 2).unwrap_or_default();
    }
    let (rate, channels, bits) = fmt.ok_or_else(|| bad("缺少 fmt 块"))?;
    if rate == 0 || rate > MAX_SAMPLE_RATE {
        return Err(bad("采样率非法"));
    }
    if channels == 0 || channels > 2 {
        return Err(bad("声道数非法"));
    }
    if bits != 8 && bits != 16 {
        return Err(bad("仅支持 8/16bit"));
    }
    Ok((rate, channels, bits, data_len))
}

/// 把 i16 PCM 字节流解析为 [`Clip`]。
///
/// # Errors
///
/// WAV 头非法或数据长度非偶数时返回错误。
pub fn parse_wav(buf: &[u8]) -> Result<Clip> {
    let (rate, channels, bits, data_len) = parse_wav_header(buf)?;
    let end = usize::try_from(data_len)
        .ok()
        .and_then(|n| n.checked_add(44))
        .map_or(buf.len(), |n| n.min(buf.len()));
    let body = buf
        .get(44..end)
        .ok_or_else(|| Error::Asset { path: "wav".into(), reason: "data 块越界" })?;
    // 8bit 为无符号（0..=255，静音点 128），统一转成 i16 并居中
    let samples: Vec<i16> = if bits == 8 {
        body.iter().map(|b| i16::from(*b) - 128).collect()
    } else {
        if body.len() % 2 != 0 {
            return Err(Error::Asset { path: "wav".into(), reason: "data 长度为奇数" });
        }
        body.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
    };
    Ok(Clip { sample_rate: rate, channels, samples, bits: u8::try_from(bits).unwrap_or(16) })
}

/// 裁剪并归一化混合结果为 i16（防爆音：`clamp` + 峰值归一）。
#[must_use]
pub fn to_i16_clipped(mixed: &[i32], peak: i32) -> Vec<i16> {
    mixed
        .iter()
        .map(|v| {
            // 用 i64 做比例运算，避免整数除法把 scale 截断成 0
            let scaled = if peak > i32::from(i16::MAX) {
                i32::try_from(i64::from(*v) * i64::from(i16::MAX) / i64::from(peak))
                    .unwrap_or(i32::from(i16::MAX))
            } else {
                *v
            };
            i16::try_from(scaled.clamp(i32::from(i16::MIN), i32::from(i16::MAX)))
                .unwrap_or(i16::MAX)
        })
        .collect()
}
