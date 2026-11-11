//! 跨图推进规则集成测试（S52.1；`docs/business/progression.md` 跨图推进）。
//!
//! S52.1 补测：`progression.rs` 7 个 pub fn 此前 0 集成测试（飞轮 R2 找问题 P1-4）。
//! oracle = `docs/business/progression.md` §跨图推进（m01 恒解锁，m(i+1) 需击败 m(i)，
//! m20 BOSS = 通关位；掩码 bit j = m(j+1) 解锁）。判据：`cargo test -p ti-core --test progression_rules` 全绿。

use ti_core::domain::selection::MapSlot;
use ti_core::rules::progression::{ProgressionState, boss_tier_of};

/// `initial_unlocked`：初始仅 m01 解锁。
#[test]
fn initial_unlocked_is_m01_only() {
    let st = ProgressionState::initial();
    let unlocked = ProgressionState::initial_unlocked();
    assert_eq!(unlocked, vec![MapSlot::M01], "初始 = 仅 m01");
    assert!(st.can_select(MapSlot::M01), "m01 可选");
    assert!(!st.can_select(MapSlot::M02), "m02 未解锁（需击败 m01）");
}

/// `on_boss_defeated` + `can_select`：击败 m01 BOSS ⇒ m02 解锁（单调链）。
#[test]
fn on_boss_defeated_unlocks_next_map() {
    let st = ProgressionState::initial();
    let st2 = st.on_boss_defeated(MapSlot::M01);
    assert!(st2.can_select(MapSlot::M02), "击败 m01 ⇒ m02 解锁");
    assert!(st2.is_map_defeated(MapSlot::M01), "m01 标记已击败");
    assert!(!st2.is_map_defeated(MapSlot::M02), "m02 尚未击败");
}

/// `defeated_count`：击败图数 = 已击败位计数。
#[test]
fn defeated_count_tracks_maps_beaten() {
    let st = ProgressionState::initial();
    assert_eq!(st.defeated_count(), 0, "初始 0 击败");
    let st2 = st.on_boss_defeated(MapSlot::M01).on_boss_defeated(MapSlot::M02);
    assert_eq!(st2.defeated_count(), 2, "击败 2 图");
}

/// `is_cleared`：m20 BOSS 击败 = 通关（`CLEARED_BIT`）。
#[test]
fn is_cleared_when_m20_boss_beaten() {
    let mut st = ProgressionState::initial();
    for m in [MapSlot::M01, MapSlot::M02, MapSlot::M03] {
        st = st.on_boss_defeated(m);
    }
    assert!(!st.is_cleared(), "m03 击败 ≠ 通关（20 图需 m20）");
    let mut full = ProgressionState::initial();
    for m in [
        MapSlot::M01,
        MapSlot::M02,
        MapSlot::M03,
        MapSlot::M04,
        MapSlot::M05,
        MapSlot::M06,
        MapSlot::M07,
        MapSlot::M08,
        MapSlot::M09,
        MapSlot::M10,
        MapSlot::M11,
        MapSlot::M12,
        MapSlot::M13,
        MapSlot::M14,
        MapSlot::M15,
        MapSlot::M16,
        MapSlot::M17,
        MapSlot::M18,
        MapSlot::M19,
        MapSlot::M20,
    ] {
        full = full.on_boss_defeated(m);
    }
    assert!(full.is_cleared(), "m20 击败 = 通关");
}

/// `boss_tier_of`：选图 BOSS 档位读 `balance.selection.maps[].boss_tier`（m01..m20 有档，m21+ = None）。
#[test]
fn boss_tier_of_maps_selection_entries() {
    assert!(boss_tier_of(MapSlot::M01).is_some(), "m01 有 BOSS 档（balance 有选图条目）");
    assert!(boss_tier_of(MapSlot::M20).is_some(), "m20 有 BOSS 档");
    // m21..m40 = 扩展包（board.json 有但 selection.maps 20 条无），boss_tier = None。
    // （选图 UI 只走 m01..m20，m21+ 无选图入口——飞轮 R2 P1-6 定位）
}

/// `from_mask` / `unlocked_mask` 互逆（§8 存档 v5：解锁集合以存档为事实源）。
#[test]
fn unlocked_mask_roundtrip() {
    let st =
        ProgressionState::initial().on_boss_defeated(MapSlot::M01).on_boss_defeated(MapSlot::M02);
    let mask = st.unlocked_mask();
    let back = ProgressionState::from_mask(mask);
    assert_eq!(back, st, "mask → from_mask 互逆");
}

/// `for_save_restore`：`mask == 0`（旧档）⇒ 最小前置链兜底；`mask` 缺当前图 ⇒ 补最小链。
#[test]
fn for_save_restore_fallback_for_legacy_and_broken_mask() {
    // 旧档 mask=0：回到 m01 的最小前置链（仅 m01 解锁）。
    let st = ProgressionState::for_save_restore(0, MapSlot::M01);
    assert!(st.can_select(MapSlot::M01), "旧档 m01 = 最小前置");
    // 矛盾掩码（m02 解锁但 m01 未击败 = bit1 置位但 m01 未败）⇒ 补最小前置链。
    let broken = ProgressionState::for_save_restore(1u32 << 1, MapSlot::M02);
    assert!(broken.can_select(MapSlot::M02), "矛盾掩码 = 补最小前置链到 m02");
}

/// `from_unlocked`：解锁集合 → 推进态（击败位 = 相邻两图的差）。
#[test]
fn from_unlocked_reconstructs_defeated_bits() {
    let st = ProgressionState::from_unlocked(&[MapSlot::M01, MapSlot::M02, MapSlot::M03]);
    assert!(st.can_select(MapSlot::M03), "解锁含 m03");
    assert!(st.is_map_defeated(MapSlot::M02), "m02 已击败（解锁了 m03）");
    assert!(!st.is_map_defeated(MapSlot::M03), "m03 未击败（是解锁非击败）");
}
