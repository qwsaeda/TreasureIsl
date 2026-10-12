//! `unpak` —— `LucencyPackage`（`.pak`/`.lpk`）解包 / 校验 / 重打包（S1.2）。
//!
//! 用法：
//! ```text
//! cargo run -p ti-tools --bin unpak -- --check-all          # P0-1 提取率 100%
//! cargo run -p ti-tools --bin unpak -- --repack             # P0-3 sha256 逐字节一致
//! cargo run -p ti-tools --bin unpak -- --out assets_raw     # 导出全部条目
//! ```
//!
//! 条目按 `key` 命名导出（`<容器>/<key:08x>.bin`）；名称型 key 可用
//! `--names <exe>` 反查成原资源名。

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_formats::Package;

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

fn parse_args() -> (Option<String>, Option<PathBuf>, bool, bool) {
    let mut out = None;
    let mut names = None;
    let mut check_all = false;
    let mut repack = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--check-all" => check_all = true,
            "--repack" => repack = true,
            "--out" => out = it.next(),
            "--names" => names = it.next().map(PathBuf::from),
            _ => {}
        }
    }
    (out, names, check_all, repack)
}

fn main() -> ExitCode {
    let (out, names, check_all, repack) = parse_args();
    if !check_all && !repack && out.is_none() {
        eprintln!("用法: unpak [--check-all] [--repack] [--out DIR] [--names EXE]");
        return ExitCode::FAILURE;
    }
    match run(out.as_deref(), names.as_deref(), check_all, repack) {
        Ok(true) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn run(
    out: Option<&str>,
    names_exe: Option<&Path>,
    check_all: bool,
    repack: bool,
) -> Result<bool, String> {
    let root = repo_root();
    let name_map = match names_exe {
        Some(p) => Some(load_name_map(p)?),
        None => None,
    };
    let out_root = out.map(PathBuf::from);

    let mut total = 0usize;
    let mut extracted = 0usize;
    let mut all_ok = true;

    for c in CONTAINERS {
        let path = root.join("legacy").join("data").join(c);
        let buf = fs::read(&path).map_err(|e| format!("读取 {c}: {e}"))?;
        let pkg = Package::parse(c, &buf).map_err(|e| format!("解析 {c}: {e}"))?;
        total += pkg.entries.len();
        let stem = c.rsplit_once('.').map_or(c, |(s, _)| s);

        if let Some(dir) = &out_root {
            let target = dir.join(stem);
            fs::create_dir_all(&target).map_err(|e| format!("建目录 {}: {e}", target.display()))?;
            let mut manifest = String::from("key,name,usize,sha_len\n");
            for (i, e) in pkg.entries.iter().enumerate() {
                let data =
                    pkg.read_entry(&buf, i).map_err(|err| format!("解压 {c}[{i}]: {err}"))?;
                let fname = name_map
                    .as_ref()
                    .and_then(|m| m.get(&e.key))
                    .map_or_else(|| format!("{:08x}.bin", e.key), Clone::clone);
                fs::write(target.join(&fname), &data)
                    .map_err(|err| format!("写 {fname}: {err}"))?;
                let _ = writeln!(manifest, "{:08x},{},{},{}", e.key, fname, e.len, data.len());
            }
            fs::write(target.join("manifest.csv"), manifest)
                .map_err(|e| format!("写 manifest: {e}"))?;
        }

        // 提取率校验（不落盘，直接解压比对）
        if check_all || repack {
            for i in 0..pkg.entries.len() {
                match pkg.read_entry(&buf, i) {
                    Ok(d) => {
                        let want = pkg.entries[i].len as usize;
                        match d.len().cmp(&want) {
                            std::cmp::Ordering::Equal => extracted += 1,
                            std::cmp::Ordering::Less | std::cmp::Ordering::Greater => {
                                all_ok = false;
                                eprintln!(
                                    "  {c}[{i}] 解压长度 {} != usize {}",
                                    d.len(),
                                    pkg.entries[i].len
                                );
                            }
                        }
                    }
                    Err(err) => {
                        all_ok = false;
                        eprintln!("  {c}[{i}] 解压失败: {err}");
                    }
                }
            }
        }

        if repack {
            let rebuilt = pkg.repack(&buf).map_err(|e| format!("重打包 {c}: {e}"))?;
            // 索引记录可重导（.lpk 原始索引实测逐字节一致）
            if let Ok(idx) = pkg.encode_index() {
                let orig_idx = &buf[0x36..pkg.data_start];
                if !pkg.index_compressed && idx != orig_idx {
                    all_ok = false;
                    eprintln!("  {c:16} 索引重编码不一致（记录布局可能有误）");
                }
            }
            if rebuilt == buf {
                eprintln!("  {c:16} repack 逐字节一致 ({} 字节)", buf.len());
            } else {
                all_ok = false;
                eprintln!(
                    "  {c:16} repack 不一致: 原 {} 字节 vs 新 {} 字节",
                    buf.len(),
                    rebuilt.len()
                );
            }
        }

        eprintln!("[unpak] {c:16} 条目 {:4}", pkg.entries.len());
    }

    eprintln!("[unpak] 合计 {total} 条，成功解压 {extracted} 条（{:.1}%）", pct(extracted, total));
    if check_all && extracted != total {
        all_ok = false;
    }
    Ok(all_ok)
}

#[allow(clippy::cast_precision_loss)]
fn pct(a: usize, b: usize) -> f64 {
    if b == 0 {
        return 100.0;
    }
    a as f64 * 100.0 / b as f64
}

/// 从 exe 抽 UTF-16 资源名 → key 映射（名称型 key 反查）。
fn load_name_map(exe: &Path) -> Result<std::collections::HashMap<u32, String>, String> {
    let data = fs::read(exe).map_err(|e| format!("读取 exe: {e}"))?;
    let mut seen = std::collections::HashSet::new();
    let mut map = std::collections::HashMap::new();
    let mut i = 0usize;
    while i + 4 < data.len() {
        if data[i + 1] == 0 && (0x20..=0x7E).contains(&data[i]) {
            let mut j = i;
            let mut buf = Vec::new();
            while j + 1 < data.len() && data[j + 1] == 0 && (0x20..=0x7E).contains(&data[j]) {
                buf.push(data[j]);
                j += 2;
            }
            let s = String::from_utf8_lossy(&buf).into_owned();
            if seen.insert(s.clone()) {
                map.insert(ti_formats::name_key(&s), s);
            }
            i = j + 2;
        } else {
            i += 1;
        }
    }
    Ok(map)
}
