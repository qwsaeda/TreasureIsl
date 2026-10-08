//! S2.4 验收：三端存储语义一致性（内存 / 桌面文件）+ 存读档往返。
//!
//! wasm(IndexedDB) 与 Android(内部存储) 后端在 S2.5/S2.6 接入 macroquad 时，
//! 必须复用本文件定义的 `Backend` 语义基准（同一套断言跑三遍）。

use std::path::PathBuf;

use ti_platform::storage::{Backend, FileStore, MemoryStore, validate_key};

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ti_store_{tag}"));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn save_blob(slot: u32, day: u16) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"TI-SAVE-v1");
    v.extend_from_slice(&slot.to_le_bytes());
    v.extend_from_slice(&day.to_le_bytes());
    v.extend_from_slice(
        &(0..32u16).map(|i| u8::try_from(i % 256).unwrap_or(0)).collect::<Vec<u8>>(),
    );
    v
}

fn keys_are_valid() {
    for good in ["gl0.sav", "gl.rec", "settings.dat", "slot_1", "A-B_c.1"] {
        assert!(validate_key(good), "{good} 应合法");
    }
    for bad in ["", "a/b", "a\\b", "a b", "../x", "键", &"x".repeat(65)] {
        assert!(!validate_key(bad), "{bad} 应非法");
    }
}

#[test]
fn key_validation_rules() {
    keys_are_valid();
}

#[test]
fn memory_store_roundtrip() {
    keys_are_valid();
    let mut s = MemoryStore::new();
    assert!(s.is_empty());
    let blob = save_blob(3, 12345);
    s.store("gl0.sav", &blob).expect("写入成功");
    assert_eq!(s.len(), 1);
    assert_eq!(s.bytes_used(), blob.len());
    let back = s.load("gl0.sav").expect("读取成功");
    assert_eq!(back, Some(blob.clone()), "读回必须逐字节一致");
    assert_eq!(s.load("missing.sav").expect("读取成功"), None);
    s.remove("gl0.sav").expect("删除成功");
    assert_eq!(s.load("gl0.sav").expect("读取成功"), None);
    s.remove("gl0.sav").expect("重复删除视为成功");
}

#[test]
fn file_store_roundtrip() {
    keys_are_valid();
    let dir = tmp_dir("file");
    let mut s = FileStore::new(&dir).expect("构造成功");
    assert_eq!(s.load("gl0.sav").expect("读取成功"), None);
    let blob = save_blob(1, 999);
    s.store("gl0.sav", &blob).expect("写入成功");
    let back = s.load("gl0.sav").expect("读取成功");
    assert_eq!(back, Some(blob.clone()), "文件存储读回必须逐字节一致");
    s.store("gl1.sav", &save_blob(2, 1000)).expect("写入成功");
    let keys = s.keys().expect("列举成功");
    assert_eq!(keys, vec!["gl0.sav", "gl1.sav"], "键名应升序");
    s.remove("gl0.sav").expect("删除成功");
    assert_eq!(s.load("gl0.sav").expect("读取成功"), None);
    assert_eq!(s.keys().expect("列举成功"), vec!["gl1.sav"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn all_backends_share_identical_semantics() {
    keys_are_valid();
    let dir = tmp_dir("parity");
    let mut mem = MemoryStore::new();
    let mut file = FileStore::new(&dir).expect("构造成功");

    let cases: [(&str, Vec<u8>); 3] = [
        ("gl0.sav", save_blob(0, 1)),
        ("gl.rec", save_blob(1, 2)),
        ("settings.dat", b"\x5D\x4A\x71\x4A\x2E\x01\x01\x01".to_vec()),
    ];
    for (k, v) in &cases {
        mem.store(k, v).expect("mem 写入");
        file.store(k, v).expect("file 写入");
    }
    for (k, v) in &cases {
        assert_eq!(
            mem.load(k).expect("mem 读"),
            file.load(k).expect("file 读"),
            "键 {k} 两端不一致"
        );
        assert_eq!(mem.load(k).expect("mem 读"), Some(v.clone()));
    }
    assert_eq!(mem.keys().expect("mem 键"), file.keys().expect("file 键"), "键集合不一致");

    // 删除语义一致
    mem.remove("gl.rec").expect("mem 删除");
    file.remove("gl.rec").expect("file 删除");
    assert_eq!(mem.load("gl.rec").expect("mem 读"), file.load("gl.rec").expect("file 读"));

    // 非法键在三端都必须失败
    assert!(mem.store("bad/key", &[1]).is_err());
    assert!(file.store("bad/key", &[1]).is_err());
    assert!(mem.load("bad/key").is_err());
    assert!(file.load("bad/key").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn large_save_blob_roundtrips() {
    keys_are_valid();
    let dir = tmp_dir("big");
    let mut mem = MemoryStore::new();
    let mut file = FileStore::new(&dir).expect("构造成功");
    let big: Vec<u8> = (0..512 * 1024u32).map(|i| (i % 251) as u8).collect();
    mem.store("gl0.sav", &big).expect("mem 写入");
    file.store("gl0.sav", &big).expect("file 写入");
    assert_eq!(mem.load("gl0.sav").expect("mem 读"), Some(big.clone()));
    assert_eq!(file.load("gl0.sav").expect("file 读"), Some(big));
    assert_eq!(mem.bytes_used(), 512 * 1024);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn overwrite_keeps_single_entry() {
    keys_are_valid();
    let dir = tmp_dir("over");
    let mut mem = MemoryStore::new();
    let mut file = FileStore::new(&dir).expect("构造成功");
    mem.store("gl0.sav", b"v1").expect("mem 写");
    file.store("gl0.sav", b"v1").expect("file 写");
    mem.store("gl0.sav", b"v2-longer").expect("mem 覆盖");
    file.store("gl0.sav", b"v2-longer").expect("file 覆盖");
    assert_eq!(mem.len(), 1, "覆盖不应新增条目");
    assert_eq!(mem.load("gl0.sav").expect("mem 读"), Some(b"v2-longer".to_vec()));
    assert_eq!(file.load("gl0.sav").expect("file 读"), Some(b"v2-longer".to_vec()));
    let _ = std::fs::remove_dir_all(&dir);
}
