//! 持久化域类型（业务文档：`docs/business/persistence.md`）。
//!
//! 存档槽（JSON v2 字段定义）+ 错误类型 + 常量。
//! 编解码实现（`rules::saveload_json`）走 serde JSON v2，迁移规则在 `rules::saveload`。

/// 存档槽位（JSON v2 schema，业务文档 `docs/business/persistence.md` §字段）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SaveSlot {
    /// 存档版本（v1=1 / v2=2；`rules::saveload::migrate_v1_to_v2` 处理迁移）。
    pub version: u32,
    /// 角色索引（`1..=5`，对应 [`crate::domain::Character`] 顺序 +1）。
    pub character: u32,
    /// 当前金。
    pub gold: u32,
    /// 地图索引（`0..=2`，对应 [`crate::domain::MapSlot`] 的 0 起 `zero_index`，
    /// 与 `rules::saveload` 写档/读档（`saveload.rs::WriteSave`/`restore_save`）及
    /// `restore_save` 的 `map_slot < maps.len()` 0 起基准校验一致；`character` 与
    /// `map_slot` 基准不同：`character` 是 1 起 `index()+1`，`map_slot` 是 0 起 `zero_index`）。
    pub map_slot: u32,
    /// 玩家所在格索引（行主序 `row × grid_w + col`，写端 `WriteSave` 走 `map_pos().index(grid_w)`；
    /// `0` = 未写位置。读档位置恢复消费此值（Q008 口径统一）。
    pub position: u32,
    /// P1-8（S1）：RNG 已消费 `next_u64` 次数（精确续流；旧存档缺省 0 = 种子重置语义）。
    /// 读档后 `GameRng::restore_state(rng_seed, rng_steps)` O(log n) 跳转。
    #[serde(default)]
    pub rng_steps: u128,
    /// 跨图推进解锁位（bit i = 第 i 张图 m0(i+1) 已解锁；`docs/business/progression.md`
    /// §跨图推进）。`0` = 旧档（无此字段）→ 读档按 `map_slot` 的最小前置链回读
    /// （`ProgressionState::min_progression_to_select`），不臆造解锁。
    ///
    /// 为什么落进 schema（S15.2）：「在 m01 击败 BOSS 解锁 m02、尚未切图时存档」若只存
    /// `map_slot`，读档会退回仅 m01，跨图进度丢失；位掩码 1 字节即可保住整条解锁链。
    #[serde(default)]
    pub unlocked_mask: u8,
    /// S21.11 船等级（1..=4；`rules::boat::BoatSession` 等级链；`balance.boat` 数值单一事实源）。
    /// v2 旧档缺此字段 ⇒ 缺省 **1（小舢板）**，读档零报错（迁移口径 = `docs/business/persistence.md`
    /// §迁移 v3）；全局账号级（不随跨图重置）。
    #[serde(default = "default_boat_level")]
    pub boat_level: u8,
    /// S28.2 v4 扩展（`None` = v3 及更早档；旧档读回零报错，迁移口径 = `rules::saveload::migrate_v3_to_v4`）。
    #[serde(default)]
    pub v4: Option<std::boxed::Box<SaveSlotV4Ext>>,
}

/// S21.11 旧档（v2，无 `boat_level`）缺省船等级 = 1（小舢板）。
fn default_boat_level() -> u8 {
    1
}

// ------------------------------------------------------- 存档 v4 扩展（S28.2 领地系统）

/// 席位存档（`docs/business/territory.md` §8：每席位独立金币/位置/建筑/船等级）。
///
/// 类型家 = domain（`rules::saveload::SeatSave` 为再导出；单一事实源 = 本定义）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SeatSave {
    /// 角色 id（`balance.selection.characters[].id`，如 `"drake"`）。
    pub character: String,
    /// 玩家席位（`false` = AI 席位）。
    pub is_player: bool,
    /// 该席位金币。
    pub gold: u32,
    /// 该席位赛道格索引（`board.json` 0 起；`0` = 起点）。
    pub position: u32,
    /// 船等级（1..=4；缺省 1 小舢板）。
    #[serde(default = "default_boat_level_v3")]
    pub boat_level: u8,
    /// 该席位已建建筑 id 列表（建筑 = 个人成长线，§9）。
    #[serde(default)]
    pub buildings: Vec<String>,
}

/// 领地存档条目（`map → [格索引 → {owner_seat, level}]`，§8；无主格 = 缺省不写）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TerritorySave {
    /// 地图 id（如 `"m01"`）。
    pub map: String,
    /// 格索引 → `{owner_seat, level}`（`owner_seat` = 席位下标 `0..4`）。
    pub cells: std::collections::BTreeMap<u32, (u8, u8)>,
}

/// v4 扩展缺省船等级（与 `default_boat_level` 同口径）。
fn default_boat_level_v3() -> u8 {
    1
}

/// 存档 v4 扩展位（territory.md §8：多席位 + 领地图 + 当前席位；`None` = v3 及更早）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SaveSlotV4Ext {
    /// 5 席位全量（§1：players+ai=5；顺序 = 角色 index 升序）。
    #[serde(default)]
    pub seats: Vec<SeatSave>,
    /// 领地图（`map → 格 → {owner, level}`；空 = 全图无主）。
    #[serde(default)]
    pub territories: Vec<TerritorySave>,
    /// 当前轮到哪个席位（热座交接；缺省 0）。
    #[serde(default)]
    pub current_seat: u8,
}

impl Default for SaveSlot {
    fn default() -> Self {
        Self {
            version: 1,
            character: 1,
            gold: 0,
            map_slot: 0,
            position: 0,
            rng_steps: 0,
            unlocked_mask: 0,
            boat_level: 1,
            v4: None,
        }
    }
}

/// 存档错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveErr {
    /// JSON 语法错误。
    Syntax,
    /// 结构不符（缺必填字段 / 类型不匹配 / 取值越界）。
    Schema,
    /// 未知版本号（`version > 2`）。
    BadVersion,
    /// 魔数不符（二进制路径：`bytes[2..6]` ≠ `REC_MAGIC`）。
    BadMagic,
    /// 字节数不足最小有效大小（二进制路径：`len < MIN_SAVE_SIZE`）。
    TooShort,
}

/// 存档魔数（`REC_MAGIC` = `0x0131_5A47` = 20011591，实测见 `docs/reverse/facts/save.md`）。
pub const REC_MAGIC: u32 = 0x0131_5A47;

/// 存档最小有效大小（JSON v2 头部的定长字段部分下限，保留供二进制路径校验）。
#[allow(clippy::doc_markdown)]
pub const MIN_SAVE_SIZE: usize = 2 + 4 + 4 * 5;
