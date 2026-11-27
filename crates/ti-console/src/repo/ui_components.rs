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
//! **S87.5b 世界层扩面**：`?mode=world`（GET）/ body `mode:"world"`（POST）复用同 4 端点，
//! 数据源切 `world_components.json`（`main_map/vs` 静态元素）；校验/写盘逻辑在同目录子模块
//! [`world`]（守住 900 行文件门禁）；合法 `world_rect` 拖拽与 `geometry` 引用白名单均受
//! `scripts/world_schema_check.py --check` 终判，FAIL 点名 + 回滚（FAIL 禁摆）。
//!
//! 本模块 **0 游戏依赖**（不引 ti-core/ti-engine；沿用 `themes.rs` 本地 `std::fs` 范式）；
//! 0 写盘面除 `apply_components` 的两个受控写点 `components_v1.json` / `world_components.json`
//! （按 `mode` 二选一，写前 `enable_write` + 白名单 action id 门控）。

/// S87.5b 世界层（main_map/vs）可视化编辑数据面（见 `world.rs` 子模块）。
mod world;

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
const MANIFEST_REL: &str = "assets/ui_manifest.json";
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

/// S87.5b 世界层唯一写点（只读源 + world 模式 apply 写点）。
pub(super) const WORLD_COMPONENTS_REL: &str = "assets/data/ui_schema/world_components.json";

/// 数组成员判定（与 Python `in set` 同义；子模块共用）。
pub(super) fn contains(pool: &[&str], value: &str) -> bool {
    pool.contains(&value)
}

/// id 合法性：`^[a-z][a-z0-9_]*$`（子模块共用，不引 regex crate）。
pub(super) fn id_ok(id: &str) -> bool {
    let Some(first) = id.chars().next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    id.chars().skip(1).all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// 读取并解析仓库 JSON（缺文件/非法 JSON = `Err`，如实上报不静默；子模块共用）。
pub(super) fn read_json(repo_root: &Path, relative: &str) -> Result<Value, String> {
    let path = repo_root.join(relative);
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("读 {relative} 失败：{e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("{relative} 非法 JSON：{e}"))
}

/// `GET /api/ui/components`：全场景实例 + `scene` 回显（0 写盘）。
///
/// 返回 `{ok, schema_version, schema, scene, mode, count, scene_counts, geometry_rects}`：
/// - `mode=ui`（默认）= `components_v1.json`（5 纯 UI 场景，intent/anchored 百分比）；
/// - `mode=world` = `world_components.json`（`main_map/vs` 静态层，`world_rect` + `geometry.ref`）。
///   `count` 在给出 `scene` 时 = 该场景元素数（世界层按场景计数：`main_map`=35 / `vs`=21）；
///   `geometry_rects` = 后端解析 geometry 引用得到的矩形（示意画布只读呈现，禁散落裸像素）。
///
/// # Errors
/// 对应 schema 文件读失败 / 非法 JSON = `Err`（如实上报，不静默）。
pub fn components_get(repo_root: &Path, scene: &str, mode: &str) -> Result<Value, String> {
    let is_world = mode == "world";
    let path = if is_world { WORLD_COMPONENTS_REL } else { COMPONENTS_REL };
    let schema = read_json(repo_root, path)?;
    let mut total = 0usize;
    let mut scene_counts = serde_json::Map::new();
    let mut scene_count = 0usize;
    if let Some(scenes) = schema.get("scenes").and_then(Value::as_object) {
        for (key, value) in scenes {
            let n = value.get("elements").and_then(Value::as_array).map_or(0, Vec::len);
            total += n;
            scene_counts.insert(key.clone(), json!(n));
            if key == scene {
                scene_count = n;
            }
        }
    }
    let count = if scene.is_empty() { total } else { scene_count };
    let geometry_rects =
        if is_world { world::geometry_rects(repo_root, &schema) } else { json!({}) };
    Ok(json!({
        "ok": true,
        "schema_version": schema.get("schema_version").cloned().unwrap_or(Value::Null),
        "schema": schema,
        "scene": scene,
        "mode": mode,
        "count": count,
        "scene_counts": scene_counts,
        "geometry_rects": geometry_rects,
        "note": if is_world {
            "S87.5b 世界层可视化编辑（world_rect 整数 1920×1080 / geometry.ref 两源白名单；FAIL 禁摆）"
        } else {
            "S86.2 组件可视化编辑器只读源；layout 禁绝对坐标（intent/anchored 百分比）"
        },
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
/// UI 模式三段判据：① Rust 镜像 enum/禁绝对坐标/登记对账；② `ui_schema_check.py --check`
/// （on-disk 基线）；③ `check_scene_narrative.py --check`（叙事约束）。
/// world 模式两段判据：① Rust 镜像 W1~W6（点名非法 type / 裸坐标 / 未登记 role / 非法 action）；
/// ② `world_schema_check.py --check --instance <内存数据落 gitignored 临时文件>`（权威门禁
/// 直接判本次数据，非仅 on-disk 基线）；临时文件用完即删，0 跟踪文件写盘。
#[must_use]
pub fn validate_components(repo_root: &Path, schema: &Value, world: bool) -> Value {
    if world {
        return world::validate(repo_root, schema);
    }
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
pub(super) fn canonicalize_indent2(raw_schema: &[u8]) -> Result<Vec<u8>, String> {
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
/// 写前快照 + 写后快照逐文件比对，所有差异必须落在写点集内（22 号 §8「0 游戏产物变更」）。
/// `world=true` 时切到 `world.rs::apply`（写点 `world_components.json`，同一门控口径）。
///
/// # Errors
/// 组件文件读/写失败 / 规范化子进程失败 = `Err`；校验 FAIL 走 `Ok({"ok":false,...})` 不写盘。
/// S88.3：上传新图（PNG 二进制）→ 受控写点 `assets/ui/<name>.png` + `ui_manifest` 登记。
/// 安全：name 白名单 `[a-z][a-z0-9_]*`（防穿越）；PNG magic 校验；目标已存在 = 拒（防覆盖）。
/// name/role 从 query 传入（serve 解析），body = 纯 PNG 二进制（无 base64 膨胀）。
#[must_use]
pub fn handle_asset_upload(repo_root: &Path, body: &[u8], name: Option<&str>, gz: bool) -> Vec<u8> {
    // PNG magic（8 字节）
    const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
    let Some(name) = name else {
        return ui_error("400: 缺 ?name=<asset_name>", "400 Bad Request", gz);
    };
    if !id_ok(name) {
        return ui_error("400: name 须 ^[a-z][a-z0-9_]*$（防路径穿越）", "400 Bad Request", gz);
    }
    if body.len() < 64 || !body.starts_with(PNG_MAGIC) {
        return ui_error("400: body 须合法 PNG（magic 校验失败）", "400 Bad Request", gz);
    }
    let rel = format!("assets/ui/{name}.png");
    let dst = repo_root.join(&rel);
    if dst.exists() {
        return ui_error("400: 资产已存在（禁覆盖，换名或先删）", "400 Bad Request", gz);
    }
    // 写盘 + ui_manifest 登记（保序 indent=2，同 save_manifest 口径）
    if std::fs::write(&dst, body).is_err() {
        return ui_error("500: 写盘失败", "500 Internal Server Error", gz);
    }
    let manifest_path = repo_root.join(MANIFEST_REL);
    let result = (|| -> Result<Vec<u8>, String> {
        let raw = std::fs::read(&manifest_path).map_err(|e| format!("读 ui_manifest 失败：{e}"))?;
        let mut doc: Value = serde_json::from_slice(&raw).map_err(|e| format!("解析失败：{e}"))?;
        let default_role = doc
            .get("roles")
            .and_then(Value::as_object)
            .and_then(|o| o.keys().next())
            .cloned()
            .unwrap_or_else(|| "background".to_owned());
        let assets = doc
            .get_mut("assets")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| "ui_manifest 缺 assets 数组".to_owned())?;
        let entry = json!({
            "name": name,
            "file": rel,
            "role": default_role,
            "params": {},
            "generated_at": chrono_like_today(),
        });
        assets.push(entry);
        let normalized =
            canonicalize_indent2(&serde_json::to_vec(&doc).map_err(|e| e.to_string())?)?;
        std::fs::write(&manifest_path, &normalized)
            .map_err(|e| format!("写 ui_manifest 失败：{e}"))?;
        Ok(normalized)
    })();
    match result {
        Ok(_) => crate::routes::json200(
            &json!({"ok":true,"name":name,"path":rel,"bytes":body.len(),
            "note":"上传登记：PNG 落盘 + ui_manifest.assets 登记（保序 indent=2）"}),
            gz,
        ),
        Err(e) => {
            let _ = std::fs::remove_file(&dst); // 登记失败回滚文件
            ui_error(&format!("500: {e}"), "500 Internal Server Error", gz)
        }
    }
}

/// 上传登记的 `generated_at`（今日 `YYYY-MM-DD`，无 chrono 依赖：从系统时间取 date 部分）。
fn chrono_like_today() -> String {
    // 用 `date` 命令避免新依赖；失败回退空串（非关键字段）。
    std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default()
}

/// S88.1：换图写点——写 `asset_roles.json` 的 `role → path` 映射（UI 资料更新，业务逻辑零改动）。
/// 校验：① path 须在 `ui_manifest.assets` 登记（禁手写路径）② role 名合法 ③ 文件存在；
/// 写后跑 `ui_schema_check.py --check` 复验，FAIL 回滚（与 `apply_components` 同范式）。
/// 换图写点：写 `asset_roles.json` 的 `role → path` 映射。
///
/// # Errors
/// 读/写文件或门禁脚本异常时返回 `Err`。
pub fn apply_asset_roles(
    repo_root: &Path,
    roles_value: &Value,
    raw: &[u8],
) -> Result<Value, String> {
    let roles_obj = roles_value
        .get("roles")
        .and_then(Value::as_object)
        .ok_or_else(|| "asset_roles.json 缺 roles 对象".to_owned())?;
    // 资产池 = ui_manifest.assets[].file（单一事实源，禁手写路径）
    let manifest =
        read_json(repo_root, MANIFEST_REL).map_err(|e| format!("读 ui_manifest 失败：{e}"))?;
    let registered: std::collections::BTreeSet<String> = manifest
        .get("assets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|a| a.get("file").and_then(Value::as_str).map(str::to_owned))
        .collect();
    let mut errors = Vec::new();
    for (role, path) in roles_obj {
        let Some(path) = path.as_str() else {
            errors.push(format!("role {role}: path 非字符串"));
            continue;
        };
        if !id_ok(role) {
            errors.push(format!("role {role}: 名字非法"));
        }
        if !registered.contains(path) {
            errors.push(format!("role {role}: path {path:?} 未在 ui_manifest 登记（禁手写路径）"));
        }
        if !repo_root.join(path).is_file() {
            errors.push(format!("role {role}: 文件 {path:?} 不存在"));
        }
    }
    if !errors.is_empty() {
        return Ok(
            json!({"ok": false, "errors": errors, "rolled_back": true, "note": "换图校验 FAIL，未写盘"}),
        );
    }
    let path = repo_root.join(ROLES_REL);
    let original = std::fs::read(&path).map_err(|e| format!("读 {ROLES_REL} 失败：{e}"))?;
    let before = FrozenSnapshot::take(repo_root);
    let normalized = canonicalize_indent2(raw)?;
    std::fs::write(&path, &normalized).map_err(|e| format!("写 {ROLES_REL} 失败：{e}"))?;
    let after_write = run_check_script(repo_root, SCHEMA_SCRIPT);
    if !after_write.ok {
        let _ = std::fs::write(&path, &original);
        return Ok(
            json!({"ok": false, "errors": [format!("写盘后 ui_schema_check.py FAIL：{}", after_write.summary)], "rolled_back": true, "note": "已还原原文件"}),
        );
    }
    let after = FrozenSnapshot::take(repo_root);
    let write_probe = before.write_probe(&after, &[ROLES_REL]);
    Ok(
        json!({"ok": true, "path": ROLES_REL, "bytes": normalized.len(), "write_probe": write_probe, "schema_check": {"ok": true, "exit_code": after_write.exit_code}, "note": "换图写点：role→path 映射 + ui_manifest 登记校验 + 写后门禁"}),
    )
}

/// `POST /api/ui/components/apply` 主写点（`components_v1.json` / `world_components.json`）。
///
/// # Errors
/// 读/写文件或门禁脚本异常时返回 `Err`，由 handler 映射为 500。
pub fn apply_components(
    repo_root: &Path,
    schema: &Value,
    raw_schema: &[u8],
    world: bool,
) -> Result<Value, String> {
    if world {
        return world::apply(repo_root, schema, raw_schema);
    }
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

/// `GET /api/ui/components?scene=&mode=`（只读；mode=world 切世界层数据源）。
#[must_use]
pub fn components_get_http(repo_root: &Path, scene: &str, mode: &str, gz: bool) -> Vec<u8> {
    match components_get(repo_root, scene, mode) {
        Ok(v) => crate::routes::json200(&v, gz),
        Err(e) => ui_error(&e, "500 Internal Server Error", gz),
    }
}

/// `GET /api/ui/roles`（只读）。
#[must_use]
pub fn roles_get_http(repo_root: &Path, gz: bool) -> Vec<u8> {
    match roles_get(repo_root) {
        Ok(v) => crate::routes::json200(&v, gz),
        Err(e) => ui_error(&e, "500 Internal Server Error", gz),
    }
}

/// POST 请求体验证链（抽离控 handler 行数）：
/// 400 Content-Type → 413 上限 → 400 JSON → 400 `enable_write` → 400 白名单 id
/// → 400 schema / `schema_text` → `(schema, 原始 schema 字节, world 模式)`。
fn parse_ui_post(
    repo_root: &Path,
    body: &[u8],
    content_type: Option<&str>,
    action: &str,
    path: &str,
    gz: bool,
    require_write: bool,
) -> Result<(Value, Vec<u8>, bool), Vec<u8>> {
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
    // world 模式复用同 4 端点 / 同白名单 action id，仅按 body `mode` 切数据源与写点。
    let world = payload.get("mode").and_then(Value::as_str) == Some("world");
    // S87.10：写面开关只管**写操作**。validate 是只读校验（前端拖动/resize/改属性
    // 每步都走它），不该被写面默认关挡住——这是读写职责不统一的修复。
    if require_write && payload.get("enable_write").and_then(Value::as_bool) != Some(true) {
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
        return Ok((parsed, text.as_bytes().to_vec(), world));
    }
    if let Some(obj) = payload.get("schema").filter(|v| v.is_object()) {
        let bytes = serde_json::to_vec(obj).map_err(|e| {
            ui_error(&format!("400: schema 序列化失败：{e}"), "400 Bad Request", gz)
        })?;
        return Ok((obj.clone(), bytes, world));
    }
    Err(ui_error("400: 缺 schema / schema_text（对象或 JSON 字符串）", "400 Bad Request", gz))
}

/// `POST /api/ui/components/validate`（只读 body 不落盘；`enable_write` 门控照旧）。
#[must_use]
pub fn validate_http(
    repo_root: &Path,
    body: &[u8],
    content_type: Option<&str>,
    gz: bool,
) -> Vec<u8> {
    let (schema, _, world) = match parse_ui_post(
        repo_root,
        body,
        content_type,
        "ui_components_validate",
        "/api/ui/components/validate",
        gz,
        false, // validate 只读校验，不要求 enable_write
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    crate::routes::json200(&validate_components(repo_root, &schema, world), gz)
}

/// `POST /api/ui/components/apply`（唯一写点 `components_v1.json`）。
#[must_use]
pub fn apply_http(repo_root: &Path, body: &[u8], content_type: Option<&str>, gz: bool) -> Vec<u8> {
    // S88.1：换图分支——body 带 asset_roles_json = 写 role→path 映射（UI 资料更新）；
    // 否则走原 components/world 写点。前置校验与 parse_ui_post 同口径（内容类型/写权限/action）。
    let asset_roles =
        serde_json::from_slice::<Value>(body).ok().and_then(|v| v.get("asset_roles_json").cloned());
    if let Some(roles_val) = asset_roles {
        let ct_ok = content_type.is_some_and(|c| {
            c.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("application/json")
        });
        let Ok(payload) = serde_json::from_slice::<Value>(body) else {
            return ui_error("400: body 非合法 JSON", "400 Bad Request", gz);
        };
        let write_ok = payload.get("enable_write").and_then(Value::as_bool) == Some(true);
        let action_ok = payload.get("action").and_then(Value::as_str).is_some_and(|a| {
            a == "ui_components_apply"
                && endpoint_allowed(repo_root, a, "POST", "/api/ui/components/apply")
        });
        if !ct_ok {
            return ui_error("400: Content-Type 必须为 application/json", "400 Bad Request", gz);
        }
        if !write_ok {
            return ui_error(
                "400: 写动作需 enable_write=true（30 号 §3.1 默认关）",
                "400 Bad Request",
                gz,
            );
        }
        if !action_ok {
            return ui_error("400: action 未登记（22 号 §4.2）", "400 Bad Request", gz);
        }
        let raw = serde_json::to_vec(&roles_val).unwrap_or_default();
        return match apply_asset_roles(repo_root, &roles_val, &raw) {
            Ok(v) => crate::routes::json200(&v, gz),
            Err(e) => ui_error(&e, "500 Internal Server Error", gz),
        };
    }
    let (schema, raw, world) = match parse_ui_post(
        repo_root,
        body,
        content_type,
        "ui_components_apply",
        "/api/ui/components/apply",
        gz,
        true, // apply 是唯一写点，要求 enable_write
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match apply_components(repo_root, &schema, &raw, world) {
        Ok(v) => crate::routes::json200(&v, gz),
        Err(e) => ui_error(&e, "500 Internal Server Error", gz),
    }
}
