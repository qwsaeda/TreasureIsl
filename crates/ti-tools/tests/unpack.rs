//! S1.2 验收：解包提取率 100%（P0-1）+ 重打包逐字节一致（P0-3）。

use std::fs;
use std::path::{Path, PathBuf};

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

#[test]
fn p0_1_every_entry_extracts() {
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut failed: Vec<String> = Vec::new();
    for c in CONTAINERS {
        let buf = fs::read(repo_root().join("data").join(c)).expect("容器存在");
        let pkg = Package::parse(c, &buf).expect("解析成功");
        for i in 0..pkg.entries.len() {
            total += 1;
            match pkg.read_entry(&buf, i) {
                Ok(d) => {
                    assert_eq!(d.len(), pkg.entries[i].len as usize, "{c}[{i}] 长度不符");
                    ok += 1;
                }
                Err(err) => failed.push(format!("{c}[{i}]: {err}")),
            }
        }
    }
    assert!(failed.is_empty(), "以下条目解压失败: {failed:?}");
    assert_eq!(ok, total, "提取率必须 100%");
    eprintln!("[p0-1] {ok}/{total} 条全部提取成功");
}

#[test]
fn p0_3_repack_is_byte_identical() {
    for c in CONTAINERS {
        let buf = fs::read(repo_root().join("data").join(c)).expect("容器存在");
        let pkg = Package::parse(c, &buf).expect("解析成功");
        let rebuilt = pkg.repack(&buf).expect("重打包成功");
        assert_eq!(rebuilt.len(), buf.len(), "{c} 重打包长度不一致");
        assert!(rebuilt == buf, "{c} 重打包字节不一致");
        // 重打包后重新解析，索引必须完全等价
        let re = Package::parse(c, &rebuilt).expect("重解析成功");
        assert_eq!(re.entries, pkg.entries, "{c} 重解析索引不一致");
        assert_eq!(re.version, pkg.version, "{c} 版本号不一致");
    }
}

#[test]
fn raw_index_reencodes_byte_identically() {
    // .lpk 索引未压缩 → 由 entries 重新编码必须与原文件逐字节一致（记录布局证明）
    for c in ["cards.lpk", "music.lpk", "goods.lpk"] {
        let buf = fs::read(repo_root().join("data").join(c)).expect("容器存在");
        let pkg = Package::parse(c, &buf).expect("解析成功");
        assert!(!pkg.index_compressed, "{c} 应为原始索引");
        let idx = pkg.encode_index().expect("索引重编码成功");
        let orig = &buf[0x36..pkg.data_start];
        assert!(idx == orig, "{c} 索引重编码与原文件不一致");
    }
}

#[test]
fn all_entries_inflate_to_nonzero_payloads() {
    let root = repo_root();
    let buf = fs::read(root.join("data").join("ui.pak")).expect("ui.pak 存在");
    let pkg = Package::parse("ui.pak", &buf).expect("解析成功");
    assert_eq!(pkg.entries.len(), 122);
    for i in 0..pkg.entries.len() {
        let d = pkg.read_entry(&buf, i).expect("解压成功");
        assert!(!d.is_empty(), "ui.pak[{i}] 解压为空");
    }
}
