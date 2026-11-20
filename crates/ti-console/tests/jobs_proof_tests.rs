#![allow(clippy::unwrap_used, clippy::expect_used)]

//! 统一作业模型证明落点（23 号 §6.2 证明行，审计 Q013）集成测试。
//!
//! 判据：`persist_proof` 在 `prove` 前缀下建 `<id>-<ts>/`，落日志副本 `run.log` +
//! 终态快照 `state.json`，并把两路径登记进作业状态 `evidence[]` ⇒
//! 「本次运行证明」与作业 id/时间戳绑定（不混静态基线证据）。

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use ti_console::runner::jobs::{persist_proof, register_evidence};

fn tmp_root(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ti_proof_test_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn proof_dir_created_with_log_and_state_snapshot() {
    let root = tmp_root("proof_dir");
    // 源日志（模拟 jobs::spawn_job 落下的 console 日志）
    let log_rel = "evidence/console/logs/test-demo-100.log";
    let log_path = root.join(log_rel);
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(&log_path, "test result: ok. 3 passed; 0 failed\n").unwrap();
    // 作业状态文件（register_evidence 要写它）
    let state_rel = "evidence/console/jobs/test-demo.json";
    let state_path = root.join(state_rel);
    fs::create_dir_all(state_path.parent().unwrap()).unwrap();
    fs::write(&state_path, json!({"id": "demo", "evidence": []}).to_string()).unwrap();

    let state = json!({
        "kind": "test", "id": "demo", "log": log_rel,
        "prove": "evidence/tests/demo/",
        "exit_code": 0, "running": false, "ended_ts": 200,
    });
    persist_proof(&root, Some("evidence/tests/demo/"), "test", "demo", Some(log_rel), 200, &state);

    let proof_dir = root.join("evidence/tests/demo/demo-200");
    assert!(proof_dir.is_dir(), "证明目录 <id>-<ts> 应创建");
    assert_eq!(
        fs::read_to_string(proof_dir.join("run.log")).unwrap(),
        "test result: ok. 3 passed; 0 failed\n",
        "日志副本应与源日志逐字一致"
    );
    let snapshot: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(proof_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(snapshot["exit_code"], 0);
    assert_eq!(snapshot["running"], false);
    // evidence[] 登记两路径（日志副本 + 状态快照）
    let updated: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let ev = updated["evidence"].as_array().unwrap();
    assert_eq!(ev.len(), 2, "evidence[] 应登记日志副本 + 状态快照两条证明");
    assert!(ev.iter().any(|p| p.as_str().unwrap().contains("run.log")));
    assert!(ev.iter().any(|p| p.as_str().unwrap().contains("state.json")));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn no_prove_field_is_noop() {
    let root = tmp_root("no_prove");
    // prove=None ⇒ 不建任何目录（兼容无 prove 字段的存量条目）
    persist_proof(&root, None, "test", "demo", None, 100, &json!({"id": "demo"}));
    assert!(!root.join("evidence/tests").exists(), "无 prove 字段时不应建证明目录");
    // 空前缀同样 no-op
    persist_proof(&root, Some(""), "test", "demo", None, 100, &json!({"id": "demo"}));
    assert!(!root.join("evidence/tests").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn register_evidence_dedups_paths() {
    let root = tmp_root("dedup");
    let p = root.join("evidence/console/jobs/test-x.json");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, json!({"id": "x", "evidence": []}).to_string()).unwrap();
    assert!(register_evidence(&root, "test", "x", &["a".into(), "a".into(), "b".into()]));
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    assert_eq!(v["evidence"].as_array().unwrap().len(), 2, "重复路径须去重");
    let _ = fs::remove_dir_all(&root);
}
