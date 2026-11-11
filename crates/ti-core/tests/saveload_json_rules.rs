//! 存档标签迁移集成测试（S52.1；`docs/business/persistence.md` 存档 v5）。
//!
//! S52.1 补测：`saveload_json.rs` `legacy_character_label` / `legacy_map_label` 此前
//! 0 集成测试（飞轮 R2 找问题 P1-22）。判据：`cargo test -p ti-core --test saveload_json_rules` 全绿。

use ti_core::rules::saveload_json::{legacy_character_label, legacy_map_label};

/// `legacy_character_label`（legacy 数字角色 1 起 → 角色 label；`0`/越界 = `panda` 兜底）。
#[test]
fn legacy_character_label_maps_digit_to_role() {
    assert_eq!(legacy_character_label(1), "panda", "1 = panda");
    assert_eq!(legacy_character_label(2), "tichy", "2 = tichy");
    assert_eq!(legacy_character_label(3), "kidd", "3 = kidd");
    // 越界 / 0 = 钳位兜底 panda（saturating_sub(1) 到 0）。
    assert_eq!(legacy_character_label(0), "panda", "0 = panda 兜底");
    assert_eq!(legacy_character_label(99), "panda", "越界 = panda 兜底");
}

/// `legacy_map_label`（legacy 数字地图 1..=20 → `mNN`；`0`/越界钳到 `m01..m20`）。
#[test]
fn legacy_map_label_clamps_to_m01_m20() {
    assert_eq!(legacy_map_label(1), "m01", "1 = m01");
    assert_eq!(legacy_map_label(3), "m03", "3 = m03");
    assert_eq!(legacy_map_label(20), "m20", "20 = m20");
    // 0 / 越界 = 钳位边界。
    assert_eq!(legacy_map_label(0), "m01", "0 钳到 m01");
    assert_eq!(legacy_map_label(99), "m20", "越界钳到 m20");
}

/// 回读一致性：`legacy_map_label` 输出 = `MapSlot::id`（`mNN` 数值对齐，存档往返）。
///
/// `MapSlot::label` 是选图 i18n key（`glmapmNN.Map`）；`legacy_map_label` 是存档
/// 内部图 id（`mNN`，= `MapSlot::id`）。两者不同面，存档字段对齐的是 `id`。
#[test]
fn legacy_map_label_matches_mapslot_id() {
    use ti_core::domain::selection::MapSlot;
    for m in [MapSlot::M01, MapSlot::M05, MapSlot::M12, MapSlot::M20] {
        let n = m.index(); // legacy 数字 1 起 = MapSlot::index（1 起）
        assert_eq!(legacy_map_label(n), m.id(), "legacy_map_label(n) = MapSlot.id");
    }
}
