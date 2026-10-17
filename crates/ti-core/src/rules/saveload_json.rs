//! S3.10 / S8.3.d / S12.5 存读档正式实现（JSON v2 + 可复现 RNG）。
//!
//! 业务规格（唯一事实源）：[`docs/business/persistence.md`](../../../docs/business/persistence.md)。
//!
//! S12.5 存档续流 + 跨包状态桥：
//! - **RNG 精确续流**：`rng_seed` + `rng_steps`（已消费 `next_u64` 次数）双字段，
//!   读档后 `GameRng::restore_state(rng_seed, rng_steps)` O(log n) 跳转；
//!   旧存档缺 `rng_steps` → `serde(default)` 补 0 = 种子重置语义（向后兼容）；
//! - **跨包状态桥**：[`SaveState`] + `capture_state`/`restore_state` 以 `domain` 公共类型
//!   （`Character`/`MapSlot`/`GridPos`/`Bank`/`SceneId`）为参数，纯函数不持 session 引用；
//!   字符串↔枚举转换走本模块自由函数 `character_from_label` / `map_slot_from_label`
//!   （不改 P4 `domain::selection` 类型定义，c1 契约约束）；
//! - 旧 26 字节二进制槽位兼容走 `domain::persistence::SaveSlot`（类型家，见 c1 契约 §1），
//!   `from_legacy_slot` 做数字→稳定字符串保守映射，正式格式为 JSON v2（本模块）。
//!
//! 设计要点：
//! - **存档 = 运行时状态**（角色/地图/场景/位置/金/HP/建筑/商品/RNG 种子+步数/银行/解锁地图）；
//! - **v1 → v2 迁移**：纯函数幂等，补 `version=2`/`bank=0`/`rng_seed` 默认；
//! - **无占位 / 不 panic**：解析失败返回 [`SaveErr`]，不静默 fallback（门禁红线）。
//!
//! 跨包 import（c1 契约）：`ti_core::domain::{Character, MapSlot, GridPos, Bank, SceneId}`；
//! 数值/文案走 `balance.save` / `strings.json`（禁写死数值）。

use serde::{Deserialize, Serialize};

use crate::domain::economy::Bank;
use crate::domain::flow::SceneId;
use crate::domain::map::GridPos;
use crate::domain::persistence::SaveSlot;
use crate::domain::selection::{Character, MapSlot};
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveV2 {
    /// 存档版本（v2 恒 2；v1 缺省 → 迁移补 2）。
    #[serde(default = "default_version")]
    pub version: u32,
    /// 角色 id（稳定字符串，如 `"drake"`）。
    pub character: String,
    /// 地图 id（如 `"m01"`）。
    pub map: String,
    /// 退出时所在场景（`"Index"`/`"MainMap"`/`"Vs"` 等，与 `SceneId` 变体名一致）。
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
    1 // 缺省按 v1，由 `migrate` 补成 2
}

fn default_scene() -> String {
    "Index".to_owned()
}

fn default_rng_seed() -> u32 {
    DEFAULT_RNG_SEED
}

// ---- 字符串↔枚举 自由函数（c1 契约：不改 P4 domain 类型定义）----

/// 存档 `character` 字符串 → 角色（未知 → `None`，门禁红线不 panic）。
#[must_use]
pub fn character_from_label(label: &str) -> Option<Character> {
    Some(match label {
        "panda" => Character::Panda,
        "tichy" => Character::Tichy,
        "kidd" => Character::Kidd,
        "drake" => Character::Drake,
        "katharina" => Character::Katharina,
        _ => return None,
    })
}

/// 存档 `map` 字符串（`"m01"`/`"m02"`/`"m03"`）→ 地图槽（未知 → `None`，门禁红线不 panic）。
#[must_use]
pub fn map_slot_from_label(label: &str) -> Option<MapSlot> {
    Some(match label {
        "m01" => MapSlot::M01,
        "m02" => MapSlot::M02,
        "m03" => MapSlot::M03,
        _ => return None,
    })
}

/// legacy 数字角色（1 起，`1..=5`）→ 角色 label（`0`/越界保守落首角色 `panda`，可回读）。
#[must_use]
pub fn legacy_character_label(n: u32) -> String {
    Character::from_index(n.saturating_sub(1)).map_or("panda".to_owned(), |c| c.label().to_owned())
}

/// legacy 数字地图（1 起，`1..=3`）→ `m0N`（`0`/越界钳到 `m01..m03`，可回读）。
#[must_use]
pub fn legacy_map_label(n: u32) -> String {
    format!("m{:02}", n.clamp(1, 3))
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

// ---- 跨包状态快照（c1 契约，`docs/contracts/c1_core_public_types.md`）----

/// 跨包状态快照（`domain` 公共类型，P1–P4 玩法层存档/读档桥接面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveState {
    /// 当前角色（`domain::selection::Character`，P4 选择域）。
    pub character: Option<Character>,
    /// 当前地图槽（`domain::selection::MapSlot`，P3 地图域）。
    pub map_slot: Option<MapSlot>,
    /// 玩家所在格（`domain::map::GridPos`，P3 地图域）。
    pub position: GridPos,
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
            position: GridPos::new(0, 0),
            bank: Bank::default(),
            scene: None,
        }
    }
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
            rng_steps: 0,
            bank: 0,
            unlocked_maps: Vec::new(),
        }
    }

    /// 从 P1–P4 玩法层状态 + RNG 精确状态构造完整存档（`capture_state` + `rng_seed`/`rng_steps`）。
    #[must_use]
    pub fn from_state(st: &SaveState, rng_seed: u32, rng_steps: u128) -> Self {
        let mut s = Self::new(String::new(), String::new());
        s.rng_seed = rng_seed;
        s.rng_steps = rng_steps;
        s.capture_state(st);
        s
    }

    /// 将 P1–P4 玩法层状态写入存档（纯函数，不改 RNG 流；`scene` 为 `None` 时写 `"Index"`）。
    pub fn capture_state(&mut self, st: &SaveState) {
        if let Some(c) = st.character {
            self.character = c.label().into();
        }
        if let Some(m) = st.map_slot {
            self.map = format!("m{:02}", m.index());
        }
        self.position = GridPosSave { row: st.position.row, col: st.position.col };
        self.bank = st.bank.gold;
        let index = "Index".to_owned();
        self.scene = st.scene.map_or_else(|| index.clone(), scene_id_label);
    }

    /// 从存档还原 P1–P4 玩法层状态（`None` = 未知/越界，不 panic，门禁红线）。
    #[must_use]
    pub fn restore_state(&self) -> SaveState {
        SaveState {
            character: character_from_label(&self.character),
            map_slot: map_slot_from_label(&self.map),
            position: GridPos::new(self.position.row, self.position.col),
            bank: Bank::new(self.bank),
            scene: scene_label_to_id(&self.scene),
        }
    }

    /// 序列化存档为 JSON 字符串（人可读、可 diff；失败返回 `Err` 形状错误）。
    ///
    /// # Errors
    /// JSON 序列化失败 → [`SaveErr::NotAnObject`]。
    pub fn to_json(&self) -> Result<String, SaveErr> {
        serde_json::to_string(self).map_err(|_| SaveErr::NotAnObject)
    }

    /// 从 JSON 解析存档（缺字段走 `#[serde(default)]`，未知版本由调用方 `migrate` 兜底）。
    ///
    /// # Errors
    /// 顶层非对象 / 语法错 → [`SaveErr::NotAnObject`]。
    pub fn from_json(json: &str) -> Result<Self, SaveErr> {
        serde_json::from_str(json).map_err(|_| SaveErr::NotAnObject)
    }

    /// v1 → v2 迁移（纯函数、幂等）：
    /// - `version < 2` → 补 `version=2`、`bank=0`、`rng_seed=DEFAULT_RNG_SEED`；
    /// - `version == 2` → 原样返回（幂等）；
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
        self.version = 2;
        self.bank = 0;
        self.rng_seed = DEFAULT_RNG_SEED;
        Ok(self)
    }

    /// 读档后注入 RNG（`docs/business/persistence.md` §RNG「读档后可续」）。
    ///
    /// S12.5：使用 `restore_state(rng_seed, rng_steps)` 精确续流
    ///（`rng_steps` 缺省 0 = 种子重置，等价旧 `GameRng::new(rng_seed)`）。
    #[must_use]
    pub fn rng(&self) -> GameRng {
        GameRng::restore_state(self.rng_seed, self.rng_steps)
    }

    /// 存档是否已迁移到 v2（`version == 2`）。
    #[must_use]
    pub fn is_v2(&self) -> bool {
        self.version == 2
    }
}

/// 存档管理器：持有槽位列表 + JSON 序列化/反序列化 + 迁移 + RNG 续流。
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

// ---- 与旧 26 字节二进制 `SaveSlot`（`domain::persistence` 类型家）的互转（历史兼容路径，非运行时依赖）----

/// 从旧 26 字节二进制 `SaveSlot`（`domain::persistence` 类型家）构造 v2 存档
/// （迁移路径，数字→稳定字符串保守映射；正式格式为 JSON v2，二进制布局仅供历史对拍）。
///
/// legacy 数字口径：`character` / `map_slot` 均 1 起（`1..=5` / `1..=3`，原版 26 字节槽位）；
/// 映射到可解析标签：角色 → 角色 label（`character_from_label` 可回读），地图 → `m0N`
/// （`map_slot_from_label` 可回读）；`0`/越界保守钳位到首项（门禁红线不 panic）。
#[must_use]
pub fn from_legacy_slot(slot: &SaveSlot) -> SaveV2 {
    SaveV2 {
        version: 2,
        character: legacy_character_label(slot.character),
        map: legacy_map_label(slot.map_slot),
        scene: "Index".to_owned(),
        position: GridPosSave { row: slot.position, col: 0 },
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
