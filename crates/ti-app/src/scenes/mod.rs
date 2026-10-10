//! S8.4 场景 trait + 场景注册表 + `SceneHost` 能力边界（`docs/plan-s8-refactor.md` §5）。
//!
//! ## 内聚边界（S8.4 判据）
//!
//! 场景 struct **不持有** `GameApp`（否则摸遍内部字段 = 全透明耦合），
//! 而持有 `&mut dyn SceneHost` —— 一个**窄能力接口**，只暴露场景真正需要的行为：
//! - [`SceneHost::enter`]：切到某场景（场景 struct 不碰 `scene` 字段）；
//! - [`SceneHost::current_scene`]：当前场景（判断是否需要重分发）；
//! - [`SceneHost::play_sfx`]：播放音效（场景 struct 不碰 `last_sfx` 字段）；
//! - [`SceneHost::advance`]：把归一输入交给宿主处理（业务判定仍在 `GameApp`，
//!   场景 struct 只做「我是哪个场景」的路由，不重抄业务）。
//!
//! `GameApp` 实现 `SceneHost`（`impl SceneHost for GameApp`），内部调私有 `handle_X`。
//! 场景 struct 与 `GameApp` 之间只有这一个 trait 边界，字段全私有（内聚 + 低耦）。

use crate::game_app::{GameInput, Scene as LegacyScene};

pub mod index;
pub mod main_map;
pub mod message;
pub mod saveload;
pub mod select_map;
pub mod select_role;
pub mod vs;

pub use index::IndexScene;
pub use main_map::MainMapScene;
pub use message::MessageScene;
pub use saveload::SaveloadScene;
pub use select_map::SelectMapScene;
pub use select_role::SelectRoleScene;
pub use vs::VsScene;

/// 场景标识（与 `LegacyScene` 变体一一对应，渲染键/跳转目标用它）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneKind {
    /// 片头。
    Intro,
    /// 主菜单。
    Index,
    /// 角色选择。
    SelectRole,
    /// 地图选择。
    SelectMap,
    /// 主地图。
    MainMap,
    /// 战斗。
    Vs,
    /// 消息/帮助。
    Message,
    /// 存读档。
    Saveload,
    /// 退出。
    Exit,
}

impl SceneKind {
    /// 旧 `LegacyScene` → `SceneKind`（迁移过渡期桥接）。
    #[must_use]
    pub fn from_legacy(s: LegacyScene) -> Self {
        match s {
            LegacyScene::Intro => Self::Intro,
            LegacyScene::Index => Self::Index,
            LegacyScene::SelectRole => Self::SelectRole,
            LegacyScene::SelectMap => Self::SelectMap,
            LegacyScene::MainMap => Self::MainMap,
            LegacyScene::Vs => Self::Vs,
            LegacyScene::Message => Self::Message,
            LegacyScene::Saveload => Self::Saveload,
            LegacyScene::Exit => Self::Exit,
        }
    }

    /// `SceneKind` → 旧 `LegacyScene`（渲染分发仍走旧枚举，过渡期）。
    #[must_use]
    pub fn to_legacy(self) -> LegacyScene {
        match self {
            Self::Intro => LegacyScene::Intro,
            Self::Index => LegacyScene::Index,
            Self::SelectRole => LegacyScene::SelectRole,
            Self::SelectMap => LegacyScene::SelectMap,
            Self::MainMap => LegacyScene::MainMap,
            Self::Vs => LegacyScene::Vs,
            Self::Message => LegacyScene::Message,
            Self::Saveload => LegacyScene::Saveload,
            Self::Exit => LegacyScene::Exit,
        }
    }
}

/// 场景宿主能力接口（S8.4 内聚边界：场景 struct 唯一的对外依赖）。
///
/// 场景 struct 经此 trait 操作宿主（切场景 / 播音效 / 把输入交回宿主判定），
/// **不接触** `GameApp` 的任何字段（字段全私有，`docs/process/01_architecture.md` §2）。
pub trait SceneHost {
    /// 当前顶层场景（场景 struct 判断分发目标）。
    fn current_scene(&self) -> LegacyScene;

    /// 切到 `next` 场景（宿主更新 `scene` 字段并重分发）。
    fn enter(&mut self, next: SceneKind);

    /// 把归一输入交给宿主业务判定（`GameApp::handle_X`，业务内聚在宿主）。
    /// 返回是否发生了场景切换。
    fn advance(&mut self, input: GameInput) -> bool;

    /// 播放音效（宿主更新 `last_sfx`，壳层消费）。
    fn play_sfx(&mut self, sfx: ti_core::oracle::menu::Sfx);

    /// 渲染本场景（宿主调私有 `render_X`；渲染细节内聚在宿主）。
    fn render_current(&mut self);
}

/// 场景处理器 trait（S8.4 核心：键盘/触摸共用 `on_input`，渲染走 `render`）。
///
/// 实现方 = 每个场景一个 struct（`scenes/*.rs`），持有 `&mut dyn SceneHost`（窄能力）。
/// 返回 `Option<SceneKind>`：`Some` = 切到该场景，`None` = 停留。
pub trait Scene {
    /// 当前场景标识。
    #[must_use]
    fn kind(&self) -> SceneKind;

    /// 处理一个输入事件（键盘/触摸统一归一到 `GameInput`）。
    /// 返回 `Some(kind)` 表示切到该场景；`None` 停留。
    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> Option<SceneKind>;

    /// 渲染当前场景到帧缓冲（经 `SceneHost::render_current`，渲染细节内聚在宿主）。
    fn render(&mut self, host: &mut dyn SceneHost);
}

/// 场景注册表：按 `SceneKind` 分发（取代 `game_app.rs` 的硬编码 match）。
///
/// 持有当前激活场景的 [`SceneKind`]（场景 struct 本身由调用方按需构造，
/// 避免 trait object 借用 `GameApp` 造成自引用）。
pub struct SceneRegistry {
    /// 当前激活场景（`None` = 未初始化，回退旧 `handle` 路径）。
    active: Option<SceneKind>,
}

impl Default for SceneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneRegistry {
    /// 新建空注册表（过渡期：未注册时 `dispatch` 返回 `None`，调用方回退旧路径）。
    #[must_use]
    pub fn new() -> Self {
        Self { active: None }
    }

    /// 注册并激活某场景（替换当前激活场景）。
    pub fn set_active(&mut self, kind: SceneKind) {
        self.active = Some(kind);
    }

    /// 当前激活场景（未注册 → `None`）。
    #[must_use]
    pub fn active_kind(&self) -> Option<SceneKind> {
        self.active
    }

    /// 是否有激活场景（迁移进度探针：全场景注册后恒 `true`）。
    #[must_use]
    pub fn has_active(&self) -> bool {
        self.active.is_some()
    }
}
