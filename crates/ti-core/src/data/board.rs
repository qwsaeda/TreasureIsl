//! 赛道数据表：`assets/data/board.json` → 每图「环岛赛道」空间序列。
//!
//! 业务语义（`docs/business/progression.md` §环岛赛道）：
//! - 赛道 = 岛屿外环（Moore 邻域轮廓追踪），保留原版地图形状与坐标；
//! - 每个空间 = 一个格子（`cell`）+ 类型（`kind`）+ 分区（`group`，1..=9）+ 地形码；
//! - 生成脚本 `scripts/extract_board.py`（可复跑；`--check` 校验一致性）。
//!
//! 约定（`docs/process/02_conventions.md` §1）：数值一律进 JSON；解析失败 = 硬错误。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{BOARD_JSON, LoadError, parse};

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
}

/// 一张地图的赛道（有序空间序列 + 网格尺寸）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardMap {
    /// 网格宽。
    pub grid_w: u32,
    /// 网格高。
    pub grid_h: u32,
    /// 赛道空间（顺序 = 前进方向，末格回到首格 = 绕岛一圈）。
    pub spaces: Vec<BoardSpace>,
}

/// 赛道表（地图 id → 赛道）。
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Deserialize)]
struct RawMap {
    grid_w: u32,
    grid_h: u32,
    spaces: Vec<RawSpace>,
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
    let value = parse(BOARD_JSON, "board.json")?;
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
            });
        }
        maps.insert(id, BoardMap { grid_w: m.grid_w, grid_h: m.grid_h, spaces });
    }
    Ok(BOARD.get_or_init(|| BoardDb { maps }))
}
