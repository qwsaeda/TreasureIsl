//! 任务收口 runner（S32.6）：`POST /api/close` 收口体检/执行 + EXP 查重。
//!
//! 白名单（`console_commands.json` 机器真源，22 号 §4.2）：
//! - `close_check`：`python3 scripts/task_close.py --task <id> --check`（只读体检，同步）；
//! - `close_run`：`python3 scripts/task_close.py --task <id> --cmd <固定串>...`（写，需 `--enable-write`）；
//! - `exp_check`：`python3 scripts/exp_log.py --check`（EXP 查重，同步）。
//!
//! **禁止透传自由字符串**（22 号 §4.2 禁止 1：`task_close.py --cmd` 是自由 shell 面 ⇒ RCE）：
//! `--cmd` 取值只能是 `console_commands.json` 登记的 **id**，平台把 id 精确映射为固定命令串
//! （`runner` + `script` + `args`），未登记 id → 400。
//!
//! **白名单对账断言**（S32.6 DoD）：22 号 §4.2 人读视图动作 id 集合 == `console_commands.json`
//! 集合（`whitelist_audit`；不一致 = `Err`，平台不得静默）。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::runner::apply_minimal_env;

/// 22 号 §4.2 人读视图动作 id 集合（机器断言镜像；S32.6 白名单对账断言）。
/// 必须与 `console_commands.json` `commands[].id` 集合逐字一致（`whitelist_audit` 断言）。
pub const HUMAN_VIEW_IDS: [&str; 23] = [
    "gate",
    "close_check",
    "close_run",
    "exp_check",
    "build_check",
    "serve_web",
    "mappack_verify",
    "mappack_check",
    "mappack_add",
    "mappack_syncmap",
    "mappack_pack",
    "mappack_install",
    "mappack_sync",
    "mappack_uninstall",
    "ui_variant",
    "map_shot",
    "publish",
    "audit_book",
    "build_apk",
    "build_wasm",
    "build_native",
    "build_list",
    "build_artifacts_check",
];

/// `console_commands.json` 路径（相对仓库根）。
fn whitelist_path(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/ti-console/console_commands.json")
}

/// EXP 分卷枚举（`exp_log.py` `VOLUMES` 镜像；22 号 §4.2 `exp_check` 的 `--volume <枚举>`）。
pub const EXP_VOLUMES: [&str; 6] = ["code", "process", "build", "tools", "formats", "reverse"];

/// 执行日志目录（与 gate 同目录面；gitignored）。
fn logs_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("evidence/console/logs")
}

/// 读 `console_commands.json` 命令清单（`commands` 数组）。
fn read_commands(repo_root: &Path) -> Result<Vec<Value>, String> {
    let path = whitelist_path(repo_root);
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("读 {} 失败：{e}", path.display()))?;
    let v: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse {}：{e}", path.display()))?;
    v.get("commands")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| "console_commands.json 缺 `commands` 数组".to_owned())
}

/// 白名单对账断言（S32.6 DoD）：22 号 §4.2 人读视图 id 集合 == `console_commands.json` id 集合。
/// 返回 `{ok, ids, count, write_required}`；集合不一致 = `Err`。
///
/// # Errors
/// 读 `console_commands.json` 失败 / 缺 `commands` 数组 / 人读视图与机器真源 id 集合不一致时
/// 返回 `Err`（平台不得静默；`POST /api/close` 据此回 500）。
pub fn whitelist_audit(repo_root: &Path) -> Result<Value, String> {
    let cmds = read_commands(repo_root)?;
    let mut json_ids: Vec<String> = cmds
        .iter()
        .filter_map(|c| c.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect();
    json_ids.sort();
    let mut human: Vec<String> = HUMAN_VIEW_IDS.iter().map(|s| (*s).to_owned()).collect();
    human.sort();
    if human != json_ids {
        return Err(format!(
            "白名单对账 FAIL：22 号 §4.2 人读视图 {human:?} ≠ console_commands.json {json_ids:?}（S32.6 对账断言）"
        ));
    }
    // 默认启用列对账（当前仅 close_run 需 --enable-write；与 22 号 §4.2 一致）。
    let write_required: Vec<String> = cmds
        .iter()
        .filter(|c| c.get("enabled_by_default").and_then(Value::as_bool) == Some(false))
        .filter_map(|c| c.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect();
    Ok(json!({
        "ok": true,
        "ids": json_ids,
        "count": json_ids.len(),
        "write_required": write_required,
    }))
}

/// 把白名单 id 精确映射为固定命令串（`runner` + `script` + `args`；22 号 §4.2 禁止 1）。
/// 未登记 id = `None`（调用方返回 400）。
///
/// `args` 已含 `script` 时（如 gate：`["-File","scripts/gate.ps1"]`，pwsh 需 `-File` 在脚本前）
/// 不再前置 script（防 `pwsh scripts/gate.ps1 -File scripts/gate.ps1` 调用序错，r1-B P3-2）。
pub fn map_cmd_id(cmds: &[Value], id: &str) -> Option<String> {
    let c = cmds.iter().find(|c| c.get("id").and_then(Value::as_str) == Some(id))?;
    let runner = c.get("runner").and_then(Value::as_str).unwrap_or("");
    let script = c.get("script").and_then(Value::as_str).unwrap_or("");
    let args: Vec<&str> = c
        .get("args")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut parts = vec![runner.to_owned()];
    // 若 `args` 已含 script（如 gate：`["-File","scripts/gate.ps1"]`，pwsh 需 `-File` 在脚本前），
    // 不再前置 script，否则生成 `pwsh scripts/gate.ps1 -File scripts/gate.ps1`（调用序错，r1-B P3-2）。
    let script_in_args = args.contains(&script);
    if !script.is_empty() && !script_in_args {
        parts.push(script.to_owned());
    }
    for a in args {
        parts.push(a.to_owned());
    }
    // self runner（serve_web/audit_book）= 平台自身动作，不作为 task_close.py --cmd 判据。
    if runner == "self" {
        return None;
    }
    Some(parts.join(" "))
}

/// 同步跑一个子进程（`task_close.py` / `exp_log.py`），捕获合并输出。
/// 程序来自白名单（固定路径），参数逐段传（禁 shell 拼接；22 号 §4.2 禁止 2）。
fn run_sync(prog: &str, args: &[String]) -> Result<(i32, String), String> {
    let mut cmd = std::process::Command::new(prog);
    cmd.args(args);
    apply_minimal_env(&mut cmd);
    cmd.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let out = cmd.output().map_err(|e| format!("spawn {prog} 失败：{e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}{stderr}");
    Ok((out.status.code().unwrap_or(-1), combined))
}

/// 落一条执行日志到 `evidence/console/logs/`（返回相对路径）。
fn write_log(repo_root: &Path, kind: &str, content: &str) -> Result<String, String> {
    let dir = logs_dir(repo_root);
    std::fs::create_dir_all(&dir).map_err(|e| format!("logs 目录创建失败 {e}"))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let name = format!("close-{kind}-{ts}.log");
    let path = dir.join(&name);
    std::fs::write(&path, content).map_err(|e| format!("日志写入失败 {e}"))?;
    Ok(format!("evidence/console/logs/{name}"))
}

/// 只读收口体检（`close_check`）：跑 `task_close.py --task <id> --check`（不改台账）。
/// 输出落 `evidence/console/logs/close-check-<ts>.log`。
///
/// # Errors
/// `task` 为空（400）或 spawn/日志写失败（500）时返回 `Err`；`task_close.py` 自身非 0 退出
/// 不作 `Err`（只读体检报告经 `exit_code` 字段返回，平台只透传不归因）。
pub fn close_check(repo_root: &Path, task: &str) -> Result<Value, (u16, String)> {
    if task.is_empty() {
        return Err((400u16, "task 不能为空".to_owned()));
    }
    let args = vec![
        "scripts/task_close.py".to_owned(),
        "--task".to_owned(),
        task.to_owned(),
        "--check".to_owned(),
    ];
    let (code, out) = run_sync("python3", &args).map_err(|e| (500u16, e))?;
    let log = write_log(repo_root, "check", &out).map_err(|e| (500u16, e))?;
    Ok(json!({
        "ok": code == 0,
        "exit_code": code,
        "task": task,
        "mode": "check",
        "log": log,
        "output_excerpt": out.chars().take(2000).collect::<String>(),
    }))
}

/// 收口执行（`close_run`）：`task_close.py --task <id> --cmd <固定串>...`（写；需 `--enable-write`）。
/// `cmd_ids` = `console_commands.json` 白名单 id（未登记 id → 400）；禁透传自由字符串。
///
/// # Errors
/// 未给 `--enable-write`（400）/ `task` 为空（400）/ 未登记判据 id（400，22 号 §4.2 禁止 1）/
/// 读白名单或 spawn 或日志写失败（500）时返回 `Err`。
pub fn close_run(
    repo_root: &Path,
    task: &str,
    cmd_ids: &[String],
    enable_write: bool,
) -> Result<Value, (u16, String)> {
    if !enable_write {
        return Err((
            400u16,
            "close_run 写操作需 `--enable-write` 显式开启（22 号 §4.2 默认关闭）".to_owned(),
        ));
    }
    if task.is_empty() {
        return Err((400u16, "task 不能为空".to_owned()));
    }
    let cmds = read_commands(repo_root).map_err(|e| (500u16, e))?;
    let mut args = vec!["scripts/task_close.py".to_owned(), "--task".to_owned(), task.to_owned()];
    for id in cmd_ids {
        let fixed = map_cmd_id(&cmds, id).ok_or_else(|| {
            (
                400u16,
                format!("未登记判据 id `{id}`（console_commands.json 白名单；22 号 §4.2 禁止 1）"),
            )
        })?;
        args.push("--cmd".to_owned());
        args.push(fixed);
    }
    let (code, out) = run_sync("python3", &args).map_err(|e| (500u16, e))?;
    let log = write_log(repo_root, "run", &out).map_err(|e| (500u16, e))?;
    Ok(json!({
        "ok": code == 0,
        "exit_code": code,
        "task": task,
        "mode": "run",
        "cmd_ids": cmd_ids,
        "log": log,
        "output_excerpt": out.chars().take(2000).collect::<String>(),
    }))
}

/// EXP 查重（`exp_check`）：跑 `exp_log.py --volume <v> --id <id> --check`（只读，单 EXP 查重）。
///
/// `volume` ∈ `EXP_VOLUMES`（枚举，22 号 §4.2）；`id` 必须匹配 `EXP-\d+`（禁自由字符串）。
///
/// # Errors
/// `volume` 非法（400）/ `id` 非 `EXP-\d+`（400）/ spawn 或日志写失败（500）时返回 `Err`；
/// `exp_log.py` 自身非 0 退出不作 `Err`（经 `exit_code` 字段返回，平台只透传不归因）。
pub fn exp_check(repo_root: &Path, volume: &str, id: &str) -> Result<Value, (u16, String)> {
    if !EXP_VOLUMES.contains(&volume) {
        return Err((
            400u16,
            format!("volume 必须为 {}（收到 `{volume}`）", EXP_VOLUMES.join("|")),
        ));
    }
    if !is_exp_id(id) {
        return Err((
            400u16,
            format!("id 必须匹配 EXP-\\d+（收到 `{id}`；22 号 §4.2 禁自由字符串）"),
        ));
    }
    let args = vec![
        "scripts/exp_log.py".to_owned(),
        "--volume".to_owned(),
        volume.to_owned(),
        "--id".to_owned(),
        id.to_owned(),
        "--check".to_owned(),
    ];
    let (code, out) = run_sync("python3", &args).map_err(|e| (500u16, e))?;
    let log = write_log(repo_root, "exp", &out).map_err(|e| (500u16, e))?;
    Ok(json!({
        "ok": code == 0,
        "exit_code": code,
        "mode": "exp",
        "volume": volume,
        "id": id,
        "log": log,
        "output_excerpt": out.chars().take(2000).collect::<String>(),
    }))
}

/// EXP 编号校验（`EXP-\d+`；22 号 §4.2 禁自由字符串）。
fn is_exp_id(id: &str) -> bool {
    let Some(num) = id.strip_prefix("EXP-") else {
        return false;
    };
    !num.is_empty() && num.bytes().all(|b| b.is_ascii_digit())
}
