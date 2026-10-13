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

use ti_core::data::balance;
use ti_core::domain::combat::{Side, Unit};
use ti_core::domain::economy::{Bank, Building, PriceTable};
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{Dir, GridPos, MapTerrain, SceneTarget};
use ti_core::domain::messages::MsgKind;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot, SelectionState};
use ti_core::rules::battle::BattleSession;
use ti_core::rules::main_loop::SceneStack;
use ti_core::rules::main_map::{MapSession, StepResult};
use ti_core::rules::messages::total_pages;
use ti_core::rules::rng::GameRng;

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
        Some(Self { player, boss_tier, session, rounds: 0, player_hp: player.hp, boss_hp, outcome })
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
    /// 主地图方向移动（`Dir` 参数）。
    Walk(Dir),
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
pub struct GameApp {
    /// 表现层（帧缓冲/字形/呈现，Index 场景渲染子视图）。
    pub shell: MenuShell,
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
    /// 已解锁地图清单（壳层探测 `glmapm01/02/03.Map` 得到）。
    pub unlocked_maps: Vec<MapSlot>,
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
    /// 建筑列表（`(kind, level)`，`rules::build` 行为镜像，`MainMap` 场景时有效）。
    pub buildings: Vec<(Building, u32)>,
    /// 战斗会话（`Vs` 场景时有效）。
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
    /// 音效（本次交互触发的音效，壳层播放）。
    pub last_sfx: Option<Sfx>,
    /// 战斗结果（`Vs` 场景结束时镜像，用于回 `Index`）。
    pub battle_won: bool,
    /// 退出标志（`Index` 场景菜单项 5 置位）。
    pub quit: bool,
    /// 主地图事件分派结果（最近一次 `dispatch`，壳层据此跳场景）。
    pub last_dispatch: Option<SceneTarget>,
    /// 精灵图集（`ui.pak` 的 120 个 PCX 贴图 + 解码缓存，`docs/design/ui_controls.md` §3）。
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
}

/// 键位快照（壳层把触摸/键映射到这些布尔位，平台无关，`docs/gamebook.md` §3）。
///
/// 各 bin 在每帧从 macroquad 按键状态读取，喂给 [`GameApp::poll`]。
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
    /// 据点触发（Space，主地图）。
    pub landmark: bool,
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
                if k.up {
                    changed = self.handle(GameInput::Walk(Dir::North)) || changed;
                }
                if k.down {
                    changed = self.handle(GameInput::Walk(Dir::South)) || changed;
                }
                if k.left {
                    changed = self.handle(GameInput::Walk(Dir::West)) || changed;
                }
                if k.right {
                    changed = self.handle(GameInput::Walk(Dir::East)) || changed;
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
            scenes: SceneStack::new(),
            scene: Scene::Index,
            menu_selected: 0,
            character: None,
            map_slot: None,
            current_boss_tier: 0,
            unlocked_maps: MapSlot::all().to_vec(),
            selection: SelectionState::new(),
            map_terrain: None,
            map_pos: None,
            map_landmarks: Vec::new(),
            gold: 0,
            price_table: PriceTable::placeholder(),
            owned: Vec::new(),
            bank: Bank::default(),
            map_session: MapSession::new(MapTerrain::open(1, 1), GridPos::new(0, 0), &[]),
            buildings: Vec::new(),
            battle: None,
            rng: GameRng::new(0),
            rng_seed: 0,
            msg_kind: None,
            msg_page: 0,
            msg_total: 0,
            save_slots: vec![SaveSlot::default(); 5],
            last_sfx: None,
            battle_won: false,
            quit: false,
            last_dispatch: None,
            atlas: None,
            scene_cache: HashMap::new(),
            terrain_atlas: None,
            frame_delta_ms: 0,
            terrain_map: None,
            #[cfg(feature = "gpu-render")]
            gpu_renderer: {
                #[cfg(any(
                    feature = "wasm-shell",
                    feature = "native-shell",
                    feature = "android-shell"
                ))]
                {
                    ti_platform::gpu_renderer::GpuRenderer::new()
                }
                #[cfg(not(any(
                    feature = "wasm-shell",
                    feature = "native-shell",
                    feature = "android-shell"
                )))]
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
    pub fn restore_save(&mut self, data: SaveSlot) {
        if data.character > 0 && data.character <= 5 {
            self.character = Character::from_index(data.character - 1);
            self.selection.character = self.character;
        }
        if data.map_slot < 3 {
            self.map_slot = MapSlot::from_index(data.map_slot);
            self.selection.map_slot = self.map_slot;
        }
        self.gold = data.gold;
        self.bank = Bank::new(data.gold);
        if self.selection.has_character() && self.selection.has_map() {
            GameApp::enter_main_map(self);
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
    /// 调用方随后调 [`Self::present_hd`] 上屏。
    #[cfg(feature = "hd-8bit")]
    pub fn render_hd(&mut self) {
        self.render();
        self.shell.sync_rgba_from_rgb565();
        // EXP-075：在 RGBA 帧缓冲上叠加 HD PNG 精灵（此前漏调，HD 贴图从未生效）
        let asset = SceneKind::from_legacy(self.scene).asset_name();
        if !asset.is_empty() {
            if let (Some(atlas), Some(scene)) = (self.atlas.as_mut(), self.scene_cache.get(asset)) {
                if !scene.words.is_empty() {
                    atlas.advance_animation(self.frame_delta_ms);
                    atlas.blit_scene_rgba(scene, &mut self.shell.fb_rgba);
                }
            }
        }
        // 地形 HD 同理（`blit_scene_rgba` 同源；terrain_atlas 无 RGBA 版接口时不接入）
        #[allow(unused_variables)]
        let _ = &self.terrain_atlas;
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
        let mut png_map: std::collections::BTreeMap<u32, Vec<(u32, Vec<u8>)>> =
            std::collections::BTreeMap::new();
        let Ok(read_iter) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in read_iter.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
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
        route_input(self, kind, input)
    }

    /// S9.5 Command 派发入口：把 [`crate::commands::GameCommand`] 路由到当前场景。
    ///
    /// 与 [`Self::handle`] 等价（`GameCommand::to_input` 后走 `route_input`），
    /// 但语义层统一为 Command 模式，供命令派发枚举调用。
    /// 返回是否消费了该输入（发生了场景切换）。
    pub fn dispatch(&mut self, cmd: crate::commands::GameCommand) -> bool {
        let kind = SceneKind::from_legacy(self.scene);
        let input = cmd.to_input();
        route_input(self, kind, input)
    }

    /// 进入主地图（`docs/gamebook.md` §3.3）：初始化 `MapSession` + 贸易/建造会话。
    fn enter_main_map(&mut self) {
        // P0-4：grid_w 统一走 terrain_map（bin 注入的真值），不再按 map_slot 推算
        let gw = self.terrain_map.as_ref().map_or(20, |t| t.grid_w);
        let gh = gw;
        // 据点档：按地图槽走 `balance.selection.maps[i].boss_tier`（m01=0, m02=1, m03=2）
        let boss_tier = match self.map_slot {
            Some(MapSlot::M01) => 0u32,
            Some(MapSlot::M02) => 1,
            _ => 2,
        };
        self.current_boss_tier = boss_tier;
        self.map_terrain = Some(MapTerrain::open(gw, gh));
        self.map_pos = Some(GridPos::new(gh / 2, gw / 2));
        self.map_landmarks = vec![(2, 2), (gh.saturating_sub(3), gw.saturating_sub(3))];
        // 主地图会话（`rules::main_map::MapSession`，卡牌池走 `balance.cards`）
        self.map_session = MapSession::new(
            MapTerrain::open(gw, gh),
            GridPos::new(gh / 2, gw / 2),
            &self.map_landmarks,
        );
        // 贸易/建造：开局金走 `balance.economy.initial_gold`（表缺失时硬错误，不 fallback，
        // `02_conventions.md` §4.4 数据失败=硬错误）
        let init_gold = match ti_core::data::balance() {
            Ok(b) => b.economy.initial_gold,
            Err(_e) => {
                #[cfg(feature = "stats")]
                eprintln!("[stats] enter_main_map: balance.json 加载失败（硬错误，不 fallback）");
                return;
            }
        };
        self.gold = init_gold;
        self.owned = Vec::new();
        self.bank = Bank::new(init_gold);
        self.buildings = Vec::new();
        // 选择完成标志
        self.selection.completed = self.selection.has_character() && self.selection.has_map();
        self.scene = Scene::MainMap;
        // 场景管理器状态码同步（`rules::main_loop` 行为）
        self.scenes.set_state(0);
    }

    /// 渲染当前场景：先由场景模块绘制（`scenes/*.rs::render`），
    /// 再叠加该场景 `.Map` 的真实精灵 blit。
    ///
    /// 调用方随后调 [`MenuShell::present`] 上屏。
    pub fn render(&mut self) {
        let kind = SceneKind::from_legacy(self.scene);
        let asset = kind.asset_name();
        {
            let host: &mut dyn SceneHost = self;
            route_render(host, kind);
            host.blit_scene_sprites(asset);
        }
    }

    /// 视觉回归基准（EXP-072）：渲染当前场景后返回 176×220 RGB565 帧缓冲只读切片。
    ///
    /// 测试专用（`tests/visual_regression.rs`）：调本方法拿像素 → 与
    /// `assets/hd_baseline/<场景>.png` 基准图逐像素 diff（RGB565→RGB 量化误差 ≤1）。
    /// 与 [`Self::render`] 同源（场景绘制 + 精灵 blit），不含 `present` 上屏副作用。
    #[must_use]
    pub fn render_to_framebuf(&mut self) -> &[u16] {
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

    fn clear_frame(&mut self, color: u16) {
        self.shell.fb.clear(color);
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.shell.fill_rect(x, y, w, h, color);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32) {
        self.shell.draw_text_row(text, x, y);
    }
}

impl SpriteHost for GameApp {
    fn blit_scene_sprites(&mut self, asset: &str) {
        if asset.is_empty() {
            return;
        }
        if let (Some(atlas), Some(scene)) = (self.atlas.as_mut(), self.scene_cache.get(asset)) {
            if !scene.words.is_empty() {
                // S6.4：按本帧时长推进动画（多帧精灵 num>1 才受影响）
                atlas.advance_animation(self.frame_delta_ms);
                atlas.blit_scene(scene, &mut self.shell.fb);
            }
        }
    }

    fn draw_terrain(&mut self) -> bool {
        match (self.terrain_atlas.as_mut(), self.terrain_map.as_ref()) {
            (Some(atlas), Some(terrain)) => atlas.blit_map(terrain, &mut self.shell.fb) > 0,
            _ => false,
        }
    }

    fn terrain_grid_w(&self) -> u32 {
        self.terrain_map.as_ref().map_or(20, |t| t.grid_w)
    }

    fn terrain_camera(&self) -> (i64, i64) {
        self.terrain_map
            .as_ref()
            .map_or((0_i64, 0), crate::terrain_atlas::TerrainAtlas::center_camera)
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

    fn enter_main_map(&mut self) {
        GameApp::enter_main_map(self);
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

    fn map_session_move(&mut self, dir: Dir) -> StepResult {
        self.map_session.move_to(dir, &mut self.rng)
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
        // P0-1：玩家单位读 `balance.combat.player`（EXP-066 重复踩坑闭环：不再写死 1000/150）
        let player = match balance() {
            Ok(b) => b.combat.player.unwrap_or(Unit { hp: 1000, atk: 150 }),
            Err(_) => Unit { hp: 1000, atk: 150 },
        };
        self.battle = BattleView::new(seed, player, tier);
        self.scene = Scene::Vs;
    }

    fn open_message(&mut self, kind: MsgKind) {
        self.msg_kind = Some(kind);
        self.msg_page = 0;
        self.msg_total = total_pages(kind);
        self.scene = Scene::Message;
    }
}

impl EconomyHost for GameApp {
    fn price_of(&self, gid: u32) -> Option<u32> {
        self.price_table.lookup(gid)
    }

    fn sell_price_of(&self, gid: u32) -> Option<u32> {
        let b = balance().ok()?;
        self.price_table
            .lookup(gid)
            .map(|buy| ti_core::rules::trade::sell_price(buy, b.trade.markup))
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
        })
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
        if data.character > 0 && data.character <= 5 {
            self.character = Character::from_index(data.character - 1);
            self.selection.character = self.character;
        }
        if data.map_slot < 3 {
            self.map_slot = MapSlot::from_index(data.map_slot);
            self.selection.map_slot = self.map_slot;
        }
        self.gold = data.gold;
        self.bank = Bank::new(data.gold);
        if self.selection.has_character() && self.selection.has_map() {
            GameApp::enter_main_map(self);
            self.selection.completed = true;
        } else {
            self.scene = Scene::SelectRole;
        }
    }

    fn in_map(&self) -> bool {
        self.map_pos.is_some()
    }

    fn in_battle(&self) -> bool {
        self.battle.is_some()
    }
}

impl SceneHost for GameApp {}
