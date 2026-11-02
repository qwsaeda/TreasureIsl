//! 只读 API 的响应组装（`/api/overview` `/api/tasks` `/api/evidence` `/api/assets` `/api/tests`
//! `/api/session`）。
//!
//! 数据源路径全部来自 [`super::config::RepoSources`]（`console_sources.json`，
//! 22 号 §5 单一事实源）；路径访问全过 [`super::guard::resolve_guarded`]（canonicalize 防穿越）；
//! 本模块 **0 写盘**（只读计算；快照对比报告落盘由调用方写 `evidence/console/`）。
//!
//! 白名单口径（22 号 §10.2「只暴露 `docs/`、`evidence/`、`plan/`、`assets/`、`dist/maps/`」）：
//! 白名单面 = **数据驱动**（`console_sources.json` 的 `repo_sources` 混合白名单：目录项前缀 + 文件项精确，
//! 22 号 §10.2「assets/ 只返回登记过的文件」= 文件项精确放行，0 目录级常量化）。

use std::path::Path;

use serde_json::{Value, json};

use super::config::RepoSources;
use super::guard::{check_relative, resolve_guarded};

/// `/api/overview`：首屏关键信息（22 号 §8「< 2 s」判据面）。
///
/// 返回 = 台账任务数 / verified 数 / 下一任务（首个非 verified 且 deps 全 verified）/
/// 已装地图包数 / UI 资产数（`ui_manifest.assets` 长度）/ 数据源可达性。
///
/// # Errors
///
/// 台账/数据源读取失败 = 带原因 `Err`（首屏 <2 s 判据面，失败不静默吞）。
pub fn overview(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    let ledger_path = resolve_guarded(repo_root, &sources.repo_sources, "plan/ledger.yaml")
        .map_err(|e| e.to_string())?;
    let ledger_text = std::fs::read_to_string(&ledger_path)
        .map_err(|e| format!("读 plan/ledger.yaml 失败：{e}"))?;
    let report = ti_devmeta::ledger::validate(&ledger_text);
    let next = next_task_id(&report);
    let maps = installed_map_packs(repo_root);
    let ui = ui_asset_count(repo_root);
    let source_count = sources.repo_sources.len();
    Ok(json!({
        "ledger": {
            "total_tasks": report.tasks.len(),
            "verified": report.verified_count(),
            "next_task": next,
            "errors": report.errors.len(),
        },
        "maps": { "installed": maps },
        "ui_assets": { "count": ui },
        "sources": { "repo_sources": source_count },
    }))
}

/// 「下一任务」= 台账中第一个非 verified 且全部 deps 已 verified 的任务 id（无 = `None`）。
#[must_use]
pub fn next_task_id(report: &ti_devmeta::ledger::Report) -> Option<String> {
    let verified: std::collections::HashSet<&str> =
        report.tasks.iter().filter(|t| t.is_verified()).map(|t| t.id.as_str()).collect();
    for task in &report.tasks {
        if task.is_verified() {
            continue;
        }
        let deps_ok = task.deps.iter().all(|d| verified.contains(d.as_str()));
        if deps_ok {
            return Some(task.id.clone());
        }
    }
    None
}

/// `/api/tasks`：台账全表（`id/title/status/deps/rounds/evidence`；dod 截断 200 字符防体积）。
///
/// # Errors
///
/// 台账读取/解析失败 = 带原因 `Err`。
pub fn tasks(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    let ledger_path = resolve_guarded(repo_root, &sources.repo_sources, "plan/ledger.yaml")
        .map_err(|e| e.to_string())?;
    let text = std::fs::read_to_string(&ledger_path)
        .map_err(|e| format!("读 {} 失败：{e}", ledger_path.display()))?;
    let report = ti_devmeta::ledger::validate(&text);
    let rows: Vec<Value> = report
        .tasks
        .iter()
        .map(|t| {
            let dod = if t.dod.len() > 200 {
                let cut: String = t.dod.chars().take(200).collect();
                format!("{cut}…")
            } else {
                t.dod.clone()
            };
            json!({
                "id": t.id,
                "title": t.title,
                "status": status_str(t),
                "deps": t.deps,
                "rounds": t.rounds,
                "evidence": t.evidence,
                "dod_excerpt": dod,
            })
        })
        .collect();
    let _ = sources;
    Ok(json!({ "tasks": rows, "count": rows.len() }))
}

/// `/api/evidence`：`evidence/` 文件名清单（含子目录名；只读；图片预览 = S32.3 消费面）。
/// `sub` 参数（如 `ui_concept`）= 枚举该子目录文件（带 `evidence/{sub}/` 前缀；仍走
/// `resolve_guarded` 白名单 + `..` 段拒，0 越界；`ui_concept/` 在 `evidence/` 目录项前缀内）。
/// # Errors
///
/// 白名单拒绝（`..` 段 / 非登记路径）或目录读取失败 = 带原因 `Err`。
pub fn evidence_list(repo_root: &Path, sources: &RepoSources, sub: &str) -> Result<Value, String> {
    let rel = if sub.is_empty() { "evidence/".to_owned() } else { format!("evidence/{sub}/") };
    check_relative(&rel, &sources.repo_sources).map_err(|e| e.reason)?;
    let root =
        resolve_guarded(repo_root, &sources.repo_sources, &rel).map_err(|e| e.to_string())?;
    let prefix = if sub.is_empty() { String::new() } else { format!("evidence/{sub}/") };
    let mut files: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // 缩略图侧车（`ui_thumb.py` 生成物，非设计稿本体）不进清单：前端卡/预览
            // 单独按 `.thumb.png` 命名约定直读（22 号 §6.1 设计图页「压缩预览」面）。
            if entry.path().is_file() && name.ends_with(".thumb.png") {
                continue;
            }
            let entry_name = if entry.path().is_dir() { format!("{name}/") } else { name };
            files.push(format!("{prefix}{entry_name}"));
        }
    }
    files.sort();
    Ok(json!({ "dir": rel, "files": files, "count": files.len() }))
}

/// `/api/assets`：`ui_manifest` + `registry.json` + `gate_data.json` +
/// 已装地图包清单（`assets/maps/<id>/manifest.json` 存在且 ≠ `_template`）+ `.default` 指针 +
/// `dist/maps/` 可分发件（gitignored，缺失不报错）。
///
/// # Errors
///
/// 数据源白名单解析失败 = 带原因 `Err`（缺文件 = 0 计数面不 Err）。
pub fn assets(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    // 3 个 JSON 数据源路径 = console_sources.json（单一事实源，22 号 §5）里
    // 登记的文件项（数据驱动，Rust 侧 0 字面量 ⇒ CS-1 0 命中）。
    let ui_path = pick_source(sources, "ui_manifest")?;
    let registry_path = pick_source(sources, "registry")?;
    let gate_data_path = pick_source(sources, "gate_data")?;
    let ui = read_json(repo_root, &sources.repo_sources, &ui_path)?;
    let registry = read_json(repo_root, &sources.repo_sources, &registry_path)?;
    let gate_data = read_json(repo_root, &sources.repo_sources, &gate_data_path)?;
    let maps = map_pack_info(repo_root);
    let default_pack = std::fs::read_to_string(repo_root.join("assets/maps/.default"))
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let dist = dist_pack_names(repo_root);
    let _ = sources;
    Ok(json!({
        "ui_manifest": ui,
        "registry": registry,
        "gate_data": gate_data,
        "map_packs": maps,
        "default_map_pack": default_pack,
        "dist": dist,
    }))
}

/// `/api/tests`：游戏测试标杆机读清单（`23_game_testing.md` §7）+ evidence 存在性对账。
///
/// 返回 = `assets/data/test_manifest.json` 的 `tests[]`（白名单数据源，路径过
/// `resolve_guarded`）逐条附 `evidence_exists`（`evidence/` 下路径存在与否，供前端标记
/// 断链）；顶层 = 按 级(L1..L4)→类→型 的分组树（`by_level`），供「测试」页左树。
///
/// # Errors
///
/// 白名单路径缺失/不可读 = 带原因 `Err`（平台数据源面如实上报，不静默）。
pub fn tests(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    let manifest_path = pick_source(sources, "test_manifest")?;
    let manifest = read_json(repo_root, &sources.repo_sources, &manifest_path)?;
    let list = manifest.get("tests").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut items: Vec<Value> = Vec::new();
    for t in &list {
        let ev = t.get("evidence").and_then(Value::as_str).unwrap_or("");
        let exists = !ev.is_empty() && repo_root.join(ev).exists();
        let mut item = t.clone();
        if let Some(obj) = item.as_object_mut() {
            obj.insert("evidence_exists".into(), json!(exists));
        }
        items.push(item);
    }
    // 按 级→类→型 分组（树序固定：L1..L4 / 6 类 / 5 型，缺省组不产出）。
    let levels = ["L1", "L2", "L3", "L4"];
    let cats = ["functional", "regression", "visual", "audio", "performance", "compatibility"];
    let types = ["smoke", "ci", "playtest", "exploratory", "benchmark"];
    let mut by_level: Vec<Value> = Vec::new();
    for lv in levels {
        let lv_items: Vec<&Value> = items.iter().filter(|i| i["level"] == lv).collect();
        if lv_items.is_empty() {
            continue;
        }
        let mut cats_out: Vec<Value> = Vec::new();
        for cat in cats {
            let cat_items: Vec<&Value> =
                lv_items.iter().copied().filter(|i| i["category"] == cat).collect();
            if cat_items.is_empty() {
                continue;
            }
            let mut types_out: Vec<Value> = Vec::new();
            for ty in types {
                let ty_items: Vec<&Value> =
                    cat_items.iter().copied().filter(|i| i["type"] == ty).collect();
                if ty_items.is_empty() {
                    continue;
                }
                types_out.push(json!({ "type": ty, "tests": ty_items }));
            }
            cats_out.push(json!({ "category": cat, "types": types_out }));
        }
        by_level.push(json!({ "level": lv, "categories": cats_out }));
    }
    Ok(json!({
        "manifest": manifest_path,
        "count": items.len(),
        "tests": items,
        "by_level": by_level,
    }))
}

/// `/api/tests?run=<id>`：后台运行 `test_manifest` 登记的一个测试命令（23 号 §6 平台集成「能实际运行」）。
///
/// 安全口径（22 号 §10.3 白名单）：`<id>` 必须是 `test_manifest.json` 登记的 `tests[].id`（枚举，
/// 非自由字符串）；命令从该 test 的 `command` 字段取，**逐参数** `Command::new` 执行（禁 `sh -c`/
/// `cmd /c` 字符串拼接，22 号禁止 2）；`cargo`/`python3`/`pwsh`/`powershell` 之外的程序 = 拒绝。
/// 后台 spawn（`Stdio::null`），日志落 `evidence/console/logs/test-<id>-<ts>.log`（gitignored）。
/// 轮询 `/api/tests` 或 `/api/build`（logs 面）查结果。
///
/// # Errors
///
/// id 未登记 / 命令程序不在白名单 / spawn 失败 = 带原因 `Err`。
pub fn run_test(repo_root: &Path, sources: &RepoSources, test_id: &str) -> Result<Value, String> {
    let manifest_path = pick_source(sources, "test_manifest")?;
    let manifest = read_json(repo_root, &sources.repo_sources, &manifest_path)?;
    let list = manifest.get("tests").and_then(Value::as_array).cloned().unwrap_or_default();
    let test = list
        .iter()
        .find(|t| t.get("id").and_then(Value::as_str) == Some(test_id))
        .ok_or_else(|| {
            format!("run_test: id `{test_id}` 未在 test_manifest.json 登记（23 号 §7；枚举 id）")
        })?;
    let command = test.get("command").and_then(Value::as_str).unwrap_or("");
    if command.is_empty() {
        return Err(format!(
            "run_test: `{test_id}` 无 command 字段（不可自动运行，如人工视觉验收）"
        ));
    }
    // 逐参数拆分（空格；非 shell——命令串来自 test_manifest 登记，无引号嵌套）
    let args: Vec<&str> = command.split_whitespace().collect();
    if args.is_empty() {
        return Err(format!("run_test: `{test_id}` command 为空"));
    }
    let program = args[0];
    // 程序白名单（22 号 §10.3）：cargo / python3 / pwsh / powershell；其余（含 gate.ps1 直跑）= 拒。
    if !matches!(program, "cargo" | "python3" | "pwsh" | "powershell") {
        return Err(format!(
            "run_test: `{test_id}` 程序 `{program}` 不在白名单（cargo/python3/pwsh/powershell；22 号 §10.3）"
        ));
    }
    // 日志落 evidence/console/logs/（gitignored；与 build 同目录面）
    let logs_dir = repo_root.join("evidence/console/logs");
    std::fs::create_dir_all(&logs_dir).map_err(|e| format!("run_test: logs 目录创建失败 {e}"))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let log_name = format!("test-{test_id}-{ts}.log");
    let log_path = logs_dir.join(&log_name);
    let log_file =
        std::fs::File::create(&log_path).map_err(|e| format!("run_test: 日志文件创建失败 {e}"))?;
    let err_file = log_file.try_clone().map_err(|e| format!("run_test: 日志克隆失败 {e}"))?;
    let mut cmd = std::process::Command::new(program);
    cmd.args(&args[1..]).current_dir(repo_root);
    // 22 号 §10.3 最小环境集（env_clear + 白名单）；r4-b P2 修复。
    crate::runner::apply_minimal_env(&mut cmd);
    let child = cmd
        .stdout(std::process::Stdio::from(log_file))
        .stderr(std::process::Stdio::from(err_file))
        .spawn()
        .map_err(|e| format!("run_test: 后台 spawn 失败（{program} 缺位？）：{e}"))?;
    Ok(json!({
        "started": true,
        "pid": child.id(),
        "id": test_id,
        "command": command,
        "log": format!("evidence/console/logs/{log_name}"),
        "note": "后台测试已启动（日志落 evidence/console/logs/test-<id>-<ts>.log）；轮询 /api/build 的 logs 面查结果",
    }))
}

/// `/api/build`：构建产物清单 + 构建编排状态（S35.1，24 号 §4.1；只读）。
///
/// 口径：`scripts/build_artifacts.py list`（已构建产物 体积/mtime）+ 最近一次构建日志
/// （`evidence/console/logs/build-*.log`，gitignored；平台不执行构建，构建触发走
/// `console_commands.json` 白名单 id `build_apk`/`build_wasm`/`build_native` 后台执行）。
///
/// # Errors
///
/// `build_artifacts.py` 执行失败（`python3` 缺位）= `Err`（带原因，数据源面如实上报）。
pub fn build_artifacts(
    repo_root: &Path,
    _sources: &RepoSources,
    run_kind: Option<&str>,
) -> Result<Value, String> {
    // ?run=<kind> = 后台 spawn 构建编排器（S35.1；kind 白名单 apk|wasm|native，自由字符串 = 400）
    if let Some(kind) = run_kind {
        if !matches!(kind, "apk" | "wasm" | "native") {
            return Err(format!(
                "run 参数只接受 apk|wasm|native（22 号 §10.3 白名单；非自由字符串）：{kind}"
            ));
        }
        let mut cmd = std::process::Command::new("python3");
        cmd.arg(repo_root.join("scripts/build_artifacts.py")).arg("run").arg(kind);
        // 22 号 §10.3 最小环境集（env_clear + 白名单）；r4-b P2 修复。
        crate::runner::apply_minimal_env(&mut cmd);
        let child = cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("后台构建 spawn 失败（python3 缺位？）：{e}"))?;
        return Ok(json!({
            "started": true,
            "pid": child.id(),
            "kind": kind,
            "note": "后台构建已启动（日志落 evidence/console/logs/build-*.log；构建后自动复检）；轮询 /api/build 查产物",
        }));
    }
    let mut list_cmd = std::process::Command::new("python3");
    list_cmd.arg(repo_root.join("scripts/build_artifacts.py")).arg("list");
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
    let list_body = String::from_utf8_lossy(&out.stdout).into_owned();
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
    Ok(json!({
        "list": list_body,
        "last_build_log": last_log,
        "tool": "scripts/build_artifacts.py（list 只读；触发构建 = console_commands build_{apk,wasm,native}）",
    }))
}

/// `/api/apk-size`：APK 体积构成体检（S35.0，24 号标准；只读，平台不执行构建）。
///
/// 口径：`scripts/apk_audit.py --apk <最新 dist/TreasureIsl-arm64.apk>`（判据①② = .so 压缩率
/// + 未引用 UI 漂移）+ `--refset`（引用集组成）。dist/ 无 APK = 「未构建」降级（不报错，24 号 §4）。
///
/// # Errors
///
/// `apk_audit.py` 执行失败（`python3` 缺位 / 输出解析失败）= `Err`（带原因，数据源面如实上报）。
pub fn apk_size(repo_root: &Path, _sources: &RepoSources) -> Result<Value, String> {
    let dist_dir = repo_root.join("dist");
    let dist = dist_dir.join("TreasureIsl-arm64.apk");
    if !dist.is_file() {
        return Ok(json!({
            "built": false,
            "note": "dist/TreasureIsl-arm64.apk 不存在（未构建；先跑 scripts/build_apk.sh）",
        }));
    }
    let mut apk_cmd = std::process::Command::new("python3");
    apk_cmd.arg(repo_root.join("scripts/apk_audit.py")).arg("--apk").arg(&dist).arg("--json");
    // 22 号 §10.3 最小环境集（env_clear + 白名单）；r6-b P2 修复。
    crate::runner::apply_minimal_env(&mut apk_cmd);
    let out = apk_cmd.output();
    let out = match out {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            // 判据 FAIL（exit 3/4）= 体积口径违规，如实上报（平台显示 FAIL 灯，非 500）
            let err = String::from_utf8_lossy(&o.stderr).trim().to_owned();
            return Ok(json!({
                "built": true,
                "apk": dist.file_name().unwrap_or_default().to_str().unwrap_or_default(),
                "verdict": "FAIL",
                "fail_detail": if err.is_empty() { "apk_audit.py 非 0 退出".into() } else { err },
                "note": "24 号判据 FAIL（.so 未压缩 或 未引用资产进包）；先修构建口径再发",
            }));
        }
        Err(e) => return Err(format!("apk_audit.py 执行失败（python3 缺位？）：{e}")),
    };
    let body = String::from_utf8_lossy(&out.stdout);
    let breakdown: Value =
        serde_json::from_str(&body).map_err(|e| format!("apk_audit.py 输出解析失败：{e}"))?;
    let mut refset_cmd = std::process::Command::new("python3");
    refset_cmd.arg(repo_root.join("scripts/apk_audit.py")).arg("--refset");
    // 22 号 §10.3 最小环境集（env_clear + 白名单）；r6-b P2 修复。
    crate::runner::apply_minimal_env(&mut refset_cmd);
    let refset_out = refset_cmd.output();
    let refset: Value = match refset_out {
        Ok(o) if o.status.success() => {
            serde_json::from_slice::<Value>(&o.stdout).unwrap_or(Value::Null)
        }
        _ => Value::Null,
    };
    let lib = breakdown.get("lib").cloned().unwrap_or(Value::Null);
    let file_size = lib.get("file_size").and_then(Value::as_u64).unwrap_or(0);
    let compress_size = lib.get("compress_size").and_then(Value::as_u64).unwrap_or(0);
    // 判据①（24 号标准）：.so 必须 ZIP_DEFLATED。deflate-9 对 .so 实测 ratio≈0.38，
    // Stored = 1.0；阈值 0.90 = 安全分界（压缩后必 <0.5，未压缩必 =1.0）。
    // 整数比较避免 u64→f64 精度损失（clippy cast_precision_loss）：compress/file < 0.90
    // ⇔ compress * 10 < file * 9（两侧为正整数，无溢出风险——.so 数 MB 量级）。
    let so_compressed: bool =
        file_size > 0 && compress_size.saturating_mul(10) < file_size.saturating_mul(9);
    // 显示比率（仅 JSON 呈现，不参与判据①阈值比较——比较已在上面用整数完成；
    // u64→f64 仅用于展示，codebase 既有模式 = ti-render allow cast_precision_loss）。
    #[allow(clippy::cast_precision_loss)]
    let ratio: f64 = if file_size > 0 { compress_size as f64 / file_size as f64 } else { 0.0 };
    let verdict: &str = if so_compressed { "PASS" } else { "FAIL" };
    Ok(json!({
        "built": true,
        "apk": "dist/TreasureIsl-arm64.apk",
        "verdict": verdict,
        "so_ratio": ratio,
        "breakdown": breakdown,
        "refset": refset,
        "tool": "scripts/apk_audit.py（--apk 判据①② / --refset 引用集；24 号 §2）",
    }))
}

/// `/api/snapshot`：22 号 §8「0 游戏产物变更」内容级指纹快照（S32.2 载体消费点；
/// 只读计算：冻结路径逐文件内容指纹 + 未跟踪清单；差值报告落盘 = 调用方写 `evidence/console/`）。
#[must_use]
pub fn snapshot(repo_root: &Path, sources: &RepoSources) -> Value {
    let snap = super::snapshot::FrozenSnapshot::take(repo_root);
    let kind = super::snapshot::fingerprint_kind();
    // 差值面：与快照报告落盘面（evidence/console/）配套 = 动作前后各取一次，
    // `diff_report` 由调用方组装落盘；此处返回当前快照 + 差值助手单测口径。
    let baseline = super::snapshot::snapshot_from(std::collections::BTreeMap::new());
    let diff_entries = baseline.diff(&snap).len();
    let _ = super::snapshot::diff_report(&baseline, &snap);
    let _ = super::snapshot::no_change_probe(&baseline, &snap);
    let _ = sources;
    json!({
        "kind": format!("{kind}（22 号 §8；禁用 git status --porcelain）"),
        "frozen_paths": super::snapshot::FROZEN_PATHS,
        "file_count": snap.fingerprint_count(),
        "untracked_count": snap.untracked.len(),
        "diff_vs_empty_baseline": diff_entries,
        "fingerprint_sample": snapshot_sample(&snap),
    })
}

/// 指纹抽样面（防响应体积：前 8 条 + 总计数；全量 = 快照报告落盘面）。
fn snapshot_sample(snap: &super::snapshot::FrozenSnapshot) -> Vec<Value> {
    snap.files.iter().take(8).map(|(k, v)| json!({ "path": k, "fingerprint": v })).collect()
}

/// `/api/session`：平台会话信息（与 S32.3 设置页 token 粘贴 + 写操作开关配套；
/// 只读：不读 token 值，只报「当前 token 是否存在于环境」+ 写操作开关的默认态 = 关）。
///
/// `/api/audit`：双审计报告清单（`evidence/console/audit-*.md` 文件名 + 抬头 §7.3
/// 四行机器可解析字段〔总判定/新发现/误报/残留〕的解析；只读，0 写盘）。
///
/// 文件正文读取走 `resolve_guarded`（`evidence/` 目录项前缀白名单）；
/// 标题面 = 文件名模式 `audit-<task>-r<N>-<ab>.md`（task 段允许点分，如 `S32.0a`）。
/// # Errors
///
/// 目录读取失败 = 带原因 `Err`（0 报告时返回空清单不 Err）。
/// `/api/audit`：双审计报告清单 + 收敛判定（22 号 §7.3/§7.4；S32.5 起机械判定）。
///
/// - 报告清单：`evidence/console/audit-*.md`（**排除** `.prompt.md` 任务书）；
///   每份含 §7.3 四行机器字段（`parseable` 区分 parsed/legacy/unparseable）；
/// - 收敛表（按 task 分组）：轮次完整性（1..=max 连续 + 每轮 a/b 在盘）+ 收敛状态
///   （`pass` = 连续 2 轮全 0 / `in_progress` / `escalate` = 连续 3 轮未收敛）+ 未闭环清单。
///
/// 目录读取失败 = 带原因 `Err`（0 报告时返回空清单不 Err）。
pub fn audit_list(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    use crate::audit::judge::{HeadStatus, ReportMeta, parse_head_status, parse_report_filename};
    let base = resolve_guarded(repo_root, &sources.repo_sources, "evidence/console/")
        .map_err(|e| e.to_string())?;
    let mut reports: Vec<Value> = Vec::new();
    let mut metas: Vec<ReportMeta> = Vec::new();
    let Ok(rd) = std::fs::read_dir(&base) else {
        return Ok(json!({ "reports": reports, "count": 0, "base": "evidence/console/" }));
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("audit-") || !name.to_lowercase().ends_with(".md") {
            continue;
        }
        // 任务书（.prompt.md）非报告，排除（防「不可解析」误入报告清单）。
        if name.ends_with(".prompt.md") {
            continue;
        }
        let Some((task, round, side)) = parse_report_filename(&name) else {
            continue;
        };
        let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
        // 祖父条款：r1~r3（契约引入前）人工口径读，不因缺四行判「不可解析」误报。
        let legacy = round <= 3;
        let status = parse_head_status(&text, legacy);
        let (verdict, new, false_pos, residual, st) = match &status {
            HeadStatus::Parsed(f) => {
                (f.verdict.clone(), f.found, f.false_pos, f.residual, "parsed")
            }
            HeadStatus::Legacy => (None, None, None, None, "legacy"),
            HeadStatus::Unparseable => (None, None, None, None, "unparseable"),
        };
        reports.push(json!({
            "file": format!("evidence/console/{name}"),
            "task": task.clone(),
            "round": round,
            "side": side.to_string(),
            "verdict": verdict,
            "new": new,
            "false_pos": false_pos,
            "residual": residual,
            "parseable": matches!(status, HeadStatus::Parsed(_)),
            "status": st,
        }));
        metas.push(ReportMeta {
            task,
            round,
            side,
            file: format!("evidence/console/{name}"),
            status,
        });
    }
    reports.sort_by(|a, b| {
        let ak = a.get("file").and_then(Value::as_str).unwrap_or("");
        let bk = b.get("file").and_then(Value::as_str).unwrap_or("");
        ak.cmp(bk)
    });
    // 按 task 分组 → 收敛表 + 未闭环清单（22 号 §7.4 机械判定）。
    let conv = convergence_rows(metas);
    Ok(json!({
        "reports": reports,
        "count": reports.len(),
        "convergence": conv,
        "base": "evidence/console/",
    }))
}

/// 收敛表 + 未闭环清单（22 号 §7.4 机械判定；按 task 分组）。
/// `metas` = 该仓库全部审计报告元数据（S32.5 起）。
fn convergence_rows(metas: Vec<crate::audit::judge::ReportMeta>) -> Vec<Value> {
    use crate::audit::judge::{
        ReportMeta, aggregate, convergence, round_complete, unclosed_findings,
    };
    let mut by_task: std::collections::BTreeMap<String, Vec<ReportMeta>> =
        std::collections::BTreeMap::new();
    for m in metas {
        by_task.entry(m.task.clone()).or_default().push(m);
    }
    let mut conv: Vec<Value> = Vec::new();
    for (task, mut ms) in by_task {
        ms.sort_by_key(|m| (m.round, m.side));
        let rounds = aggregate(&ms);
        let (complete, missing) = round_complete(&rounds);
        let (state, detail) = match convergence(&rounds) {
            crate::audit::judge::ConvergeResult::Pass { consecutive } => {
                ("pass", json!(consecutive))
            }
            crate::audit::judge::ConvergeResult::InProgress { consecutive } => {
                ("in_progress", json!(consecutive))
            }
            crate::audit::judge::ConvergeResult::Escalate { unclosed } => {
                ("escalate", json!(unclosed))
            }
        };
        let unclosed: Vec<Value> = unclosed_findings(&rounds)
            .into_iter()
            .map(|(side, new, res, file)| {
                json!({ "side": side.to_string(), "new": new, "residual": res, "file": file })
            })
            .collect();
        conv.push(json!({
            "task": task,
            "rounds": rounds.iter().map(|r| json!({
                "round": r.round,
                "a_file": r.a_file,
                "b_file": r.b_file,
                "complete": r.complete(),
                "converged": r.converged(),
                "legacy": r.legacy(),
                "unparseable": r.unparseable(),
            })).collect::<Vec<_>>(),
            "complete": complete,
            "missing_rounds": missing,
            "state": state,
            "detail": detail,
            "unclosed": unclosed,
        }));
    }
    conv
}

/// `/api/map`：地图包清单 + 描述符浏览（21 号地图包标准；只读，0 写盘）。
///
/// - 数据源 = `assets/maps/<id>/`（`assets/maps/` 白名单目录项前缀放行）
/// - 返回 = 已装包列表（`installed_packs()` 口径：有 `manifest.json` 且 ≠ `_template`）
///   加每包描述符摘要（31 格不变量、环几何、资产数等）
/// - 新建地图/换图 = 走 `scripts/map_pack.py` 工具链（`add`/`pack`/`install`，
///   21 号 §8），平台本端点**只读**，0 制作写操作
/// # Errors
///
/// `assets/maps/` 白名单解析失败 = 带原因 `Err`（目录不存在 = 空清单不 Err）。
pub fn map_packs_detail(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    // `assets/maps/` = 白名单目录项（22 号 §10.2）；root 走 `resolve_guarded` 防穿越，
    // 逐包 `manifest.json` 在其下相对读（0 越出 `assets/maps/` 前缀）。
    let maps_dir = resolve_guarded(repo_root, &sources.repo_sources, "assets/maps/")
        .map_err(|e| e.to_string())?;
    let mut packs: Vec<Value> = Vec::new();
    let Ok(rd) = std::fs::read_dir(&maps_dir) else {
        return Ok(json!({ "packs": packs, "count": 0, "tool": "scripts/map_pack.py" }));
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.path().is_dir() || name == "_template" {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        let Some(manifest) = read_map_json(&manifest_path) else {
            // 无 manifest.json = 在途空目录骨架（如实标注不崩）。
            packs.push(json!({
                "id": name,
                "manifest_present": false,
                "valid": false,
                "pack_dir": format!("assets/maps/{name}"),
                "note": "空骨架（在途，无 manifest.json；填 map.json 后先 map_pack.py sync）",
            }));
            continue;
        };
        // 草稿态判定：有 manifest 但无 `map` 键 = 在途草稿（m04 S16.4 面，
        // 口径 = `map_pack.py installed_packs()` 的 manifest 在位 + 有效集 =
        // `verify_pack` 0 违规；平台如实标注 valid=false + 原因，不崩）。
        let has_map = manifest.get("map").is_some();
        let ring = manifest.get("layout").and_then(|l| l.get("ring"));
        let spaces =
            manifest.get("map").and_then(|m| m.get("spaces")).and_then(serde_json::Value::as_array);
        let assets_n =
            manifest.get("assets").and_then(serde_json::Value::as_array).map_or(0, Vec::len);
        let invariants = manifest.get("invariants").cloned().unwrap_or(serde_json::Value::Null);
        packs.push(json!({
            "id": name,
            "manifest_present": true,
            "valid": has_map,
            "draft_note": if has_map { Value::Null } else {
                json!("草稿态（manifest 在位但无 map 键；verify 必有违规 = 未过 I1~I8，填 31 格后 verify 才进有效集）")
            },
            "pack_dir": format!("assets/maps/{name}"),
            "schema": manifest.get("schema").cloned().unwrap_or(serde_json::Value::Null),
            "version": manifest.get("version").cloned().unwrap_or(serde_json::Value::Null),
            "data_version": manifest.get("data_version").cloned().unwrap_or(serde_json::Value::Null),
            "central_asset": manifest.get("central_asset").cloned().unwrap_or(serde_json::Value::Null),
            "isle_assets": manifest.get("isle_assets").and_then(serde_json::Value::as_array).map_or(0, Vec::len),
            "spaces": spaces.map_or(0, Vec::len),
            "ring": ring.cloned().unwrap_or(serde_json::Value::Null),
            "assets_count": assets_n,
            "invariants": invariants,
        }));
    }
    packs.sort_by(|a, b| {
        a.get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .cmp(b.get("id").and_then(serde_json::Value::as_str).unwrap_or(""))
    });
    Ok(json!({
        "packs": packs,
        "count": packs.len(),
        "default": default_map_pack(repo_root, sources),
        "tool": "scripts/map_pack.py（add 新建 / pack 打包 / install 装上 / verify 校验 / check 漂移对账，21 号 §8；平台本端点只读）",
        "spec": "docs/process/21_map_package_spec.md（地图包标准 v1 单一事实源）",
    }))
}

/// 读地图包 `manifest.json`（缺文件/非法 JSON = `None`，空骨架不崩）。
fn read_map_json(path: &std::path::Path) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// `.default` 指针（缺 = 无默认包，不报错；白名单精确项 `assets/maps/.default`）。
fn default_map_pack(repo_root: &Path, sources: &RepoSources) -> Option<String> {
    let path = resolve_guarded(repo_root, &sources.repo_sources, "assets/maps/.default").ok()?;
    std::fs::read_to_string(&path).ok().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}

/// `/api/whitelist?path=<relative>`：白名单判定 API（22 号 §10.2 混合白名单的机器面；
/// 只读：`in_sources` 纯计算 + 参数白名单校验，0 写盘）。
#[must_use]
pub fn whitelist_probe(repo_root: &Path, relative: &str) -> Value {
    let sources = crate::repo::config::load(repo_root).unwrap_or_default();
    json!({
        "path": relative,
        "in_whitelist": in_sources(relative, &sources),
    })
}

/// `/api/session`：平台会话信息（只读；token 存在性 + 写操作默认态，22 号 §10.1）。
#[must_use]
pub fn session() -> Value {
    let token_present = std::env::var("TI_CONSOLE_TOKEN").is_ok();
    json!({
        "token_present": token_present,
        "write_enabled_default": false,
        "single_instance_lock": "evidence/console/.lock（S32.4 完整规格：起始时间 + 动作 id + 30 分钟 stale 接管）",
    })
}

/// 白名单判定：`relative` 是否落在 `RepoSources.repo_sources`（混合白名单）内
/// （目录项前缀 + 文件项精确，22 号 §10.2）。
#[must_use]
/// 路径是否落在仓库白名单源目录内（S32 平台 API 判据；单测与 API 共用）。
pub fn in_sources(relative: &str, sources: &RepoSources) -> bool {
    super::guard::check_relative(relative, &sources.repo_sources).is_ok()
}

/// 从数据源清单（`console_sources.json` 的 `repo_sources`）里挑出文件名含 `needle`
/// 的 `assets/*.json` 项（数据驱动，Rust 侧 0 资产路径字面量；缺项 = Err，API 面 500 如实）。
fn pick_source(sources: &RepoSources, needle: &str) -> Result<String, String> {
    for s in &sources.repo_sources {
        if s.contains(".json") && s.contains(needle) {
            return Ok(s.clone());
        }
    }
    Err(format!("console_sources.json 缺 {needle} 数据源项（单一事实源漏登记）"))
}

/// `ui_asset_count` 的兜底清单面（overview 首屏不依赖完整 `RepoSources` 解析）。
fn load_sources_for_fallback(repo_root: &Path) -> RepoSources {
    crate::repo::config::load(repo_root).unwrap_or_default()
}

// ---- 内部助手 ----

fn read_json(repo_root: &Path, repo_sources: &[String], relative: &str) -> Result<Value, String> {
    let path = resolve_guarded(repo_root, repo_sources, relative).map_err(|e| e.to_string())?;
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("读 {relative} 失败：{e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("{relative} 非法 JSON：{e}"))
}

/// 已装地图包（`installed_packs()` 口径：`assets/maps/<id>/manifest.json` 存在且目录名 ≠ `_template`）。
fn map_pack_info(repo_root: &Path) -> Vec<Value> {
    let maps_dir = repo_root.join("assets/maps");
    let mut out: Vec<Value> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&maps_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() && name != "_template" {
                let manifest = entry.path().join("manifest.json");
                out.push(json!({
                    "id": name,
                    "manifest_present": manifest.is_file(),
                    "pack_dir": format!("assets/maps/{name}"),
                }));
            }
        }
    }
    out.sort_by(|a, b| {
        let ak = a.get("id").and_then(Value::as_str).unwrap_or("");
        let bk = b.get("id").and_then(Value::as_str).unwrap_or("");
        ak.cmp(bk)
    });
    out
}

/// `dist/maps/*.map-pack` 目录名清单（gitignored；缺失 = 空清单，不报错）。
fn dist_pack_names(repo_root: &Path) -> Vec<String> {
    let dist = repo_root.join("dist/maps");
    let mut names: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dist) {
        for entry in rd.flatten() {
            let n = entry.file_name().to_string_lossy().to_string();
            if n.ends_with(".map-pack") {
                names.push(n.clone());
            }
        }
    }
    names.sort();
    names
}

fn status_str(task: &ti_devmeta::ledger::Task) -> &'static str {
    use ti_devmeta::ledger::Status;
    match task.status {
        Status::Pending => "pending",
        Status::InProgress => "in_progress",
        Status::Blocked => "blocked",
        Status::Verified => "verified",
    }
}

/// 已装地图包数（`overview` 面；与 `map_pack_info` 同口径）。
fn installed_map_packs(repo_root: &Path) -> usize {
    map_pack_info(repo_root).len()
}

/// UI 资产数（`ui_manifest.assets` 长度；读文件失败 = 0，不阻断首屏）。
fn ui_asset_count(repo_root: &Path) -> u64 {
    let Ok(manifest) = pick_source(&load_sources_for_fallback(repo_root), "ui_manifest") else {
        return 0;
    };
    let Ok(raw) = std::fs::read_to_string(repo_root.join(manifest)) else {
        return 0;
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return 0;
    };
    v.get("assets").and_then(Value::as_array).map_or(0, |a| a.len() as u64)
}
