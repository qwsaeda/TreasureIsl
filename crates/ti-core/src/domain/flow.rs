//! 流程域类型（业务文档：`docs/business/loop_core.md`）。
//!
//! 场景标识 / 菜单事件 / 音效 / 场景切换状态机。

/// 菜单场景（业务文档 `docs/business/loop_core.md` §场景图）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MenuScene {
    /// 片头过场。
    Intro,
    /// 主菜单。
    Index,
    /// 角色选择。
    SelectRole,
    /// 地图选择。
    SelectMap,
    /// 存读档（子窗口叠加）。
    Saveload,
    /// 选项。
    Option,
    /// 帮助/说明。
    Help,
    /// 退出。
    Exit,
}

impl MenuScene {
    /// 全部场景（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [MenuScene; 8] {
        [
            Self::Intro,
            Self::Index,
            Self::SelectRole,
            Self::SelectMap,
            Self::Saveload,
            Self::Option,
            Self::Help,
            Self::Exit,
        ]
    }

    /// 场景判别值（0..=7，一场景一值，`docs/business/07_ui_flow.md` §1 硬规则 2）。
    #[must_use]
    pub const fn discriminant(self) -> u32 {
        self as u32
    }
}

/// 音效种类（业务文档 `docs/business/loop_core.md` §反馈）。
///
/// | 变体 | 语义 | 触发点 |
/// |---|---|---|
/// | `Press` | 有效操作 | 菜单确认 / 选择生效 |
/// | `Invalid` | 无效操作 | 边界输入（金不足 / 无库存 / 不可建造） |
/// | `Victory` | 战斗胜利 | BOSS 击败 |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    /// 点击有效菜单项。
    Press,
    /// 无效操作。
    Invalid,
    /// 战斗胜利。
    Victory,
}

/// 菜单事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    /// 点击事件，参数为事件码（`0..=5` = 菜单项索引）。
    Click(u32),
    /// 无效操作。
    Invalid,
    /// 退出。
    Quit,
}

impl MenuEvent {
    /// 全部事件变体（遍历用，`docs/spec/menu.md` §a 事件码表）。
    ///
    /// `Click(0)..Click(5)`（菜单项索引）+ `Click(13)`（真实确定码）
    /// + `Click(27)`（返回码）+ `Click(37)`（无效码）+ `Click(38)`（上页码）
    /// + `Invalid` + `Quit`。
    #[must_use]
    pub const fn all() -> [MenuEvent; 12] {
        [
            Self::Click(0),
            Self::Click(1),
            Self::Click(2),
            Self::Click(3),
            Self::Click(4),
            Self::Click(5),
            Self::Click(13),
            Self::Click(27),
            Self::Click(37),
            Self::Invalid,
            Self::Quit,
            Self::Click(38),
        ]
    }
}

/// 菜单光标步进方向（`docs/business/07_ui_flow.md` §3.1 `MoveUp`/`MoveDown` 归一化）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuDir {
    /// 上移（`↑`）。
    Up,
    /// 下移（`↓`）。
    Down,
}

/// 主菜单项目标（`docs/business/07_ui_flow.md` §1 转换表 Index 出边的路由语义）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTarget {
    /// 菜单项 0「新游戏」→ 角色选择。
    SelectRole,
    /// 菜单项 1「继续」→ 存读档。
    Saveload,
    /// 菜单项 2/3/4「选项/说明」→ 消息（`MsgKind::Help` 打开源场景 `FromIndex`）。
    Help,
    /// 菜单项 5「退出」→ 退出请求。
    Exit,
}

impl MenuTarget {
    /// 菜单项索引（0..=5）→ 目标（越界返回 `None`，不 panic）。
    #[must_use]
    pub const fn from_index(idx: usize) -> Option<Self> {
        Some(match idx {
            0 => Self::SelectRole,
            1 => Self::Saveload,
            2..=4 => Self::Help,
            5 => Self::Exit,
            _ => return None,
        })
    }
}

/// 场景切换状态（业务文档 `docs/business/loop_core.md` §切换时序）。
///
/// 过渡期状态机：`Menu → (Unloading) → Transition → 目标场景`，
/// 战斗经 `Battle`，过场动画经 `Anim26`/`Anim27`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneState {
    /// 主菜单态。
    Menu,
    /// 场景卸载中。
    Unloading,
    /// 回主菜单。
    BackToMenu,
    /// 过场/坐标切换。
    Transition,
    /// 战斗态。
    Battle,
    /// 过场动画 phase A。
    Anim26,
    /// 过场动画 phase B。
    Anim27,
}

impl SceneState {
    /// 状态码 → 枚举（未知码 `None`）。
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

    /// 全部合法状态码（7 个，按数值排序，遍历用）。
    #[must_use]
    pub const fn all_codes() -> [u32; 7] {
        [0, 2, 3, 4, 25, 26, 27]
    }
}

/// 可装载的场景资源 id。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    /// 片头。
    Intro,
    /// 角色选择。
    SelectRole,
    /// 地图选择。
    SelectMap,
    /// 主菜单。
    Index,
    /// 主地图。
    MainMap,
    /// 战斗。
    Vs,
}

impl SceneId {
    /// 场景判别值（0..=5，一场景一值，`docs/business/07_ui_flow.md` §1 硬规则 2）。
    #[must_use]
    pub const fn discriminant(self) -> u32 {
        match self {
            Self::Intro => 0,
            Self::SelectRole => 1,
            Self::SelectMap => 2,
            Self::Index => 3,
            Self::MainMap => 4,
            Self::Vs => 5,
        }
    }
}
