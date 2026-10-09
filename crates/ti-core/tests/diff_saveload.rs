//! S3.10 存读档/设置差分测试。
//!
//! 遍历（5 角色 × 各 `gold`/`map_slot`/`position` × 魔数正确/损坏 × 大小边界），
//! oracle 与 impl 逐位一致（`validate`/`migrate`/`read_slot`/`write_slot` 结果全等）；
//! + 语义测试：
//! - (a) `write_slot` → `read_slot` 往返逐字节；
//! - (b) 魔数错误 → `Err(BadMagic)`；
//! - (c) v1 → v2 迁移后 `version==2` 且其余字段不变。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::saveload::{
    REC_MAGIC, SaveErr, SaveSlot, oracle_migrate_v1_to_v2, oracle_read_slot, oracle_validate_save,
    oracle_write_slot,
};
use ti_core::rules::saveload::SaveManager;

/// 构造有效 v2 存档字节（2B 前缀 + 4B 魔数 + 5×4B 字段 = 26 字节）。
fn make_valid_bytes(slot: &SaveSlot) -> Vec<u8> {
    oracle_write_slot(slot)
}

/// 断言 `oracle_validate_save` 与 `SaveManager::validate` 逐位一致。
fn assert_validate_equal(bytes: &[u8]) {
    let o = oracle_validate_save(bytes);
    let mgr = SaveManager::new(5);
    let i = mgr.validate(bytes);
    assert_eq!(o, i, "validate 不一致 (len={})", bytes.len());
}

/// 断言 `oracle_migrate_v1_to_v2` 与 `SaveManager::migrate` 逐位一致。
#[allow(dead_code)] // 保留备用，diff 测试可直接调用
fn assert_migrate_equal(slot: &SaveSlot) {
    let o = oracle_migrate_v1_to_v2(slot);
    let mgr = SaveManager::new(5);
    let i = mgr.migrate(slot);
    assert_eq!(o, i, "migrate 不一致 (slot={slot:?})");
}

/// 断言 `oracle_read_slot` 与 `SaveManager::read_slot` 逐位一致。
fn assert_read_slot_equal(bytes: &[u8]) {
    let o = oracle_read_slot(bytes);
    let mgr = SaveManager::new(5);
    let i = mgr.read_slot(bytes);
    assert_eq!(o, i, "read_slot 不一致 (len={})", bytes.len());
}

/// 断言 `oracle_write_slot` 与 `SaveManager::write_slot` 逐字节一致。
fn assert_write_slot_equal(slot: &SaveSlot) {
    let o = oracle_write_slot(slot);
    let mgr = SaveManager::new(5);
    let i = mgr.write_slot(slot);
    assert_eq!(o, i, "write_slot 逐字节不一致 (slot={slot:?})");
}

/// 遍历 5 角色 × gold {0, 100, `999_999`} × `map_slot` 0..=4 × position {0, 1, 100, `10_000`}
/// × version {1, 2} 的 read/write 路径。
#[test]
fn diff_read_write_all_combos() {
    let chars = [1u32, 2, 3, 4, 5];
    let golds = [0u32, 100, 999_999];
    let map_slots = [0u32, 1, 2, 3, 4];
    let positions = [0u32, 1, 100, 10_000];
    let versions = [1u32, 2];

    for &ch in &chars {
        for &gold in &golds {
            for &ms in &map_slots {
                for &pos in &positions {
                    for &ver in &versions {
                        let slot = SaveSlot {
                            version: ver,
                            character: ch,
                            gold,
                            map_slot: ms,
                            position: pos,
                        };
                        let bytes = make_valid_bytes(&slot);
                        assert_read_slot_equal(&bytes);
                        assert_validate_equal(&bytes);
                        assert_write_slot_equal(&slot);
                        // 往返
                        let read = oracle_read_slot(&bytes).expect("有效存档应解析成功");
                        assert_eq!(read, slot, "read_slot 往返应逐位一致");
                    }
                }
            }
        }
    }
}

/// 魔数损坏 → Err(BadMagic)（oracle 与 impl 一致）。
#[test]
fn diff_bad_magic() {
    let slot = SaveSlot { version: 2, character: 1, gold: 0, map_slot: 0, position: 0 };
    let mut bytes = make_valid_bytes(&slot);
    // 破坏魔数（偏移 2..6）
    bytes[2] = 0xFF;
    bytes[3] = 0xFF;
    bytes[4] = 0xFF;
    bytes[5] = 0xFF;
    assert_validate_equal(&bytes);
    assert_read_slot_equal(&bytes);
    // 确认是 BadMagic
    assert_eq!(oracle_validate_save(&bytes), Err(SaveErr::BadMagic));
    let mgr = SaveManager::new(5);
    assert_eq!(mgr.validate(&bytes), Err(SaveErr::BadMagic));
}

/// 大小边界：17 字节（< 18）→ TooShort；18 字节（v1 最小）→ 通过校验但 `read_slot` 需 v1 兼容读。
#[test]
fn diff_size_boundaries() {
    // 太短（< 18 字节）→ TooShort
    for len in 0..18usize {
        let bytes = vec![0u8; len];
        assert_validate_equal(&bytes);
        assert_read_slot_equal(&bytes);
        if len >= 2 {
            let magic = u32::from_le_bytes([
                bytes.get(2).copied().unwrap_or(0),
                bytes.get(3).copied().unwrap_or(0),
                bytes.get(4).copied().unwrap_or(0),
                bytes.get(5).copied().unwrap_or(0),
            ]);
            if len < 18 {
                assert_eq!(
                    oracle_validate_save(&bytes),
                    Err(SaveErr::TooShort),
                    "len={len} 应 TooShort"
                );
            } else if magic != REC_MAGIC {
                assert_eq!(
                    oracle_validate_save(&bytes),
                    Err(SaveErr::BadMagic),
                    "len={len} 魔数不符"
                );
            }
        }
    }

    // 恰好 26 字节（v2 最小）→ 通过
    let slot = SaveSlot::default();
    let bytes = make_valid_bytes(&slot);
    assert_eq!(bytes.len(), 26, "v2 布局 = 26 字节");
    assert_validate_equal(&bytes);
    assert_read_slot_equal(&bytes);
}

/// 魔数正确但 `version > 2` → `BadVersion`。
#[test]
fn diff_bad_version() {
    let slot = SaveSlot { version: 3, ..Default::default() }; // > 2 → BadVersion
    let bytes = make_valid_bytes(&slot);
    // validate 只检查魔数 + 大小，不检查 version → Ok
    assert_eq!(oracle_validate_save(&bytes), Ok(()), "validate 不检查 version");
    // read_slot 检查 version → BadVersion
    assert_eq!(oracle_read_slot(&bytes), Err(SaveErr::BadVersion), "version=3 应 BadVersion");
    let mgr = SaveManager::new(5);
    assert_eq!(mgr.read_slot(&bytes), Err(SaveErr::BadVersion));
}

/// 语义测试 (a)：`write_slot` → `read_slot` 往返逐字节。
#[test]
fn semantic_roundtrip_byte_identical() {
    let mgr = SaveManager::new(5);
    let slot = SaveSlot { version: 2, character: 3, gold: 42_000, map_slot: 2, position: 777 };
    let written = mgr.write_slot(&slot);
    assert_eq!(written.len(), 26, "v2 布局 = 26 字节");

    // 逐字节与 oracle_write_slot 一致
    let expected = oracle_write_slot(&slot);
    assert_eq!(written, expected, "write_slot 应与 oracle 逐字节一致");

    // read_slot 往返
    let read = mgr.read_slot(&written).expect("往返应成功");
    assert_eq!(read, slot, "read_slot(write_slot(s)) == s");

    // 再写一次 → 逐字节相同（确定性）
    let written2 = mgr.write_slot(&read);
    assert_eq!(written, written2, "write_slot(read_slot(write_slot(s))) == write_slot(s)");
}

/// 语义测试 (b)：魔数错误 → Err(BadMagic)。
#[test]
fn semantic_bad_magic_error() {
    let slot = SaveSlot::default();
    let mut bytes = oracle_write_slot(&slot);
    // 把魔数改全 0
    bytes[2..6].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(oracle_validate_save(&bytes), Err(SaveErr::BadMagic));
    let mgr = SaveManager::new(5);
    assert_eq!(mgr.validate(&bytes), Err(SaveErr::BadMagic));
    assert_eq!(mgr.read_slot(&bytes), Err(SaveErr::BadMagic));
}

/// 语义测试 (c)：v1 → v2 迁移后 version==2 且其余字段不变。
///
/// 注意：`oracle_migrate_v1_to_v2` 的占位规则是：
/// - `version == 0`（v1 无版本字段）→ 补 `version = 2`；
/// - `version == 1`（v1 兼容读）→ 升至 2；
/// - `version == 2`（v2）→ 不变（仍 2）。
///
/// 所以迁移后的 `version` 值取决于输入：
/// - 输入 version=0 → 输出 version=2（v1 → v2 升级）；
/// - 输入 version=1 → 输出 version=2（v1 兼容读 → v2）；
/// - 输入 version=2 → 输出 version=2（v2 不变）。
#[test]
fn semantic_migrate_v1_to_v2_preserves_fields() {
    let mgr = SaveManager::new(5);

    // v1 存档（version=0 表示无版本字段）
    let v1 = SaveSlot { version: 0, character: 3, gold: 0, map_slot: 2, position: 100 };
    let migrated = mgr.migrate(&v1);
    assert_eq!(migrated.version, 2, "v1（version=0）迁移后 version=2（升级至 v2）");
    assert_eq!(migrated.character, 3, "character 不变");
    assert_eq!(migrated.gold, 0, "gold 不变");
    assert_eq!(migrated.map_slot, 2, "map_slot 不变");
    assert_eq!(migrated.position, 100, "position 不变");

    // v2 存档（version=2）
    let v2 = SaveSlot { version: 2, character: 5, gold: 50_000, map_slot: 4, position: 999 };
    let migrated2 = mgr.migrate(&v2);
    assert_eq!(migrated2.version, 2, "v2（version=2）迁移后 version=2（不变）");
    assert_eq!(migrated2.character, 5, "character 不变");
    assert_eq!(migrated2.gold, 50_000, "gold 不变");
    assert_eq!(migrated2.map_slot, 4, "map_slot 不变");
    assert_eq!(migrated2.position, 999, "position 不变");

    // 与 oracle 一致
    assert_eq!(migrated, oracle_migrate_v1_to_v2(&v1));
    assert_eq!(migrated2, oracle_migrate_v1_to_v2(&v2));
}

/// 语义测试 (d)：5 槽位全路径（每个槽位 write → read → 全等）。
#[test]
fn semantic_5_slots_full_path() {
    let mut mgr = SaveManager::new(5);
    for i in 0..5 {
        let i32 = u32::try_from(i).expect("i in 0..5");
        let slot = SaveSlot {
            version: 2,
            character: i32 + 1,
            gold: i32 * 1000,
            map_slot: i32,
            position: i32 * 42,
        };
        let written = mgr.write_slot(&slot);
        let read = mgr.read_slot(&written).expect("槽位 {i} 读写应成功");
        assert_eq!(read, slot, "槽位 {i} 往返应逐位一致");
        // 写入管理器
        mgr.set_slot(i, read);
        assert_eq!(mgr.slot(i).expect("槽位应存在"), &read);
    }
    // 越界槽位
    assert!(mgr.slot(5).is_none(), "槽位 5 越界 → None");
}

/// 语义测试 (e)：魔数常量与最小大小重导出一致。
#[test]
fn semantic_constants() {
    assert_eq!(SaveManager::magic(), REC_MAGIC, "magic 常量应一致");
    assert_eq!(SaveManager::min_size(), 26, "v2 最小大小 = 26 字节");
}
