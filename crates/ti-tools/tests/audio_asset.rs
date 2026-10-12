//! S1.7 验收：`music.lpk` 素材识别 + WAV 抽取/重采样 + MOD 头部规格。

use std::path::{Path, PathBuf};

use ti_formats::lucency::Package;
use ti_platform::audio_asset::{Asset, Kind, build_wav_8bit};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn entries() -> Vec<(usize, Vec<u8>)> {
    let buf = std::fs::read(repo_root().join("legacy").join("data").join("music.lpk"))
        .expect("music.lpk 存在");
    let pkg = Package::parse("music.lpk", &buf).expect("解析成功");
    (0..pkg.entries.len()).map(|i| (i, pkg.read_entry(&buf, i).expect("解压成功"))).collect()
}

#[test]
fn detects_five_wavs_and_one_mod() {
    let mut wav = 0;
    let mut mod_ = 0;
    let mut binary = 0;
    for (_i, raw) in entries() {
        let a = Asset::detect(&raw);
        match a.kind {
            Kind::Wav => wav += 1,
            Kind::Mod => {
                mod_ += 1;
                assert_eq!(&raw[..8], b"INHL.MOD", "MOD 签名应为 INHL.MOD");
                assert_eq!(a.title, "I Need Her Love", "MOD 标题");
            }
            Kind::Binary => binary += 1,
        }
    }
    assert_eq!(wav, 5, "应有 5 条 WAV");
    assert_eq!(mod_, 1, "应有 1 条 MOD");
    assert_eq!(binary, 0, "不应出现 Binary 类型");
}

#[test]
fn every_wav_track_is_playable() {
    for (i, raw) in entries() {
        let a = Asset::detect(&raw);
        if a.kind != Kind::Wav {
            continue;
        }
        let clip = a.as_wav().expect("WAV 解析成功");
        assert!(!clip.samples.is_empty(), "音轨 {i} 无采样");
        assert!(clip.duration_secs() > 0.0);
    }
}

#[test]
fn resample_to_22050_produces_valid_wav() {
    for (i, raw) in entries() {
        let a = Asset::detect(&raw);
        if a.kind != Kind::Wav {
            continue;
        }
        let out = a.to_pcm_wav(22_050).expect("重采样成功");
        assert_eq!(&out[..4], b"RIFF", "音轨 {i} 输出应为 RIFF");
        assert_eq!(&out[8..12], b"WAVE");
        assert_eq!(
            u32::from_le_bytes([out[24], out[25], out[26], out[27]]),
            22_050,
            "采样率应为 22050"
        );
        assert_eq!(u16::from_le_bytes([out[34], out[35]]), 8, "应为 8bit");
    }
}

#[test]
fn build_wav_8bit_is_self_consistent() {
    let data: Vec<u8> = (0..100u32).map(|i| u8::try_from(i % 256).unwrap_or(0)).collect();
    let w = build_wav_8bit(8000, &data);
    assert_eq!(&w[..4], b"RIFF");
    assert_eq!(w.len(), 44 + data.len());
    let riff = u32::from_le_bytes([w[4], w[5], w[6], w[7]]) as usize;
    assert_eq!(riff, w.len() - 8);
    assert_eq!(&w[44..], &data[..]);
}
