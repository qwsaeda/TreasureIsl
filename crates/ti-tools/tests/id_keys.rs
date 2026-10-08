//! S1.11 验收：ID 型 key 的绑定机制 = 容器内 `mapinfo.ini` 文件名经 `djb2` 闭合。
//!
//! 结论（实测 2026-10，见 F-209/F-210/F-306 与 `evidence/S1.11-audit.json`）：
//! - `ui.pak` / `glmap_*.pak` 内各有一份 `mapinfo.ini`（key `0x3753DC98`，
//!   UTF-16LE+BOM，`[filenum]` + 逐条 `filename=`），`key = djb2(filename)` 100% 闭合；
//! - `DoD` 范围 217 条（ui/glmap/lpk）全部命名可解：mapinfo 清单 + exe 资源名 +
//!   多键结构证实名（`appo01~04.pcx` / `card1~2.pcx`）+ `Thumbs.db`（内容为 OLE 文档）；
//! - UI 场景 `.Map` 记录布局 = `[16B 头][子头 4×u32][count][count×6]`，20/20 匹配。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use ti_formats::{Ini, MapScene, Package, name_key};

/// `DoD` 范围容器（217 条 = 122 + 47 + 37 + 4 + 6 + 1）。
const DOD_CONTAINERS: [&str; 6] =
    ["ui.pak", "glmap_m01.pak", "glmap_m02.pak", "cards.lpk", "music.lpk", "goods.lpk"];

/// 内含 `mapinfo.ini` 的容器（文件名行数 = ui 121 / m01 49 / m02 34）。
const MAPINFO_CONTAINERS: [(&str, usize); 3] =
    [("ui.pak", 121), ("glmap_m01.pak", 49), ("glmap_m02.pak", 34)];

/// 不在 exe 字符串与 mapinfo 清单内、经多键结构/内容格式证实的权威名。
const EXTRAS: [&str; 8] = [
    "mapinfo.ini",
    "Thumbs.db",
    "appo01.pcx",
    "appo02.pcx",
    "appo03.pcx",
    "appo04.pcx",
    "card1.pcx",
    "card2.pcx",
];

/// UI 场景族 `.Map` 数量（F-301，实测 20）。
const UI_SCENE_COUNT: usize = 20;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load(c: &str) -> (Vec<u8>, Package) {
    let buf = fs::read(repo_root().join("data").join(c)).expect("容器存在");
    let pkg = Package::parse(c, &buf).expect("容器解析成功");
    (buf, pkg)
}

/// 从 exe 提取 UTF-16 资源名（全串 + 反斜杠叶名，大小写保留，`name_key` 自行小写）。
fn extract_names(exe: &[u8]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let push = |s: &str, out: &mut Vec<String>, seen: &mut HashSet<String>| {
        if seen.insert(s.to_string()) {
            out.push(s.to_string());
        }
    };
    let mut i = 0usize;
    while i + 4 < exe.len() {
        if exe[i + 1] == 0 && (0x20..=0x7E).contains(&exe[i]) {
            let mut j = i;
            let mut buf = Vec::new();
            while j + 1 < exe.len() && exe[j + 1] == 0 && (0x20..=0x7E).contains(&exe[j]) {
                buf.push(exe[j]);
                j += 2;
            }
            let s = String::from_utf8_lossy(&buf).into_owned();
            if is_resource_name(&s) {
                push(&s, &mut out, &mut seen);
                if let Some(leaf) = s.rsplit('\\').next() {
                    if leaf != s {
                        push(leaf, &mut out, &mut seen);
                    }
                }
            }
            i = j + 2;
        } else {
            i += 1;
        }
    }
    out
}

fn is_resource_name(s: &str) -> bool {
    if s.len() < 4 || s.len() > 64 {
        return false;
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '\\' | '.' | '-')) {
        return false;
    }
    let leaf = s.rsplit('\\').next().unwrap_or(s);
    let ext = leaf.rsplit_once('.').map_or("", |(_, e)| e).to_ascii_lowercase();
    matches!(ext.as_str(), "pcx" | "ani" | "ini" | "wav" | "mod" | "mdu" | "db")
}

/// 读出容器内 `mapinfo.ini` 的全部 `filename=` 行。
fn mapinfo_filenames(pkg: &Package, buf: &[u8]) -> Vec<String> {
    let idx = pkg.find("mapinfo.ini").expect("容器应含 mapinfo.ini");
    let raw = pkg.read_entry(buf, idx).expect("mapinfo.ini 解压成功");
    let ini = Ini::parse_utf16le("mapinfo.ini", &raw).expect("mapinfo 解析成功");
    ini.entries
        .iter()
        .filter(|e| e.key.eq_ignore_ascii_case("filename"))
        .map(|e| e.value.clone())
        .collect()
}

#[test]
fn mapinfo_filenames_close_over_container_keys() {
    for (container, expect_rows) in MAPINFO_CONTAINERS {
        let (buf, pkg) = load(container);
        let rows = mapinfo_filenames(&pkg, &buf);
        assert_eq!(rows.len(), expect_rows, "{container} mapinfo 行数不符");

        let keys: HashSet<u32> = pkg.entries.iter().map(|e| e.key).collect();
        let mut bad = 0usize;
        for name in &rows {
            if !keys.contains(&name_key(name)) {
                bad += 1;
                eprintln!("[s1.11] {container} 未闭合: {name}");
            }
        }
        assert_eq!(bad, 0, "{container} 有 {bad} 个 filename 未闭合");
        eprintln!("[s1.11] {container}: mapinfo {}/{} 全部闭合", rows.len(), expect_rows);
    }
}

#[test]
fn dod_scope_keys_are_all_named() {
    let exe = fs::read(repo_root().join("TreasureIsl.exe")).expect("exe 存在");
    let mut auth: HashSet<u32> = extract_names(&exe).iter().map(|n| name_key(n)).collect();
    for n in EXTRAS {
        auth.insert(name_key(n));
    }
    for (container, _) in MAPINFO_CONTAINERS {
        let (buf, pkg) = load(container);
        for name in mapinfo_filenames(&pkg, &buf) {
            auth.insert(name_key(&name));
        }
    }

    let mut total = 0usize;
    let mut hit = 0usize;
    for c in DOD_CONTAINERS {
        let (_buf, pkg) = load(c);
        let h = pkg.entries.iter().filter(|e| auth.contains(&e.key)).count();
        total += pkg.entries.len();
        hit += h;
        assert_eq!(h, pkg.entries.len(), "{c} 存在未命名 key");
        eprintln!("[s1.11] {c:16} {h:4}/{} 已命名", pkg.entries.len());
    }
    assert_eq!(total, 217, "DoD 范围应为 217 条");
    assert_eq!(hit, 217, "DoD 覆盖应为 217/217");

    // glpics.pak 为名称哈希型（S1.1 已验收 85/88），此处仅作回归观察
    let (_gbuf, gpkg) = load("glpics.pak");
    let ghit = gpkg.entries.iter().filter(|e| auth.contains(&e.key)).count();
    assert!(ghit >= 85, "glpics 至少 85/88 可由 exe 资源名解释，实际 {ghit}");
    eprintln!("[s1.11] DoD 217/217 已命名；glpics {ghit}/88");
}

#[test]
fn key_vectors_and_ui_ap_structure() {
    // 等差公差：字符位权 33^4 = 0x121881（进位 09→10 = 24×33^4）。⚠ 旧记录 0x121101 是错的。
    const STEP: u32 = 33u32.pow(4);

    // ui.pak 索引按 key 升序：100.pcx..120.pcx、mapinfo.ini、01.pcx...
    let (buf, ui) = load("ui.pak");
    assert_eq!(ui.entries.len(), 122, "ui.pak 条目数");
    assert_eq!(ui.entries[0].key, name_key("100.pcx"), "idx0 应为 100.pcx");
    assert_eq!(ui.entries[20].key, name_key("120.pcx"), "idx20 应为 120.pcx");
    assert_eq!(ui.entries[21].key, name_key("mapinfo.ini"), "idx21 应为 mapinfo.ini");
    assert_eq!(ui.entries[22].key, name_key("01.pcx"), "idx22 应为 01.pcx");
    assert_eq!(ui.find("Thumbs.db"), Some(121), "ui idx121 应为 Thumbs.db");
    let rows = mapinfo_filenames(&ui, &buf);
    assert_eq!(rows.iter().filter(|n| n.as_str() == "50.pcx").count(), 2, "50.pcx 应重复");

    // 等差公差按字符位权实算（进位 09→10 = 24×33^4）
    assert_eq!(STEP, 0x121_881, "33^4 常量");
    let mut steps: HashSet<u32> = HashSet::new();
    for pair in ui.entries[..20].windows(2) {
        steps.insert(pair[1].key.wrapping_sub(pair[0].key));
    }
    assert_eq!(steps, HashSet::from([STEP, 24 * STEP]), "idx0..20 公差集合");

    // 多键结构证实名（单键命中可能是假 preimage，多键同时命中则碰撞可忽略）
    let (_m2buf, m02) = load("glmap_m02.pak");
    assert_eq!(m02.entries.len(), 37, "glmap_m02 条目数");
    for (i, name) in ["appo01.pcx", "appo02.pcx", "appo03.pcx", "appo04.pcx"].iter().enumerate() {
        assert_eq!(m02.entries[13 + i].key, name_key(name), "m02 idx{} 应为 {name}", 13 + i);
    }

    let (_cbuf, cards) = load("cards.lpk");
    assert_eq!(cards.find("event.ini"), Some(0), "event.ini idx0");
    assert_eq!(cards.find("card1.pcx"), Some(1), "card1.pcx idx1");
    assert_eq!(cards.find("card2.pcx"), Some(2), "card2.pcx idx2");
    assert_eq!(cards.find("cards.ini"), Some(3), "cards.ini idx3");
    eprintln!("[s1.11] 关键向量 + UI 等差结构验证通过");
}

#[test]
fn ui_scene_map_record_layout_matches_count() {
    // F-306：[16B 头][子头 4×u32][count u32][count × 6 × u32] => words.len() == 5 + 6*count
    let dir = repo_root().join("data");
    let mut checked = 0usize;
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).expect("data 目录可读") {
        let entry = entry.expect("目录项可读");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("Map") {
            continue;
        }
        let raw = fs::read(&path).expect("Map 可读");
        let scene = MapScene::parse(path.to_str().unwrap_or("map"), &raw).expect("Map 解析成功");
        if scene.header.width != 176 {
            continue; // 地形族（20×20，F-301）
        }
        let w = &scene.words;
        assert!(w.len() >= 5, "{path:?} 词流过短");
        let count = w[4] as usize;
        assert!(count >= 1, "{path:?} count 为 0");
        assert_eq!(w.len(), 5 + count * 6, "{path:?} 记录布局与 count 不符");
        checked += 1;
        names.push(path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string());
    }
    assert_eq!(checked, UI_SCENE_COUNT, "UI 场景应为 20 个，实测 {checked}: {names:?}");
    eprintln!("[s1.11] UI 场景 {checked}/{UI_SCENE_COUNT} 记录布局全部匹配");
}
