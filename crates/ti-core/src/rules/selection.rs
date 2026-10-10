//! S3.4 / S8.3.e selectrole/selectmap 正式实现（数据化：转移逻辑落本文件，不再调 oracle）。
//!
//! S8 定位变更：旧 `oracle::selection::oracle_select_role`/`oracle_select_map`/
//! `oracle_selection_flow` 降级参照物；本模块的选角/选图状态机直接落本文件
//! （`docs/business/loop_core.md` §选角选图），行为与 oracle 一致（同输入同输出），
//! 但**不再调用** oracle 自由函数。
//!
//! [`Character`]/[`MapSlot`]/[`SelectionState`] 仍从 oracle 重导出（枚举定义非推测公式，
//! S8.3.e 允许）；角色/地图目录走 `balance.selection`（`character_count`/`map_count`），
//! 选角/选图标签走 `strings.json`（`menu.*`/`scene.*` key，非中文字面量）。
//!
//! 事件码（`docs/business/loop_core.md`，实测）：
//! - 13 = 确定/Enter（选择角色/地图）；27 = 返回；37 = 无效；
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
    /// 新建选择流程：空状态，`unlocked` 为已解锁地图清单（`selectmap` `CreateFileW` 探测结果）。
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
    /// 规则（`docs/business/loop_core.md` §选角，与 oracle 行为一致但本文件为唯一事实源）：
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

    /// 完整选择流程（角色数 × 已解锁地图数 组合，目录走 `balance.selection`）。
    ///
    /// 遍历 `balance.selection` 的角色目录（缺数回退 `Character::all()`）× 已解锁地图，
    /// 每个组合从空状态出发选角色 + 选地图，断言 `completed=true`
    /// （与 oracle 流程行为一致，本文件为唯一事实源）。
    ///
    /// # Panics
    ///
    /// 任一组合的 `completed` 不为 `true` 时 panic（不应发生，因为已解锁地图
    /// 选择后角色 + 地图均选定即完成）。
    #[must_use]
    pub fn all_flows(&self) -> Vec<(Character, MapSlot)> {
        let all = Character::all();
        let characters = match crate::data::balance() {
            Ok(b) => all.iter().take(b.selection.character_count as usize).copied().collect(),
            Err(_) => all.to_vec(),
        };
        let unlocked = &self.unlocked;
        let mut flows: Vec<(Character, MapSlot)> = Vec::new();

        for ch in characters {
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
    /// 默认全部 3 地图已解锁（`selectmap` 探测 `glmapm01/02/03.Map` 全存在）。
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
