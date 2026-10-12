//! oracle：选人/选图差分基准，测试专用（S11 起降级为纯基准函数）。
//!
//! **类型已迁至 [`crate::domain::selection`]**（S11 归档改造：业务类型不该住在逆向目录）。
//! 本模块只保留从伪代码直译的**差分基准行为函数**（选角 / 选图 / 完整流程）。
//!
//! 业务规则以 `docs/business/progression.md` 为准；本 oracle 是**历史参照**，不是规则来源。

pub use ti_core::domain::flow::Sfx;
pub use ti_core::domain::selection::{Character, MapSlot, SelectionState};

/// oracle：选择角色。
///
/// 规则（直译 `docs/spec/selection.md` §a/§d）：
/// - 记录角色（覆盖之前选择）；
/// - 若地图已选（`map_slot` 非 `None`）且角色已选 → `completed=true`（触发 `main.Map`）；
/// - 音效：`Press`（有效选择，`music\press.wav`，**推测**框架层触发）。
///
/// 返回 `(新状态, 可选音效)`。
#[must_use]
pub fn oracle_select_role(state: SelectionState, ch: Character) -> (SelectionState, Option<Sfx>) {
    let mut next = state;
    next.character = Some(ch);
    // 地图已选 + 角色刚选 → 完成（角色后选的情形）
    if next.has_map() {
        next.completed = true;
    }
    (next, Some(Sfx::Press))
}

/// oracle：选择地图。
///
/// 规则（直译 `docs/spec/selection.md` §b/§d）：
/// - 未解锁地图（`m` 不在 `unlocked` 列表）→ `completed=false` + `Sfx::Invalid`（`music\invalid.wav`，**推测**）；
/// - 已解锁地图：
///   - 记录地图（覆盖之前选择）；
///   - 若角色已选（`character` 非 `None`）且地图已选 → `completed=true`（触发 `main.Map`）；
///   - 音效：`Press`（有效选择）。
///
/// 返回 `(新状态, 可选音效)`。
#[must_use]
pub fn oracle_select_map(
    state: SelectionState,
    m: MapSlot,
    unlocked: &[MapSlot],
) -> (SelectionState, Option<Sfx>) {
    // 未解锁 → 无效（状态不变，`completed` 保持 `false`）
    if !unlocked.contains(&m) {
        return (state, Some(Sfx::Invalid));
    }

    let mut next = state;
    next.map_slot = Some(m);
    // 角色已选 + 地图刚选 → 完成
    if next.has_character() {
        next.completed = true;
    }
    (next, Some(Sfx::Press))
}

/// oracle：完整选择流程脚本（5 角色 × 3 地图 = 15 组合）。
///
/// 遍历 `Character::all()` × `MapSlot::all()`，每个组合从空状态出发：
/// 1. `oracle_select_role(state, ch)` → 选角色；
/// 2. `oracle_select_map(state, m, unlocked)` → 选地图（`unlocked` = 全部 3 地图）；
/// 3. 断言 `completed=true`（角色 + 地图均选定 → 触发 `main.Map`）。
///
/// 返回 15 个 `(角色, 地图)` 组合。
///
/// # Panics
///
/// 任一组合的 `completed` 不为 `true` 时 panic（不应发生，因为已解锁地图
/// 选择后角色 + 地图均选定即完成）。
#[must_use]
pub fn oracle_selection_flow() -> Vec<(Character, MapSlot)> {
    let unlocked = MapSlot::all();
    let mut flows: Vec<(Character, MapSlot)> = Vec::new();

    for ch in Character::all() {
        for m in MapSlot::all() {
            let state0 = SelectionState::new();
            let (state1, _) = oracle_select_role(state0, ch);
            let (state2, _) = oracle_select_map(state1, m, &unlocked);
            assert!(
                state2.completed,
                "组合 ({ch:?}, {m:?}) 应完成，实际 completed={}",
                state2.completed
            );
            flows.push((ch, m));
        }
    }

    flows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_state_new() {
        let s = SelectionState::new();
        assert!(!s.has_character());
        assert!(!s.has_map());
        assert!(!s.completed);
    }

    #[test]
    fn select_role_sets_character() {
        let s = SelectionState::new();
        let (next, sfx) = oracle_select_role(s, Character::Panda);
        assert_eq!(next.character, Some(Character::Panda));
        assert_eq!(next.map_slot, None);
        assert!(!next.completed);
        assert_eq!(sfx, Some(Sfx::Press));
    }

    #[test]
    fn select_map_unlocked_sets_map() {
        let s = SelectionState::new();
        let (s, _) = oracle_select_role(s, Character::Kidd);
        let unlocked = [MapSlot::M01, MapSlot::M02];
        let (next, sfx) = oracle_select_map(s, MapSlot::M01, &unlocked);
        assert_eq!(next.map_slot, Some(MapSlot::M01));
        assert_eq!(next.character, Some(Character::Kidd));
        assert!(next.completed);
        assert_eq!(sfx, Some(Sfx::Press));
    }

    #[test]
    fn select_map_locked_invalid() {
        let s = SelectionState::new();
        let (s, _) = oracle_select_role(s, Character::Drake);
        let unlocked = [MapSlot::M01];
        let (next, sfx) = oracle_select_map(s, MapSlot::M03, &unlocked);
        assert_eq!(next.map_slot, None, "未解锁地图不应记录");
        assert_eq!(next.character, Some(Character::Drake), "角色应保留");
        assert!(!next.completed);
        assert_eq!(sfx, Some(Sfx::Invalid));
    }

    #[test]
    fn selection_flow_15_combinations() {
        let flows = oracle_selection_flow();
        assert_eq!(flows.len(), 15, "5 角色 × 3 地图 = 15 组合");
        for (ch, m) in &flows {
            let s0 = SelectionState::new();
            let (s1, _) = oracle_select_role(s0, *ch);
            let (s2, _) = oracle_select_map(s1, *m, &MapSlot::all());
            assert!(s2.completed, "({ch:?}, {m:?}) 应完成");
            assert_eq!(s2.character, Some(*ch));
            assert_eq!(s2.map_slot, Some(*m));
        }
    }
}
