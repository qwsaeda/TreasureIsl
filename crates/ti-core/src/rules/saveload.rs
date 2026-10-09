//! S3.10 存读档/设置正式实现。
//!
//! 与 `oracle::saveload` 行为逐位一致（同输入产生逐位相同的
//! 校验/迁移/读档/写档结果），但结构不同：用 `SaveManager`
//! 结构体 + 方法承载状态（槽位列表），而非自由函数。
//!
//! 依据 `docs/spec/save_load.md`：
//! - 存档槽 [`SaveSlot`]、魔数 [`REC_MAGIC`]、错误 [`SaveErr`]（从 oracle 重导出）；
//! - [`SaveManager`]：存档管理器（`slots: Vec<SaveSlot>`）；
//! - `validate` / `migrate` / `read_slot` / `write_slot` 方法。

use crate::oracle::saveload::{
    MIN_SAVE_SIZE, REC_MAGIC, SaveErr, SaveSlot, oracle_migrate_v1_to_v2, oracle_read_slot,
    oracle_validate_save, oracle_write_slot,
};

/// 存档管理器：持有槽位列表，提供校验/迁移/读档/写档。
#[derive(Debug, Clone)]
pub struct SaveManager {
    slots: Vec<SaveSlot>,
}

impl SaveManager {
    /// 新建存档管理器（指定槽位数，全部初始化为 `Default`）。
    #[must_use]
    pub fn new(slot_count: usize) -> Self {
        Self { slots: vec![SaveSlot::default(); slot_count] }
    }

    /// 槽位数（只读）。
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// 某槽位只读访问（越界 → `None`）。
    #[must_use]
    pub fn slot(&self, index: usize) -> Option<&SaveSlot> {
        self.slots.get(index)
    }

    /// 校验存档字节（行为与 `oracle_validate_save` 逐位一致，spec §d）。
    ///
    /// # Errors
    ///
    /// - [`SaveErr::TooShort`]：字节数不足最小有效大小；
    /// - [`SaveErr::BadMagic`]：魔数不符。
    #[allow(clippy::unused_self)]
    pub fn validate(&self, bytes: &[u8]) -> Result<(), SaveErr> {
        oracle_validate_save(bytes)
    }

    /// v1 → v2 迁移（行为与 `oracle_migrate_v1_to_v2` 逐位一致，spec §e）。
    ///
    /// 返回迁移后的 `SaveSlot`（`version` 已补，其余字段不变）。
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn migrate(&self, s: &SaveSlot) -> SaveSlot {
        oracle_migrate_v1_to_v2(s)
    }

    /// 从字节解析存档槽位（行为与 `oracle_read_slot` 逐位一致，spec §b）。
    ///
    /// # Errors
    ///
    /// - [`SaveErr::TooShort`]：字节数不足最小有效大小；
    /// - [`SaveErr::BadMagic`]：魔数不符；
    /// - [`SaveErr::BadVersion`]：未知版本号。
    #[allow(clippy::unused_self)]
    pub fn read_slot(&self, bytes: &[u8]) -> Result<SaveSlot, SaveErr> {
        oracle_read_slot(bytes)
    }

    /// 序列化为字节（行为与 `oracle_write_slot` 逐位一致，spec §b v2 布局）。
    ///
    /// 返回 26 字节（2 前缀 + 4 魔数 + 5×4 u32 字段），与 `read_slot` 往返逐字节一致。
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn write_slot(&self, s: &SaveSlot) -> Vec<u8> {
        oracle_write_slot(s)
    }

    /// 全部槽位（只读，spec §b 5 槽位）。
    #[must_use]
    pub fn slots(&self) -> &[SaveSlot] {
        &self.slots
    }

    /// 某槽位写入（越界 → 静默忽略，与 `SaveManager` 状态机语义一致）。
    pub fn set_slot(&mut self, index: usize, s: SaveSlot) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = s;
        }
    }

    /// 魔数常量（`REC_MAGIC` 重导出，spec §d）。
    #[must_use]
    pub const fn magic() -> u32 {
        REC_MAGIC
    }

    /// 最小存档大小（`MIN_SAVE_SIZE` 重导出，spec §d）。
    #[must_use]
    pub const fn min_size() -> usize {
        MIN_SAVE_SIZE
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_validate_good() {
        let mgr = SaveManager::new(5);
        let slot = SaveSlot { version: 2, character: 1, gold: 100, map_slot: 0, position: 50 };
        let bytes = mgr.write_slot(&slot);
        assert_eq!(mgr.validate(&bytes), Ok(()));
    }

    #[test]
    fn manager_migrate_v1() {
        let mgr = SaveManager::new(5);
        let v1 = SaveSlot { version: 0, character: 3, gold: 0, map_slot: 2, position: 100 };
        let migrated = mgr.migrate(&v1);
        assert_eq!(migrated.version, 2, "v1 迁移后 version = 2");
        assert_eq!(migrated.character, 3);
    }

    #[test]
    fn manager_roundtrip() {
        let mgr = SaveManager::new(5);
        for ch in 1..=5u32 {
            for gold in [0u32, 999_999] {
                for ms in 0..=4u32 {
                    let slot =
                        SaveSlot { version: 2, character: ch, gold, map_slot: ms, position: 42 };
                    let bytes = mgr.write_slot(&slot);
                    let read = mgr.read_slot(&bytes).unwrap_or_default();
                    assert_eq!(read, slot, "roundtrip 应逐位一致");
                }
            }
        }
    }

    #[test]
    fn manager_5_slots() {
        let mgr = SaveManager::new(5);
        assert_eq!(mgr.slot_count(), 5, "5 槽位");
        for i in 0..5 {
            assert!(mgr.slot(i).is_some(), "槽位 {i} 应存在");
        }
        assert!(mgr.slot(5).is_none(), "越界 → None");
    }

    #[test]
    fn manager_write_read_byte_identical() {
        let mgr = SaveManager::new(5);
        let slot = SaveSlot { version: 2, character: 2, gold: 500, map_slot: 3, position: 777 };
        let bytes = mgr.write_slot(&slot);
        assert_eq!(bytes.len(), 26, "v2 布局 = 26 字节");
        let expected = oracle_write_slot(&slot);
        assert_eq!(bytes, expected, "write_slot 应与 oracle_write_slot 逐字节一致");
    }
}
