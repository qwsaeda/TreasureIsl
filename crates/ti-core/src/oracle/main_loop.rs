//! oracle 场景切换状态机差分基准（S11 降级为纯基准函数；类型在 domain::flow）。
//!
//! 差分基准 = 从 `legacy/docs/spec/*.md` 伪代码直译的纯函数，供 `tests/diff_*.rs` 对账
//! 正向实现（`rules::`）；本模块**不参与运行时**，S11 起物理归档 `legacy/` 前保留为对照基准。
#![allow(clippy::doc_markdown)]

pub use crate::domain::flow::{SceneId, SceneState};

/// oracle：给定当前状态码 + 触发事件码，返回 (新状态码, 目标场景 id)。
///
/// 规则完全照 spec §4.3 两处 switch 的 case 行为直译。
/// 状态码取 7 个合法值（`SceneState::all_codes()`）；未知码保持不变。
/// 触发事件码 0..=30，只有 5 个值有规则：
/// - `1` = 进战斗（装载 vs.Map）
/// - `2` = 战斗结束（回主菜单）
/// - `3` = 回菜单
/// - `4` = 触发过场动画
/// - `5` = 过场动画完成
/// - `0` 及其他值 = 无触发（状态保持不变）
///
/// case 行为（spec §4.3）：
/// - case 0（状态 0）：若 trigger=1 进战斗（新状态 25，场景 Vs）；否则保持。
/// - case 2（状态 2）：若 trigger=5 过场动画完成 → 回主菜单（新状态 0，场景 Index）；否则保持。
/// - case 3（状态 3）：回主菜单（新状态 0，场景 Index），任何 trigger 都触发。
/// - case 4（状态 4）：若 trigger=4 触发过场动画 → 新状态 27，场景 Intro；否则保持。
/// - case 25（状态 25）：若 trigger=2 战斗结束 → 回主菜单（新状态 0，场景 Index）；否则保持。
/// - case 26（状态 26）：若 trigger=5 过场动画完成 → 新状态 27，场景 Intro；否则保持。
/// - case 27（状态 27）：若 trigger=5 过场动画完成 → 回主菜单（新状态 0，场景 Index）；否则保持。
#[must_use]
pub fn oracle_next_state(s: u32, trigger: u32) -> (u32, Option<SceneId>) {
    // 直译 spec §4.3 case 行为
    match s {
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
        // 未知状态码：保持不变（diff 测试只用 7 个合法值，不会走到这里）。
        _ => (s, None),
    }
}
