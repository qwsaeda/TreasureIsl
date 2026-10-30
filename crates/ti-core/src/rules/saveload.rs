//! S3.10 / S8.3.d / S12.5 存读档/设置正式实现（JSON v2 + 可复现 RNG，去二进制推测格式）。
//!
//! 本模块与 [`saveload_json`](super::saveload_json) 语义等价，API 面略宽
//! （`SaveSlotV2` 替代 `SaveV2`，含 `capture_state`/`restore_state` 跨包状态桥）；
//! 数值/槽位数走 `balance.save.slot_count`（`from_balance` 注入，单一事实源）。
//!
//! S12.5：
//! - **RNG 精确续流**：存档含 `rng_seed` + `rng_steps`，读档后
//!   `GameRng::restore_state(rng_seed, rng_steps)` O(log n) 跳转（非逐次回放）；
//! - **跨包状态桥**：[`SaveState`] + `capture_state`/`restore_state` 以 `domain` 公共类型
//!   （`Character`/`MapSlot`/`GridPos`/`Bank`/`SceneId`）为参数（c1 契约，纯函数不持 session）；
//! - 字符串↔枚举转换复用 `saveload_json` 的自由函数 `character_from_label` / `map_slot_from_label`
//!   （不改 P4 `domain::selection` 类型定义）；
//! - 旧 26 字节二进制槽位兼容走 `domain::persistence::SaveSlot`（类型家，见 c1 契约 §1），
//!   `from_legacy_slot` 做数字→稳定字符串保守映射，正式格式为 JSON v2（`saveload_json`）。
//!
//! 迁移（`docs/business/persistence.md` §迁移）：v1 缺 `version`/`bank`/`rng_seed` →
//! 补默认（`version=2`、`bank=0`、`rng_seed` 默认种子）；迁移纯函数幂等可测。

use serde::{Deserialize, Serialize};

use crate::domain::economy::Bank;
use crate::domain::flow::SceneId;
use crate::domain::map::GridPos as DomainGridPos;
use crate::domain::persistence::SaveSlot;
use crate::domain::selection::{Character, MapSlot};
use crate::rules::rng::GameRng;
use crate::rules::saveload_json::{
    character_from_label, legacy_character_label, legacy_map_label, map_slot_from_label,
};

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
    /// 退出时所在场景（`"Index"`/`"MainMap"` 等，与 `SceneId` 变体名一致）。
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
    /// 已建建筑 id 列表（稳定字符串，如 `"Castle"`）。
    #[serde(default)]
    pub buildings: Vec<String>,
    /// 持有商品（id + 数量）。
    #[serde(default)]
    pub goods: Vec<GoodsSave>,
    /// **复现关键**：RNG 当前种子（读档后 `GameRng::restore_state(rng_seed, rng_steps)` 续流）。
    #[serde(default = "default_rng_seed")]
    pub rng_seed: u32,
    /// RNG 已消费 `next_u64` 次数（精确续流；旧存档缺省 → 0 = 种子重置语义）。
    #[serde(default)]
    pub rng_steps: u128,
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
// ---- 跨包状态快照（c1 契约，`docs/contracts/c1_core_public_types.md`）----

/// 跨包状态快照（`domain` 公共类型，P1–P4 玩法层存档/读档桥接面）。
///
/// P1–P4 玩法层在存档/读档时通过 [`SaveSlotV2::capture_state`] /
/// [`SaveSlotV2::restore_state`] 桥接：纯函数，不持 session 引用，数值全走 `balance.json`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveState {
    /// 当前角色（`domain::selection::Character`，P4 选择域）。
    pub character: Option<Character>,
    /// 当前地图槽（`domain::selection::MapSlot`，P3 地图域）。
    pub map_slot: Option<MapSlot>,
    /// 玩家所在格（`domain::map::GridPos`，P3 地图域）。
    pub position: DomainGridPos,
    /// 银行（`domain::economy::Bank`，P2 经济域；存 `gold` 字段）。
    pub bank: Bank,
    /// 当前场景（`domain::flow::SceneId`，P4 流程域；`None` = 主菜单）。
    pub scene: Option<SceneId>,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            character: None,
            map_slot: None,
            position: DomainGridPos::new(0, 0),
            bank: Bank::default(),
            scene: None,
        }
    }
}

/// `SceneId` → 存档 `scene` 字段（稳定字符串，与 `SceneId` 变体名一致）。
fn scene_id_label(s: SceneId) -> String {
    match s {
        SceneId::Intro => "Intro".to_owned(),
        SceneId::SelectRole => "SelectRole".to_owned(),
        SceneId::SelectMap => "SelectMap".to_owned(),
        SceneId::Index => "Index".to_owned(),
        SceneId::MainMap => "MainMap".to_owned(),
        SceneId::Vs => "Vs".to_owned(),
    }
}

/// 存档 `scene` 字符串 → `SceneId`（未知字符串 → `None`，不 panic）。
fn scene_label_to_id(label: &str) -> Option<SceneId> {
    Some(match label {
        "Intro" => SceneId::Intro,
        "SelectRole" => SceneId::SelectRole,
        "SelectMap" => SceneId::SelectMap,
        "Index" => SceneId::Index,
        "MainMap" => SceneId::MainMap,
        "Vs" => SceneId::Vs,
        _ => return None,
    })
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
            rng_steps: 0,
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

    /// 读档后注入 RNG（`docs/business/persistence.md` §RNG「读档后可续」）。
    ///
    /// S12.5：使用 `restore_state(rng_seed, rng_steps)` 精确续流
    ///（`rng_steps` 缺省 0 = 种子重置，等价旧 `GameRng::new(rng_seed)`）。
    #[must_use]
    pub fn rng(&self) -> GameRng {
        GameRng::restore_state(self.rng_seed, self.rng_steps)
    }

    /// 是否已迁移到 v2（`version == 2`）。
    #[must_use]
    pub fn is_v2(&self) -> bool {
        self.version == 2
    }

    /// 从旧 26 字节二进制 `SaveSlot`（`domain::persistence` 类型家）构造 v2 存档
    /// （迁移路径，数字→稳定字符串保守映射；正式格式为 JSON v2，二进制布局仅供历史对拍）。
    ///
    /// legacy 数字口径：`character` / `map_slot` 均 1 起（`1..=5` / `1..=3`，原版 26 字节槽位）；
    /// 映射到可解析标签：角色 → 角色 label（`character_from_label` 可回读），地图 → `m0N`
    /// （`map_slot_from_label` 可回读）；`0`/越界保守钳位到首项（门禁红线不 panic）。
    #[must_use]
    pub fn from_legacy_slot(slot: &SaveSlot) -> Self {
        Self {
            version: 2,
            character: legacy_character_label(slot.character),
            map: legacy_map_label(slot.map_slot),
            scene: "Index".to_owned(),
            position: GridPos { row: slot.position, col: 0 },
            gold: slot.gold,
            hp: 0,
            buildings: Vec::new(),
            goods: Vec::new(),
            rng_seed: DEFAULT_RNG_SEED,
            rng_steps: 0,
            bank: 0,
            unlocked_maps: Vec::new(),
        }
    }

    /// 将 P1–P4 玩法层状态写入存档（纯函数，不改 RNG 流；`scene` 为 `None` 时写 `"Index"`）。
    ///
    /// 用法：P4 流程层在切场景前调 `save.capture_state(st)` 把当前状态打进 `SaveSlotV2`；
    /// 存档字段全部来自 `domain` 公共类型（c1 契约），不直接访问 P1–P4 session 私有字段。
    pub fn capture_state(&mut self, st: &SaveState) {
        if let Some(c) = st.character {
            self.character = c.label().into();
        }
        if let Some(m) = st.map_slot {
            self.map = format!("m{:02}", m.index());
        }
        self.position = GridPos { row: st.position.row, col: st.position.col };
        self.bank = st.bank.gold;
        let index = "Index".to_owned();
        self.scene = st.scene.map_or_else(|| index.clone(), scene_id_label);
    }

    /// 从存档还原 P1–P4 玩法层状态（`None` = 未知/越界，不 panic，门禁红线）。
    ///
    /// 用法：P4 流程层在读档后调 `save.restore_state()` 拿回 `SaveState`，
    /// 注入各域 session 重建当前状态（`character_from_label` / `map_slot_from_label` 查表）。
    #[must_use]
    pub fn restore_state(&self) -> SaveState {
        SaveState {
            character: character_from_label(&self.character),
            map_slot: map_slot_from_label(&self.map),
            position: DomainGridPos::new(self.position.row, self.position.col),
            bank: Bank::new(self.bank),
            scene: scene_label_to_id(&self.scene),
        }
    }

    /// 从 P1–P4 玩法层状态 + RNG 精确状态构造完整存档（`capture_state` + `rng_seed`/`rng_steps`）。
    ///
    /// 用法：P4 流程层存档时调 `SaveSlotV2::from_state(st, rng_seed, rng_steps)`，
    /// 一步完成「状态 → 存档」，避免多次调 `capture_state` 漏字段。
    #[must_use]
    pub fn from_state(st: &SaveState, rng_seed: u32, rng_steps: u128) -> Self {
        let mut s = Self::new(String::new(), String::new());
        s.rng_seed = rng_seed;
        s.rng_steps = rng_steps;
        s.capture_state(st);
        s
    }
}

/// S21.11 存档槽位（v3 = v2 + `boat_level` 船等级；JSON 显式字段）。
///
/// 迁移口径（`docs/business/persistence.md` §迁移 v3）：v2 旧档无 `boat_level` ⇒ 缺省
/// **1（小舢板）**（读档零报错，不臆造船等级）；船等级全局账号级（不随跨图重置）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotV3 {
    /// 版本号（v3 恒 3；v1/v2 → 迁移补 3）。
    #[serde(default = "default_version_v3")]
    pub version: u32,
    /// 角色 id（稳定字符串，如 `"drake"`）。
    pub character: String,
    /// 地图 id（如 `"m01"`）。
    pub map: String,
    /// 退出时所在场景（`"Index"`/`"MainMap"` 等，与 `SceneId` 变体名一致）。
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
    /// 已建建筑 id 列表（稳定字符串，如 `"Castle"`）。
    #[serde(default)]
    pub buildings: Vec<String>,
    /// 持有商品（id + 数量）。
    #[serde(default)]
    pub goods: Vec<GoodsSave>,
    /// RNG 当前种子（读档后 `GameRng::restore_state(rng_seed, rng_steps)` 续流）。
    #[serde(default = "default_rng_seed")]
    pub rng_seed: u32,
    /// RNG 已消费 `next_u64` 次数（精确续流；旧存档缺省 → 0 = 种子重置语义）。
    #[serde(default)]
    pub rng_steps: u128,
    /// 银行存金（v1 缺省 → 迁移补 0）。
    #[serde(default)]
    pub bank: u32,
    /// 已解锁地图 id 列表。
    #[serde(default)]
    pub unlocked_maps: Vec<String>,
    /// S21.11 船等级（1..=4；旧档缺省 = 1 小舢板）。
    #[serde(default = "default_boat_level_v3")]
    pub boat_level: u8,
}

/// v1/v2 旧档迁移到 v3 时的缺省版本号。
fn default_version_v3() -> u32 {
    3
}

/// v2 旧档（无 `boat_level`）缺省船等级 = 1（小舢板）。
fn default_boat_level_v3() -> u8 {
    1
}

impl SaveSlotV3 {
    /// v2 → v3 迁移（**纯函数、幂等**）：补 `version=3` + `boat_level=1`（小舢板）；
    /// 其余字段原样承接（不臆造船等级）。
    #[must_use]
    pub fn migrate_v2_to_v3(v2: &SaveSlotV2) -> Self {
        Self {
            version: 3,
            character: v2.character.clone(),
            map: v2.map.clone(),
            scene: v2.scene.clone(),
            position: v2.position,
            gold: v2.gold,
            hp: v2.hp,
            buildings: v2.buildings.clone(),
            goods: v2.goods.clone(),
            rng_seed: v2.rng_seed,
            rng_steps: v2.rng_steps,
            bank: v2.bank,
            unlocked_maps: v2.unlocked_maps.clone(),
            boat_level: default_boat_level_v3(),
        }
    }

    /// 自迁移（v1/v2 → v3；已是 v3 ⇒ 原样返回 ⇒ 幂等）。
    #[must_use]
    pub fn migrate(mut self) -> Self {
        if self.version < 3 {
            self.version = 3;
            self.boat_level = default_boat_level_v3();
        }
        self
    }

    /// 是否已迁移到 v3（`version == 3`）。
    #[must_use]
    pub fn is_v3(&self) -> bool {
        self.version == 3
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
}

// ------------------------------------------------------- 存档 v4（S28.2 领地系统）

pub use crate::domain::persistence::{SeatSave, TerritorySave};

/// 席位存档 / 领地存档条目类型家 = `domain::persistence`（S28.2；单一事实源）。
/// 存档槽位（v4 = v3 + 多席位 + 领地图；`docs/business/territory.md` §8）。
///
/// 迁移口径：v3 → v4 缺省 = **单席位（主角色）+ 4 个 AI 补位 + 空领地图 + `current_seat = 0`**；
/// v2 → v3 迁移链保持不变。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotV4 {
    /// 版本号（v4 恒 4；v1..v3 → 迁移补 4）。
    #[serde(default = "default_version_v4")]
    pub version: u32,
    /// 角色 id（v4 保留 = 主玩家席位角色；席位全量在 `seats`）。
    pub character: String,
    /// 地图 id（如 `"m01"`）。
    pub map: String,
    /// 退出时所在场景。
    #[serde(default = "default_scene")]
    pub scene: String,
    /// 主玩家所在格（v4 起席位位置走 `seats[].position`；本字段保留 v3 兼容）。
    #[serde(default)]
    pub position: GridPos,
    /// 主玩家金币（v4 起走 `seats[]`；本字段保留 v3 兼容）。
    #[serde(default)]
    pub gold: u32,
    /// 主玩家 HP。
    #[serde(default)]
    pub hp: u32,
    /// 已建建筑 id 列表（v4 起走 `seats[].buildings`；本字段保留 v3 兼容）。
    #[serde(default)]
    pub buildings: Vec<String>,
    /// 持有商品（id + 数量）。
    #[serde(default)]
    pub goods: Vec<GoodsSave>,
    /// RNG 当前种子。
    #[serde(default = "default_rng_seed")]
    pub rng_seed: u32,
    /// RNG 已消费 `next_u64` 次数。
    #[serde(default)]
    pub rng_steps: u128,
    /// 银行存金。
    #[serde(default)]
    pub bank: u32,
    /// 已解锁地图 id 列表。
    #[serde(default)]
    pub unlocked_maps: Vec<String>,
    /// 船等级（1..=4；缺省 1）。
    #[serde(default = "default_boat_level_v3")]
    pub boat_level: u8,
    /// S28.2 席位全量（缺省 = 空；读档按 §8 迁移口径补单席位 + 4 AI）。
    #[serde(default)]
    pub seats: Vec<SeatSave>,
    /// S28.2 领地图（缺省 = 空 = 全图无主）。
    #[serde(default)]
    pub territories: Vec<TerritorySave>,
    /// S28.2 当前轮到哪个席位（热座交接用；缺省 0）。
    #[serde(default)]
    pub current_seat: u8,
}

/// v4 缺省版本号。
fn default_version_v4() -> u32 {
    4
}

impl SaveSlotV4 {
    /// v3 → v4 迁移（**纯函数、幂等**；§8 缺省口径 = 单席位（主角色，玩家）+ 4 AI 补位 +
    /// 空领地图 + `current_seat = 0`；席位金币/位置/建筑/船等级从 v3 单角色字段承接）。
    #[must_use]
    pub fn migrate_v3_to_v4(v3: &SaveSlotV3) -> Self {
        Self {
            version: 4,
            character: v3.character.clone(),
            map: v3.map.clone(),
            scene: v3.scene.clone(),
            position: v3.position,
            gold: v3.gold,
            hp: v3.hp,
            buildings: v3.buildings.clone(),
            goods: v3.goods.clone(),
            rng_seed: v3.rng_seed,
            rng_steps: v3.rng_steps,
            bank: v3.bank,
            unlocked_maps: v3.unlocked_maps.clone(),
            boat_level: v3.boat_level,
            seats: Vec::new(),
            territories: Vec::new(),
            current_seat: 0,
        }
    }

    /// 自迁移（v1..v3 → v4；已是 v4 ⇒ 原样返回 ⇒ 幂等）。
    #[must_use]
    pub fn migrate(mut self) -> Self {
        if self.version < 4 {
            self.version = 4;
        }
        self
    }

    /// 是否已迁移到 v4（`version == 4`）。
    #[must_use]
    pub fn is_v4(&self) -> bool {
        self.version == 4
    }

    /// 序列化存档为 JSON 字符串（失败返回空串，不 panic）。
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// 从 JSON 解析存档槽（缺字段走 `#[serde(default)]`；解析失败 → `None`，不 panic）。
    #[must_use]
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

// ------------------------------------------------------- 存档 v5（S33.4 领地 v2）

/// 存档槽位（v5 = v4 + 席位人手/武器 + 领地驻军；`docs/business/territory.md` §8）。
///
/// 迁移口径：v4 → v5 = 席位补默认人手/武器（`balance.manpower.initial_crew/arms`）+ 领地驻军空；
/// v3 → v4 迁移链保持不变。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotV5 {
    /// 版本号（v5 恒 5；v1..v4 → 迁移补 5）。
    #[serde(default = "default_version_v5")]
    pub version: u32,
    /// 角色 id（主玩家席位角色；席位全量在 `seats`）。
    pub character: String,
    /// 地图 id（如 `"m01"`）。
    pub map: String,
    /// 退出时所在场景。
    #[serde(default = "default_scene")]
    pub scene: String,
    /// 主玩家所在格（v4 起席位位置走 `seats[].position`；保留 v3 兼容）。
    #[serde(default)]
    pub position: GridPos,
    /// 主玩家金币（v4 起走 `seats[]`；保留 v3 兼容）。
    #[serde(default)]
    pub gold: u32,
    /// 主玩家 HP。
    #[serde(default)]
    pub hp: u32,
    /// 已建建筑 id 列表（v4 起走 `seats[].buildings`；保留 v3 兼容）。
    #[serde(default)]
    pub buildings: Vec<String>,
    /// 持有商品（id + 数量）。
    #[serde(default)]
    pub goods: Vec<GoodsSave>,
    /// RNG 当前种子。
    #[serde(default = "default_rng_seed")]
    pub rng_seed: u32,
    /// RNG 已消费 `next_u64` 次数。
    #[serde(default)]
    pub rng_steps: u128,
    /// 银行存金。
    #[serde(default)]
    pub bank: u32,
    /// 已解锁地图 id 列表。
    #[serde(default)]
    pub unlocked_maps: Vec<String>,
    /// 船等级（1..=4；缺省 1）。
    #[serde(default = "default_boat_level_v3")]
    pub boat_level: u8,
    /// 席位全量（缺省 = 空；读档按 §8 迁移口径补单席位 + 4 AI）。
    #[serde(default)]
    pub seats: Vec<SeatSave>,
    /// 领地图（缺省 = 空 = 全图无主）。
    #[serde(default)]
    pub territories: Vec<TerritorySave>,
    /// 当前轮到哪个席位（热座交接用；缺省 0）。
    #[serde(default)]
    pub current_seat: u8,
}

/// v5 缺省版本号。
fn default_version_v5() -> u32 {
    5
}

impl SaveSlotV5 {
    /// v4 → v5 迁移（**纯函数、幂等**；§8 缺省口径 = 席位补默认人手/武器 + 领地驻军空；
    /// v4 席位 `crew/arms` 为 0（serde default）时补 `balance.manpower.initial_crew/arms`）。
    #[must_use]
    pub fn migrate_v4_to_v5(v4: &SaveSlotV4) -> Self {
        let default_crew = crate::data::balance().map_or(20, |b| b.manpower.initial_crew);
        let default_arms = crate::data::balance().map_or(5, |b| b.manpower.initial_arms);
        let seats = v4
            .seats
            .iter()
            .cloned()
            .map(|mut s| {
                if s.crew == 0 {
                    s.crew = default_crew;
                }
                if s.arms == 0 {
                    s.arms = default_arms;
                }
                s
            })
            .collect();
        let territories = v4
            .territories
            .iter()
            .cloned()
            .map(|mut t| {
                t.garrisons.clear();
                t
            })
            .collect();
        Self {
            version: 5,
            character: v4.character.clone(),
            map: v4.map.clone(),
            scene: v4.scene.clone(),
            position: v4.position,
            gold: v4.gold,
            hp: v4.hp,
            buildings: v4.buildings.clone(),
            goods: v4.goods.clone(),
            rng_seed: v4.rng_seed,
            rng_steps: v4.rng_steps,
            bank: v4.bank,
            unlocked_maps: v4.unlocked_maps.clone(),
            boat_level: v4.boat_level,
            seats,
            territories,
            current_seat: v4.current_seat,
        }
    }

    /// 自迁移（v1..v4 → v5；已是 v5 ⇒ 原样返回 ⇒ 幂等）。
    #[must_use]
    pub fn migrate(mut self) -> Self {
        if self.version < 5 {
            self.version = 5;
        }
        self
    }

    /// 是否已迁移到 v5（`version == 5`）。
    #[must_use]
    pub fn is_v5(&self) -> bool {
        self.version == 5
    }

    /// 序列化存档为 JSON 字符串（失败返回空串，不 panic）。
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// 从 JSON 解析存档槽（缺字段走 `#[serde(default)]`；解析失败 → `None`，不 panic）。
    #[must_use]
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
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
    ///
    /// S12.5：使用 `restore_state(rng_seed, rng_steps)` 精确续流。
    #[must_use]
    pub fn rng_for_slot(&self, index: usize) -> GameRng {
        match self.slots.get(index) {
            Some(s) => s.rng(),
            None => GameRng::new(DEFAULT_RNG_SEED),
        }
    }
}
