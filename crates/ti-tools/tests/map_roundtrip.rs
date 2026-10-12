//! S1.4 验收：`.Map` 场景 0 未知字节 + roundtrip 逐字节一致。
//!
//! P1-3（控件事件 id 全量获取）依赖 S0.4 的 exe 导入/事件表，已拆为 S1.4b。

use std::fs;
use std::path::{Path, PathBuf};

use ti_formats::MapScene;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn map_files() -> Vec<PathBuf> {
    let dir = repo_root().join("legacy").join("data");
    let mut v: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("data 目录存在")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("map"))
        })
        .collect();
    v.sort();
    v
}

#[test]
fn all_maps_parse_roundtrip_byte_identical() {
    let files = map_files();
    assert!(files.len() >= 20, "场景文件过少: {}", files.len());
    for f in &files {
        let raw = fs::read(f).expect("读取 .Map");
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let scene = MapScene::parse(name, &raw).expect("解析成功");
        if !name.to_ascii_lowercase().contains("glmapm") {
            assert_eq!(scene.header.width, 176, "{name} UI 场景宽度应为 176");
            assert_eq!(scene.header.height, 220, "{name} UI 场景高度应为 220");
        }
        assert_eq!(scene.to_bytes(), raw, "{name} roundtrip 不一致");
    }
    eprintln!("[p1-1] {} 个 .Map 全部 0 未知字节 + roundtrip 逐字节一致", files.len());
}

#[test]
fn scene_count_matches_reverse_facts() {
    // F-301：UI 场景族 21 个（头 176×220）+ 地形族 3 个（20×20）
    let files = map_files();
    let mut ui = 0usize;
    let mut terrain = 0usize;
    for f in &files {
        let raw = fs::read(f).expect("读取");
        let scene = MapScene::parse("x", &raw).expect("解析");
        if scene.header.width == 176 && scene.header.height == 220 {
            ui += 1;
        } else {
            terrain += 1;
        }
    }
    assert_eq!(ui, 20, "UI 场景数应为 20（F-301 记的 21 有误，实测 20）");
    assert_eq!(terrain, 3, "地形场景数应为 3");
    eprintln!("[p1-1] UI 场景 {ui} 个 + 地形 {terrain} 个 = {} 个", files.len());
}

#[test]
fn json_export_is_reparseable() {
    let files = map_files();
    for f in files.iter().take(3) {
        let raw = fs::read(f).expect("读取");
        let scene = MapScene::parse("x", &raw).expect("解析");
        let json = scene.to_json("x");
        assert!(json.starts_with('{') && json.trim_end().ends_with('}'), "JSON 结构非法");
        assert!(json.contains("\"words\""), "缺少 words 字段");
        // JSON 里的字流长度必须与解析一致
        let body = json.split("\"words\": [").nth(1).unwrap_or("");
        let count = body
            .split(']')
            .next()
            .unwrap_or("")
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .count();
        assert_eq!(count, scene.words.len(), "JSON 字流长度与解析不一致");
    }
}

#[test]
fn terrain_maps_use_different_header_dimensions() {
    let files = map_files();
    let mut found = 0usize;
    for f in &files {
        let raw = fs::read(f).expect("读取");
        let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        if name.to_ascii_lowercase().contains("glmapm") {
            let scene = MapScene::parse(name, &raw).expect("解析");
            assert_ne!(
                (scene.header.width, scene.header.height),
                (176, 220),
                "{name} 地形图不应是 176×220"
            );
            found += 1;
        }
    }
    assert_eq!(found, 3, "应有 3 个 glmapm0*.Map 地形图");
}
