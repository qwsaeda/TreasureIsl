//! 场景管理正式实现（`docs/spec/main_loop.md` §4.3 场景切换状态机）。
//!
//! 与 `oracle::main_loop::oracle_next_state` 的行为逐位一致，但结构不同：
//! 用 `SceneStack` 结构体 + `advance` 方法承载状态与切换历史，而非自由函数。
//!
//! 行为必须与 oracle 完全一致（同一状态 + 同一 trigger 产生同一输出）。

use crate::oracle::main_loop::SceneId;

/// 场景管理器：当前状态码、当前场景 id、切换历史栈。
#[derive(Debug, Clone)]
pub struct SceneStack {
    state: u32,
    scene: Option<SceneId>,
    history: Vec<(u32, Option<SceneId>)>,
}

impl SceneStack {
    /// 新建场景管理器：初始状态码 0（主菜单），无场景，无历史。
    #[must_use]
    pub fn new() -> Self {
        Self { state: 0, scene: None, history: Vec::new() }
    }

    /// 当前状态码。
    #[must_use]
    pub fn state(&self) -> u32 {
        self.state
    }

    /// 当前场景 id（`None` = 无场景）。
    #[must_use]
    pub fn current_scene(&self) -> Option<SceneId> {
        self.scene
    }

    /// 切换历史栈（每次 `advance` 产生一次记录）。
    #[must_use]
    pub fn history(&self) -> &[(u32, Option<SceneId>)] {
        &self.history
    }

    /// 手动设置状态码（用于 diff 测试：构造任意状态码而不走 advance 链）。
    pub fn set_state(&mut self, s: u32) {
        self.state = s;
    }

    /// 手动压入一个场景 id（用于场景装载器侧的显式装载）。
    pub fn push_scene(&mut self, s: SceneId) {
        self.scene = Some(s);
        self.history.push((self.state, self.scene));
    }

    /// 推进状态机：给定触发事件码，更新状态与场景，返回 (新状态码, 目标场景 id)。
    ///
    /// 行为与 `oracle_next_state` 逐位一致（同一状态 + 同一 trigger → 同一输出）。
    #[must_use]
    pub fn advance(&mut self, trigger: u32) -> (u32, Option<SceneId>) {
        let next = match self.state {
            // case 0（主菜单态）：若 trigger=1 进战斗（新状态 25，场景 Vs）；否则保持。
            0 => {
                if trigger == 1 {
                    (25, Some(SceneId::Vs))
                } else {
                    (0, None)
                }
            }
            // case 2（场景卸载中）：若 trigger=5 过场动画完成 → 回主菜单（新状态 0，场景 Index）；否则保持。
            2 => {
                if trigger == 5 {
                    (0, Some(SceneId::Index))
                } else {
                    (2, None)
                }
            }
            // case 3（回主菜单）：任何 trigger 都触发回主菜单（新状态 0，场景 Index）。
            3 => (0, Some(SceneId::Index)),
            // case 4（过场/坐标）：若 trigger=4 触发过场动画 → 新状态 27，场景 Intro；否则保持。
            4 => {
                if trigger == 4 {
                    (27, Some(SceneId::Intro))
                } else {
                    (4, None)
                }
            }
            // case 25（战斗态）：若 trigger=2 战斗结束 → 回主菜单（新状态 0，场景 Index）；否则保持。
            25 => {
                if trigger == 2 {
                    (0, Some(SceneId::Index))
                } else {
                    (25, None)
                }
            }
            // case 26（过场动画 phase 26）：若 trigger=5 过场动画完成 → 新状态 27，场景 Intro；否则保持。
            26 => {
                if trigger == 5 {
                    (27, Some(SceneId::Intro))
                } else {
                    (26, None)
                }
            }
            // case 27（过场动画 phase 27）：若 trigger=5 过场动画完成 → 回主菜单（新状态 0，场景 Index）；否则保持。
            27 => {
                if trigger == 5 {
                    (0, Some(SceneId::Index))
                } else {
                    (27, None)
                }
            }
            // 未知状态码：保持不变。
            _ => (self.state, None),
        };

        self.history.push((self.state, self.scene));
        self.state = next.0;
        if next.1.is_some() {
            self.scene = next.1;
        }
        next
    }
}

impl Default for SceneStack {
    fn default() -> Self {
        Self::new()
    }
}
