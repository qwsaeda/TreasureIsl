//! 地形表：`assets/data/terrain.json` → 每格地形码 + 全局图例（色令牌名）。
//!
//! S18.9：主地图棋盘从"单色平铺菱形"升级为"航海图"——地形由**数据**给出
//! （`sea`/`reef`/`land`/`treasure`），配色由**图例**给出（theme 令牌名），
//! 渲染层只做"按格取色"（不认场景语义，契约同 `09_render_framework.md` §5）。
//!
//! 生成规则（`scripts/` 侧一次性生成，数据已落库）：固定 seed（地图 id 哈希）→
//! 中心衰减椭圆岛 + 确定性抖动 → 岛缘浅礁 → BOSS/陆格宝藏 → 起点强制海面。
//! ⇒ 同一地图永远同一地形（快照/基线可复现）。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{LoadError, TERRAIN_JSON, parse};

/// 地形图例一项：地形码 → 配色。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainLegend {
    /// 语义名（`sea`/`reef`/`land`/`treasure`；仅信息，不参与渲染分支）。
    pub name: String,
    /// 瓦片底色令牌名（`theme.by_name`）。
    pub token: String,
    /// 瓦片顶棱高光令牌名。
    pub edge: String,
    /// 交错变体令牌名（`None` = 不交错）。
    pub alt: Option<String>,
}

/// 一张地图的地形（每行一个字符串，字符 = 地形码）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainMap {
    /// 行数据（长度 = `grid_h`，每行长度 = `grid_w`）。
    pub rows: Vec<String>,
}

/// 地形表（全局图例 + 各图地形）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainDb {
    /// 地形码 → 图例（全局共用）。
    pub legend: BTreeMap<String, TerrainLegend>,
    /// 地图 id → 地形。
    pub maps: BTreeMap<String, TerrainMap>,
}

impl TerrainDb {
    /// 某图某格的地形码（如 `'L'`）；越界 / 缺数据 = `None`。
    #[must_use]
    pub fn code_at(&self, map_id: &str, row: u32, col: u32) -> Option<char> {
        let m = self.maps.get(map_id)?;
        let r = m.rows.get(usize::try_from(row).ok()?)?;
        r.chars().nth(usize::try_from(col).ok()?)
    }

    /// 地形码 → 图例项。
    #[must_use]
    pub fn legend_of(&self, code: char) -> Option<&TerrainLegend> {
        self.legend.get(&code.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct RawLegend {
    name: String,
    token: String,
    edge: String,
    #[serde(default)]
    alt: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawMap {
    rows: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawDb {
    legend: BTreeMap<String, RawLegend>,
    maps: BTreeMap<String, RawMap>,
}

static TERRAIN: OnceLock<TerrainDb> = OnceLock::new();

/// 加载地形表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 字段缺失时返回 [`LoadError`]（不 fallback，`docs/business/00_index.md`）。
pub fn terrain() -> Result<&'static TerrainDb, LoadError> {
    if let Some(v) = TERRAIN.get() {
        return Ok(v);
    }
    let value = parse(TERRAIN_JSON, "terrain.json")?;
    let raw: RawDb = serde_json::from_value(value)
        .map_err(|e| LoadError::Schema(format!("terrain.json bad shape: {e}")))?;
    let mut legend = BTreeMap::new();
    for (code, r) in raw.legend {
        legend
            .insert(code, TerrainLegend { name: r.name, token: r.token, edge: r.edge, alt: r.alt });
    }
    let mut maps = BTreeMap::new();
    for (id, r) in raw.maps {
        maps.insert(id, TerrainMap { rows: r.rows });
    }
    Ok(TERRAIN.get_or_init(|| TerrainDb { legend, maps }))
}
