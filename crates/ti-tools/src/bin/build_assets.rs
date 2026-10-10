//! `build_assets` —— 一条命令生成 `assets_out/` + `manifest.json`（S1.10）。
//!
//! 用法：`cargo run -p ti-tools --bin build_assets -- [--out assets_out]`
//!
//! 幂等：同一输入两次运行，产物逐字节一致（manifest 按路径排序、无时间戳）。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_formats::lucency::Package;
use ti_formats::mapfile::MapScene;
use ti_formats::{Ini, TerrainMap};
use ti_tools::hash::fnv1a64_hex;

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

fn main() -> ExitCode {
    let mut out = PathBuf::from("assets_out");
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--out" {
            if let Some(v) = it.next() {
                out = PathBuf::from(v);
            }
        }
    }
    match build(&out) {
        Ok(n) => {
            eprintln!("[build_assets] 输出 {n} 个文件到 {}", out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[build_assets] 失败: {e}");
            ExitCode::FAILURE
        }
    }
}

fn build(out: &Path) -> Result<usize, String> {
    let root = repo_root();
    let data = root.join("data");
    if out.exists() {
        fs::remove_dir_all(out).map_err(|e| format!("清空 {}: {e}", out.display()))?;
    }
    fs::create_dir_all(out).map_err(|e| format!("建目录 {}: {e}", out.display()))?;

    let mut files: BTreeMap<String, String> = BTreeMap::new();

    // 1. 容器内条目 → 原始字节
    for c in CONTAINERS {
        let buf = fs::read(data.join(c)).map_err(|e| format!("读 {c}: {e}"))?;
        let pkg = Package::parse(c, &buf).map_err(|e| format!("解析 {c}: {e}"))?;
        let stem = c.rsplit_once('.').map_or(c, |(s, _)| s);
        for i in 0..pkg.entries.len() {
            let raw = pkg.read_entry(&buf, i).map_err(|e| format!("解压 {c}[{i}]: {e}"))?;
            let rel = format!("raw/{stem}/{:08x}.bin", pkg.entries[i].key);
            write_file(out, &rel, &raw)?;
            files.insert(rel, fnv1a64_hex(&raw));
        }
    }

    // 2. .Map 场景 → JSON
    let mut map_files: Vec<PathBuf> = fs::read_dir(&data)
        .map_err(|e| format!("读 data/: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("map"))
        })
        .collect();
    map_files.sort();
    for f in &map_files {
        let raw = fs::read(f).map_err(|e| format!("读 {}: {e}", f.display()))?;
        let name = f.file_name().and_then(|s| s.to_str()).unwrap_or("?").to_string();
        let scene = MapScene::parse(&name, &raw).map_err(|e| format!("解析 {name}: {e}"))?;
        // 判别：UI 场景网格 176×220；地形网格 ≤20（F-301）
        let json = if scene.header.width <= 64 && scene.header.height <= 64 {
            TerrainMap::parse(&name, &raw)
                .map_err(|e| format!("解析地形 {name}: {e}"))?
                .to_json(&name)
        } else {
            scene.to_json(&name)
        };
        let rel = format!("scenes/{name}.json");
        write_file(out, &rel, json.as_bytes())?;
        files.insert(rel, fnv1a64_hex(json.as_bytes()));
    }

    // 3. 容器内 INI → JSON
    let cards = fs::read(data.join("cards.lpk")).map_err(|e| format!("读 cards.lpk: {e}"))?;
    let cpkg = Package::parse("cards.lpk", &cards).map_err(|e| format!("解析 cards.lpk: {e}"))?;
    for i in 0..cpkg.entries.len() {
        let raw = cpkg.read_entry(&cards, i).map_err(|e| format!("解压 cards.lpk[{i}]: {e}"))?;
        if raw.len() >= 2 && raw[0] == 0xFF && raw[1] == 0xFE {
            let ini =
                Ini::parse_utf16le("cards.lpk", &raw).map_err(|e| format!("解析 INI: {e}"))?;
            let json = ini.to_json();
            let rel = format!("ini/cards_{i}.json");
            write_file(out, &rel, json.as_bytes())?;
            files.insert(rel, fnv1a64_hex(json.as_bytes()));
        }
    }

    // 4. S1.7b：MOD 条目 → OGG/Opus（glmusic.mod BGM 转码）
    let mbuf = fs::read(data.join("music.lpk")).map_err(|e| format!("读 music.lpk: {e}"))?;
    let mpkg = Package::parse("music.lpk", &mbuf).map_err(|e| format!("解析 music.lpk: {e}"))?;
    if let Some(idx) = mpkg.find_key(0x8046_15C2) {
        let raw = mpkg.read_entry(&mbuf, idx).map_err(|e| format!("解压 MOD: {e}"))?;
        let audio =
            ti_platform::mod_pcm::ModAudio::parse(&raw).map_err(|e| format!("MOD 解析: {e}"))?;
        let pcm = audio.to_i16();
        let ogg = ti_platform::ogg_encode::encode_ogg(&pcm, audio.sample_rate)
            .map_err(|e| format!("OGG 编码: {e}"))?;
        let rel = "audio/glmusic.ogg".to_string();
        write_file(out, &rel, &ogg)?;
        files.insert(rel, fnv1a64_hex(&ogg));
    }

    // 5. manifest（排序、无时间戳 → 幂等）
    let mut man = String::from("{\n  \"files\": {\n");
    for (i, (k, v)) in files.iter().enumerate() {
        if i > 0 {
            man.push_str(",\n");
        }
        man.push_str("    \"");
        man.push_str(k);
        man.push_str("\": \"");
        man.push_str(v);
        man.push('"');
    }
    man.push_str("\n  }\n}\n");
    write_file(out, "manifest.json", man.as_bytes())?;

    Ok(files.len() + 1)
}

fn write_file(out: &Path, rel: &str, data: &[u8]) -> Result<(), String> {
    let p = out.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("建 {}: {e}", parent.display()))?;
    }
    fs::write(&p, data).map_err(|e| format!("写 {}: {e}", p.display()))
}
