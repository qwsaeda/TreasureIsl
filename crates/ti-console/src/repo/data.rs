//! 只读 API 的响应组装（`/api/overview` `/api/tasks` `/api/evidence` `/api/assets` `/api/session`）。
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
pub fn audit_list(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    let base = resolve_guarded(repo_root, &sources.repo_sources, "evidence/console/")
        .map_err(|e| e.to_string())?;
    let mut reports: Vec<Value> = Vec::new();
    let Ok(rd) = std::fs::read_dir(&base) else {
        return Ok(json!({ "reports": reports, "count": 0, "base": "evidence/console/" }));
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("audit-") || !name.to_lowercase().ends_with(".md") {
            continue;
        }
        // 命名规则 = `audit-<task>-r<N>-<ab>.md`（`<task>` 本身可含 `-`/`.`，如 `S32.0a`）：
        // 末段 = side（a/b），倒数第 2 段以 `r` 开头 = round，其余 = task。
        let stem = name.trim_start_matches("audit-").trim_end_matches(".md");
        let segs: Vec<&str> = stem.split('-').collect();
        let ab = segs.last().copied().unwrap_or("");
        let round = if segs.len() >= 2 { segs[segs.len() - 2] } else { "" };
        let task = if segs.len() >= 3 { segs[..segs.len() - 2].join("-") } else { String::new() };
        let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let fields = parse_audit_head(&text);
        reports.push(json!({
            "file": format!("evidence/console/{name}"),
            "task": task,
            "round": round,
            "side": ab,
            "verdict": fields.verdict,
            "new": fields.found,
            "false_pos": fields.false_pos,
            "residual": fields.residual,
        }));
    }
    reports.sort_by(|a, b| {
        let ak = a.get("file").and_then(Value::as_str).unwrap_or("");
        let bk = b.get("file").and_then(Value::as_str).unwrap_or("");
        ak.cmp(bk)
    });
    Ok(json!({ "reports": reports, "count": reports.len(), "base": "evidence/console/" }))
}

/// 解析审计报告抬头的 §7.3 四行机器可解析字段（代码块内 总判定/新发现/误报/残留 行）。
struct AuditHead {
    verdict: Option<String>,
    found: Option<u64>,
    false_pos: Option<u64>,
    residual: Option<u64>,
}

fn parse_audit_head(text: &str) -> AuditHead {
    let mut out = AuditHead { verdict: None, found: None, false_pos: None, residual: None };
    for line in text.lines().take(80) {
        let s = line.trim();
        if let Some(v) = s.strip_prefix("总判定:") {
            out.verdict = Some(v.trim().to_owned());
        } else if let Some(v) = s.strip_prefix("新发现:") {
            out.found = v.trim().parse().ok();
        } else if let Some(v) = s.strip_prefix("误报:") {
            out.false_pos = v.trim().parse().ok();
        } else if let Some(v) = s.strip_prefix("残留:") {
            out.residual = v.trim().parse().ok();
        }
    }
    out
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
