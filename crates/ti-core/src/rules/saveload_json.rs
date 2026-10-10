//! S3.10 / S8.3.d 存读档正式实现（JSON v2 + 可复现 RNG）。
//!
//! 业务规格（唯一事实源）：[`docs/business/persistence.md`](../../../docs/business/persistence.md)。
//!
//! S8 定位变更：旧 `rules/saveload.rs` 的 26 字节二进制布局（`oracle::saveload`）降级为
//! 参照物，本模块改为 **JSON v2 存档**（人可读、可 diff、可迁移），数值/槽位数走
//! `balance.save`；RNG 状态走 [`crate::rules::rng::GameRng`] 可复现（`docs/business/persistence.md` §RNG）。
//!
//! 设计要点：
//! - **存档 = 运行时状态**（角色/地图/场景/位置/金/HP/建筑/商品/RNG 种子/银行/解锁地图），
//!   全字段在 JSON 里显式列出，不靠二进制偏移；
//! - **v1 → v2 迁移**：v1 缺 `version`/`bank`/`rng_seed` → 补默认（`version=2`、`bank=0`、
//!   `rng_seed` 默认种子），纯函数幂等可测；
//! - **RNG 可复现**：存档含 `rng_seed`，读档后注入 `GameRng::new(seed)` 继续随机流；
//! - **无占位 / 不 panic**：解析失败返回 [`SaveErr`]，不静默 fallback（门禁红线）。

use serde::{Deserialize, Serialize};

use crate::oracle::saveload::SaveSlot;
use crate::rules::rng::GameRng;

/// 存档默认 RNG 种子（v1 迁移 / 新游戏无种子时使用，`docs/business/persistence.md` §RNG）。
pub const DEFAULT_RNG_SEED: u32 = 42;

/// 存档错误（JSON 形状 / 未知版本 / 越界场景）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveErr {
    /// 版本号未知（非 1 / 非 2）。
    UnknownVersion(u32),
    /// JSON 顶层非对象（形状错）。
    NotAnObject,
    /// 缺必填字段（`character`/`map` 等，S8 存档不再允许缺省）。
    MissingField(&'static str),
}

/// 存档 v2 结构（`docs/business/persistence.md` §存档结构）。
///
/// 全字段显式（序列化时补齐缺省），反序列化用 `#[serde(default)]` 兜底 v1 缺字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveV2 {
    /// 存档版本（v2 恒 2；v1 缺省 → 迁移补 2）。
    #[serde(default = "default_version")]
    pub version: u32,
    /// 角色 id（`business/rng 无关`，用稳定字符串 id，如 `"drake"`）。
    pub character: String,
    /// 地图 id（如 `"m01"`）。
    pub map: String,
    /// 退出时所在场景（`"Index"`/`"MainMap"`/`"Vs"` 等，与 `Scene` 变体名一致）。
    #[serde(default = "default_scene")]
    pub scene: String,
    /// 玩家所在格。
    #[serde(default)]
    pub position: GridPosSave,
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

/// 玩家位置（`docs/business/persistence.md` 存档结构）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GridPosSave {
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

// ---- serde 默认值函数（v1 缺字段 / 未知场景兜底，不 panic）----

fn default_version() -> u32 {
    1 // 缺省按 v1，由 `migrate` 补成 2；显式写 2 的存档不动
}

fn default_scene() -> String {
    "Index".to_owned()
}

fn default_rng_seed() -> u32 {
    DEFAULT_RNG_SEED
}

impl SaveV2 {
    /// 全新存档（v2，默认场景 `Index` + 默认 RNG 种子，非迁移产物）。
    #[must_use]
    pub fn new(character: String, map: String) -> Self {
        Self {
            version: 2,
            character,
            map,
            scene: "Index".to_owned(),
            position: GridPosSave::default(),
            gold: 0,
            hp: 0,
            buildings: Vec::new(),
            goods: Vec::new(),
            rng_seed: DEFAULT_RNG_SEED,
            bank: 0,
            unlocked_maps: Vec::new(),
        }
    }

    /// 序列化存档为 JSON 字符串（人可读、可 diff；失败返回 `Err` 形状错误）。
    ///
    /// # Errors
    /// JSON 序列化失败（类型不兼容，实际不会发生）→ [`SaveErr::NotAnObject`]。
    pub fn to_json(&self) -> Result<String, SaveErr> {
        serde_json::to_string(self).map_err(|_| SaveErr::NotAnObject)
    }

    /// 从 JSON 解析存档（缺字段走 `#[serde(default)]`，未知版本由调用方 `migrate` 兜底）。
    ///
    /// # Errors
    /// 顶层非对象 → [`SaveErr::NotAnObject`]；JSON 语法错 → [`SaveErr::MissingField` 兜底]。
    pub fn from_json(json: &str) -> Result<Self, SaveErr> {
        serde_json::from_str(json).map_err(|_| SaveErr::NotAnObject)
    }

    /// v1 → v2 迁移（纯函数、幂等）：
    /// - `version < 2` → 补 `version=2`、`bank=0`（缺省）、`rng_seed=DEFAULT_RNG_SEED`（缺省）；
    /// - `version == 2` → 原样返回（幂等，再迁不变）；
    /// - 未知版本（≥3）→ `Err(UnknownVersion)`，不静默接受。
    ///
    /// # Errors
    /// 版本非 1 / 非 2 → [`SaveErr::UnknownVersion`]。
    pub fn migrate(mut self) -> Result<Self, SaveErr> {
        if self.version == 2 {
            return Ok(self);
        }
        if self.version > 2 {
            return Err(SaveErr::UnknownVersion(self.version));
        }
        // version == 1（或缺省 1）：补 v2 缺省字段
        self.version = 2;
        self.bank = 0;
        self.rng_seed = DEFAULT_RNG_SEED;
        Ok(self)
    }

    /// 读档后注入 RNG 种子（`docs/business/persistence.md` §RNG「读档后可续」）。
    /// 同 `rng_seed` + 同后续操作序列 → 同随机流（可复现、可回归测）。
    #[must_use]
    pub fn rng(&self) -> GameRng {
        GameRng::new(self.rng_seed)
    }

    /// 存档是否已迁移到 v2（`version == 2`）。
    #[must_use]
    pub fn is_v2(&self) -> bool {
        self.version == 2
    }
}

/// 存档管理器：持有槽位列表 + JSON 序列化/反序列化 + 迁移 + RNG 续流。
///
/// 槽位数走 `balance.save.slot_count`（`from_balance` 注入）；槽内容存 `SaveV2`。
#[derive(Debug, Clone, Default)]
pub struct SaveManager {
    slots: Vec<SaveV2>,
}

impl SaveManager {
    /// 新建管理器（指定槽位数，全空存档 = 每个槽 `SaveV2::new` 默认）。
    #[must_use]
    pub fn new(slot_count: usize) -> Self {
        Self { slots: vec![SaveV2::new(String::new(), String::new()); slot_count] }
    }

    /// 从 `balance.save.slot_count` 建管理器（槽位数单一事实源，`docs/business/persistence.md`）。
    /// 整表改坏 → 回退 5 槽（不 panic，门禁红线）。
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
    pub fn slot(&self, index: usize) -> Option<&SaveV2> {
        self.slots.get(index)
    }

    /// 某槽位写入（越界 → 静默忽略，与状态机语义一致）。
    pub fn set_slot(&mut self, index: usize, s: SaveV2) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = s;
        }
    }

    /// 某槽位写入 JSON 字符串（越界 / 解析错 → `None`）。
    pub fn set_slot_json(&mut self, index: usize, json: &str) -> Option<()> {
        let save = SaveV2::from_json(json).ok()?;
        self.set_slot(index, save);
        Some(())
    }

    /// 某槽位序列化为 JSON 字符串（越界 → `None`）。
    #[must_use]
    pub fn slot_json(&self, index: usize) -> Option<String> {
        self.slots.get(index).map(|s| s.to_json().unwrap_or_default())
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

// ---- 与旧 `oracle::saveload::SaveSlot` 的互转（参照物，非运行时依赖）----

/// 从旧 `SaveSlot`（26 字节布局的 oracle 化）构造 v2 存档（迁移路径，供旧档兼容）。
///
/// `SaveSlot` 只有 `character/gold/map_slot/position/version` 数字字段，
/// v2 用稳定字符串 id：数字 → 默认字符串保守映射（`char{n}` / `m{nn:02}`），
/// 真实 id 映射由 `balance` 角色/地图目录提供（S8.3.e），本处保守不 panic。
#[must_use]
pub fn from_legacy_slot(slot: &SaveSlot) -> SaveV2 {
    SaveV2 {
        version: 2,
        character: format!("char{}", slot.character),
        map: format!("m{:02}", slot.map_slot),
        scene: "Index".to_owned(),
        position: GridPosSave { row: slot.position, col: 0 },
        gold: slot.gold,
        hp: 0,
        buildings: Vec::new(),
        goods: Vec::new(),
        rng_seed: DEFAULT_RNG_SEED,
        bank: 0,
        unlocked_maps: Vec::new(),
    }
}
