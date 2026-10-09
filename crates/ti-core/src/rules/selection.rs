//! S3.4 selectrole/selectmap 正式实现。
//!
//! 与 `oracle::selection` 行为逐位一致（同一输入产生逐位相同的
//! `(SelectionState, Option<Sfx>)`），但结构不同：用 `SelectionFlow` 结构体 +
//! 方法承载状态，而非自由函数。
//!
//! 依据 `docs/spec/selection.md`：
//! - 角色枚举 [`Character`]（5 个，从 oracle 重导出）；
//! - 地图槽位枚举 [`MapSlot`]（3 个，从 oracle 重导出）；
//! - 选择状态 [`SelectionState`]（角色 + 地图 + 完成标志）；
//! - `select_role` / `select_map` 方法返回可选音效并更新内部状态；
//! - `is_completed` / `all_flows` 方法。
//!
//! 事件码（spec §d，`sub_41354`/`sub_3F4E4`/`sub_4085C` 实测）：
//! - 13 = 确定/Enter（选择角色/地图）；27 = 返回；37 = 无效（**推测**）；
//! - 38 = 上一个角色/地图；40 = 下一个角色/地图。

use crate::oracle::menu::Sfx;
use crate::oracle::selection::{Character, MapSlot, SelectionState};

/// 选择流程状态机：当前选择状态 + 已解锁地图清单。
#[derive(Debug, Clone)]
pub struct SelectionFlow {
    state: SelectionState,
    unlocked: Vec<MapSlot>,
}

impl SelectionFlow {
    /// 新建选择流程：空状态，`unlocked` 为已解锁地图清单（`sub_402D8` `CreateFileW` 探测结果）。
    #[must_use]
    pub fn new(unlocked: &[MapSlot]) -> Self {
        Self { state: SelectionState::new(), unlocked: unlocked.to_vec() }
    }

    /// 当前选择状态（只读）。
    #[must_use]
    pub fn state(&self) -> &SelectionState {
        &self.state
    }

    /// 是否完成（角色 + 地图均选定 → 触发 `main.Map`）。
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.state.completed
    }

    /// 已解锁地图清单（只读）。
    #[must_use]
    pub fn unlocked(&self) -> &[MapSlot] {
        &self.unlocked
    }

    /// 选择角色（事件码 13 确定 / 38 上一个 / 40 下一个循环）。
    ///
    /// 行为与 `oracle::selection::oracle_select_role` 逐位一致：
    /// - 记录角色（覆盖之前选择）；
    /// - 若地图已选 + 角色已选 → `completed=true`；
    /// - 音效：`Sfx::Press`。
    ///
    /// 返回可选音效。
    #[must_use]
    pub fn select_role(&mut self, ch: Character) -> Option<Sfx> {
        let (next, sfx) = transition_role(self.state, ch);
        self.state = next;
        sfx
    }

    /// 选择地图（事件码 13 确定 / 38 上一个 / 40 下一个循环）。
    ///
    /// 行为与 `oracle::selection::oracle_select_map` 逐位一致：
    /// - 未解锁地图 → `Sfx::Invalid`（状态不变）；
    /// - 已解锁地图：记录地图；若角色已选 + 地图已选 → `completed=true`；
    /// - 音效：`Sfx::Press`。
    ///
    /// 返回可选音效。
    #[must_use]
    pub fn select_map(&mut self, m: MapSlot) -> Option<Sfx> {
        let (next, sfx) = transition_map(self.state, m, &self.unlocked);
        self.state = next;
        sfx
    }

    /// 完整选择流程（5 角色 × 已解锁地图数 组合）。
    ///
    /// 行为与 `oracle::selection::oracle_selection_flow` 逐位一致（当 `unlocked` = 全部 3 地图时
    /// 返回 15 组合；否则返回 `5 × unlocked.len()` 组合）。
    ///
    /// 遍历 `Character::all()` × 已解锁地图，每个组合从空状态出发选角色 + 选地图，
    /// 断言 `completed=true`。
    ///
    /// # Panics
    ///
    /// 任一组合的 `completed` 不为 `true` 时 panic（不应发生，因为已解锁地图
    /// 选择后角色 + 地图均选定即完成）。
    #[must_use]
    pub fn all_flows(&self) -> Vec<(Character, MapSlot)> {
        let unlocked = &self.unlocked;
        let mut flows: Vec<(Character, MapSlot)> = Vec::new();

        for ch in Character::all() {
            for m in unlocked.iter().copied() {
                let state0 = SelectionState::new();
                let (state1, _) = transition_role(state0, ch);
                let (state2, _) = transition_map(state1, m, unlocked);
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
}

impl Default for SelectionFlow {
    /// 默认全部 3 地图已解锁（`sub_402D8` 探测 `glmapm01/02/03.Map` 全存在）。
    fn default() -> Self {
        Self::new(&MapSlot::all())
    }
}

/// 内部自由函数：与 `oracle_select_role` 逻辑相同，但作为结构体方法分派的入口。
#[must_use]
fn transition_role(state: SelectionState, ch: Character) -> (SelectionState, Option<Sfx>) {
    let mut next = state;
    next.character = Some(ch);
    if next.has_map() {
        next.completed = true;
    }
    (next, Some(Sfx::Press))
}

/// 内部自由函数：与 `oracle_select_map` 逻辑相同。
#[must_use]
fn transition_map(
    state: SelectionState,
    m: MapSlot,
    unlocked: &[MapSlot],
) -> (SelectionState, Option<Sfx>) {
    if !unlocked.contains(&m) {
        return (state, Some(Sfx::Invalid));
    }

    let mut next = state;
    next.map_slot = Some(m);
    if next.has_character() {
        next.completed = true;
    }
    (next, Some(Sfx::Press))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_new_default() {
        let mut flow = SelectionFlow::default();
        let sfx = flow.select_role(Character::Panda);
        assert!(flow.state().has_character());
        assert!(!flow.is_completed());
        assert_eq!(sfx, Some(Sfx::Press));
        let sfx = flow.select_map(MapSlot::M01);
        assert!(flow.is_completed());
        assert_eq!(sfx, Some(Sfx::Press));
    }

    #[test]
    fn flow_locked_map_invalid() {
        let mut flow = SelectionFlow::new(&[MapSlot::M01]);
        let _ = flow.select_role(Character::Kidd);
        let sfx = flow.select_map(MapSlot::M03);
        assert!(!flow.is_completed());
        assert_eq!(sfx, Some(Sfx::Invalid));
    }

    #[test]
    fn flow_all_flows_full() {
        let flow = SelectionFlow::default();
        let flows = flow.all_flows();
        assert_eq!(flows.len(), 15, "5 角色 × 3 地图 = 15 组合");
    }

    #[test]
    fn flow_all_flows_partial_unlocked() {
        let flow = SelectionFlow::new(&[MapSlot::M01, MapSlot::M02]);
        let flows = flow.all_flows();
        assert_eq!(flows.len(), 10, "5 角色 × 2 已解锁地图 = 10 组合");
    }
}
