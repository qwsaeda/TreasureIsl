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
use crate::rules::saveload_json::{character_from_label, map_slot_from_label};

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
