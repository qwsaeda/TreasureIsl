//! oracle：S3.1 差分基准，直译 `场景管理器帧更新总控`/`主地图帧更新` 伪代码，测试专用。
//!
//! 按 `docs/spec/main_loop.md` §4.3「场景切换状态机」的 case 行为直译：
//! - 状态码 = `*(a1+716)`，合法取值 0/2/3/4/25/26/27；
//! - 触发事件（trigger）取值 0..=30；
//! - 返回 (新状态码, 目标场景 id)。
//!
//! 本 oracle 只用 if/else 写死 spec 规则，不追求优雅。
//! 与 `rules::main_loop::SceneStack` 对拍必须逐位一致。

/// 场景状态码（`*(a1+716)` 枚举值）。
///
/// 语义（spec §4.3）：
/// - `0`  = 主菜单态（case 0 装载 vs.Map 进入战斗；case 3 退出 vs 回主菜单）
/// - `2`  = 场景卸载中（case 2：停止音效 + 场景卸载虚函数）
/// - `3`  = 回主菜单
/// - `4`  = 过场/坐标（case 4：读玩家/目标位置、设相机、销毁旧对象）
/// - `25` = 战斗态
/// - `26` = 过场动画 phase 26
/// - `27` = 过场动画 phase 27
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneState {
    /// 主菜单态（`case 0`/`case 3` 所在 switch）。
    Menu,
    /// 场景卸载中（`case 2`）。
    Unloading,
    /// 回主菜单（`case 3` 的语义状态）。
    BackToMenu,
    /// 过场/坐标（`case 4`）。
    Transition,
    /// 战斗态（`case 25`）。
    Battle,
    /// 过场动画 phase 26。
    Anim26,
    /// 过场动画 phase 27。
    Anim27,
}

impl SceneState {
    /// 状态码 → 枚举。未知码返回 `None`。
    #[must_use]
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::Menu,
            2 => Self::Unloading,
            3 => Self::BackToMenu,
            4 => Self::Transition,
            25 => Self::Battle,
            26 => Self::Anim26,
            27 => Self::Anim27,
            _ => return None,
        })
    }

    /// 枚举 → 状态码。
    #[must_use]
    pub fn code(self) -> u32 {
        match self {
            Self::Menu => 0,
            Self::Unloading => 2,
            Self::BackToMenu => 3,
            Self::Transition => 4,
            Self::Battle => 25,
            Self::Anim26 => 26,
            Self::Anim27 => 27,
        }
    }

    /// 全部合法场景状态码（7 个，按数值排序，diff 测试遍历用）。
    #[must_use]
    pub const fn all_codes() -> [u32; 7] {
        [0, 2, 3, 4, 25, 26, 27]
    }
}

/// 场景 id（spec §4.1 的 5 个装载器 + vs.Map）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    /// `intro.Map`（`intro`）。
    Intro,
    /// `selectrole.Map`（`selectrole`）。
    SelectRole,
    /// `selectmap.Map`（`selectmap`）。
    SelectMap,
    /// `index.Map`（`index`）。
    Index,
    /// `main.Map`（`main`）。
    MainMap,
    /// `vs.Map`（`战斗对象构造`）。
    Vs,
}

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
