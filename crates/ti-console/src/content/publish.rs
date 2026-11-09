//! S44.4 打包/分发面（30 号 §2.3）：`POST /api/publish`（发布编排，默认关）+ `GET /api/publish`（只读状态）。
//!
//! 发布 = `publish_web.ps1`（S16.3 一条命令全链路：gate 前置、重建 web/、本地静态断言、
//! 发布历史入 `docs/release.md`、gh-pages 推流）。写面最大（`web/` + `docs/release.md` + gh-pages 远端），
//! 故单独默认关（30 号 §2.3：须 `enable_write` 显式开启，镜像 22 号 §4.2 默认启用列口径）。
//!
//! 两种模式：`check` = 只读前置体检（`publish_web.ps1 -Check`，同步，不构建/不推/不写 release.md，免 `enable_write`）；
//! `run` = 后台 spawn（`publish_web.ps1`，写 `web/` + `docs/release.md` + gh-pages），须 `enable_write`。
//!
//! 后台 `run` 的**非 0 退出上报**：收割线程等子进程退出 → 写 `publish-<ts>.status.json`
//! （`running=false` + `exit_code`），`GET /api/publish` 读回（镜像 `runner::gate` 的锁文件口径）。

use std::path::Path;

use serde_json::Value;

/// `POST /api/publish`（S44.4 发布面）：body `{"mode":"check"|"run","enable_write"?}`。
/// 鉴权（`403`）→ Content-Type application/json（`400`）→ mode 白名单（`400`）→ run 无 `enable_write`（`400`）。
/// `check` = 只读体检（同步，`exit_code` 直接回显）；`run` = 后台 spawn（退出码经 `GET /api/publish` 上报）。
pub(crate) fn handle_publish(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    // 写路由须 X-TI-Token（22 号 §10.1）；缺 token ⇒ 403（镜像 handle_wizard_step / handle_audit_taskbook）。
    if !crate::routes::auth_ok(token) {
        return crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"403: 写操作需 X-TI-Token（22 号 §10.1）"}"#.as_bytes(),
            "403 Forbidden",
            "no-store",
            None,
            want_gzip,
        );
    }
    // Content-Type 必须 application/json（22 号 §10.3；镜像 handle_gate/handle_audit_taskbook）。
    let ct_ok = content_type.is_some_and(|c| {
        let base = c.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        base == "application/json"
    });
    if !ct_ok {
        return crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"400: Content-Type 必须为 application/json（22 号 §10.3）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        );
    }
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            return crate::routes::status_json(
                400,
                &serde_json::json!({"ok": false, "error": format!("body 非合法 JSON：{e}")}),
                want_gzip,
            );
        }
    };
    let mode = payload.get("mode").and_then(Value::as_str).unwrap_or("").to_owned();
    if !matches!(mode.as_str(), "check" | "run") {
        return crate::routes::status_json(
            400,
            &serde_json::json!({"ok": false, "error": "mode 只接受 check|run（白名单；非自由字符串）"}),
            want_gzip,
        );
    }
    let enable_write = payload.get("enable_write").and_then(Value::as_bool).unwrap_or(false);
    // run 为写面（web/ + release.md + gh-pages），须 enable_write 显式开启（30 号 §2.3 默认关）。
    if mode == "run" && !enable_write {
        return crate::routes::status_json(
            400,
            &serde_json::json!({
                "ok": false,
                "error": "publish run 为写动作（web/ + docs/release.md + gh-pages），需 `enable_write=true`（30 号 §2.3 默认关）"
            }),
            want_gzip,
        );
    }
    match publish(repo_root, &mode) {
        Ok(v) => crate::routes::status_json(200, &v, want_gzip),
        Err((code, msg)) => crate::routes::status_json(
            code,
            &serde_json::json!({"ok": false, "error": msg}),
            want_gzip,
        ),
    }
}

/// 执行发布（`check` 同步 / `run` 后台 spawn）。解释器回退 pwsh → powershell（22 号 §4.3；Windows 仅 PS 5.1）。
/// 执行发布（`check` 同步只读体检 / `run` 后台 spawn）。
/// 解释器回退 pwsh → powershell（22 号 §4.3；Windows 仅 PS 5.1）——`check` 走同步路径自行回退，
/// `run` 走共用实现 `runner::spawn`（回退在其内部）。
fn publish(repo_root: &Path, mode: &str) -> Result<Value, (u16, String)> {
    if mode == "check" {
        return publish_check(repo_root);
    }
    publish_run(repo_root)
}

/// `check` 模式：同步跑 `publish_web.ps1 -Check`（只读：不构建/不推/不写 release.md），
/// 回显退出码 + 输出尾段（非 0 如实上报）。
fn publish_check(repo_root: &Path) -> Result<Value, (u16, String)> {
    let mut last_err = String::new();
    for interp in ["pwsh", "powershell"] {
        let mut cmd = std::process::Command::new(interp);
        cmd.arg("-File").arg("scripts/publish_web.ps1").arg("-Check");
        // 22 号 §10.3 最小环境集（env_clear + 白名单；pwsh 依赖 PATH/HOME）。
        crate::runner::apply_minimal_env(&mut cmd);
        cmd.current_dir(repo_root);
        // 只读体检：同步捕获输出，report exit + 尾部输出（非 0 退出如实上报）。
        let out = match cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                last_err = format!("{interp}: {e}");
                continue;
            }
        };
        let exit = out.status.code().unwrap_or(-1);
        let mut output = String::from_utf8_lossy(&out.stdout).into_owned();
        output.push_str(&String::from_utf8_lossy(&out.stderr));
        return Ok(serde_json::json!({
            "ok": exit == 0,
            "mode": "check",
            "exit_code": exit,
            "output": output.chars().take(2000).collect::<String>(),
            "note": "publish_web.ps1 -Check 只读前置体检（未构建/未推送/未写 release.md）",
        }));
    }
    Err((500u16, format!("发布体检 spawn 失败（pwsh/powershell 均缺位？）：{last_err}")))
}

/// `run` 模式：后台 spawn（共用实现 `runner::spawn`：解释器回退 + 日志落盘 + 收割线程记退出码），
/// 日志落 `evidence/console/logs/publish-<ts>.log`（gitignored；`GET /api/publish` 轮询状态）。
fn publish_run(repo_root: &Path) -> Result<Value, (u16, String)> {
    let logs = repo_root.join("evidence/console/logs");
    if let Err(e) = std::fs::create_dir_all(&logs) {
        return Err((500u16, format!("建 publish 日志目录失败：{e}")));
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let log_name = format!("publish-{ts}.log");
    let log_path = logs.join(&log_name);
    let status_path = logs.join(format!("publish-{ts}.status.json"));
    let status_meta = serde_json::json!({
        "kind": "publish",
        "id": "publish",
        "started_ts": ts,
        "log": format!("evidence/console/logs/{log_name}"),
    });
    // S49.7 统一作业模型（23 号 §6.2）：spawn + 收割走 `runner::jobs` 单一实现；
    // 发布保留自身状态文件（`publish-<ts>.status.json`，`GET /api/publish` 读回不变）
    // + 写统一作业镜像（`evidence/console/jobs/publish-publish.json`）⇒ `/api/jobs` 可查。
    let owned_args: Vec<String> = vec!["-File".to_owned(), "scripts/publish_web.ps1".to_owned()];
    let mirror = crate::runner::jobs::job_state_path(repo_root, "publish", "publish");
    let outcome = match crate::runner::jobs::spawn_program_logged(&crate::runner::jobs::SpawnOpts {
        candidates: crate::runner::spawn::SCRIPT_INTERPS.iter().map(|s| (*s).to_owned()).collect(),
        args: owned_args,
        cwd: repo_root.to_path_buf(),
        log_path: Some(log_path.clone()),
        state_path: status_path.clone(),
        status_meta,
        timeout_s: None,
        mirror_path: Some(mirror.clone()),
    }) {
        Ok(v) => v,
        Err(e) => return Err((500u16, e)),
    };
    Ok(serde_json::json!({
        "started": true,
        "pid": outcome.pid,
        "mode": "run",
        "interp": outcome.program,
        "log": format!("evidence/console/logs/{log_name}"),
        "status": format!("evidence/console/logs/publish-{ts}.status.json"),
        "note": "后台发布已启动（publish_web.ps1：gate 前置 + 重建 web + 静态断言 + release.md 发布历史 + gh-pages 推流）；退出码经 GET /api/publish 上报（非 0 = 硬失败）",
    }))
}

/// `GET /api/publish`（S44.4 发布面只读状态）：运行中标志 + 末次状态（含 `exit_code`）+ 日志历史 + 指定日志尾部。
/// 免 token（只读面，镜像 `GET /api/gate` 口径）。
#[must_use]
pub fn publish_status(repo_root: &Path, log_name: Option<String>) -> Value {
    let dir = repo_root.join("evidence/console/logs");
    // 末次状态 = mtime 最新的 `publish-*.status.json`。
    let mut statuses: Vec<(u64, Value)> = Vec::new();
    let mut history: Vec<Value> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let mtime = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            if name.starts_with("publish-") && name.to_ascii_lowercase().ends_with(".status.json") {
                if let Ok(raw) = std::fs::read_to_string(e.path()) {
                    if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                        statuses.push((mtime, v));
                    }
                }
                continue;
            }
            if name.starts_with("publish-") && name.to_ascii_lowercase().ends_with(".log") {
                let size = e.metadata().map_or(0, |m| m.len());
                history.push(serde_json::json!({"name": name, "size": size, "mtime": mtime}));
            }
        }
    }
    statuses.sort_by_key(|(mtime, _)| *mtime);
    let last = statuses.last().map_or(Value::Null, |(_, v)| v.clone());
    let running = last.get("running").and_then(Value::as_bool).unwrap_or(false);
    history.sort_by_key(|a| std::cmp::Reverse(a.get("mtime").and_then(Value::as_u64).unwrap_or(0)));

    // 指定日志尾部（?log=<name>；白名单形状校验 + 防穿越（拒 `..`/`/`/`\`），最多 400 行）。
    let tail = log_name.and_then(|name| {
        if !name.starts_with("publish-")
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

    serde_json::json!({
        "running": running,
        "last": last,
        "history": history,
        "tail": tail,
        "note": "publish 发布面：check = 只读体检（同步回显 exit_code）；run = 后台 spawn（退出码经本端点上报，非 0 = 硬失败；日志落 evidence/console/logs/publish-<ts>.log）",
    })
}
