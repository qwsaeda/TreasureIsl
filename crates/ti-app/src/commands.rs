//! S9.5：Command 派发模式——把归一输入（[`GameInput`]）统一为可派发命令枚举，
//! 每个命令变体对应一种游戏操作，经 [`Command::execute`] 路由到 host 的
//! 8 个子 trait 方法（`NavHost`/`RenderHost`/`SpriteHost`/`MenuHost`/
//! `SelectionHost`/`MapHost`/`EconomyHost`/`SessionHost`）。
//!
//! 设计：
//! - [`Command`] trait：`fn execute(&self, host: &mut dyn SceneHost) -> bool`
//!   （返回是否消费了该输入，与 `route_input` 语义一致）。
//! - [`GameCommand`]：可派发枚举，每个变体 = 一种游戏操作；
//!   `impl Command for GameCommand` 把各变体委托给 `scenes::route_input`。
//! - 命令枚举可自由携带参数（`Building`/`Dir`/`u32`/`usize`），
//!   与现有 `GameInput` 1:1 对齐，新增变体只需加一个 match 分支。

use ti_core::domain::economy::Building;
use ti_core::domain::map::Dir;

use crate::game_app::GameInput;
use crate::scenes::{SceneHost, SceneKind, route_input};

/// 可派发命令 trait。
///
/// `execute` 把 `self` 路由到当前场景（`host.current_scene()`），
/// 返回是否发生了场景切换（与 [`GameApp::handle`](crate::game_app::GameApp::handle) 一致）。
pub trait Command {
    /// 在 `host` 上执行本命令，返回是否消费了该输入。
    fn execute(&self, host: &mut dyn SceneHost) -> bool;
}

/// 可派发命令枚举（与 [`GameInput`] 1:1 对齐）。
///
/// 每个变体 = 一种游戏操作；`Command` 实现把变体映射回 `GameInput` 并
/// 经 `scenes::route_input` 路由到对应场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCommand {
    /// 菜单/选项上移（循环）。
    MoveUp,
    /// 菜单/选项下移（循环）。
    MoveDown,
    /// 确认（菜单执行/角色/地图选定/战斗攻击）。
    Confirm,
    /// 返回（回上级场景/战斗退出）。
    Back,
    /// 上一页/上一个（选角/选图）。
    Prev,
    /// 下一页/下一个（选角/选图）。
    Next,
    /// 主地图方向移动。
    Walk(Dir),
    /// 主地图据点触发（事件码 1..8）。
    Landmark(u32),
    /// 主地图贸易：买（`gid`）。
    TradeBuy(u32),
    /// 主地图贸易：卖（`gid`）。
    TradeSell(u32),
    /// 主地图建造（`kind`）。
    Build(Building),
    /// 战斗：攻击（推进一回合）。
    Attack,
    /// 存读档：读（`slot` 索引 0..2）。
    LoadSave(usize),
    /// 存读档：写（`slot` 索引 0..2）。
    WriteSave(usize),
    /// 消息翻页：下一页。
    MsgNext,
    /// 消息翻页：上一页。
    MsgPrev,
    /// 消息关闭（回场景源）。
    MsgClose,
    /// 存读档子窗口关闭（回 `Index`）。
    SaveloadClose,
    /// 主地图回主菜单（回 `Index`）。
    MapBack,
}

impl GameCommand {
    /// 转回 `GameInput`（与 `scenes::route_input` 消费的类型一致）。
    #[must_use]
    pub fn to_input(self) -> GameInput {
        match self {
            Self::MoveUp => GameInput::MoveUp,
            Self::MoveDown => GameInput::MoveDown,
            Self::Confirm => GameInput::Confirm,
            Self::Back => GameInput::Back,
            Self::Prev => GameInput::Prev,
            Self::Next => GameInput::Next,
            Self::Walk(d) => GameInput::Walk(d),
            Self::Landmark(c) => GameInput::Landmark(c),
            Self::TradeBuy(g) => GameInput::TradeBuy(g),
            Self::TradeSell(g) => GameInput::TradeSell(g),
            Self::Build(b) => GameInput::Build(b),
            Self::Attack => GameInput::Attack,
            Self::LoadSave(s) => GameInput::LoadSave(s),
            Self::WriteSave(s) => GameInput::WriteSave(s),
            Self::MsgNext => GameInput::MsgNext,
            Self::MsgPrev => GameInput::MsgPrev,
            Self::MsgClose => GameInput::MsgClose,
            Self::SaveloadClose => GameInput::SaveloadClose,
            Self::MapBack => GameInput::MapBack,
        }
    }

    /// 命令是否携带参数（无参命令 = 纯状态机推进）。
    #[must_use]
    pub const fn has_param(&self) -> bool {
        matches!(
            self,
            Self::Walk(_)
                | Self::Landmark(_)
                | Self::TradeBuy(_)
                | Self::TradeSell(_)
                | Self::Build(_)
                | Self::LoadSave(_)
                | Self::WriteSave(_)
        )
    }
}

impl Command for GameCommand {
    fn execute(&self, host: &mut dyn SceneHost) -> bool {
        let input = (*self).to_input();
        let kind = SceneKind::from_legacy(host.current_scene());
        route_input(host, kind, input)
    }
}
