//! S2.5 补实验收：`music.lpk` → 解包 → `parse_wav` → `Mixer` → WAV 字节
//! 是**真实音频链路**（对应「假 2」：旧版只是 6 个编造的字符串，从未播放）。
//! S1.7b 追加：MOD 条目（INHL.MOD）→ PCM 提取 → OGG/Opus 转码（`opus-pure`），
//! `prepare_music` 产出 `ogg_tracks`，`macroquad::audio::load_sound_from_bytes` 直接可播。
//!
//! 断言覆盖：
//! 1. 6 条条目里 5 条 WAV + 1 条 MOD（F-208），key 与实测一致；
//! 2. `MIX_VOLUME = 255` 时混音输出与原 WAV 的 PCM **逐样本一致**（链路不丢数据）；
//! 3. 每条曲目非静音；
//! 4. `wav_pcm16` 的产物能被 `parse_wav` 原样读回（macroquad 后端就吃这个字节流）；
//! 5. 全链路确定性（两次准备结果完全相同）；
//! 6. S1.7b：MOD → OGG 转码产出 `OggTrack`，OGG 魔数 `OggS`，确定性。

use std::fs;
use std::path::{Path, PathBuf};

use ti_app::audio_prep::{MIX_VOLUME, prepare_music, wav_pcm16};
use ti_formats::Package;
use ti_platform::audio::parse_wav;

/// `music.lpk` 6 条条目的实测 key（S1.2 unpak manifest）。
const EXPECTED_KEYS: [u32; 6] =
    [0x1B8A_9203, 0x551F_F8B9, 0x77A3_E1D4, 0x8046_15C2, 0x9EF9_B7A9, 0xAE1A_80CC];
/// MOD 条目的 key（`INHL.MOD`，播放器属 S1.7b，当前不接入）。
const MOD_KEY: u32 = 0x8046_15C2;
/// 最长 WAV（壳用作循环 BGM）。
const LONGEST_WAV_KEY: u32 = 0xAE1A_80CC;
/// 最短 WAV（壳用作菜单确认音效）。
const SHORTEST_WAV_KEY: u32 = 0x9EF9_B7A9;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn music_bytes() -> Vec<u8> {
    fs::read(repo_root().join("data").join("music.lpk")).expect("music.lpk 存在")
}

#[test]
fn music_lpk_unpacks_five_wav_tracks_and_one_mod() {
    let bundle = prepare_music(&music_bytes()).expect("准备管线成功");
    assert_eq!(bundle.tracks.len(), 5, "5 条 WAV 应全部过混音器");
    assert_eq!(bundle.mod_count, 1, "1 条 MOD（F-208）");
    assert_eq!(bundle.binary_count, 0, "没有其他二进制条目");
    #[cfg(feature = "mod-ogg")]
    {
        assert_eq!(bundle.skipped, 0, "没有条目被跳过");
        // S1.7b：MOD 条目转 OGG 后应产生 1 条 OGG 曲目
        assert_eq!(bundle.ogg_tracks.len(), 1, "1 条 MOD 应转成 1 条 OGG");
        let ogg = &bundle.ogg_tracks[0];
        assert_eq!(ogg.key, MOD_KEY, "OGG 曲目 key 应为 MOD 条目");
        assert_eq!(&ogg.ogg[..4], b"OggS", "OGG 魔数");
        assert!(!ogg.ogg.is_empty(), "OGG 非空");
        assert!(!ogg.title.is_empty(), "INHL.MOD 标题非空");
    }
    #[cfg(not(feature = "mod-ogg"))]
    {
        assert_eq!(bundle.skipped, 1, "1 条 MOD 计跳过（在线转码未开启）");
        assert_eq!(bundle.ogg_tracks.len(), 0, "无 OGG 曲目（在线转码未开启）");
    }

    let keys: Vec<u32> = bundle.tracks.iter().map(|track| track.key).collect();
    let expected: Vec<u32> = EXPECTED_KEYS.iter().copied().filter(|key| *key != MOD_KEY).collect();
    assert_eq!(keys, expected, "WAV 曲目 key 顺序应与容器索引一致");

    for track in &bundle.tracks {
        assert!(track.frames > 0, "曲目 {:08x} 帧数应 > 0", track.key);
        assert!(track.sample_rate > 0, "曲目 {:08x} 采样率应 > 0", track.key);
        assert!(track.duration_secs() > 0.0, "曲目 {:08x} 时长应 > 0", track.key);
    }
    eprintln!(
        "[s2.5] 5 条 WAV 过 Mixer + 1 条 MOD：{}",
        bundle
            .tracks
            .iter()
            .map(|t| format!("{:08x}@{}Hz/{:.2}s", t.key, t.sample_rate, t.duration_secs()))
            .collect::<Vec<_>>()
            .join(" ")
    );
    #[cfg(feature = "mod-ogg")]
    eprintln!(
        "[s1.7b] OGG 曲目: {} @ {} bytes",
        bundle.ogg_tracks[0].title,
        bundle.ogg_tracks[0].ogg.len()
    );
}

#[test]
fn mixer_output_equals_original_pcm_at_full_volume() {
    let raw = music_bytes();
    let pkg = Package::parse("music.lpk", &raw).expect("容器解析成功");
    let bundle = prepare_music(&raw).expect("准备管线成功");
    assert_eq!(MIX_VOLUME, 255, "本断言基于满音量 = 原样混音");

    let mut global_peak = 0u16;
    for track in &bundle.tracks {
        let index =
            pkg.entries.iter().position(|entry| entry.key == track.key).expect("key 在索引内");
        let entry_raw = pkg.read_entry(&raw, index).expect("解压成功");
        let original = parse_wav(&entry_raw).expect("原 WAV 可解析");
        assert_eq!(track.sample_rate, original.sample_rate, "采样率不应被改写");
        assert_eq!(track.frames, original.frames(), "帧数不应被改写");
        assert_eq!(track.pcm, original.samples, "混音输出应与原 PCM 逐样本一致");
        assert!(track.pcm.iter().any(|&sample| sample != 0), "曲目 {:08x} 不能是静音", track.key);
        let peak =
            track.pcm.iter().map(|sample| sample.unsigned_abs()).max().expect("至少一个采样");
        global_peak = global_peak.max(peak);
    }
    assert!(global_peak >= 1000, "最响曲目峰值 {global_peak} 应达到可听水平");
    eprintln!("[s2.5] 混音输出 = 原 PCM 逐样本一致，全曲最响峰值 {global_peak}");
}

#[test]
fn wav_pcm16_round_trips_through_parse_wav() {
    let bundle = prepare_music(&music_bytes()).expect("准备管线成功");
    for track in &bundle.tracks {
        let wav = wav_pcm16(track.sample_rate, &track.pcm);
        assert_eq!(&wav[0..4], b"RIFF", "RIFF 魔数");
        assert_eq!(&wav[8..12], b"WAVE", "WAVE 标识");
        let back = parse_wav(&wav).expect("打包产物应可被 macroquad 后端同样的解析器读回");
        assert_eq!(back.sample_rate, track.sample_rate);
        assert_eq!(back.channels, 1, "单声道");
        assert_eq!(back.bits, 16, "16bit 打包");
        assert_eq!(back.samples, track.pcm, "PCM 往返无损");
        assert_eq!(back.frames(), track.frames);
    }
}

#[test]
fn bgm_and_se_selection_match_measured_track_lengths() {
    let bundle = prepare_music(&music_bytes()).expect("准备管线成功");
    let longest = bundle.tracks.iter().max_by_key(|track| track.frames).expect("至少一条曲目");
    let shortest = bundle.tracks.iter().min_by_key(|track| track.frames).expect("至少一条曲目");
    assert_eq!(longest.key, LONGEST_WAV_KEY, "最长 WAV = ae1a80cc（壳的 BGM）");
    assert_eq!(shortest.key, SHORTEST_WAV_KEY, "最短 WAV = 9ef9b7a9（壳的 SE）");
    assert_ne!(longest.key, shortest.key, "BGM 与 SE 不应是同一条");
}

#[test]
fn prepare_music_is_deterministic() {
    let raw = music_bytes();
    let first = prepare_music(&raw).expect("第一次准备成功");
    let second = prepare_music(&raw).expect("第二次准备成功");
    assert_eq!(first, second, "同一输入必须产出完全相同的混音结果");
}
