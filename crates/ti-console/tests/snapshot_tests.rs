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
