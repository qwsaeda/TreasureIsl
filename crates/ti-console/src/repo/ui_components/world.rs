//! S87.5b 世界层（main_map/vs）可视化编辑数据面（22 号 §13 扩面）：
//!
//! - `validate_value`：Rust 镜像 `scripts/world_schema_check.py` W1~W6（非法 type / 裸坐标 /
//!   未登记 role / 非法 action / geometry.ref 不存在 → FAIL 点名 `场景.元素 id`）；
//! - `script_check`：把内存数据落 gitignored `evidence/console/world_validate_<pid>_<seq>/`
//!   临时文件后跑权威门禁 `--check --instance <path>`，用完即删（0 跟踪文件写盘）；
//! - `geometry_rects`：解析两源 geometry 引用 → 矩形，供示意画布只读呈现；
//! - `apply`：indent=2 保序写 `world_components.json` + 写后门禁复验（FAIL 还原）+ write_probe。
//!
//! 0 游戏依赖（沿用父模块本地 `std::fs` 范式）；与 UI 模式复用同 4 端点 /
//! enable_write / 白名单 action id，仅按 body `mode:"world"` 切数据源与写点。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Map, Value, json};

use super::{WORLD_COMPONENTS_REL, canonicalize_indent2, contains, id_ok, read_json};
use crate::repo::snapshot::FrozenSnapshot;

/// 世界层场景闭集（`world_components.schema.json::scene_id` 镜像）。
const SCENE_IDS: [&str; 2] = ["main_map", "vs"];
/// CPU `DrawOp` 三型（禁小写 UI 型 / grad/glow/panel）。
const TYPES: [&str; 3] = ["Rect", "Text", "Image"];
/// 元素顶层禁止字段（合法锚点只能是 `world_rect` 或 `geometry.ref`）。
const FORBIDDEN_COORD: [&str; 7] = ["x", "y", "w", "h", "rect", "layout", "intent"];
/// `UiAction` 15 项闭集（`components_v1.schema.json` / `world_components.schema.json` 同源）。
const ACTIONS: [&str; 15] = [
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
    "vs_skip",
    "vs_confirm",
    "vs_reset",
];
/// geometry 两源闭集。
const SOURCES: [&str; 2] = ["main_map_v6_layout_landscape.json", "scenes.json"];
/// `&dyn SceneHost` 白名单 path（`binding.path` / `visible_when.path` 共用）。
const HOST_PATHS: [&str; 14] = [
    "EconomyHost.gold",
    "EconomyHost.cargo",
    "MapHost.dice",
    "MapHost.seat",
    "MapHost.panel",
    "MapHost.handoff_pending",
    "BattleHost.hp_player",
    "BattleHost.hp_enemy",
    "BattleHost.crew",
    "BattleHost.round",
    "BattleHost.report",
    "BattleHost.phase",
    "ToastHost.text",
    "OutcomeHost.text",
];
/// `visible_when.op` 闭集。
const OPS: [&str; 4] = ["is_none", "is_some", "eq", "ne"];
const TIERS: [&str; 3] = ["title", "body", "hint"];
const STATES: [&str; 4] = ["normal", "selected", "hover", "disabled"];
/// 世界层门禁脚本（子进程，权威终判）。
const CHECK_SCRIPT: &str = "scripts/world_schema_check.py";
const SCREEN_W: i64 = 1920;
const SCREEN_H: i64 = 1080;

/// 临时数据目录序号（同进程多次 validate 不互相覆盖）。
static VALIDATE_SEQ: AtomicU64 = AtomicU64::new(0);

/// 只读数据面（role→资产 / i18n keys / geometry 两源 / 仓库根）。
struct WorldCtx {
    roles: BTreeMap<String, String>,
    strings: BTreeSet<String>,
    sources: BTreeMap<String, Value>,
    repo_root: PathBuf,
}

fn new_ctx(repo_root: &Path) -> WorldCtx {
    let roles = read_json(repo_root, "assets/data/ui_schema/asset_roles.json")
        .ok()
        .and_then(|v| v.get("roles").and_then(Value::as_object).cloned())
        .map(|obj| {
            obj.iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|path| (key.clone(), path.to_owned()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let strings = read_json(repo_root, "assets/data/strings.json")
        .ok()
        .and_then(|v| v.as_object().cloned())
        .map(|obj| {
            obj.keys()
                .filter(|key| key.as_str() != "schema_version")
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut sources = BTreeMap::new();
    for (name, relative) in [
        (SOURCES[0], "assets/data/main_map_v6_layout_landscape.json"),
        (SOURCES[1], "assets/data/scenes.json"),
    ] {
        if let Ok(value) = read_json(repo_root, relative) {
            sources.insert(name.to_owned(), value);
        }
    }
    WorldCtx { roles, strings, sources, repo_root: repo_root.to_path_buf() }
}

/// `geometry.ref` 解析：dot 路径 + `[n]` 下标（`[*]` 仅用于门禁存在性，矩形解析取不到即回退）。
fn ref_node<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
    let mut node = root;
    for segment in reference.split('.') {
        let split = segment.find('[').unwrap_or(segment.len());
        node = node.get(&segment[..split])?;
        for token in segment[split..].split(['[', ']']).filter(|part| !part.is_empty()) {
            // `[*]` = 通配下标（权威门禁同样按「数组非空」判定存在），不做数值索引。
            if token == "*" {
                let empty = match node.as_array() {
                    Some(items) => items.is_empty(),
                    None => true,
                };
                if empty {
                    return None;
                }
                continue;
            }
            node = node.get(token.parse::<usize>().ok()?)?;
        }
    }
    Some(node)
}

/// 元素矩形：`world_rect` 直用；geometry 两源解析（v6 中心+宽 / scenes 矩形），供示意画布。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn rect_of(ctx: &WorldCtx, elem: &Value) -> Option<Value> {
    if let Some(rect) = elem.get("world_rect") {
        return Some(rect.clone());
    }
    let geo = elem.get("geometry")?;
    let source = geo.get("source")?.as_str()?;
    let reference = geo.get("ref")?.as_str()?;
    let root = ctx.sources.get(source)?;
    if source == "scenes.json" {
        let (scene_id, key) = reference.split_once('.')?;
        let rect = root.get(scene_id)?.get(key)?;
        return Some(json!({
            "x": rect.get("x")?, "y": rect.get("y")?, "w": rect.get("w")?, "h": rect.get("h")?
        }));
    }
    let node = ref_node(root, reference)?;
    let cx = node.get("cx")?.as_f64()?;
    let cy = node.get("cy")?.as_f64()?;
    let side = node
        .get("w")
        .and_then(Value::as_f64)
        .or_else(|| root.get("tile_r").and_then(Value::as_f64).map(|radius| radius * 2.0))?;
    let half = side / 2.0;
    Some(json!({
        "x": (cx - half).round() as i32,
        "y": (cy - half).round() as i32,
        "w": side.round() as i32,
        "h": side.round() as i32,
    }))
}

/// `GET ?mode=world` 附带：`"场景.元素 id" → world_rect`（示意画布只读呈现 geometry 元素）。
#[must_use]
pub fn geometry_rects(repo_root: &Path, schema: &Value) -> Value {
    let ctx = new_ctx(repo_root);
    let mut out = Map::new();
    if let Some(scenes) = schema.get("scenes").and_then(Value::as_object) {
        for (scene_id, scene) in scenes {
            if let Some(elements) = scene.get("elements").and_then(Value::as_array) {
                for elem in elements {
                    let Some(id) = elem.get("id").and_then(Value::as_str) else { continue };
                    if let Some(rect) = rect_of(&ctx, elem) {
                        out.insert(format!("{scene_id}.{id}"), rect);
                    }
                }
            }
        }
    }
    Value::Object(out)
}

/// W1~W6 内存镜像：返回违规清单（空 = PASS），每条点名 `场景.元素 id`。
#[must_use]
pub fn validate_value(repo_root: &Path, schema: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if schema.get("schema_version").and_then(Value::as_i64) != Some(1) {
        errors.push(format!("schema_version 应为 1，实得 {:?}", schema.get("schema_version")));
    }
    let ctx = new_ctx(repo_root);
    let Some(scenes) = schema.get("scenes").and_then(Value::as_object) else {
        errors.push("world: 缺 scenes 对象".to_owned());
        return errors;
    };
    for (scene_id, scene) in scenes {
        if !contains(&SCENE_IDS, scene_id) {
            errors.push(format!("world: 场景 {scene_id} 不在闭集 {SCENE_IDS:?}"));
        }
        if scene.get("scene_id").and_then(Value::as_str) != Some(scene_id.as_str()) {
            errors.push(format!("world: 场景 {scene_id} 的 scene_id 与键名不一致"));
        }
        let Some(elements) = scene.get("elements").and_then(Value::as_array) else {
            errors.push(format!("world: 场景 {scene_id} 缺 elements 数组"));
            continue;
        };
        let mut seen = BTreeSet::new();
        for (index, elem) in elements.iter().enumerate() {
            let id = elem.get("id").and_then(Value::as_str).unwrap_or("");
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
            validate_element(&ctx, &loc, elem, &mut errors);
        }
    }
    errors
}

/// 单元素核心校验：type / 裸坐标 / `world_rect` / 锚点 / z / role / pick（W2~W4 + 部分 W5）。
fn validate_element(ctx: &WorldCtx, loc: &str, elem: &Value, errors: &mut Vec<String>) {
    let typ = elem.get("type").and_then(Value::as_str).unwrap_or("");
    if !contains(&TYPES, typ) {
        errors.push(format!("{loc}: type {typ:?} 非法（世界层 CPU DrawOp 三型 Rect/Text/Image）"));
    }
    let bad: Vec<&str> =
        FORBIDDEN_COORD.iter().filter(|key| elem.get(**key).is_some()).copied().collect();
    if !bad.is_empty() {
        errors.push(format!(
            "{loc}: 元素顶层出现裸坐标/布局字段 {bad:?}（只能写在 world_rect / geometry.ref）"
        ));
    }
    if let Some(rect) = elem.get("world_rect") {
        if !rect.is_object()
            || rect.as_object().is_some_and(|obj| {
                obj.len() != 4 || ["x", "y", "w", "h"].iter().any(|key| rect.get(key).is_none())
            })
        {
            errors.push(format!("{loc}: world_rect 须恰含 x/y/w/h 四键"));
        } else {
            for (key, limit) in [("x", SCREEN_W), ("y", SCREEN_H), ("w", SCREEN_W), ("h", SCREEN_H)]
            {
                let value = rect.get(key).and_then(Value::as_i64);
                if !matches!(value, Some(number) if (0..=limit).contains(&number)) {
                    errors
                        .push(format!("{loc}: world_rect.{key}={value:?} 非法（整数 0..{limit}）"));
                }
            }
        }
    }
    if elem.get("geometry").is_none() && elem.get("world_rect").is_none() {
        errors.push(format!("{loc}: 缺 geometry / world_rect 锚点"));
    }
    if let Some(z) = elem.get("z") {
        if z.as_i64().is_none() {
            errors.push(format!("{loc}: z {z} 非整数"));
        }
    }
    let role = elem.get("role").and_then(Value::as_str).unwrap_or("");
    if !role.is_empty() {
        if !id_ok(role) {
            errors.push(format!("{loc}: role {role:?} 非法（须 ^[a-z][a-z0-9_]*$）"));
        } else if let Some(asset) = ctx.roles.get(role) {
            if !ctx.repo_root.join(asset).is_file() {
                errors.push(format!("{loc}: role {role:?} 资产 '{asset}' 文件不存在"));
            }
        } else {
            errors.push(format!("{loc}: role {role:?} 未在 asset_roles.json 登记"));
        }
    }
    if let Some(pick) = elem.get("pick") {
        if !pick.is_object() || pick.get("action").is_none() || pick.get("index").is_none() {
            errors.push(format!("{loc}: pick 须恰含 action/index 两键"));
        } else {
            let action = pick.get("action").and_then(Value::as_str).unwrap_or("");
            if !contains(&ACTIONS, action) {
                errors.push(format!("{loc}: pick.action {action:?} 非法（UiAction 15 项闭集）"));
            }
            if !matches!(pick.get("index").and_then(Value::as_i64), Some(number) if number >= 0) {
                errors.push(format!("{loc}: pick.index 非法（须非负整数）"));
            }
        }
    }
    validate_attrs(ctx, loc, elem, errors);
}

/// 属性校验：binding / tier / state / `visible_when` / geometry（W5~W6）。
fn validate_attrs(ctx: &WorldCtx, loc: &str, elem: &Value, errors: &mut Vec<String>) {
    if let Some(binding) = elem.get("binding") {
        if let Some(key) = binding.as_str() {
            if key.contains('.') && !ctx.strings.contains(key) {
                errors.push(format!("{loc}: binding '{key}' 在 strings.json 不存在"));
            }
        } else if let Some(obj) = binding.as_object() {
            if obj.get("kind").and_then(Value::as_str) != Some("host") {
                errors.push(format!("{loc}: binding.kind 非法（只支持 host）"));
            }
            let path = obj.get("path").and_then(Value::as_str).unwrap_or("");
            if !contains(&HOST_PATHS, path) {
                errors.push(format!("{loc}: binding.path {path:?} 不在 SceneHost 白名单"));
            }
        } else {
            errors.push(format!("{loc}: binding 须 i18n 字符串或 host 对象"));
        }
    }
    if let Some(tier) = elem.get("tier").and_then(Value::as_str) {
        if !contains(&TIERS, tier) {
            errors.push(format!("{loc}: tier {tier:?} 非法"));
        }
    }
    if let Some(states) = elem.get("state").and_then(Value::as_array) {
        for state in states {
            let name = state.as_str().unwrap_or("");
            if !contains(&STATES, name) {
                errors.push(format!("{loc}: state {name:?} 非法"));
            }
        }
    }
    if let Some(when) = elem.get("visible_when") {
        if when.is_object() {
            let path = when.get("path").and_then(Value::as_str).unwrap_or("");
            if !contains(&HOST_PATHS, path) {
                errors.push(format!("{loc}: visible_when.path {path:?} 不在白名单"));
            }
            let op = when.get("op").and_then(Value::as_str).unwrap_or("");
            if !contains(&OPS, op) {
                errors.push(format!("{loc}: visible_when.op {op:?} 非法"));
            }
            if matches!(op, "eq" | "ne") && when.get("value").is_none() {
                errors.push(format!("{loc}: visible_when.op={op} 须带 value"));
            }
        } else {
            errors.push(format!("{loc}: visible_when 须为对象"));
        }
    }
    if let Some(geo) = elem.get("geometry") {
        if !geo.is_object() || geo.get("source").is_none() || geo.get("ref").is_none() {
            errors.push(format!("{loc}: geometry 须恰含 source/ref 两键"));
        } else {
            let source = geo.get("source").and_then(Value::as_str).unwrap_or("");
            if contains(&SOURCES, source) {
                let reference = geo.get("ref").and_then(Value::as_str).unwrap_or("");
                let hit =
                    ctx.sources.get(source).is_some_and(|root| ref_node(root, reference).is_some());
                if !hit {
                    errors.push(format!("{loc}: geometry.ref '{reference}' 在 {source} 不存在"));
                }
            } else {
                errors.push(format!("{loc}: geometry.source {source:?} 非法（只许两源）"));
            }
        }
    }
}

/// 子进程权威门禁结果（world 模式 validate / apply 共用）。
#[derive(Debug, Clone)]
pub struct WorldCheck {
    pub ok: bool,
    pub exit_code: i32,
    pub summary: String,
}

/// 把内存数据落 gitignored 临时文件后跑权威门禁 `--check --instance <path>`，用完即删。
fn script_check(repo_root: &Path, schema: &Value) -> WorldCheck {
    let seq = VALIDATE_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = repo_root.join("evidence/console").join(format!(
        "world_validate_{}_{}",
        std::process::id(),
        seq
    ));
    let result = (|| -> Result<WorldCheck, String> {
        std::fs::create_dir_all(&dir).map_err(|e| format!("建临时校验目录失败：{e}"))?;
        let text = serde_json::to_string_pretty(schema).map_err(|e| format!("序列化失败：{e}"))?;
        std::fs::write(dir.join("world_components.json"), format!("{text}\n"))
            .map_err(|e| format!("写临时校验文件失败：{e}"))?;
        let output = Command::new("python3")
            .arg(repo_root.join(CHECK_SCRIPT))
            .arg("--check")
            .arg("--instance")
            .arg(dir.join("world_components.json"))
            .current_dir(repo_root)
            .output()
            .map_err(|e| format!("spawn {CHECK_SCRIPT} 失败：{e}"))?;
        let code = output.status.code().unwrap_or(-1);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let first = text
            .lines()
            .find(|line| {
                line.contains("FAIL") || line.contains("ERROR") || line.contains("failure")
            })
            .unwrap_or_else(|| text.lines().next().unwrap_or(""));
        Ok(WorldCheck {
            ok: code == 0,
            exit_code: code,
            summary: first.chars().take(400).collect(),
        })
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result.unwrap_or_else(|error| WorldCheck { ok: false, exit_code: -1, summary: error })
}

/// world 模式完整校验结果（POST /validate 数据源）。
#[must_use]
pub fn validate(repo_root: &Path, schema: &Value) -> Value {
    let mut errors = validate_value(repo_root, schema);
    let check = script_check(repo_root, schema);
    if !check.ok {
        errors.push(format!("world_schema_check.py --check FAIL：{}", check.summary));
    }
    json!({
        "ok": errors.is_empty(),
        "errors": errors,
        "world_check": {"ok": check.ok, "exit_code": check.exit_code, "summary": check.summary},
        "mode": "world",
        "note": "W1~W6 内存镜像 + world_schema_check.py --check（内存数据落 gitignored 临时文件，用完即删）",
    })
}

/// world 模式唯一写点：indent=2 保序写 `world_components.json` + 门禁复验（FAIL 还原）。
pub fn apply(repo_root: &Path, schema: &Value, raw_schema: &[u8]) -> Result<Value, String> {
    let errors = validate_value(repo_root, schema);
    if !errors.is_empty() {
        return Ok(
            json!({"ok": false, "errors": errors, "rolled_back": true, "note": "world 校验 FAIL，未写盘"}),
        );
    }
    let check = script_check(repo_root, schema);
    if !check.ok {
        return Ok(json!({
            "ok": false,
            "errors": [format!("world_schema_check.py --check FAIL：{}", check.summary)],
            "rolled_back": true,
            "note": "写前权威门禁 FAIL，未写盘",
        }));
    }
    let path = repo_root.join(WORLD_COMPONENTS_REL);
    let original =
        std::fs::read(&path).map_err(|e| format!("读 {WORLD_COMPONENTS_REL} 失败：{e}"))?;
    let before = FrozenSnapshot::take(repo_root);
    let normalized = canonicalize_indent2(raw_schema)?;
    std::fs::write(&path, &normalized)
        .map_err(|e| format!("写 {WORLD_COMPONENTS_REL} 失败：{e}"))?;
    let after_write = script_check_on_disk(repo_root, &path);
    if !after_write.ok {
        let _ = std::fs::write(&path, &original);
        return Ok(json!({
            "ok": false,
            "errors": [format!("写盘后 world_schema_check.py --check FAIL：{}", after_write.summary)],
            "rolled_back": true,
            "note": "已还原原文件",
        }));
    }
    let after = FrozenSnapshot::take(repo_root);
    let write_probe = before.write_probe(&after, &[WORLD_COMPONENTS_REL]);
    Ok(json!({
        "ok": true,
        "path": WORLD_COMPONENTS_REL,
        "bytes": normalized.len(),
        "write_probe": write_probe,
        "world_check": {"ok": after_write.ok, "exit_code": after_write.exit_code, "summary": after_write.summary},
        "note": "world indent=2 保序写 + world_schema_check 复验 + write_probe（唯一写点 world_components.json）",
    }))
}

/// 对已落盘文件跑权威门禁（apply 写后复验，无临时目录）。
fn script_check_on_disk(repo_root: &Path, path: &Path) -> WorldCheck {
    let output = Command::new("python3")
        .arg(repo_root.join(CHECK_SCRIPT))
        .arg("--check")
        .arg("--instance")
        .arg(path)
        .current_dir(repo_root)
        .output();
    let out = match output {
        Ok(value) => value,
        Err(error) => {
            return WorldCheck {
                ok: false,
                exit_code: -1,
                summary: format!("spawn {CHECK_SCRIPT} 失败：{error}"),
            };
        }
    };
    let code = out.status.code().unwrap_or(-1);
    let text =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let first = text
        .lines()
        .find(|line| line.contains("FAIL") || line.contains("ERROR") || line.contains("failure"))
        .unwrap_or_else(|| text.lines().next().unwrap_or(""));
    WorldCheck { ok: code == 0, exit_code: code, summary: first.chars().take(400).collect() }
}
