//! S8.4 场景层：`SceneHost` 能力接口 + `Scene` trait + 路由骨架。
//!
//! ## 分层边界（S9 骨架改造）
//!
//! - `GameApp`（`game_app.rs`）= **组合根**：持有全部状态，实现 [`SceneHost`]，
//!   只保留键位映射（`poll`）、触摸入口（`poll_touch`）、帧调度（`render`）。
//! - `scenes/*.rs` = **场景模块**：每个场景一个 struct，实现 [`Scene`]，
//!   持有该场景的输入判定（`on_input`）、触摸手势（`on_touch`）与绘制（`render`）。
//! - 场景 struct **不持有** `GameApp`，只持有 `&mut dyn SceneHost`，
//!   因此业务判定与渲染细节真正离开组合根，`game_app.rs` 不再有
//!   `handle_X` / `touch_X` / `render_X` 巨��分支。
//!
//! ## 为什么是「一个宽接口」而不是 7 个窄接口
//!
//! Rust 不允许 trait object 作为另一个 trait 的 supertrait
//! （`trait A: B` + `dyn A` 会因 `B: Sized` 报错），
//! 因此动态分发只能落在**单个宽接口**上。这是过渡期形态：
//! 场景层一旦稳定，可把宽接口按场景切成若干**具体泛型**参数
//! （`IndexScene<H: IndexOps>` 等），届时再收窄。收敛判据见
//! `docs/process/01_architecture.md` §2。
//!
//! 所有方法都是**一行的字段读写**，不含业务判定——业务在 `scenes/*.rs`。

use std::marker::PhantomData;

use ti_core::domain::economy::Building;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{Dir, GridPos, MapTerrain, SceneTarget};
use ti_core::domain::messages::MsgKind;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot, SelectionState};
use ti_core::rules::main_map::StepResult;

use crate::game_app::{BattleSnapshot, GameInput, Scene as LegacyScene};
use crate::menu_shell::MenuShell;
use crate::touch::TouchSample;

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

    /// 场景对应的 `.Map` 精灵资源名（`docs/design/ui_controls.md` §3）。
    ///
    /// `Intro` 复用 `index.Map`；`Exit` 无资源（纯清屏）。
    #[must_use]
    pub const fn asset_name(self) -> &'static str {
        match self {
            Self::Index | Self::Intro => "index.Map",
            Self::SelectRole => "selectrole.Map",
            Self::SelectMap => "selectmap.Map",
            Self::MainMap => "main.Map",
            Self::Vs => "vs.Map",
            Self::Message => "msgs.Map",
            Self::Saveload => "saveload.Map",
            Self::Exit => "",
        }
    }

    /// 场景唯一判别值（0..=8，9 个变体各占一个 i32，无碰撞）。
    ///
    /// 用于壳层静态帧优化（`last_rendered_scene`），替代 `Scene::key()` 的
    /// `i32` 键（`Intro|Exit => 7` 碰撞，S10.2 修复）。
    #[must_use]
    pub const fn discriminant(self) -> i32 {
        match self {
            Self::Intro => 0,
            Self::Index => 1,
            Self::SelectRole => 2,
            Self::SelectMap => 3,
            Self::MainMap => 4,
            Self::Vs => 5,
            Self::Message => 6,
            Self::Saveload => 7,
            Self::Exit => 8,
        }
    }
}

/// Type-State 场景转换：编译期保证只有合法转换才能构造（S9.4）。
///
/// `From`/`To` 为场景标记类型（[`markers`]），只有 impl 块中声明的合法转换
/// 才能构造；非法转换（如 `SelectRole → Vs`、`MainMap → Saveload`）无 impl，
/// 编译期直接报错。合法转换表见下方 impl 块注释（与 `scenes/*.rs` 实际跳转一一对应）。
///
/// 使用示例（编译期可静态证明 `SelectMap → MainMap` 合法、`SelectRole → Vs` 非法）：
///
/// ```compile_fail
/// use ti_app::scenes::{SceneTransition, markers};
/// // 合法转换可构造：
/// let _ok: SceneTransition<markers::SelectMap, markers::MainMap> = SceneTransition::new();
/// // 非法转换编译报错（无对应 impl 块）：
/// let _bad: SceneTransition<markers::SelectRole, markers::Vs> = SceneTransition::new();
/// ```
pub struct SceneTransition<From, To> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
}

pub mod markers {
    //! 场景类型标记（零大小，仅用于类型参数），供 [`SceneTransition`] 使用。

    /// `Intro` 场景标记。
    pub enum Intro {}
    /// `Index` 场景标记。
    pub enum Index {}
    /// `SelectRole` 场景标记。
    pub enum SelectRole {}
    /// `SelectMap` 场景标记。
    pub enum SelectMap {}
    /// `MainMap` 场景标记。
    pub enum MainMap {}
    /// `Vs` 场景标记。
    pub enum Vs {}
    /// `Message` 场景标记。
    pub enum Message {}
    /// `Saveload` 场景标记。
    pub enum Saveload {}
    /// `Exit` 场景标记。
    pub enum Exit {}
}

// 合法转换表（业务侧唯一事实源 = `docs/business/07_ui_flow.md` §1 场景流）：
//
//   Intro      → Index
//   Index      → SelectRole / Saveload / Message / Exit
//   SelectRole → SelectMap / MainMap / Index
//   SelectMap  → MainMap / SelectRole
//   MainMap    → Vs / Message / Index
//   Vs         → Index
//   Message    → MainMap / Index
//   Saveload   → Index / MainMap / SelectRole
//
// 非法转换（如 `SelectRole → Vs`、`MainMap → Saveload`）没有 macro 调用 = 没有 impl 块，
// 编译期直接报错（与 `scenes/*.rs` 实际跳转一一对应）。

/// 生成一条合法场景转换的构造器 + `Default`（类型即约束，S9.4）。
///
/// 每条合法转换调用一次；`Default` 只为满足 trait 约定，语义等价 [`SceneTransition::new`]。
macro_rules! scene_transition {
    ($from:ident, $to:ident, $doc:expr) => {
        #[doc = $doc]
        impl SceneTransition<markers::$from, markers::$to> {
            /// 构造该场景转换（无字段；能否构造取决于 impl 块是否存在）。
            #[must_use]
            pub fn new() -> Self {
                Self { _from: PhantomData, _to: PhantomData }
            }
        }

        impl Default for SceneTransition<markers::$from, markers::$to> {
            /// 与 [`SceneTransition::new`] 等价。
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

scene_transition!(Intro, Index, "`Intro` → `Index`（片头结束即跳主菜单）");
scene_transition!(Index, SelectRole, "`Index` → `SelectRole`（菜单项 0 新游戏）");
scene_transition!(Index, Saveload, "`Index` → `Saveload`（菜单项 1 继续）");
scene_transition!(Index, Message, "`Index` → `Message`（菜单项 2/3/4 选项/说明）");
scene_transition!(Index, Exit, "`Index` → `Exit`（菜单项 5 退出）");
scene_transition!(SelectRole, SelectMap, "`SelectRole` → `SelectMap`（角色选定，地图未定）");
scene_transition!(SelectRole, MainMap, "`SelectRole` → `MainMap`（角色 + 地图齐全）");
scene_transition!(SelectRole, Index, "`SelectRole` → `Index`（返回）");
scene_transition!(SelectMap, MainMap, "`SelectMap` → `MainMap`（地图选定）");
scene_transition!(SelectMap, SelectRole, "`SelectMap` → `SelectRole`（返回）");
scene_transition!(MainMap, Vs, "`MainMap` → `Vs`（事件码 6 dispatch）");
scene_transition!(MainMap, Message, "`MainMap` → `Message`（事件码 1/4/2/5/7/8 dispatch）");
scene_transition!(MainMap, Index, "`MainMap` → `Index`（`MapBack` 返回键）");
scene_transition!(Vs, Index, "`Vs` → `Index`（战斗结束/中途退出）");
scene_transition!(Message, MainMap, "`Message` → `MainMap`（关闭，场景源为主地图）");
scene_transition!(Message, Index, "`Message` → `Index`（关闭，场景源为主菜单）");
scene_transition!(Saveload, Index, "`Saveload` → `Index`（子窗口关闭）");
scene_transition!(Saveload, MainMap, "`Saveload` → `MainMap`（读档恢复：角色 + 地图齐全）");
scene_transition!(Saveload, SelectRole, "`Saveload` → `SelectRole`（读档恢复：角色/地图缺失）");

/// 场景宿主能力接口：`GameApp` 实现的**唯一**状态/表现出口。
///
/// 场景 struct 经此 trait 读写状态与帧缓冲，**不接触** `GameApp` 任何字段
/// （`docs/process/01_architecture.md` §2）。每个方法都是一行读写，
/// 业务判定留在 `scenes/*.rs`。
pub trait SceneHost {
    // ── 场景控制 ────────────────────────────────────────────────────────
    /// 当前顶层场景（宿主自有字段）。
    fn current_scene(&self) -> LegacyScene;

    /// 切到 `next` 场景（宿主更新 `scene` 字段并重分发）。
    fn enter(&mut self, next: SceneKind);

    /// 把归一输入交回宿主路由（`GameApp::handle`）——触摸/手势内部再次派生
    /// `GameInput` 时经此递归进入同一场景的 `on_input`。
    fn advance(&mut self, input: GameInput) -> bool;

    // ── 表现层 ──────────────────────────────────────────────────────────
    /// 只读帧缓冲/字形视图（场景渲染读 `selected` 等）。
    fn shell(&self) -> &MenuShell;

    /// 可写帧缓冲/字形视图（场景渲染写像素与文本）。
    fn shell_mut(&mut self) -> &mut MenuShell;

    /// 清屏到 `color`（565 色）。
    fn clear_frame(&mut self, color: u16);

    /// 填矩形。
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16);

    /// 绘制一行字形。
    fn draw_text(&mut self, text: &str, x: i32, y: i32);

    /// 把 `asset` 对应的 `.Map` 真实精灵 blit 到帧缓冲（宿主内做图集/帧缓冲
    /// 的同时可变借用，场景层拿不到两个借用）。
    fn blit_scene_sprites(&mut self, asset: &str);

    /// 绘制真实地形瓦片（等距投影 + 摄像机），返回是否真的画出了瓦片。
    fn draw_terrain(&mut self) -> bool;

    /// 地形网格宽（缺资源时回退 20）。
    fn terrain_grid_w(&self) -> u32;

    /// 地形摄像机中心偏移 `(x, y)`（缺资源时 `(0, 0)`）。
    fn terrain_camera(&self) -> (i64, i64);

    /// 是否加载了真实地形瓦片图集（决定触摸命中走等距逆变换还是占位网格）。
    fn has_terrain_atlas(&self) -> bool;

    // ── 音效 ────────────────────────────────────────────────────────────
    /// 置本次交互的音效（宿主 `last_sfx`，壳层消费）。
    fn play_sfx(&mut self, sfx: Sfx);

    // ── Index 场景 ──────────────────────────────────────────────────────
    /// 主菜单选中项。
    fn menu_selected(&self) -> usize;
    /// 置主菜单选中项。
    fn set_menu_selected(&mut self, value: usize);
    /// 新游戏：重置选择状态 + 清空角色/地图。
    fn reset_selection(&mut self);
    /// 打开帮助消息（`MsgKind::Help`，首页）。
    fn open_help(&mut self);
    /// 置退出标志。
    fn request_quit(&mut self);

    // ── 选择场景 ────────────────────────────────────────────────────────
    /// 当前角色。
    fn character(&self) -> Option<Character>;
    /// 置当前角色。
    fn set_character(&mut self, value: Option<Character>);
    /// 当前地图槽。
    fn map_slot(&self) -> Option<MapSlot>;
    /// 置当前地图槽。
    fn set_map_slot(&mut self, value: Option<MapSlot>);
    /// 选择状态快照（`Copy`）。
    fn selection(&self) -> SelectionState;
    /// 选择状态可写引用。
    fn selection_mut(&mut self) -> &mut SelectionState;
    /// 是否已解锁 `slot`。
    fn map_unlocked(&self, slot: MapSlot) -> bool;
    /// 进入主地图（初始化 `MapSession` + 经济会话，三场景共用）。
    fn enter_main_map(&mut self);

    // ── 主地图 ──────────────────────────────────────────────────────────
    /// 当前玩家格。
    fn map_pos(&self) -> Option<GridPos>;
    /// 置当前玩家格。
    fn set_map_pos(&mut self, value: Option<GridPos>);
    /// 地形（`Copy` 快照，触摸预检用）。
    fn map_terrain(&self) -> Option<MapTerrain>;
    /// 据点坐标（拷贝出，避免与帧缓冲借用重叠）。
    fn map_landmarks(&self) -> Vec<(u32, u32)>;
    /// 走一步（宿主内同时借用 `map_session` 与 `rng`，场景层拿不到两个借用）。
    fn map_session_move(&mut self, dir: Dir) -> StepResult;
    /// 事件码 → 场景目标（`rules::main_map::MapSession::dispatch`）。
    fn map_session_dispatch(&self, code: u32) -> Option<SceneTarget>;
    /// 记录最近一次分派结果（供 HUD/测试观察）。
    fn set_last_dispatch(&mut self, target: Option<SceneTarget>);
    /// 开一场战斗（宿主取 RNG seed + tier 建会话）。
    fn start_battle(&mut self);
    /// 打开指定消息会话（首页 + 总页数 + 切场景）。
    fn open_message(&mut self, kind: MsgKind);

    // ── 经济 ────────────────────────────────────────────────────────────
    /// 商品 `gid` 单价（买入价，`balance.trade.goods`）。
    fn price_of(&self, gid: u32) -> Option<u32>;
    /// 商品 `gid` 卖出单价（`round(buy × markup)`，`balance.trade.markup`，S10.3 改动 3）。
    fn sell_price_of(&self, gid: u32) -> Option<u32>;
    /// 当前金。
    fn gold(&self) -> u32;
    /// 置当前金。
    fn set_gold(&mut self, value: u32);
    /// 持有表中 `gid` 的下标。
    fn owned_index_of(&self, gid: u32) -> Option<usize>;
    /// 持有表追加 `(gid, qty)`。
    fn owned_push(&mut self, gid: u32, qty: u32);
    /// 持有表第 `index` 项数量 +1。
    fn owned_bump(&mut self, index: usize);
    /// 持有表 `gid` 的总数量。
    fn owned_count(&self, gid: u32) -> u32;
    /// 持有表条数（HUD 用）。
    fn owned_total(&self) -> usize;
    /// 移除 `gid` 全部持有，返回移除数量。
    fn owned_take_all(&mut self, gid: u32) -> u32;
    /// 银行金。
    fn bank_gold(&self) -> u32;
    /// 银行扣款。
    fn spend_bank_gold(&mut self, amount: u32);
    /// 建筑列表 `kind` 等级 +1（不存在则新增）。
    fn build_increment(&mut self, kind: Building);

    // ── 战斗 ────────────────────────────────────────────────────────────
    /// 推进一个回合并同步胜负/胜利音效（宿主内就地改 `BattleView`）。
    fn battle_step(&mut self);
    /// 战斗面板快照（渲染用）。
    fn battle_snapshot(&self) -> Option<BattleSnapshot>;

    // ── 消息 ────────────────────────────────────────────────────────────
    /// 当前消息种类。
    fn msg_kind(&self) -> Option<MsgKind>;
    /// 当前页码（0 起）。
    fn msg_page(&self) -> u32;
    /// 总页数。
    fn msg_total(&self) -> u32;
    /// 下一页（末页保持）。
    fn msg_next_page(&mut self);
    /// 上一页（首页回末页）。
    fn msg_prev_page(&mut self);
    /// 关闭消息并回到场景源。
    fn close_message(&mut self);

    // ── 存读档 ──────────────────────────────────────────────────────────
    /// 存档槽快照（`Copy` 切片）。
    fn save_slots(&self) -> &[SaveSlot];
    /// 写存档槽（越界忽略）。
    fn write_save_slot(&mut self, slot: usize, data: SaveSlot);
    /// 按存档快照恢复状态并跳到主地图/选角。
    fn restore_save(&mut self, data: SaveSlot);

    // ── 状态查询 ────────────────────────────────────────────────────────
    /// 当前是否处于主地图（有玩家格）。
    fn in_map(&self) -> bool;
    /// 当前是否有战斗会话。
    fn in_battle(&self) -> bool;
}

/// 场景处理器 trait：键盘/触摸共用输入面，渲染单独一方法。
///
/// 实现方 = 每个场景一个 struct（`scenes/*.rs`），只持有 `&mut dyn SceneHost`。
pub trait Scene {
    /// 当前场景标识。
    #[must_use]
    fn kind(&self) -> SceneKind;

    /// 处理一个归一输入事件。返回是否发生了场景切换。
    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool;

    /// 处理一次完成的触摸手势（默认不消费）。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        let _ = (host, sample);
        false
    }

    /// 渲染本场景到帧缓冲。
    fn render(&mut self, host: &mut dyn SceneHost);
}

/// 据点事件分派目标 → 切场景（`MainMap` 触发，`Message`/`Vs` 共用）。
///
/// 跨场景的公共跳转点，故放路由层而非任一场景模块。
pub(crate) fn enter_target(host: &mut dyn SceneHost, target: SceneTarget) -> bool {
    match target {
        SceneTarget::Vs => {
            host.start_battle();
            true
        }
        SceneTarget::Msgl => {
            host.open_message(MsgKind::Msgl);
            true
        }
        // 卡片事件码（`spec/main_map.md §d` 3/5）与 Msgm 共用 `msgm.Map` 会话
        SceneTarget::Msgm | SceneTarget::Msgs | SceneTarget::Card | SceneTarget::Bigcard => {
            host.open_message(MsgKind::Msgm);
            true
        }
    }
}

/// 路由：一个输入事件交给 `kind` 对应场景的 [`Scene::on_input`]。
///
/// `Intro`/`Exit` 无场景 struct，在这里就地处理。
#[must_use]
pub fn route_input(host: &mut dyn SceneHost, kind: SceneKind, input: GameInput) -> bool {
    match kind {
        SceneKind::Intro => {
            host.enter(SceneKind::Index);
            true
        }
        SceneKind::Exit => false,
        SceneKind::Index => IndexScene.on_input(host, input),
        SceneKind::SelectRole => SelectRoleScene.on_input(host, input),
        SceneKind::SelectMap => SelectMapScene.on_input(host, input),
        SceneKind::MainMap => MainMapScene.on_input(host, input),
        SceneKind::Vs => VsScene.on_input(host, input),
        SceneKind::Message => MessageScene.on_input(host, input),
        SceneKind::Saveload => SaveloadScene.on_input(host, input),
    }
}

/// 路由：一次触摸手势交给 `kind` 对应场景的 [`Scene::on_touch`]。
#[must_use]
pub fn route_touch(host: &mut dyn SceneHost, kind: SceneKind, sample: &TouchSample) -> bool {
    match kind {
        SceneKind::Intro => {
            host.enter(SceneKind::Index);
            true
        }
        SceneKind::Exit => false,
        SceneKind::Index => IndexScene.on_touch(host, sample),
        SceneKind::SelectRole => SelectRoleScene.on_touch(host, sample),
        SceneKind::SelectMap => SelectMapScene.on_touch(host, sample),
        SceneKind::MainMap => MainMapScene.on_touch(host, sample),
        SceneKind::Vs => VsScene.on_touch(host, sample),
        SceneKind::Message => MessageScene.on_touch(host, sample),
        SceneKind::Saveload => SaveloadScene.on_touch(host, sample),
    }
}

/// 路由：绘制 `kind` 对应场景（`Exit` 清屏）。
pub fn route_render(host: &mut dyn SceneHost, kind: SceneKind) {
    match kind {
        // 片头与主菜单共用同一张 `index.Map` + 同一套菜单绘制
        SceneKind::Intro | SceneKind::Index => IndexScene.render(host),
        SceneKind::Exit => host.clear_frame(0x0000),
        SceneKind::SelectRole => SelectRoleScene.render(host),
        SceneKind::SelectMap => SelectMapScene.render(host),
        SceneKind::MainMap => MainMapScene.render(host),
        SceneKind::Vs => VsScene.render(host),
        SceneKind::Message => MessageScene.render(host),
        SceneKind::Saveload => SaveloadScene.render(host),
    }
}

/// 场景注册表（迁移进度探针 + 显式路由入口）。
///
/// 真正的路由已由 [`route_input`] / [`route_touch`] / [`route_render`] 承担；
/// 本 struct 只保留「当前激活场景」这一可观测状态，供壳层/HUD 查询。
pub struct SceneRegistry {
    /// 当前激活场景（`None` = 未初始化）。
    active: Option<SceneKind>,
}

impl Default for SceneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneRegistry {
    /// 新建空注册表。
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
