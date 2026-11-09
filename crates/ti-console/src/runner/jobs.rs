//! 统一作业模型（23 号 §6.2；S49.7）——平台任何「跑一条命令」的动作共用此实现。
//!
//! 动机（用户 2026-11-09 拍板「从架构上要统一设计，复用」）：平台原有四类作业各写一套
//! spawn/状态/日志（测试自写且**不记 `exit_code`**、构建自写 + 自写锁、发布/门禁各拼状态路径）
//! ⇒ 「跑完了吗 / 成没成 / 证明在哪」三个问题没有统一答案，平台测试页交互无法闭环。
//!
//! 统一口径（23 号 §6.2）：
//! - **作业种类** = `test` / `build` / `publish` / `gate`（四类，见 `JOB_KINDS`）；
//! - **状态文件** `evidence/console/jobs/<kind>-<id>.json`：
//!   `{kind,id,running,pid,exit_code,started_ts,ended_ts,log,evidence[],note,timeout_s}`；
//! - **日志** `evidence/console/logs/<kind>-<id>-<ts>.log`（流式写 ⇒ 输出实时可见）；
//! - **收割线程**：等子进程退出 ⇒ 写 `running=false` + `exit_code` + `ended_ts`
//!   （信号终止无退出码 = `-1`；超时 = `-1` + note `timeout`）⇒ 结果即判据，不靠日志字符串猜；
//! - **证明**：作业产物登记进状态文件 `evidence[]` ⇒ 证明与本次运行绑定。
//!
//! 安全口径（22 号 §10.3）：程序白名单 `cargo`/`python3`/`pwsh`/`powershell` + **逐参数非 shell**
//! （禁 `sh -c`/`cmd /c`）；最小环境集 [`crate::runner::apply_minimal_env`]。
//!
//! 本模块 **0 游戏依赖**（不引 ti-core/ti-engine/bevy）；只管进程与文件。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// 作业种类（23 号 §6.2）。
pub const JOB_KINDS: [&str; 4] = ["test", "build", "publish", "gate"];

/// 程序白名单（22 号 §10.3）。
pub const JOB_PROGRAMS: [&str; 4] = ["cargo", "python3", "pwsh", "powershell"];

/// 脚本解释器候选（22 号 §4.3 回退序：`pwsh` → `powershell`）。
const SCRIPT_INTERPS: [&str; 2] = ["pwsh", "powershell"];

/// 作业条目（声明式；`program` + `args` = 逐参数执行，禁 shell 拼接）。
#[derive(Debug, Clone)]
pub struct JobSpec {
    /// 作业种类（`test`/`build`/`publish`/`gate`）。
    pub kind: String,
    /// 业务 id（测试 id / `apk` / `publish` / `gate-fast` …；须匹配 `[a-z0-9_-]{1,64}`）。
    pub id: String,
    /// 程序名（白名单内）。
    pub program: String,
    /// 参数（逐参数；禁 shell 拼接面）。
    pub args: Vec<String>,
    /// 超时秒数（`None` = 不限）。
    pub timeout_s: Option<u64>,
    /// 附加字段（写进状态文件，如 `command` / `tier` / `note`）。
    pub meta: Value,
}

impl JobSpec {
    /// 作业条目构造（`kind`/`id`/`program` 合法性在此把关）。
    ///
    /// # Errors
    ///
    /// `kind` 不在四种之一 / `id` 不合法 / `program` 不在白名单 ⇒ 报错（不 spawn）。
    pub fn new(
        kind: &str,
        id: &str,
        program: &str,
        args: Vec<String>,
        meta: Value,
    ) -> Result<Self, String> {
        if !JOB_KINDS.contains(&kind) {
            return Err(format!("作业种类 `{kind}` 不在 {JOB_KINDS:?}（23 号 §6.2）"));
        }
        if !is_legal_job_id(id) {
            return Err(format!("作业 id `{id}` 非法（须 ^[a-z0-9_-]{{1,64}}$）"));
        }
        if !JOB_PROGRAMS.contains(&program) {
            return Err(format!("程序 `{program}` 不在白名单 {JOB_PROGRAMS:?}（22 号 §10.3）"));
        }
        Ok(Self {
            kind: kind.to_owned(),
            id: id.to_owned(),
            program: program.to_owned(),
            args,
            timeout_s: None,
            meta,
        })
    }
}

/// 作业 id 合法性（写状态/日志文件名前置，防路径穿越）。
fn is_legal_job_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Unix 秒（系统时钟异常 = 0）。
fn now_unix_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// 作业状态目录（`evidence/console/jobs/`；gitignored）。
fn jobs_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("evidence/console/jobs")
}

/// 作业日志目录（`evidence/console/logs/`；gitignored）。
fn logs_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("evidence/console/logs")
}

/// 作业状态文件路径。
#[must_use]
pub fn job_state_path(repo_root: &Path, kind: &str, id: &str) -> PathBuf {
    jobs_dir(repo_root).join(format!("{kind}-{id}.json"))
}

/// 作业日志路径（含时间戳 ⇒ 每次运行独立留档）。
#[must_use]
pub fn job_log_path(repo_root: &Path, kind: &str, id: &str, ts: u64) -> PathBuf {
    logs_dir(repo_root).join(format!("{kind}-{id}-{ts}.log"))
}

/// 后台 spawn 一条作业（统一模型：日志流式写 + 收割线程记终态）。
///
/// # Errors
///
/// 日志/状态目录创建失败、日志文件创建失败、spawn 失败（程序缺位）、`meta` 序列化失败 ⇒ 报错。
pub fn spawn_job(repo_root: &Path, spec: &JobSpec) -> Result<Value, String> {
    std::fs::create_dir_all(logs_dir(repo_root))
        .map_err(|e| format!("作业日志目录创建失败 {e}"))?;
    std::fs::create_dir_all(jobs_dir(repo_root))
        .map_err(|e| format!("作业状态目录创建失败 {e}"))?;
    let ts = now_unix_secs();
    let log_path = job_log_path(repo_root, &spec.kind, &spec.id, ts);
    let state_path = job_state_path(repo_root, &spec.kind, &spec.id);
    let log_rel = log_path.strip_prefix(repo_root).map_or_else(
        |_| log_path.to_string_lossy().to_string(),
        |p| p.to_string_lossy().replace('\\', "/"),
    );
    let mut meta = spec.meta.clone();
    if let Some(obj) = meta.as_object_mut() {
        obj.insert("kind".into(), json!(spec.kind));
        obj.insert("id".into(), json!(spec.id));
        obj.insert("started_ts".into(), json!(ts));
        obj.insert("ended_ts".into(), Value::Null);
        obj.insert("exit_code".into(), Value::Null);
        obj.insert("running".into(), json!(true));
        obj.insert("pid".into(), json!(0));
        obj.insert("log".into(), json!(log_rel));
        obj.insert("timeout_s".into(), json!(spec.timeout_s));
        if !obj.contains_key("evidence") {
            obj.insert("evidence".into(), json!([] as [String; 0]));
        }
    }
    // 脚本类走解释器回退（pwsh → powershell）；cargo/python3 单候选。
    let candidates: Vec<String> = if SCRIPT_INTERPS.contains(&spec.program.as_str()) {
        SCRIPT_INTERPS.iter().map(|s| (*s).to_owned()).collect()
    } else {
        vec![spec.program.clone()]
    };
    let outcome = spawn_program_logged(&SpawnOpts {
        candidates: candidates.clone(),
        args: spec.args.clone(),
        cwd: repo_root.to_path_buf(),
        log_path: Some(log_path.clone()),
        state_path: state_path.clone(),
        status_meta: meta.clone(),
        timeout_s: spec.timeout_s,
        mirror_path: None,
    })?;
    // 回填真实 pid + 命令（状态文件初值已写 pid=0 占位）。
    if let Ok(text) = std::fs::read_to_string(&state_path) {
        if let Ok(mut v) = serde_json::from_str::<Value>(&text) {
            if let Some(obj) = v.as_object_mut() {
                obj.insert("pid".into(), json!(outcome.pid));
                obj.insert(
                    "command".into(),
                    json!(format!("{} {}", spec.program, spec.args.join(" ")).trim()),
                );
            }
            let _ = std::fs::write(&state_path, v.to_string());
        }
    }
    Ok(json!({
        "started": true,
        "kind": spec.kind,
        "id": spec.id,
        "pid": outcome.pid,
        "log": log_rel,
        "status": state_path
            .strip_prefix(repo_root)
            .map_or_else(
                |_| state_path.to_string_lossy().to_string(),
                |p| p.to_string_lossy().replace('\\', "/"),
            ),
        "note": "作业已后台启动（统一作业模型 23 号 §6.2）；轮询 GET /api/jobs?kind=<kind>&id=<id> 查状态/退出码/证明",
    }))
}

/// 后台 spawn 参数包（参数过多 ⇒ 收敛为 struct；23 号 §6.2 作业模型）。
#[derive(Debug, Clone)]
pub struct SpawnOpts {
    /// 候选程序（回退序；`pwsh` → `powershell`；拥有型，避免 `'static` 借用约束）。
    pub candidates: Vec<String>,
    /// 参数（逐参数；禁 shell 拼接面）。
    pub args: Vec<String>,
    /// 工作目录（仓库根）。
    pub cwd: PathBuf,
    /// 日志落盘目标（`None` = stdout/stderr 丢弃；脚本自写日志时用）。
    pub log_path: Option<PathBuf>,
    /// 状态文件（JSON）路径。
    pub state_path: PathBuf,
    /// 状态文件初始内容（helper 追加 `pid` / `running`）。
    pub status_meta: Value,
    /// 超时秒数（`None` = 不限）。
    pub timeout_s: Option<u64>,
    /// 统一作业状态镜像路径（`None` = 不镜像）。
    pub mirror_path: Option<PathBuf>,
}

/// 后台 spawn（候选程序回退 + 日志落盘 + 收割线程记终态 + 可选超时 + 可选统一状态镜像）。
///
/// 收割线程：轮询 `try_wait`（超时判定）→ 正常退出记 `exit_code`（信号终止 = `-1`）；
/// 超时则 `kill` 子进程并记 `exit_code=-1` + `note="timeout"`。
/// `mirror_path` = 统一作业状态镜像（`evidence/console/jobs/<kind>-<id>.json`）⇒ 门禁/发布/构建
/// 保留各自状态文件（互斥锁语义不变）的同时，**统一查询面**也能读到（23 号 §6.2 查询行）。
///
/// # Errors
///
/// 日志文件创建/克隆失败、全部候选程序 spawn 失败 ⇒ 报错（附最后一次失败原因）。
pub fn spawn_program_logged(opts: &SpawnOpts) -> Result<SpawnOutcome, String> {
    let log_file = match &opts.log_path {
        Some(path) => {
            Some(std::fs::File::create(path).map_err(|e| format!("日志文件创建失败 {e}"))?)
        }
        None => None,
    };
    let mut last_err = String::new();
    for program in &opts.candidates {
        let mut cmd = std::process::Command::new(program);
        cmd.args(&opts.args);
        cmd.current_dir(&opts.cwd);
        // 22 号 §10.3 最小环境集（env_clear + 白名单）。
        crate::runner::apply_minimal_env(&mut cmd);
        if let Some(file) = &log_file {
            let stdout = file.try_clone().map_err(|e| format!("日志克隆失败 {e}"))?;
            let stderr = file.try_clone().map_err(|e| format!("日志克隆失败 {e}"))?;
            cmd.stdout(std::process::Stdio::from(stdout));
            cmd.stderr(std::process::Stdio::from(stderr));
        } else {
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
        }
        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                let mut meta = opts.status_meta.clone();
                if let Some(obj) = meta.as_object_mut() {
                    obj.insert("pid".into(), json!(pid));
                    obj.insert("running".into(), json!(true));
                }
                let _ = std::fs::write(&opts.state_path, meta.to_string());
                // 统一作业镜像：spawn 后**立即**写初值（running 期间 `/api/jobs` 可查）
                if let Some(mirror_path) = &opts.mirror_path {
                    let _ = std::fs::write(mirror_path, meta.to_string());
                }
                // 收割线程：等退出（可超时）⇒ 写 running=false + exit_code + ended_ts。
                let meta_path = opts.state_path.clone();
                let mirror_path = opts.mirror_path.clone();
                let timeout_s = opts.timeout_s;
                std::thread::spawn(move || {
                    let mut code: Option<i32> = None;
                    let mut timed_out = false;
                    if let Some(limit) = timeout_s {
                        let deadline = std::time::Instant::now()
                            + std::time::Duration::from_secs(limit.max(1));
                        loop {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    code = status.code();
                                    break;
                                }
                                Ok(None) => {
                                    if std::time::Instant::now() >= deadline {
                                        let _ = child.kill();
                                        timed_out = true;
                                        break;
                                    }
                                    std::thread::sleep(std::time::Duration::from_millis(200));
                                }
                                Err(_) => break,
                            }
                        }
                    } else {
                        code = child.wait().ok().and_then(|s| s.code());
                    }
                    let exit_code = if timed_out { -1 } else { code.unwrap_or(-1) };
                    let ended_ts = now_unix_secs();
                    let mut done = meta;
                    if let Some(obj) = done.as_object_mut() {
                        obj.insert("running".into(), json!(false));
                        obj.insert("exit_code".into(), json!(exit_code));
                        obj.insert("ended_ts".into(), json!(ended_ts));
                        if timed_out {
                            obj.insert("note".into(), json!("timeout"));
                        }
                    }
                    let _ = std::fs::write(&meta_path, done.to_string());
                    // 统一作业状态镜像（jobs 目录）⇒ /api/jobs 可查（23 号 §6.2 查询行）。
                    if let Some(mirror_path) = mirror_path {
                        let _ = std::fs::write(&mirror_path, done.to_string());
                    }
                });
                return Ok(SpawnOutcome { pid, program: program.clone() });
            }
            Err(e) => last_err = format!("{program}: {e}"),
        }
    }
    Err(format!("后台 spawn 失败（{:?} 均缺位？）：{last_err}", opts.candidates))
}

/// 后台 spawn 结果。
#[derive(Debug, Clone)]
pub struct SpawnOutcome {
    /// 子进程 pid。
    pub pid: u32,
    /// 实际使用的程序名（回退后的）。
    pub program: String,
}

/// 读作业状态（状态文件不存在 = `None`）。
#[must_use]
pub fn job_status(repo_root: &Path, kind: &str, id: &str) -> Option<Value> {
    let path = job_state_path(repo_root, kind, id);
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// 列作业状态（`kind` = `None` ⇒ 全部种类；按 `started_ts` 降序）。
#[must_use]
pub fn job_list(repo_root: &Path, kind: Option<&str>) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    let Ok(rd) = std::fs::read_dir(jobs_dir(repo_root)) else {
        return out;
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.to_ascii_lowercase().ends_with(".json") {
            continue;
        }
        let Some(kind_of) = name.split_once('-').map(|(k, _)| k.to_owned()) else {
            continue;
        };
        if let Some(want) = kind {
            if kind_of != want {
                continue;
            }
        }
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            out.push(v);
        }
    }
    out.sort_by(|a, b| {
        let key = |v: &Value| v.get("started_ts").and_then(Value::as_u64).unwrap_or(0);
        key(b).cmp(&key(a))
    });
    out
}

/// 作业日志尾部（末 `lines` 行；状态文件里的 `log` 路径优先，缺失 = 无日志）。
pub fn job_log_tail(repo_root: &Path, kind: &str, id: &str, lines: usize) -> Option<String> {
    let state = job_status(repo_root, kind, id)?;
    let rel = state.get("log").and_then(Value::as_str)?;
    let path = repo_root.join(rel);
    let text = std::fs::read_to_string(path).ok()?;
    let all: Vec<&str> = text.lines().collect();
    let take = lines.clamp(1, 400);
    let start = all.len().saturating_sub(take);
    Some(all[start..].join("\n"))
}

/// 登记作业证明（产物路径并入状态文件 `evidence[]`；幂等去重）。
pub fn register_evidence(repo_root: &Path, kind: &str, id: &str, paths: &[String]) -> bool {
    let path = job_state_path(repo_root, kind, id);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(mut v) = serde_json::from_str::<Value>(&text) else {
        return false;
    };
    let Some(obj) = v.as_object_mut() else {
        return false;
    };
    let mut list: Vec<String> = obj
        .get("evidence")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
        .unwrap_or_default();
    for p in paths {
        if !list.iter().any(|e| e == p) {
            list.push(p.clone());
        }
    }
    obj.insert("evidence".into(), json!(list));
    std::fs::write(&path, v.to_string()).is_ok()
}

/// 作业查询面（23 号 §6.2 查询行）：列表 / 详情（含日志尾 + 证明）。
///
/// - `id` 缺省 = 列表（`kind` 缺省 = 全部种类；按 `started_ts` 降序）；
/// - `id` 给出 = 详情（`state` + `log_tail` 末 200 行 + `evidence[]`）。
#[must_use]
pub fn jobs_view(repo_root: &Path, kind: Option<&str>, id: Option<&str>) -> Value {
    if let Some(test_id) = id.filter(|s| !s.is_empty()) {
        let kind = kind.filter(|k| JOB_KINDS.contains(k)).unwrap_or("test");
        let state = job_status(repo_root, kind, test_id);
        let tail = job_log_tail(repo_root, kind, test_id, 200);
        let evidence = state
            .as_ref()
            .and_then(|s| s.get("evidence").cloned())
            .unwrap_or_else(|| json!([] as [String; 0]));
        return json!({
            "ok": state.is_some(),
            "kind": kind,
            "id": test_id,
            "state": state.unwrap_or(Value::Null),
            "log_tail": tail.unwrap_or_default(),
            "evidence": evidence,
            "note": "统一作业模型查询面（23 号 §6.2）；exit_code = 机器判据（0 通过），evidence[] = 本次运行证明",
        });
    }
    let list = job_list(repo_root, kind.filter(|k| !k.is_empty()));
    json!({
        "ok": true,
        "kind": kind.unwrap_or(""),
        "count": list.len(),
        "jobs": list,
        "kinds": JOB_KINDS,
        "note": "统一作业模型列表（23 号 §6.2）",
    })
}
