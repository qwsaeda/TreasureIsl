//! 赛道数据表：`assets/data/board.json` → 每图「环岛赛道」空间序列。
//!
//! 业务语义（`docs/business/progression.md` §环岛赛道）：
//! - 赛道 = 岛屿外环（Moore 邻域轮廓追踪），保留原版地图形状与坐标；
//! - 每个空间 = 一个格子（`cell`）+ 类型（`kind`）+ 分区（`group`，1..=9）+ 地形码；
//! - 生成脚本 `scripts/extract_board.py`（可复跑；`--check` 校验一致性）。
//!
//! 领地 v2（S33.2，`docs/business/territory.md` §4.2/§6/§7）：
//! - 领地格逐格 `resource`（产出地形：金矿/渔业/水果/礁石…）+ `building`（初始建筑等级）；
//! - 每图 `events[]`（特有随机事件，与 `balance.events` 常用集合并）+ `merchants[]`
//!   （武器/奴隶商人，`21_map_package_spec.md` §3.1）。
//!
//! 约定（`docs/process/02_conventions.md` §1）：数值一律进 JSON；解析失败 = 硬错误。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{BOARD_JSON, LoadError, parse};
use super::{map_pack, table_source};

/// 赛道空间（`board.json` 的 `spaces[]` 一项）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardSpace {
    /// 行坐标（`0..grid_h`）。
    pub row: u32,
    /// 列坐标（`0..grid_w`）。
    pub col: u32,
    /// 空间类型键（`start`/`plain`/`port`/`city`/`event`/`card`/`battle`/`treasure`/`boss`）。
    pub kind: String,
    /// 分区号（`1..=9`，对齐原版地形类别码）。
    pub group: u8,
    /// 地形码（`S`/`R`/`L`/`T`，与 `terrain.json` 同码表）。
    pub terrain: char,
    /// 岛原型 id（S28.3：`balance.territory.prototypes[].id`；领地格有值，功能格 = `None`）。
    pub isle: Option<String>,
    /// 领地 v2 产出地形（`territory.md` §4.2：金矿/渔业/水果/礁石…；领地格有值）。
    pub resource: Option<String>,
    /// 领地 v2 初始建筑等级（§4.2；领地格恒 `1..=上限`，功能格 = `0`）。
    pub building: u8,
}

/// 每图特有随机事件（`map.json events[]`；`territory.md` §6；`rules::events` 消费）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEvent {
    /// 事件 id（地图内唯一）。
    pub id: String,
    /// 事件类型键（`rules::events::MaritimeEvent` kind：wind_god/shipwreck/…）。
    pub kind: String,
    /// 风神保佑跳格参数（`kind=wind_god`；`0` = 多扔骰子一次；其他事件 = `0`）。
    pub jump_cells: i32,
}

/// 每图商人（`map.json merchants[]`；`territory.md` §7；`rules::merchant` 消费）。
#[derive(Debug, Clone, PartialEq)]
pub struct MapMerchant {
    /// 商人 id（`weapon`/`slave`）。
    pub id: String,
    /// 商人驻点行坐标。
    pub row: u32,
    /// 商人驻点列坐标。
    pub col: u32,
    /// 价格倍率（`1.0` = 按 `balance.merchants` 原价；>1 = 溢价）。
    pub price_mult: f32,
}

/// 一张地图的赛道（有序空间序列 + 网格尺寸 + 每图事件/商人）。
#[derive(Debug, Clone, PartialEq)]
pub struct BoardMap {
    /// 网格宽。
    pub grid_w: u32,
    /// 网格高。
    pub grid_h: u32,
    /// 赛道空间（顺序 = 前进方向，末格回到首格 = 绕岛一圈）。
    pub spaces: Vec<BoardSpace>,
    /// 领地 v2 每图特有随机事件（`territory.md` §6）。
    pub events: Vec<MapEvent>,
    /// 领地 v2 每图商人（武器/奴隶；§7）。
    pub merchants: Vec<MapMerchant>,
}

/// 赛道表（地图 id → 赛道）。
#[derive(Debug, Clone, PartialEq)]
pub struct BoardDb {
    /// 各图赛道。
    pub maps: BTreeMap<String, BoardMap>,
}

impl BoardDb {
    /// 某图赛道（未命中 → `None`）。
    #[must_use]
    pub fn map(&self, map_id: &str) -> Option<&BoardMap> {
        self.maps.get(map_id)
    }
}

#[derive(Debug, Deserialize)]
struct RawSpace {
    cell: [u32; 2],
    kind: String,
    group: u8,
    terrain: String,
    #[serde(default)]
    isle: Option<String>,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default)]
    building: u8,
}

#[derive(Debug, Deserialize)]
struct RawEvent {
    id: String,
    kind: String,
    #[serde(default)]
    jump_cells: i32,
}

#[derive(Debug, Deserialize)]
struct RawMerchant {
    id: String,
    cell: [u32; 2],
    #[serde(default = "default_price_mult")]
    price_mult: f32,
}

fn default_price_mult() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
struct RawMap {
    grid_w: u32,
    grid_h: u32,
    spaces: Vec<RawSpace>,
    #[serde(default)]
    events: Vec<RawEvent>,
    #[serde(default)]
    merchants: Vec<RawMerchant>,
}

#[derive(Debug, Deserialize)]
struct RawDb {
    maps: BTreeMap<String, RawMap>,
}

static BOARD: OnceLock<BoardDb> = OnceLock::new();

/// 加载赛道表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 字段缺失时返回 [`LoadError`]（不 fallback）。
pub fn board() -> Result<&'static BoardDb, LoadError> {
    if let Some(v) = BOARD.get() {
        return Ok(v);
    }
    let value = parse(
        table_source::table_text(map_pack::table_key::BOARD, BOARD_JSON).as_ref(),
        "board.json",
    )?;
    let raw: RawDb = serde_json::from_value(value)
        .map_err(|e| LoadError::Schema(format!("board.json bad shape: {e}")))?;
    let mut maps = BTreeMap::new();
    for (id, m) in raw.maps {
        let mut spaces = Vec::with_capacity(m.spaces.len());
        for s in m.spaces {
            let terrain = s.terrain.chars().next().unwrap_or('S');
            spaces.push(BoardSpace {
                row: s.cell[0],
                col: s.cell[1],
                kind: s.kind,
                group: s.group,
                terrain,
                isle: s.isle,
                resource: s.resource,
                building: s.building,
            });
        }
        let events = m
            .events
            .into_iter()
            .map(|e| MapEvent { id: e.id, kind: e.kind, jump_cells: e.jump_cells })
            .collect();
        let merchants = m
            .merchants
            .into_iter()
            .map(|mc| MapMerchant {
                id: mc.id,
                row: mc.cell[0],
                col: mc.cell[1],
                price_mult: mc.price_mult,
            })
            .collect();
        maps.insert(id, BoardMap { grid_w: m.grid_w, grid_h: m.grid_h, spaces, events, merchants });
    }
    Ok(BOARD.get_or_init(|| BoardDb { maps }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S33.2：领地 v2 新字段解析（每图 events/merchants + 逐格 resource/building）。
    #[test]
    fn parses_v2_fields_per_map() {
        let db = board().expect("board.json 应可加载");
        let m01 = db.map("m01").expect("m01 应存在");
        // 每图事件非空 + 商人含 weapon/slave
        assert!(!m01.events.is_empty(), "m01 events 非空");
        let ids: Vec<&str> = m01.merchants.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"weapon"), "m01 有武器商人");
        assert!(ids.contains(&"slave"), "m01 有奴隶商人");
        // 逐格：领地格有 resource + building ≥1；功能格无
        let mut terr = 0;
        for s in &m01.spaces {
            if s.isle.is_some() {
                terr += 1;
                assert!(s.resource.is_some(), "领地格 {terr} 有 resource");
                assert!(s.building >= 1, "领地格 {terr} building ≥1");
            }
        }
        assert_eq!(terr, 20, "m01 领地格 20");
    }

    /// S33.2：全图一致（每图都配 events + weapon/slave 商人）。
    #[test]
    fn all_maps_have_v2_events_and_merchants() {
        let db = board().expect("board.json 应可加载");
        for (id, m) in &db.maps {
            assert!(!m.events.is_empty(), "{id} events 非空");
            let m_ids: Vec<&str> = m.merchants.iter().map(|x| x.id.as_str()).collect();
            assert!(m_ids.contains(&"weapon"), "{id} 有武器商人");
            assert!(m_ids.contains(&"slave"), "{id} 有奴隶商人");
        }
    }
}
