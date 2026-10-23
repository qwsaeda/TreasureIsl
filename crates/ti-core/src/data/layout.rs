//! 场景布局：`assets/data/scenes.json` → `SceneLayout`（`Rect` / `ListLayout`）。
//! 拆分自 `data.rs`（S11）；顶层 re-export `scenes`，外部 API 不变。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{
    AmbientStar, ListLayout, LoadError, Primitive, Rect, SCENES_JSON, SCHEMA_VERSION_KEY,
    SceneLayout, parse,
};

static SCENES: OnceLock<BTreeMap<String, SceneLayout>> = OnceLock::new();
/// 加载场景布局表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 布局字段缺失 / 非法矩形时返回 [`LoadError`]（不 fallback）。
#[allow(clippy::too_many_lines)]
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
        // EXP-074/RectDefault：extra 嵌套对象（如 status/rounds 只有 x/y 无 w/h）用 RectJson 解析
        // （RectJson 的 w/h 有 #[serde(default)]），缺 w/h 时填 0 而非整体解析失败兜底全 0。
        let extra = r
            .extra
            .iter()
            .map(|(k, v)| {
                let rj: RectJson = serde_json::from_value(v.clone()).unwrap_or_default();
                (k.clone(), Rect::from(rj))
            })
            .collect();
        // EXP-074：`extra` 值可能是标量字段（如 `locked:{"text_x":130}`），而非 Rect 形状。
        // 逐 key 抽取 `*_x` / `*_y` 标量，避免缺 key 被静默置 0 丢失坐标。
        // FIND-106/FIND-202：分诊——非数值标量（字符串/嵌套对象等，如装饰性 color 字段）
        // 静默跳过；数值但 i64 溢出或 i32 越界才 eprintln（真异常，validate_data.py V2 兜底）。
        let mut extra_scalar = BTreeMap::new();
        for (k, v) in &r.extra {
            if let Some(obj) = v.as_object() {
                for (fk, fv) in obj {
                    match fv.as_i64() {
                        Some(n) => match i32::try_from(n) {
                            Ok(s) => {
                                extra_scalar.insert(format!("{k}.{fk}"), s);
                            }
                            Err(_) => eprintln!(
                                "scenes.json: `{k}.{fk}` = {fv} out of i32 range, extra_scalar skipped"
                            ),
                        },
                        None => {
                            if fv.is_number() {
                                eprintln!(
                                    "scenes.json: `{k}.{fk}` = {fv} is numeric but not an i64 scalar, extra_scalar skipped"
                                );
                            }
                        }
                    }
                }
            }
        }
        // §4 立绘允许区清单（scenes.json 的 `portrait_zones`，多块；缺 = 空 Vec）。
        // EXP-074/RectDefault：嵌套对象用 RectJson 解析（w/h 有 #[serde(default)]），缺 w/h 填 0。
        let portrait_zones: Vec<Rect> = r
            .portrait_zones
            .into_iter()
            .map(|v| {
                let rj: RectJson = serde_json::from_value(v).unwrap_or_default();
                Rect::from(rj)
            })
            .collect();
        // S18.6 声明式原语表（`primitives` 数组；有序 = 绘制顺序；缺 = 空 Vec）。
        // 单项解析失败（形状不对）= 跳过该项（不 panic、不整场景失败）：数据先行、渲染后补。
        let primitives: Vec<Primitive> = r
            .primitives
            .into_iter()
            .filter_map(|v| {
                let pj: PrimitiveJson = serde_json::from_value(v).ok()?;
                Some(Primitive {
                    kind: pj.kind,
                    rect: Rect::from(pj.rect),
                    param: pj.param,
                    color: pj.color,
                    text: pj.text,
                    text_key: pj.text_key,
                    params: pj.params,
                    when: pj.when,
                    item: pj.item,
                    z: pj.z,
                    tier: pj.tier,
                })
            })
            .collect();
        // 飞轮 R1 ④-05：v3 氛围星空星位表（`ambient_art.stars` 数组；单项解析失败 = 跳过该项，
        // 与 `primitives` 同容错口径；缺键 = 空 Vec = 不画星点）。
        let ambient_stars: Vec<AmbientStar> = r
            .extra
            .get("ambient_art")
            .and_then(|v| v.get("stars"))
            .and_then(serde_json::Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<AmbientStar>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();
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
                    highlight_offset: r.list.as_ref().map_or(0, |l| l.highlight_offset),
                    gold_x: r.list.as_ref().map_or(0, |l| l.gold_x),
                    summary_dy: r.list.as_ref().map_or(0, |l| l.summary_dy),
                },
                hint: r.hint.map_or_else(Rect::default, Rect::from),
                hud: r.hud.map_or_else(Rect::default, Rect::from),
                page: r.page.map_or_else(Rect::default, Rect::from),
                extra,
                extra_scalar,
                portrait_zones,
                primitives,
                ambient_stars,
            },
        );
    }
    Ok(SCENES.get_or_init(|| out))
}

#[derive(Deserialize)]
struct RawScene {
    /// S15.6 裁决①：`bg` 键已删（运行时零消费——渲染底色 = chrome `COLOR_PARCH_DARK` 深底；
    /// 旧浅米白 65440 与代码相反）。解析容错：键缺省 = 0（哨兵，无消费方）。
    #[serde(default)]
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
    /// §4 立绘允许区清单（`08_layout_design.md` §4.1/§4.2 声明的大立绘可落区，多块；缺 = 无声明）。
    #[serde(default)]
    portrait_zones: Vec<serde_json::Value>,
    /// S18.6 声明式原语表（有序 = 绘制顺序；缺 = 空）。
    #[serde(default)]
    primitives: Vec<serde_json::Value>,
}

/// 单个原语的数据形状（`scenes.json` `primitives[]` 一项）。
#[derive(Deserialize, Default)]
struct PrimitiveJson {
    /// 原语名（dispatcher 键）。
    #[serde(default)]
    kind: String,
    /// 主矩形参数。
    #[serde(default)]
    rect: RectJson,
    /// 副参数（语义随 `kind` 变）。
    #[serde(default)]
    param: i32,
    /// 配色令牌名（空 = 原语默认）。
    #[serde(default)]
    color: String,
    /// 文本内容。
    #[serde(default)]
    text: String,
    /// i18n key（优先于 `text`）。
    #[serde(default)]
    text_key: String,
    /// 通用参数（数值/字符串皆可）。
    #[serde(default)]
    params: BTreeMap<String, serde_json::Value>,
    /// 显示条件（空 = 总是）。
    #[serde(default)]
    when: String,
    /// 所属菜单项序号（-1 = 不属于菜单项）。
    #[serde(default)]
    item: i64,
    /// 层序（负值；Bevy 相机朝 -Z）。
    #[serde(default)]
    z: i32,
    /// 文本档位（title/body/hint）。
    #[serde(default)]
    tier: String,
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
    /// EXP-074：高亮矩形相对基线上移像素（旧代码 `y - 10`/`y - 12`）。
    #[serde(default)]
    highlight_offset: i32,
    /// 次要列文字 x（如 saveload 金字号列；EXP-074）。
    #[serde(default)]
    gold_x: i32,
    /// 第二行摘要 y 偏移（`scenes.json` `list.summary_dy`；飞轮 R1 ③-15）。
    #[serde(default)]
    summary_dy: i32,
}
