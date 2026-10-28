//! 可玩交互核心（组合根）：跨场景状态 + 输入路由 + 表现转发。
//!
//! S20.0b 拆分（原 `game_app.rs` 2181 行上帝文件）：本文件只留**类型定义与组合根**，
//! 按职责分模块：
//!
//! | 模块 | 职责 |
//! |---|---|
//! | `state` | 状态访问器（gold/owned/bank/buildings/...） |
//! | `input` | 键位快照 / 触摸手势 / Command 分发 |
//! | `render` | 帧渲染入口 + 动效 + 摄像机 |
//! | `assets` | HD 资产注入（atlas / terrain / HD pack） |
//! | `board` | 环岛赛道（进入主地图 / 摇骰 / 落格结算） |
//! | `battle` | 战斗会话（`BattleView`） |
//! | `saveload` | 存读档（存档槽读写 + 状态恢复） |
//! | `host` | 9 个宿主 trait 接线（`SceneHost` / `MapHost` / ...） |
//!
//! 设计原则（`docs/gamebook.md` §7）：不重抄数值 / 零占位 / RNG 单一源 / 渲染复用。

mod assets;
mod battle;
mod board;
mod host;
mod input;
mod render;
mod saveload;
mod state;

pub use battle::BattleView;

use ti_core::data;
use ti_core::domain::combat::Side;
use ti_core::domain::economy::{Bank, Building, PriceTable};
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{GridPos, MapTerrain, SceneTarget};
use ti_core::domain::messages::MsgKind;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot, SelectionState};
use ti_core::rules::board::BoardSession;
use ti_core::rules::boat::BoatSession;
use ti_core::rules::cards;
use ti_core::rules::main_loop::SceneStack;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::progression::ProgressionState;
use ti_core::rules::rng::GameRng;
use ti_core::rules::territory::{TerritoryState, TurnCycle};

use crate::menu_shell::MenuShell;

use crate::sprite_atlas::SpriteAtlas;
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
    /// S21.12 主地图造船（船系统单向升级，`rules::boat::BoatSession`；与 4 建筑并列第 5 入口）。
    BoatUpgrade,
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

/// S29.2：拾取命中事件（B1：命中/几何进 Bevy 拾取层，逻辑核心消费归一事件，headless 可测）。
///
/// 由 Bevy 壳从 [`ti_engine::pick::PickEvent`] 映射（或 headless 测试直接构造）；
/// `key`/`index` = 被点元素（数据 `pick_key`/`pick_index`，opaque；语义按场景解释），
/// `swipe` = 滑动方向（`0` = 点按 / 无滑动，`1..=4` = 上/下/左/右，与
/// `ti_engine::pick::SWIPE_*` 对齐）。替代旧 `TouchSample`（手势 + 坐标 → 手写几何逆）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HitData {
    /// 目标族键（`0` = 无元素命中 / 任意点按 / 空白）。
    pub key: u32,
    /// 目标项下标（`-1` = 无）。
    pub index: i32,
    /// 滑动方向（`0` = 点按；`1`/`2`/`3`/`4` = 上/下/左/右）。
    pub swipe: u8,
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
    /// S29.3（B1）：UI chrome 是否由 Bevy `bevy_ui` 绘制（true = 已迁 `bevy_ui` 的原语
    /// 由 `ti-engine` 画，场景层跳过，避免双画；false = CPU 路径照旧）。
    pub chrome_in_bevy_ui: bool,
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
    /// S21.11 船会话（`rules::boat::BoatSession`；等级链 1..=4，全局账号级；存档持久化
    /// `SaveSlot::boat_level`，读档按存档值恢复）。
    pub boat: BoatSession,
    /// 落格提示条文本（`Some` = 有提示待显示；随帧递减过期）。
    pub board_toast: Option<String>,
    /// 落格提示条剩余帧数。
    pub board_toast_ticks: u32,
    /// S28.7 最近一次 AI 行动提示（独立于玩家落格 toast 的展示通道；不覆盖玩家签条，
    /// 主地图 v5 渲染可挂独立一行 = AI 回合可见反馈，territory.md §6）。
    pub last_ai_toast: Option<String>,
    /// S28.4 玩家席位标记（`territory.md` §1：触摸行点选 toggle；空 = 单选旧流回退
    /// 主角色单席；非空 = 多选玩家席 + 其余自动 AI，`rules::selection::seat_pool`）。
    pub seat_marks: Vec<ti_core::domain::selection::Character>,
    /// S28.6 待确认的副本入口（`Some(格序号)` = 落格提示在位，主地图 `Confirm` 进副本战；
    /// 再次摇骰 / 其他行动 = 自动清除（新落格覆盖，territory.md §4 玩家选择权））。
    pub pending_dungeon: Option<u32>,
    /// S21.7 赛道节点预览选中（`Some(index)` = 该节点高亮 + 信息签；只读预览不移动，
    /// 摇骰时清空；主地图场景时有效，非主地图 = `None`）。
    pub track_node_sel: Option<usize>,
    /// S21.12 建造面板是否打开（主地图场景时有效；`false` = 关闭）。
    ///
    /// 面板 = 4 建筑 + 「造船」第 5 入口（`docs/business/economy.md` §船系统 规则 1）。
    pub build_panel_open: bool,
    /// S21.12 建造面板当前选中项（0..=4 = 城堡/投资/银行/要塞/造船）。
    pub build_panel_sel: u8,
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
    /// 已解析的 `.Map` 场景缓存已随飞轮 R1 ⑦-11 删除（零读方死字段；
    /// legacy `.Map` 解析归 `legacy/` 参照，契约 1.3）。
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
    /// S28.7 领地账本（`rules::territory::TerritoryState`，`MainMap` 场景时有效；开档/切图时构建）。
    pub territory: Option<TerritoryState>,
    /// S28.7 各席位金币（`[u32; 5]`，§1 席位独立经济；主玩家金 = 本席位值，`gold` 字段保留主席镜像）。
    pub seat_gold: [u32; 5],
    /// S28.7 席位轮转（§1：5 席顺序 = 角色 index 升序；`current` = 当前轮到谁）。
    pub turn_cycle: TurnCycle,
    /// S28.7 AI 席赛道会话（`[5]`；主玩家席走 `board` 字段，AI 席各持独立位置/圈数）。
    pub ai_sessions: [Option<BoardSession>; 5],
}
/// S18.5：FX 粒子 LCG 固定 seed（确定性视觉验收；不进业务 RNG `rules::rng::GameRng`）。
pub const FX_SEED: u64 = 0xC0_FF_EE;
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
/// 场景切换过渡状态机（EXP-081 → S17.4：真交叉淡化，`MOTION_TRANSITION_MS` 时间制推进）。
#[derive(Debug, Clone, Default)]
pub struct TransitionState {
    /// 旧画面快照（`enter()` 前抓的帧缓冲，`render_to_framebuf` / 快照路径为 `None`）。
    pub prev: Option<Box<[u16]>>,
    /// 当前插值进度（0.0 = 全旧画面，1.0 = 全新画面；按 ms 推进，S17.4）。
    pub t: f32,
}
impl GameApp {
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
            chrome_in_bevy_ui: false,
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
            boat: BoatSession::from_balance()
                .unwrap_or_else(|_| BoatSession::new(data::BoatCfg::default(), 1)),
            board_toast: None,
            board_toast_ticks: 0,
            track_node_sel: None,
            build_panel_open: false,
            build_panel_sel: 0,
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
            // S28.7 领地/席位初值（`territory.md` §1：单席位开局，领地空账本）。
            territory: None,
            seat_gold: [0; 5],
            turn_cycle: TurnCycle::default(),
            ai_sessions: core::array::from_fn(|_| None),
            last_ai_toast: None,
            pending_dungeon: None,
            seat_marks: Vec::new(),
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
}
/// 地图槽位 → 地图 id（`terrain.json` 键；S18.9 地形查找）。
#[must_use]
pub fn map_slot_id(slot: MapSlot) -> &'static str {
    slot.id()
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
