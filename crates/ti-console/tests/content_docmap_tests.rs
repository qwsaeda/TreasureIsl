#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! S44.5 固化收口单测：29 号 §2/§3 流程 ↔ 平台向导 **1:1 映射断言**。
//!
//! 断言三件（DoD「每步 id/判据/写面登记一致」）：
//! 1. **id** = 文档「平台向导步骤」列 ↔ 平台步骤 id（§2 严格序号 1:1；§3 锚点映射）；
//! 2. **判据** = 文档「判据」列含判据关键词（关键词 ↔ 平台 `parse_criterion` 描述不漂移）；
//! 3. **写面** = 文档「写面」列列出有效写点（白名单 `console_commands.json` 的 `writes`）；
//!    有效写点为空 ⇒ 须标「只读」。
//!
//! 正例 = 真实仓库文档零问题；反例 = 篡改文档（改步骤 id / 删判据关键词 / 删写点）⇒ 断言必报。
//! 仓库根 = `CARGO_MANIFEST_DIR/../..`（同 `close_tests` 口径，脚本须读真实仓库）。

use ti_console::content::docmap::{DOC_PATH, assert_mapping, mapping_report};

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn real_doc() -> String {
    std::fs::read_to_string(repo_root().join(DOC_PATH)).expect("29 号文档应可读")
}

#[test]
fn real_doc_mapping_is_clean() {
    // 正例：真实仓库的 29 号文档 ↔ 平台向导零漂移。
    let problems = assert_mapping(&real_doc(), &repo_root());
    assert!(problems.is_empty(), "29 号 ↔ 平台向导映射应有 0 问题，实际：{problems:?}");
}

#[test]
fn mapping_report_ok_and_counts() {
    // 映射报告：ok=true + 地图十步 + UI 六步。
    let report = mapping_report(&repo_root());
    assert_eq!(report["ok"], serde_json::json!(true), "报告应 ok");
    assert_eq!(report["map"].as_array().unwrap().len(), 10, "地图向导 10 步");
    assert_eq!(report["ui"].as_array().unwrap().len(), 7, "UI 向导 7 步（S48.0 插 theme）");
    assert_eq!(report["problems"].as_array().unwrap().len(), 0, "报告 0 问题");
}

#[test]
fn ui_anchor_mapping_merges_and_splits() {
    // §3 锚点映射：§3.2 拆 variant+layout（两平台步）；§3.5+§3.6 合 zero_rust（一步两行）。
    let report = mapping_report(&repo_root());
    let ui = report["ui"].as_array().unwrap();
    let rows_of = |step: &str| -> Vec<usize> {
        ui.iter()
            .find(|m| m["step"] == serde_json::json!(step))
            .map(|m| {
                m["doc_rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| usize::try_from(v.as_u64().unwrap()).unwrap())
                    .collect()
            })
            .unwrap()
    };
    assert_eq!(rows_of("variant"), vec![2], "§3.2 → variant");
    assert_eq!(rows_of("theme"), vec![2], "§3.2 → theme（S48.0 全主题创新）");
    assert_eq!(rows_of("layout"), vec![2], "§3.2 → layout");
    assert_eq!(rows_of("zero_rust"), vec![5, 6], "§3.5+§3.6 → zero_rust");
    assert_eq!(rows_of("design"), vec![1]);
    assert_eq!(rows_of("build"), vec![3]);
    assert_eq!(rows_of("shot"), vec![4]);
}

#[test]
fn negative_wrong_step_id_is_reported() {
    // 反例 1：篡改 §2 第 3 行的步骤 id ⇒ 断言必报 id 不一致。
    let doc = real_doc()
        .replace("| `syncmap` | `mappack_syncmap` |", "| `syncmapX` | `mappack_syncmap` |");
    assert_ne!(doc, real_doc(), "篡改应生效（文档含目标串）");
    let problems = assert_mapping(&doc, &repo_root());
    assert!(
        problems.iter().any(|p| p.contains("syncmap")),
        "应报步骤 id 不一致，实际：{problems:?}"
    );
}

#[test]
fn negative_missing_criterion_key_is_reported() {
    // 反例 2：删掉 §2 第 4 行的判据关键词 ⇒ 断言必报判据缺关键词。
    let doc = real_doc().replace(
        "| `verify` | `mappack_verify` | exit=0 + 输出含 0 违规/0 漂移/0 差 | 只读 |",
        "| `verify` | `mappack_verify` | exit=0 | 只读 |",
    );
    assert_ne!(doc, real_doc(), "篡改应生效（文档含目标串）");
    let problems = assert_mapping(&doc, &repo_root());
    assert!(problems.iter().any(|p| p.contains("判据")), "应报判据不一致，实际：{problems:?}");
}

#[test]
fn negative_missing_write_path_is_reported() {
    // 反例 3：删掉 §2 第 6 行的写点 ⇒ 断言必报写面缺写点。
    let doc = real_doc().replace(
        "| `assets/maps/` + `assets/data/board.json` + `assets/maps/index.json` + `assets/ui_manifest.json` |",
        "| `assets/maps/` |",
    );
    assert_ne!(doc, real_doc(), "篡改应生效（文档含目标串）");
    let problems = assert_mapping(&doc, &repo_root());
    assert!(
        problems.iter().any(|p| p.contains("写面") || p.contains("写点")),
        "应报写面不一致，实际：{problems:?}"
    );
}

#[test]
fn negative_readonly_step_without_label_is_reported() {
    // 反例 4：只读步的「写面」列未标「只读」⇒ 断言必报。
    let doc = real_doc().replace(
        "| `verify` | `mappack_verify` | exit=0 + 输出含 0 违规/0 漂移/0 差 | 只读 |",
        "| `verify` | `mappack_verify` | exit=0 + 输出含 0 违规/0 漂移/0 差 | 无 |",
    );
    assert_ne!(doc, real_doc(), "篡改应生效（文档含目标串）");
    let problems = assert_mapping(&doc, &repo_root());
    assert!(problems.iter().any(|p| p.contains("只读")), "应报只读标注缺失，实际：{problems:?}");
}

#[test]
fn negative_dropped_row_is_reported() {
    // 反例 5：删掉 §2 一整行 ⇒ 断言必报行数/缺行。
    let doc = real_doc();
    let lines: Vec<&str> = doc.lines().collect();
    let idx = lines
        .iter()
        .position(|l| l.contains("| `check` | `mappack_check` |"))
        .expect("应含 check 行");
    let mut kept: Vec<&str> = lines.clone();
    kept.remove(idx);
    let doc = kept.join("\n");
    let problems = assert_mapping(&doc, &repo_root());
    assert!(!problems.is_empty(), "删行后断言必报问题，实际 0 问题");
}
