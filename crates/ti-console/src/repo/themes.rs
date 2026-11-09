//! `/api/themes`：主题管理面（S49.3，S48.0 全主题创新）。
//!
//! 职责（30 号 §2.2.1 全主题规范 + 14 号 `ui_theme.py` 工具契约）：
//! - `list`：`assets/themes/` 全部主题包（`compose` 合法 + 资产齐 + `applied` 态 vs 当前 `scenes.json`）；
//! - `verify <tid>`：主题包校验（`compose`/`landscape_primitives`/`assets` 源 + `ui_manifest` 登记，只读）；
//! - `apply <tid>`：`ui_theme.py apply`（写 `scenes.json` 构图 + 部署资产 + `validate_data`），
//!   写面由调用方门控（X-TI-Token + `enable_write`，22 号 §10.1；同 S48.0 wizard theme 步路径）。
//!
//! 本模块 **0 游戏依赖**；只读面（list/verify）0 写盘；apply 写点 = `ui_theme.py` 子进程
//! （`assets/data/scenes.json` + `assets/ui/` + `assets/ui_manifest.json`，S48.0 wizard theme 步同路径）。

use std::path::Path;

use serde_json::{Value, json};

use super::config::RepoSources;

/// 主题包 id 合法性（`^[a-z0-9_]{1,16}$`；与 map 包 id 同口径，防路径穿越/自由字符串）。
#[must_use]
pub fn is_valid_theme_id(tid: &str) -> bool {
    !tid.is_empty()
        && tid.len() <= 16
        && tid.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// 从数据源清单取 scenes.json 路径（`assets/data/scenes.json`，S49.3 已登记 `repo_sources`）。
///
/// # Errors
/// `console_sources.json` 未登记 scenes.json 项 / 路径越白名单 = `Err`（平台不得静默）。
fn pick_scenes_path(repo_root: &Path, sources: &RepoSources) -> Result<std::path::PathBuf, String> {
    for s in &sources.repo_sources {
        if s.ends_with("scenes.json") {
            return super::guard::resolve_guarded(repo_root, &sources.repo_sources, s)
                .map_err(|e| e.to_string());
        }
    }
    Err("console_sources.json 缺 scenes.json 数据源项（S49.3 漏登记）".to_owned())
}

/// `/api/themes` 只读面：全部主题包清单。
///
/// `applied` = 当前 `scenes.json index.landscape.primitives` == 该主题 `landscape_primitives`
/// （S48.0 回注验证后恢复 S39 默认；运行时默认不变，主题 apply 切换后匹配）。
///
/// # Errors
/// scenes.json 读/解析失败（数据源面不可用）= `Err`（如实上报）。
pub fn themes_list(repo_root: &Path, sources: &RepoSources) -> Result<Value, String> {
    let scenes_path = pick_scenes_path(repo_root, sources)?;
    let scenes_raw =
        std::fs::read_to_string(&scenes_path).map_err(|e| format!("读 scenes.json 失败：{e}"))?;
    let scenes: Value =
        serde_json::from_str(&scenes_raw).map_err(|e| format!("scenes.json 非法 JSON：{e}"))?;
    let current = scenes
        .get("index")
        .and_then(|i| i.get("landscape"))
        .and_then(|l| l.get("primitives"))
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    let themes_dir = repo_root.join("assets/themes");
    let mut themes: Vec<Value> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&themes_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let compose_path = themes_dir.join(&name).join("compose.json");
            if !compose_path.is_file() {
                continue;
            }
            let raw = std::fs::read_to_string(&compose_path).unwrap_or_default();
            let Ok(compose) = serde_json::from_str::<Value>(&raw) else {
                themes.push(json!({
                    "id": name, "name": "", "scene": "", "primitives": 0,
                    "assets": [], "valid": false, "applied": false,
                    "error": "compose.json 非法 JSON",
                }));
                continue;
            };
            let prims = compose
                .get("landscape_primitives")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let assets_obj =
                compose.get("assets").and_then(Value::as_object).cloned().unwrap_or_default();
            let mut asset_present = true;
            for (_, src) in &assets_obj {
                let src_rel = src.as_str().unwrap_or("");
                if src_rel.is_empty() || !repo_root.join(src_rel).is_file() {
                    asset_present = false;
                }
            }
            let valid = !prims.is_empty() && asset_present;
            let applied = !prims.is_empty() && current == Value::Array(prims.clone());
            themes.push(json!({
                "id": compose.get("id").and_then(Value::as_str).unwrap_or(&name),
                "name": compose.get("name").and_then(Value::as_str).unwrap_or(""),
                "scene": compose.get("scene").and_then(Value::as_str).unwrap_or(""),
                "primitives": prims.len(),
                "assets": assets_obj.keys().cloned().collect::<Vec<_>>(),
                "valid": valid,
                "applied": applied,
                "error": "",
            }));
        }
    }
    themes.sort_by(|a, b| a["id"].as_str().unwrap_or("").cmp(b["id"].as_str().unwrap_or("")));
    Ok(json!({
        "count": themes.len(),
        "themes": themes,
        "note": "S49.3 主题管理（S48.0 全主题创新）：applied = 当前 scenes.json index.landscape.primitives 匹配该主题",
    }))
}

/// `/api/themes?verify=<tid>`：主题包校验（只读）。
///
/// 判据 = `compose` 合法（`id`/`name`/`scene`/`landscape_primitives`/`assets`）+ `landscape_primitives` 非空
/// + 每个 `compose.assets` 源文件在位 + 每个部署资产 `assets/ui/<name>.png` 在位
/// + `ui_manifest` 登记（`file=assets/ui/<name>.png`）。违规清单非空 = `ok:false`。
///
/// # Errors
/// theme id 非法（`^[a-z0-9_]{1,16}$` 白名单）= `Err`（22 号 §10.3 非自由字符串）。
pub fn theme_verify(repo_root: &Path, tid: &str) -> Result<Value, String> {
    if !is_valid_theme_id(tid) {
        return Err(format!("theme id 非法（^[a-z0-9_]{{1,16}}$）：{tid}"));
    }
    let mut violations: Vec<String> = Vec::new();
    let compose_path = repo_root.join("assets/themes").join(tid).join("compose.json");
    if !compose_path.is_file() {
        return Ok(json!({
            "ok": false, "tid": tid,
            "violations": vec![format!("{tid} 缺 compose.json（assets/themes/{tid}/）")],
        }));
    }
    let raw = std::fs::read_to_string(&compose_path).unwrap_or_default();
    let compose: Value = match serde_json::from_str(&raw) {
        Ok(c) => c,
        Err(e) => {
            return Ok(json!({
                "ok": false, "tid": tid,
                "violations": vec![format!("compose.json 非法 JSON：{e}")],
            }));
        }
    };
    if compose.get("id").and_then(Value::as_str) != Some(tid) {
        violations.push(format!(
            "compose.id ≠ 目录名（{} vs {tid}）",
            compose.get("id").unwrap_or(&Value::Null)
        ));
    }
    let prims =
        compose.get("landscape_primitives").and_then(Value::as_array).cloned().unwrap_or_default();
    if prims.is_empty() {
        violations.push("landscape_primitives 空或缺失".to_owned());
    }
    let assets_obj = compose.get("assets").and_then(Value::as_object).cloned().unwrap_or_default();
    if assets_obj.is_empty() {
        violations.push("assets 空（compose.assets 无资产）".to_owned());
    }
    // ui_manifest 登记集合（`file=assets/ui/<name>.png` 匹配）
    let manifest_path = repo_root.join("assets/ui_manifest.json");
    let manifest: Value = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|r| serde_json::from_str(&r).ok())
        .unwrap_or(Value::Null);
    let manifest_entries =
        manifest.get("assets").and_then(Value::as_array).cloned().unwrap_or_default();
    for (name, src) in &assets_obj {
        let src_rel = src.as_str().unwrap_or("");
        if src_rel.is_empty() || !repo_root.join(src_rel).is_file() {
            violations.push(format!("资产源缺失 {src_rel}（{name}）"));
        }
        let deployed = repo_root.join("assets/ui").join(format!("{name}.png"));
        if !deployed.is_file() {
            violations.push(format!("部署资产缺失 assets/ui/{name}.png（apply 后生成）"));
        }
        let expected_file = format!("assets/ui/{name}.png");
        let registered = manifest_entries.iter().any(|e| {
            e.get("name").and_then(Value::as_str) == Some(name.as_str())
                && e.get("file").and_then(Value::as_str) == Some(expected_file.as_str())
        });
        if !registered {
            violations.push(format!(
                "ui_manifest 未登记 {name}（source=ui_theme，apply --register 后登记）"
            ));
        }
    }
    Ok(json!({
        "ok": violations.is_empty(),
        "tid": tid,
        "violations": violations,
    }))
}

/// `/api/themes/preview`（只读出图，零写面）：运行时注入主题构图 + 资产回退出图。
///
/// 命令 = `treasure-bevy-native --shot --scene=index --viewport=1920x1080`（bin 路径 =
/// [`crate::content::ui_wizard::resolve_bin_path`]），env `TI_THEME=<tid>`（引擎侧
/// [`apply_theme_preview`] 注入 `index.landscape.primitives`，**不 apply 不改仓库**）。
/// 无头环境走 `xvfb-run -a`（Linux 迁移口径，同 [`scene_shot_command`]）。
/// 出图 = `evidence/bevy_native_theme_<tid>_1920x1080.png`（gitignored 侧车，平台面板内嵌展示）。
///
/// # Errors
/// theme id 非法 = `Err`；bin 缺位 / spawn 失败 = `Err`（如实上报）。
pub fn theme_preview(repo_root: &Path, tid: &str) -> Result<Value, String> {
    if !is_valid_theme_id(tid) {
        return Err(format!("theme id 非法（^[a-z0-9_]{{1,16}}$）：{tid}"));
    }
    let bin = crate::content::ui_wizard::resolve_bin_path(repo_root);
    if !bin.is_file() {
        return Err(format!(
            "treasure-bevy-native 未构建：{}（先 cargo build -p ti-engine --bin treasure-bevy-native）",
            bin.display()
        ));
    }
    // 无头环境需虚拟显示（xvfb-run -a，Linux 迁移口径；同 scene_shot_command）。
    let xvfb = std::path::Path::new("/usr/bin/xvfb-run");
    let mut cmd = if xvfb.is_file() {
        let mut c = std::process::Command::new(xvfb);
        c.arg("-a").arg(&bin);
        c
    } else {
        std::process::Command::new(&bin)
    };
    cmd.env("TI_THEME", tid).arg("--shot").arg("--scene=index").arg("--viewport=1920x1080");
    cmd.current_dir(repo_root);
    crate::runner::apply_minimal_env(&mut cmd);
    // apply_minimal_env 会 env_clear + 白名单，清掉 TI_THEME —— 需在清后重设。
    // TI_SHOT_NAME 固定出图名（bin 默认 scene+viewport 后缀会互相覆盖；S27.7 口径）。
    cmd.env("TI_THEME", tid);
    cmd.env("TI_SHOT_NAME", format!("bevy_native_theme_{tid}"));
    let out = cmd
        .output()
        .map_err(|e| format!("theme preview spawn 失败（{e}）：{:?}", bin.display()))?;
    let output =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let exit_code = out.status.code().unwrap_or(-1);
    let shot_ok = output.contains("已写");
    // bin 命名口径：TI_SHOT_NAME 只覆盖基名，`_<WxH>` 后缀仍追加（viewport 固定 1920×1080）。
    let out_name = format!("bevy_native_theme_{tid}_1920x1080.png");
    let out_path = repo_root.join("evidence").join(&out_name);
    Ok(json!({
        "ok": exit_code == 0 && shot_ok && out_path.is_file(),
        "tid": tid,
        "exit_code": exit_code,
        "shot_ok": shot_ok,
        "image": format!("evidence/{out_name}"),
        "output": output.chars().take(2000).collect::<String>(),
        "note": "theme preview：TI_THEME 运行时注入 index.landscape.primitives + 资产回退，--shot 出图（零写面，不 apply 不改仓库）",
    }))
}

/// `/api/themes/apply`（写，调用方门控）：`ui_theme.py apply --theme <tid> [--register]`。
///
/// 子进程走最小环境集（22 号 §10.3）；判据 = exit 0 + 输出含 `validate_data` 成功
/// （`0 违规` / `OK`）。写 `scenes.json` 构图 + 部署资产 + 登记 + `validate_data`（工具契约）。
///
/// # Errors
/// theme id 非法 = `Err`（22 号 §10.3 白名单）；`ui_theme.py` 执行失败（python3 缺位）= `Err`。
pub fn theme_apply(repo_root: &Path, tid: &str, register: bool) -> Result<Value, String> {
    if !is_valid_theme_id(tid) {
        return Err(format!("theme id 非法（^[a-z0-9_]{{1,16}}$）：{tid}"));
    }
    let mut cmd = std::process::Command::new("python3");
    cmd.arg(repo_root.join("scripts/ui_theme.py")).arg("apply").arg("--theme").arg(tid);
    if register {
        cmd.arg("--register");
    }
    cmd.current_dir(repo_root);
    crate::runner::apply_minimal_env(&mut cmd);
    let out = cmd.output().map_err(|e| format!("ui_theme.py 执行失败（python3 缺位？）：{e}"))?;
    let output =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let exit_code = out.status.code().unwrap_or(-1);
    let ok = exit_code == 0
        && (output.contains("0 违规") || output.contains("validate_data") && output.contains("OK"));
    Ok(json!({
        "ok": ok,
        "tid": tid,
        "register": register,
        "exit_code": exit_code,
        "output": output.chars().take(3000).collect::<String>(),
        "note": "ui_theme.py apply：回注 scenes.json index.landscape.primitives + 部署资产 + 登记 + validate_data（S48.0）",
    }))
}

// ---- S49.3 HTTP 面（routes.rs 薄分派；响应构造 + 写面门控在此，控 routes.rs 规模） ----

/// `GET /api/themes`：`?verify=<tid>` 只读校验；无参数 = list（全部主题包 + applied 态）。
#[must_use]
pub fn themes_get(repo_root: &Path, url: &str, gz: bool) -> Vec<u8> {
    if let Some(tid) = crate::routes::query_param(url, "verify") {
        match theme_verify(repo_root, tid) {
            Ok(v) => crate::routes::json200(&v, gz),
            Err(e) => crate::routes::header_body(
                "application/json",
                format!(r#"{{"ok":false,"error":"{e}"}}"#).as_bytes(),
                "400 Bad Request",
                "no-store",
                None,
                gz,
            ),
        }
    } else if let Some(tid) = crate::routes::query_param(url, "preview") {
        // S49.4c 主题预览（只读出图，零写面）：TI_THEME 运行时注入 + --shot 出图。
        match theme_preview(repo_root, tid) {
            Ok(v) => crate::routes::json200(&v, gz),
            Err(e) => crate::routes::header_body(
                "application/json",
                format!(r#"{{"ok":false,"error":"{e}"}}"#).as_bytes(),
                "400 Bad Request",
                "no-store",
                None,
                gz,
            ),
        }
    } else {
        crate::routes::api_500_or_json(
            crate::routes::load_json(repo_root, |s| themes_list(repo_root, s)),
            gz,
        )
    }
}

/// `POST /api/themes/apply`：应用主题包（写 scenes.json + 部署资产）。
/// 写操作鉴权（403）→ Content-Type application/json（400）→ body ≤ 64KiB（413）→
/// body `{"theme","register"?,"enable_write"}` → `enable_write=true`（400，30 号 §3.1 默认关）→
/// [`theme_apply`]（`ui_theme.py apply` 子进程，工具契约）。
#[must_use]
pub fn handle_themes_apply(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    let (theme, register) = match parse_themes_apply(body, token, content_type, want_gzip) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    match theme_apply(repo_root, &theme, register) {
        Ok(v) => crate::routes::json200(&v, want_gzip),
        Err(e) => {
            let body = format!(r#"{{"ok":false,"error":"{}"}}"#, e.replace('"', "\\\""));
            crate::routes::header_body(
                "application/json",
                body.as_bytes(),
                "400 Bad Request",
                "no-store",
                None,
                want_gzip,
            )
        }
    }
}

/// `/api/themes/apply` 请求体验证链（抽离控 `handle_themes_apply` 行数）。
/// 返回 `(theme, register)` 或**已构建的错误响应**。
fn parse_themes_apply(
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
) -> Result<(String, bool), Vec<u8>> {
    if !crate::routes::auth_ok(token) {
        return Err(crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"403: 写操作需 X-TI-Token（22 号 §10.1）"}"#.as_bytes(),
            "403 Forbidden",
            "no-store",
            None,
            want_gzip,
        ));
    }
    let ct_ok = content_type.is_some_and(|c| {
        let base = c.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        base == "application/json"
    });
    if !ct_ok {
        return Err(crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"400: Content-Type 必须为 application/json（22 号 §10.3）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        ));
    }
    if body.len() > 65_536 {
        return Err(crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"413: 请求体超 64 KiB 上限（22 号 §10.1）"}"#.as_bytes(),
            "413 Payload Too Large",
            "no-store",
            None,
            want_gzip,
        ));
    }
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            let text = format!(r#"{{"ok":false,"error":"400: body 非合法 JSON：{e}"}}"#);
            return Err(crate::routes::header_body(
                "application/json",
                text.as_bytes(),
                "400 Bad Request",
                "no-store",
                None,
                want_gzip,
            ));
        }
    };
    // 30 号 §3.1 写面默认关（北极星「0 游戏产物变更」）：写动作必须显式 enable_write=true。
    if payload.get("enable_write").and_then(Value::as_bool) != Some(true) {
        return Err(crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"400: 写动作需 `enable_write=true`（30 号 §3.1 默认关）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        ));
    }
    let theme = payload.get("theme").and_then(Value::as_str).unwrap_or("");
    if theme.is_empty() || !is_valid_theme_id(theme) {
        return Err(crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"400: theme 必须为 ^[a-z0-9_]{1,16}$（22 号 §10.3 白名单；非自由字符串）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        ));
    }
    let register = payload.get("register").and_then(Value::as_bool).unwrap_or(false);
    Ok((theme.to_owned(), register))
}
