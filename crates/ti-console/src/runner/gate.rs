//! 门禁执行 runner（S32.4）：`POST /api/gate` 后台跑 `gate.ps1`（tier 分档）。
//!
//! 白名单（`console_commands.json` 机器真源，22 号 §4.2）：
//! id=`gate`，runner=`pwsh`，script=`scripts/gate.ps1`，tier∈{fast,touch,full}
//! → `-Fast` / `-Touch` / 无开关（full = 全量）。
//!
//! 单实例锁：`evidence/console/gate.lock`（JSON：`pid/tier/started_ts/log/running`；
//! 收割线程等子进程退出后置 `running=false` + `exit_code`）。锁存在且 `running=true`
//! 且 pid 存活 = 忙（409）；否则视为陈旧，清除后可重新启动。
//!
//! 门禁既有副作用（`evidence/web_playtest.{log,png}`、`evidence/font_render.txt`）
//! **只记录不判定**（22 号 §3.1 已列；平台不对门禁产物做断言，判定归 `gate.ps1` 自身）。

use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// tier 白名单（`console_commands.json` `enum_param.values` 镜像；22 号 §4.2）。
pub const TIERS: [&str; 3] = ["fast", "touch", "full"];

/// 门禁既有副作用路径（`console_commands.json` `gate.writes` 镜像；22 号 §3.1）：
/// 跑一次 gate 必写这三个跟踪文件。**只记录不判定**（平台不对产物做断言，判定归 gate.ps1）。
pub const SIDE_EFFECTS: [&str; 3] =
    ["evidence/web_playtest.log", "evidence/web_playtest.png", "evidence/font_render.txt"];

/// 单实例锁路径（`evidence/console/gate.lock`；区别于 main.rs 的平台进程锁 `.lock`）。
fn lock_path(repo_root: &Path) -> PathBuf {
    repo_root.join("evidence/console/gate.lock")
}

/// 执行日志目录（与 `run_test`/`build` 同目录面；gitignored）。
fn logs_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("evidence/console/logs")
}

/// tier → gate.ps1 开关参数（full = 无开关 = 全量；与 `console_commands.json` `arg_map` 一致）。
fn tier_args(tier: &str) -> Option<Vec<&'static str>> {
    match tier {
        "fast" => Some(vec!["-Fast"]),
        "touch" => Some(vec!["-Touch"]),
        "full" => Some(vec![]),
        _ => None,
    }
}

/// 读锁元数据（gate.lock JSON；缺/非法 = None）。
fn read_lock(repo_root: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(lock_path(repo_root)).ok()?;
    serde_json::from_str(&content).ok()
}

/// 进程存活（Linux = `/proc/<pid>` 存在；非 Linux = 锁存在即活，由收割线程清锁）。
fn process_alive(pid: u32) -> bool {
    if cfg!(target_os = "linux") {
        return Path::new(&format!("/proc/{pid}")).exists();
    }
    true
}

/// 陈旧接管阈值（22 号 §10.4 口径：起始时间超 30 分钟 + pid 不存在 ⇒ 接管）。
const STALE_TAKEOVER_SECS: u64 = 1800;

/// 损坏锁接管最小年龄（r3-b P2 修复）：锁文件存在但非合法 JSON（进程崩溃于
/// `create_new`→`writeln` 微秒窗口）时，仅当文件已存在超此秒数才视为「崩溃遗留」接管，
/// 防误删「正持锁写入中」的活锁（否则双启动竞态）。
const CORRUPT_LOCK_MIN_AGE_SECS: u64 = 5;

/// 锁文件是否已存在超过 `secs` 秒（mtime 判龄；损坏锁接管防活锁误删用）。
fn lock_older_than(path: &Path, secs: u64) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let Ok(age) = modified.elapsed() else {
        return false;
    };
    age.as_secs() > secs
}

/// 锁是否陈旧可接管（running=false=已完成，或 pid 死且起始超 30 分钟）。
fn lock_is_stale(meta: &Value) -> bool {
    if meta.get("running").and_then(Value::as_bool) == Some(false) {
        return true; // 已完成 → 下次启动可复用
    }
    let pid = meta.get("pid").and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok());
    if pid.is_some_and(process_alive) {
        return false; // 活 → 真忙
    }
    let started = meta.get("started_ts").and_then(Value::as_u64).unwrap_or(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    now.saturating_sub(started) > STALE_TAKEOVER_SECS
}

/// 锁预检（22 号 §10.4 + r3-b P2 修复）：锁存在且非陈旧 → 409；陈旧/损坏（超 5 秒）→ 清除（接管）；
/// 损坏 < 5 秒（可能正写入）→ 409。返回 `Err((code, msg))` = 拒绝启动；`Ok(())` = 可继续 `create_new`。
/// `pub` 供 `gate_tests.rs` 单测直接验证损坏锁接管分支（r4-b P2 修复）。
///
/// # Errors
///
/// `(409, _)` = 锁活跃或损坏 < 5 秒（可能正写入）；`(500, _)` 不用（仅 IO 预检路径，预检本身不建文件）。
pub fn check_lock_available(repo_root: &Path) -> Result<(), (u16, String)> {
    if !lock_path(repo_root).exists() {
        return Ok(());
    }
    match read_lock(repo_root) {
        Some(meta) if !lock_is_stale(&meta) => Err((
            409u16,
            format!("门禁正在运行（{meta}）；等待完成或先删 {}", lock_path(repo_root).display()),
        )),
        Some(_) => {
            // 陈旧（running=false 或死 pid 超 30 分钟）→ 接管。
            let _ = std::fs::remove_file(lock_path(repo_root));
            Ok(())
        }
        None => {
            // 锁存在但非合法 JSON：仅当超 5 秒 = 崩溃遗留 → 接管；< 5 秒 = 可能正写入 → 409。
            if lock_older_than(&lock_path(repo_root), CORRUPT_LOCK_MIN_AGE_SECS) {
                let _ = std::fs::remove_file(lock_path(repo_root));
                Ok(())
            } else {
                Err((
                    409u16,
                    format!(
                        "门禁正在运行（锁写入中或损坏 <5 秒）；稍后再试或先删 {}",
                        lock_path(repo_root).display()
                    ),
                ))
            }
        }
    }
}

/// 后台启动 gate.ps1（tier 分档）。
///
/// # Errors
///
/// - `(400, msg)` = 非法 tier（`tier_args` 未命中）；
/// - `(409, msg)` = 单实例锁占用（门禁正在运行；含并发竞态 `create_new` 失败）；
/// - `(500, msg)` = IO（`logs 目录` / `日志文件` / `锁写入`）或 spawn 失败（pwsh/powershell 均缺位）。
pub fn run_gate(repo_root: &Path, tier: &str) -> Result<Value, (u16, String)> {
    let args = tier_args(tier)
        .ok_or_else(|| (400u16, format!("非法 tier `{tier}`（可选 {}", TIERS.join("/"))))?;

    // 单实例锁（22 号 §10.4）：先查忙/陈旧/损坏（存在且非陈旧 → 409），再原子 create_new 占锁（防双启动竞态）。
    check_lock_available(repo_root)?;

    let dir = logs_dir(repo_root);
    std::fs::create_dir_all(&dir).map_err(|e| (500u16, format!("logs 目录创建失败 {e}")))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let log_name = format!("gate-{ts}.log");
    let log_path = dir.join(&log_name);

    // 原子占锁（O_EXCL create_new）：并发第二个 POST 在此 409（而非先 spawn 再写锁的竞态窗口，
    // A 侧 r1 P2-1 修复）；spawn 前占锁 = 单实例保证严格。
    // 日志文件创建**移到锁成功之后**（r2-a P2 修复）：并发同秒两 POST 时，只有持锁者建日志，
    // 409 败者不再触碰（否则败者 remove_file 会误删持锁者正在写入的日志）。
    let lock_p = lock_path(repo_root);
    let lock_file = std::fs::OpenOptions::new().write(true).create_new(true).open(&lock_p);
    let mut lock_file = match lock_file {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let meta = read_lock(repo_root).unwrap_or(Value::Null);
            return Err((
                409u16,
                format!("门禁正在运行（{meta}）；等待完成或先删 {}", lock_p.display()),
            ));
        }
        Err(e) => return Err((500u16, format!("写锁失败 {e}"))),
    };
    // 占位锁（pid=0 = reserving，spawn 后更新真实 pid）。
    let placeholder = json!({
        "pid": 0,
        "tier": tier,
        "started_ts": ts,
        "log": format!("evidence/console/logs/{log_name}"),
        "running": true,
        "reserving": true,
        "side_effects": SIDE_EFFECTS,
    });
    if writeln!(lock_file, "{placeholder}").is_err() {
        let _ = std::fs::remove_file(&lock_p);
        return Err((500u16, "写锁内容失败".to_owned()));
    }
    // 后台 spawn（共用实现 `runner::spawn`：解释器回退 + 日志落盘 + 收割线程记退出码；
    // 失败释放锁 + 删日志）。状态文件 = 锁文件本身（收割线程更新 running/exit_code/ended_ts）。
    let lock_meta = json!({
        "kind": "gate",
        "id": tier,
        "tier": tier,
        "started_ts": ts,
        "log": format!("evidence/console/logs/{log_name}"),
        "reserving": false,
        "side_effects": SIDE_EFFECTS,
    });
    // S49.7 统一作业模型（23 号 §6.2）：spawn + 收割（exit_code/ended_ts）走 `runner::jobs` 单一实现；
    // 门禁保留自身单实例锁文件（= 状态载体，`GET /api/gate` 读回不变）+ 写统一作业镜像
    // （`evidence/console/jobs/gate-<tier>.json`）⇒ `/api/jobs` 统一查询面可查门禁结果。
    let owned_args: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
    let mirror = crate::runner::jobs::job_state_path(repo_root, "gate", tier);
    let outcome = match crate::runner::jobs::spawn_program_logged(&crate::runner::jobs::SpawnOpts {
        candidates: crate::runner::spawn::SCRIPT_INTERPS.iter().map(|s| (*s).to_owned()).collect(),
        args: owned_args,
        cwd: repo_root.to_path_buf(),
        log_path: Some(log_path.clone()),
        state_path: lock_p.clone(),
        status_meta: lock_meta,
        timeout_s: None,
        mirror_path: Some(mirror.clone()),
    }) {
        Ok(v) => v,
        Err(e) => {
            let _ = std::fs::remove_file(&lock_p);
            let _ = std::fs::remove_file(&log_path);
            return Err((500u16, e));
        }
    };
    let pid = outcome.pid;
    let interp: &str = &outcome.program;

    Ok(json!({
        "started": true,
        "pid": pid,
        "tier": tier,
        "command": format!("{interp} -File scripts/gate.ps1 {}", args.join(" ")),
        "interp": interp,
        "log": format!("evidence/console/logs/{log_name}"),
        "side_effects": SIDE_EFFECTS,
        "note": "门禁已后台启动（日志落 evidence/console/logs/gate-<ts>.log）；轮询 GET /api/gate 查运行状态与历史。门禁既有副作用（side_effects 列出）只记录不判定（22 号 §3.1）",
    }))
}

/// `GET /api/gate`：运行状态 + 历史（gate-*.log 清单）+ 指定日志尾部（?log=<name>）。
#[must_use]
pub fn gate_status(repo_root: &Path, log_name: Option<&str>) -> Value {
    let meta = read_lock(repo_root);
    let running = meta.as_ref().is_some_and(|m| !lock_is_stale(m));

    let dir = logs_dir(repo_root);
    let mut history: Vec<Value> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("gate-") || !name.to_ascii_lowercase().ends_with(".log") {
                continue;
            }
            let size = e.metadata().map_or(0, |m| m.len());
            let mtime = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            history.push(json!({ "name": name, "size": size, "mtime": mtime }));
        }
    }
    history.sort_by(|a, b| b["mtime"].as_u64().cmp(&a["mtime"].as_u64()));

    // 指定日志尾部（?log=<name>；白名单形状校验 + 防穿越（拒 `..`/`/`/`\`），最多 400 行）。
    let tail = log_name.and_then(|name| {
        if !name.starts_with("gate-")
            || !name.to_ascii_lowercase().ends_with(".log")
            || name.contains("..")
            || name.contains('/')
            || name.contains('\\')
        {
            return None;
        }
        let p = dir.join(name);
        std::fs::read_to_string(&p).ok().map(|t| {
            let lines: Vec<&str> = t.lines().collect();
            let start = lines.len().saturating_sub(400);
            lines[start..].join("\n")
        })
    });

    json!({
        "running": running,
        "last": meta,
        "history": history,
        "tail": tail,
        "note": "gate 门禁执行面：tier=fast/touch/full → -Fast/-Touch/全量；fast/touch 为开发档，不构成 gate 全绿证据（S32.4 dod；全量 full 才可置 verified）",
    })
}
