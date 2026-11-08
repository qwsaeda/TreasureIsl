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
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
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
    let mut cmd = std::process::Command::new("python3");
    cmd.arg(repo_root.join("scripts/build_artifacts.py")).arg("run").arg(kind);
    // 22 号 §10.3 最小环境集（env_clear + 白名单）；r4-b P2 修复。
    crate::runner::apply_minimal_env(&mut cmd);
    let mut child =
        match cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_file(lock_path);
                return Err(format!("后台构建 spawn 失败（python3 缺位？）：{e}"));
            }
        };
    let pid = child.id();
    // 更新锁为真实 pid + 建日志路径字段。
    let mut meta = placeholder;
    meta["pid"] = json!(pid);
    meta["log"] = json!(format!("evidence/console/logs/build-{kind}-{ts}.log"));
    let _ = std::fs::write(lock_path, meta.to_string());
    // 收割线程：等构建进程退出 → 锁置 running=false + exit_code + ended_ts（保留末次态供 /api/build 上报）。
    let lock_meta = meta.clone();
    let lock_p = lock_path.to_path_buf();
    std::thread::spawn(move || {
        let code = child.wait().ok().and_then(|s| s.code()).unwrap_or(-1);
        let ended = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let mut done = lock_meta;
        done["running"] = json!(false);
        done["exit_code"] = json!(code);
        done["ended_ts"] = json!(ended);
        let _ = std::fs::write(&lock_p, done.to_string());
    });
    Ok(json!({
        "started": true,
        "pid": pid,
        "kind": kind,
        "note": "后台构建已启动（日志落 evidence/console/logs/build-*.log；构建后自动复检）；轮询 /api/build 查产物",
    }))
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
    // 最近一次构建日志（按 mtime 降序取 1；gitignored 子目录，缺失 = 未跑过构建）
    let logs_dir = repo_root.join("evidence/console/logs");
    let mut last_log: Option<Value> = None;
    if let Ok(rd) = std::fs::read_dir(&logs_dir) {
        let mut cands: Vec<(std::path::PathBuf, u64)> = Vec::new();
        for e in rd.flatten() {
            let n = e.file_name();
            let s = n.to_string_lossy().into_owned();
            if s.starts_with("build-") && s.to_ascii_lowercase().ends_with(".log") {
                let p = repo_root.join("evidence/console/logs").join(&n);
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
            let t = std::fs::read_to_string(p).unwrap_or_default();
            last_log = Some(json!({
                "path": p.display().to_string(),
                "tail": t.lines().take(8).collect::<Vec<_>>(),
            }));
        }
    }
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
        "tool": "scripts/build_artifacts.py（list --json 结构化 + list 纯文本派生；触发构建 = console_commands build_{apk,wasm,native}）",
    }))
}
