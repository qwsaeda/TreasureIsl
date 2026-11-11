//! 领地 v1 规则集成测试（S52.1；`docs/business/territory.md` §4 可测规则）。
//!
//! S52.1 补测：`territory.rs` 4 个 pub fn 此前 0 集成测试（飞轮 R2 找问题 P1-2）。
//! oracle = `docs/business/territory.md` §4.1（驻军守恒：每岛只计一次）/ §4.3（落格分叉）。
//! 判据：`cargo test -p ti-core --test territory_rules_v1` 全绿。

use ti_core::data::{balance, board};
use ti_core::rules::territory::{TerritoryState, TurnCycle};

fn fixture() -> (TerritoryState, &'static ti_core::data::Balance, &'static ti_core::data::BoardMap)
{
    let bal = balance().expect("balance.json");
    let db = board().expect("board.json");
    let map = db.map("m01").expect("m01");
    let state = TerritoryState::new(map, bal).expect("领地账本构建");
    (state, bal, map)
}

/// `owned_indices`（§4.1）：该席位全部占领格下标（升序）；无主席 = 空。
#[test]
fn owned_indices_lists_all_cells_of_seat() {
    let mut state = fixture().0;
    assert!(state.owned_indices(0).is_empty(), "初始无主 = 0 格");
    // 占两个岛各 1 格（副本胜利口径：每类岛 2 格同归，此处用 mutate 直接造态）。
    let h = state.isle_cells("harbour").to_vec();
    let w = state.isle_cells("wreck").to_vec();
    assert!(!h.is_empty() && !w.is_empty());
    state.mutate(h[0], ti_core::rules::territory::Ownership::occupied(0, 1, 10));
    state.mutate(w[0], ti_core::rules::territory::Ownership::occupied(0, 1, 10));
    let owned = state.owned_indices(0);
    assert_eq!(owned, vec![h[0], w[0]], "seat 0 占 2 格（升序）");
    assert!(state.owned_indices(1).is_empty(), "未占的席位 = 空");
}

/// `garrison_total`（§4.1 守恒口径）：每**岛**只计一次驻军，两格同岛不重复。
#[test]
fn garrison_total_counts_once_per_isle() {
    let mut state = fixture().0;
    assert_eq!(state.garrison_total(0), 0, "无占领 = 驻军 0");
    let h = state.isle_cells("harbour").to_vec();
    // 同岛两格都归 seat 0，各留驻军 30 ⇒ 该岛总驻军 = 30（按岛计，非按格累加 60）。
    state.mutate(h[0], ti_core::rules::territory::Ownership::occupied(0, 1, 30));
    state.mutate(h[1], ti_core::rules::territory::Ownership::occupied(0, 1, 30));
    assert_eq!(state.garrison_total(0), 30, "同岛两格按岛计 = 30（非 60）");
    // 再占另一岛 1 格驻军 20 ⇒ 总驻军 = 30 + 20 = 50（跨岛累加）。
    let w = state.isle_cells("wreck").to_vec();
    state.mutate(w[0], ti_core::rules::territory::Ownership::occupied(0, 1, 20));
    assert_eq!(state.garrison_total(0), 50, "跨岛累加 = 30 + 20 = 50");
}

/// `TurnCycle::is_playing`（§1.2 热座）：在局且存活才轮转；出局席不再轮。
///
/// `SeatPool::build` = v2 全 5 席在局（`is_active: true`，AI 补满 5）；v3 在局口径须
/// `build_with_ai`（AI 席数可配，未选角色 `is_active = false`）。本用例测 v2 全在局
/// + 出局（`mark_bankrupt`）不再轮转。
#[test]
fn turn_cycle_is_playing_requires_active_and_alive() {
    let pool = ti_core::rules::territory::SeatPool::build(&[0, 2]); // 玩家 2 席 + AI 补满 5
    let mut cycle = TurnCycle::new(&pool);
    assert_eq!(pool.active_seats(), vec![0, 1, 2, 3, 4], "v2 build = 全 5 席在局");
    assert!(cycle.is_playing(0), "在局席 0 = 轮转中");
    assert!(cycle.is_playing(2), "在局席 2 = 轮转中");
    // 出局席（`alive = false`）不再轮转（§1.2：热座只轮到在局且存活席）。
    cycle.mark_bankrupt(0);
    assert!(!cycle.is_playing(0), "出局席 0 = 不轮转");
    assert!(cycle.is_playing(2), "出局后其余在局席仍轮转");
    // v3 口径：build_with_ai 人类 2 + AI 1 = 在局 3 席（未选角色不在局）。
    let pool3 = ti_core::rules::territory::SeatPool::build_with_ai(&[0, 2], 1);
    let cycle3 = TurnCycle::new(&pool3);
    assert!(cycle3.is_playing(0) && cycle3.is_playing(2), "v3 人类席在局");
    assert!(!cycle3.is_playing(4), "v3 未选且未补 AI 席 = 不在局（不轮转）");
}
