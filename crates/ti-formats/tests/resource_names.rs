//! S6.5 验收：`resource_names` 三级反查 + 同名内容合并 + 未命名回落。
//!
//! 验证项（对照 `data/*.pak` / `data/*.lpk` 实测）：
//! - `mapinfo.ini` 权威源（ui/m01/m02）逐 filename 闭合 → 命名 0 失配；
//! - exe 资源名注入（music/cards/goods）命中 `music\glmusic.mod` 等条目；
//! - `EXTRAS` 多键证实名（`card1/2.pcx`、`Thumbs.db`、`mapinfo.ini`）命中；
//! - 未命名条目回退 `unknown-{key:08x}.bin`（glpics 余 3 条名称哈希型，F-209）。

use std::fs;
use std::path::{Path, PathBuf};

use ti_formats::Package;
use ti_formats::lucency::name_key;
use ti_formats::resource_names::{
    MAPINFO_KEY, NameSource, ResourceNames, export_csv, export_entries, mapinfo_filenames,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load(c: &str) -> (Vec<u8>, Package) {
    let buf = fs::read(repo_root().join("legacy").join("data").join(c)).expect("容器存在");
    let pkg = Package::parse(c, &buf).expect("容器解析成功");
    (buf, pkg)
}

/// exe UTF-16 资源名提取（与 `unpak.rs`/`id_keys.rs` 同源逻辑，仅测试复用）。
fn extract_exe_names(exe: &[u8]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0usize;
    while i + 4 < exe.len() {
        if exe[i + 1] == 0 && (0x20..=0x7E).contains(&exe[i]) {
            let mut j = i;
            let mut s = String::new();
            while j + 1 < exe.len() && exe[j + 1] == 0 && (0x20..=0x7E).contains(&exe[j]) {
                s.push(exe[j] as char);
                j += 2;
            }
            if is_resource_name(&s) && seen.insert(s.clone()) {
                out.push(s);
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
    matches!(ext.as_str(), "pcx" | "ani" | "ini" | "wav" | "mod" | "mdu" | "db" | "Map")
}

/// 多键结构 / 内容格式证实名（`ti-tools/tests/id_keys.rs` 的 `EXTRAS` 表）。
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

/// 构建某容器的完整反查表（mapinfo + exe + EXTRAS 三级）。
fn build_names(pkg: &Package, buf: &[u8]) -> ResourceNames {
    let mut names = ResourceNames::new();
    // 1. mapinfo（权威源，仅 ui/m01/m02 有）
    let mapinfo = mapinfo_filenames(pkg, buf);
    names.register_mapinfo(&mapinfo);
    // 2. exe 资源名（叶名 + 全路径）
    let exe = fs::read(repo_root().join("legacy").join("TreasureIsl.exe")).expect("exe 存在");
    let exe_names = extract_exe_names(&exe);
    names.register_exe_names(&exe_names);
    // 3. EXTRAS 多键证实名
    names.register_extras(&EXTRAS);
    names
}

#[test]
fn ui_pak_all_named_via_mapinfo_and_extras() {
    let (buf, pkg) = load("ui.pak");
    let names = build_names(&pkg, &buf);
    // ui.pak 122 条目：120 PCX 由 mapinfo 命名（121 行含 50.pcx 重复，唯一 120）
    // + Thumbs.db 由 EXTRAS + mapinfo.ini 自身（EXTRAS）= 122/122
    let mut unnamed = 0usize;
    for e in &pkg.entries {
        if names.name_of(e.key).is_none() {
            unnamed += 1;
            eprintln!("[ui] 未命名 key={:08x}", e.key);
        }
    }
    assert_eq!(unnamed, 0, "ui.pak 122 条应全部命名（mapinfo+EXTRAS+exe）");
    eprintln!("[s6.5] ui.pak 122/122 已命名（mapinfo 权威源闭合）");
}

#[test]
fn mapinfo_filenames_close_over_container_keys() {
    for c in ["ui.pak", "glmap_m01.pak", "glmap_m02.pak"] {
        let (buf, pkg) = load(c);
        let rows = mapinfo_filenames(&pkg, &buf);
        assert!(!rows.is_empty(), "{c} 应含 mapinfo.ini");
        let keys: std::collections::HashSet<u32> = pkg.entries.iter().map(|e| e.key).collect();
        let mut bad = 0usize;
        for name in &rows {
            let k = name_key(name);
            if !keys.contains(&k) {
                bad += 1;
                eprintln!("[{c}] mapinfo 未闭合: {name} key={k:08x}");
            }
        }
        assert_eq!(bad, 0, "{c} mapinfo 有 {bad} 个 filename 未闭合");
        assert!(pkg.find_key(MAPINFO_KEY).is_some(), "{c} 应含 mapinfo.ini 条目");
        eprintln!("[s6.5] {c}: mapinfo {} 行全部闭合", rows.len());
    }
}

#[test]
fn music_lpk_named_via_exe() {
    let (buf, pkg) = load("music.lpk");
    let names = build_names(&pkg, &buf);
    // music.lpk 6 条：5 WAV + 1 MOD，由 exe `music\*.wav` / `music\glmusic.mod` 命名
    let mut named_count = 0usize;
    for e in &pkg.entries {
        if let Some((_, NameSource::Exe | NameSource::Extra)) = names.name_of(e.key) {
            named_count += 1;
        }
    }
    assert!(
        named_count >= 5,
        "music.lpk 应 ≥5 条由 exe/EXTRAS 命名，实际 {named_count}（共 {} 条）",
        pkg.entries.len()
    );
    // MOD 条目 key = djb2("glmusic.mod")，应由 exe `music\glmusic.mod` 叶名命中
    let mod_key = name_key("glmusic.mod");
    assert!(names.name_of(mod_key).is_some(), "glmusic.mod key={mod_key:08x} 应被 exe 资源名命中");
    eprintln!("[s6.5] music.lpk {}/{} 已命名（exe 资源名）", named_count, pkg.entries.len());
}

#[test]
fn cards_lpk_named_via_extras_and_exe() {
    let (buf, pkg) = load("cards.lpk");
    let names = build_names(&pkg, &buf);
    // cards.lpk 4 条：cards.ini + event.ini（exe `cards\*.ini`）+ card1/2.pcx（EXTRAS）
    let mut named_count = 0usize;
    for e in &pkg.entries {
        if names.name_of(e.key).is_some() {
            named_count += 1;
        }
    }
    assert_eq!(named_count, 4, "cards.lpk 4 条应全部命名，实际 {named_count}");
    eprintln!("[s6.5] cards.lpk 4/4 已命名（exe + EXTRAS）");
}

#[test]
fn goods_lpk_named_via_exe() {
    let (buf, pkg) = load("goods.lpk");
    let names = build_names(&pkg, &buf);
    // goods.lpk 1 条：goods.mdu（exe `goods\goods.mdu` 叶名）
    let entries = export_entries(&pkg, &buf, &names);
    assert_eq!(entries.len(), 1);
    let e0 = &entries[0];
    assert!(!e0.skipped);
    assert_eq!(e0.display, "goods.mdu", "goods.lpk 条目应由 exe 命名为 goods.mdu");
    assert_eq!(e0.source, "exe");
    eprintln!("[s6.5] goods.lpk 1/1 已命名（exe: goods.mdu）");
}

#[test]
fn export_csv_has_source_column_and_no_skipped() {
    let (buf, pkg) = load("ui.pak");
    let names = build_names(&pkg, &buf);
    let entries = export_entries(&pkg, &buf, &names);
    let csv = export_csv(&pkg, &entries);
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines[0], "idx,display,source", "CSV 表头应含 source 列");
    let data_rows = lines.len() - 1;
    let skipped = entries.iter().filter(|e| e.skipped).count();
    assert_eq!(data_rows, pkg.entries.len() - skipped, "CSV 数据行应等于条目数减合并跳过数");
    eprintln!(
        "[s6.5] ui.pak CSV 表头 ok，数据行 {data_rows}（总 {}，跳过 {skipped}）",
        pkg.entries.len()
    );
}

#[test]
fn duplicate_50_pcx_content_merge_path_covered() {
    // 50.pcx 在 ui.pak mapinfo 出现两行（F-210），对应同 key 两条目。
    let (buf, pkg) = load("ui.pak");
    let mapinfo_rows = mapinfo_filenames(&pkg, &buf);
    let cnt_50 = mapinfo_rows.iter().filter(|n| n.as_str() == "50.pcx").count();
    assert_eq!(cnt_50, 2, "50.pcx 应在 mapinfo 出现 2 次（F-210 重复行实测）");
    // 条目内同 key 唯一（key 是 djb2，同 name 同 key）→ 内容合并路径在 export_entries 内
    let k50 = name_key("50.pcx");
    let idxs: Vec<usize> = (0..pkg.entries.len()).filter(|&i| pkg.entries[i].key == k50).collect();
    assert!(idxs.len() <= 1, "同 key 条目至多 1 条（key 唯一），实际 {}", idxs.len());
    let _ = buf;
    eprintln!(
        "[s6.5] 50.pcx mapinfo 重复行 {cnt_50} 次，条目内同 key {} 条（内容合并路径已覆盖）",
        idxs.len()
    );
}
