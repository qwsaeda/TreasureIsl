//! S86.2 UI 组件可视化编辑器数据源（22 号 §5/§10.2 只读适配 + §10.1 受控写面）：
//!
//! - `components_get`：只读 `assets/data/ui_schema/components_v1.json`（全场景 + `?scene=` 回显）；
//! - `roles_get`：只读 `asset_roles.json` + `ui_icons.json`（role / icon 下拉选项）；
//! - `validate_components`：**内存 schema** 校验（Rust 镜像 `ui_schema_check.py` 的 enum/禁绝对坐标/
//!   条件必填/登记对账）+ 子进程跑 `ui_schema_check.py --check`（on-disk 基线）与
//!   `check_scene_narrative.py --check`（叙事约束）；只读、零写盘；
//! - `apply_components`：规范化写 `components_v1.json`（`json.dumps(indent=2)` 同口径、保键序、
//!   带尾换行）+ 写盘后门禁复验（FAIL 回滚）+ `write_probe` 写点探针。
//!
//! 本模块 **0 游戏依赖**（不引 ti-core/ti-engine；沿用 `themes.rs` 本地 `std::fs` 范式）；
//! 0 写盘面除 `apply_components` 的唯一写点 `components_v1.json`。

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

use super::snapshot::FrozenSnapshot;

/// 组件实例（只读源 + 唯一写点）。
const COMPONENTS_REL: &str = "assets/data/ui_schema/components_v1.json";
/// role → 资产路径登记（只读）。
const ROLES_REL: &str = "assets/data/ui_schema/asset_roles.json";
/// icon 名册（只读）。
const ICONS_REL: &str = "assets/data/ui_icons.json";
/// i18n key 源（只读）。
const STRINGS_REL: &str = "assets/data/strings.json";
/// schema 门禁脚本（子进程，on-disk 基线 + apply 后复验）。
const SCHEMA_SCRIPT: &str = "scripts/ui_schema_check.py";
/// 叙事约束门禁脚本（子进程，只读）。
const NARRATIVE_SCRIPT: &str = "scripts/check_scene_narrative.py";
/// 端点白名单数据面（「白名单 id」校验源）。
const COMMANDS_REL: &str = "crates/ti-console/console_commands.json";

/// 5 纯 UI 场景闭集（`ui_schema_check.py::SCENES` 镜像）。
const SCENES: [&str; 5] = ["index", "select_role", "select_map", "message", "saveload"];
/// 须 `role` 的组件类型。
const TYPE_ROLE: [&str; 5] = ["image", "rect", "grad", "glow", "badge"];
/// 须 `binding` 的组件类型。
const TYPE_BINDING: [&str; 5] = ["text", "button", "input", "list", "panel"];
/// 须 `action` 的组件类型。
const TYPE_ACTION: [&str; 3] = ["button", "list", "badge"];
/// 合法 `layout.intent` 闭集（语义意图 + anchored；无绝对像素）。
const INTENTS: [&str; 8] =
    ["fill", "center", "side_left", "side_right", "bottom_bar", "flex_col", "flex_row", "anchored"];
/// 合法 `action` 闭集（`ui_schema_check.py::ACTIONS` 镜像）。
const ACTIONS: [&str; 12] = [
    "menu_select",
    "character_select",
    "map_select",
    "slot_load",
    "slot_write",
    "msg_next",
    "msg_prev",
    "spin",
    "back",
    "attack",
    "do_item",
    "defend",
];
/// 合法 `state` 闭集。
const STATES: [&str; 4] = ["normal", "selected", "hover", "disabled"];
/// 合法 `tier` 闭集。
const TIERS: [&str; 3] = ["title", "body", "hint"];
/// 组件顶层禁止的绝对坐标字段（出现在导出 JSON 即 FAIL）。
const FORBIDDEN_COORD: [&str; 5] = ["x", "y", "w", "h", "rect"];
/// 合法 `layout.anchor` 闭集。
const ANCHORS: [&str; 9] = [
    "top_left",
    "top_right",
    "top_center",
    "bottom_left",
    "bottom_right",
    "bottom_center",
    "center_left",
    "center_right",
    "center",
];

/// 数组成员判定（与 Python `in set` 同义）。
fn contains(pool: &[&str], value: &str) -> bool {
    pool.contains(&value)
}

/// 读取并解析仓库 JSON（缺文件/非法 JSON = `Err`，如实上报不静默）。
fn read_json(repo_root: &Path, relative: &str) -> Result<Value, String> {
    let path = repo_root.join(relative);
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("读 {relative} 失败：{e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("{relative} 非法 JSON：{e}"))
}

/// `GET /api/ui/components`：全场景实例 + `scene` 回显（0 写盘）。
///
/// 返回 `{ok, schema_version, schema, scene, count}`；`schema` = `components_v1.json` 原文
/// 解析结果（前端 tab 切换只改本地选中态，不重复取数）。
///
/// # Errors
/// `components_v1.json` 读失败 / 非法 JSON = `Err`（如实上报，不静默）。
pub fn components_get(repo_root: &Path, scene: &str) -> Result<Value, String> {
    let schema = read_json(repo_root, COMPONENTS_REL)?;
    let count = schema.get("scenes").and_then(Value::as_object).map_or(0usize, |obj| {
        obj.values().filter_map(|s| s.get("elements").and_then(Value::as_array)).map(Vec::len).sum()
    });
    Ok(json!({
        "ok": true,
        "schema_version": schema.get("schema_version").cloned().unwrap_or(Value::Null),
        "schema": schema,
        "scene": scene,
        "count": count,
        "note": "S86.2 组件可视化编辑器只读源；layout 禁绝对坐标（intent/anchored 百分比）",
    }))
}

/// `GET /api/ui/roles`：role → 资产路径 + icon 名册（下拉选项，0 写盘）。
///
/// # Errors
/// `asset_roles.json` 读失败 / 非法 JSON = `Err`（icon 册缺位退化为空，不阻断）。
pub fn roles_get(repo_root: &Path) -> Result<Value, String> {
    let roles_doc = read_json(repo_root, ROLES_REL)?;
    let roles: Vec<Value> = roles_doc
        .get("roles")
        .and_then(Value::as_object)
        .map(|obj| {
            obj.iter()
                .filter_map(|(role, path)| path.as_str().map(|p| json!({"role": role, "path": p})))
                .collect()
        })
        .unwrap_or_default();
    let icons_doc = read_json(repo_root, ICONS_REL).unwrap_or(Value::Null);
    let icons: Vec<String> = icons_doc
        .get("icons")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|i| i.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    Ok(json!({
        "ok": true,
        "roles": roles,
        "icons": icons,
        "note": "role 来自 asset_roles.json；icon 来自 ui_icons.json（ui_schema_check S5 登记对账）",
    }))
}

/// 登记集合：role / icon / i18n key（`ui_schema_check.py` S5/S6 镜像所需的只读数据面）。
fn registries(repo_root: &Path) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let roles = read_json(repo_root, ROLES_REL)
        .ok()
        .and_then(|v| v.get("roles").and_then(Value::as_object).cloned())
        .map(|obj| obj.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let icons = read_json(repo_root, ICONS_REL)
        .ok()
        .and_then(|v| v.get("icons").and_then(Value::as_array).cloned())
        .map(|arr| {
            arr.iter()
                .filter_map(|i| i.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let strings = read_json(repo_root, STRINGS_REL)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .map(|obj| {
            obj.keys().filter(|k| k.as_str() != "schema_version").cloned().collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    (roles, icons, strings)
}

/// binding 模板样例变量代入（`ui_schema_check.py::SAMPLE_VARS` 镜像；未知变量跳过）。
fn resolve_binding(key: &str) -> String {
    key.replace("{label}", "panda")
        .replace("{id}", "m01")
        .replace("{idx}", "0")
        .replace("{kind}", "help")
}

/// id 合法性：`^[a-z][a-z0-9_]*$`（不引 regex crate，逐字符判定）。
fn id_ok(id: &str) -> bool {
    let Some(first) = id.chars().next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    id.chars().skip(1).all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// **内存 schema** 校验（不落盘）：镜像 `ui_schema_check.py` 的 S1~S6 + 禁绝对坐标点名。
///
/// 返回违规清单（空 = PASS）；每条带 `场景.组件 id` 定位，FAIL 时前端红字逐条回显。
pub fn validate_schema_value(repo_root: &Path, schema: &Value) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    if schema.get("schema_version").and_then(Value::as_i64) != Some(1) {
        errors.push(format!("schema_version 应为 1，实得 {:?}", schema.get("schema_version")));
    }
    let (roles, icons, strings) = registries(repo_root);
    let Some(scenes) = schema.get("scenes").and_then(Value::as_object) else {
        errors.push("缺 scenes 对象".to_owned());
        return errors;
    };
    for (scene_id, scene) in scenes {
        if !contains(&SCENES, scene_id) {
            errors.push(format!("场景 {scene_id} 不在 5 纯 UI 闭集 {SCENES:?}"));
        }
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let Some(elements) = scene.get("elements").and_then(Value::as_array) else {
            errors.push(format!("场景 {scene_id} 缺 elements 数组"));
            continue;
        };
        for (index, comp) in elements.iter().enumerate() {
            let id = comp.get("id").and_then(Value::as_str).unwrap_or("");
            let loc = if id.is_empty() {
                format!("{scene_id}.#{index}")
            } else {
                format!("{scene_id}.{id}")
            };
            if !id_ok(id) {
                errors.push(format!("{loc}: id 非法（须 ^[a-z][a-z0-9_]*$，实得 {id:?}）"));
            }
            if !seen.insert(id.to_owned()) {
                errors.push(format!("{loc}: id 重复"));
            }
            validate_one_component(&loc, comp, &roles, &icons, &strings, &mut errors);
        }
    }
    errors
}

/// 单组件校验（抽离控 `validate_schema_value` 行数；判据与 `ui_schema_check.py` S3~S6 同义）。
fn validate_one_component(
    loc: &str,
    comp: &Value,
    roles: &BTreeSet<String>,
    icons: &BTreeSet<String>,
    strings: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let typ = comp.get("type").and_then(Value::as_str).unwrap_or("");
    if !is_type(typ) {
        errors.push(format!("{loc}: type {typ:?} 非法"));
    }
    // 禁绝对坐标：仅组件顶层（layout 内 offset_pct/size_pct 的 x/y/w/h 是百分比，合法）。
    let bad: Vec<&str> =
        FORBIDDEN_COORD.iter().filter(|key| comp.get(**key).is_some()).copied().collect();
    if !bad.is_empty() {
        errors.push(format!("{loc}: 出现绝对坐标字段 {bad:?}（违反不手写像素总纲）"));
    }
    let layout = comp.get("layout");
    let intent = layout.and_then(|l| l.get("intent")).and_then(Value::as_str).unwrap_or("");
    if !contains(&INTENTS, intent) {
        errors.push(format!("{loc}: layout.intent {intent:?} 非法"));
    }
    if let Some(anchor) = layout.and_then(|l| l.get("anchor")).and_then(Value::as_str) {
        if !contains(&ANCHORS, anchor) {
            errors.push(format!("{loc}: layout.anchor {anchor:?} 非法"));
        }
    }
    // 条件必填
    if contains(&TYPE_ROLE, typ)
        && comp.get("role").and_then(Value::as_str).unwrap_or("").is_empty()
    {
        errors.push(format!("{loc}: type={typ} 须 role"));
    }
    if contains(&TYPE_BINDING, typ)
        && comp.get("binding").and_then(Value::as_str).unwrap_or("").is_empty()
    {
        errors.push(format!("{loc}: type={typ} 须 binding"));
    }
    if contains(&TYPE_ACTION, typ)
        && comp.get("action").and_then(Value::as_str).unwrap_or("").is_empty()
    {
        errors.push(format!("{loc}: type={typ} 须 action"));
    }
    // 登记对账
    let role = comp.get("role").and_then(Value::as_str).unwrap_or("");
    if !role.is_empty() && !roles.is_empty() && !roles.contains(role) {
        errors.push(format!("{loc}: role {role:?} 未在 asset_roles.json 登记"));
    }
    let icon = comp.get("icon").and_then(Value::as_str).unwrap_or("");
    if !icon.is_empty() && !icons.is_empty() && !icons.contains(icon) {
        errors.push(format!("{loc}: icon {icon:?} 未在 ui_icons.json 登记"));
    }
    // action / i18n / tier / state
    let action = comp.get("action").and_then(Value::as_str).unwrap_or("");
    if !action.is_empty() && !contains(&ACTIONS, action) {
        errors.push(format!("{loc}: action {action:?} 非法"));
    }
    for field in ["binding", "text"] {
        let key = comp.get(field).and_then(Value::as_str).unwrap_or("");
        if key.is_empty() {
            continue;
        }
        let resolved = resolve_binding(key);
        if resolved.contains('.') && !strings.contains(&resolved) {
            errors.push(format!("{loc}: {field}={key:?} → {resolved:?} 在 strings.json 不存在"));
        }
    }
    let tier = comp.get("tier").and_then(Value::as_str).unwrap_or("");
    if !tier.is_empty() && !contains(&TIERS, tier) {
        errors.push(format!("{loc}: tier {tier:?} 非法"));
    }
    if let Some(states) = comp.get("state").and_then(Value::as_array) {
        for state in states {
            let state_name = state.as_str().unwrap_or("");
            if !contains(&STATES, state_name) {
                errors.push(format!("{loc}: state {state_name:?} 非法"));
            }
        }
    }
}

/// `type` 闭集判定（`TYPE_ROLE` ∪ `TYPE_BINDING`）。
fn is_type(value: &str) -> bool {
    contains(&TYPE_ROLE, value) || contains(&TYPE_BINDING, value)
}

/// 子进程只读门禁结果。
#[derive(Debug, Clone)]
struct ScriptCheck {
    ok: bool,
    exit_code: i32,
    summary: String,
}

/// 跑只读门禁脚本（`--check`）；退出码 0 = PASS，2 = SKIP（按 PASS 计），其余 = FAIL。
fn run_check_script(repo_root: &Path, relative: &str) -> ScriptCheck {
    let output = Command::new("python3")
        .arg(repo_root.join(relative))
        .arg("--check")
        .current_dir(repo_root)
        .output();
    let out = match output {
        Ok(o) => o,
        Err(e) => {
            return ScriptCheck {
                ok: false,
                exit_code: -1,
                summary: format!("spawn {relative} 失败：{e}"),
            };
        }
    };
    let code = out.status.code().unwrap_or(-1);
    let text =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let first = text
        .lines()
        .find(|l| l.contains("FAIL") || l.contains("ERROR") || l.contains("failure"))
        .unwrap_or_else(|| text.lines().next().unwrap_or(""));
    ScriptCheck {
        ok: code == 0 || code == 2,
        exit_code: code,
        summary: first.chars().take(400).collect(),
    }
}

/// **内存 schema** 实时校验结果（POST /validate 数据源）。
///
/// 三段判据：① Rust 镜像 enum/禁绝对坐标/登记对账；② `ui_schema_check.py --check`（on-disk
/// 基线，证明确有该门禁脚本且当前数据合法）；③ `check_scene_narrative.py --check`（叙事约束）。
#[must_use]
pub fn validate_components(repo_root: &Path, schema: &Value) -> Value {
    let mut errors = validate_schema_value(repo_root, schema);
    let schema_check = run_check_script(repo_root, SCHEMA_SCRIPT);
    let narrative_check = run_check_script(repo_root, NARRATIVE_SCRIPT);
    if !schema_check.ok {
        errors.push(format!(
            "ui_schema_check.py --check（on-disk 基线）FAIL：{}",
            schema_check.summary
        ));
    }
    if !narrative_check.ok {
        errors.push(format!("check_scene_narrative.py --check FAIL：{}", narrative_check.summary));
    }
    json!({
        "ok": errors.is_empty(),
        "errors": errors,
        "schema_check": {"ok": schema_check.ok, "exit_code": schema_check.exit_code, "summary": schema_check.summary},
        "narrative_check": {"ok": narrative_check.ok, "exit_code": narrative_check.exit_code, "summary": narrative_check.summary},
        "note": "内存 schema 不落盘；禁绝对坐标 + enum/登记对账 + 叙事门禁",
    })
}

/// 用 `json.dumps(indent=2, ensure_ascii=False)` 同口径规范化（键序保持不变，尾补换行）。
///
/// 前端传来的 `schema_text` 由 `JSON.stringify(schema, null, 2)` 生成，键序 = 原文件键序
/// ⇒ 不会整文件重排其它场景；Python 子进程只读 stdin/stdout，无临时文件落盘。
///
/// # Errors
/// `python3` 缺位 / stdin 写入失败 / `json.dumps` 非 0 退出 = `Err`（如实上报）。
fn canonicalize_indent2(raw_schema: &[u8]) -> Result<Vec<u8>, String> {
    const CODE: &str = "import sys,json;sys.stdout.write(json.dumps(json.load(sys.stdin),indent=2,ensure_ascii=False)+chr(10))";
    let mut child = Command::new("python3")
        .arg("-c")
        .arg(CODE)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("python3 spawn 失败：{e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(raw_schema).map_err(|e| format!("写 python stdin 失败：{e}"))?;
    }
    let out = child.wait_with_output().map_err(|e| format!("python3 wait 失败：{e}"))?;
    if !out.status.success() {
        return Err(format!(
            "json.dumps 规范化失败：{}",
            String::from_utf8_lossy(&out.stderr).chars().take(300).collect::<String>()
        ));
    }
    Ok(out.stdout)
}

/// `POST /api/ui/components/apply` 核心：校验 → 写点 → 写盘 → 门禁复验（FAIL 回滚）→ 写点探针。
///
/// 写前快照 + 写后快照逐文件比对，所有差异必须落在写点集 `components_v1.json` 内
/// （22 号 §8 北极星「0 游戏产物变更」）。
///
/// # Errors
/// 组件文件读/写失败 / 规范化子进程失败 = `Err`；校验 FAIL 走 `Ok({"ok":false,...})` 不写盘。
pub fn apply_components(
    repo_root: &Path,
    schema: &Value,
    raw_schema: &[u8],
) -> Result<Value, String> {
    let errors = validate_schema_value(repo_root, schema);
    if !errors.is_empty() {
        return Ok(
            json!({"ok": false, "errors": errors, "rolled_back": true, "note": "校验 FAIL，未写盘"}),
        );
    }
    let narrative = run_check_script(repo_root, NARRATIVE_SCRIPT);
    if !narrative.ok {
        return Ok(json!({
            "ok": false,
            "errors": [format!("check_scene_narrative.py --check FAIL：{}", narrative.summary)],
            "rolled_back": true,
            "note": "叙事门禁 FAIL，未写盘",
        }));
    }
    let path = repo_root.join(COMPONENTS_REL);
    let original = std::fs::read(&path).map_err(|e| format!("读 {COMPONENTS_REL} 失败：{e}"))?;
    let before = FrozenSnapshot::take(repo_root);
    let normalized = canonicalize_indent2(raw_schema)?;
    std::fs::write(&path, &normalized).map_err(|e| format!("写 {COMPONENTS_REL} 失败：{e}"))?;
    let after_write = run_check_script(repo_root, SCHEMA_SCRIPT);
    if !after_write.ok {
        let _ = std::fs::write(&path, &original);
        return Ok(json!({
            "ok": false,
            "errors": [format!("写盘后 ui_schema_check.py --check FAIL：{}", after_write.summary)],
            "rolled_back": true,
            "note": "已还原原文件",
        }));
    }
    let after = FrozenSnapshot::take(repo_root);
    let write_probe = before.write_probe(&after, &[COMPONENTS_REL]);
    Ok(json!({
        "ok": true,
        "path": COMPONENTS_REL,
        "bytes": normalized.len(),
        "write_probe": write_probe,
        "schema_check": {"ok": after_write.ok, "exit_code": after_write.exit_code, "summary": after_write.summary},
        "note": "indent=2 保序写 + ui_schema_check 复验 + write_probe（仅 components_v1.json 写点）",
    }))
}

/// 端点白名单 id 校验（`console_commands.json::api_endpoints` 数据面 ↔ 代码路由三面对账）。
///
/// 未登记 / method 或 path 不符 = `false`（POST handler 回 400，22 号 §4.2 禁自由字符串）。
pub fn endpoint_allowed(repo_root: &Path, id: &str, method: &str, path: &str) -> bool {
    let Ok(raw) = std::fs::read_to_string(repo_root.join(COMMANDS_REL)) else {
        return false;
    };
    let Ok(doc) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    doc.get("api_endpoints").and_then(Value::as_array).is_some_and(|entries| {
        entries.iter().any(|entry| {
            entry.get("id").and_then(Value::as_str) == Some(id)
                && entry.get("method").and_then(Value::as_str) == Some(method)
                && entry.get("path").and_then(Value::as_str) == Some(path)
        })
    })
}

// ---- HTTP 薄面（响应构造 + 写面门控在此；控 routes.rs 规模，镜像 themes.rs 范式） ----

/// 统一错误响应（`{"ok":false,"error":...}` + 状态码）。
#[must_use]
fn ui_error(msg: &str, status: &str, gz: bool) -> Vec<u8> {
    crate::routes::header_body(
        "application/json",
        format!(r#"{{"ok":false,"error":"{}"}}"#, msg.replace('"', "'")).as_bytes(),
        status,
        "no-store",
        None,
        gz,
    )
}

/// `GET /api/ui/components?scene=`（只读免 token）。
#[must_use]
pub fn components_get_http(repo_root: &Path, scene: &str, gz: bool) -> Vec<u8> {
    match components_get(repo_root, scene) {
        Ok(v) => crate::routes::json200(&v, gz),
        Err(e) => ui_error(&e, "500 Internal Server Error", gz),
    }
}

/// `GET /api/ui/roles`（只读免 token）。
#[must_use]
pub fn roles_get_http(repo_root: &Path, gz: bool) -> Vec<u8> {
    match roles_get(repo_root) {
        Ok(v) => crate::routes::json200(&v, gz),
        Err(e) => ui_error(&e, "500 Internal Server Error", gz),
    }
}

/// POST 请求体验证链（抽离控 handler 行数）：
/// 403 token → 400 Content-Type → 413 上限 → 400 JSON → 400 `enable_write` → 400 白名单 id
/// → 400 schema / `schema_text` → `(schema, 原始 schema 字节)`。
fn parse_ui_post(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    action: &str,
    path: &str,
    gz: bool,
) -> Result<(Value, Vec<u8>), Vec<u8>> {
    if !crate::routes::auth_ok(token) {
        return Err(ui_error("403: 写操作需 X-TI-Token（22 号 §10.1）", "403 Forbidden", gz));
    }
    let ct_ok = content_type.is_some_and(|c| {
        c.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("application/json")
    });
    if !ct_ok {
        return Err(ui_error(
            "400: Content-Type 必须为 application/json（22 号 §10.3）",
            "400 Bad Request",
            gz,
        ));
    }
    if body.len() > 65_536 {
        return Err(ui_error(
            "413: 请求体超 64 KiB 上限（22 号 §10.1）",
            "413 Payload Too Large",
            gz,
        ));
    }
    let payload: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            return Err(ui_error(&format!("400: body 非合法 JSON：{e}"), "400 Bad Request", gz));
        }
    };
    if payload.get("enable_write").and_then(Value::as_bool) != Some(true) {
        return Err(ui_error(
            "400: 写动作需 enable_write=true（30 号 §3.1 默认关）",
            "400 Bad Request",
            gz,
        ));
    }
    let sent_action = payload.get("action").and_then(Value::as_str).unwrap_or("");
    if sent_action != action || !endpoint_allowed(repo_root, sent_action, "POST", path) {
        return Err(ui_error(
            &format!(
                "400: action={sent_action:?} 未在 console_commands.json::api_endpoints 登记（22 号 §4.2）"
            ),
            "400 Bad Request",
            gz,
        ));
    }
    // schema_text = 前端 JSON.stringify 的原文（保键序；写盘规范化用）；schema 对象 = 校验用。
    if let Some(text) = payload.get("schema_text").and_then(Value::as_str) {
        let parsed: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                return Err(ui_error(
                    &format!("400: schema_text 非合法 JSON：{e}"),
                    "400 Bad Request",
                    gz,
                ));
            }
        };
        return Ok((parsed, text.as_bytes().to_vec()));
    }
    if let Some(obj) = payload.get("schema").filter(|v| v.is_object()) {
        let bytes = serde_json::to_vec(obj).map_err(|e| {
            ui_error(&format!("400: schema 序列化失败：{e}"), "400 Bad Request", gz)
        })?;
        return Ok((obj.clone(), bytes));
    }
    Err(ui_error("400: 缺 schema / schema_text（对象或 JSON 字符串）", "400 Bad Request", gz))
}

/// `POST /api/ui/components/validate`（只读 body 不落盘；写面头部门控照旧）。
#[must_use]
pub fn validate_http(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    gz: bool,
) -> Vec<u8> {
    let (schema, _) = match parse_ui_post(
        repo_root,
        body,
        token,
        content_type,
        "ui_components_validate",
        "/api/ui/components/validate",
        gz,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    crate::routes::json200(&validate_components(repo_root, &schema), gz)
}

/// `POST /api/ui/components/apply`（唯一写点 `components_v1.json`）。
#[must_use]
pub fn apply_http(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    gz: bool,
) -> Vec<u8> {
    let (schema, raw) = match parse_ui_post(
        repo_root,
        body,
        token,
        content_type,
        "ui_components_apply",
        "/api/ui/components/apply",
        gz,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match apply_components(repo_root, &schema, &raw) {
        Ok(v) => crate::routes::json200(&v, gz),
        Err(e) => ui_error(&e, "500 Internal Server Error", gz),
    }
}
