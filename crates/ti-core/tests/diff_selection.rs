//! S3.4 selectrole/selectmap 差分测试。
//!
//! 遍历 15 组合（5 角色 × 3 地图）× 各状态 × 各事件，oracle 与 impl 逐位一致
//! （新 state + Sfx + completed 全等）；
//! 另加 2 个语义测试。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::menu::Sfx;
use ti_core::oracle::selection::{
    Character, MapSlot, SelectionState, oracle_select_map, oracle_select_role,
    oracle_selection_flow,
};
use ti_core::rules::selection::SelectionFlow;

/// 断言 oracle 与 impl 的选择结果逐位一致。
fn assert_selection_equal(
    oracle_state: SelectionState,
    oracle_sfx: Option<Sfx>,
    impl_flow: &SelectionFlow,
    impl_sfx: Option<Sfx>,
) {
    assert_eq!(oracle_state, *impl_flow.state(), "状态不一致");
    assert_eq!(oracle_sfx, impl_sfx, "音效不一致");
    assert_eq!(oracle_state.completed, impl_flow.is_completed(), "completed 标志不一致");
}

/// 遍历 5 角色 × 3 地图 = 15 组合，oracle vs impl 逐位一致（先角色后地图）。
#[test]
fn diff_role_then_map_15_combinations() {
    for ch in Character::all() {
        for m in MapSlot::all() {
            // oracle 路径
            let state0 = SelectionState::new();
            let (state1, sfx1_o) = oracle_select_role(state0, ch);
            let (state2, sfx2_o) = oracle_select_map(state1, m, &MapSlot::all());

            // impl 路径
            let mut flow = SelectionFlow::default();
            let sfx1_i = flow.select_role(ch);
            assert_selection_equal(state1, sfx1_o, &flow, sfx1_i);
            let sfx2_i = flow.select_map(m);
            assert_selection_equal(state2, sfx2_o, &flow, sfx2_i);
        }
    }
}

/// 遍历 5 角色 × 3 地图 = 15 组合，oracle vs impl 逐位一致（先地图后角色）。
#[test]
fn diff_map_then_role_15_combinations() {
    for ch in Character::all() {
        for m in MapSlot::all() {
            // oracle 路径
            let state0 = SelectionState::new();
            let (state1, sfx1_o) = oracle_select_map(state0, m, &MapSlot::all());
            let (state2, sfx2_o) = oracle_select_role(state1, ch);

            // impl 路径
            let mut flow = SelectionFlow::default();
            let sfx1_i = flow.select_map(m);
            assert_selection_equal(state1, sfx1_o, &flow, sfx1_i);
            let sfx2_i = flow.select_role(ch);
            assert_selection_equal(state2, sfx2_o, &flow, sfx2_i);
        }
    }
}

/// 遍历未解锁地图场景：5 角色 × 3 地图 × 各解锁子集，oracle vs impl 逐位一致。
#[test]
fn diff_locked_maps_all_unlocked_subsets() {
    let all_maps = MapSlot::all();

    // 遍历所有非空解锁子集（3 个地图 → 7 个非空子集）
    for mask in 1..=(1u32 << 3) {
        let unlocked: Vec<MapSlot> = all_maps
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, m)| *m)
            .collect();

        for ch in Character::all() {
            for m in all_maps {
                let is_unlocked = unlocked.contains(&m);

                // oracle 路径
                let state0 = SelectionState::new();
                let (state1, sfx1_o) = oracle_select_role(state0, ch);
                let (state2, sfx2_o) = oracle_select_map(state1, m, &unlocked);

                // impl 路径
                let mut flow = SelectionFlow::new(&unlocked);
                let sfx1_i = flow.select_role(ch);
                assert_selection_equal(state1, sfx1_o, &flow, sfx1_i);
                let sfx2_i = flow.select_map(m);
                assert_selection_equal(state2, sfx2_o, &flow, sfx2_i);

                // 语义断言：未解锁 → Invalid + 未完成
                if !is_unlocked {
                    assert_eq!(sfx2_o, Some(Sfx::Invalid), "未解锁地图应返回 Invalid");
                    assert!(!state2.completed, "未解锁地图不应完成");
                }
            }
        }
    }
}

/// 遍历重选角色（覆盖之前选择），oracle vs impl 逐位一致。
#[test]
fn diff_reselect_role_15_combinations() {
    for ch1 in Character::all() {
        for ch2 in Character::all() {
            for m in MapSlot::all() {
                // oracle 路径
                let state0 = SelectionState::new();
                let (state1, _) = oracle_select_role(state0, ch1);
                let (state2, sfx_o) = oracle_select_map(state1, m, &MapSlot::all());
                let (state3, _) = oracle_select_role(state2, ch2);

                // impl 路径
                let mut flow = SelectionFlow::default();
                let _ = flow.select_role(ch1);
                let _ = flow.select_map(m);
                let _ = flow.select_role(ch2);

                // state3 应覆盖为 ch2
                assert_eq!(state3, *flow.state(), "oracle 与 impl 状态应一致（重选角色）");
                assert_eq!(state3.character, Some(ch2), "重选角色应覆盖");
                assert!(state3.completed, "地图已选 + 重选角色 → 完成");
                assert!(flow.is_completed(), "impl 重选角色 → 完成");
                assert_eq!(sfx_o, Some(Sfx::Press), "地图选择音效应为 Press");
            }
        }
    }
}

/// 遍历重选地图（覆盖之前选择），oracle vs impl 逐位一致。
#[test]
fn diff_reselect_map_15_combinations() {
    for ch in Character::all() {
        for m1 in MapSlot::all() {
            for m2 in MapSlot::all() {
                // oracle 路径
                let state0 = SelectionState::new();
                let (state1, _) = oracle_select_role(state0, ch);
                let (state2, _) = oracle_select_map(state1, m1, &MapSlot::all());
                let (state3, sfx_o) = oracle_select_map(state2, m2, &MapSlot::all());

                // impl 路径
                let mut flow = SelectionFlow::default();
                let _ = flow.select_role(ch);
                let _ = flow.select_map(m1);
                let sfx_i = flow.select_map(m2);

                assert_eq!(state3, *flow.state(), "oracle 与 impl 状态应一致（重选地图）");
                assert_eq!(state3.map_slot, Some(m2), "重选地图应覆盖");
                assert!(state3.completed, "角色已选 + 重选地图 → 完成");
                assert!(flow.is_completed(), "impl 重选地图 → 完成");
                assert_eq!(sfx_o, sfx_i, "重选地图音效应一致");
            }
        }
    }
}

/// oracle 的 `oracle_selection_flow` 与 impl 的 `all_flows` 逐位一致（15 组合）。
#[test]
fn diff_flow_script() {
    let o_flows = oracle_selection_flow();
    let flow = SelectionFlow::default();
    let i_flows = flow.all_flows();
    assert_eq!(o_flows, i_flows, "oracle 与 impl 流程脚本不一致");
    assert_eq!(o_flows.len(), 15, "5 角色 × 3 地图 = 15 组合");
}

/// 语义测试 (a)：选 Panda + M01（已解锁）→ completed。
#[test]
fn semantic_panda_m01_completed() {
    let mut flow = SelectionFlow::default();
    let sfx1 = flow.select_role(Character::Panda);
    assert_eq!(sfx1, Some(Sfx::Press), "选角色应触发 Press");
    assert!(!flow.is_completed(), "仅选角色未完成");

    let sfx2 = flow.select_map(MapSlot::M01);
    assert_eq!(sfx2, Some(Sfx::Press), "选已解锁地图应触发 Press");
    assert!(flow.is_completed(), "Panda + M01 应完成");
    assert_eq!(flow.state().character, Some(Character::Panda));
    assert_eq!(flow.state().map_slot, Some(MapSlot::M01));

    // oracle 对照
    let state0 = SelectionState::new();
    let (state1, _) = oracle_select_role(state0, Character::Panda);
    let (state2, _) = oracle_select_map(state1, MapSlot::M01, &MapSlot::all());
    assert!(state2.completed, "oracle: Panda + M01 应完成");
    assert_eq!(state2, *flow.state(), "oracle 与 impl 状态应一致");
}

/// 语义测试 (b)：选 Kidd + M03（未解锁）→ `Sfx::Invalid` 且 completed=false。
#[test]
fn semantic_kidd_m03_locked_invalid() {
    let unlocked = [MapSlot::M01, MapSlot::M02];
    let mut flow = SelectionFlow::new(&unlocked);
    let _ = flow.select_role(Character::Kidd);
    assert!(!flow.is_completed(), "仅选角色未完成");

    let sfx = flow.select_map(MapSlot::M03);
    assert_eq!(sfx, Some(Sfx::Invalid), "未解锁地图应触发 Invalid");
    assert!(!flow.is_completed(), "未解锁地图不应完成");
    assert_eq!(flow.state().map_slot, None, "未解锁地图不应记录");

    // oracle 对照
    let state0 = SelectionState::new();
    let (state1, _) = oracle_select_role(state0, Character::Kidd);
    let (state2, sfx_o) = oracle_select_map(state1, MapSlot::M03, &unlocked);
    assert_eq!(sfx_o, Some(Sfx::Invalid), "oracle: 未解锁地图应触发 Invalid");
    assert!(!state2.completed, "oracle: 未解锁地图不应完成");
    assert_eq!(state2.map_slot, None, "oracle: 未解锁地图不应记录");
    assert_eq!(state2, *flow.state(), "oracle 与 impl 状态应一致");
}
