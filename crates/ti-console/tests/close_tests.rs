#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S32.6 close runner 单测（src 内联测试迁移；tests/ 目录内允许 `unwrap`/`expect`）。
//!
//! 只测**非 spawn 面**（白名单对账 / 参数校验 / 只读体检报告 / 未登记 id 400 / `map_cmd_id` 映射）：
//! - 真实 spawn（`python3 scripts/task_close.py` 等）是慢路径，归 e2e 验收（POST /api/close 实跑）；
//! - 单测用真实仓库根（`CARGO_MANIFEST_DIR/../..`）验证 `whitelist_audit` 对账 + 参数校验。
//! - `close_check`/`exp_check` 测试会把执行日志写进**真实** `evidence/console/logs/close-*.log`
//!   （`.gitignore` L67 已忽略，无仓库污染；非 hermetic 但同 `gate_tests` 口径——脚本须读真实台账）。
//! - `map_cmd_id` 测试读真实 `console_commands.json`（纯函数，无副作用）。

use ti_console::runner::close::{
    EXP_VOLUMES, close_check, close_run, exp_check, map_cmd_id, whitelist_audit,
};

/// 读 `console_commands.json` 命令数组（测试用；真实仓库根）。
fn commands() -> Vec<serde_json::Value> {
    let p = repo_root().join("crates/ti-console/console_commands.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    v["commands"].as_array().unwrap().clone()
}

/// 真实仓库根（`task_close.py` / `exp_log.py` / `console_commands.json` 所在）。
fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn exp_volumes_six() {
    // volume 枚举与 exp_log.py VOLUMES 对齐（22 号 §4.2）。
    assert_eq!(EXP_VOLUMES, ["code", "process", "build", "tools", "formats", "reverse"]);
}

#[test]
fn whitelist_audit_ok() {
    // 白名单对账断言：22 号 §4.2 人读视图 id 集合 == console_commands.json（真实仓库）。
    // S44.1 新增 9 动作（mappack_add/syncmap/pack/install/sync/uninstall + ui_variant + map_shot + publish）。
    let v = whitelist_audit(&repo_root()).expect("白名单对账应通过");
    assert_eq!(v["ok"], true);
    assert_eq!(v["count"], 24);
    // 写面动作默认关（enabled_by_default=false）：close_run + 9 个内容运营写动作（30 号 §3.1）；
    // map_shot 只读出图默认开。S48.0 加 ui_theme（主题构图应用，写 scenes.json + assets/ui + ui_manifest）。
    let wr: Vec<&str> = v["write_required"]
        .as_array()
        .expect("write_required 数组")
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(wr.len(), 10);
    for id in [
        "close_run",
        "mappack_add",
        "mappack_syncmap",
        "mappack_pack",
        "mappack_install",
        "mappack_sync",
        "mappack_uninstall",
        "ui_variant",
        "ui_theme",
        "publish",
    ] {
        assert!(wr.contains(&id), "write_required 缺 {id}");
    }
    assert!(!wr.contains(&"map_shot"), "map_shot 只读出图不应默认关");
}

#[test]
fn close_check_unknown_task_ok_but_reports() {
    // 只读体检：不存在的任务 → task_close.py 返回非 0（台账无此任务），但平台不 Err（只读体检报告）。
    let v = close_check(&repo_root(), "S32.6").expect("close_check 不应 Err（只读体检）");
    assert_eq!(v["task"], "S32.6");
    assert_eq!(v["mode"], "check");
    assert!(v["exit_code"].as_i64().is_some());
}

#[test]
fn close_run_requires_enable_write() {
    // close_run 未给 --enable-write → 400（22 号 §4.2 默认关闭）。
    let e = close_run(&repo_root(), "S32.6", &[], false).unwrap_err();
    assert_eq!(e.0, 400);
}

#[test]
fn close_run_unknown_cmd_id_400() {
    // 未登记判据 id → 400（22 号 §4.2 禁止 1：禁透传自由字符串）。
    let e =
        close_run(&repo_root(), "S32.6", &["sh".to_owned(), "-c".to_owned()], true).unwrap_err();
    assert_eq!(e.0, 400);
    assert!(e.1.contains("未登记判据 id"), "{}", e.1);
}

#[test]
fn exp_check_readonly() {
    // EXP 查重：exp_log.py --volume <v> --id <EXP-id> --check（0 = 无重复 EXP）。
    let v = exp_check(&repo_root(), "code", "EXP-116").expect("exp_check 不应 Err");
    assert_eq!(v["mode"], "exp");
    assert_eq!(v["volume"], "code");
    assert_eq!(v["id"], "EXP-116");
    assert!(v["exit_code"].as_i64().is_some());
}

#[test]
fn exp_check_invalid_volume_400() {
    let e = exp_check(&repo_root(), "codex", "EXP-116").unwrap_err();
    assert_eq!(e.0, 400);
}

#[test]
fn exp_check_invalid_id_400() {
    for bad in ["EXP-", "EXP-x", "sh; rm -rf /", "116", ""] {
        let e = exp_check(&repo_root(), "code", bad).unwrap_err();
        assert_eq!(e.0, 400, "id `{bad}` 应 400");
    }
}

#[test]
fn map_cmd_id_gate_no_dup_script() {
    // gate args 已含脚本（pwsh 需 `-File` 在脚本前），map 不重复前置 → `pwsh -File scripts/gate.ps1`。
    let m = map_cmd_id(&commands(), "gate").expect("gate 应可映射");
    assert_eq!(m, "pwsh -File scripts/gate.ps1", "gate 映射应无重复脚本（r1-B P3-2）");
}

#[test]
fn map_cmd_id_close_check_prepends_script() {
    // close_check args 不含脚本（仅 --check），map 前置脚本 → `python3 scripts/task_close.py --check`。
    let m = map_cmd_id(&commands(), "close_check").expect("close_check 应可映射");
    assert_eq!(m, "python3 scripts/task_close.py --check");
}

#[test]
fn map_cmd_id_unknown_none() {
    // 未登记 id → None（调用方 400）。
    assert!(map_cmd_id(&commands(), "sh").is_none());
    assert!(map_cmd_id(&commands(), "not-a-real-id").is_none());
}

#[test]
fn map_cmd_id_self_runner_none() {
    // runner=="self"（serve_web/audit_book）= 平台自身动作，不作 task_close.py --cmd 判据。
    for id in ["serve_web", "audit_book"] {
        assert!(map_cmd_id(&commands(), id).is_none(), "{id} 应 None（self runner）");
    }
}
