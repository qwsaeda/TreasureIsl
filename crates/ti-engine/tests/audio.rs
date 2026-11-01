//! S34 音频契约测试（用户 2026-11-01 拍板「音乐动效也没有」→ 补原版音频）。
//!
//! `bevy_audio` 实际播放需真实音频设备（headless 无设备 → `AudioOutput::default` 记 WARN
//! `stream=None`，`audio_output_available` false，播放系统不跑，不 panic）；
//! 本测试验证**可机器判定的资产契约**（`23_game_testing.md` §2「音频」类）：
//!
//! ① `registry.json` `audio[]` 含 4 项 = BGM `glmusic.wav`（`loop:true`）+ SFX
//!    `press/invalid/victory.wav`（一次），`role` 唯一；
//! ② 4 个 WAV 文件存在且 RIFF/WAVE 魔数有效（可被 `AudioSource` 加载，`bevy_audio` wav=hound）；
//! ③ 引用集契约：`assets/audio/` 下每个 `.wav` 都登记在 `registry`（双向同步，
//!    `validate_data::check_asset_registry` 已兜底，本测试独立复核防漂移）。

use std::path::Path;

/// 测试根 = 仓库根（tests 工作目录 = crate 根，向上两级 = 仓库根）。
fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf)
}

fn registry() -> serde_json::Value {
    let root = repo_root();
    let raw =
        std::fs::read_to_string(root.join("assets/registry.json")).expect("registry.json 可读");
    serde_json::from_str(&raw).expect("registry.json 合法 JSON")
}

fn audio_entries() -> Vec<serde_json::Value> {
    registry().get("audio").and_then(serde_json::Value::as_array).cloned().unwrap_or_default()
}

/// ① registry.audio 含 4 项，role 唯一，BGM 标 loop:true。
#[test]
fn registry_has_bgm_and_three_sfx() {
    let entries = audio_entries();
    assert_eq!(entries.len(), 4, "registry.audio 应含 4 项（BGM + 3 SFX）");

    let mut roles: Vec<String> = Vec::new();
    let mut has_bgm_loop = false;
    for e in &entries {
        let role = e.get("role").and_then(serde_json::Value::as_str).unwrap_or("");
        assert!(!role.is_empty(), "audio 项缺 role");
        roles.push(role.to_owned());
        if role == "bgm" {
            assert_eq!(
                e.get("loop").and_then(serde_json::Value::as_bool),
                Some(true),
                "BGM 必须 loop:true（PlaybackSettings::LOOP 契约）"
            );
            has_bgm_loop = true;
        }
    }
    assert!(has_bgm_loop, "registry.audio 缺 bgm 项");
    // role 唯一（bgm/sfx_press/sfx_invalid/sfx_victory）
    let uniq: std::collections::HashSet<_> = roles.iter().collect();
    assert_eq!(uniq.len(), roles.len(), "audio role 必须唯一");
}

/// ② 每个登记音频文件存在且是合法 RIFF/WAVE（`bevy_audio` wav = hound 可解）。
#[test]
fn audio_files_exist_and_are_valid_wav() {
    let root = repo_root();
    let entries = audio_entries();
    assert_ne!(entries.len(), 0, "registry.audio 为空（补原版音频登记缺失）");
    for e in &entries {
        let path = e.get("path").and_then(serde_json::Value::as_str).unwrap_or("");
        assert!(!path.is_empty(), "audio 项缺 path");
        let full = root.join(path);
        assert!(full.is_file(), "音频文件缺失：{path}");
        // RIFF/WAVE 魔数（WAV 容器头：'RIFF' + size + 'WAVE'）
        let bytes = std::fs::read(&full).expect("音频可读");
        assert!(bytes.len() > 44, "音频过小（非 WAV？）：{path}");
        assert_eq!(&bytes[0..4], b"RIFF", "缺 RIFF 头：{path}");
        assert_eq!(&bytes[8..12], b"WAVE", "缺 WAVE 标记：{path}");
    }
}

/// ③ 双向同步：`assets/audio/` 目录下每个 `.wav` 都在 registry.audio 登记
/// （与 `validate_data::check_asset_registry` 同口径，独立复核）。
#[test]
fn audio_dir_matches_registry() {
    let root = repo_root();
    let audio_dir = root.join("assets/audio");
    let mut disk: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&audio_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.to_ascii_lowercase().ends_with(".wav") {
                disk.push(format!("assets/audio/{name}"));
            }
        }
    }
    disk.sort();
    let mut reg: Vec<String> = audio_entries()
        .iter()
        .filter_map(|e| e.get("path").and_then(serde_json::Value::as_str).map(str::to_owned))
        .collect();
    reg.sort();
    assert_eq!(
        disk,
        reg,
        "assets/audio/ 与 registry.audio 双向不同步（磁盘 {} vs 登记 {}）",
        disk.len(),
        reg.len()
    );
    // 至少 BGM + 3 SFX 在位
    assert!(disk.len() >= 4, "assets/audio/ 音频不足 4 个");
}

/// 辅助：`Path` 直接判存在（供断言可读，无副作用）。
#[allow(dead_code)]
fn _exists(p: &Path) -> bool {
    p.exists()
}
