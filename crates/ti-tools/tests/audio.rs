//! S2.3 验收：通道/音量/BGM 循环/防爆音（桌面播放部分在 S2.5 接入 macroquad 后人工验证）。

use std::path::{Path, PathBuf};

use ti_platform::audio::{Clip, Mixer, parse_wav, to_i16_clipped};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn tone(frames: usize, value: i16) -> Clip {
    Clip { sample_rate: 8_000, channels: 1, bits: 16, samples: vec![value; frames] }
}

#[test]
fn sixteen_channels_max_no_clipping() {
    // 16 路满音量同相叠加 = 16 × 32767，必须归一而不是溢出（爆音）
    let mut m = Mixer::new(16);
    for ch in 0..16 {
        m.play(ch, tone(64, i16::MAX), 255, false).expect("播放成功");
    }
    assert_eq!(m.active_count(), 16);
    let mut out = Vec::new();
    m.render(64, &mut out).expect("渲染成功");
    let peak = out.iter().fold(0i32, |acc, v| acc.max(*v));
    assert!(peak > i32::from(i16::MAX), "叠加应超峰值");
    let pcm = to_i16_clipped(&out, peak);
    assert!(pcm.iter().all(|v| i32::from(*v) <= i32::from(i16::MAX)), "归一后不得溢出");
    assert!(pcm.contains(&i16::MAX), "归一后应触顶但不削波");
}

#[test]
fn bgm_loops_forever_and_sfx_stops() {
    let mut m = Mixer::new(2);
    m.play(0, tone(10, 1000), 255, true).expect("BGM 循环播放");
    m.play(1, tone(5, 500), 255, false).expect("SFX 播放");
    let mut out = Vec::new();
    m.render(20, &mut out).expect("渲染成功");
    assert_eq!(m.state(0).map(|s| s.active), Some(true), "BGM 应持续循环");
    assert_eq!(m.state(1).map(|s| s.active), Some(false), "SFX 播完应停止");
    assert!(out.len() == 20);
}

#[test]
fn volume_zero_silences_channel() {
    let mut m = Mixer::new(1);
    m.play(0, tone(8, 20000), 0, false).expect("播放成功");
    let mut out = Vec::new();
    m.render(8, &mut out).expect("渲染成功");
    assert!(out.iter().all(|v| *v == 0), "音量 0 应完全静音");
}

#[test]
fn set_volume_takes_effect_immediately() {
    let mut m = Mixer::new(1);
    m.play(0, tone(64, 1000), 255, true).expect("播放成功");
    m.set_volume(0, 0).expect("设音量成功");
    let mut a = Vec::new();
    m.render(8, &mut a).expect("渲染成功");
    assert!(a.iter().all(|v| *v == 0));
    m.set_volume(0, 255).expect("设音量成功");
    let mut b = Vec::new();
    m.render(8, &mut b).expect("渲染成功");
    assert!(b.iter().all(|v| *v == 1000));
}

#[test]
fn real_music_lpk_clips_decode() {
    let root = repo_root();
    let buf = std::fs::read(root.join("data").join("music.lpk")).expect("music.lpk 存在");
    let pkg = ti_formats::lucency::Package::parse("music.lpk", &buf).expect("容器解析成功");
    assert_eq!(pkg.entries.len(), 6, "music.lpk 应有 6 条音轨");
    let mut wav_count = 0usize;
    for i in 0..pkg.entries.len() {
        let raw = pkg.read_entry(&buf, i).expect("解压成功");
        if &raw[..4] != b"RIFF" {
            continue; // glmusic.mod 非 WAV，归 S1.7（MOD→OGG）
        }
        let clip = parse_wav(&raw).expect("WAV 解析成功");
        assert_eq!(clip.channels, 1, "音轨 {i} 应为单声道");
        assert!(clip.duration_secs() > 0.0, "音轨 {i} 时长应 > 0");
        wav_count += 1;
    }
    assert_eq!(wav_count, 5, "music.lpk 应有 5 条 WAV + 1 条 MOD");

    // F-208：首条为 8kHz 8bit
    let first = pkg.read_entry(&buf, 0).expect("解压成功");
    let clip = parse_wav(&first).expect("解析成功");
    assert_eq!(clip.sample_rate, 8_000, "F-208：首条 8kHz");
    assert_eq!(clip.bits, 8, "F-208：首条 8bit");
}

#[test]
fn clipping_free_across_all_music_tracks() {
    // 把全部 WAV 音轨同时播放并整段渲染，确认无削波/爆音
    let root = repo_root();
    let buf = std::fs::read(root.join("data").join("music.lpk")).expect("music.lpk 存在");
    let pkg = ti_formats::lucency::Package::parse("music.lpk", &buf).expect("解析成功");
    let mut m = Mixer::new(8);
    let mut played = 0usize;
    for i in 0..pkg.entries.len() {
        let raw = pkg.read_entry(&buf, i).expect("解压成功");
        if &raw[..4] != b"RIFF" {
            continue;
        }
        let clip = parse_wav(&raw).expect("解析成功");
        m.play(played, clip, 255, false).expect("播放成功");
        played += 1;
    }
    assert_eq!(played, 5);
    let mut out = Vec::new();
    m.render(4000, &mut out).expect("渲染成功");
    let peak = out.iter().fold(0i32, |acc, v| acc.max(v.abs()));
    let pcm = to_i16_clipped(&out, peak);
    assert_eq!(pcm.len(), 4000);
    assert!(pcm.iter().all(|v| i32::from(*v) <= i32::from(i16::MAX)));
    eprintln!("[audio] 5 音轨同时播放 4000 帧，峰值 {peak}，归一后无削波");
}
