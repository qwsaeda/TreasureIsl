//! 统一作业模型（23 号 §6.2；S49.7）——平台任何「跑一条命令」的动作共用此实现。
//!
//! 动机（用户 2026-11-09 拍板「从架构上要统一设计，复用」）：平台原有四类作业各写一套
//! spawn/状态/日志（测试自写且**不记 `exit_code`**、构建自写 + 自写锁、发布/门禁各拼状态路径）
//! ⇒ 「跑完了吗 / 成没成 / 证明在哪」三个问题没有统一答案，平台测试页交互无法闭环。
//!
//! 统一口径（23 号 §6.2）：
//! - **作业种类** = `test` / `build` / `publish` / `gate` / `preview`（五类，见 `JOB_KINDS`；
//!   `preview` = S86.3 真渲染预览，`python3` 包装 native `--shot`）；
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

use crate::runner::result_schema;

/// 作业种类（23 号 §6.2；S86.3 增 `preview` = 真渲染预览后台出图）。
pub const JOB_KINDS: [&str; 5] = ["test", "build", "publish", "gate", "preview"];

/// 程序白名单（22 号 §10.3）。
pub const JOB_PROGRAMS: [&str; 4] = ["cargo", "python3", "pwsh", "powershell"];

/// 脚本解释器候选（22 号 §4.3 回退序：`pwsh` → `powershell`）。
const SCRIPT_INTERPS: [&str; 2] = ["pwsh", "powershell"];

/// 作业条目（声明式；`program` + `args` = 逐参数执行，禁 shell 拼接）。
#[derive(Debug, Clone)]
pub struct JobSpec {
    /// 作业种类（`test`/`build`/`publish`/`gate`/`preview`）。
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
    /// `kind` 不在五种之一 / `id` 不合法 / `program` 不在白名单 ⇒ 报错（不 spawn）。
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

/// 证明落点（23 号 §6.2 证明行，审计 Q013）：`prove` 前缀（如 `evidence/tests/<id>/`）
/// 下建 `<id>-<ts>/`，落日志副本 `run.log` + 终态快照 `state.json`，并把两路径登记进
/// 作业状态 `evidence[]` ⇒ 「本次运行证明」与作业 id/时间戳绑定。
pub fn persist_proof(
    repo_root: &Path,
    prove_prefix: Option<&str>,
    kind: &str,
    id: &str,
    log_rel: Option<&str>,
    ended_ts: u64,
    state: &Value,
) {
    let Some(prefix) = prove_prefix else { return };
    if prefix.is_empty() || id.is_empty() {
        return;
    }
    let stamp = format!("{id}-{ended_ts}");
    let prove_dir = repo_root.join(prefix).join(&stamp);
    if std::fs::create_dir_all(&prove_dir).is_err() {
        return;
    }
    let log_copy = prove_dir.join("run.log");
    if let Some(rel) = log_rel {
        if let Ok(bytes) = std::fs::read(repo_root.join(rel)) {
            let _ = std::fs::write(&log_copy, bytes);
        }
    }
    let snapshot = prove_dir.join("state.json");
    let _ = std::fs::write(&snapshot, state.to_string());
    let rel = |p: &PathBuf| -> String {
        p.strip_prefix(repo_root).map_or_else(
            |_| p.to_string_lossy().to_string(),
            |q| q.to_string_lossy().replace('\\', "/"),
        )
    };
    let mut paths = vec![rel(&snapshot)];
    if log_copy.exists() {
        paths.insert(0, rel(&log_copy));
    }
    register_evidence(repo_root, kind, id, &paths);
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
                // 证明落点（23 号 §6.2 证明行）：test 作业 meta.prove 给出落点前缀
                // （如 `evidence/tests/<id>/`），线程结束时在该前缀下建 `<ts>/` 子目录，
                // 落日志副本 + 终态快照，并把两者登记进作业状态 `evidence[]`
                // ⇒ 「本次运行证明」与作业 id/时间戳绑定（审计 Q013 落地）。
                let meta_path = opts.state_path.clone();
                let mirror_path = opts.mirror_path.clone();
                let timeout_s = opts.timeout_s;
                // prove/kind/id/log 已在 meta 内（spawn_job 写入），终态 `done` 直接读取，无需额外捕获。
                let job_cwd = opts.cwd.clone();
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
                    // 证明落点（Q013）：prove 前缀 + <ts>/ 落日志副本 + 终态快照 + 登记 evidence[]。
                    let prove_root = done.get("prove").and_then(Value::as_str);
                    let job_kind = done.get("kind").and_then(Value::as_str).unwrap_or("test");
                    let job_id = done.get("id").and_then(Value::as_str).unwrap_or("");
                    let job_log = done.get("log").and_then(Value::as_str);
                    persist_proof(&job_cwd, prove_root, job_kind, job_id, job_log, ended_ts, &done);
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

/// 作业日志**完整内容**（无 400 行 clamp）——结构化结论解析专用读路径。
///
/// `job_log_tail` 的 `lines.clamp(1, 400)` 是**前端回显**上限（实时展示最近输出），
/// 不能用于结论判定：`gate_full`/`smoke` 多 binary 长日志的前段 binary 会被静默截断
/// （S74.2 诊断根因）。本函数直读整份日志，供 [`test_run_conclusion`] 构造 C1 结构化结果。
#[must_use]
pub fn job_log_full(repo_root: &Path, kind: &str, id: &str) -> Option<String> {
    let state = job_status(repo_root, kind, id)?;
    let rel = state.get("log").and_then(Value::as_str)?;
    std::fs::read_to_string(repo_root.join(rel)).ok()
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
        // 2026-11-10「自解释结论」层：test 作业已结束 ⇒ 解析日志生成结构化结论
        //（哪些测试 binary / 通过数 / 失败数 / 对应说明），替代裸 `test result: ok` 回显。
        let conclusion =
            if kind == "test" { test_run_conclusion(repo_root, "test", test_id) } else { None };
        return json!({
            "ok": state.is_some(),
            "kind": kind,
            "id": test_id,
            "state": state.unwrap_or(Value::Null),
            "log_tail": tail.unwrap_or_default(),
            "evidence": evidence,
            "conclusion": conclusion,
            "note": "统一作业模型查询面（23 号 §6.2）；exit_code = 机器判据（0 通过），evidence[] = 本次运行证明；conclusion = C1 结构化结论（test kind：source=result_json|full_log|legacy，tests[]={name,passed,failed,msg}）",
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

/// 解析 cargo test 日志 → 旧格式自解释结论（2026-11-10「单独 Agent 也能看懂」层）。
///
/// S74.2 起为**兼容回退**：`test_run_conclusion` 优先 C1 结构化结果 + 完整日志解析
/// （[`result_schema::parse_test_blocks`]），本函数保留供既有前端 `details` 字段与测试消费。
///
/// 支持两种日志格式：
/// - 完整 cargo test（`Running tests/xxx.rs` 头 + 逐 test 行 + `test result:`）
/// - smoke.ps1 简化（`Running tests/xxx.rs` + `test result:` 对）
///
/// binary 名关联 manifest `desc`（做什么/测什么/达到什么效果）。
#[must_use]
pub fn parse_test_log(log_text: &str, repo_root: &Path) -> Value {
    let manifest_map = manifest_binary_desc_map(repo_root);
    let blocks = parse_cargo_test_blocks(log_text);
    let total_passed: u32 = blocks.iter().map(|b| b.passed).sum();
    let total_failed: u32 = blocks.iter().map(|b| b.failed).sum();
    let all_ok = total_failed == 0;

    let details: Vec<Value> = blocks
        .iter()
        .map(|b| {
            let desc = manifest_map.get(&b.name).cloned().unwrap_or_default();
            let desc_chars: Vec<char> = desc.chars().collect();
            let desc_short = if desc_chars.len() > 100 {
                format!("{}…", desc_chars.iter().take(80).collect::<String>())
            } else {
                desc
            };
            json!({
                "binary": b.name,
                "passed": b.passed,
                "failed": b.failed,
                "failures": b.failures,
                "desc": desc_short,
            })
        })
        .collect();

    let fail_count = blocks.iter().filter(|b| b.failed > 0).count();
    json!({
        "ok": all_ok,
        "total_passed": total_passed,
        "total_failed": total_failed,
        "binary_count": blocks.len(),
        "summary": if all_ok {
            format!("全部通过：{} 个测试 binary，共 {total_passed} 条断言。", blocks.len())
        } else {
            format!("有失败：{fail_count} 个 binary 共 {total_failed} 条失败。")
        },
        "details": details,
    })
}

/// manifest `id` → `desc` 映射（id = test binary 名，与 `Running tests/<id>.rs` 对齐）。
fn manifest_binary_desc_map(repo_root: &Path) -> std::collections::HashMap<String, String> {
    let mut m: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mf = repo_root.join("assets/data/test_manifest.json");
    if let Ok(text) = std::fs::read_to_string(&mf) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            for t in v.get("tests").and_then(Value::as_array).into_iter().flatten() {
                let tid = t.get("id").and_then(Value::as_str).unwrap_or("");
                let desc = t.get("desc").and_then(Value::as_str).unwrap_or("");
                if !tid.is_empty() {
                    m.insert(tid.to_owned(), desc.to_owned());
                }
            }
        }
    }
    m
}

/// 旧日志格式兼容回退：`parse_test_log` 使用的块结构。
/// 解析逻辑已收敛到 [`result_schema::parse_test_blocks`]（C1 单一事实源），此处只做类型转换。
struct TestBlock {
    name: String,
    passed: u32,
    failed: u32,
    failures: Vec<String>,
}

/// 分块解析委托给 C1 单一事实源（避免两份解析逻辑漂移）。
fn parse_cargo_test_blocks(log_text: &str) -> Vec<TestBlock> {
    result_schema::parse_test_blocks(log_text)
        .into_iter()
        .map(|b| TestBlock {
            name: b.name,
            passed: b.passed,
            failed: b.failed,
            failures: b.failures,
        })
        .collect()
}

/// 找外部落盘的 C1 `result.json`（W1/W3/W4 判据产物；S74.2 结构化结果优先源）。
///
/// 优先级 ① 状态字段 `result_json` 显式路径 ② `evidence[]` 中 `*result.json` 路径
/// ③ `prove` 前缀下 `<id>-<ts>/result.json`（取字典序最大 = 最新时间戳）。
fn find_result_json(repo_root: &Path, state: &Value) -> Option<String> {
    if let Some(p) = state.get("result_json").and_then(Value::as_str) {
        let path = repo_root.join(p);
        if path.is_file() {
            return std::fs::read_to_string(path).ok();
        }
    }
    if let Some(arr) = state.get("evidence").and_then(Value::as_array) {
        for v in arr {
            if let Some(p) = v.as_str().filter(|s| s.ends_with("result.json")) {
                if let Ok(text) = std::fs::read_to_string(repo_root.join(p)) {
                    return Some(text);
                }
            }
        }
    }
    let prove = state.get("prove").and_then(Value::as_str).unwrap_or("");
    let id = state.get("id").and_then(Value::as_str).unwrap_or("");
    if !prove.is_empty() && !id.is_empty() {
        let mut cands: Vec<(String, PathBuf)> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(repo_root.join(prove)) {
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(id) && entry.path().is_dir() {
                    let rj = entry.path().join("result.json");
                    if rj.is_file() {
                        cands.push((name, rj));
                    }
                }
            }
        }
        cands.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some((_, path)) = cands.last() {
            return std::fs::read_to_string(path).ok();
        }
    }
    None
}

/// C1 结构化结果 → `conclusion` JSON（保留旧字段 `ok/total_*/binary_count/summary/details`
/// 供既有前端与测试消费，新增 `source`/C1 形状字段为权威结论）。
fn structured_conclusion(tr: &result_schema::TestResult, source: &str, legacy: &Value) -> Value {
    let mut v = tr.to_value();
    if let Some(obj) = v.as_object_mut() {
        obj.insert("source".into(), json!(source));
        obj.insert("ok".into(), json!(tr.is_ok()));
        obj.insert("total_passed".into(), json!(tr.total_passed()));
        obj.insert("total_failed".into(), json!(tr.total_failed()));
        obj.insert("binary_count".into(), json!(tr.tests.len()));
        obj.insert(
            "summary".into(),
            json!(if tr.is_ok() {
                format!("全部通过：{} 个 binary，共 {} 条断言。", tr.tests.len(), tr.total_passed())
            } else {
                format!("有失败：共 {} 条失败。", tr.total_failed())
            }),
        );
        if let Some(details) = legacy.get("details") {
            obj.insert("details".into(), details.clone());
        }
    }
    v
}

/// 测试作业「自解释结论」（S74.2 结构化重做）：
///
/// ① 优先外部结构化 `result.json`（C1 契约产物，W1/W3/W4 判据输出同源）；
/// ② 回退读**完整日志**解析 C1 结构化结果（根治旧实现只读尾部 400 行导致长日志前段
///    binary 静默漏检——`gate_full`/`smoke` 多 binary 场景结论失真）；
/// ③ 兼容回退旧 `parse_test_log`（无 `Running` 块可解析的裸文本日志）。
///
/// 仅对已结束（`running=false`）的 test 作业有效；缺失/未完成 = `None`。
#[must_use]
pub fn test_run_conclusion(repo_root: &Path, kind: &str, id: &str) -> Option<Value> {
    let state = job_status(repo_root, kind, id)?;
    if state.get("running").and_then(Value::as_bool).unwrap_or(true) {
        return None;
    }
    let exit_code =
        i32::try_from(state.get("exit_code").and_then(Value::as_i64).unwrap_or(-1)).unwrap_or(-1);
    let started = state.get("started_ts").and_then(Value::as_u64).unwrap_or(0);
    let ended = state.get("ended_ts").and_then(Value::as_u64).unwrap_or(0);
    let duration_ms = ended.saturating_sub(started).saturating_mul(1000);
    let full = job_log_full(repo_root, kind, id);

    // ① 外部结构化结果（C1）——优先，W1/W3/W4 判据产物直接回显。
    if let Some(text) = find_result_json(repo_root, &state) {
        if let Some(tr) = result_schema::parse_result_json(&text) {
            let legacy = parse_test_log(full.as_deref().unwrap_or(""), repo_root);
            return Some(structured_conclusion(&tr, "result_json", &legacy));
        }
    }
    // ② 完整日志 → C1 结构化结果（截断修复主路径）。
    if let Some(text) = full.as_deref() {
        let tr = result_schema::parse_cargo_test_result(text, id, duration_ms, exit_code);
        if !tr.tests.is_empty() {
            let legacy = parse_test_log(text, repo_root);
            return Some(structured_conclusion(&tr, "full_log", &legacy));
        }
    }
    // ③ 旧日志格式兼容回退（尾部 400 行；仅对无结构可解析的裸日志）。
    let tail = job_log_tail(repo_root, kind, id, 400)?;
    Some(parse_test_log(&tail, repo_root))
}
