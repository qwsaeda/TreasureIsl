#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S32.2 data 模块单测（src 内联 mod tests 迁移；gate tests/ 目录内允许 unwrap/expect）。

use serde_json::json;
use ti_console::repo::config::RepoSources;
use ti_console::repo::data::{evidence_list, map_packs_detail};

fn test_sources() -> RepoSources {
    RepoSources::from_json(&json!({
        "repo_sources": ["plan/ledger.yaml", "docs/", "evidence/", "assets/maps/"],
        "write_whitelist": ["evidence/console/"]
    }))
    .unwrap()
}

#[test]
fn evidence_list_lists_dir() {
    let tmp = std::env::temp_dir().join("ti_console_ev_test");
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::create_dir_all(tmp.join("evidence/console"));
    let _ = std::fs::write(tmp.join("evidence/A.log"), "x");
    let src = test_sources();
    let v = evidence_list(&tmp, &src, "").unwrap();
    let names: Vec<String> = v["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f.as_str())
        .map(ToOwned::to_owned)
        .collect();
    assert!(names.iter().any(|n| n == "A.log"));
    assert!(names.iter().any(|n| n == "console/"));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn evidence_list_sub_dir() {
    let tmp = std::env::temp_dir().join("ti_console_ev_sub");
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::create_dir_all(tmp.join("evidence/ui_concept"));
    let _ = std::fs::write(tmp.join("evidence/ui_concept/x.png"), "i");
    let src = test_sources();
    let v = evidence_list(&tmp, &src, "ui_concept").unwrap();
    let files = v["files"].as_array().unwrap();
    // sub 面返回带 `evidence/{sub}/` 前缀的完整相对路径（数据面口径）。
    assert!(files.iter().any(|f| f.as_str() == Some("evidence/ui_concept/x.png")));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn map_packs_detail_lists() {
    let tmp = std::env::temp_dir().join("ti_console_mp");
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::create_dir_all(tmp.join("assets/maps/m01"));
    let _ = std::fs::write(
        tmp.join("assets/maps/m01/manifest.json"),
        r#"{"id":"m01","version":"1","map":{"spaces":[1,2]}}"#,
    );
    // m04 = 草稿态：manifest 在位但无 map 键（S16.4 在途面，valid=false 不崩）。
    let _ = std::fs::create_dir_all(tmp.join("assets/maps/m04"));
    let _ = std::fs::write(
        tmp.join("assets/maps/m04/manifest.json"),
        r#"{"id":"m04","version":"0.1.0-draft"}"#,
    );
    let _ = std::fs::write(tmp.join("assets/maps/.default"), "m01\n");
    let src = test_sources();
    let v = map_packs_detail(&tmp, &src).unwrap();
    let packs = v["packs"].as_array().unwrap();
    assert_eq!(packs.len(), 2);
    assert_eq!(packs[0]["id"], "m01");
    assert_eq!(packs[0]["valid"], true);
    assert_eq!(packs[1]["id"], "m04");
    assert_eq!(packs[1]["valid"], false);
    assert!(packs[1]["draft_note"].is_string());
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn in_sources_hit() {
    use ti_console::repo::data::in_sources;
    let src = test_sources();
    assert!(in_sources("docs/a.md", &src));
    assert!(!in_sources("secrets/x", &src));
}
