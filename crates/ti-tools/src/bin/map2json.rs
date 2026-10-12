//! `map2json` —— `.Map` 场景文件导出 JSON（S1.4）。
//!
//! 用法：`cargo run -p ti-tools --bin map2json -- [--out DIR]`
//!
//! 导出内容：头部（width/height/flags/reserved）+ 完整 u32 字流，
//! 保证「JSON → 二进制 roundtrip 无损」（P1-1）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_formats::MapScene;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn main() -> ExitCode {
    let mut out: Option<PathBuf> = None;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--out" {
            out = it.next().map(PathBuf::from);
        }
    }
    match run(out.as_deref()) {
        Ok(n) => {
            eprintln!("[map2json] 导出 {n} 个场景");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[map2json] 失败: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(out: Option<&Path>) -> Result<usize, String> {
    let data_dir = repo_root().join("legacy").join("data");
    if let Some(dir) = out {
        fs::create_dir_all(dir).map_err(|e| format!("建目录 {}: {e}", dir.display()))?;
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&data_dir)
        .map_err(|e| format!("读 data/: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("map"))
        })
        .collect();
    files.sort();

    let mut n = 0usize;
    for f in &files {
        let raw = fs::read(f).map_err(|e| format!("读 {}: {e}", f.display()))?;
        let name = f
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("文件名非法: {}", f.display()))?;
        let scene = MapScene::parse(name, &raw).map_err(|e| format!("解析 {name}: {e}"))?;

        // roundtrip 自检：还原字节必须与原文件完全一致
        if scene.to_bytes() != raw {
            return Err(format!("{name} roundtrip 不一致（0 未知字节校验失败）"));
        }

        let json = scene.to_json(name);
        match out {
            Some(dir) => {
                let target = dir.join(format!("{name}.json"));
                fs::write(&target, json).map_err(|e| format!("写 {}: {e}", target.display()))?;
            }
            None => print!("{json}"),
        }
        n += 1;
    }
    Ok(n)
}
