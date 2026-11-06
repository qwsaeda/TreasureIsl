#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S32.2 snapshot 模块单测（src 内联 mod tests 迁移；gate tests/ 目录内允许 unwrap/expect）。

use std::collections::BTreeMap;
use ti_console::repo::snapshot::{diff_report, fingerprint_kind, no_change_probe, sha256_hex};

#[test]
fn sha256_hex_stable() {
    let h = sha256_hex(b"hello");
    assert_eq!(h.len(), 32);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(h, sha256_hex(b"hello"));
}

#[test]
fn fingerprint_kind_is_content_level() {
    let kind = fingerprint_kind();
    assert!(kind.contains("content") || kind.contains("sha"));
}

#[test]
fn no_change_probe_same() {
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    m.insert("a".into(), "x".into());
    let before = ti_console::repo::snapshot::snapshot_from(m.clone());
    let after = ti_console::repo::snapshot::snapshot_from(m);
    assert!(no_change_probe(&before, &after));
}

#[test]
fn diff_report_nonempty_on_change() {
    let mut m1: BTreeMap<String, String> = BTreeMap::new();
    m1.insert("a".into(), "x".into());
    let mut m2 = m1.clone();
    m2.insert("b".into(), "y".into());
    let before = ti_console::repo::snapshot::snapshot_from(m1);
    let after = ti_console::repo::snapshot::snapshot_from(m2);
    let report = diff_report(&before, &after);
    assert!(!report.trim().is_empty(), "{report}");
}

/// S44.1 北极星写点探针（30 号 §3.2）：`write_probe(动作写点集)` 豁免登记写点，
/// 其余冻结路径差异 = 漂移；`import_probe` 收敛为 `write_probe`（S32.15 行为不变）。
#[test]
fn write_probe_exempts_registered_write_paths() {
    let mut before: BTreeMap<String, String> = BTreeMap::new();
    before.insert("assets/maps/m01/manifest.json".into(), "h1".into());
    before.insert("assets/data/board.json".into(), "h2".into());
    before.insert("crates/ti-core/src/rules/combat.rs".into(), "h3".into());
    let b = ti_console::repo::snapshot::snapshot_from(before.clone());

    // 动作写点 = assets/maps/ + board.json（mappack_install/sync 类）→ 豁免这两处，crates/ 差异 = 漂移。
    let mut after = b.files.clone();
    after.insert("assets/maps/m01/manifest.json".into(), "h1x".into()); // 写点内
    after.insert("assets/data/board.json".into(), "h2x".into()); // 写点内
    let a = ti_console::repo::snapshot::snapshot_from(after.clone());
    assert!(
        b.write_probe(&a, &["assets/maps/", "assets/data/board.json"]),
        "写点差异豁免应判 0 变更"
    );

    // 写点外差异（crates/ 产品源码）仍算漂移。
    let mut after2 = after.clone();
    after2.insert("crates/ti-core/src/rules/combat.rs".into(), "h3x".into());
    let a2 = ti_console::repo::snapshot::snapshot_from(after2);
    assert!(!b.write_probe(&a2, &["assets/maps/", "assets/data/board.json"]), "写点外差异应判漂移");
}

/// S44.1：`import_probe` 收敛为 `write_probe`（S32.15 行为不变 = 豁免 assets/maps/ + board.json）。
#[test]
fn import_probe_wrapper_matches_write_probe() {
    let mut before: BTreeMap<String, String> = BTreeMap::new();
    before.insert("assets/maps/m01/manifest.json".into(), "h1".into());
    before.insert("crates/ti-core/src/rules/combat.rs".into(), "h2".into());
    let b = ti_console::repo::snapshot::snapshot_from(before.clone());

    let mut after = before.clone();
    after.insert("assets/maps/m01/manifest.json".into(), "h1x".into());
    let a = ti_console::repo::snapshot::snapshot_from(after);
    assert!(b.import_probe(&a), "import 写点（assets/maps/）应豁免");
    assert!(b.write_probe(&a, &["assets/maps/", "assets/data/board.json"]), "write_probe 同口径");
}
