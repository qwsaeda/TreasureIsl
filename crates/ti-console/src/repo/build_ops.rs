//! `/api/build`：构建编排状态面（S35.1，24 号 §4.1）。
//!
//! 职责：只读面 = `scripts/build_artifacts.py list`（产物体积/mtime）+ 最近一次构建日志 +
//! **构建锁状态**（`evidence/console/logs/build.lock`：`running`/`pid`/`exit_code`/`kind`，`S46.3`）；
//! 写面 = `?run=<kind>` 后台 spawn 编排器（kind 白名单 `apk|wasm|native`，自由字符串 = 400）。
//!
//! `S46.3` 单构建互斥：`build_apk.sh` 会**临时改根 `Cargo.toml`**（members 加 android-activity），
//! 并发构建（尤其中途再点）会互踩 ⇒ 用 `build.lock`（`create_new` 原子占位 + `/proc/<pid>` 存活预检）
//! 强制串行；忙时返回 `busy:true`（HTTP 200，前端显示「已在跑」而非再 spawn）。
//!
//! 本模块 **0 游戏依赖**；写点仅 `evidence/console/logs/`（平台归档目录）。

use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;

use serde_json::{Value, json};

use super::config::RepoSources;

/// Unix 秒（`SystemTime` → `UNIX_EPOCH` 秒数；溢出 / 系统时钟异常 = 0，不 panic）。
fn now_unix_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// `/api/build`：构建产物清单 + 构建编排状态（S35.1，24 号 §4.1）。
///
/// - `run_kind == None`：只读面。`scripts/build_artifacts.py list`（已构建产物 体积/mtime）+
///   最近一次构建日志（`evidence/console/logs/build-*.log`，gitignored）+ 构建锁状态
///   （`running` / `running_pid` / `last_exit` / `last_kind`，`S46.3`）。
/// - `run_kind == Some(kind)`：后台 spawn 构建编排器（kind 白名单 `apk|wasm|native`，
///   24 号 §4.1；自由字符串 = 400），平台不经手拼任意 shell；**单构建互斥**（`S46.3`）——
///   已有构建在跑返回 `started:false, busy:true`。
///
/// # Errors
///
/// `build_artifacts.py` 执行失败（`python3` 缺位 / 非 0 退出）= `Err`（带原因，数据源面如实上报）；
/// 非法 `run_kind` = `Err`（白名单校验，22 号 §10.3）。
pub fn build_artifacts(
    repo_root: &Path,
    _sources: &RepoSources,
    run_kind: Option<&str>,
) -> Result<Value, String> {
    let lock_path = repo_root.join("evidence/console/logs/build.lock");
    match run_kind {
        Some(kind) => run_build(repo_root, kind, &lock_path),
        None => list_build(repo_root, &lock_path),
    }
}

/// 锁是否**陈旧**（非真占用）：锁文件缺失 / `running=false` / pid 已死。
///
/// 用于获取锁时判定：锁文件永存（完成后 `running=false` 状态记录供上报），`create_new` 必失败；
/// 陈旧锁 = 无构建在跑 ⇒ 允许移除重试（`S46.3`）。
fn lock_is_stale(lock_path: &Path) -> bool {
    let Ok(lock_text) = std::fs::read_to_string(lock_path) else {
        return true; // 锁文件缺失 = 无占用
    };
    let lock_v: Value = serde_json::from_str(&lock_text).unwrap_or(Value::Null);
    let running = lock_v.get("running").and_then(Value::as_bool).unwrap_or(false);
    if !running {
        return true;
    }
    let pid = lock_v.get("pid").and_then(Value::as_u64).unwrap_or(0);
    pid == 0 || !std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// `?run=<kind>` 后台 spawn 构建编排器（`S46.3` 单构建互斥）。
///
/// 白名单 `apk|wasm|native`（24 号 §4.1；自由字符串 = `Err`）；`build.lock` 用 `create_new`
/// 原子占位（防 TOCTOU），已有构建在跑（锁 `running:true` 且 `/proc/<pid>` 存活）返回
/// `started:false, busy:true`（HTTP 200，前端显示「已在跑」而非再 spawn）。
fn run_build(repo_root: &Path, kind: &str, lock_path: &Path) -> Result<Value, String> {
    if !matches!(kind, "apk" | "wasm" | "native") {
        return Err(format!(
            "run 参数只接受 apk|wasm|native（22 号 §10.3 白名单；非自由字符串）：{kind}"
        ));
    }
    // 锁预检：锁存在且 pid 存活（/proc/<pid>）⇒ 已有构建在跑（防止多击并发踩 Cargo.toml）。
    if let Ok(lock_text) = std::fs::read_to_string(lock_path) {
        let lock_v: Value = serde_json::from_str(&lock_text).unwrap_or(Value::Null);
        let running = lock_v.get("running").and_then(Value::as_bool).unwrap_or(false);
        let pid = lock_v.get("pid").and_then(Value::as_u64).unwrap_or(0);
        if running && pid > 0 && std::path::Path::new(&format!("/proc/{pid}")).exists() {
            return Ok(json!({
                "started": false,
                "busy": true,
                "pid": pid,
                "kind": kind,
                "note": format!("已有构建在跑（pid {pid}）；单构建互斥（build_apk.sh 临时改根 Cargo.toml），请等完成或轮询 /api/build"),
            }));
        }
    }
    // 持锁（create_new：原子占位，防 TOCTOU）。
    // S46.3 锁完成后文件永存（running=false 状态记录供 /api/build 上报）⇒ 下次 create_new 必失败；
    // 处理：create_new 失败且既有锁为**陈旧**（running=false 或 pid 死）= 非真占用 ⇒ 移除后重试一次。
    let mut lock_file =
        match std::fs::OpenOptions::new().write(true).create_new(true).open(lock_path) {
            Ok(f) => f,
            Err(_) if lock_is_stale(lock_path) => {
                let _ = std::fs::remove_file(lock_path);
                match std::fs::OpenOptions::new().write(true).create_new(true).open(lock_path) {
                    Ok(f) => f,
                    Err(_) => {
                        return Ok(json!({
                            "started": false,
                            "busy": true,
                            "note": "构建锁被占用（build.lock 存在）；请稍后重试或轮询 /api/build",
                        }));
                    }
                }
            }
            Err(_) => {
                return Ok(json!({
                    "started": false,
                    "busy": true,
                    "note": "构建锁被占用（build.lock 存在）；请稍后重试或轮询 /api/build",
                }));
            }
        };
    let ts = now_unix_secs();
    let placeholder = json!({
        "pid": 0,
        "kind": kind,
        "running": true,
        "started_ts": ts,
        "exit_code": Value::Null,
        "ended_ts": Value::Null,
        "note": "构建占位（pid=0 = reserving，spawn 后更新真实 pid）",
    });
    let _ = writeln!(lock_file, "{placeholder}");
    // 统一作业模型（23 号 §6.2；S49.7）：spawn + 收割（exit_code/ended_ts）+ 状态写入**单一实现**
    // `runner::jobs::spawn_program_logged`；构建保留自身互斥锁文件（`build.lock` = 状态载体，
    // `/api/build` 读回不变）+ 额外写统一作业镜像（`evidence/console/jobs/build-<kind>.json`）
    // ⇒ `/api/jobs` 统一查询面可查构建结果（零行为变化：日志仍由 build_artifacts.py 自写）。
    let mut meta = placeholder;
    // 注：**不写** `kind`/`id` 字段 —— `build.lock` 的 `kind` 语义 = 构建种类（native/apk/wasm，
    // `list_build` 读它上报）；作业种类用 `job_kind` 表达（避免语义污染 ⇒ 前端 last_kind 回归）。
    meta["job_kind"] = json!("build");
    meta["log"] = json!(format!("evidence/console/logs/build-{kind}-{ts}.log"));
    let args = vec![
        repo_root.join("scripts/build_artifacts.py").to_string_lossy().to_string(),
        "run".to_owned(),
        kind.to_owned(),
    ];
    let mirror = crate::runner::jobs::job_state_path(repo_root, "build", kind);
    let outcome = match crate::runner::jobs::spawn_program_logged(&crate::runner::jobs::SpawnOpts {
        candidates: vec!["python3".to_owned()],
        args,
        cwd: repo_root.to_path_buf(),
        // 构建脚本自写日志（evidence/console/logs/build-*.log）⇒ stdout/stderr 丢弃
        log_path: None,
        state_path: lock_path.to_path_buf(),
        status_meta: meta,
        timeout_s: None,
        mirror_path: Some(mirror.clone()),
    }) {
        Ok(v) => v,
        Err(e) => {
            let _ = std::fs::remove_file(lock_path);
            return Err(format!("后台构建 spawn 失败（python3 缺位？）：{e}"));
        }
    };
    Ok(json!({
        "started": true,
        "pid": outcome.pid,
        "kind": kind,
        "note": "后台构建已启动（统一作业模型 23 号 §6.2；日志落 evidence/console/logs/build-*.log；构建后自动复检）；轮询 /api/build 查产物 / /api/jobs?kind=build 查作业状态",
    }))
}

/// 最近一次构建日志解析（S46.5 抽出，降 `list_build` 行数）：优先取 `build.lock` 记录的
/// `log` 字段（收割线程落盘的真实末次路径）；锁缺失 / 无 log 字段 / 路径不存在时回退全目录
/// 扫 `build-*.log` 按 mtime 降序取 1（gitignored 子目录，全缺 = 未跑过构建）。原实现纯靠
/// mtime 扫描：写日志的文件句柄在构建过程中持续刷新 mtime ⇒ 排序可能取到**旧日志**
///（S46.5 用户报「日志不更新」根因）。
fn resolve_last_build_log(repo_root: &Path, lock_path: &Path) -> Option<Value> {
    let tail_of = |p: &Path| {
        let t = std::fs::read_to_string(p).unwrap_or_default();
        json!({
            "path": p.display().to_string(),
            "tail": t.lines().take(8).collect::<Vec<_>>(),
            "head": t.lines().take(2).collect::<Vec<_>>(),
        })
    };
    let lock_v0: Value = std::fs::read_to_string(lock_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    if let Some(rel) = lock_v0.get("log").and_then(Value::as_str) {
        let p = repo_root.join(rel);
        if p.is_file() {
            return Some(tail_of(&p));
        }
    }
    let logs_dir = repo_root.join("evidence/console/logs");
    if let Ok(rd) = std::fs::read_dir(&logs_dir) {
        let mut cands: Vec<(std::path::PathBuf, u64)> = Vec::new();
        for e in rd.flatten() {
            let n = e.file_name();
            let s = n.to_string_lossy().into_owned();
            if s.starts_with("build-") && s.to_ascii_lowercase().ends_with(".log") {
                let p = logs_dir.join(&n);
                // mtime：`metadata→modified→elapsed` 链，`as_millis` 截断用 `try_from` 保底
                // （u128→u64 无溢出风险——mtime 毫秒量级，clippy cast_possible_truncation）。
                let mtime = p
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.elapsed().ok())
                    .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
                cands.push((p, mtime));
            }
        }
        cands.sort_by_key(|c| std::cmp::Reverse(c.1));
        if let Some((p, _)) = cands.first() {
            return Some(tail_of(p));
        }
    }
    None
}

/// 只读面：`build_artifacts.py list --json`（结构化产物体积/mtime）+ 最近一次构建日志 + 构建锁状态。
fn list_build(repo_root: &Path, lock_path: &Path) -> Result<Value, String> {
    let mut list_cmd = std::process::Command::new("python3");
    list_cmd.arg(repo_root.join("scripts/build_artifacts.py")).arg("list").arg("--json");
    // 22 号 §10.3 最小环境集（env_clear + 白名单）；r4-b P2 修复。
    crate::runner::apply_minimal_env(&mut list_cmd);
    let out = list_cmd.output();
    let out = match out {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            return Err(format!(
                "build_artifacts.py list 非 0 退出：{}",
                String::from_utf8_lossy(&o.stderr).trim()
            ));
        }
        Err(e) => return Err(format!("build_artifacts.py 执行失败（python3 缺位？）：{e}")),
    };
    // S49.5：`list --json` 结构化产物（artifacts[]，kind/artifact/size_mb/mtime/present/note）
    // 供前端渲染卡片；`list` 纯文本面派生（与 `build_artifacts.py list` 同格式）保持兼容。
    let artifacts: Vec<Value> = String::from_utf8_lossy(&out.stdout)
        .parse::<Value>()
        .ok()
        .and_then(|v| v.get("artifacts").cloned())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let mut list_body = String::new();
    for a in &artifacts {
        let kind = a.get("kind").and_then(Value::as_str).unwrap_or("?");
        if a.get("present").and_then(Value::as_bool).unwrap_or(false) {
            let art = a.get("artifact").and_then(Value::as_str).unwrap_or("");
            let mb = a.get("size_mb").and_then(Value::as_f64).unwrap_or(0.0);
            let mtime = a.get("mtime").and_then(Value::as_str).unwrap_or("").to_string();
            // 派生纯文本 mtime = "%m-%d %H:%M"（从 "%Y-%m-%d %H:%M" 去年份段）
            let short = if mtime.len() >= 11 { &mtime[5..] } else { mtime.as_str() };
            // clippy format_push_string：push_str(&format!(..)) → writeln!（写 String 免临时）
            let _ = writeln!(list_body, "{kind:8} {art}  {mb:.1}MB  mtime={short}");
        } else {
            let _ = writeln!(list_body, "{kind:8} 未构建");
        }
    }
    // 最近一次构建日志：优先取 `build.lock` 记录的 `log` 字段（收割线程落盘的真实末次路径）；
    // 锁缺失 / 无 log 字段 / 路径不存在时回退全目录扫 `build-*.log` 按 mtime 降序取 1
    // （gitignored 子目录，全缺 = 未跑过构建）。原实现纯靠 mtime 扫描：写日志的文件句柄在
    // 构建过程中持续刷新 mtime ⇒ 排序可能取到**旧日志**（S46.5 用户报「日志不更新」根因）。
    let last_log = resolve_last_build_log(repo_root, lock_path);
    // S46.3：构建锁状态上报（running / exit_code / 末次 kind），前端据此禁用按钮 + 显示完成态。
    let lock_state: Value = std::fs::read_to_string(lock_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    let mut running = false;
    let mut running_pid = 0u64;
    if let Some(v) = lock_state.as_object() {
        if let Some(r) = v.get("running").and_then(Value::as_bool) {
            running = r;
        }
        if let Some(p) = v.get("pid").and_then(Value::as_u64) {
            running_pid = p;
        }
        // 锁标记 running 但进程已死（收割线程未及更新/崩溃）⇒ 视作未在跑。
        if running
            && running_pid > 0
            && !std::path::Path::new(&format!("/proc/{running_pid}")).exists()
        {
            running = false;
        }
    }
    Ok(json!({
        "list": list_body,
        "artifacts": artifacts,
        "last_build_log": last_log,
        "running": running,
        "running_pid": running_pid,
        "last_exit": lock_state.get("exit_code").cloned().unwrap_or(Value::Null),
        "last_kind": lock_state.get("kind").cloned().unwrap_or(Value::Null),
        "started_ts": lock_state.get("started_ts").cloned().unwrap_or(Value::Null),
        "ended_ts": lock_state.get("ended_ts").cloned().unwrap_or(Value::Null),
        "tool": "scripts/build_artifacts.py（list --json 结构化 + list 纯文本派生；触发构建 = console_commands build_{apk,wasm,native}）",
    }))
}
