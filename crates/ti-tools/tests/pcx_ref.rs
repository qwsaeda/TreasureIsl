//! S1.3 验收：PCX 解码与独立 Python 参考实现**逐像素一致**（P0-4）。
//!
//! 参考实现 `scripts/pcx_ref.py`（无第三方依赖）输出 `WIDTH HEIGHT SHA256(pixels)`；
//! 本测试对同一批容器条目比对 Rust 解码结果，sha256 必须完全相同。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ti_formats::{Package, pcx};

const CONTAINERS: [&str; 4] = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak"];
const SAMPLE_PER_CONTAINER: usize = 12;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn fingerprint(bytes: &[u8]) -> String {
    let mut h1: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h1 ^= u64::from(*b);
        h1 = h1.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h1:016x}")
}

#[test]
fn rust_decoder_matches_python_reference() {
    let root = repo_root();
    let script = root.join("scripts").join("pcx_ref.py");
    let py = std::env::var("PYTHON").unwrap_or_else(|_| "python".to_string());
    let tmp = std::env::temp_dir().join("ti_pcx_ref");
    let _ = fs::create_dir_all(&tmp);

    let mut checked = 0usize;
    let mut mismatches: Vec<String> = Vec::new();

    for c in CONTAINERS {
        let buf = fs::read(root.join("data").join(c)).expect("容器存在");
        let pkg = Package::parse(c, &buf).expect("解析成功");
        let step = (pkg.entries.len() / SAMPLE_PER_CONTAINER).max(1);
        for i in (0..pkg.entries.len()).step_by(step) {
            let Ok(raw) = pkg.read_entry(&buf, i) else {
                continue;
            };
            if raw.len() < 128 || raw[0] != 0x0A {
                continue;
            }
            let img = match pcx::Image::decode("x", &raw) {
                Ok(v) => v,
                Err(err) => {
                    mismatches.push(format!("{c}[{i}] Rust 解码失败: {err}"));
                    continue;
                }
            };

            let f = tmp.join(format!("{c}_{i}.pcx"));
            fs::write(&f, &raw).expect("写临时 pcx");
            let out =
                Command::new(&py).arg(&script).arg(&f).output().expect("运行 python 参考实现");
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let parts: Vec<&str> = text.split_whitespace().collect();
            if parts.len() != 3 {
                mismatches.push(format!(
                    "{c}[{i}] 参考实现输出异常: {text:?} / {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
                continue;
            }
            let pw: u16 = parts[0].parse().unwrap_or(0);
            let ph: u16 = parts[1].parse().unwrap_or(0);
            let ref_hash = parts[2];

            let our_hash = fingerprint(&img.pixels);
            if pw != img.width || ph != img.height || ref_hash != our_hash {
                mismatches.push(format!(
                    "{c}[{i}] 不一致: py={pw}x{ph}/{ref_hash} rust={}x{}/{our_hash}",
                    img.width, img.height
                ));
            } else {
                checked += 1;
            }
            let _ = fs::remove_file(&f);
        }
    }

    assert!(checked > 0, "未比对任何 PCX");
    assert!(mismatches.is_empty(), "不一致 {} 处: {mismatches:#?}", mismatches.len());
    eprintln!("[p0-4] {checked} 张 PCX 与 Python 参考实现逐像素一致");
}

#[test]
fn first_ui_pak_entry_matches_f205() {
    // F-205: ui.pak 首条 = PCX 头 0A 05 01 08，8bpp 1 平面 34×12
    let root = repo_root();
    let buf = fs::read(root.join("data").join("ui.pak")).expect("ui.pak 存在");
    let pkg = Package::parse("ui.pak", &buf).expect("解析成功");
    let img = pcx::decode_from_package("ui.pak[0]", &pkg, &buf, 0).expect("解码成功");
    assert_eq!((img.width, img.height), (34, 12), "F-205 尺寸不符");
    assert_eq!(img.pixels.len(), 34 * 12);
    assert!(!img.is_empty());
}

#[test]
fn all_image_entries_in_glpics_decode() {
    let root = repo_root();
    let buf = fs::read(root.join("data").join("glpics.pak")).expect("glpics.pak 存在");
    let pkg = Package::parse("glpics.pak", &buf).expect("解析成功");
    let mut decoded = 0usize;
    let mut failed: Vec<String> = Vec::new();
    for i in 0..pkg.entries.len() {
        let raw = pkg.read_entry(&buf, i).expect("解压成功");
        if raw.first() == Some(&0x0A) {
            match pcx::Image::decode("x", &raw) {
                Ok(img) => {
                    assert_eq!(img.pixels.len(), img.len(), "{i} 像素数与尺寸不符");
                    decoded += 1;
                }
                Err(err) => failed.push(format!("{i}: {err}")),
            }
        }
    }
    assert!(decoded >= 80, "glpics.pak 应有 ≥80 张 PCX，实际 {decoded}");
    assert!(failed.is_empty(), "以下条目解码失败: {failed:?}");
    eprintln!("[p0-4] glpics.pak {decoded} 张 PCX 全部解码成功");
}
