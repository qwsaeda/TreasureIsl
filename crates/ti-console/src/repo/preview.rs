//! S86.3 真渲染预览服务（`docs/plan-s86-visual-editor.md` §2 定案 A）：
//!
//! `POST /api/preview` = 平台**受控写点**（仅落 gitignored `evidence/console/preview/<id>/`）：
//! 校验 body 枚举 → 未落盘编辑数据写临时目录 → `runner::jobs` 后台 spawn
//! `python3 scripts/preview_shot.py`（程序白名单内，逐参数非 shell）→ 返回 job id + PNG 路径；
//! 轮询走统一作业面 `GET /api/jobs?kind=preview&id=<id>`（23 号 §6.2）。
//!
//! **零跟踪文件写盘**：body 数据只写 `evidence/console/preview/`（写点白名单已有），
//! 不写 `assets/data/` 或仓库其它跟踪位置。引擎侧由 `TI_PREVIEW_DIR`  env 读同目录，
//! 首访前经 `ui_schema::install_injected` / `table_source::install` 注入（OnceLock 一次性，
//! 与 `--shot` 冷启天然契合）。
//!
//! 安全口径（22 号 §10.3）：Content-Type application/json 400
//! + 请求体 ≤ 64 KiB 413 + scene/viewport/id 白名单枚举 400；禁自由 shell / 自由路径。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::routes::{json200, status_json};
use crate::runner::jobs::{self, JobSpec};

/// 可预览场景白名单（7 场景全量；S87.5 起 vs 世界层纳入可视化编辑器，见 22 号 §world 模式契约）。
pub const PREVIEW_SCENES: [&str; 7] =
    ["index", "select_role", "select_map", "saveload", "message", "main_map", "vs"];

/// 视口白名单（S84 纯横屏主判据 = 1920x1080；其余为 S29.5 多视口出图已有口径）。
pub const PREVIEW_VIEWPORTS: [&str; 4] = ["1920x1080", "1080x1920", "1280x720", "960x540"];

/// 请求体上限（22 号 §10.1，与 `routes::handle_post` 同口径）。
const MAX_BODY: usize = 65_536;

/// 单次出图超时（debug bin + 软件渲染 llvmpipe 实测单 job 3~5min，多 job 并发更慢；
/// 留足余量，仍由 job 模型硬超时兜底，防死循环）。
const JOB_TIMEOUT_S: u64 = 900;

/// 写点根目录（相对仓库根；`console_sources.json::write_whitelist` 已登记 `evidence/console/`）。
const PREVIEW_ROOT: &str = "evidence/console/preview";

/// body JSON blob 名 → 临时文件名。`@` 前缀 = 地图包段（落 `<dir>/<map_id>/`），
/// 其余 = 整表/schema 文本（落 `<dir>/`）。名字全为固定字面量，不接受自由路径。
const BLOB_FILES: [(&str, &str); 9] = [
    ("schema_json", "components_v1.json"),
    ("world_components_json", "world_components.json"),
    ("asset_roles_json", "asset_roles.json"),
    ("board_json", "board.json"),
    ("scenes_json", "scenes.json"),
    ("strings_json", "strings.json"),
    ("map_json", "@map.json"),
    ("layout_json", "@layout.json"),
    ("pack_strings_json", "@strings.json"),
];

/// `POST /api/preview` 总入口（Content-Type → body 上限 → 参数白名单 → 后台 job）。
#[must_use]
pub fn handle_preview(
    repo_root: &Path,
    body: &[u8],
    content_type: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    let ct_ok = content_type.is_some_and(|value| {
        value.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("application/json")
    });
    if !ct_ok {
        return err_resp(
            400,
            "400: Content-Type 必须为 application/json（22 号 §10.3）",
            want_gzip,
        );
    }
    if body.len() > MAX_BODY {
        return err_resp(413, "413: 请求体超 64 KiB 上限（22 号 §10.1）", want_gzip);
    }
    let payload: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(error) => return err_resp(400, &format!("400: body 非合法 JSON：{error}"), want_gzip),
    };
    // 预览是**只读观感操作**：只写 gitignored 临时目录（evidence/console/preview/<id>/）+
    // 出一张 PNG，**不改任何游戏数据**（assets/data 下跟踪文件零写）⇒ 不属 30 号 §3.1 的
    // 「写数据」面。旧实现要求 enable_write=true 是语义错配（拿「允许改游戏数据」的开关
    // 去管一个只读预览），已移除：预览的写面风险由「参数白名单 + 逐参数非 shell + 后台 job
    // 硬超时 + 只写 gitignored 路径」四重约束覆盖，不靠 enable_write。
    // 白名单 action id（与 console_commands.json::api_endpoints 对账；缺省 = 按端点自身 id）。
    let action = str_field(&payload, "action").unwrap_or_default();
    if !action.is_empty() && action != "preview" {
        return err_resp(400, "400: action 非法（本端点仅接受 preview）", want_gzip);
    }
    match start_preview(repo_root, &payload) {
        Ok(value) => json200(&value, want_gzip),
        Err((code, message)) => err_resp(code, &message, want_gzip),
    }
}

/// 统一错误响应（`{"ok":false,"error":...}` + 状态码）。
fn err_resp(code: u16, message: &str, want_gzip: bool) -> Vec<u8> {
    status_json(code, &json!({"ok": false, "error": message}), want_gzip)
}

/// 校验通过的预览请求（所有字段均为白名单内值 / 固定文件名）。
struct PreviewRequest {
    /// 场景 id（白名单 [`PREVIEW_SCENES`]）。
    scene: String,
    /// 视口 `WxH`（白名单 [`PREVIEW_VIEWPORTS`]）。
    viewport: String,
    /// 作业 id（同时作临时目录名与出图基名后缀）。
    job_id: String,
    /// 地图包 id（合法时复制已装包文件到临时目录作注入基线）。
    map_id: Option<String>,
    /// UI schema id（仅作追踪元数据，schema 正文来自 `schema_json`）。
    schema_id: Option<String>,
    /// 未落盘 JSON blob（字段名 → 值，文件名映射见 [`BLOB_FILES`]）。
    blobs: Vec<(String, Value)>,
}

/// 解析 + 校验 body；非法枚举 / 路径字符 / blob 类型 = 400（不 spawn）。
fn parse_request(payload: &Value) -> Result<PreviewRequest, (u16, String)> {
    let scene = str_field(payload, "scene").unwrap_or_default();
    if !PREVIEW_SCENES.contains(&scene.as_str()) {
        return Err((400, format!("400: scene 非法（白名单 {PREVIEW_SCENES:?}）")));
    }
    let viewport = {
        let raw = str_field(payload, "viewport").unwrap_or_else(|| "1920x1080".to_owned());
        if !PREVIEW_VIEWPORTS.contains(&raw.as_str()) {
            return Err((400, format!("400: viewport 非法（白名单 {PREVIEW_VIEWPORTS:?}）")));
        }
        raw
    };
    let job_id = match str_field(payload, "id") {
        Some(id) if !id.is_empty() => {
            if !legal_id(&id) {
                return Err((400, "400: id 非法（须 ^[a-z0-9_-]{1,64}$）".to_owned()));
            }
            id
        }
        _ => new_job_id(&scene),
    };
    let map_id = optional_id(payload, "map_id", "map_id")?;
    let schema_id = optional_id(payload, "schema_id", "schema_id")?;
    let mut blobs: Vec<(String, Value)> = Vec::new();
    for (field, file) in BLOB_FILES {
        let Some(value) = payload.get(field) else { continue };
        if !value.is_object() {
            return Err((400, format!("400: {field} 必须是 JSON object（整表/schema 形状）")));
        }
        if file.starts_with('@') && map_id.is_none() {
            return Err((400, format!("400: {field} 需同时给出 map_id（地图包段落点）")));
        }
        blobs.push((field.to_owned(), value.clone()));
    }
    Ok(PreviewRequest { scene, viewport, job_id, map_id, schema_id, blobs })
}

/// 取字符串字段（非字符串视为缺省；空串由调用方判）。
fn str_field(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(Value::as_str).map(str::to_owned).filter(|s| !s.trim().is_empty())
}

/// 可选 id 字段：给了就须匹配 `^[a-z0-9_]{1,64}$`（防目录穿越）。
fn optional_id(payload: &Value, key: &str, label: &str) -> Result<Option<String>, (u16, String)> {
    let Some(raw) = str_field(payload, key) else { return Ok(None) };
    if raw.len() > 64
        || !raw.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err((400, format!("400: {label} 非法（须 ^[a-z0-9_]{{1,64}}$）")));
    }
    Ok(Some(raw))
}

/// 作业 / 目录 id 合法性（防路径穿越，与 `jobs::is_legal_job_id` 同口径）。
fn legal_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// 生成唯一定时 id（毫秒时间戳；同场景并发请求不互相覆盖临时目录 / 出图）。
fn new_job_id(scene: &str) -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    let stamp = u64::try_from(millis).unwrap_or(0);
    format!("preview_{scene}_{stamp}")
}

/// 落临时目录 + 后台 spawn；返回 `{ok,id,kind,scene,viewport,image,status,log,poll}`。
fn start_preview(repo_root: &Path, payload: &Value) -> Result<Value, (u16, String)> {
    let request = parse_request(payload)?;
    let dir = stage_preview(repo_root, &request)?;
    let image = format!("evidence/bevy_native_preview_{}_{}.png", request.job_id, request.viewport);
    let preview_dir_rel = dir.strip_prefix(repo_root).map_or_else(
        |_| dir.to_string_lossy().into_owned(),
        |p| p.to_string_lossy().replace('\\', "/"),
    );
    let mut args = vec![
        "scripts/preview_shot.py".to_owned(),
        "--scene".to_owned(),
        request.scene.clone(),
        "--viewport".to_owned(),
        request.viewport.clone(),
        "--preview-dir".to_owned(),
        dir.to_string_lossy().into_owned(),
        "--id".to_owned(),
        request.job_id.clone(),
    ];
    if let Some(map_id) = &request.map_id {
        args.push("--map-id".to_owned());
        args.push(map_id.clone());
    }
    let meta = json!({
        "scene": request.scene,
        "viewport": request.viewport,
        "image": image,
        "preview_dir": preview_dir_rel,
        "map_id": request.map_id,
        "schema_id": request.schema_id,
        "note": "S86.3 真渲染预览：python3 包装 treasure-bevy-native --shot（零跟踪文件写盘）",
    });
    let mut spec = JobSpec::new("preview", &request.job_id, "python3", args, meta)
        .map_err(|message| (400, format!("400: {message}")))?;
    spec.timeout_s = Some(JOB_TIMEOUT_S);
    let spawned = jobs::spawn_job(repo_root, &spec)
        .map_err(|message| (500, format!("500: preview 作业 spawn 失败：{message}")))?;
    // 证明与作业绑定（23 号 §6.2 evidence[]）：PNG + 临时注入清单随 job id 可查。
    jobs::register_evidence(
        repo_root,
        "preview",
        &request.job_id,
        &[image.clone(), format!("{preview_dir_rel}/preview.json")],
    );
    Ok(json!({
        "ok": true,
        "id": request.job_id,
        "kind": "preview",
        "scene": request.scene,
        "viewport": request.viewport,
        "image": image,
        "preview_dir": preview_dir_rel,
        "status": spawned.get("status").cloned().unwrap_or(Value::Null),
        "log": spawned.get("log").cloned().unwrap_or(Value::Null),
        "pid": spawned.get("pid").cloned().unwrap_or(Value::Null),
        "poll": format!("/api/jobs?kind=preview&id={}", request.job_id),
        "note": "轮询 GET /api/jobs?kind=preview&id=<id> 取 exit_code/log_tail/evidence；成功后 <img src=\"/{image}?t=<ts>\"> 破缓存",
    }))
}

/// 把 body 数据写进 gitignored 临时目录（固定文件名 + 已校验 id；0 自由路径）。
fn stage_preview(repo_root: &Path, request: &PreviewRequest) -> Result<PathBuf, (u16, String)> {
    let dir = repo_root.join(PREVIEW_ROOT).join(&request.job_id);
    std::fs::create_dir_all(&dir)
        .map_err(|error| (500, format!("500: 建预览目录失败：{error}")))?;
    for (field, value) in &request.blobs {
        let file = BLOB_FILES
            .iter()
            .find(|(name, _)| name == field)
            .map_or_else(String::new, |(_, file)| (*file).to_owned());
        let target = match file.strip_prefix('@') {
            Some(pack_file) => {
                let Some(map_id) = request.map_id.as_deref() else {
                    return Err((400, format!("400: {field} 需 map_id")));
                };
                dir.join(map_id).join(pack_file)
            }
            None => dir.join(&file),
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| (500, format!("500: 建预览子目录失败：{error}")))?;
        }
        let text = serde_json::to_string_pretty(value)
            .map_err(|error| (500, format!("500: {field} 序列化失败：{error}")))?;
        std::fs::write(&target, format!("{text}\n"))
            .map_err(|error| (500, format!("500: 写 {field} 失败：{error}")))?;
    }
    if let Some(map_id) = &request.map_id {
        stage_pack(repo_root, &dir, map_id)?;
    }
    let meta = json!({
        "schema_version": "preview/1",
        "id": request.job_id,
        "scene": request.scene,
        "viewport": request.viewport,
        "map_id": request.map_id,
        "schema_id": request.schema_id,
        "blobs": request.blobs.iter().map(|(name, _)| name).collect::<Vec<_>>(),
    });
    let text = serde_json::to_string_pretty(&meta)
        .map_err(|error| (500, format!("500: preview.json 序列化失败：{error}")))?;
    std::fs::write(dir.join("preview.json"), format!("{text}\n"))
        .map_err(|error| (500, format!("500: 写 preview.json 失败：{error}")))?;
    Ok(dir)
}

/// 复制已装包的四文件作注入基线（只复制存在的；body 的 `*_json` 覆盖优先，故目标存在即跳过）。
fn stage_pack(repo_root: &Path, dir: &Path, map_id: &str) -> Result<(), (u16, String)> {
    let pack_dir = dir.join(map_id);
    std::fs::create_dir_all(&pack_dir)
        .map_err(|error| (500, format!("500: 建地图包临时目录失败：{error}")))?;
    let source = repo_root.join("assets/maps").join(map_id);
    for file in ["manifest.json", "map.json", "layout.json", "strings.json"] {
        let target = pack_dir.join(file);
        if target.exists() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(source.join(file)) else { continue };
        std::fs::write(&target, text)
            .map_err(|error| (500, format!("500: 复制包文件 {file} 失败：{error}")))?;
    }
    Ok(())
}
