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
    /// 地图索引（`1..=3`，对应 [`crate::domain::MapSlot`] 顺序 +1）。
    pub map_slot: u32,
    /// 游戏进度（业务定义：已探索据点数）。
    pub position: u32,
}

impl Default for SaveSlot {
    fn default() -> Self {
        Self { version: 1, character: 1, gold: 0, map_slot: 0, position: 0 }
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
