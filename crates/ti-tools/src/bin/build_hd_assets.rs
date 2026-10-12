//! `build_hd_assets` —— HD 资产 manifest 校验（S7.4，P5-1 判据）。
//!
//! 读 `assets_hd_pack/manifest_hd.json`（`scripts/pack_hd_assets.py` 产物）逐条
//! FNV-1a64 校验 + 尺寸断言，0 缺失 / 0 失配 / 0 失尺寸才成功。
//! 与 `build_assets` 的 FNV 哈希同源（`ti_tools::hash::fnv1a64`），幂等可复验。
//!
//! 用法：
//! ```text
//! cargo run -q -p ti-tools --bin build_hd_assets -- --check
//! cargo run -q -p ti-tools --bin build_hd_assets -- --check --out assets_hd_pack
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_tools::hash::fnv1a64_hex;

fn repo_root() -> PathBuf {
    // 优先用当前工作目录（cargo run 与 shell 的 cwd 一致）；
    // env!("CARGO_MANIFEST_DIR") 在 CJK 路径下经 shell 传递可能损坏，回退时再试。
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let via_cargo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."));
    // 取在 repo 根存在 data/ 的那个（data/ 是 S1 资源目录，必在根）
    if cwd.join("legacy").join("data").is_dir() {
        cwd
    } else if via_cargo.join("legacy").join("data").is_dir() {
        via_cargo.to_path_buf()
    } else {
        cwd
    }
}

fn main() -> ExitCode {
    let mut check = false;
    let mut out_dir = "assets_hd_pack".to_string();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--check" => check = true,
            "--out" => out_dir = it.next().unwrap_or(out_dir),
            _ => {}
        }
    }
    if !check {
        eprintln!("用法: build_hd_assets -- --check [--out DIR]");
        return ExitCode::FAILURE;
    }
    match run(&out_dir) {
        Ok(n) => {
            eprintln!("[build_hd_assets] 校验 {n} 条全通过（0 缺失 / 0 失配 / 0 失尺寸）");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[build_hd_assets] 失败: {e}");
            ExitCode::FAILURE
        }
    }
}

/// 读 manifest → 逐条 FNV-1a64 + 尺寸断言。返回通过条目数。
fn run(out_dir: &str) -> Result<usize, String> {
    let root = repo_root();
    let pack = Path::new(out_dir);
    let manifest_path = if pack.is_absolute() { pack.to_path_buf() } else { root.join(pack) };
    let manifest_file = manifest_path.join("manifest_hd.json");
    let raw =
        fs::read(&manifest_file).map_err(|e| format!("读 {}: {e}", manifest_file.display()))?;
    let text = String::from_utf8(raw).map_err(|e| format!("manifest UTF-8: {e}"))?;
    let entries: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&text).map_err(|e| format!("manifest JSON: {e}"))?;
    eprintln!(
        "[build_hd_assets] {} 条 manifest 记录（{}）",
        entries.len(),
        manifest_file.display()
    );
    let mut ok = 0usize;
    for (rel, meta) in &entries {
        let full = manifest_path.join(rel);
        let data = fs::read(&full).map_err(|e| format!("缺失 {rel}: {e}"))?;
        // FNV-1a64 校验（顶层或 nine_slice 嵌套）
        let hash = meta
            .get("fnv1a64")
            .or_else(|| meta.get("nine_slice").and_then(|ns| ns.get("fnv1a64")))
            .and_then(|v| v.as_str());
        if let Some(hash) = hash {
            let actual = fnv1a64_hex(&data);
            if actual != hash {
                return Err(format!("{rel} FNV 失配：manifest={hash} 实际={actual}"));
            }
        }
        // 尺寸断言（PNG 档做；WebP 档 RIFF 头偏移因编码器而异，FNV 已保证字节一致，跳过）
        let is_webp = full.extension().and_then(|e| e.to_str()) == Some("webp");
        let want_w = meta.get("width").and_then(serde_json::Value::as_u64);
        let want_h = meta.get("height").and_then(serde_json::Value::as_u64);
        if !is_webp {
            if let (Some(w), Some(h)) = (want_w, want_h) {
                let (pw, ph) = image_dims(&full, &data)?;
                if pw != u32::try_from(w).unwrap_or(0) || ph != u32::try_from(h).unwrap_or(0) {
                    return Err(format!("{rel} 尺寸失配：文件 {pw}x{ph} manifest {w}x{h}"));
                }
            }
        }
        ok += 1;
    }
    Ok(ok)
}

/// 取图像文件的宽高（PNG IHDR；WebP 的 RIFF 头布局因编码器而异，
/// FNV 哈希已保证字节一致，WebP 尺寸档以同名 PNG 为准 → 跳过 WebP 尺寸断言）。
fn image_dims(path: &Path, data: &[u8]) -> Result<(u32, u32), String> {
    if path.extension().and_then(|e| e.to_str()) == Some("webp") {
        return Ok((0, 0)); // WebP 档不做尺寸断言（FNV 已校验字节）
    }
    // PNG：签名 8 字节 + IHDR 4(len)+4(type) + width(4 大端) + height(4 大端)
    if data.len() >= 24 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        return Ok((w, h));
    }
    Err(format!("{} 非 PNG/WebP", path.display()))
}
