//! 音频混音内核单测（S2.3）——置于集成测试目录，豁免门禁的 unwrap/expect 扫描。

use std::path::{Path, PathBuf};

use ti_formats::lucency::Package;
use ti_platform::audio::{Clip, Mixer, parse_wav, to_i16_clipped};

fn tone(frames: usize, value: i16) -> Clip {
    Clip { sample_rate: 8_000, channels: 1, bits: 16, samples: vec![value; frames] }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[test]
fn play_stop_channel_lifecycle() {
    let mut m = Mixer::new(4);
    assert_eq!(m.channel_count(), 4);
    assert_eq!(m.active_count(), 0);
    m.play(0, tone(100, 1000), 255, false).expect("播放成功");
    assert_eq!(m.active_count(), 1);
    assert_eq!(m.state(0).map(|s| s.volume), Some(255));
    m.stop(0).expect("停止成功");
    assert_eq!(m.active_count(), 0);
}

#[test]
fn out_of_range_channel_errors() {
    let mut m = Mixer::new(2);
    assert!(m.play(5, tone(4, 1), 128, false).is_err());
    assert!(m.set_volume(9, 100).is_err());
    assert!(m.stop(9).is_err());
}

#[test]
fn empty_clip_is_rejected() {
    let mut m = Mixer::new(1);
    let empty = Clip { sample_rate: 8000, channels: 1, bits: 16, samples: Vec::new() };
    assert!(m.play(0, empty, 255, false).is_err());
}

#[test]
fn volume_scales_output() {
    let mut m = Mixer::new(1);
    m.play(0, tone(4, 1000), 255, false).expect("播放成功");
    let mut out = Vec::new();
    m.render(4, &mut out).expect("渲染成功");
    assert_eq!(out[0], 1000);

    let mut m2 = Mixer::new(1);
    m2.play(0, tone(4, 1000), 128, false).expect("播放成功");
    let mut out2 = Vec::new();
    m2.render(4, &mut out2).expect("渲染成功");
    assert_eq!(out2[0], 1000 * 128 / 255);
}

#[test]
fn multiple_channels_sum() {
    let mut m = Mixer::new(2);
    m.play(0, tone(4, 100), 255, false).expect("ok");
    m.play(1, tone(4, 200), 255, false).expect("ok");
    let mut out = Vec::new();
    m.render(2, &mut out).expect("渲染成功");
    assert_eq!(out[0], 300);
}

#[test]
fn non_looping_channel_auto_stops_at_end() {
    let mut m = Mixer::new(1);
    m.play(0, tone(3, 500), 255, false).expect("ok");
    let mut out = Vec::new();
    m.render(5, &mut out).expect("渲染成功");
    assert_eq!(m.active_count(), 0, "非循环通道播完应自动停止");
}

#[test]
fn looping_channel_wraps_to_zero() {
    let mut m = Mixer::new(1);
    m.play(0, tone(3, 500), 255, true).expect("ok");
    let mut out = Vec::new();
    m.render(6, &mut out).expect("渲染成功");
    assert_eq!(m.active_count(), 1, "循环通道应保持活跃");
    assert!(m.state(0).is_some_and(|s| s.position < 3), "游标应回绕");
    assert!(out.iter().all(|v| *v == 500), "循环期间应持续有声音");
}

#[test]
fn clipping_prevents_overshoot() {
    let clamp = to_i16_clipped(&[50_000, -50_000], 32_767);
    assert_eq!(clamp, vec![i16::MAX, i16::MIN], "超出 i16 范围应被硬裁剪");
    let norm = to_i16_clipped(&[40_000, -40_000], 40_000);
    assert_eq!(norm, vec![i16::MAX, -i16::MAX], "超峰值应对称归一");
    let small = to_i16_clipped(&[100], 32_767);
    assert_eq!(small[0], 100, "未超峰值时不应改动");
}

#[test]
fn parse_wav_rejects_garbage() {
    assert!(parse_wav(&[0u8; 8]).is_err());
    let mut hdr = b"RIFF\x00\x00\x00\x00WAVE".to_vec();
    hdr.extend_from_slice(&[0u8; 64]);
    assert!(parse_wav(&hdr).is_err(), "缺少 fmt 块应报错");
}

#[test]
fn parses_music_lpk_wav_entry() {
    let buf = std::fs::read(repo_root().join("legacy").join("data").join("music.lpk"))
        .expect("music.lpk 存在");
    let pkg = Package::parse("music.lpk", &buf).expect("容器解析成功");
    let wav = pkg.read_entry(&buf, 0).expect("首条解压成功");
    let clip = parse_wav(&wav).expect("WAV 解析成功");
    assert_eq!(clip.sample_rate, 8_000, "F-208：8kHz");
    assert_eq!(clip.bits, 8, "F-208：8bit");
    assert_ne!(clip.samples, [] as [i16; 0]);
}

#[test]
fn duration_and_frame_helpers() {
    let c = tone(8000, 0);
    assert_eq!(c.frames(), 8000);
    assert!((c.duration_secs() - 1.0).abs() < 0.01, "8000 帧 @8kHz = 1 秒");
    assert_eq!(c.sample_at(10, 0), 0);
    assert_eq!(c.sample_at(usize::MAX, 0), 0, "越界返回 0");
}
