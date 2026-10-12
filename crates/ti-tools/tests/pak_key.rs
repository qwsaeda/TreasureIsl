//! S1.1 验收：`name→key` 哈希函数实测验证（P1-2 修订口径）。
//!
//! 结论（实测 2026-10）：`key = djb2(name.to_ascii_lowercase())`，种子 0。
//! - `glpics.pak` 88 条中 85 条可由 `TreasureIsl.exe` 的 UTF-16 资源名唯一解释；
//! - 其余容器（`ui.pak` / `glmap_*.pak` / `*.lpk`）的 key 属**ID 型**，
//!   由容器内 `mapinfo.ini` 文件名清单绑定（S1.11 / F-210，见测试 `id_keys`，EXP-017）；
//! - 因此验收断言 = 「可由资源名解释的条目必须 100% 无误，且数量不低于实测下界」。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use ti_formats::{Package, name_key};

const CONTAINERS: [&str; 7] = [
    "ui.pak",
    "glpics.pak",
    "glmap_m01.pak",
    "glmap_m02.pak",
    "cards.lpk",
    "music.lpk",
    "goods.lpk",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
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
    matches!(ext.as_str(), "pcx" | "map" | "ini" | "wav" | "mod" | "mdu" | "dat" | "fnt" | "rec")
}

fn extract_names(exe: &[u8]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
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

fn load(c: &str) -> (Vec<u8>, Package) {
    let buf = fs::read(repo_root().join("legacy").join("data").join(c)).expect("容器存在");
    let pkg = Package::parse(c, &buf).expect("容器解析成功");
    (buf, pkg)
}

#[test]
fn name_derivable_entries_are_all_explained() {
    let exe = fs::read(repo_root().join("legacy").join("TreasureIsl.exe")).expect("exe 存在");
    let names = extract_names(&exe);
    assert!(names.len() >= 100, "资源名过少: {}", names.len());

    let mut by_key: HashMap<u32, Vec<&str>> = HashMap::new();
    for n in &names {
        by_key.entry(name_key(n)).or_default().push(n);
    }

    let mut total = 0usize;
    let mut explained = 0usize;
    let mut glpics_hit = 0usize;

    for c in CONTAINERS {
        let (_buf, pkg) = load(c);
        let mut hit = 0usize;
        for e in &pkg.entries {
            total += 1;
            if by_key.contains_key(&e.key) {
                hit += 1;
                explained += 1;
            }
        }
        if c == "glpics.pak" {
            glpics_hit = hit;
        }
        eprintln!("[pak_key] {c:16} 条目 {:4} 可由资源名解释 {hit:4}", pkg.entries.len());
    }

    assert!(glpics_hit >= 85, "glpics.pak 至少 85 条可解释，实际 {glpics_hit}");
    assert!(explained >= 85, "全容器至少 85 条可解释，实际 {explained}");
    eprintln!("[pak_key] 资源名 {} 个，条目 {total}，其中 {explained} 条可解释", names.len());
}

#[test]
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn index_encoding_follows_zlib_magic_not_extension() {
    for c in CONTAINERS {
        let (_buf, pkg) = load(c);
        let expect_compressed = c.ends_with(".pak");
        assert_eq!(pkg.index_compressed, expect_compressed, "{c} 索引压缩标志错误");
        assert!(!pkg.entries.is_empty(), "{c} 条目为空");
    }
}

#[test]
fn entry_offset_chains_are_contiguous() {
    for c in CONTAINERS {
        let (_buf, pkg) = load(c);
        for pair in pkg.entries.windows(2) {
            assert_eq!(pair[1].off, pair[0].off + pair[0].clen, "{c} off 链断裂");
        }
    }
}

#[test]
fn first_entry_signatures_match_reverse_facts() {
    // F-205：ui.pak 首条 = PCX 头 0A 05 01 08
    let (buf, pkg) = load("ui.pak");
    let first = pkg.read_entry(&buf, 0).expect("ui.pak 首条解压成功");
    assert_eq!(&first[..4], &[0x0A, 0x05, 0x01, 0x08], "PCX 头不符");

    // F-208：cards.lpk 首条 = UTF-16 INI（含 ';' 注释）
    let (cbuf, cpkg) = load("cards.lpk");
    let head = cpkg.read_entry(&cbuf, 0).expect("cards.lpk 首条解压成功");
    let units: Vec<u16> =
        head.chunks_exact(2).take(16).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    let text = String::from_utf16_lossy(&units);
    assert!(text.contains(';'), "INI 注释符 ';' 缺失: {text:?}");

    // F-208：music.lpk 首条 = RIFF/WAVE
    let (mbuf, mpkg) = load("music.lpk");
    let wav = mpkg.read_entry(&mbuf, 0).expect("music.lpk 首条解压成功");
    assert_eq!(&wav[..4], b"RIFF", "music.lpk 首条应为 RIFF");
    assert_eq!(&wav[8..12], b"WAVE", "music.lpk 首条应为 WAVE");
}
