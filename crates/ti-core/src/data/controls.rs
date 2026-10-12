//! 控件表：`assets/data/controls.json`（`scripts/gen_layout.py` 从 `ref/controls.json` 生成）。
//! 拆分自 `data.rs`（S11）；顶层 re-export `controls`，外部 API 不变。

use serde::Deserialize;

use std::sync::OnceLock;

use super::{CONTROLS_JSON, ControlRec, ControlsCfg, LoadError, SceneControls, parse};

#[derive(Deserialize)]
struct RawControls {
    #[serde(default)]
    source: String,
    #[serde(default)]
    scene_count: u32,
    #[serde(default)]
    record_total: u32,
    #[serde(default)]
    scenes: Vec<RawSceneControls>,
}

/// 单场景控件 serde 形状（`controls.json` 的 `scenes[]`）。
#[derive(Deserialize)]
struct RawSceneControls {
    id: String,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
    #[serde(default)]
    records: Vec<RawControlRec>,
}

/// 单控件 serde 形状（`controls.json` 的 `records[]`）。
#[derive(Deserialize)]
struct RawControlRec {
    #[serde(default)]
    kind: u32,
    #[serde(default)]
    kind_name: String,
    #[serde(default)]
    x: u32,
    #[serde(default)]
    y: u32,
    #[serde(default)]
    w: u32,
    #[serde(default)]
    h: u32,
    #[serde(default)]
    flags: u32,
    #[serde(default)]
    extra: u32,
}

/// `kind_name`（`"none"` / `"kind1"`..`"kind44"`）→ `&'static str`（静态表内插值）。
///
/// 脚本 `gen_layout.py` 只生成这 45 种名字（`KIND_NAMES`），故静态表可覆盖全部；
/// 非预期名字 → 退 `"kind0"`（不 panic、不泄漏，门禁红线）。
fn control_kind_name_static(name: &str) -> &'static str {
    if name == "none" {
        return "none";
    }
    if let Some(n) = name.strip_prefix("kind") {
        if let Ok(k) = n.parse::<u32>() {
            if (1..=44).contains(&k) {
                return KIND_NAME_TABLE
                    .get(usize::try_from(k).unwrap_or(0))
                    .copied()
                    .unwrap_or("kind1");
            }
        }
    }
    "kind1"
}

/// 静态控件类型名表（`gen_layout.py` 的 `KIND_NAMES` 镜像，索引 1..=44）。
static KIND_NAME_TABLE: [&str; 45] = [
    "none", "kind1", "kind2", "kind3", "kind4", "kind5", "kind6", "kind7", "kind8", "kind9",
    "kind10", "kind11", "kind12", "kind13", "kind14", "kind15", "kind16", "kind17", "kind18",
    "kind19", "kind20", "kind21", "kind22", "kind23", "kind24", "kind25", "kind26", "kind27",
    "kind28", "kind29", "kind30", "kind31", "kind32", "kind33", "kind34", "kind35", "kind36",
    "kind37", "kind38", "kind39", "kind40", "kind41", "kind42", "kind43", "kind44",
];

/// 控件表全局缓存（`OnceLock`，编译期内联 JSON 反序列化一次）。
static CONTROLS: OnceLock<ControlsCfg> = OnceLock::new();

/// 加载控件表（首次解析，之后命中缓存；S8.5 控件层数据源）。
///
/// # Errors
///
/// JSON 语法错误 / 顶层形状非对象（缺 `scenes`）时返回 [`LoadError`]，不 fallback
/// （数据表硬错误，`docs/business/00_index.md` 约定）。
pub fn controls() -> Result<&'static ControlsCfg, LoadError> {
    if let Some(v) = CONTROLS.get() {
        return Ok(v);
    }
    let raw: RawControls = parse(CONTROLS_JSON, "controls.json")?;
    let scenes: Vec<SceneControls> = raw
        .scenes
        .into_iter()
        .map(|sc| SceneControls {
            id: sc.id,
            width: sc.width,
            height: sc.height,
            records: sc
                .records
                .into_iter()
                .map(|r| ControlRec {
                    kind: r.kind,
                    kind_name: control_kind_name_static(&r.kind_name),
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                    flags: r.flags,
                    extra: r.extra,
                })
                .collect(),
        })
        .collect();
    Ok(CONTROLS.get_or_init(|| ControlsCfg {
        source: raw.source,
        scene_count: raw.scene_count,
        record_total: raw.record_total,
        scenes,
    }))
}

// ------------------------------------------------------------------- strings
