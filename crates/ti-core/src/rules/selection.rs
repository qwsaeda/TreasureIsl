//! S3.4 / S8.3.e / S12.4 selectrole/selectmap 正式实现（数据化：转移逻辑落本文件）。
//!
//! 本模块的选角/选图状态机直接落本文件
//! （`docs/business/loop_core.md` §选角选图 + `07_ui_flow.md` §3 输入映射），
//! 类型全部来自 `domain::selection` / `domain::flow`（S11 类型唯一家）。
//!
//! 事件码（`docs/business/loop_core.md`，实测）：
//! - 13 = 确定/Enter（选择角色/地图）；27 = 返回；37 = 无效；
//! - 38 = 上一个角色/地图；40 = 下一个角色/地图。

use crate::domain::flow::Sfx;
use crate::domain::selection::{Character, MapSlot, SelectionState};

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

    /// 当前选择状态的可变句柄（场景切换 `reset` 复位、测试改字段用）。
    #[must_use]
    pub fn state_mut(&mut self) -> &mut SelectionState {
        &mut self.state
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
    /// 规则（`docs/business/loop_core.md` §选角，本文件为唯一事实源）：
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
    /// 选图规则（本文件为唯一事实源）：
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

    /// 场景切换复位（`07_ui_flow.md` §1 硬规则 4 + §6 规则 12
    /// `scene_switch_resets_state`）：切场景后选中项必须复位，禁止残留。
    ///
    /// 复位 `SelectionState`（角色/地图/completed 全清），已解锁地图清单保留
    /// （`unlocked` 是探测事实，不是选择状态）。
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// 完整选择流程（角色数 × 已解锁地图数 组合，目录走 `balance.selection`）。
    ///
    /// 遍历 `balance.selection` 的角色目录（缺数回退 `Character::all()`）× 已解锁地图，
    /// 每个组合从空状态出发选角色 + 选地图，只保留 `completed=true` 的组合
    /// （本文件为唯一事实源）。
    ///
    /// 库代码 0 panic：不满足完成不变式的组合被**过滤丢弃**（非 panic），
    /// 调用方可用 `all_flows` 长度与 `角色数 × 地图数` 对账判完成性。
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
                if state2.completed {
                    flows.push((ch, m));
                }
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

/// 内部自由函数：选角转移，作为结构体方法分派的入口。
#[must_use]
fn transition_role(state: SelectionState, ch: Character) -> (SelectionState, Option<Sfx>) {
    let mut next = state;
    next.character = Some(ch);
    if next.has_map() {
        next.completed = true;
    }
    (next, Some(Sfx::Press))
}

/// 内部自由函数：选图转移。
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

// ------------------------------------------------------- S28.4 席位多选（`docs/business/territory.md` §1）

use crate::rules::territory::SeatPool;

/// 席位多选不变量（§1）：玩家选 **≥1** 名角色为**玩家席**，其余角色自动补 AI 席，合计 5。
///
/// `seats` = 玩家选中的角色列表（0..=4 唯一；顺序 = 角色 index 升序）；
/// `count` = 可选角色数（`balance.selection.character_count`，缺省 5）。
///
/// 返回音效（`Sfx::Invalid` = 状态未变的无效操作）。
#[must_use]
pub fn seat_toggle(seats: &mut Vec<Character>, ch: Character, count: u32) -> Sfx {
    if ch.index() >= count {
        return Sfx::Invalid;
    }
    if seats.contains(&ch) {
        // 取消玩家席：最后一名玩家不可取消（§1 不变量 players ≥ 1）
        if seats.len() == 1 {
            return Sfx::Invalid;
        }
        seats.retain(|c| *c != ch);
        return Sfx::Press;
    }
    seats.push(ch);
    seats.sort_by_key(|c| c.index());
    Sfx::Press
}

/// 玩家选中的席位角色 → 席位池（§1：`players + ai = 5`；未选中角色自动 AI）。
#[must_use]
pub fn seat_pool(seats: &[Character]) -> SeatPool {
    let indexes: Vec<u32> = seats.iter().map(|c| c.index()).collect();
    SeatPool::build(&indexes)
}
