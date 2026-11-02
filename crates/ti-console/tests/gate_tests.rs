#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S32.4 gate runner 单测（src 内联 mod tests 迁移；tests/ 目录内允许 unwrap/expect）。
//!
//! 只测**非 spawn 面**（tier 校验 / 锁状态 / 历史 / 尾部）：
//! - 真实 spawn（`pwsh -File scripts/gate.ps1` 全量跑）是慢路径，归 e2e 验收（POST /api/gate 实跑 fast）；
//! - 单测用**当前进程 pid** 造「活锁」（409）与**死 pid** 造「陈旧锁」（可复用）。

use ti_console::runner::gate::{TIERS, check_lock_available, gate_status, run_gate};

/// 临时仓库根（每测试独立目录；evidence/console/{logs,gate.lock} 面）。
fn tmp_root(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ti_console_gate_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("evidence/console/logs")).unwrap();
    d
}

#[test]
fn tiers_enum_three() {
    // tier 白名单与 console_commands.json enum_param.values 对齐（22 号 §4.2）。
    assert_eq!(TIERS, ["fast", "touch", "full"]);
}

#[test]
fn invalid_tier_is_400() {
    let root = tmp_root("bad_tier");
    // 非法 tier（未命中 tier_args）→ 400，不 spawn。
    for t in ["", "slow", "FAST", "fullx"] {
        let e = run_gate(&root, t).unwrap_err();
        assert_eq!(e.0, 400, "tier `{t}` 应 400");
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn status_empty_no_lock() {
    let root = tmp_root("empty");
    let v = gate_status(&root, None);
    assert_eq!(v["running"], false);
    assert!(v["last"].is_null());
    assert_eq!(v["history"].as_array().unwrap().len(), 0, "history 应空");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn status_stale_lock_is_not_running() {
    let root = tmp_root("stale");
    // 活锁（running=true + 当前进程 pid 存活）→ running=true。
    let live = serde_json::json!({
        "pid": std::process::id(), "tier": "fast", "started_ts": 0,
        "log": "evidence/console/logs/gate-x.log", "running": true,
    });
    std::fs::write(root.join("evidence/console/gate.lock"), live.to_string()).unwrap();
    assert_eq!(gate_status(&root, None)["running"], true);
    // 活锁 → run_gate 409（不 spawn）。
    let e = run_gate(&root, "fast").unwrap_err();
    assert_eq!(e.0, 409, "活锁应 409");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn status_dead_pid_is_reusable() {
    let root = tmp_root("dead");
    // running=true 但 pid 已死（取一个几乎肯定不存在的 pid）→ 视为陈旧 → 非 running。
    let dead = serde_json::json!({
        "pid": 999_999_999u32, "tier": "fast", "started_ts": 0,
        "log": "evidence/console/logs/gate-x.log", "running": true,
    });
    std::fs::write(root.join("evidence/console/gate.lock"), dead.to_string()).unwrap();
    let v = gate_status(&root, None);
    assert_eq!(v["running"], false, "死 pid 锁应为非 running（陈旧）");
    // 不调 run_gate（会 spawn pwsh）；409 只保留给活锁——断言逻辑由 status_stale 覆盖。
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn history_and_tail() {
    let root = tmp_root("hist");
    let logs = root.join("evidence/console/logs");
    // 造两个 gate-*.log（一个非 gate 前缀应被过滤）。
    std::fs::write(logs.join("gate-100.log"), "Report PASS a\nReport PASS b\n").unwrap();
    std::fs::write(logs.join("gate-200.log"), "Report FAIL c\n").unwrap();
    std::fs::write(logs.join("test-1.log"), "not gate\n").unwrap();
    let v = gate_status(&root, None);
    let hist = v["history"].as_array().unwrap();
    // gate-200 更新（更大 ts？不排序保证）——断言两个 gate 日志都在、非 gate 被滤。
    assert_eq!(hist.len(), 2, "history 应只含 gate-*.log（非 gate 滤掉）");
    // 尾部读取（?log=gate-100.log）。
    let tail = gate_status(&root, Some("gate-100.log"));
    assert!(tail["tail"].as_str().unwrap().contains("Report PASS a"));
    // 穿越拒绝（.. 在 name 内）→ tail 为 null。
    let bad = gate_status(&root, Some("gate-../etc/passwd.log"));
    assert!(bad["tail"].is_null(), "穿越应拒绝");
    let _ = std::fs::remove_dir_all(&root);
}

/// 写损坏锁（非合法 JSON）+ 设 mtime 为 `secs_ago` 秒前。
fn write_corrupt_lock(root: &std::path::Path, secs_ago: u64) {
    let p = root.join("evidence/console/gate.lock");
    std::fs::write(&p, "not-valid-json{{").unwrap();
    let f = std::fs::File::open(&p).unwrap();
    let past = std::time::SystemTime::now() - std::time::Duration::from_secs(secs_ago);
    f.set_times(std::fs::FileTimes::new().set_modified(past)).unwrap();
}

#[test]
fn corrupt_lock_old_is_takeover() {
    // 损坏锁（非合法 JSON）超 5 秒 = 崩溃遗留 → 接管（Ok，锁被清除可重新占）。
    let root = tmp_root("corrupt_old");
    write_corrupt_lock(&root, 10);
    let r = check_lock_available(&root);
    assert!(r.is_ok(), "损坏锁超 5 秒应接管（Ok），got {r:?}");
    assert!(!root.join("evidence/console/gate.lock").exists(), "接管后锁应被清除");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn corrupt_lock_recent_is_409() {
    // 损坏锁 < 5 秒 = 可能正写入（活锁）→ 409（不接管、不误删）。
    let root = tmp_root("corrupt_recent");
    write_corrupt_lock(&root, 1);
    let e = check_lock_available(&root).unwrap_err();
    assert_eq!(e.0, 409, "损坏锁 < 5 秒应 409（可能正写入），got {e:?}");
    assert!(root.join("evidence/console/gate.lock").exists(), "近期损坏锁不应被删");
    let _ = std::fs::remove_dir_all(&root);
}
