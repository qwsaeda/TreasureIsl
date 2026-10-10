//! 场景状态机（S8.3.e 数据化：`SceneId` 从 `oracle::main_loop` 重导出为唯一事实源，
//! 转移规则依据 `docs/business/loop_core.md`）。
//!
//! S8 定位变更：旧 `oracle::main_loop::oracle_next_state` 降级参照物；本模块的转移表
//! 与 oracle 行为一致（同状态 + 同 trigger → 同输出），但**不再调用** oracle 自由函数，
//! 转移逻辑直接落本文件（`docs/business/loop_core.md` 场景切换状态机）。
//!
//! `SceneId` 仍从 oracle 重导出（S8.3.e 允许：枚举定义非「推测公式」，是稳定标识）；
//! 状态码/trigger 码表在 `docs/business/loop_core.md`，数值在 `balance.json`（场景键）。

use crate::oracle::main_loop::SceneId;

/// 场景管理器：当前状态码、当前场景 id、切换历史栈。
///
/// 状态码/trigger 语义（`docs/business/loop_core.md`）：
/// - 0 = 主菜单；2 = 场景卸载中；3 = 回主菜单；4 = 过场/坐标；
/// - 25 = 战斗；26/27 = 过场动画 phase；trigger 1/2/4/5 = 各场景切换事件。
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

    /// 手动设置状态码（用于构造任意状态码，测试 / 读档恢复）。
    pub fn set_state(&mut self, s: u32) {
        self.state = s;
    }

    /// 手动压入一个场景 id（显式装载）。
    pub fn push_scene(&mut self, s: SceneId) {
        self.scene = Some(s);
        self.history.push((self.state, self.scene));
    }

    /// 推进状态机：给定触发事件码，更新状态与场景，返回 (新状态码, 目标场景 id)。
    ///
    /// 转移表（`docs/business/loop_core.md`，与 oracle 行为一致但本文件为唯一事实源）：
    /// - 0 + trigger 1 → (25, Vs)；否则保持；
    /// - 2 + trigger 5 → (0, Index)；否则保持；
    /// - 3 → 任意 trigger → (0, Index)；
    /// - 4 + trigger 4 → (27, Intro)；否则保持；
    /// - 25 + trigger 2 → (0, Index)；否则保持；
    /// - 26 + trigger 5 → (27, Intro)；否则保持；
    /// - 27 + trigger 5 → (0, Index)；否则保持；
    /// - 未知状态 → 保持。
    #[must_use]
    pub fn advance(&mut self, trigger: u32) -> (u32, Option<SceneId>) {
        let next = match self.state {
            0 => {
                if trigger == 1 {
                    (25, Some(SceneId::Vs))
                } else {
                    (0, None)
                }
            }
            2 => {
                if trigger == 5 {
                    (0, Some(SceneId::Index))
                } else {
                    (2, None)
                }
            }
            3 => (0, Some(SceneId::Index)),
            4 => {
                if trigger == 4 {
                    (27, Some(SceneId::Intro))
                } else {
                    (4, None)
                }
            }
            25 => {
                if trigger == 2 {
                    (0, Some(SceneId::Index))
                } else {
                    (25, None)
                }
            }
            26 => {
                if trigger == 5 {
                    (27, Some(SceneId::Intro))
                } else {
                    (26, None)
                }
            }
            27 => {
                if trigger == 5 {
                    (0, Some(SceneId::Index))
                } else {
                    (27, None)
                }
            }
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
