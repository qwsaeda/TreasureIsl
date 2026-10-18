//! 持久化域类型（业务文档：`docs/business/persistence.md`）。
//!
//! 存档槽（JSON v2 字段定义）+ 错误类型 + 常量。
//! 编解码实现（`rules::saveload_json`）走 serde JSON v2，迁移规则在 `rules::saveload`。

/// 存档槽位（JSON v2 schema，业务文档 `docs/business/persistence.md` §字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
