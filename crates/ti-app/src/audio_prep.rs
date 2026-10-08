//! `music.lpk` 音效准备管线（S2.5 补实）：解包 → `parse_wav` → [`Mixer`]
//! → 混音 PCM → 可交给 macroquad 的 WAV 字节。
//!
//! 纯函数、无 IO、不碰窗口/声卡 → `tests/audio_prep.rs` 对全链路自动断言。
//!
//! 为什么终点是 WAV 字节：macroquad 0.4.16 的音频输出只有
//! `audio::load_sound_from_bytes(&[u8])`（源码 `macroquad-0.4.16/src/audio.rs`），
//! 其后端 quad-snd 在 wasm 上把字节交给 `decodeAudioData`（`quad-snd-0.2.0/
//! src/web_snd.rs` + `js/audio.js`），**不提供 PCM 推流接口**；因此
//! [`Mixer`] 渲染出的 PCM 必须重新打包成 WAV —— 也就是说浏览器里真正
//! 出声的内容就是本模块的混音输出，混音器在真实输出链路上。

use ti_formats::Package;
use ti_platform::audio::{Mixer, parse_wav, to_i16_clipped};
use ti_platform::audio_asset::{Asset, Kind};

use crate::Result;

/// 混音音量（0..=255）：255 = 原样混音，响度交给播放端的 `PlaySoundParams`。
pub const MIX_VOLUME: u8 = 255;

/// `fmt ` 块长度（PCM 固定 16 字节）。
const FMT_CHUNK_LEN: u32 = 16;
/// WAV 格式号 = 1（未压缩 PCM）。
const FORMAT_PCM: u16 = 1;
/// 单声道。
const CHANNELS_MONO: u16 = 1;
/// 每帧字节数（单声道 × 16bit = 2）。
const BLOCK_ALIGN: u16 = 2;
/// 位深（打包输出固定 16bit）。
const BITS_PER_SAMPLE: u16 = 16;
/// WAV 头固定开销（`RIFF`/`WAVE`/`fmt `/`data` 共 44 字节）。
const WAV_HEADER_BYTES: usize = 44;
/// i16 采样占 2 字节。
const BYTES_PER_SAMPLE: usize = 2;

/// 混音后的单条曲目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedTrack {
    /// `music.lpk` 条目 key（容器索引记录的 `Entry::key`）。
    pub key: u32,
    /// 原 WAV 采样率（Hz）。
    pub sample_rate: u32,
    /// 混音输出帧数（单声道采样数）。
    pub frames: usize,
    /// 混音输出 PCM（i16 单声道；`MIX_VOLUME` 下与原 PCM 逐样本一致）。
    pub pcm: Vec<i16>,
}

impl MixedTrack {
    /// 时长（秒）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // 帧数/采样率都是小整数，f32 精度足够
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.frames as f32 / self.sample_rate as f32
    }
}

/// `music.lpk` 准备结果。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MusicBundle {
    /// 成功过混音器的 WAV 曲目（保持条目顺序）。
    pub tracks: Vec<MixedTrack>,
    /// 识别为 MOD 的条目数（原版 BGM，播放器属 S1.7b，当前未接入）。
    pub mod_count: usize,
    /// 识别为其他二进制的条目数。
    pub binary_count: usize,
    /// 未能进入混音的条目数（解压失败 / 非法 WAV / 非单声道 / 空采样）。
    pub skipped: usize,
}

/// 解包 `music.lpk` 并把每条 WAV 曲目过一遍 [`Mixer`]（`looping = true`）。
///
/// 每条曲目单独占一个通道、渲染其完整长度（恰好一遍循环）后停掉，
/// 因此 [`MixedTrack::pcm`] 是**该曲目单独**的混音结果，可直接作为独立音轨播放。
///
/// # Errors
///
/// 容器索引无法解析时返回错误；单条条目失败只计入 [`MusicBundle::skipped`]，不中断。
pub fn prepare_music(buf: &[u8]) -> Result<MusicBundle> {
    let pkg = Package::parse("music.lpk", buf)?;
    let mut mixer = Mixer::new(pkg.entries.len().max(1));
    let mut out = MusicBundle::default();
    let mut accum: Vec<i32> = Vec::new();
    let mut channel = 0usize;
    for (index, entry) in pkg.entries.iter().enumerate() {
        let Ok(raw) = pkg.read_entry(buf, index) else {
            out.skipped += 1;
            continue;
        };
        let asset = Asset::detect(&raw);
        match asset.kind {
            Kind::Mod => out.mod_count += 1,
            Kind::Binary => out.binary_count += 1,
            Kind::Wav => {
                let Ok(clip) = parse_wav(&raw) else {
                    out.skipped += 1;
                    continue;
                };
                // Mixer 只混第 0 声道，立体声直接丢右声道 = 静默丢数据，故跳过
                if clip.channels != CHANNELS_MONO || clip.frames() == 0 || clip.sample_rate == 0 {
                    out.skipped += 1;
                    continue;
                }
                if channel >= mixer.channel_count() {
                    out.skipped += 1;
                    continue;
                }
                let frames = clip.frames();
                let sample_rate = clip.sample_rate;
                if mixer.play(channel, clip, MIX_VOLUME, true).is_err() {
                    out.skipped += 1;
                    continue;
                }
                let rendered = mixer.render(frames, &mut accum);
                let _ = mixer.stop(channel);
                channel += 1;
                if rendered.is_err() {
                    out.skipped += 1;
                    continue;
                }
                let pcm = to_i16_clipped(&accum, i32::from(i16::MAX));
                out.tracks.push(MixedTrack { key: entry.key, sample_rate, frames: pcm.len(), pcm });
            }
        }
    }
    Ok(out)
}

/// 把 i16 单声道 PCM 打包成 16bit RIFF/WAVE（标准 44 字节头）。
///
/// 产出的字节直接喂给 `macroquad::audio::load_sound_from_bytes`。
#[must_use]
pub fn wav_pcm16(sample_rate: u32, pcm: &[i16]) -> Vec<u8> {
    let data_bytes = pcm.len().saturating_mul(BYTES_PER_SAMPLE);
    let data_len = u32::try_from(data_bytes).unwrap_or(u32::MAX);
    let riff_len = u32::try_from(data_bytes.saturating_add(36)).unwrap_or(u32::MAX);
    let mut out = Vec::with_capacity(WAV_HEADER_BYTES + data_bytes);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&FMT_CHUNK_LEN.to_le_bytes());
    out.extend_from_slice(&FORMAT_PCM.to_le_bytes());
    out.extend_from_slice(&CHANNELS_MONO.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate.saturating_mul(u32::from(BLOCK_ALIGN));
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&BLOCK_ALIGN.to_le_bytes());
    out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in pcm {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}
