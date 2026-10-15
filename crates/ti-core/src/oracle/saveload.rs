// oracle：存档二进制编解码差分基准，测试专用（S11 起降级为纯基准函数）。
//
// **类型已迁至 [`crate::domain::persistence`]**（S11 归档改造）。
// 本模块只保留**二进制存档路径**的差分基准函数（校验 / v1→v2 迁移 / 读写）。
// 正式存档格式是 **JSON v2**（`rules::saveload_json`），二进制路径仅供历史对拍。
//
// 业务规则以 `docs/business/persistence.md` 为准。

pub use crate::domain::persistence::{MIN_SAVE_SIZE, REC_MAGIC, SaveErr, SaveSlot};

/// oracle：校验存档字节（魔数 + 最小大小，spec §d）。
///
/// 规则（`存档读取` 行 92-101 直译 + spec §d 最小大小）：
/// - `bytes.len() < MIN_SAVE_SIZE` → `Err(TooShort)`；
/// - `bytes[2..6]`（跳过 2 字节前缀）的 u32 LE ≠ `REC_MAGIC` → `Err(BadMagic)`；
/// - 否则 → `Ok(())`。
///
/// # Errors
///
/// - [`SaveErr::TooShort`]：字节数不足最小有效大小；
/// - [`SaveErr::BadMagic`]：魔数不符。
pub fn oracle_validate_save(bytes: &[u8]) -> Result<(), SaveErr> {
    if bytes.len() < MIN_SAVE_SIZE {
        return Err(SaveErr::TooShort);
    }
    let magic = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    if magic != REC_MAGIC {
        return Err(SaveErr::BadMagic);
    }
    Ok(())
}

/// oracle：v1 → v2 迁移（spec §e，**推测**占位规则）。
///
/// 规则（spec §e 直译）：
/// - v1 存档（`version == 0` 或缺失）→ 补 `version = 2`；
/// - v2 存档（`version == 2`）→ 不变；
/// - 其余字段（`character`/`gold`/`map_slot`/`position`）不变。
///
/// 返回新的 `SaveSlot`（`version` 已迁移至 2），其余字段逐位不变。
#[must_use]
pub fn oracle_migrate_v1_to_v2(s: &SaveSlot) -> SaveSlot {
    let new_version = if s.version == 0 || s.version == 1 { 2 } else { s.version };
    SaveSlot {
        version: new_version,
        character: s.character,
        gold: s.gold,
        map_slot: s.map_slot,
        position: s.position,
        rng_steps: s.rng_steps,
    }
}

/// oracle：从字节解析存档槽位（spec §b v2 布局）。
///
/// 布局（v2，spec §b **推测** 简化）：
/// ```text
/// offset 0:  2B 前缀（跳过）
/// offset 2:  4B 魔数（u32 LE，= REC_MAGIC）
/// offset 6:  4B version（u32 LE，1=v1 兼容读 / 2=v2）
/// offset 10: 4B character（u32 LE）
/// offset 14: 4B gold（u32 LE）
/// offset 18: 4B map_slot（u32 LE）
/// offset 22: 4B position（u32 LE）
/// ```
///
/// # Errors
///
/// - [`SaveErr::TooShort`]：`bytes.len() < MIN_SAVE_SIZE`；
/// - [`SaveErr::BadMagic`]：魔数不符；
/// - [`SaveErr::BadVersion`]：`version > 2`。
pub fn oracle_read_slot(bytes: &[u8]) -> Result<SaveSlot, SaveErr> {
    oracle_validate_save(bytes)?;
    let version = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
    if version > 2 {
        return Err(SaveErr::BadVersion);
    }
    let character = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
    let gold = u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
    let map_slot = u32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);
    let position = u32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]);
    Ok(SaveSlot { version, character, gold, map_slot, position, rng_steps: 0 })
}

/// oracle：序列化为字节（与 `oracle_read_slot` 往返逐字节一致，spec §b v2 布局）。
///
/// 布局同 `oracle_read_slot`，写入 2 字节前缀（`0x00 0x00`，**推测**）+ 魔数 + 5 个 u32 字段。
#[must_use]
pub fn oracle_write_slot(slot: &SaveSlot) -> Vec<u8> {
    let mut buf = vec![0u8; MIN_SAVE_SIZE];
    buf[0..2].copy_from_slice(&[0x00, 0x00]); // 前缀（**推测**）
    buf[2..6].copy_from_slice(&REC_MAGIC.to_le_bytes());
    buf[6..10].copy_from_slice(&slot.version.to_le_bytes());
    buf[10..14].copy_from_slice(&slot.character.to_le_bytes());
    buf[14..18].copy_from_slice(&slot.gold.to_le_bytes());
    buf[18..22].copy_from_slice(&slot.map_slot.to_le_bytes());
    buf[22..26].copy_from_slice(&slot.position.to_le_bytes());
    buf
}

/// oracle：v1 兼容读（`version == 1` 时 `gold` 字段缺失 → 默认 0，**推测**）。
///
/// v1 布局（spec §e **推测**）：
/// ```text
/// offset 0:  2B 前缀
/// offset 2:  4B 魔数
/// offset 6:  4B character
/// offset 10: 4B map_slot
/// offset 14: 4B position
/// ```
/// v1 无 `version` 字段，读入后补 `version = 1`、`gold = 0`。
///
/// # Errors
///
/// 同 [`oracle_read_slot`]。
pub fn oracle_read_slot_v1(bytes: &[u8]) -> Result<SaveSlot, SaveErr> {
    if bytes.len() < 18 {
        return Err(SaveErr::TooShort);
    }
    let magic = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    if magic != REC_MAGIC {
        return Err(SaveErr::BadMagic);
    }
    let character = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
    let map_slot = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
    let position = u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
    Ok(SaveSlot { version: 1, character, gold: 0, map_slot, position, rng_steps: 0 })
}

/// 常量：`size_of::<SaveSlot>()`（diff 测试用，验证字段数一致性）。
#[must_use]
pub const fn slot_size() -> usize {
    size_of::<SaveSlot>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_good_save() {
        let slot = SaveSlot {
            version: 2,
            character: 1,
            gold: 100,
            map_slot: 0,
            position: 50,
            rng_steps: 0,
        };
        let bytes = oracle_write_slot(&slot);
        assert_eq!(oracle_validate_save(&bytes), Ok(()), "有效存档应通过校验");
    }

    #[test]
    fn validate_bad_magic() {
        let mut bytes = oracle_write_slot(&SaveSlot::default());
        bytes[2] = 0xFF; // 破坏魔数
        assert_eq!(oracle_validate_save(&bytes), Err(SaveErr::BadMagic));
    }

    #[test]
    fn validate_too_short() {
        let bytes = [0u8; 10];
        assert_eq!(oracle_validate_save(&bytes), Err(SaveErr::TooShort));
    }

    #[test]
    fn roundtrip() {
        for ch in 1..=5u32 {
            for gold in [0u32, 100, 999_999] {
                for ms in 0..=4u32 {
                    for pos in [0u32, 1, 100, 10_000] {
                        let slot = SaveSlot {
                            version: 2,
                            character: ch,
                            gold,
                            map_slot: ms,
                            position: pos,
                            rng_steps: 0,
                        };
                        let bytes = oracle_write_slot(&slot);
                        let read = oracle_read_slot(&bytes).unwrap_or_default();
                        assert_eq!(
                            read, slot,
                            "roundtrip 应逐位一致 (ch={ch} gold={gold} ms={ms} pos={pos})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn migrate_v1_to_v2() {
        let v1 = SaveSlot {
            version: 0,
            character: 3,
            gold: 0,
            map_slot: 2,
            position: 100,
            rng_steps: 0,
        };
        let migrated = oracle_migrate_v1_to_v2(&v1);
        assert_eq!(migrated.version, 2, "v1 迁移后 version 应为 2");
        assert_eq!(migrated.character, 3, "character 不变");
        assert_eq!(migrated.map_slot, 2, "map_slot 不变");
        assert_eq!(migrated.position, 100, "position 不变");
    }

    #[test]
    fn read_slot_v1_compatible() {
        // v1 布局：2B 前缀 + 4B 魔数 + 4B character + 4B map_slot + 4B position = 18 字节
        let mut bytes = [0u8; 18];
        bytes[2..6].copy_from_slice(&REC_MAGIC.to_le_bytes());
        bytes[6..10].copy_from_slice(&3u32.to_le_bytes()); // character = 3
        bytes[10..14].copy_from_slice(&1u32.to_le_bytes()); // map_slot = 1
        bytes[14..18].copy_from_slice(&200u32.to_le_bytes()); // position = 200
        let slot = oracle_read_slot_v1(&bytes).unwrap_or_default();
        assert_eq!(slot.version, 1, "v1 默认 version = 1");
        assert_eq!(slot.character, 3);
        assert_eq!(slot.gold, 0, "v1 无 gold 字段 → 默认 0");
        assert_eq!(slot.map_slot, 1);
        assert_eq!(slot.position, 200);
    }

    #[test]
    fn bad_version() {
        let bytes = oracle_write_slot(&SaveSlot { version: 3, ..Default::default() });
        // version = 3 > 2 → BadVersion
        let err = oracle_read_slot(&bytes);
        assert_eq!(err, Err(SaveErr::BadVersion), "version=3 应 BadVersion");
    }
}
