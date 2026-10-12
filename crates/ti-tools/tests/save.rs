//! S1.9 验收：`settings.dat` 往返逐字节无损 + 存档封套编解码往返。
//!
//! 原版 `gl*.sav` / `gl.rec` 在资源包中不存在（运行时写出），字段表落地在 S3.10。

use std::path::{Path, PathBuf};

use ti_formats::save::{SETTINGS_LEN, SaveEnvelope, Settings};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[test]
fn settings_dat_roundtrip() {
    let raw = std::fs::read(repo_root().join("legacy").join("data").join("settings.dat"))
        .expect("settings.dat 存在");
    assert_eq!(raw, vec![0x5D, 0x4A, 0x71, 0x4A, 0x2E, 0x01, 0x01, 0x01], "F-304 字节");
    let s = Settings::parse("settings.dat", &raw).expect("解析成功");
    assert_eq!(s.to_bytes(), raw, "往返必须逐字节一致");
    assert_eq!(raw.len(), SETTINGS_LEN);
}

#[test]
fn settings_rejects_bad_magic_and_length() {
    let mut bad = vec![0x00, 0x4A, 0x71, 0x4A, 0x2E, 0x01, 0x01, 0x01];
    assert!(Settings::parse("s", &bad).is_err(), "魔数不符必须报错");
    bad[0] = 0x5D;
    assert!(Settings::parse("s", &bad).is_ok());
    assert!(Settings::parse("s", &bad[..4]).is_err(), "长度不符必须报错");
}

#[test]
fn save_envelope_roundtrips() {
    let payload: Vec<u8> = (0..1024u32).map(|i| (i % 253) as u8).collect();
    for v in [1u16, 2] {
        let env = SaveEnvelope::new(v, payload.clone());
        let bytes = env.encode();
        let back = SaveEnvelope::decode("gl0.sav", &bytes).expect("解码成功");
        assert_eq!(back, env, "v{v} 封套往返不一致");
        assert_eq!(back.payload, payload);
    }
}

#[test]
fn save_envelope_rejects_truncated_and_bad_magic() {
    let env = SaveEnvelope::new(1, vec![1, 2, 3]);
    let bytes = env.encode();
    assert!(SaveEnvelope::decode("s", &bytes[..4]).is_err(), "截断必须报错");
    let mut bad = bytes.clone();
    bad[0] = b'X';
    assert!(SaveEnvelope::decode("s", &bad).is_err(), "魔数不符必须报错");
    let mut lying = bytes.clone();
    lying[4] = 0xFF;
    assert!(SaveEnvelope::decode("s", &lying).is_err(), "长度字段不符必须报错");
}
