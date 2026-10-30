#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S32.2 config 模块单测（src 内联 mod tests 迁移；gate tests/ 目录内允许 unwrap/expect）。

use serde_json::json;
use ti_console::repo::config::{RepoSources, load};

#[test]
fn parse_ok() {
    let srcs = RepoSources::from_json(&json!({
        "repo_sources": ["plan/ledger.yaml", "docs/", "evidence/"],
        "write_whitelist": ["evidence/console/"]
    }))
    .unwrap();
    assert_eq!(srcs.repo_sources.len(), 3);
    assert_eq!(srcs.write_whitelist, vec!["evidence/console/"]);
}

#[test]
fn empty_sources_err() {
    let bad = json!({ "repo_sources": [] });
    assert!(RepoSources::from_json(&bad).is_err());
}

#[test]
fn non_string_item_errs() {
    let bad = json!({ "repo_sources": [7] });
    assert!(RepoSources::from_json(&bad).is_err());
}

#[test]
fn load_missing_err() {
    let tmp = std::env::temp_dir();
    assert!(load(&tmp.join("nonexistent_repo_root")).is_err());
}
