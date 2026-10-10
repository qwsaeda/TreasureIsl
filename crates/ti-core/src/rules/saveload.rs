//! S3.10 / S8.3.d 存读档/设置正式实现（JSON v2 + 可复现 RNG，去二进制推测格式）。
//!
//! S8 定位变更：旧 26 字节二进制布局（`oracle::saveload` 的 `oracle_write_slot`/
//! `oracle_read_slot`，**推测**格式）降级参照物；本模块改为 **JSON v2 存档**
//! （人可读、可 diff、可迁移），依据 `docs/business/persistence.md` §存档结构：
//! - 存档槽 [`SaveSlotV2`]（`character`/`map`/`scene`/`position`/`gold`/`hp`/`buildings`/
//!   `goods`/`rng_seed`/`bank`/`unlocked_maps` 全显式字段）；
//! - [`SaveManager`]：槽位管理（JSON 序列化/反序列化 + v1→v2 迁移 + RNG 续流）；
//! - **RNG 可复现**：存档含 `rng_seed`，读档后注入 `GameRng::new(seed)` 续随机流
//!   （`docs/business/persistence.md` §RNG「读档后可续」）。
//!
//! 迁移（`docs/business/persistence.md` §迁移）：v1 缺 `version`/`bank`/`rng_seed` →
//! 补默认（`version=2`、`bank=0`、`rng_seed` 默认种子）；迁移纯函数幂等可测。
//!
//! 数值（槽位数）走 `balance.save.slot_count`（`from_balance` 注入，单一事实源）。

use crate::oracle::saveload::SaveSlot;
use crate::rules::rng::GameRng;
use serde::{Deserialize, Serialize};

/// 存档槽位（v2，JSON 显式字段；`docs/business/persistence.md` §存档结构）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotV2 {
    /// 版本号（v2 恒 2；v1 缺省 → 迁移补 2）。
    #[serde(default = "default_version")]
    pub version: u32,
    /// 角色 id（稳定字符串，如 `"drake"`）。
    pub character: String,
    /// 地图 id（如 `"m01"`）。
    pub map: String,
    /// 退出时所在场景（`"Index"`/`"MainMap"` 等，与 `Scene` 变体名一致）。
    #[serde(default = "default_scene")]
    pub scene: String,
    /// 玩家所在格。
    #[serde(default)]
    pub position: GridPos,
    /// 金币。
    #[serde(default)]
    pub gold: u32,
    /// 玩家 HP。
    #[serde(default)]
    pub hp: u32,
    /// 已建建筑 id 列表（稳定字符串，如 `"palisade"`）。
    #[serde(default)]
    pub buildings: Vec<String>,
    /// 持有商品（id + 数量）。
    #[serde(default)]
    pub goods: Vec<GoodsSave>,
    /// **复现关键**：RNG 当前种子（读档后 `GameRng::new(rng_seed)` 续流）。
    #[serde(default = "default_rng_seed")]
    pub rng_seed: u32,
    /// 银行存金（v1 缺省 → 迁移补 0）。
    #[serde(default)]
    pub bank: u32,
    /// 已解锁地图 id 列表。
    #[serde(default)]
    pub unlocked_maps: Vec<String>,
}

/// 玩家位置（`docs/business/persistence.md` §存档结构）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GridPos {
    /// 行。
    pub row: u32,
    /// 列。
    pub col: u32,
}

/// 一条持有商品（`id` 稳定字符串 + 数量）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoodsSave {
    /// 商品稳定 id（如 `"spice"`，对应 `balance.trade.goods[].id`）。
    pub id: String,
    /// 数量。
    pub qty: u32,
}

// ---- serde 默认值（v1 缺字段兜底，不 panic；门禁红线）----

fn default_version() -> u32 {
    1 // 缺省按 v1，由 `migrate` 补成 2
}

fn default_scene() -> String {
    "Index".to_owned()
}

/// 存档默认 RNG 种子（v1 迁移 / 新游戏无种子时使用，`docs/business/persistence.md` §RNG）。
pub const DEFAULT_RNG_SEED: u32 = 42;

fn default_rng_seed() -> u32 {
    DEFAULT_RNG_SEED
}

impl SaveSlotV2 {
    /// 全新存档（v2，默认场景 `Index` + 默认 RNG 种子，非迁移产物）。
    #[must_use]
    pub fn new(character: String, map: String) -> Self {
        Self {
            version: 2,
            character,
            map,
            scene: "Index".to_owned(),
            position: GridPos::default(),
            gold: 0,
            hp: 0,
            buildings: Vec::new(),
            goods: Vec::new(),
            rng_seed: DEFAULT_RNG_SEED,
            bank: 0,
            unlocked_maps: Vec::new(),
        }
    }

    /// 序列化存档为 JSON 字符串（人可读、可 diff；失败返回空串，不 panic）。
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// 从 JSON 解析存档槽（缺字段走 `#[serde(default)]`；解析失败 → `None`，不 panic）。
    #[must_use]
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    /// v1 → v2 迁移（纯函数、幂等）：`version<2` → 补 `version=2`/`bank=0`/`rng_seed` 默认；
    /// `version==2` → 原样返回（幂等，再迁不变）。
    #[must_use]
    pub fn migrate(mut self) -> Self {
        if self.version < 2 {
            self.version = 2;
            self.bank = 0;
            self.rng_seed = DEFAULT_RNG_SEED;
        }
        self
    }

    /// 读档后注入 RNG 种子（`docs/business/persistence.md` §RNG「读档后可续」）。
    /// 同 `rng_seed` + 同后续操作序列 → 同随机流（可复现、可回归测）。
    #[must_use]
    pub fn rng(&self) -> GameRng {
        GameRng::new(self.rng_seed)
    }

    /// 是否已迁移到 v2（`version == 2`）。
    #[must_use]
    pub fn is_v2(&self) -> bool {
        self.version == 2
    }

    /// 从旧 26 字节 `SaveSlot`（oracle 化）构造 v2 存档（迁移路径，数字→稳定字符串保守映射）。
    #[must_use]
    pub fn from_legacy_slot(slot: &SaveSlot) -> Self {
        Self {
            version: 2,
            character: format!("char{}", slot.character),
            map: format!("m{:02}", slot.map_slot),
            scene: "Index".to_owned(),
            position: GridPos { row: slot.position, col: 0 },
            gold: slot.gold,
            hp: 0,
            buildings: Vec::new(),
            goods: Vec::new(),
            rng_seed: DEFAULT_RNG_SEED,
            bank: 0,
            unlocked_maps: Vec::new(),
        }
    }
}

/// 存档管理器：持有槽位列表 + JSON 序列化/反序列化 + 迁移 + RNG 续流。
///
/// 槽位数走 `balance.save.slot_count`（`from_balance` 注入，`docs/business/persistence.md`）；
/// 槽内容存 [`SaveSlotV2`]。
#[derive(Debug, Clone, Default)]
pub struct SaveManager {
    slots: Vec<SaveSlotV2>,
}

impl SaveManager {
    /// 新建管理器（指定槽位数，全空存档）。
    #[must_use]
    pub fn new(slot_count: usize) -> Self {
        Self { slots: vec![SaveSlotV2::new(String::new(), String::new()); slot_count] }
    }

    /// 从 `balance.save.slot_count` 建管理器（槽位数单一事实源）。整表改坏 → 回退 5 槽（不 panic）。
    #[must_use]
    pub fn from_balance() -> Self {
        let n = match crate::data::balance() {
            Ok(b) => usize::try_from(b.save.slot_count).unwrap_or(5),
            Err(_) => 5,
        };
        Self::new(n)
    }

    /// 槽位数（只读）。
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// 某槽位只读（越界 → `None`）。
    #[must_use]
    pub fn slot(&self, index: usize) -> Option<&SaveSlotV2> {
        self.slots.get(index)
    }

    /// 某槽位写入（越界 → 静默忽略）。
    pub fn set_slot(&mut self, index: usize, s: SaveSlotV2) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = s;
        }
    }

    /// 某槽位写入 JSON 字符串（越界 / 解析错 → `None`）。
    #[must_use]
    pub fn set_slot_json(&mut self, index: usize, json: &str) -> Option<()> {
        let save = SaveSlotV2::from_json(json)?;
        self.set_slot(index, save);
        Some(())
    }

    /// 某槽位序列化为 JSON 字符串（越界 → `None`）。
    #[must_use]
    pub fn slot_json(&self, index: usize) -> Option<String> {
        self.slots.get(index).map(SaveSlotV2::to_json)
    }

    /// 读档续流：取某槽的 `GameRng`（越界 → 用默认种子 RNG，不 panic）。
    #[must_use]
    pub fn rng_for_slot(&self, index: usize) -> GameRng {
        match self.slots.get(index) {
            Some(s) => s.rng(),
            None => GameRng::new(DEFAULT_RNG_SEED),
        }
    }
}
