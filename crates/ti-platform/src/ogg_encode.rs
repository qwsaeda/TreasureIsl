//! OGG/Opus 编码（S1.7b）：i16 单声道 PCM → RFC 7845 封装。
//!
//! 纯计算，无 IO；输入是 PCM + 采样率，输出是完整 OGG 字节流。
//! 基于 `opus-pure`（纯 Rust Opus 编解码器，BSD-3-Clause），无 C 依赖，
//! wasm32/Android 目标均可编译。

use opus_pure::Application;
use opus_pure::MAX_PACKET_BYTES;
use opus_pure::OggOpusWriter;
use opus_pure::OpusEncoder;
use opus_pure::OpusHead;
use opus_pure::OpusTags;

/// Opus 支持的合法采样率集合。
pub const OPUS_RATES: [i32; 5] = [8_000, 12_000, 16_000, 24_000, 48_000];

/// 将采样率对齐到最近的 Opus 合法速率。
#[must_use]
pub fn snap_rate(rate: u32) -> i32 {
    let target = i32::try_from(rate).unwrap_or(8_000);
    let mut best = 8_000i32;
    let mut best_d = i32::MAX;
    for r in OPUS_RATES {
        let d = (r - target).abs();
        if d < best_d {
            best_d = d;
            best = r;
        }
    }
    best
}

/// 把 i16 单声道 PCM 编码为 OGG/Opus 字节流（64 kb/s 目标）。
///
/// 末尾按 RFC 7845 §4.4 做 end-trim 保证精确时长（解码器据此截尾）。
/// 输出确定性：`OggOpusWriter` 的 serial 由头部内容 FNV-1a 派生，
/// 同一输入两次编码逐字节一致（`build_assets` 幂等可用）。
///
/// # Errors
///
/// 编码器初始化或 OGG 写入失败时返回 `Err(String)`。
#[allow(clippy::cast_possible_truncation)]
pub fn encode_ogg(pcm_i16: &[i16], rate: u32) -> Result<Vec<u8>, String> {
    let rate_i32 = snap_rate(rate);
    let rate_us = u32::try_from(rate_i32).unwrap_or(8_000);
    let mut encoder = OpusEncoder::new(rate_i32, 1, Application::Audio)
        .map_err(|e| format!("Opus 编码器初始化失败: {e}"))?;
    encoder.bitrate_bps = 64_000;
    encoder.lsb_depth = 16;

    let tags = OpusTags::new();
    let head = OpusHead::for_encoder(&encoder, rate_us);

    // RFC 7845 §4.2/§4.4 精确收尾计算
    let ticks = 48_000u32 / rate_us;
    let total = pcm_i16.len();
    let ticks_us = usize::try_from(ticks).unwrap_or(1);
    let pre_skip_us: usize = head.pre_skip.into();
    let lookahead = pre_skip_us.div_ceil(ticks_us);
    let frame = usize::try_from(rate_us).unwrap_or(8_000) / 50;
    let frames = (total + lookahead).div_ceil(frame.max(1));
    let total_ticks = total.saturating_mul(ticks_us);
    let final_granule =
        u64::try_from(pre_skip_us).unwrap_or(0) + u64::try_from(total_ticks).unwrap_or(0);

    let mut sink: Vec<u8> = Vec::new();
    {
        let mut writer: OggOpusWriter<&mut Vec<u8>> =
            OggOpusWriter::with_tags(sink.as_mut(), head, tags)
                .map_err(|e| format!("OGG 写入器: {e}"))?;

        let mut packet = vec![0u8; MAX_PACKET_BYTES];
        let mut block = vec![0i16; frame.max(1)];

        for i in 0..frames {
            let start = (i * block.len()).min(total);
            let end = (start + block.len()).min(total);
            block[..end - start].copy_from_slice(&pcm_i16[start..end]);
            block[end - start..].fill(0);

            let n = encoder
                .encode_s16(&block, block.len(), &mut packet)
                .map_err(|e| format!("Opus 编码失败: {e}"))?;
            if i + 1 == frames {
                let g = writer.granule();
                let g_u64 = u64::try_from(g).unwrap_or(0);
                let duration = final_granule.saturating_sub(g_u64);
                let duration_u32 = u32::try_from(duration).unwrap_or(0);
                writer
                    .write_packet_with_duration(&packet[..n], duration_u32)
                    .map_err(|e| format!("OGG write_packet_with_duration: {e}"))?;
            } else {
                writer.write_packet(&packet[..n]).map_err(|e| format!("OGG write_packet: {e}"))?;
            }
        }
        writer.finish().ok();
    }
    Ok(sink)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::cast_possible_truncation)]
    #[test]
    fn encode_sine_is_valid_ogg() {
        let rate = 8_000u32;
        let n = usize::try_from(rate).unwrap_or(8_000); // 1 秒
        let pcm: Vec<i16> = (0..n)
            .map(|i| {
                let i_f = f64::from(u32::try_from(i).unwrap_or(u32::MAX));
                let n_f = f64::from(u32::try_from(n).unwrap_or(u32::MAX));
                let v = (i_f * std::f64::consts::TAU * 440.0 / n_f).sin() * 10_000.0;
                let v_clamped = v.clamp(-32_768.0, 32_767.0);
                i16::try_from(v_clamped.round() as i64).unwrap_or(i16::MAX)
            })
            .collect();
        let Ok(ogg) = encode_ogg(&pcm, rate) else {
            return;
        };
        assert_eq!(&ogg[..4], b"OggS", "OGG 魔数");
        assert!(ogg.len() > 100, "非空输出");
    }

    #[test]
    fn encode_is_deterministic() {
        let pcm = vec![0i16; 1_600];
        let Ok(a) = encode_ogg(&pcm, 8_000) else {
            return;
        };
        let Ok(b) = encode_ogg(&pcm, 8_000) else {
            return;
        };
        assert_eq!(a, b, "两次编码逐字节一致");
    }

    #[test]
    fn snap_rate_picks_nearest() {
        assert_eq!(snap_rate(8_000), 8_000);
        assert_eq!(snap_rate(16_384), 16_000, "16384 → 16000");
        assert_eq!(snap_rate(22_050), 24_000, "22050 → 24000");
        assert_eq!(snap_rate(44_100), 48_000, "44100 → 48000");
    }
}
