//! S74.2 结构化结果协议（C1）+ 长日志截断修复判据。
//!
//! 诊断根因：`test_run_conclusion` 旧实现读 `job_log_tail(..., 400)`（clamp 400 行）
//! 再 grep `Running tests/` + `test result:`。多 binary 长日志（`gate_full` / `smoke`）
//! 首块落在前 400 行之外 ⇒ 结论静默漏检。本测试构造 >400 行样本做新旧对比。
//!
//! 本文件属 tests/，允许断言用 `unwrap/expect`（红线 §1 豁免）。

use std::fmt::Write as _;

use serde_json::{Value, json};

/// 构造 >400 行日志：首块在最前（旧实现测不到），尾块在末尾（旧实现只看到这个）。
fn long_cargo_log() -> String {
    let mut s = String::new();
    // 首块（旧 tail=400 必然截掉）：7 passed
    s.push_str("   Compiling demo v0.1.0\n");
    s.push_str("     Running tests/first_block.rs (target/debug/deps/first_block-abc)\n");
    s.push_str("test a_case ... ok\n");
    s.push_str("test b_case ... ok\n");
    s.push_str("test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured\n");
    // 中间填充 >400 行
    for i in 0..500 {
        writeln!(s, "   filler log line {i} — not a test block").expect("write");
    }
    // 尾块（旧 tail 可见）：3 passed
    s.push_str("     Running tests/tail_block.rs (target/debug/deps/tail_block-def)\n");
    s.push_str("test z_case ... ok\n");
    s.push_str("test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured\n");
    s
}

/// 模拟旧实现的 400 行尾部裁剪（与 `job_log_tail` 的 `lines.clamp(1, 400)` 同义）。
fn simulated_old_tail(full: &str) -> String {
    let all: Vec<&str> = full.lines().collect();
    let take = 400usize.clamp(1, 400);
    let start = all.len().saturating_sub(take);
    all[start..].join("\n")
}

#[test]
fn full_log_parser_detects_first_block_that_tail_400_misses() {
    let full = long_cargo_log();
    let old = simulated_old_tail(&full);
    let new = ti_console::runner::result_schema::parse_cargo_test_result(&full, "demo", 1234, 0);
    let old_parsed =
        ti_console::runner::result_schema::parse_cargo_test_result(&old, "demo", 1234, 0);

    let new_names: Vec<&str> = new.tests.iter().map(|t| t.name.as_str()).collect();
    let old_names: Vec<&str> = old_parsed.tests.iter().map(|t| t.name.as_str()).collect();
    eprintln!("[S74.2 对比] 日志总行数={}", full.lines().count());
    eprintln!("[S74.2 对比] 旧实现(tail=400) 测到的 binary: {old_names:?}");
    eprintln!("[S74.2 对比] 新实现(full)     测到的 binary: {new_names:?}");
    eprintln!(
        "[S74.2 对比] 新实现 passed 合计={} failed 合计={}",
        new.total_passed(),
        new.total_failed()
    );

    assert!(new_names.contains(&"first_block"), "新实现必须测到首块 first_block");
    assert!(new_names.contains(&"tail_block"), "新实现必须测到尾块 tail_block");
    assert_eq!(new.total_passed(), 10, "7 + 3 = 10");
    assert!(!old_names.contains(&"first_block"), "旧实现应测不到首块（对比基线）");
    assert!(old_names.contains(&"tail_block"), "旧实现应只看到尾块");
}

/// 真实作业状态 + 日志落盘，验证平台 `conclusion` 字段走完整日志（端到端）。
#[test]
fn conclusion_reads_full_log_end_to_end() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::from(d.subsec_nanos()));
    let root = std::env::temp_dir().join(format!("ti_w2_s74_2_{}_{nanos}", std::process::id()));
    let jobs = root.join("evidence/console/jobs");
    let logs = root.join("evidence/console/logs");
    std::fs::create_dir_all(&jobs).expect("jobs dir");
    std::fs::create_dir_all(&logs).expect("logs dir");

    let log_rel = "evidence/console/logs/test-longlog_demo-1.log";
    let state = json!({
        "kind": "test", "id": "longlog_demo", "running": false,
        "exit_code": 0, "started_ts": 1000, "ended_ts": 1002,
        "log": log_rel, "evidence": []
    });
    std::fs::write(jobs.join("test-longlog_demo.json"), state.to_string()).expect("state write");
    std::fs::write(root.join(log_rel), long_cargo_log()).expect("log write");

    let conclusion = ti_console::runner::jobs::test_run_conclusion(&root, "test", "longlog_demo")
        .expect("结构化结论应存在");
    eprintln!("[S74.2 端到端] conclusion = {conclusion}");
    assert_eq!(conclusion["source"].as_str(), Some("full_log"), "应走完整日志路径");
    assert_eq!(conclusion["duration_ms"].as_u64(), Some(2000), "2s → 2000ms");
    assert_eq!(conclusion["total_passed"].as_u64(), Some(10));
    let names: Vec<&str> = conclusion["tests"]
        .as_array()
        .expect("tests 数组")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"first_block"), "首块必须在 conclusion.tests 中");
    assert!(names.contains(&"tail_block"), "尾块必须在 conclusion.tests 中");
    assert_eq!(conclusion["exit_code"].as_i64(), Some(0));

    let _ = std::fs::remove_dir_all(&root);
}

/// C1 `result.json` 产物优先于日志解析（`source=result_json`）。
#[test]
fn conclusion_prefers_external_result_json() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::from(d.subsec_nanos()));
    let root = std::env::temp_dir().join(format!("ti_w2_s74_2_rj_{}_{nanos}", std::process::id()));
    let jobs = root.join("evidence/console/jobs");
    std::fs::create_dir_all(&jobs).expect("jobs dir");

    let rj_rel = "evidence/tests/w1_demo/w1_demo-99/result.json";
    std::fs::create_dir_all(root.join("evidence/tests/w1_demo/w1_demo-99")).expect("prove dir");
    let rj = json!({
        "binary": "w1_demo", "duration_ms": 777, "exit_code": 0,
        "tests": [{"name": "w1_demo", "passed": 42, "failed": 0, "msg": "from W1"}]
    });
    std::fs::write(root.join(rj_rel), rj.to_string()).expect("result.json write");

    let state = json!({
        "kind": "test", "id": "w1_demo", "running": false, "exit_code": 0,
        "started_ts": 10, "ended_ts": 11,
        "prove": "evidence/tests/w1_demo/", "evidence": [rj_rel]
    });
    std::fs::write(jobs.join("test-w1_demo.json"), state.to_string()).expect("state write");

    let conclusion = ti_console::runner::jobs::test_run_conclusion(&root, "test", "w1_demo")
        .expect("结构化结论应存在");
    eprintln!("[S74.2 result.json 优先] conclusion = {conclusion}");
    assert_eq!(conclusion["source"].as_str(), Some("result_json"));
    assert_eq!(conclusion["binary"].as_str(), Some("w1_demo"));
    assert_eq!(conclusion["duration_ms"].as_u64(), Some(777));
    assert_eq!(conclusion["tests"][0]["passed"].as_u64(), Some(42));
    assert_eq!(conclusion["tests"][0]["msg"].as_str(), Some("from W1"));

    let _ = std::fs::remove_dir_all(&root);
}

/// C1 形状往返：`parse_result_json(to_value)` 幂等。
#[test]
fn c1_shape_roundtrip() {
    let v: Value = json!({
        "binary": "b", "tests": [{"name":"b","passed":1,"failed":0,"msg":"ok"}],
        "duration_ms": 5, "exit_code": 0
    });
    let tr = ti_console::runner::result_schema::parse_result_json(&v.to_string()).expect("parse");
    assert_eq!(tr.to_value(), v);
    assert!(tr.is_ok());
}
