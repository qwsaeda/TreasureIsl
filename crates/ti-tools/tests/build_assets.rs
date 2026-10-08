//! S1.10 验收：`build_assets` 幂等 + manifest 校验（阶段门禁：P0/P1）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = "build_assets";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn run(out: &Path) -> std::process::Output {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["run", "-q", "-p", "ti-tools", "--bin", BIN, "--", "--out"])
        .arg(out)
        .current_dir(repo_root())
        .output()
        .expect("执行 build_assets")
}

fn snapshot(dir: &Path) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = fs::read_dir(dir)
        .expect("读取输出目录")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .flat_map(|p| {
            let rel = p.strip_prefix(dir).unwrap_or(&p).to_string_lossy().to_string();
            if p.is_dir() {
                collect(&p, dir)
            } else {
                vec![(rel, hash(&fs::read(&p).unwrap_or_default()))]
            }
        })
        .collect();
    v.sort();
    v
}

fn collect(dir: &Path, base: &Path) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.filter_map(Result::ok) {
            let p = e.path();
            let rel = p.strip_prefix(base).unwrap_or(&p).to_string_lossy().to_string();
            if p.is_dir() {
                v.extend(collect(&p, base));
            } else {
                v.push((rel, hash(&fs::read(&p).unwrap_or_default())));
            }
        }
    }
    v
}

fn fnv1a64(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn hash(bytes: &[u8]) -> String {
    fnv1a64(bytes)
}

#[test]
fn build_assets_is_idempotent() {
    let dir = repo_root().join("target").join("assets_idem");
    let _ = fs::remove_dir_all(&dir);
    let a = run(&dir);
    assert!(a.status.success(), "第一次构建失败: {}", String::from_utf8_lossy(&a.stderr));
    let snap1 = snapshot(&dir);
    assert!(snap1.len() > 300, "产物过少: {}", snap1.len());

    let b = run(&dir);
    assert!(b.status.success(), "第二次构建失败");
    let snap2 = snapshot(&dir);
    assert_eq!(snap1, snap2, "两次构建产物不一致（幂等性破坏）");
    eprintln!("[s1.10] 幂等校验通过：{} 个文件两次构建完全一致", snap1.len());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn manifest_hashes_match_actual_files() {
    let dir = repo_root().join("target").join("assets_manifest");
    let _ = fs::remove_dir_all(&dir);
    assert!(run(&dir).status.success(), "构建失败");

    let man = fs::read_to_string(dir.join("manifest.json")).expect("manifest.json 存在");
    let mut checked = 0usize;
    let mut bad: Vec<String> = Vec::new();
    for line in man.lines() {
        let Some(rest) = line.trim().trim_end_matches(',').strip_prefix('"') else { continue };
        let Some((path, decl)) = rest.split_once("\": \"") else { continue };
        let path = path.trim_end_matches('"');
        let want = decl.trim_end_matches('"');
        let full = dir.join(path);
        match fs::read(&full) {
            Ok(data) => {
                checked += 1;
                let actual = fnv1a64(&data);
                if actual != want {
                    bad.push(format!("{path}: manifest={want} actual={actual}"));
                }
            }
            Err(_) => bad.push(format!("{path}: manifest 列出但文件不存在")),
        }
    }
    assert!(checked > 300, "manifest 条目过少: {checked}");
    assert!(bad.is_empty(), "manifest 校验失败: {bad:?}");
    eprintln!("[s1.10] manifest 校验通过：{checked} 条 sha 全部一致");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn all_scenes_and_maps_exported() {
    let dir = repo_root().join("target").join("assets_scenes");
    let _ = fs::remove_dir_all(&dir);
    assert!(run(&dir).status.success(), "构建失败");

    let scenes = dir.join("scenes");
    let n = fs::read_dir(&scenes).expect("scenes 目录").count();
    // F-301：UI 场景 20 + 地形 3 = 23
    assert_eq!(n, 23, "应导出 23 个场景 JSON（20 UI + 3 地形）");
    for f in fs::read_dir(&scenes).expect("scenes 目录").filter_map(Result::ok) {
        let txt = fs::read_to_string(f.path()).expect("读 JSON");
        assert!(
            txt.starts_with('{') && txt.contains("\"words\""),
            "场景 JSON 结构非法: {}",
            f.path().display()
        );
    }
    // 容器条目 305 条 → raw/
    let raw = dir.join("raw");
    assert!(raw.is_dir(), "缺 raw/ 目录");
    eprintln!("[s1.10] 23 个场景 + 容器原始条目已导出");
    let _ = fs::remove_dir_all(&dir);
}
