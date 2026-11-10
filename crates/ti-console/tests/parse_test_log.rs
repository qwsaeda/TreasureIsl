//! `parse_test_log` 集成测试（2026-11-10「自解释结论」层）。

use serde_json::Value;

/// 仓库根路径（cargo test 的 CWD = crates/ti-console/，`../..` = 仓库根）。
fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from("../..")
}

#[test]
fn parse_smoke_style_log() {
    // smoke.ps1 简化输出：Running + test result 对（binary 名关联 manifest desc）
    let text = "  Running tests/rules_battle.rs (path)\n  test result: ok. 20 passed; 0 failed;\n  Running tests/exhaust_combat.rs (path)\n  test result: ok. 5 passed; 0 failed;\n";
    let v: Value = ti_console::runner::jobs::parse_test_log(text, repo_root().as_path());
    assert_eq!(v["binary_count"].as_u64().unwrap_or(0), 2, "应解析出 2 个 binary");
    assert_eq!(v["total_passed"].as_u64().unwrap_or(0), 25, "20 + 5 = 25");
    assert!(v["ok"].as_bool().unwrap_or(false), "全过应 ok=true");
    // binary 名应关联 manifest desc（rules_battle 的 desc 含「战斗」）
    let d0 = &v["details"][0];
    assert_eq!(d0["binary"].as_str(), Some("rules_battle"), "binary 名应对上 rules_battle");
    let desc0 = d0["desc"].as_str().unwrap_or("");
    assert!(desc0.contains("战斗"), "desc 应来自 manifest，got: {desc0}");
}

#[test]
fn parse_failures() {
    let text = "Running tests/foo.rs\n  test a ... ok\n  test b ... FAILED\n  test c ... ok\n\ntest result: FAILED. 2 passed; 1 failed;\n";
    let v: Value = ti_console::runner::jobs::parse_test_log(text, repo_root().as_path());
    assert!(!v["ok"].as_bool().unwrap_or(true), "有失败应 ok=false");
    assert_eq!(v["total_failed"].as_u64().unwrap_or(0), 1);
    assert_eq!(v["total_passed"].as_u64().unwrap_or(0), 2);
    let fails = v["details"][0]["failures"].as_array().expect("failures 应为数组");
    assert_eq!(fails.first().and_then(Value::as_str), Some("b"), "失败用例名应对上 b");
}

#[test]
fn parse_real_rules_battle_log() {
    let text =
        std::fs::read_to_string("../../evidence/console/logs/test-rules_battle-1794280140.log")
            .expect("日志文件存在（test-rules_battle-1794280140.log）");
    let v: Value = ti_console::runner::jobs::parse_test_log(&text, repo_root().as_path());
    assert!(v["ok"].as_bool().unwrap_or(false), "rules_battle 应全过");
    assert_eq!(v["total_passed"].as_u64().unwrap_or(0), 20, "20 条全过");
    assert_eq!(v["binary_count"].as_u64().unwrap_or(0), 1);
    let d = &v["details"][0];
    assert_eq!(d["binary"].as_str(), Some("rules_battle"), "binary 名应对上 rules_battle");
    let desc_r = d["desc"].as_str().unwrap_or("");
    assert!(desc_r.contains("战斗"), "desc 应含「战斗」，got: {desc_r}");
}
