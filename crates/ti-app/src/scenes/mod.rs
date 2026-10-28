//! S8.4 场景层：`SceneHost` 能力接口 + `Scene` trait + 路由骨架。
//!
//! ## 分层边界（S9 骨架改造 + S9.2 接口拆分）
//!
//! - `GameApp`（`game_app.rs`）= **组合根**：持有全部状态，实现 [`SceneHost`]
//!   （8 个子 trait 的 supertrait 聚合），只保留键位映射（`poll`）、触摸入口
//!   （`poll_touch`）、帧调度（`render`）。
//! - `scenes/*.rs` = **场景模块**：每个场景一个 struct，实现 [`Scene`]，
//!   持有该场景的输入判定（`on_input`）、拾取命中（`on_hit`）与绘制（`render`）。
//! - 场景 struct **不持有** `GameApp`，只持有 `&mut dyn SceneHost`，
//!   因此业务判定与渲染细节真正离开组合根，`game_app.rs` 不再有
//!   `handle_X` / `touch_X` / `render_X` 巨大分支。
//!
//! ## S9.2 接口拆分（8 个子 trait）
//!
//! 原单一宽接口 `SceneHost`（62 方法）按职责拆为 8 个子 trait：
//! [`NavHost`] / [`RenderHost`] / [`SpriteHost`] / [`MenuHost`] /
//! [`SelectionHost`] / [`MapHost`] / [`EconomyHost`] / [`SessionHost`]。
//! [`SceneHost`] 退化为 8 个子 trait 的 supertrait 聚合（空 trait），
//! `dyn SceneHost` 仍可访问全部方法（supertrait 机制）。
//!
//! 所有方法都是**一行的字段读写**，不含业务判定——业务在 `scenes/*.rs`。

use std::marker::PhantomData;

use ti_core::domain::economy::Building;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{GridPos, MapTerrain, SceneTarget};
use ti_core::domain::messages::MsgKind;
use ti_core::domain::persistence::{SaveSlot, SaveSlotV4Ext};
use ti_core::domain::selection::{Character, MapSlot, SelectionState};
use ti_core::rules::board::{TrackSpace, Turn};
use ti_core::rules::territory::Ownership;

use crate::game_app::{BattleSnapshot, GameInput, HitData, Scene as LegacyScene};
use crate::menu_shell::MenuShell;

pub mod chrome;
pub mod index;
pub mod main_map;
pub mod main_map_island;
pub mod main_map_node;
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

/// S9.2：场景导航接口（`GameApp` 实现，场景侧经 `dyn SceneHost` 调用）。
pub trait NavHost {
    /// 当前顶层场景（宿主自有字段）。
    fn current_scene(&self) -> LegacyScene;

    /// 切到 `next` 场景（宿主更新 `scene` 字段并重分发）。
    fn enter(&mut self, next: SceneKind);

    /// 把归一输入交回宿主路由（`GameApp::handle`）——触摸/手势内部再次派生
    /// `GameInput` 时经此递归进入同一场景的 `on_input`。
    fn advance(&mut self, input: GameInput) -> bool;
}

/// S9.2：帧渲染基础接口（帧缓冲/字形读写，不含精灵/地形）。
pub trait RenderHost {
    /// 只读帧缓冲/字形视图（场景渲染读 `selected` 等）。
    fn shell(&self) -> &MenuShell;

    /// 可写帧缓冲/字形视图（场景渲染写像素与文本）。
    fn shell_mut(&mut self) -> &mut MenuShell;

    /// 渲染主题（颜色 + 字号，数据驱动）。
    fn theme(&self) -> &ti_render::Theme {
        &self.shell().theme
    }

    /// **绘制目标**（S18.7a 可插拒）：CPU 帧缓冲（`MenuShell`）或 **绘制列表**
    /// （`ti_render::drawlist::DrawList`，Bevy 路径）
    ///
    /// 场景绘制代码**不变**：两种目标用同一套原语命令，
    /// ⇒ CPU / Bevy 两后端同画面。
    fn draw_target(&mut self) -> &mut dyn ti_render::canvas::Canvas;

    /// S29.3（B1 定案）：**UI chrome 是否已迁到 Bevy `bevy_ui`**。
    ///
    /// 默认 `false` = CPU 路径照旧（Android macroquad 壳等无 `bevy_ui` 的后端）；
    /// Bevy 发货壳置 `true` ⇒ 场景层**跳过已迁 chrome 的原语**
    /// （`params["ui_layer"] == "bevy_ui"`），那些 chrome 由 `ti-engine` 的 `bevy_ui` 层绘制
    /// （避免 1080 与 `bevy_ui` **双画**）。
    ///
    /// 口径：`ti-engine` = Bevy 壳（UI chrome 进 `bevy_ui`，`docs/process/19_bevy_ui.md`）；
    /// 本方法只回答「在不在 Bevy 侧」，不含任何游戏语义。
    fn chrome_in_bevy_ui(&self) -> bool {
        false
    }

    /// 清屏到 `color`（565 色；QA120：色值走 `menu_shell` 命名常量，不写内联 hex——
    /// `check_hardcode.py` 布局调用正则已含 `clear_frame`）。
    fn clear_frame(&mut self, color: u16) {
        self.draw_target().clear_frame(color);
    }

    /// 填矩形。
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.draw_target().fill_rect(x, y, w, h, color);
    }

    /// 贴图（S18.8b `image` 原语）：资源路径 = 相对仓库根（如 `assets/ui/board.png`）。
    ///
    /// CPU 目标 = 帧缓冲（读文件 + 解码 + alpha 混合）；Bevy 目标 = 绘制列表
    /// （记录命令 → 资产服务器）。缺资源 = 不画（不 panic）。
    fn blit_image(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32) {
        self.draw_target().blit_image(asset, x, y, w, h);
    }

    /// 贴图 + 色调（S21.14 图标图片化跨层 tint）：转发到绘制目标（Bevy = 记录命令 / CPU = 帧缓冲 multiply）。
    fn blit_image_tint(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32, tint: u16) {
        self.draw_target().blit_image_tint(asset, x, y, w, h, tint);
    }

    /// 绘制一行字形（默认墨色 = `INK_COLOR`）。
    fn draw_text(&mut self, text: &str, x: i32, y: i32) {
        self.draw_target().draw_text(text, x, y, crate::menu_shell::COLOR_TEXT_ON_PARCH);
    }

    /// 绘制一行**指定墨色**的字形（正文档 44px；S15 深底需白墨）。
    fn draw_text_colored(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.draw_target().draw_text(text, x, y, color);
    }

    /// QB115：绘制一行**标题档**字形（TTF 96px / 位图回退 12px @ 26px advance，
    /// `menu_shell::TITLE_FONT_SIZE_PX`；`chrome::draw_title_text` 收敛口）。
    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.draw_target().draw_title_text(text, x, y, color);
    }

    /// QB115：绘制一行**提示档**字形（TTF 28px / 位图回退 12px @ 8px advance，
    /// `menu_shell::HINT_FONT_SIZE_PX`；hint 行/实体标签/按钮标签/页码收敛口）。
    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.draw_target().draw_hint_text(text, x, y, color);
    }

    /// 读取帧缓冲某像素颜色（供渐变/混合层读取当前状态）。
    fn pixel_at(&self, x: i32, y: i32) -> u16;
}

/// S9.2：精灵/地形渲染接口。
///
/// S15.4：legacy 像素写帧（`.Map` 精灵 blit + 等距地形瓦片 blit）已整体移除——运行时
/// 场景/主地图纯代码绘制（`08_render_arch.md` 契约 2）。本 trait 保留 2 个等距几何
/// 访问器（`terrain_grid_w` / `has_terrain_atlas`），供主地图代码棋盘（§4.3）按
/// `terrain_grid_w`² 铺菱形格。
/// S15.6 QA114：`terrain_camera`（旧 176 时代 88/110 老屏中心偏移，零消费方）已删。
pub trait SpriteHost {
    /// 地形网格宽（缺资源时回退 20）。
    fn terrain_grid_w(&self) -> u32;

    /// 是否加载了真实地形瓦片图集（决定触摸命中走等距逆变换还是占位网格）。
    fn has_terrain_atlas(&self) -> bool;
}

/// S9.2：主菜单接口（`Index` 场景的状态读写）。
pub trait MenuHost {
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

    /// 确认输入当前是否**按住**（桌面：确认键/鼠标左键每帧快照；触摸：恒 `false`，
    /// 触摸走 `chrome::flash_press` 闪烁腿）。
    ///
    /// 供 `chrome::menu_item` 判定菜单项按下凹陷（EXP-079 §7.3 三态）。
    /// 默认 `false`（无输入状态的宿主，如测试 mock / headless 渲染）。
    fn press_held(&self) -> bool {
        false
    }

    /// 本帧经过毫秒（S17.4：动效时间制的唯一时间源——翻页亮度等按 ms 推进，
    /// 帧率无关）。默认 16ms（≈60fps，测试 mock / headless 渲染用）。
    fn frame_delta_ms(&self) -> u32 {
        16
    }
}

/// S9.2：角色/地图选择接口（`SelectRole`/`SelectMap` 场景共用）。
pub trait SelectionHost {
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
    ///
    /// 返回 `false` = 数据校验失败（`balance` 缺条目 / 坐标越界），调用方
    /// （如 `restore_save`）据此回滚已写字段，避免半初始化态（FIND-204）。
    fn enter_main_map(&mut self) -> bool;
}

/// S9.2：主地图会话接口（移动/贸易/建造/事件分派）。
pub trait MapHost {
    /// 当前玩家格。
    fn map_pos(&self) -> Option<GridPos>;

    /// 置当前玩家格。
    fn set_map_pos(&mut self, value: Option<GridPos>);

    /// 地形（`Copy` 快照，触摸预检用）。
    fn map_terrain(&self) -> Option<MapTerrain>;

    /// 据点坐标（拷贝出，避免与帧缓冲借用重叠）。
    fn map_landmarks(&self) -> Vec<(u32, u32)>;

    /// BOSS 格坐标（`None` = 本地图无 BOSS 格；P0-2 渲染用，橙红三角标记）。
    fn map_boss_tile(&self) -> Option<GridPos>;

    /// 事件码 → 场景目标（`rules::main_map::MapSession::dispatch`）。
    fn map_session_dispatch(&self, code: u32) -> Option<SceneTarget>;

    /// 记录最近一次分派结果（供 HUD/测试观察）。
    fn set_last_dispatch(&mut self, target: Option<SceneTarget>);

    /// 开一场战斗（宿主取 RNG seed + tier 建会话）。
    fn start_battle(&mut self);

    /// 打开指定消息会话（首页 + 总页数 + 切场景）。
    fn open_message(&mut self, kind: MsgKind);

    /// 环岛赛道空间序列（拷贝出，渲染轨道高亮用；`MainMap` 场景时有效，无赛道 = 空）。
    /// 默认空（非主地图的 fake 宿主无需实现）。
    fn board_spaces(&self) -> Vec<TrackSpace> {
        Vec::new()
    }

    /// 当前所在赛道序号（无赛道 = 0）。
    fn board_index(&self) -> usize {
        0
    }

    /// 上一次摇出的步数（0 = 未摇过）。
    fn board_last_roll(&self) -> u32 {
        0
    }

    /// 摇骰前进 N 格 + 落格结算（`rules::board`）；返回本回合结果（`None` = 无赛道）。
    /// BOSS 触发按 `Turn::boss` 判定（`Engaged` = 现身 → 调用方切战斗场景）。默认无赛道。
    fn board_spin(&mut self) -> Option<Turn> {
        None
    }

    /// 落格提示条文本（未过期时 `Some`；场景渲染时画在棋盘下方）。默认无。
    fn board_toast(&self) -> Option<String> {
        None
    }

    /// S21.7 赛道节点预览选中（`Some(index)` = 该节点高亮 + 信息签；只读预览不移动，
    /// 摇骰时清空）。默认无（非主地图的 fake 宿主无需实现）。
    fn track_node_sel(&self) -> Option<usize> {
        None
    }

    /// S28.7 某赛道格的领地归属（`territory.md` §4；`MainMap` = 当前账本；无领地 = `None`）。
    fn territory_ownership(&self, _idx: u32) -> Option<Ownership> {
        None
    }

    /// S28.7 当前轮到的席位（§1：5 席顺序 = 角色 index 升序；`MainMap` 前 = 0）。
    fn turn_seat(&self) -> u8 {
        0
    }

    /// S28.2 存档 v4 扩展捕获（territory.md §8：5 席金/位置 + 领地图 + 当前席；无领地会话 = `None`）。
    fn save_v4_ext(&self) -> Option<SaveSlotV4Ext> {
        None
    }

    /// S28.2 存档 v4 扩展还原（席位金 / 当前席 / 领地账本；fake 宿主缺省 no-op）。
    fn restore_v4_ext(&mut self, _ext: &SaveSlotV4Ext) {}

    /// S28.2 某席位金币（`seat` 0..=4；越界 = 0）。
    fn seat_gold_of(&self, _seat: u8) -> u32 {
        0
    }

    /// S28.4 玩家席位标记（`territory.md` §1；空 = 单选旧流；fake 宿主缺省空）。
    fn player_seats(&self) -> Vec<Character> {
        Vec::new()
    }

    /// S28.4 玩家席位标记写入（`select_role` 触摸行点选 toggle 后回写）。
    fn set_player_seats(&mut self, _seats: Vec<Character>) {}

    /// S28.6 是否正在领地主副本战斗（`vs` 场景 Back 路由：副本 = 回主地图，BOSS 战 = 回主菜单）。
    fn in_dungeon_battle(&self) -> bool {
        false
    }

    /// S28.6 确认进入待决副本（`pending_dungeon` 在位时切守军战，返回 `true` = 已消费；缺省 `false`）。
    fn confirm_dungeon(&mut self) -> bool {
        false
    }

    /// 设置 / 清空赛道节点预览选中（默认 no-op）。
    fn set_track_node_sel(&mut self, _v: Option<usize>) {}

    /// S21.11 船等级（存档字段 `SaveSlot::boat_level` 读口径；1..=4，默认 1 = 小舢板）。
    fn boat_level(&self) -> u8 {
        1
    }

    /// S21.12 建造面板是否打开（默认关闭）。
    fn build_panel_open(&self) -> bool {
        false
    }

    /// S21.12 打开 / 关闭建造面板（默认 no-op）。
    fn set_build_panel_open(&mut self, _open: bool) {}

    /// S21.12 建造面板选中项（0..=4 = 城堡/投资/银行/要塞/造船；默认 0）。
    fn build_panel_sel(&self) -> u8 {
        0
    }

    /// S21.12 建造面板选中项 setter（默认 no-op）。
    fn set_build_panel_sel(&mut self, _sel: u8) {}

    /// S21.12 造船（船系统单向升级：扣金 + 抬 cargo/atk/hp，立即生效）。
    ///
    /// 返回 `true` = 升级成功；`false` = 金不够 / 已满级（场景层给提示文案）。
    /// 默认 no-op（fake 宿主不实现 ⇒ 造船永远失败，不影响其它场景）。
    fn boat_upgrade(&mut self) -> bool {
        false
    }

    /// S21.12 下一级船造价（`None` = 已满级；面板第 5 条目显示 / 满级置灰用）。
    /// 默认 `None`（fake 宿主 = 满级口径，不影响其它场景）。
    fn boat_next_cost(&self) -> Option<u32> {
        None
    }

    /// S21.12 落格提示条 setter（主地图提示签同一展示通道，120 帧寿命）。
    /// 默认 no-op。
    fn set_board_toast(&mut self, _msg: String) {}
}

/// S9.2：经济接口（贸易/建造/银行，`MainMap` 场景会话）。
pub trait EconomyHost {
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

    /// 持有表全量快照（`(goods_id, qty)`，`B` 根治：`TradeLedger` 构造用，避免逐商品 `owned_count`）。
    fn owned_snapshot(&self) -> Vec<(u32, u32)>;

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

    /// 当前金量上限（`balance.economy.gold_max`，建造可抬升；`0` = 不限制）。
    ///
    /// P1-6（T1）：经济上限机制接线——表现层卖出加金钳到 `gold_cap`，
    /// 溢出经 `settle_overflow` 进银行（`rules::trade` + `rules::build` 契约）。
    /// 默认 0（不限制），宿主可覆盖读 `balance.economy.gold_max`。
    fn gold_cap(&self) -> u32 {
        0
    }

    /// 卖出加金后结算溢出：若 `gold > gold_cap`，超出部分按 `bank_rate` 进银行。
    ///
    /// P1-6（T1）：承接 `TradeResult.overflow` 的落点（`rules::build::Bank`）。
    /// 默认 no-op（不限制），宿主实现为 `bank.deposit += excess × rate`。
    fn settle_overflow(&mut self, excess: u32) {
        let _ = excess;
    }

    /// P1-7（M5）：卡牌效果真接进可玩路径——`event_id`（`StepResult.card_event`）映射到
    /// `balance.cards` 卡并经 `rules::cards::apply_card` 结算（扣金/加金/扣 HP/加 HP）。
    ///
    /// 默认 no-op（不接卡牌），宿主实现走 `rules::cards`（数值单一事实源）。
    /// 返回是否实际改动了状态（`Applied` = 改动，`UnknownEvent` = 无改动）。
    fn apply_card_effect(&mut self, event_id: u32) -> bool {
        let _ = event_id;
        false
    }
}

/// S9.2：会话接口（音效/战斗/消息/存读档/状态查询）。
pub trait SessionHost {
    /// 置本次交互的音效（宿主 `last_sfx`，壳层消费）。
    fn play_sfx(&mut self, sfx: Sfx);

    /// 推进一个回合并同步胜负/胜利音效（宿主内就地改 `BattleView`）。
    fn battle_step(&mut self);

    /// 战斗面板快照（渲染用）。
    fn battle_snapshot(&self) -> Option<BattleSnapshot>;

    /// 显示用玩家 HP（平滑插值，EXP-085 HUD 平滑动画）。
    fn displayed_player_hp(&self) -> u32;

    /// 显示用 BOSS HP（平滑插值）。
    fn displayed_boss_hp(&self) -> u32;

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

    /// 存档槽快照（`Copy` 切片）。
    fn save_slots(&self) -> &[SaveSlot];

    /// 写存档槽（越界忽略）。
    fn write_save_slot(&mut self, slot: usize, data: SaveSlot);

    /// 按存档快照恢复状态并跳到主地图/选角。
    fn restore_save(&mut self, data: SaveSlot);

    /// 当前是否处于主地图（有玩家格）。
    fn in_map(&self) -> bool;

    /// 当前是否有战斗会话。
    fn in_battle(&self) -> bool;

    /// QB136：存读档槽光标（三态：有档=实框金边 / 空=虚线框 / 光标=金色辉光外框；
    /// 触摸落槽时 [`SessionHost::set_saveload_cursor`]，键盘面无选位键（QB141 裁决））。
    fn saveload_cursor(&self) -> usize;

    /// QB136：置存读档槽光标（越界忽略，渲染钳到槽数内）。
    fn set_saveload_cursor(&mut self, slot: usize);

    /// P1-8（S1）：当前 RNG 已消费 `next_u64` 次数（写档捕获，读档精确续流）。
    fn rng_steps(&self) -> u128;

    /// S15.2（F23）：当前跨图推进解锁位掩码（bit 0..=2 = 各图已解锁，bit 3 = 通关；
    /// 编解码口径见 `rules::progression::ProgressionState::unlocked_mask`）。
    /// 写档时落进 `SaveSlot::unlocked_mask`，保住「击败 m01 解锁 m02、尚未切图」的进度。
    fn unlocked_mask(&self) -> u8;
}

/// 场景宿主能力接口：`GameApp` 实现的**唯一**状态/表现出口。
///
/// S9.2 起退化为 8 个子 trait 的 supertrait 聚合（空 trait，方法全在子 trait）。
/// `dyn SceneHost` 仍可访问全部方法（supertrait 机制自动展开）。
///
/// 场景 struct 经此 trait 读写状态与帧缓冲，**不接触** `GameApp` 任何字段
/// （`docs/process/01_architecture.md` §2）。每个方法都是一行读写，
/// 业务判定留在 `scenes/*.rs`。
pub trait SceneHost:
    NavHost + RenderHost + SpriteHost + MenuHost + SelectionHost + MapHost + EconomyHost + SessionHost
{
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

    /// S29.2：处理一次完成的拾取命中（[`HitData`]：`key`/`index` = 被点元素，`swipe` = 滑动方向；
    /// 默认不消费）。命中/几何全在 Bevy 拾取层，本方法只消费归一事件。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        let _ = (host, hit);
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

/// 路由：一次拾取命中交给 `kind` 对应场景的 [`Scene::on_hit`]（S29.2）。
#[must_use]
pub fn route_hit(host: &mut dyn SceneHost, kind: SceneKind, hit: &HitData) -> bool {
    match kind {
        SceneKind::Intro => {
            host.enter(SceneKind::Index);
            true
        }
        SceneKind::Exit => false,
        SceneKind::Index => IndexScene.on_hit(host, hit),
        SceneKind::SelectRole => SelectRoleScene.on_hit(host, hit),
        SceneKind::SelectMap => SelectMapScene.on_hit(host, hit),
        SceneKind::MainMap => MainMapScene.on_hit(host, hit),
        SceneKind::Vs => VsScene.on_hit(host, hit),
        SceneKind::Message => MessageScene.on_hit(host, hit),
        SceneKind::Saveload => SaveloadScene.on_hit(host, hit),
    }
}

/// 路由：绘制 `kind` 对应场景（`Exit` 清屏；QA120：清屏色走 `menu_shell::COLOR_BLACK`
/// 命名常量，`check_hardcode.py` 布局调用正则已含 `clear_frame`，内联 hex 即 FAIL）。
pub fn route_render(host: &mut dyn SceneHost, kind: SceneKind) {
    match kind {
        // 片头与主菜单共用同一张 `index.Map` + 同一套菜单绘制
        SceneKind::Intro | SceneKind::Index => IndexScene.render(host),
        SceneKind::Exit => host.clear_frame(crate::menu_shell::COLOR_BLACK),
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
/// 真正的路由已由 [`route_input`] / [`route_hit`] / [`route_render`] 承担；
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
