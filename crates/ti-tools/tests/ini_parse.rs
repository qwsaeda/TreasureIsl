//! S1.6 验收：`cards.lpk` 内 UTF-16 INI 键值 100% 解析 + 往返逐字节无损。

use std::path::{Path, PathBuf};

use ti_formats::{Ini, Package};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[test]
fn all_cards_lpk_ini_entries_parse_and_roundtrip() {
    let root = repo_root();
    let buf = std::fs::read(root.join("data").join("cards.lpk")).expect("cards.lpk 存在");
    let pkg = Package::parse("cards.lpk", &buf).expect("容器解析成功");

    let mut total_ini = 0usize;
    let mut total_pairs = 0usize;
    let mut roundtrip_ok = 0usize;

    for i in 0..pkg.entries.len() {
        let raw = pkg.read_entry(&buf, i).expect("条目解压成功");
        // 非 UTF-16LE INI（如二进制表）跳过
        if raw.len() < 4 || !(raw[0] == 0xFF && raw[1] == 0xFE) {
            continue;
        }
        total_ini += 1;
        let ini = Ini::parse_utf16le("cards.lpk", &raw).expect("INI 解析成功");
        assert!(!ini.entries.is_empty(), "INI[{i}] 应至少解析出键值对");
        total_pairs += ini.entries.len();

        // 往返无损
        let re = ini.to_utf16le();
        assert_eq!(re.len(), raw.len(), "INI[{i}] 往返长度不一致");
        assert!(re == raw, "INI[{i}] 往返字节不一致");
        roundtrip_ok += 1;

        // 所有键值对必须有非空段名与键名
        for e in &ini.entries {
            assert!(!e.key.is_empty(), "INI[{i}] 存在空键名");
        }
    }

    assert_eq!(total_ini, 2, "cards.lpk 应有 2 个 UTF-16 INI（cards.ini + event.ini）");
    assert!(total_pairs > 50, "键值对过少: {total_pairs}");
    assert_eq!(roundtrip_ok, total_ini);
    eprintln!("[s1.6] {total_ini} 个 INI，{total_pairs} 个键值对全部解析 + 往返无损");
}

#[test]
fn cards_ini_has_card_catalog() {
    // 实测（djb2 小写）：event.ini=0x43104390（索引 0）、cards.ini=0xCA00521B（索引 3）
    // cards.ini 内容 = [card] imageCount=2 cardCount=26 + [imageN]/[cardN]
    let root = repo_root();
    let buf = std::fs::read(root.join("data").join("cards.lpk")).expect("cards.lpk 存在");
    let pkg = Package::parse("cards.lpk", &buf).expect("解析成功");
    assert_eq!(pkg.find("event.ini"), Some(0), "event.ini 应是首条");
    let idx = pkg.find("cards.ini").expect("cards.ini 应存在");
    let raw = pkg.read_entry(&buf, idx).expect("解压成功");
    let ini = Ini::parse_utf16le("cards.lpk", &raw).expect("解析成功");
    assert_eq!(ini.get("card", "cardCount"), Some("26"), "[card] cardCount 应为 26");
    assert_eq!(ini.get("card", "imageCount"), Some("2"), "[card] imageCount 应为 2");
    assert_eq!(ini.get("image1", "file"), Some("cards\\card1.pcx"), "[image1] file");
    let j = ini.to_json();
    assert!(j.contains("\"section\""), "{j}");
    assert!(j.contains("cardCount"), "{j}");
}

#[test]
fn event_ini_entries_have_ids_and_names() {
    let root = repo_root();
    let buf = std::fs::read(root.join("data").join("cards.lpk")).expect("cards.lpk 存在");
    let pkg = Package::parse("cards.lpk", &buf).expect("解析成功");
    let idx = pkg.find("event.ini").expect("event.ini 应存在");
    let raw = pkg.read_entry(&buf, idx).expect("解压成功");
    let ini = Ini::parse_utf16le("cards.lpk", &raw).expect("解析成功");
    assert_eq!(ini.get("event", "Count"), Some("22"), "[event] Count 应为 22");
    assert!(ini.raw.contains("event catalog"), "应保留注释原文");
    let ids: Vec<&str> = ini
        .entries
        .iter()
        .filter(|e| e.section.starts_with("event") && e.key.eq_ignore_ascii_case("id"))
        .map(|e| e.value.as_str())
        .collect();
    assert_eq!(ids.len(), 22, "应有 22 个事件 id");
    assert_eq!(ids[0], "1");
    assert_eq!(ids[21], "22");
}
