//! 场景布局：`assets/data/scenes.json` → `SceneLayout`（`Rect` / `ListLayout`）。
//! 拆分自 `data.rs`（S11）；顶层 re-export `scenes`，外部 API 不变。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{ListLayout, LoadError, Rect, SCENES_JSON, SCHEMA_VERSION_KEY, SceneLayout, parse};

static SCENES: OnceLock<BTreeMap<String, SceneLayout>> = OnceLock::new();
/// 加载场景布局表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 布局字段缺失 / 非法矩形时返回 [`LoadError`]（不 fallback）。
pub fn scenes() -> Result<&'static BTreeMap<String, SceneLayout>, LoadError> {
    if let Some(v) = SCENES.get() {
        return Ok(v);
    }
    let raw_map: BTreeMap<String, serde_json::Value> = parse(SCENES_JSON, "scenes.json")?;
    let mut raw = BTreeMap::new();
    for (k, v) in raw_map {
        if k == SCHEMA_VERSION_KEY {
            continue;
        }
        match serde_json::from_value::<RawScene>(v) {
            Ok(s) => {
                raw.insert(k, s);
            }
            Err(e) => return Err(LoadError::Schema(format!("scenes.json: `{k}` bad shape: {e}"))),
        }
    }
    let mut out = BTreeMap::new();
    for (id, r) in raw {
        let extra = r
            .extra
            .iter()
            .map(|(k, v)| {
                let rect: Rect = serde_json::from_value(v.clone()).unwrap_or_default();
                (k.clone(), rect)
            })
            .collect();
        out.insert(
            id.clone(),
            SceneLayout {
                id,
                bg: r.bg,
                title: r.title.map_or_else(Rect::default, Rect::from),
                list: ListLayout {
                    x: r.list.as_ref().map_or(0, |l| l.x),
                    y: r.list.as_ref().map_or(0, |l| l.y),
                    row_h: r.list.as_ref().map_or(0, |l| l.row_h),
                    item_h: r.list.as_ref().map_or(0, |l| l.item_h),
                    text_x: r.list.as_ref().map_or(0, |l| l.text_x),
                    w: r.list.as_ref().map_or(0, |l| l.w),
                },
                hint: r.hint.map_or_else(Rect::default, Rect::from),
                hud: r.hud.map_or_else(Rect::default, Rect::from),
                page: r.page.map_or_else(Rect::default, Rect::from),
                extra,
            },
        );
    }
    Ok(SCENES.get_or_init(|| out))
}

#[derive(Deserialize)]
struct RawScene {
    bg: u16,
    #[serde(default)]
    title: Option<RectJson>,
    #[serde(default)]
    list: Option<ListJson>,
    #[serde(default)]
    hint: Option<RectJson>,
    #[serde(default)]
    hud: Option<RectJson>,
    #[serde(default)]
    page: Option<RectJson>,
    /// 场景专属额外矩形（`player_hp`/`boss_hp`/`rounds`/`outcome`/`empty`/`body`/`locked` 等）。
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Default)]
struct RectJson {
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default)]
    w: i32,
    #[serde(default)]
    h: i32,
}

impl From<RectJson> for Rect {
    fn from(r: RectJson) -> Self {
        Self { x: r.x, y: r.y, w: r.w, h: r.h }
    }
}

#[derive(Deserialize, Default)]
struct ListJson {
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default)]
    w: i32,
    #[serde(default)]
    row_h: i32,
    #[serde(default)]
    item_h: i32,
    #[serde(default)]
    text_x: i32,
}
