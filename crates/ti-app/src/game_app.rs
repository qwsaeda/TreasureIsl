//! 可玩交互核心：把 `ti-core::rules` 的 13 个已验证玩法模块接进表现层
//! （`docs/gamebook.md` §2/§3/§5 接线规格）。
//!
//! 可玩交互核心 = **组合根**（S9 骨架改造后本文件不再含场景业务）。
//!
//! 职责只剩三件：
//! 1. 持有全部跨场景状态（角色/地图/经济/战斗/消息/存档/表现层资源）；
//! 2. 把键位快照（`poll`）、触摸手势（`poll_touch`）、帧（`render`）
//!    **路由**到 `scenes/` 里对应的场景模块；
//! 3. 实现 [`SceneHost`]——一行状态读写 / 表现转发，供场景模块调用。
//!
//! 各场景的输入判定、触摸手势、绘制逻辑全部在 `scenes/*.rs`
//! （`docs/process/01_architecture.md` §2 分层边界）。
//!
//! 设计原则（`docs/gamebook.md` §7）：
//! - **不重抄数值**：一切判定/数值走 `ti-core::rules`，本层只编排状态与路由；
//! - **零占位**：不保留任何临时脚手架；
//! - **RNG 单一源**：战斗/事件随机全走 `rules::rng::GameRng`（可播种 PCG，`rand_pcg`，S8.7 落地；EXP-040 确定性）；
//! - **渲染复用**：场景绘制走 [`menu_shell`] 的 `Framebuf`/`FramePresenter`/字形机制
//!   （`docs/gamebook.md` §4），各场景分发到同一帧缓冲，脏行带机制不变（S4.3）。
//!
//! 与 [`MenuShell`] 的关系：`GameApp` 取代 `MenuShell` 成为两壳共用的交互核心；
//! `MenuShell` 退化为 `GameApp` 在 `Index` 场景下的渲染子视图（帧缓冲/字形/布局常量
//! 复用，见 [`menu_shell`]）。

use std::collections::HashMap;

use ti_core::data;
use ti_core::data::balance;
use ti_core::domain::combat::{Side, Unit};
use ti_core::domain::economy::{Bank, Building, PriceTable};
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{GridPos, MapTerrain, SceneTarget};
use ti_core::domain::messages::MsgKind;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot, SelectionState};
use ti_core::rules::battle::BattleSession;
use ti_core::rules::board::{BoardSession, BossTrigger, Landing, Track, TrackSpace, Turn};
use ti_core::rules::build;
use ti_core::rules::cards;
use ti_core::rules::cards_draw;
use ti_core::rules::main_loop::SceneStack;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::messages::total_pages;
use ti_core::rules::progression::{CLEARED_BIT, ProgressionState};
use ti_core::rules::rng;
use ti_core::rules::rng::GameRng;
use ti_core::rules::trade;

use crate::menu_shell::MenuShell;
use crate::scenes::{
    EconomyHost, MapHost, MenuHost, NavHost, RenderHost, SceneHost, SceneKind, SelectionHost,
    SessionHost, SpriteHost, route_input, route_render, route_touch,
};
use crate::sprite_atlas::SpriteAtlas;
use crate::touch::TouchSample;
use ti_platform::gfx::UploadMode;

/// 顶层场景（`docs/gamebook.md` §2，与 `main_loop::SceneId` 对齐）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    /// 片头（触发即跳 Index，可选）。
    Intro,
    /// 主菜单。
    Index,
    /// 角色选择。
    SelectRole,
    /// 地图选择。
    SelectMap,
    /// 主地图（移动/贸易/建造/事件分派）。
    MainMap,
    /// 战斗。
    Vs,
    /// 消息/帮助/信息（据点/面板/卡片）。
    Message,
    /// 存读档（子窗口叠加）。
    Saveload,
    /// 退出。
    Exit,
}

impl Scene {
    /// 场景键（0..=7，壳层静态帧优化用；切换即整帧重渲染，`docs/gamebook.md` §4）。
    #[must_use]
    pub const fn key(self) -> i32 {
        match self {
            Self::Index => 0,
            Self::SelectRole => 1,
            Self::SelectMap => 2,
            Self::MainMap => 3,
            Self::Vs => 4,
            Self::Message => 5,
            Self::Saveload => 6,
            Self::Exit | Self::Intro => 7,
        }
    }
}

/// 战斗会话封装（持 `rules::battle::BattleSession`，壳层只编排）。
#[derive(Debug, Clone)]
pub struct BattleView {
    /// 玩家单位（HP/atk，来自角色）。
    pub player: Unit,
    /// BOSS 档 tier（0..=4，`balance.combat.boss_tiers` 表）。
    pub boss_tier: u32,
    /// 战斗会话（`rules::battle::BattleSession`，数值全走 `balance().combat`）。
    pub session: BattleSession,
    /// 已战回合数（`step` 后同步，`BattleSession::round` 镜像）。
    pub rounds: u32,
    /// 玩家当前 HP（`step` 后同步，`BattleSession` 无只读访问器）。
    pub player_hp: u32,
    /// BOSS 当前 HP（`step` 后同步，`BattleSession::boss_hp` 镜像）。
    pub boss_hp: u32,
    /// 胜方（`step` 后同步，`None` = 未结束）。
    pub outcome: Option<Side>,
    /// 玩家初始 HP（血条分母，`Unit.hp`）。
    pub player_hp_max: u32,
    /// BOSS 初始 HP（血条分母，`BattleSession::boss_hp()` 构造时快照）。
    pub boss_hp_max: u32,
}

impl BattleView {
    /// 新建战斗视图（`BattleSession::from_balance`，数值走 `balance().combat`）。
    ///
    /// `balance()` 加载失败返回 `None`（硬错误，不 fallback，`02_conventions.md` §4.4）。
    #[must_use]
    pub fn new(seed: u32, player: Unit, boss_tier: u32) -> Option<Self> {
        let session = BattleSession::from_balance(seed, player, boss_tier).ok()?;
        let outcome = session.outcome();
        let boss_hp = session.boss_hp();
        Some(Self {
            player_hp_max: player.hp,
            boss_hp_max: boss_hp,
            player,
            boss_tier,
            session,
            rounds: 0,
            player_hp: player.hp,
            boss_hp,
            outcome,
        })
    }

    /// 是否结束。
    #[must_use]
    pub fn finished(&self) -> bool {
        self.outcome.is_some()
    }

    /// 胜方（`None` = 未结束，`outcome` 字段投影，测试用）。
    #[must_use]
    pub fn outcome(&self) -> Option<Side> {
        self.outcome
    }

    /// 玩家当前 HP（`player_hp` 字段投影，测试用）。
    #[must_use]
    pub fn player_hp(&self) -> u32 {
        self.player_hp
    }

    /// BOSS 当前 HP（`boss_hp` 字段投影，测试用）。
    #[must_use]
    pub fn boss_hp(&self) -> u32 {
        self.boss_hp
    }

    /// 单回合推进（`BattleSession::step`，确定性走 `balance().combat` + `GameRng`）。
    ///
    /// 已结束的会话不再推进（与 `BattleSession::step` 一致）。
    pub fn step(&mut self) {
        let (ph, eh, winner) = self.session.step();
        self.player_hp = ph;
        self.boss_hp = eh;
        self.rounds = self.session.round();
        self.outcome = winner;
    }
}

/// 战斗面板快照（渲染层只读视图，避免与帧缓冲借用重叠）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleSnapshot {
    /// 玩家当前 HP。
    pub player_hp: u32,
    /// BOSS 当前 HP。
    pub boss_hp: u32,
    /// 已战回合数。
    pub rounds: u32,
    /// 胜方（`None` = 未结束）。
    pub outcome: Option<Side>,
    /// 玩家初始 HP（血条分母，`balance.combat.player.hp`）。
    pub player_hp_max: u32,
    /// BOSS 初始 HP（血条分母，`balance.combat.boss_tiers[tier].hp`）。
    pub boss_hp_max: u32,
}

/// 归一输入事件（壳层负责把触摸/键/拖拽映射到这些变体，`docs/gamebook.md` §3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum GameInput {
    /// 菜单/选项上移（循环）。
    MoveUp,
    /// 菜单/选项下移（循环）。
    MoveDown,
    /// 确认（事件码 13，真实确定键 `spec/menu.md §a`）。
    Confirm,
    /// 返回（事件码 27，回上级场景）。
    Back,
    /// 上一页/上一个（事件码 38）。
    Prev,
    /// 下一页/下一个（事件码 40）。
    Next,
    /// 主地图：摇骰前进（沿环岛赛道前进 N 格，`docs/business/progression.md` §环岛赛道）。
    Spin,
    /// 主地图据点触发（`code` = 事件码 1..8，`spec/main_map.md §d`）。
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

/// 交互壳：把 `ti-core::rules` 的 13 个玩法模块接进表现层（`docs/gamebook.md` §2/§3）。
///
/// - `shell`：复用 [`MenuShell`] 的帧缓冲/字形/呈现机制（`Index` 场景渲染子视图）；
/// - `scene`：当前顶层场景；
/// - 各 rules 会话按需持有；
/// - 跨场景状态由顶层字段传递（`character`/`map_slot`/`gold`，`docs/gamebook.md` §7.3）。
///
/// 不走 `#[derive]`：`MenuShell`（帧缓冲/字形）与 `PriceTable` 无 `Debug`/`Clone`/`Default`
/// 派生；`GameApp` 由 [`GameApp::new`] 构造，`scenes`/`rng` 等规则态自带 `Default`。
///
/// `struct_excessive_bools` 不适用：组合根的跨场景布尔状态（`quit`/`press_held`/
/// `battle_won`）各属不同子系统，非「同类状态打包」场景。
#[allow(clippy::struct_excessive_bools)]
pub struct GameApp {
    /// 表现层（帧缓冲/字形/呈现，Index 场景渲染子视图）。
    pub shell: MenuShell,
    /// 绘制列表（S18.7a 记录模式目标：Bevy 路径）。
    pub draw_list: ti_render::drawlist::DrawList,
    /// 是否记录到绘制列表（true = Bevy 路径；false = CPU 帧缓冲）。
    pub record_draw: bool,
    /// 场景管理器（`rules::main_loop::SceneStack`，状态码语义与业务文档对齐）。
    pub scenes: SceneStack,
    /// 顶层场景。
    pub scene: Scene,
    /// 主菜单选中项（`Index` 场景）。
    pub menu_selected: usize,
    /// 已选角色（`None` = 未选）。
    pub character: Option<Character>,
    /// 已选地图（`None` = 未选）。
    pub map_slot: Option<MapSlot>,
    /// 当前地图的 BOSS 档（`MapSlot` → `balance.selection.maps[i].boss_tier`，`enter_main_map` 时写入）。
    pub current_boss_tier: u32,
    /// 已解锁地图清单（跨图推进，`business/progression.md`）：开档 =
    /// `ProgressionState::initial_unlocked()`（仅 m01），`battle_step` 胜利钩规则层推进，
    /// `restore_save` 按存档 `unlocked_mask` 忠实还原（旧档 mask=0 退最小前置链）。
    pub unlocked_maps: Vec<MapSlot>,
    /// 是否已通关（`business/progression.md` §通关：击败 m03 BOSS = 全图通杀）。
    /// 开档 = false；`battle_step` 在 m03 胜时置 true；`restore_save` 按存档掩码
    /// 的 CLEARED 位（bit 3）还原。解锁集本身不含通关标志（`from_unlocked` 口径），
    /// 故单列此字段。
    pub cleared: bool,
    /// 选择状态（`rules::selection` 行为镜像）。
    pub selection: SelectionState,
    /// 主地图会话（`rules::main_map` 行为镜像，`MainMap` 场景时有效）。
    pub map_terrain: Option<MapTerrain>,
    /// 当前玩家位置（`GridPos`，`MainMap` 场景时有效）。
    pub map_pos: Option<GridPos>,
    /// 据点坐标列表（`(row, col)`，`MainMap` 场景时有效）。
    pub map_landmarks: Vec<(u32, u32)>,
    /// 当前金（`rules::trade` 账本镜像，壳层拆分持有，`MainMap` 场景时有效）。
    pub gold: u32,
    /// 固定价格表（`domain::economy::PriceTable`，`rules::trade` 行为）。
    pub price_table: PriceTable,
    /// 持有量（`(goods_id, qty)`，`rules::trade` 行为）。
    pub owned: Vec<(u32, u32)>,
    /// 银行（`rules::build::Bank`，`MainMap` 场景时有效）。
    pub bank: Bank,
    /// 主地图会话（`rules::main_map::MapSession`，`MainMap` 场景时有效；
    /// 卡牌事件走 `balance.cards` + `GameRng`，可复现）。
    pub map_session: MapSession,
    /// 环岛赛道会话（`rules::board::BoardSession`，`MainMap` 场景时有效）。
    /// 摇骰 → 沿赛道前进 N 格 → 落格结算（`docs/business/progression.md` §环岛赛道）。
    pub board: Option<BoardSession>,
    /// 落格提示条文本（`Some` = 有提示待显示；随帧递减过期）。
    pub board_toast: Option<String>,
    /// 落格提示条剩余帧数。
    pub board_toast_ticks: u32,
    /// 建筑列表（`(kind, level)`，`rules::build` 行为镜像，`MainMap` 场景时有效）。
    pub buildings: Vec<(Building, u32)>,
    /// P1-7（M5）：卡牌效果状态（`rules::cards::CardState`，`MainMap` 场景时有效）。
    /// 初始 hp/gold = 0，`hp_max`/`gold_max` 钳制上限走 `balance.combat.player` / `balance.economy.gold_max`。
    pub card_state: cards::CardState,
    /// 战斗会话（`Vs` 场景时有效）。
    /// 初始值来自 `BattleSession::from_balance`（`balance.combat.player` + `boss_tier`）。
    pub battle: Option<BattleView>,
    /// 全局 RNG（可播种 PCG，`rand_pcg`，`rules::rng` 行为，EXP-040 确定性；S8.7 前为 MSVC LCG 历史参照）。
    pub rng: GameRng,
    /// 全局 RNG seed（供战斗/事件随机，壳层播种）。
    pub rng_seed: u32,
    /// 消息会话（`rules::messages` 行为镜像，`Message` 场景时有效）。
    pub msg_kind: Option<MsgKind>,
    /// 当前消息页码（0 起，`rules::messages` 行为）。
    pub msg_page: u32,
    /// 当前消息总页数（`rules::messages::total_pages`，`balance.messages.pages` 行为）。
    pub msg_total: u32,
    /// 存读档槽位（`rules::saveload` 行为，`Saveload` 场景时有效）。
    pub save_slots: Vec<SaveSlot>,
    /// QB136（S15.6）：存读档槽光标（三态渲染锚点；触摸落槽时 `set_saveload_cursor`，
    /// 默认 0 = 第 1 槽为初始候选；越界由渲染侧钳到槽数内）。
    pub saveload_cursor: usize,
    /// 音效（本次交互触发的音效，壳层播放）。
    pub last_sfx: Option<Sfx>,
    /// 本场战斗是否玩家胜（`battle_step` 结果落定时重算；`start_battle` 开新战斗时复位，避免陈旧态）。
    pub battle_won: bool,
    /// 退出标志（`Index` 场景菜单项 5 置位）。
    pub quit: bool,
    /// 主地图事件分派结果（最近一次 `dispatch`，壳层据此跳场景）。
    pub last_dispatch: Option<SceneTarget>,
    /// 精灵图集（`ui.pak` 的 120 个 PCX 贴图 + 解码缓存，`legacy/docs/design/ui_controls.md` §3）。
    /// 入口 bin 启动时注入，`render_scene` 据此 blit 真实贴图。
    pub atlas: Option<SpriteAtlas>,
    /// 已解析的 `.Map` 场景缓存（`scene_name → MapScene`，渲染时 blit 用）。
    pub scene_cache: HashMap<&'static str, ti_formats::mapfile::MapScene>,
    /// 地形图集（`glmap_m0*.pak` 瓦片 + 解码缓存，S6.3）。
    pub terrain_atlas: Option<crate::terrain_atlas::TerrainAtlas>,
    /// S6.4：本帧经过毫秒（壳层每帧传入，驱动精灵动画帧推进）。
    pub frame_delta_ms: u32,
    /// 已解析的地形（glmapm0*.Map，MainMap 场景渲染用）。
    pub terrain_map: Option<ti_formats::terrain::TerrainMap>,
    /// S10.4 GPU 渲染器（feature `gpu-render`；默认 headless no-op，
    /// 非 headless bin 经 `GpuRenderer::new` 注入真实路径）。
    #[cfg(feature = "gpu-render")]
    pub gpu_renderer: ti_platform::gpu_renderer::GpuRenderer,
    /// 场景切换过渡状态（None = 无过渡，正常渲染；EXP-079 淡入淡出）。
    pub transition: Option<TransitionState>,
    /// 确认输入当前是否按住（每帧由 `poll` 从 `KeyState.confirm` 快照，
    /// 驱动菜单项按下凹陷，EXP-079 §7.3）。
    pub press_held: bool,
    /// 是否已完成过至少一次场景渲染（首帧前无旧画面可混合，跳过过渡防黑闪，EXP-081）。
    pub rendered_once: bool,
    /// 显示用玩家 HP（平滑插值，每帧向实际值推进，EXP-085 HUD 平滑动画）。
    pub displayed_player_hp: u32,
    /// 显示用 BOSS HP（平滑插值）。
    pub displayed_boss_hp: u32,
    /// S18.5：主地图相机 zoom/pan 纯函数模型（`world_canvas` 镜头；Bevy 壳 GPU 侧接线，
    /// CPU terrain 相机接线挂 S18.6 像素基线重建同轮）。
    pub world_cam: crate::world_camera::WorldCam,
    /// S18.5：轻量确定性粒子（金币光/战斗命中火花；FX 专用 LCG，seed = [`FX_SEED`]）。
    pub fx: crate::particles::ParticleFx,
}

/// HP 平滑插值（S17.4 时间制）：向目标推进「剩余差值 × 本帧时间占比」，帧率无关；
/// 每次至少推进 1（防小差值卡死），`frac <= 0` 时保持不动。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn approach_ms(current: u32, target: u32, frac: f64) -> u32 {
    if current == target || frac <= 0.0 {
        return current;
    }
    let diff = target.abs_diff(current);
    let step = ((f64::from(diff) * frac).round().max(1.0) as u32).min(diff);
    if current < target {
        current.saturating_add(step).min(target)
    } else {
        current.saturating_sub(step).max(target)
    }
}

/// S17.4：过渡进度时间制推进（`MOTION_TRANSITION_MS` 内 delta 占比；f64 中间值 →
/// f32 末位截断不影响单调性）。delta 按 `u16` 上限封顶（壳层本就 clamp ≤250ms）。
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn advance_transition_t(t: f32, delta_ms: u32) -> f32 {
    let denom = f64::from(crate::scenes::chrome::MOTION_TRANSITION_MS.max(1));
    let step = f64::from(u16::try_from(delta_ms.max(1)).unwrap_or(u16::MAX)) / denom;
    t + step as f32
}

/// S17.9：交叉淡化缓动曲线（smoothstep ease-in-out，`10_ui_redesign` §动效「240ms
/// ease-in-out」定稿落地；旧 S17.4 为线性，观感生硬）。进度 `t∈[0,1]` → 混合因子
/// `t²(3−2t)`（端点不变：0→0、1→1，中段前慢后快再收尾；单调不递减，像素插值无回跳）。
/// S18.5：`pub` = Bevy 壳过渡同曲线复用（单一事实源，双渲染期两侧 ease 不漂移）。
#[must_use]
pub fn transition_ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// S18.5：FX 触发坐标常量（1080×1920 逻辑坐标；视觉验收可微调，S18.7 贴图管线接手版面）。
/// 战斗命中火花 = 战斗场景角色区中心；金币光 = HUD 金币显示区（`08_layout_design.md` HUD 行）。
pub const FX_BATTLE_HIT_POS: (f32, f32) = (540.0, 880.0);
/// S18.5：金币光触发坐标（HUD 金币显示区）。
pub const FX_COIN_GOLD_POS: (f32, f32) = (900.0, 140.0);
/// S18.5：FX 粒子 LCG 固定 seed（确定性视觉验收；不进业务 RNG `rules::rng::GameRng`）。
pub const FX_SEED: u64 = 0xC0_FF_EE;

/// 场景切换过渡状态机（EXP-081 → S17.4：真交叉淡化，`MOTION_TRANSITION_MS` 时间制推进）。
#[derive(Debug, Clone, Default)]
pub struct TransitionState {
    /// 旧画面快照（`enter()` 前抓的帧缓冲，`render_to_framebuf` / 快照路径为 `None`）。
    pub prev: Option<Box<[u16]>>,
    /// 当前插值进度（0.0 = 全旧画面，1.0 = 全新画面；按 ms 推进，S17.4）。
    pub t: f32,
}

/// 键位快照（壳层把触摸/键映射到这些布尔位，平台无关，`docs/gamebook.md` §3）。
///
/// 各壳在每帧从平台按键状态读取（Bevy 壳 = `ButtonInput<KeyCode>`），喂给 [`GameApp::poll`]。
///
/// 用位标志位映射键位状态（8 个独立键位，语义清晰）；
/// 非状态机，仅输入快照，故 `struct_excessive_bools` 不适用。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyState {
    /// 上 / 左。
    pub up: bool,
    /// 下 / 右。
    pub down: bool,
    /// 左。
    pub left: bool,
    /// 右。
    pub right: bool,
    /// 确认（Enter / Space / 鼠标左键）。
    pub confirm: bool,
    /// 返回（Backspace / Escape）。
    pub back: bool,
    /// 据点触发（E，主地图）。
    pub landmark: bool,
    /// 摇骰前进（Space，主地图）。
    pub spin: bool,
    /// 买（T）。
    pub buy: bool,
    /// 卖（Y）。
    pub sell: bool,
    /// 建造（B）。
    pub build: bool,
}

impl GameApp {
    /// 把按键快照映射为 `GameInput` 事件并逐条 `handle`（平台无关，`docs/gamebook.md` §3）。
    ///
    /// 返回是否发生场景切换（壳层据此强制整帧渲染）。
    /// 同一帧内多个键可同时命中（如 confirm + move），按场景语义合并。
    pub fn poll(&mut self, k: &KeyState) -> bool {
        // 按住态快照（菜单项按下凹陷，EXP-079 §7.3）：每帧从键位布尔取。
        self.press_held = k.confirm;
        let mut changed = false;
        match self.scene {
            Scene::Index => {
                if k.up {
                    changed = self.handle(GameInput::MoveUp) || changed;
                }
                if k.down {
                    changed = self.handle(GameInput::MoveDown) || changed;
                }
                if k.confirm {
                    changed = self.handle(GameInput::Confirm) || changed;
                }
            }
            Scene::SelectRole | Scene::SelectMap => {
                if k.up || k.left {
                    changed = self.handle(GameInput::Prev) || changed;
                }
                if k.down || k.right {
                    changed = self.handle(GameInput::Next) || changed;
                }
                if k.confirm {
                    changed = self.handle(GameInput::Confirm) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::Back) || changed;
                }
            }
            Scene::MainMap => {
                // 环岛赛道玩法（S19）：主地图不再 4 方向自由走位，
                // 一次操作 = 摇骰（Space）→ 沿赛道前进 N 格 → 落格结算。
                if k.spin {
                    changed = self.handle(GameInput::Spin) || changed;
                }
                if k.buy {
                    changed = self.handle(GameInput::TradeBuy(1)) || changed;
                }
                if k.sell {
                    changed = self.handle(GameInput::TradeSell(1)) || changed;
                }
                if k.build {
                    changed = self.handle(GameInput::Build(Building::Castle)) || changed;
                }
                if k.landmark {
                    changed = self.handle(GameInput::Landmark(1)) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::MapBack) || changed;
                }
            }
            Scene::Vs => {
                if k.confirm || k.landmark {
                    changed = self.handle(GameInput::Attack) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::Back) || changed;
                }
            }
            Scene::Message => {
                if k.down {
                    changed = self.handle(GameInput::MsgNext) || changed;
                }
                if k.up {
                    changed = self.handle(GameInput::MsgPrev) || changed;
                }
                if k.confirm || k.back {
                    changed = self.handle(GameInput::MsgClose) || changed;
                }
            }
            Scene::Saveload => {
                if k.confirm {
                    changed = self.handle(GameInput::WriteSave(0)) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::SaveloadClose) || changed;
                }
            }
            Scene::Intro | Scene::Exit => {
                if k.confirm {
                    self.scene = Scene::Index;
                    changed = true;
                }
            }
        }
        changed
    }

    /// 把一次完成的触摸手势映射为 `GameInput` 并 `handle`（`docs/modernization.md` §5）。
    ///
    /// **直接触摸界面元素，不画虚拟按键**：点菜单行 = 选中并执行；点地图格 =
    /// 朝该格走一步；点角色/地图/存档槽 = 选中该项；滑动 = 翻页/移动；下滑 = 返回。
    ///
    /// 返回是否发生场景切换（壳层据此强制整帧渲染）。`sample = None`（手势未完成
    /// 或触屏被禁用）时无副作用。
    pub fn poll_touch(&mut self, sample: Option<TouchSample>) -> bool {
        let Some(s) = sample else { return false };
        let kind = SceneKind::from_legacy(self.scene);
        route_touch(self, kind, &s)
    }

    /// 新建交互壳：默认 `Index` 场景，未选角色/地图，3 存档槽。
    ///
    /// 调用方随后填充 `shell.font` / `shell.wav_count` 等再调 `render`
    /// （与 [`MenuShell::new`] 同机制，`docs/gamebook.md` §4）。
    ///
    /// 默认走 [`UploadMode::Partial`]（wasm 路径）。native 桌面端用
    /// [`GameApp::with_upload_mode`] 选 [`UploadMode::FullRebuild`]（EXP-045）。
    #[must_use]
    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        Self::with_upload_mode(screen_w, screen_h, UploadMode::Partial)
    }

    /// 指定上传策略新建交互壳（native 桌面用 [`UploadMode::FullRebuild`]，EXP-045）。
    #[must_use]
    pub fn with_upload_mode(screen_w: f32, screen_h: f32, mode: UploadMode) -> Self {
        Self {
            shell: MenuShell::with_upload_mode(screen_w, screen_h, mode),
            // S18.7a：绘制列表目标（Bevy 路径；初始为 CPU 帧缓冲模式）。
            draw_list: ti_render::drawlist::DrawList::default(),
            record_draw: false,
            scenes: SceneStack::new(),
            scene: Scene::Index,
            menu_selected: 0,
            character: None,
            map_slot: None,
            current_boss_tier: 0,
            // 跨图推进（`docs/business/progression.md` §跨图推进）：新档仅 m01 解锁，
            // 击败 BOSS 才解锁下一图（`battle_step` 结算钩）；不走 `MapSlot::all()` 全开。
            unlocked_maps: ProgressionState::initial_unlocked(),
            cleared: false,
            selection: SelectionState::new(),
            map_terrain: None,
            map_pos: None,
            map_landmarks: Vec::new(),
            gold: 0,
            price_table: PriceTable::from_balance(),
            owned: Vec::new(),
            bank: Bank::default(),
            map_session: MapSession::new(MapTerrain::open(1, 1), GridPos::new(0, 0), &[]),
            board: None,
            board_toast: None,
            board_toast_ticks: 0,
            buildings: Vec::new(),
            card_state: cards::CardState::default(),
            battle: None,
            rng: GameRng::new(0),
            rng_seed: 0,
            msg_kind: None,
            msg_page: 0,
            msg_total: 0,
            save_slots: {
                let count = data::balance().map_or(5, |b| b.save.slot_count) as usize;
                vec![SaveSlot::default(); count]
            },
            saveload_cursor: 0,
            last_sfx: None,
            battle_won: false,
            quit: false,
            last_dispatch: None,
            atlas: None,
            scene_cache: HashMap::new(),
            terrain_atlas: None,
            frame_delta_ms: 0,
            terrain_map: None,
            transition: None,
            press_held: false,
            rendered_once: false,
            displayed_player_hp: 0,
            displayed_boss_hp: 0,
            world_cam: crate::world_camera::WorldCam::default(),
            fx: crate::particles::ParticleFx::deterministic(FX_SEED),
            #[cfg(feature = "gpu-render")]
            gpu_renderer: {
                // S18.8a：macroquad 发货壳退役后，仅 android 壳仍需真实 GPU API
                // （Bevy 壳走 ti-engine 自己的渲染器；诊断 bin 不绘制 GPU 帧）。
                #[cfg(any(feature = "native-shell", feature = "android-shell"))]
                {
                    ti_platform::gpu_renderer::GpuRenderer::new()
                }
                #[cfg(not(any(feature = "native-shell", feature = "android-shell")))]
                {
                    ti_platform::gpu_renderer::GpuRenderer::new_headless()
                }
            },
        }
    }

    /// 注入精灵图集（`ui.pak` 字节 → `SpriteAtlas`）。失败保持 `None`（渲染退回占位）。
    pub fn set_atlas(&mut self, pkg_bytes: &[u8]) {
        if let Ok(atlas) = SpriteAtlas::new(pkg_bytes) {
            self.atlas = Some(atlas);
        }
    }

    /// 注入已解析的 `.Map` 场景（`name → MapScene`，`render` 时 blit 精灵用）。
    ///
    /// 解析失败的场景跳过（不阻塞其他场景渲染）。
    pub fn add_scene(&mut self, name: &'static str, bytes: &[u8]) {
        if let Ok(scene) = ti_formats::mapfile::MapScene::parse(name, bytes) {
            self.scene_cache.insert(name, scene);
        }
    }

    /// 注入地形（瓦片 pak + 地形 Map），MainMap 场景据此 blit 真实地形（S6.3）。
    ///
    /// 解析失败保持 None（主地图退回占位网格，不黑屏）。
    pub fn set_terrain(&mut self, pak_bytes: &[u8], map_bytes: &[u8], map_name: &str) {
        if let Ok(atlas) = crate::terrain_atlas::TerrainAtlas::new(pak_bytes) {
            self.terrain_atlas = Some(atlas);
        }
        if let Ok(t) = ti_formats::terrain::TerrainMap::parse(map_name, map_bytes) {
            self.terrain_map = Some(t);
        }
    }

    /// 当前场景（只读）。
    #[must_use]
    pub fn scene(&self) -> Scene {
        self.scene
    }

    /// 是否退出。
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// 当前选中角色（只读）。
    #[must_use]
    pub fn character(&self) -> Option<Character> {
        self.character
    }

    /// 当前选中地图槽位（只读）。
    #[must_use]
    pub fn map_slot(&self) -> Option<MapSlot> {
        self.map_slot
    }

    /// 当前地图坐标（只读）。
    #[must_use]
    pub fn map_pos(&self) -> Option<GridPos> {
        self.map_pos
    }

    /// 主地图地形网格宽（v3 `chrome::board_fit` 的 G 依赖源；未载图缺省 20）。
    ///
    /// 暴露给测试/上层：`board_fit`/`iso_cell_fit` 正逆几何须与实际渲染同 G
    /// （`main_map.rs` render 用 `host.terrain_grid_w()`，S17.8）。
    #[must_use]
    pub fn map_grid_width(&self) -> u32 {
        self.terrain_grid_w()
    }

    /// 是否已通关（`is_cleared` 逐图门禁，`business/progression.md` §通关）：
    /// m03 的 BOSS 已击败 = 全图通杀（`battle_step` 在 m03 胜时置位，
    /// `restore_save` 按存档掩码 CLEARED 位（bit 3）还原）。
    #[must_use]
    pub fn is_cleared(&self) -> bool {
        self.cleared
    }

    /// 某图 BOSS 是否已击败（逐图门禁判据，`rules::progression`）：
    /// m01/m02 的击败位 = 下一图已解锁（`can_select` 口径），m03 的击败位 = 通关位。
    #[must_use]
    pub fn is_map_cleared(&self, m: MapSlot) -> bool {
        match m {
            MapSlot::M01 => self.unlocked_maps.contains(&MapSlot::M02),
            MapSlot::M02 => self.unlocked_maps.contains(&MapSlot::M03),
            MapSlot::M03 => self.cleared,
        }
    }

    /// BOSS 格坐标（`MapSession::boss_tile`，主地图渲染橙红标记 + 战斗触发用，只读）。
    #[must_use]
    pub fn map_boss_tile(&self) -> Option<GridPos> {
        self.map_session.boss_tile()
    }

    /// 当前金币数（只读）。
    #[must_use]
    pub fn gold(&self) -> u32 {
        self.gold
    }

    /// 已拥有物品列表（只读）。
    #[must_use]
    pub fn owned(&self) -> &[(u32, u32)] {
        &self.owned
    }

    /// 经济管理器（只读）。
    #[must_use]
    pub fn bank(&self) -> &Bank {
        &self.bank
    }

    /// 价格表（只读，`PriceTable` 来自 `domain::economy` 的域类型）。
    #[must_use]
    pub fn price_table(&self) -> &PriceTable {
        &self.price_table
    }

    /// GPU 渲染器只读访问器（feature `gpu-render`，P4-4 测试用）。
    #[cfg(feature = "gpu-render")]
    #[must_use]
    pub fn gpu_renderer(&self) -> &ti_platform::gpu_renderer::GpuRenderer {
        &self.gpu_renderer
    }

    /// GPU 渲染器可变访问器（feature `gpu-render`，blit 路径用）。
    #[cfg(feature = "gpu-render")]
    pub fn gpu_renderer_mut(&mut self) -> &mut ti_platform::gpu_renderer::GpuRenderer {
        &mut self.gpu_renderer
    }

    /// GPU 累计 draw call 数（feature `gpu-render`，P4-4 判据 2/3）。
    #[cfg(feature = "gpu-render")]
    #[must_use]
    pub fn gpu_draw_calls(&self) -> u64 {
        self.gpu_renderer.draw_call_count()
    }

    /// 已建造列表（只读）。
    #[must_use]
    pub fn buildings(&self) -> &[(Building, u32)] {
        &self.buildings
    }

    /// 全部已建建筑提供的战斗 atk 总加成（`Σ atk_bonus × level`，`docs/business/combat.md` §建造加成）。
    ///
    /// 成长机制入口：`start_battle` 用 `player.atk + build_atk_bonus()`。无建筑 / balance 加载失败 → 0。
    #[must_use]
    pub fn build_atk_bonus(&self) -> u32 {
        let Some(cfg) = data::balance().ok().map(|b| &b.build) else {
            return 0;
        };
        self.buildings.iter().map(|(k, lvl)| cfg.atk_bonus_at(build::kind_id(*k), *lvl)).sum()
    }

    /// 全部已建建筑提供的战斗 hp 总加成（`Σ hp_bonus × level`，同上）。
    ///
    /// 成长机制入口：`start_battle` 用 `player.hp + build_hp_bonus()`。无建筑 / balance 加载失败 → 0。
    #[must_use]
    pub fn build_hp_bonus(&self) -> u32 {
        let Some(cfg) = data::balance().ok().map(|b| &b.build) else {
            return 0;
        };
        self.buildings.iter().map(|(k, lvl)| cfg.hp_bonus_at(build::kind_id(*k), *lvl)).sum()
    }

    /// 主菜单选中项索引（只读）。
    #[must_use]
    pub fn menu_selected(&self) -> usize {
        self.menu_selected
    }

    /// 消息页码（只读）。
    #[must_use]
    pub fn msg_page(&self) -> u32 {
        self.msg_page
    }

    /// 存档槽位列表（只读）。
    #[must_use]
    pub fn save_slots(&self) -> &[SaveSlot] {
        &self.save_slots
    }

    /// 写入存档槽（越界忽略，测试 / 存档往返用）。
    pub fn write_save_slot(&mut self, slot: usize, data: SaveSlot) {
        if let Some(dst) = self.save_slots.get_mut(slot) {
            *dst = data;
        }
    }

    /// 按存档槽恢复状态并跳场景（测试 / 存档往返用，与 `SceneHost::restore_save` 同逻辑）。
    ///
    /// FIND-107：角色/地图边界改读 `balance.selection.characters/maps` 目录长度，
    /// 不再硬编码 5/3（调目录只需改 balance.json）。
    ///
    /// FIND-204：字段写入前先快照旧值，`enter_main_map` 校验失败（返回 `false`）时
    /// 回滚 7 个字段（`character`/`map_slot`/`selection`/`gold`/`bank`/`unlocked_maps`/`rng`），不留下半初始化态；成功才提交。
    ///
    /// 跨图推进（`business/progression.md`）：`SaveSlot` 无解锁链字段（schema 稳定），
    /// 读档按存档 `map_slot` 的「最小前置链」恢复解锁（选中 m(i) ⟹ m(0)..m(i-1) 已击败）。
    ///
    /// Q007（第 12 轮审计）：读档成功路径按存档恢复 `gold`/位置（`enter_main_map_at` 会用
    /// 初始值重铺经济/位置，成功后须再覆写存档值）；`SaveSlot` 无 `owned`/`buildings` 字段，
    /// 持有/建造不可恢复（schema 限制，`persistence.md` 明示）。
    pub fn restore_save(&mut self, data: SaveSlot) {
        let bounds = balance()
            .map_or((0usize, 0usize), |b| (b.selection.characters.len(), b.selection.maps.len()));
        let (char_count, map_count) = bounds;
        // FIND-204：快照调用前 7 字段，enter_main_map 失败时回滚
        let prev_character = self.character;
        let prev_map_slot = self.map_slot;
        let prev_selection = self.selection;
        let prev_gold = self.gold;
        let prev_bank = self.bank;
        let prev_unlocked_maps = self.unlocked_maps.clone();
        let prev_rng = self.rng.clone();
        let char_idx = data.character.saturating_sub(1);
        if usize::try_from(char_idx).is_ok_and(|i| i < char_count) {
            if let Some(ch) = Character::from_index(char_idx) {
                self.character = Some(ch);
                self.selection.character = Some(ch);
            }
        }
        let mc_u32 = u32::try_from(map_count).unwrap_or(0);
        if data.map_slot < mc_u32 {
            if let Some(ms) = MapSlot::from_index(data.map_slot) {
                self.map_slot = Some(ms);
                self.selection.map_slot = Some(ms);
                // 读档解锁链恢复（跨图推进，S15.2/F23）：优先用 `unlocked_mask` 忠实还原
                // （保住「击败 m01 解锁 m02、尚未切图」的进度）；`mask==0`（旧档无字段）
                // 退回「选中 m(i) 的最小前置链」，保证旧档仍可回读。
                self.unlocked_maps =
                    ProgressionState::for_save_restore(data.unlocked_mask, ms).unlocked();
            }
        }
        // 通关位还原（S15.2 通关门禁）：掩码 CLEARED 位（bit 3）= m03 BOSS 已击败。
        // 旧档无该字段 → mask 全 0 → cleared 保持开档 false，行为不回归。
        self.cleared = data.unlocked_mask & CLEARED_BIT != 0;
        self.gold = data.gold;
        self.bank = Bank::new(data.gold);
        // P1-8（S1）：RNG 精确续流——读档后 `restore_state(rng_seed, rng_steps)` O(log n) 跳转，
        // 后续卡牌/地图随机流与"未存档"一致（`persistence.md` §RNG 可复现性）。
        self.rng = rng::GameRng::restore_state(self.rng_seed, data.rng_steps);
        // Q007：读档位置恢复——`data.position` = 行主序格索引（写端 `map_pos().index(grid_w)`，
        // `0` = 未写位置）。越界 / 无该图定义 → 回退 `player_start`（不拦读档）。
        let start_override = balance().ok().and_then(|b| {
            let def = b.selection.maps.iter().find(|m| m.index == data.map_slot)?;
            if data.position == 0 {
                return None;
            }
            let (row, col) = (data.position / def.grid_w.max(1), data.position % def.grid_w.max(1));
            if row < def.grid_h && col < def.grid_w { Some(GridPos::new(row, col)) } else { None }
        });
        if self.selection.has_character() && self.selection.has_map() {
            let ok = GameApp::enter_main_map_at(self, start_override);
            if !ok {
                // FIND-204：校验失败回滚 7 字段 + selection，跳到 SelectRole（不污染状态）
                self.character = prev_character;
                self.map_slot = prev_map_slot;
                self.selection = prev_selection;
                self.gold = prev_gold;
                self.bank = prev_bank;
                self.unlocked_maps = prev_unlocked_maps;
                self.rng = prev_rng;
                self.scene = Scene::SelectRole;
                return;
            }
            // Q007：`enter_main_map_at` 按初始值重铺了经济/位置，成功后再覆写存档值
            //（`owned`/`buildings` 不在 `SaveSlot` schema 内，保持初始空——schema 限制，`persistence.md`）。
            self.gold = data.gold;
            self.bank = Bank::new(data.gold);
            self.selection.completed = true;
        } else {
            self.scene = Scene::SelectRole;
        }
    }

    /// 战斗会话（只读）。
    #[must_use]
    pub fn battle(&self) -> Option<&BattleView> {
        self.battle.as_ref()
    }

    /// 取可变 RNG 句柄（测试用）。
    #[must_use]
    pub fn rng_mut(&mut self) -> &mut GameRng {
        &mut self.rng
    }

    /// 设置 RNG 种子（测试用）。
    /// S18.7a：切换绘制目标模式。
    ///
    /// - `true` = **记录模式**：绘制命令进 `绘制列表`（Bevy 路径；实体由 `ti-engine` 转换）；
    /// - `false` = CPU 帧缓冲模式（诊断 bin 路径；macroquad 发货壳 S18.8a 已退役）。
    ///
    /// 切换时同步度量口径（主题 + 三档 TTF）→ 两路径版面完全一致。
    pub fn set_record_draw(&mut self, on: bool) {
        if on && !self.record_draw {
            self.draw_list.theme = self.shell.theme.clone();
            self.draw_list.ttf = ti_render::drawlist::TtfSet {
                body: self.shell.ttf_font.clone(),
                title: self.shell.ttf_title.clone(),
                hint: self.shell.ttf_hint.clone(),
            };
            self.draw_list.clear();
        }
        self.record_draw = on;
    }

    /// S18.7a：取本帧绘制列表（记录模式下有效；CPU 模式下为空）。
    #[must_use]
    pub fn draw_list(&self) -> &ti_render::drawlist::DrawList {
        &self.draw_list
    }

    /// 设置随机种子（测试可复现）。
    pub fn set_rng_seed(&mut self, seed: u32) {
        self.rng_seed = seed;
        self.rng.set_seed(seed);
    }

    /// 取渲染壳（只读）。
    #[must_use]
    pub fn shell(&self) -> &MenuShell {
        &self.shell
    }

    /// 取可变渲染壳（壳层对 `MenuShell` 的可变操作，如注入字体/音频、切缩放模式、`present`）。
    ///
    /// `shell()` 给 `&MenuShell` 无法驱动 `MenuShell` 的 `&mut` 方法，bins 统一走本访问器。
    pub fn shell_mut(&mut self) -> &mut MenuShell {
        &mut self.shell
    }

    /// 音效访问器（`last_sfx` 字段的 `&mut` 视图，壳层交互后读/清）。
    pub fn last_sfx(&mut self) -> &mut Option<Sfx> {
        &mut self.last_sfx
    }

    /// 清空本帧音效（`last_sfx = None`，壳层交互后调用，避免同一音效连播）。
    pub fn clear_last_sfx(&mut self) {
        self.last_sfx = None;
    }

    /// 设置本帧经过毫秒（S6.4，驱动多帧精灵动画推进）。
    pub fn set_frame_delta_ms(&mut self, ms: u32) {
        self.frame_delta_ms = ms;
    }

    /// 取可变精灵图集（测试用）。
    #[must_use]
    pub fn atlas_mut(&mut self) -> Option<&mut SpriteAtlas> {
        self.atlas.as_mut()
    }

    /// S7.5（feature `hd-8bit`）：HD 渲染路径——复用既有 RGB565 [`Self::render`] 派发，
    /// 再把 RGB565 帧缓冲扩展成 RGBA8（`sync_rgba_from_rgb565`），让 HD 帧就绪。
    ///
    /// S15.4：legacy 逐像素 HD PNG 精灵叠加（`blit_scene_rgba`）已移除；HD 帧 = 纯代码
    /// 绘制 UI 的 RGBA8 扩色版（`08_render_arch.md` 契约 2）。
    ///
    /// 调用方随后调 [`Self::present_hd`] 上屏。
    #[cfg(feature = "hd-8bit")]
    pub fn render_hd(&mut self) {
        self.render();
        self.shell.sync_rgba_from_rgb565();
    }

    /// S7.5（feature `hd-8bit`）：RGBA8 HD 帧缓冲上屏（同 `present` 上传策略，真彩无 565 量化）。
    #[cfg(feature = "hd-8bit")]
    pub fn present_hd(&mut self) {
        self.shell.present_rgba();
    }

    /// S7.5（feature `hd-8bit`）：HD RGBA8 缓存条目数（诊断用，未注入时 0）。
    #[cfg(feature = "hd-8bit")]
    #[must_use]
    pub fn atlas_hd_cache_size(&self) -> usize {
        self.atlas.as_ref().map_or(0, SpriteAtlas::hd_cache_size)
    }

    /// S7.5（feature `hd-8bit`）：注入 HD PNG（`f1 → 多帧 RGBA`），未注入时跳过（PCX 回退）。
    #[cfg(feature = "hd-8bit")]
    pub fn load_hd_pngs(&mut self, f1_to_rgba: impl IntoIterator<Item = (u32, Vec<Vec<u8>>)>) {
        if let Some(atlas) = self.atlas.as_mut() {
            atlas.load_hd_pngs(f1_to_rgba);
        }
    }

    /// S7.5（feature `hd-8bit`）：从 `assets_hd_pack/1x/` 目录注入 HD PNG。
    ///
    /// 文件命名：静态精灵 `NN_hd.png`（1 帧）；多帧 `NN_hd_f0.png`~`f4.png`。
    /// 按帧号排序组成 `f1 → 帧列表`，喂 [`Self::load_hd_pngs`]。目录不存在或
    /// 无可解码 PNG 时返回 `false`（PCX 回退，不 panic）。
    /// 仅 native 宿主可用（依赖 `std::fs` 目录枚举）；wasm32 目标不编译。
    #[cfg(all(feature = "hd-8bit", not(target_arch = "wasm32")))]
    pub fn load_hd_pack_from_dir(&mut self, dir: &std::path::Path) -> bool {
        // S10.2：递归 DFS 收集子目录 PNG（assets_hd_pack/1x/{ui,glmap_m01,glmap_m02,
        // glpics,cards}/...）。不引入 walkdir（std 手动递归）。
        fn collect_pngs_recursive(
            root: &std::path::Path,
            png_map: &mut std::collections::BTreeMap<u32, Vec<(u32, Vec<u8>)>>,
        ) {
            let Ok(entries) = std::fs::read_dir(root) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_pngs_recursive(&path, png_map);
                    continue;
                }
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
                {
                    continue;
                }
                let stem = name.strip_suffix(".png").unwrap_or(name);
                // 静态："NN_hd" → f1=NN, frame=0；多帧："NN_hd_fK" → f1=NN, frame=K
                let (f1_str, frame_str) = match stem.rsplit_once("_f") {
                    Some((a, b)) => (a.strip_suffix("_hd").unwrap_or(a), b),
                    None => (stem.strip_suffix("_hd").unwrap_or(stem), "0"),
                };
                let Ok(f1) = f1_str.parse::<u32>() else {
                    continue;
                };
                let frame_no: u32 = frame_str.parse().unwrap_or(0);
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                    if img.rgba.iter().any(|&b| b > 0) {
                        png_map.entry(f1).or_default().push((frame_no, bytes));
                    }
                }
            }
        }
        let mut png_map: std::collections::BTreeMap<u32, Vec<(u32, Vec<u8>)>> =
            std::collections::BTreeMap::new();
        collect_pngs_recursive(dir, &mut png_map);
        if png_map.is_empty() {
            return false;
        }
        let png_bytes: Vec<(u32, Vec<Vec<u8>>)> = png_map
            .into_iter()
            .map(|(f1, mut frames)| {
                frames.sort_by_key(|(n, _)| *n);
                (f1, frames.into_iter().map(|(_, b)| b).collect())
            })
            .collect();
        self.load_hd_pngs(png_bytes.clone());
        // P2-8：同时喂 terrain_atlas（地形 HD 接线，`assets_hd_pack/1x/` 同档）
        #[cfg(feature = "hd-8bit")]
        if let Some(ta) = self.terrain_atlas.as_mut() {
            let terrain_png: Vec<(u32, Vec<u8>)> = png_bytes
                .iter()
                .filter_map(|(f1, frames)| frames.first().map(|b| (*f1, b.clone())))
                .collect();
            ta.load_hd_pngs(terrain_png);
        }
        true
    }

    /// N7（EXP-048/060）：manifest 驱动的 HD PNG 注入核心逻辑（同步，宿主传 fetch 闭包）。
    ///
    /// 与 native 的 [`Self::load_hd_pack_from_dir`] 同源：注入数 = manifest 全量
    /// （`static_f1` + `multiframe_f1`），不硬编码 f1 数组。
    /// `static_f1` → `NN_hd.png`（1 帧）；`multiframe_f1` → `NN_hd_f0..f4.png`（5 帧）。
    /// 返回实际注入的 `(f1, 帧列表)`（与 `terrain_atlas` 同档喂入）；调用方按
    /// `static_f1 + multiframe_f1` 断言全量（EXP-048/060）。
    #[cfg(feature = "hd-8bit")]
    pub fn load_hd_pngs_from_manifest(
        &mut self,
        static_f1: &[u32],
        multiframe_f1: &[u32],
        fetch: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Vec<(u32, Vec<Vec<u8>>)> {
        let mut mapping: Vec<(u32, Vec<Vec<u8>>)> = Vec::new();
        for f1 in static_f1.iter().chain(multiframe_f1.iter()) {
            let f1 = *f1;
            let mut frames: Vec<Vec<u8>> = Vec::new();
            if multiframe_f1.contains(&f1) {
                for k in 0..5u32 {
                    let name = format!("assets_hd_pack/1x/ui/{f1}_hd_f{k}.png");
                    if let Some(bytes) = fetch(&name) {
                        if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                            if img.rgba.iter().any(|&b| b > 0) {
                                frames.push(bytes);
                            }
                        }
                    }
                }
            } else {
                let name = format!("assets_hd_pack/1x/ui/{f1}_hd.png");
                if let Some(bytes) = fetch(&name) {
                    if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                        if img.rgba.iter().any(|&b| b > 0) {
                            frames.push(bytes);
                        }
                    }
                }
            }
            if !frames.is_empty() {
                mapping.push((f1, frames));
            }
        }
        if !mapping.is_empty() {
            self.load_hd_pngs(mapping.clone());
            if let Some(ta) = self.terrain_atlas.as_mut() {
                let terrain_png: Vec<(u32, Vec<u8>)> = mapping
                    .iter()
                    .filter_map(|(f1, frames)| frames.first().map(|b| (*f1, b.clone())))
                    .collect();
                ta.load_hd_pngs(terrain_png);
            }
        }
        mapping
    }

    /// 处理输入事件：路由到 `scenes/` 里对应场景的 [`crate::scenes::Scene::on_input`]。
    ///
    /// 调用方随后调 `render` 刷新帧缓冲。返回是否发生了场景切换。
    pub fn handle(&mut self, input: GameInput) -> bool {
        let kind = SceneKind::from_legacy(self.scene);
        let gold_before = self.gold;
        let consumed = route_input(self, kind, input);
        // S18.5 FX 触发（纯视觉层：不改业务判定；攻击回合 / 金币变动才 spawn）。
        if consumed {
            match input {
                GameInput::Attack => {
                    self.fx.spawn_hit_spark(FX_BATTLE_HIT_POS.0, FX_BATTLE_HIT_POS.1);
                }
                GameInput::TradeBuy(_) | GameInput::TradeSell(_) if self.gold != gold_before => {
                    self.fx.spawn_coin_glint(FX_COIN_GOLD_POS.0, FX_COIN_GOLD_POS.1);
                }
                _ => {}
            }
        }
        consumed
    }

    /// S9.5 Command 派发入口：把 [`crate::commands::GameCommand`] 路由到当前场景。
    ///
    /// 与 [`Self::handle`] 等价（`GameCommand::to_input` 后走 `route_input`），
    /// 但语义层统一为 Command 模式，供命令派发枚举调用。
    /// 返回是否消费了该输入（发生了场景切换）。
    pub fn dispatch(&mut self, cmd: crate::commands::GameCommand) -> bool {
        let kind = SceneKind::from_legacy(self.scene);
        let input = GameInput::from(cmd);
        route_input(self, kind, input)
    }

    /// S18.5：相机平移（壳层把鼠标/触摸拖拽的屏幕 px 步长喂入；世界步 = 屏步 / zoom）。
    pub fn camera_pan(&mut self, step_x: f32, step_y: f32) {
        self.world_cam.pan(step_x, step_y);
    }

    /// S18.5：以 view 空间锚点为中心缩放（滚轮喂入；zoom 钳制 [`crate::world_camera::clamp_zoom`]）。
    pub fn camera_zoom_at(&mut self, anchor_view: [f32; 2], factor: f32, view: [f32; 2]) {
        self.world_cam.zoom_at(anchor_view, factor, view);
    }

    /// S18.5：相机只读视图（Bevy 壳 GPU 侧 zoom/pan 接线）。
    #[must_use]
    pub fn world_cam(&self) -> &crate::world_camera::WorldCam {
        &self.world_cam
    }

    /// S18.5：FX 只读视图（确定性自检 / 视觉验收）。
    #[must_use]
    pub fn fx(&self) -> &crate::particles::ParticleFx {
        &self.fx
    }

    /// 进入主地图（`docs/gamebook.md` §3.3）：初始化 `MapSession` + 贸易/建造会话。
    ///
    /// `start_override` = 入口格：`None` = 该图 `player_start`；`Some(pos)` = 读档位置恢复
    /// （Q007 第 12 轮审计）；越界 → 校验失败返回 `false`，调用方回滚。
    ///
    /// 返回 `false` = 数据校验失败（`balance.json` 加载失败 / `selection.maps` 缺条目 /
    /// `boss_tiers` 缺档 / `player_start` 或 `landmarks` 坐标越界），此时本方法自身的
    /// 字段写入未发生（校验在写入前），但调用方 `restore_save` 已先写入
    /// `character`/`map_slot`/`gold`/`bank`，须据返回值回滚（FIND-204 半初始化态消除）。
    /// 返回 `true` = 成功进入主地图（`scene` 已切 `MainMap`）。
    fn enter_main_map_at(&mut self, start_override: Option<GridPos>) -> bool {
        let b = match data::balance() {
            Ok(b) => b,
            Err(_e) => {
                #[cfg(feature = "stats")]
                eprintln!("[stats] enter_main_map: balance.json 加载失败（硬错误，不 fallback）");
                return false;
            }
        };
        let init_gold = b.economy.initial_gold;
        let zero_index = self.map_slot.map_or(0, MapSlot::zero_index);
        let Some(map_def) = b.selection.maps.iter().find(|m| m.index == zero_index) else {
            #[cfg(feature = "stats")]
            eprintln!(
                "[stats] enter_main_map: balance.selection.maps 缺 index={zero_index} 条目（硬错误，不 fallback）"
            );
            return false;
        };
        let gw = map_def.grid_w;
        let gh = map_def.grid_h;
        let boss_tier = map_def.boss_tier;
        // FIND-103：boss_tier 越界校验（与 start_battle combat.player 缺失同口径）
        if b.combat.boss(boss_tier).is_none() {
            eprintln!(
                "enter_main_map: balance.combat.boss_tiers 缺 tier={boss_tier}（硬错误，不 fallback）"
            );
            return false;
        }
        let start: [u32; 2] = match start_override {
            Some(pos) => [pos.row, pos.col],
            None => map_def.player_start,
        };
        // FIND-105：player_start 坐标越界校验
        if start[0] >= gh || start[1] >= gw {
            eprintln!(
                "enter_main_map: player_start [{} {}] 越界（grid {gw}×{gh}，硬错误，不 fallback）",
                start[0], start[1]
            );
            return false;
        }
        let landmarks: Vec<(u32, u32)> =
            map_def.landmarks.iter().map(|pair| (pair[0], pair[1])).collect();
        // FIND-105：landmarks 坐标越界校验
        for &(lr, lc) in &landmarks {
            if lr >= gh || lc >= gw {
                eprintln!(
                    "enter_main_map: landmark [{lr} {lc}] 越界（grid {gw}×{gh}，硬错误，不 fallback）"
                );
                return false;
            }
        }
        // P0-2（M1）：BOSS 格真接入可玩路径——`MapSession::with_boss` 注入 `boss_tile`，
        // 玩家 `move_to` 落到 BOSS 格时 `StepResult::boss=true`（`progression.md` 落格结算）。
        let boss_tile = map_def.boss_tile.as_ref().map(|pt| GridPos::new(pt[0], pt[1]));
        self.current_boss_tier = boss_tier;
        self.map_terrain = Some(MapTerrain::open(gw, gh));
        self.map_pos = Some(GridPos::new(start[0], start[1]));
        self.map_landmarks = landmarks;
        // 主地图会话（`rules::main_map::MapSession::with_boss`，卡牌池走 `balance.cards` + BOSS 格真触发）
        let pool = cards_draw::CardPool::from_balance();
        self.map_session = MapSession::with_boss(
            MapTerrain::open(gw, gh),
            GridPos::new(start[0], start[1]),
            &self.map_landmarks,
            pool,
            boss_tile,
        );
        // 环岛赛道（`rules::board`）：赛道数据来自 `assets/data/board.json`
        // （`scripts/extract_board.py` 生成）；起点 = 赛道 `start` 格；
        // 读档位置（`start_override`）若不在赛道上 → 就近吸附到最近赛道格。
        self.board = build_board_session(&map_def.id, start_override);
        // 玩家位置 = 当前赛道格（S19 环岛赛道）：无赛道时回退 balance `player_start`。
        self.map_pos =
            self.board.as_ref().map(BoardSession::pos).or(Some(GridPos::new(start[0], start[1])));
        self.gold = init_gold;
        self.owned = Vec::new();
        self.bank = Bank::new(init_gold);
        self.buildings = Vec::new();
        // 选择完成标志
        self.selection.completed = self.selection.has_character() && self.selection.has_map();
        self.scene = Scene::MainMap;
        // 场景管理器状态码同步（`rules::main_loop` 行为）
        self.scenes.set_state(0);
        true
    }

    /// 进入主地图（入口格 = 该图 `player_start`；选图 Confirm 路径，`start_override` = `None`）。
    fn enter_main_map(&mut self) -> bool {
        self.enter_main_map_at(None)
    }

    /// 摇骰前进（环岛赛道，`docs/business/progression.md` §环岛赛道）：
    /// 摇骰定步数（`1..=dice_sides`）→ 沿赛道前进 N 格 → 落格结算。
    ///
    /// 结算（数值全走 `balance.board`，规则层 `rules::board::Turn`/`Landing` 分派）：
    /// - 绕岛过起点 → 每圈 `lap_bonus_gold`；
    /// - 港口 → 提示条（贸易/建造经 HUD 命令盘）；城镇 → `city_gold`；
    /// - 事件/卡牌格 → 抽一张卡经 `apply_card_effect` 结算；
    /// - 遭遇格 → 扣 `battle_damage` HP + 失 `battle_gold_loss` 金；
    /// - 宝藏格 → `treasure_gold`；
    /// - BOSS（S19.3）：**经过或落在 BOSS 格**即遇 BOSS；绕岛满 `boss_min_laps` 圈
    ///   才现身（`Turn::boss = Engaged` → 调用方进战斗场景），未满圈 = 未现
    ///   （`Locked` → 提示「还需 N 圈」）；现身时位置停到 BOSS 格（被拦停）。
    ///
    /// 返回本回合结果（`None` = 无赛道，`board` 未构建）。
    pub fn spin_board(&mut self) -> Option<Turn> {
        let b = match balance() {
            Ok(b) => b,
            Err(_e) => {
                #[cfg(feature = "stats")]
                eprintln!("[stats] spin_board: balance.json 加载失败（硬错误，不 fallback）");
                return None;
            }
        };
        let session = self.board.as_mut()?;
        let sides = b.board.dice_sides.min(b.board.max_steps);
        let laps_before = session.laps();
        let turn = session.step(&mut self.rng, sides, b.board.boss_min_laps);
        let laps_after = session.laps();
        self.map_pos = Some(session.pos());
        // 绕圈补给（大富翁「经过起点」语义）
        let lap_delta = laps_after.saturating_sub(laps_before);
        if lap_delta > 0 {
            let bonus = b.board.lap_bonus_gold.saturating_mul(lap_delta);
            self.gold = self.gold.saturating_add(bonus).min(b.economy.gold_max);
        }
        let landing = session.landing();
        match landing {
            Landing::Battle => {
                self.card_state.hp = self.card_state.hp.saturating_add(b.board.battle_damage);
                self.gold = self.gold.saturating_sub(b.board.battle_gold_loss);
                self.board_toast = Some(crate::i18n::sgetf(
                    "scene.main_map.toast.battle_fmt",
                    &[&b.board.battle_damage],
                ));
                self.board_toast_ticks = 120;
                self.last_sfx = Some(Sfx::Invalid);
            }
            Landing::Treasure => {
                self.gold = self.gold.saturating_add(b.board.treasure_gold).min(b.economy.gold_max);
                self.board_toast = Some(crate::i18n::sgetf(
                    "scene.main_map.toast.treasure_fmt",
                    &[&b.board.treasure_gold],
                ));
                self.board_toast_ticks = 120;
                self.last_sfx = Some(Sfx::Press);
            }
            Landing::City => {
                self.gold = self.gold.saturating_add(b.board.city_gold).min(b.economy.gold_max);
                self.board_toast = Some(crate::i18n::sgetf(
                    "scene.main_map.toast.city_fmt",
                    &[&b.board.city_gold],
                ));
                self.board_toast_ticks = 120;
                self.last_sfx = Some(Sfx::Press);
            }
            Landing::Port => {
                self.board_toast =
                    Some(String::from(crate::i18n::sget("scene.main_map.toast.port")));
                self.board_toast_ticks = 120;
                self.last_sfx = Some(Sfx::Press);
            }
            Landing::Event | Landing::Card => {
                let pool = cards_draw::CardPool::from_balance();
                let hand = pool.draw_one(&mut self.rng, b.cards.hand_size.max(1));
                if let Some(card_id) = hand.first().copied() {
                    let name_key = b
                        .cards
                        .deck
                        .iter()
                        .find(|c| c.id == card_id)
                        .map(|c| c.name_key.clone())
                        .unwrap_or_default();
                    if self.apply_card_effect(card_id) {
                        let card_name = crate::i18n::sget(&name_key);
                        self.board_toast = Some(crate::i18n::sgetf(
                            "scene.main_map.toast.card_fmt",
                            &[&card_name],
                        ));
                        self.board_toast_ticks = 120;
                        self.last_sfx = Some(Sfx::Press);
                    }
                }
            }
            // BOSS 格本身无金/HP 结算（提示条由下方 `Turn::boss` 分支给出：
            // 未满圈 = 未现 / 满圈 = 现身）；起点格与普通格同样无结算。
            Landing::Boss | Landing::Start | Landing::Plain => {}
        }
        // BOSS 触发（S19.3）：经过或落在 BOSS 格即遇 BOSS；满 `boss_min_laps` 圈才现身。
        match turn.boss {
            BossTrigger::None => {}
            BossTrigger::Locked => {
                let remain = b.board.boss_min_laps.saturating_sub(laps_after).max(1);
                self.board_toast =
                    Some(crate::i18n::sgetf("scene.main_map.toast.boss_locked_fmt", &[&remain]));
                self.board_toast_ticks = 120;
                self.last_sfx = Some(Sfx::Invalid);
            }
            BossTrigger::Engaged => {
                // 位置已在结算前停到 BOSS 格（见上），此处只给提示条
                self.board_toast =
                    Some(String::from(crate::i18n::sget("scene.main_map.toast.boss")));
                self.board_toast_ticks = 120;
            }
        }
        Some(turn)
    }

    /// 渲染当前场景：场景模块绘制（`scenes/*.rs::render`），纯代码绘制
    /// （`08_render_arch.md` 契约 2：S15.4 起无 legacy 精灵层叠加）。
    ///
    /// 调用方随后调 [`MenuShell::present`] 上屏。
    pub fn render(&mut self) {
        // S18.7a：记录模式每帧先清空绘制列表（否则命令逐帧堆积 →
        // 实体 z 越叠越负 → 超出相机 far 平面 被裁切）。
        if self.record_draw {
            self.draw_list.clear();
        }
        // S17.4：动效推进全部时间制（本帧经过毫秒）——帧率无关（旧按帧计数）。
        crate::scenes::chrome::tick_press_flash(self.frame_delta_ms);
        // S18.5：FX 粒子推进（确定性：同 seed + 同 delta 序列 = 同画面；快照路径先 clear）。
        self.fx.advance(self.frame_delta_ms);
        // HP 条平滑插值（EXP-085）：每帧向实际值推进，步长 = 差值的 25% 或最小 1。
        self.update_displayed_hp();
        // 环岛赛道落格提示条：逐帧衰减（主地图渲染读 `board_toast` 时已过期则不画）。
        if self.board_toast_ticks > 0 {
            self.board_toast_ticks -= 1;
            if self.board_toast_ticks == 0 {
                self.board_toast = None;
            }
        }
        if self.transition.is_none() {
            self.render_scene();
            self.draw_fx();
            return;
        }
        // 取出过渡态（避开与 `render_scene` 的 `&mut self` 借用冲突）。
        let Some(mut trans) = self.transition.take() else {
            self.render_scene();
            self.draw_fx();
            return;
        };
        // S17.4：时间制推进（MOTION_TRANSITION_MS=240ms，帧率无关；delta 下限 1ms 防
        // `frame_delta_ms=0` 卡死过渡）。
        let next_t = advance_transition_t(trans.t, self.frame_delta_ms);
        let done = next_t >= 1.0;
        self.render_scene();
        if done {
            // 终帧：直接显示全新画面（不混合），过渡自然结束。
            self.draw_fx();
            return;
        }
        match trans.prev.take() {
            Some(prev) => {
                // 真交叉淡化：旧画面 → 新画面逐帧插值（EXP-081；S17.9：缓动 t，ease-in-out）。
                self.shell.crossfade_from(&prev, transition_ease(trans.t));
                trans.prev = Some(prev);
            }
            None => {
                // 无旧画面快照（进入前未渲染过）：退回亮度调制（S17.9 同套缓动）。
                self.shell.apply_brightness(transition_ease(trans.t));
            }
        }
        trans.t = next_t;
        self.transition = Some(trans);
        self.draw_fx();
    }

    /// S18.5：FX 粒子渲染（z = [`crate::z_layers::Z_FX`] 层；FX = 强调层 = 场景+chrome 之后绘制，
    /// 调用序 = 层序；`08_render_arch.md` 契约 1）。淡出 = 亮度 4 档量化（CPU Framebuf 无 alpha）。
    fn draw_fx(&mut self) {
        for (x, y, w, h, c) in self.fx.render_rects() {
            // S18.7a: FX 走**绘制目标**（CPU = 帧缓冲 / Bevy = draw list）→ 两后端一致。
            self.draw_target().fill_rect(x, y, w, h, c);
        }
    }

    /// 实际场景渲染（不含过渡逻辑）。
    fn render_scene(&mut self) {
        let kind = SceneKind::from_legacy(self.scene);
        // S15.4：legacy 像素写帧层移除后，场景渲染 = 纯代码绘制（chrome/menu_shell/scenes，
        // 1080×1920，`08_render_arch.md` 契约 2）。不再叠加 `.Map` 精灵层 / 地形瓦片。
        {
            let host: &mut dyn SceneHost = self;
            route_render(host, kind);
        }
        self.rendered_once = true;
    }

    /// HP 条平滑插值（EXP-085 → S17.4 时间制）：按 `MOTION_HP_LERP_MS`=160ms 线性趋近目标，
    /// 帧率无关（旧实现按帧推进差值的 25%）。
    fn update_displayed_hp(&mut self) {
        let Some(b) = self.battle.as_ref() else {
            return;
        };
        let frac = f64::from(self.frame_delta_ms)
            / f64::from(crate::scenes::chrome::MOTION_HP_LERP_MS.max(1));
        let target_p = b.player_hp;
        let target_b = b.boss_hp;
        self.displayed_player_hp = approach_ms(self.displayed_player_hp, target_p, frac);
        self.displayed_boss_hp = approach_ms(self.displayed_boss_hp, target_b, frac);
    }

    /// 场景切换过渡是否进行中（S17.9：主循环 render 门控用——过渡窗口内强制逐帧
    /// `render()`，否则 S4.3 静止帧跳渲染会把交叉淡化冻结在首帧 ~7% 旧画面，切屏残影）。
    #[must_use]
    pub fn transition_active(&self) -> bool {
        self.transition.is_some()
    }

    /// 启动场景切换过渡（EXP-081 真交叉淡化 → S17.4 时间制：`MOTION_TRANSITION_MS`=240ms）。
    ///
    /// 抓取当前帧缓冲作为「旧画面」；`enter()` 在切场景前调用本方法，
    /// 此刻帧缓冲仍持有旧场景像素，故快照有效。
    ///
    /// 两道守卫（否则首帧黑闪）：
    /// - 首帧前（`rendered_once == false`）无旧画面可混合 → 跳过过渡；
    /// - 过渡已在进行时再次切场景 → 沿用原 `prev` 并重置进度（不重抓半混合帧）。
    pub fn start_transition(&mut self) {
        if !self.rendered_once {
            self.transition = None;
            return;
        }
        if let Some(existing) = self.transition.as_mut() {
            existing.t = 0.0;
            return;
        }
        self.transition = Some(TransitionState { prev: Some(self.shell.fb.snapshot()), t: 0.0 });
    }

    /// 视觉回归基准（EXP-072）：渲染当前场景后返回 1080×1920 RGB565 帧缓冲只读切片。
    ///
    /// 测试专用（`tests/visual_regression.rs`）：调本方法拿像素 → 与
    /// `assets/hd_baseline/<场景>.png` 基准图逐像素 diff（RGB565→RGB 量化误差 ≤1）。
    /// 与 [`Self::render`] 同源（场景纯代码绘制，`08_render_arch.md` 契约 2），
    /// 不含 `present` 上屏副作用。
    ///
    /// 快照确定性：先清过渡态 + 重置动画帧计数（微光/翻页正弦），
    /// 使同一场景的快照像素与历史渲染次数无关（EXP-079 动画化回归防护）。
    /// S15.6 QA121：重置后预推 5 帧翻页计数（淡入窗口 0..=4 全过），快照锁实机
    /// **稳态**亮度 1.0——旧实现 `PAGE_FRAME`=0 时亮度 0.6，三套基线锁的是过渡中间态。
    #[must_use]
    pub fn render_to_framebuf(&mut self) -> &[u16] {
        // S18.5 快照口径（EXP-079 稳态）：像素基线 / selftest 不含 FX 粒子（clear 后渲染）；
        // 实玩路径（`render`）粒子照常。
        self.fx.clear();
        self.transition = None;
        crate::scenes::chrome::reset_animation_state();
        // S17.4：翻页亮度推进到稳态（时间制：一次推进满 `MOTION_PAGE_MS`）。
        crate::scenes::chrome::tick_page_frame(crate::scenes::chrome::MOTION_PAGE_MS);
        self.render();
        self.shell.fb.as_slice()
    }
}

/// `GameApp` 的 8 个 [`SceneHost`] 子 trait 实现：全部是一行状态读写 / 表现转发，
/// 业务判定在 `scenes/*.rs`（`docs/process/01_architecture.md` §2）。
impl NavHost for GameApp {
    fn current_scene(&self) -> Scene {
        self.scene
    }

    fn enter(&mut self, next: SceneKind) {
        self.scene = next.to_legacy();
        self.start_transition();
    }

    fn advance(&mut self, input: GameInput) -> bool {
        self.handle(input)
    }
}

impl RenderHost for GameApp {
    fn shell(&self) -> &MenuShell {
        &self.shell
    }

    fn shell_mut(&mut self) -> &mut MenuShell {
        &mut self.shell
    }

    /// 绘制目标（S18.7a 可插拒）：记录模式 → 绘制列表（Bevy 路径）；
    /// 否则 → CPU 帧缓冲（诊断 bin 路径；macroquad 发货壳 S18.8a 已退役）。
    fn draw_target(&mut self) -> &mut dyn ti_render::canvas::Canvas {
        if self.record_draw { &mut self.draw_list } else { &mut self.shell }
    }

    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        let w = crate::menu_shell::LOGICAL_W_USIZE;
        let h = crate::menu_shell::LOGICAL_H_I32;
        let wi = i32::try_from(w).unwrap_or(i32::MAX);
        if x < 0 || y < 0 || x >= wi || y >= h {
            return 0;
        }
        let xi = usize::try_from(x).unwrap_or(0);
        let yi = usize::try_from(y).unwrap_or(0);
        let idx = yi * w + xi;
        self.shell.fb.as_slice().get(idx).copied().unwrap_or(0)
    }
}

impl SpriteHost for GameApp {
    // S15.4：legacy 像素写帧（`blit_scene_sprites` / `draw_terrain`）已移除——运行时
    // 场景/主地图纯代码绘制（`08_render_arch.md` 契约 2），不再叠 `.Map` 精灵层 / 地形瓦片。
    // 保留 2 个等距几何访问器（`terrain_grid_w` / `has_terrain_atlas`），
    // 主地图代码棋盘仍据 `terrain_grid_w`² 铺菱形格（§4.3）。
    // S15.6 QA114：旧 `terrain_camera`（176 时代 88/110 老屏中心偏移，零消费方）已删。

    fn terrain_grid_w(&self) -> u32 {
        self.terrain_map.as_ref().map_or(20, |t| t.grid_w)
    }

    fn has_terrain_atlas(&self) -> bool {
        self.terrain_atlas.is_some()
    }
}

impl MenuHost for GameApp {
    fn menu_selected(&self) -> usize {
        self.menu_selected
    }

    fn set_menu_selected(&mut self, value: usize) {
        self.menu_selected = value;
    }

    fn reset_selection(&mut self) {
        self.selection = SelectionState::new();
        self.character = None;
        self.map_slot = None;
    }

    fn open_help(&mut self) {
        self.open_message(MsgKind::Help);
    }

    fn request_quit(&mut self) {
        self.quit = true;
    }

    fn press_held(&self) -> bool {
        self.press_held
    }

    fn frame_delta_ms(&self) -> u32 {
        self.frame_delta_ms
    }
}

impl SelectionHost for GameApp {
    fn character(&self) -> Option<Character> {
        self.character
    }

    fn set_character(&mut self, value: Option<Character>) {
        self.character = value;
    }

    fn map_slot(&self) -> Option<MapSlot> {
        self.map_slot
    }

    fn set_map_slot(&mut self, value: Option<MapSlot>) {
        self.map_slot = value;
    }

    fn selection(&self) -> SelectionState {
        self.selection
    }

    fn selection_mut(&mut self) -> &mut SelectionState {
        &mut self.selection
    }

    fn map_unlocked(&self, slot: MapSlot) -> bool {
        self.unlocked_maps.contains(&slot)
    }

    fn enter_main_map(&mut self) -> bool {
        GameApp::enter_main_map(self)
    }
}

impl MapHost for GameApp {
    fn map_pos(&self) -> Option<GridPos> {
        self.map_pos
    }

    fn set_map_pos(&mut self, value: Option<GridPos>) {
        self.map_pos = value;
    }

    fn map_terrain(&self) -> Option<MapTerrain> {
        self.map_terrain.clone()
    }

    fn map_landmarks(&self) -> Vec<(u32, u32)> {
        self.map_landmarks.clone()
    }

    /// P0-2：BOSS 格坐标（`MapSession::boss_tile`，渲染橙红三角用）。
    fn map_boss_tile(&self) -> Option<GridPos> {
        self.map_session.boss_tile()
    }

    fn map_session_dispatch(&self, code: u32) -> Option<SceneTarget> {
        self.map_session.dispatch(code)
    }

    fn set_last_dispatch(&mut self, target: Option<SceneTarget>) {
        self.last_dispatch = target;
    }

    fn start_battle(&mut self) {
        let seed = self.rng.next().unsigned_abs();
        // 据点档用当前地图对应的固定 boss_tier（非随机）：`enter_main_map` 已按 map_slot 写入
        let tier = self.current_boss_tier;
        // FIND-013：玩家单位读 `balance.combat.player`（EXP-066 重复踩坑闭环：不再写死 1000/150）；
        // 硬报错口径与 `enter_main_map` 的 initial_gold 一致（数据缺失 = eprintln + return，不静默 fallback）
        let b = match balance() {
            Ok(b) => b,
            Err(_e) => {
                #[cfg(feature = "stats")]
                eprintln!("[stats] start_battle: balance.json 加载失败（硬错误，不 fallback）");
                return;
            }
        };
        let Some(player) = b.combat.player else {
            eprintln!("start_battle: balance.combat.player 缺失（硬错误，不 fallback）");
            return;
        };
        // 成长机制（docs/business/combat.md §建造加成）：玩家 atk/hp = 基础值 + 已建建筑加成。
        // 无建筑 → 加成为 0，退化为 base 战斗（与原行为一致，EXP-066 闭环不受影响）。
        let player_atk = player.atk.saturating_add(self.build_atk_bonus());
        // P1-7（M5）：卡牌 Damage 暂存（`card_state.hp`）并入战斗——减益后开打。
        let player_hp = player.hp.saturating_add(self.build_hp_bonus());
        let player_hp = player_hp.saturating_sub(self.card_state.hp);
        let player_unit = Unit { hp: player_hp, atk: player_atk };
        self.card_state = cards::CardState::default();
        self.battle = BattleView::new(seed, player_unit, tier);
        self.battle_won = false; // Q005：开新战斗复位（防上一场陈旧结果）
        self.scene = Scene::Vs;
    }

    fn open_message(&mut self, kind: MsgKind) {
        self.msg_kind = Some(kind);
        self.msg_page = 0;
        self.msg_total = total_pages(kind);
        self.scene = Scene::Message;
    }

    fn board_spaces(&self) -> Vec<TrackSpace> {
        self.board.as_ref().map(|s| s.track().spaces().to_vec()).unwrap_or_default()
    }

    fn board_index(&self) -> usize {
        self.board.as_ref().map_or(0, BoardSession::index)
    }

    fn board_last_roll(&self) -> u32 {
        self.board.as_ref().map_or(0, BoardSession::last_roll)
    }

    fn board_spin(&mut self) -> Option<Turn> {
        self.spin_board()
    }

    fn board_toast(&self) -> Option<String> {
        if self.board_toast_ticks > 0 { self.board_toast.clone() } else { None }
    }
}

impl EconomyHost for GameApp {
    fn price_of(&self, gid: u32) -> Option<u32> {
        self.price_table.lookup(gid)
    }

    fn sell_price_of(&self, gid: u32) -> Option<u32> {
        let b = balance().ok()?;
        self.price_table.lookup(gid).map(|buy| trade::sell_price(buy, b.trade.markup))
    }

    fn gold(&self) -> u32 {
        self.gold
    }

    fn set_gold(&mut self, value: u32) {
        self.gold = value;
    }

    fn owned_index_of(&self, gid: u32) -> Option<usize> {
        self.owned.iter().position(|(id, _)| *id == gid)
    }

    fn owned_push(&mut self, gid: u32, qty: u32) {
        self.owned.push((gid, qty));
    }

    fn owned_bump(&mut self, index: usize) {
        if let Some(entry) = self.owned.get_mut(index) {
            entry.1 += 1;
        }
    }

    fn owned_count(&self, gid: u32) -> u32 {
        self.owned.iter().find(|(id, _)| *id == gid).map_or(0, |&(_, q)| q)
    }

    fn owned_snapshot(&self) -> Vec<(u32, u32)> {
        self.owned.clone()
    }

    fn owned_total(&self) -> usize {
        self.owned.len()
    }

    fn owned_take_all(&mut self, gid: u32) -> u32 {
        let removed = self.owned_count(gid);
        self.owned.retain(|(id, _)| *id != gid);
        removed
    }

    fn bank_gold(&self) -> u32 {
        self.bank.gold
    }

    fn spend_bank_gold(&mut self, amount: u32) {
        self.bank.gold -= amount;
    }

    fn build_increment(&mut self, kind: Building) {
        match self.buildings.iter_mut().find(|(k, _)| *k == kind) {
            Some(entry) => entry.1 += 1,
            None => self.buildings.push((kind, 1)),
        }
    }

    /// P1-6（T1）：金量上限读 `balance.economy.gold_max`（建造可抬升，`rules::build`）。
    fn gold_cap(&self) -> u32 {
        match data::balance() {
            Ok(b) => b.economy.gold_max,
            Err(_) => 0,
        }
    }

    /// P1-6（T1）：卖出加金溢出经 `bank_rate` 进银行（`rules::build::Bank` 承接）。
    fn settle_overflow(&mut self, excess: u32) {
        if excess == 0 {
            return;
        }
        let rate = match data::balance() {
            Ok(b) => b.economy.bank_rate,
            Err(_) => 1.0,
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let to_bank =
            ((f64::from(excess) * f64::from(rate)).round()).min(f64::from(u32::MAX)) as u32;
        self.bank.deposit = self.bank.deposit.saturating_add(to_bank);
    }

    /// P1-7（M5）：卡牌效果真接进可玩路径——`event_id` 经 `rules::cards::CardEngine`
    /// 映射到 `balance.cards` 卡，`apply_card` 结算（扣金/加金/扣 HP/加 HP，区间钳制）。
    ///
    /// 返回是否实际改动状态（`Applied` = true，`UnknownEvent` = false）。
    fn apply_card_effect(&mut self, event_id: u32) -> bool {
        let engine = cards::CardEngine::from_balance();
        let Some(card) = cards::event_to_card(event_id, engine.catalog()) else {
            return false;
        };
        // 钳制上限：hp_max = balance.combat.player.hp，gold_max = balance.economy.gold_max
        let (hp_max, gold_max) = match data::balance() {
            Ok(b) => {
                let hp = match b.combat.player {
                    Some(p) => p.hp,
                    None => u32::MAX,
                };
                (hp, b.economy.gold_max)
            }
            Err(_) => (u32::MAX, u32::MAX),
        };
        // 卡牌状态从宿主现状出发（Damage/Heal 改当前 hp，Gold 改当前 gold）
        let mut state = cards::CardState { hp: 0, gold: self.gold, hp_max, gold_max };
        let result = cards::apply_card(card, &mut state);
        match result {
            cards::CardResult::Applied(eff) => {
                match eff {
                    cards::CardEffect::Gold(n) => {
                        // 卡牌加金走宿主 gold（saturating_add + 钳 economy.gold_max）
                        self.gold = self.gold.saturating_add(n).min(gold_max);
                    }
                    cards::CardEffect::Damage(n) => {
                        // 扣 HP：暂存到 `card_state.hp`（本次掉了 n），
                        // 下次 `start_battle` 时并入玩家 hp（成长机制 + 卡牌 debuff 同通道）。
                        self.card_state.hp = self.card_state.hp.saturating_add(n);
                    }
                    cards::CardEffect::Heal(n) => {
                        // 加 HP：从暂存扣减（Heal 抵消之前的 Damage 暂存，钳 0）
                        self.card_state.hp = self.card_state.hp.saturating_sub(n);
                    }
                    cards::CardEffect::Move | cards::CardEffect::Unknown => {}
                }
                true
            }
            cards::CardResult::UnknownEvent => false,
        }
    }
}

impl SessionHost for GameApp {
    fn play_sfx(&mut self, sfx: Sfx) {
        self.last_sfx = Some(sfx);
    }

    fn battle_step(&mut self) {
        if let Some(b) = self.battle.as_mut() {
            b.step();
            if b.outcome.is_some() {
                self.battle_won = matches!(b.outcome, Some(Side::Player));
                if self.battle_won {
                    self.last_sfx = Some(Sfx::Victory);
                    // 跨图推进（`business/progression.md`）：击败当前图 BOSS → 解锁下一图。
                    // 结算走规则层（`from_unlocked` 重建 + `on_boss_defeated`，幂等）：
                    // 重复攻击已结束时集合不变；失败（`battle_won=false`）不解锁（可重试当前图）。
                    if let Some(ms) = self.map_slot {
                        self.unlocked_maps = ProgressionState::from_unlocked(&self.unlocked_maps)
                            .on_boss_defeated(ms)
                            .unlocked();
                        // 通关门禁（`business/progression.md` §通关）：击败 m03 BOSS = 全图通杀，
                        // 置通关位（`is_cleared`）；解锁集不变（m03 无下一图）。
                        if ms == MapSlot::M03 {
                            self.cleared = true;
                        }
                    }
                }
            }
        }
    }

    fn battle_snapshot(&self) -> Option<BattleSnapshot> {
        self.battle.as_ref().map(|b| BattleSnapshot {
            player_hp: b.player_hp,
            boss_hp: b.boss_hp,
            rounds: b.rounds,
            outcome: b.outcome,
            player_hp_max: b.player_hp_max,
            boss_hp_max: b.boss_hp_max,
        })
    }

    fn displayed_player_hp(&self) -> u32 {
        self.displayed_player_hp
    }

    fn displayed_boss_hp(&self) -> u32 {
        self.displayed_boss_hp
    }

    fn msg_kind(&self) -> Option<MsgKind> {
        self.msg_kind
    }

    fn msg_page(&self) -> u32 {
        self.msg_page
    }

    fn msg_total(&self) -> u32 {
        self.msg_total
    }

    fn msg_next_page(&mut self) {
        if self.msg_page + 1 < self.msg_total {
            self.msg_page += 1;
        }
    }

    fn msg_prev_page(&mut self) {
        if self.msg_page == 0 {
            self.msg_page = self.msg_total.saturating_sub(1);
        } else {
            self.msg_page -= 1;
        }
    }

    fn close_message(&mut self) {
        self.scene = if self.map_pos.is_some() && self.battle.is_none() {
            Scene::MainMap
        } else {
            Scene::Index
        };
        self.msg_kind = None;
    }

    fn save_slots(&self) -> &[SaveSlot] {
        &self.save_slots
    }

    fn write_save_slot(&mut self, slot: usize, data: SaveSlot) {
        if let Some(dst) = self.save_slots.get_mut(slot) {
            *dst = data;
        }
    }

    fn restore_save(&mut self, data: SaveSlot) {
        GameApp::restore_save(self, data);
    }

    fn in_map(&self) -> bool {
        self.map_pos.is_some()
    }

    fn in_battle(&self) -> bool {
        self.battle.is_some()
    }

    /// P1-8（S1）：当前 RNG 已消费次数（写档捕获，读档 `restore_state` 精确续流）。
    fn rng_steps(&self) -> u128 {
        self.rng.steps_taken()
    }

    /// S15.2（F23）：写档捕获当前跨图推进解锁位掩码（口径 = 规则层 `unlocked_mask()`）。
    fn unlocked_mask(&self) -> u8 {
        let base = ProgressionState::from_unlocked(&self.unlocked_maps).unlocked_mask();
        if self.cleared { base | CLEARED_BIT } else { base }
    }

    fn saveload_cursor(&self) -> usize {
        self.saveload_cursor
    }

    fn set_saveload_cursor(&mut self, slot: usize) {
        self.saveload_cursor = slot;
    }
}

impl SceneHost for GameApp {}

/// 地图槽位 → 地图 id（`terrain.json` 键；S18.9 地形查找）。
#[must_use]
pub fn map_slot_id(slot: MapSlot) -> &'static str {
    slot.id()
}

/// 构建环岛赛道会话（`rules::board`）：
/// - 赛道数据 = `assets/data/board.json`（`scripts/extract_board.py` 生成）；
/// - 开档（`start_override = None`）= 赛道 `start` 格；
/// - 读档（`start_override = Some(cell)`）= 该格在赛道上 → 直接定位；不在 → 就近吸附最近赛道格。
///
/// 数据缺失/赛道构建失败 = 硬报错（`eprintln` + `None`，不静默 fallback；`docs/business/00_index.md` 口径）。
fn build_board_session(map_id: &str, start_override: Option<GridPos>) -> Option<BoardSession> {
    let db = match data::board() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("enter_main_map: board.json 加载失败（硬错误，不 fallback）：{e}");
            return None;
        }
    };
    let Some(m) = db.map(map_id) else {
        eprintln!("enter_main_map: board.json 缺 {map_id} 赛道（硬错误，不 fallback）");
        return None;
    };
    let track = match Track::from_map(m) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("enter_main_map: 赛道构建失败（硬错误，不 fallback）：{e}");
            return None;
        }
    };
    let index = match start_override {
        Some(cell) => track.index_of(cell).unwrap_or_else(|| nearest_track_index(&track, cell)),
        None => track.start_index(),
    };
    Some(BoardSession::new(track, index))
}

/// 就近赛道格序号（曼哈顿距离；同距取先出现的序号，确定性）。
fn nearest_track_index(track: &Track, cell: GridPos) -> usize {
    let mut best = 0usize;
    let mut best_d = u32::MAX;
    for (i, s) in track.spaces().iter().enumerate() {
        let d = s.cell.row.abs_diff(cell.row) + s.cell.col.abs_diff(cell.col);
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}
#[cfg(test)]
mod transition_tests {
    use super::transition_ease;

    /// S17.9：缓动曲线端点保持（0→0、1→1）+ [0,1] 上单调不递减 + 值域不越界。
    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn ease_endpoints_and_monotone() {
        assert!((transition_ease(0.0) - 0.0).abs() < f32::EPSILON, "t=0 必须 = 0（全旧画面）");
        assert!((transition_ease(1.0) - 1.0).abs() < f32::EPSILON, "t=1 必须 = 1（全新画面）");
        let mut prev = 0.0f32;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let e = transition_ease(t);
            assert!((0.0..=1.0).contains(&e), "缓动值越界 t={t}");
            assert!(e >= prev, "缓动非单调 t={t}");
            prev = e;
        }
        // 越界输入被 clamp（不 panic、不越值域）。
        assert!((transition_ease(-0.5) - 0.0).abs() < f32::EPSILON, "负输入 clamp 到 0");
        assert!((transition_ease(1.5) - 1.0).abs() < f32::EPSILON, "超 1 输入 clamp 到 1");
    }
}
