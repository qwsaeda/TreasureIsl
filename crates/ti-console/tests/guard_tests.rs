#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S32.2 guard 模块单测（src 内联 mod tests 迁移；gate tests/ 目录内允许 unwrap/expect）。

use ti_console::repo::guard::{check_relative, resolve_guarded};

#[test]
fn reject_dotdot() {
    let repo_sources: Vec<String> = vec!["docs/".into(), "evidence/".into()];
    assert!(check_relative("../x", &repo_sources).is_err());
}

#[test]
fn reject_outside_whitelist() {
    let repo_sources: Vec<String> = vec!["docs/".into()];
    let e = check_relative("evidence/x", &repo_sources).unwrap_err();
    assert!(!e.reason.trim().is_empty(), "{e:?}");
}

#[test]
fn allow_inside_whitelist() {
    let repo_sources: Vec<String> = vec!["docs/".into()];
    assert!(check_relative("docs/a.md", &repo_sources).is_ok());
}

#[test]
fn resolve_guarded_returns_canonical() {
    let tmp = std::env::temp_dir();
    let rel = "ti_guard_probe.txt";
    let target = tmp.join(rel);
    std::fs::write(&target, b"x").unwrap();
    let repo_sources: Vec<String> =
        vec![format!("{}/", tmp.display()), "ti_guard_probe.txt".into()];
    let got = resolve_guarded(tmp.as_path(), &repo_sources, rel).unwrap();
    assert_eq!(got, target.canonicalize().unwrap());
    let _ = std::fs::remove_file(&target);
}
