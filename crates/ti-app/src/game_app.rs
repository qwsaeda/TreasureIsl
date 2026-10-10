//! 可玩交互核心：把 `ti-core::rules` 的 13 个已验证玩法模块接进表现层
//! （`docs/gamebook.md` §2/§3/§5 接线规格）。
//!
//! 设计原则（`docs/gamebook.md` §7）：
//! - **不重抄数值**：壳层一切判定/数值走 `ti-core::rules`，本模块只编排 UI/输入/渲染；
//! - **零占位**：不保留任何临时脚手架；
//! - **RNG 单一源**：战斗/事件随机全走 `rules::rng::GameRng`（LCG，EXP-040 确定性）；
//! - **渲染复用**：`render_*` 走 [`menu_shell`] 的 `Framebuf`/`FramePresenter`/字形机制
//!   （`docs/gamebook.md` §4），各场景分发到同一帧缓冲，脏行带机制不变（S4.3）。
//!
//! 与 [`MenuShell`] 的关系：`GameApp` 取代 `MenuShell` 成为两壳共用的交互核心；
//! `MenuShell` 退化为 `GameApp` 在 `Index` 场景下的渲染子视图（帧缓冲/字形/布局常量
//! 复用，见 [`menu_shell`]）。

use std::collections::HashMap;

use ti_core::oracle::battle::{Side, Unit, oracle_battle_round, oracle_boss_hp};
use ti_core::oracle::build::{Bank, Building};
use ti_core::oracle::main_map::{Dir, GridPos, MapTerrain, SceneTarget, oracle_move};
use ti_core::oracle::menu::Sfx;
use ti_core::oracle::messages::{MsgKind, oracle_text_line};
use ti_core::oracle::saveload::SaveSlot;
use ti_core::oracle::selection::{Character, MapSlot, SelectionState};
use ti_core::oracle::trade::PriceTable;
use ti_core::rules::main_loop::SceneStack;
use ti_core::rules::rng::GameRng;

use crate::i18n;
use crate::menu_shell::{
    COLOR_BG, COLOR_LANDMARK, COLOR_PLAYER, COLOR_SELECTED, COLOR_TITLE_BAR, MenuShell,
};
use crate::sprite_atlas::SpriteAtlas;
use crate::touch::{TouchSample, layout};
use ti_platform::gfx::UploadMode;
use ti_platform::settings::Gesture;

/// 顶层场景（`docs/gamebook.md` §2，与 `oracle::menu::MenuScene` + `main_loop::SceneId` 对齐）。
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

/// 战斗会话封装（持有 `rules::battle::BattleSession` 的输入，壳层只编排）。
#[derive(Debug, Clone)]
pub struct BattleView {
    /// 玩家单位（HP/atk，来自角色）。
    pub player: Unit,
    /// BOSS 档 tier（0..=4，`oracle_boss_hp` 表）。
    pub boss_tier: u32,
    /// 战斗 seed（`GameRng` 派生，确定性）。
    pub seed: u32,
    /// 已战回合数。
    pub rounds: u32,
    /// 玩家当前 HP。
    pub player_hp: u32,
    /// BOSS 当前 HP。
    pub boss_hp: u32,
    /// 胜方（`None` = 未结束）。
    pub outcome: Option<Side>,
}

impl BattleView {
    /// 新建战斗视图（调用 `oracle_boss_hp` 初始化 BOSS HP，行为与 `rules::battle::BattleSession` 一致）。
    #[must_use]
    pub fn new(seed: u32, player: Unit, boss_tier: u32) -> Self {
        let boss_hp = oracle_boss_hp(boss_tier);
        let outcome = if boss_hp == 0 { Some(Side::Player) } else { None };
        Self { player, boss_tier, seed, rounds: 0, player_hp: player.hp, boss_hp, outcome }
    }

    /// 是否结束。
    #[must_use]
    pub fn finished(&self) -> bool {
        self.outcome.is_some()
    }

    /// 单回合推进（调用 `oracle_battle_round` 行为，壳层只更新本视图字段）。
    ///
    /// 已结束的会话不再推进（与 `BattleSession::step` 一致）。
    pub fn step(&mut self) {
        if self.outcome.is_some() {
            return;
        }
        let player = Unit { hp: self.player_hp, atk: self.player.atk };
        let enemy = Unit { hp: self.boss_hp, atk: 10 };
        let (ph, eh, winner) = oracle_battle_round(self.seed, player, enemy);
        self.player_hp = ph;
        self.boss_hp = eh;
        self.rounds += 1;
        self.seed = self.seed.wrapping_add(1);
        self.outcome = winner;
    }
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
    /// 场景管理器（`rules::main_loop::SceneStack`，状态码语义与 oracle 逐位一致）。
    pub scenes: SceneStack,
    /// 顶层场景。
    pub scene: Scene,
    /// 主菜单选中项（`Index` 场景）。
    pub menu_selected: usize,
    /// 已选角色（`None` = 未选）。
    pub character: Option<Character>,
    /// 已选地图（`None` = 未选）。
    pub map_slot: Option<MapSlot>,
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
    /// 贸易账本（`rules::trade` 行为镜像，`MainMap` 场景时有效）。
    pub gold: u32,
    /// 固定价格表（`oracle::trade::PriceTable`，`rules::trade` 行为）。
    pub price_table: PriceTable,
    /// 持有量（`(goods_id, qty)`，`rules::trade` 行为）。
    pub owned: Vec<(u32, u32)>,
    /// 建造管理器（`rules::build` 行为镜像）。
    pub bank: Bank,
    /// 建筑列表（`(kind, level)`，`rules::build` 行为）。
    pub buildings: Vec<(Building, u32)>,
    /// 战斗会话（`Vs` 场景时有效）。
    pub battle: Option<BattleView>,
    /// 全局 RNG（MSVC LCG，`rules::rng` 行为，EXP-040 确定性）。
    pub rng: GameRng,
    /// 全局 RNG seed（供战斗/事件随机，壳层播种）。
    pub rng_seed: u32,
    /// 消息会话（`rules::messages` 行为镜像，`Message` 场景时有效）。
    pub msg_kind: Option<MsgKind>,
    /// 当前消息页码（0 起，`rules::messages` 行为）。
    pub msg_page: u32,
    /// 当前消息总页数（`oracle_total_pages` 取，`rules::messages` 行为）。
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
                use ti_core::oracle::main_map::Dir;
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
                    changed = self
                        .handle(GameInput::Build(ti_core::oracle::build::Building::Castle))
                        || changed;
                }
                if k.landmark {
                    changed = self.handle(GameInput::Landmark(6)) || changed;
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
        let mut changed = false;
        match self.scene {
            Scene::Index => changed = self.touch_index(s),
            Scene::SelectRole => changed = self.touch_select_role(s),
            Scene::SelectMap => changed = self.touch_select_map(s),
            Scene::MainMap => changed = self.touch_main_map(s),
            Scene::Vs => changed = self.touch_vs(s),
            Scene::Message => changed = self.touch_message(s),
            Scene::Saveload => changed = self.touch_saveload(s),
            Scene::Intro => {
                self.scene = Scene::Index;
                changed = true;
            }
            Scene::Exit => {}
        }
        changed
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
            unlocked_maps: MapSlot::all().to_vec(),
            selection: SelectionState::new(),
            map_terrain: None,
            map_pos: None,
            map_landmarks: Vec::new(),
            gold: 0,
            price_table: PriceTable::placeholder(),
            owned: Vec::new(),
            bank: Bank::default(),
            buildings: Vec::new(),
            battle: None,
            rng: GameRng::new(0),
            rng_seed: 0,
            msg_kind: None,
            msg_page: 0,
            msg_total: 0,
            save_slots: vec![SaveSlot::default(); 3],
            last_sfx: None,
            battle_won: false,
            quit: false,
            last_dispatch: None,
            atlas: None,
            scene_cache: HashMap::new(),
            terrain_atlas: None,
            frame_delta_ms: 0,
            terrain_map: None,
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

    /// 取可变精灵图集（测试用）。
    #[must_use]
    pub fn atlas_mut(&mut self) -> Option<&mut SpriteAtlas> {
        self.atlas.as_mut()
    }

    #[cfg(feature = "hd-8bit")]
    #[must_use]
    pub fn atlas_hd_cache_size(&self) -> usize {
        self.atlas.as_ref().map_or(0, |a| a.hd_cache_size())
    }

    #[cfg(feature = "hd-8bit")]
    pub fn load_hd_pngs(&mut self, f1_to_rgba: impl IntoIterator<Item = (u32, Vec<Vec<u8>>)>) {
        if let Some(atlas) = self.atlas.as_mut() {
            atlas.load_hd_pngs(f1_to_rgba);
        }
    }

    /// 处理输入事件：更新内部 rules 状态 + 顶层场景跳转。
    ///
    /// 调用方随后调 `render` 刷新帧缓冲。返回是否发生了场景切换。
    pub fn handle(&mut self, input: GameInput) -> bool {
        match self.scene {
            Scene::Index => self.handle_index(input),
            Scene::SelectRole => self.handle_select_role(input),
            Scene::SelectMap => self.handle_select_map(input),
            Scene::MainMap => self.handle_main_map(input),
            Scene::Vs => self.handle_vs(input),
            Scene::Message => self.handle_message(input),
            Scene::Saveload => self.handle_saveload(input),
            Scene::Intro => {
                self.scene = Scene::Index;
                true
            }
            Scene::Exit => false,
        }
    }

    /// Index 场景：菜单项选择（`docs/gamebook.md` §3.1）。
    ///
    /// 菜单项 0..5（`spec/menu.md §a` case 0..=5）：
    /// - 0 新游戏 → `SelectRole`；1 继续 → `Saveload`；
    /// - 2/3/4 选项/说明 → `Message`（Help kind）；5 退出 → 置 `quit`。
    fn handle_index(&mut self, input: GameInput) -> bool {
        match input {
            GameInput::MoveUp => {
                self.menu_selected = self.menu_selected.wrapping_add(5) % 6;
                false
            }
            GameInput::MoveDown => {
                self.menu_selected = (self.menu_selected + 1) % 6;
                false
            }
            GameInput::Confirm => {
                self.last_sfx = Some(Sfx::Press);
                match self.menu_selected {
                    0 => {
                        self.selection = SelectionState::new();
                        self.character = None;
                        self.map_slot = None;
                        self.scene = Scene::SelectRole;
                        true
                    }
                    1 => {
                        self.scene = Scene::Saveload;
                        true
                    }
                    2..=4 => {
                        self.msg_kind = Some(MsgKind::Help);
                        self.msg_page = 0;
                        self.msg_total = 3;
                        self.scene = Scene::Message;
                        true
                    }
                    5 => {
                        self.quit = true;
                        self.scene = Scene::Exit;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// `SelectRole` 场景：角色选择（`docs/gamebook.md` §3.2，`rules::selection` 行为）。
    fn handle_select_role(&mut self, input: GameInput) -> bool {
        match input {
            GameInput::Prev => {
                let cur = self.character.map_or(0, Character::index);
                let n = if cur == 0 { 4 } else { cur - 1 };
                self.character = Character::from_index(n);
                self.selection.character = self.character;
                self.last_sfx = Some(Sfx::Press);
                false
            }
            GameInput::Next => {
                let cur = self.character.map_or(4, Character::index);
                let n = (cur + 1) % 5;
                self.character = Character::from_index(n);
                self.selection.character = self.character;
                self.last_sfx = Some(Sfx::Press);
                false
            }
            GameInput::Confirm => {
                self.selection.character = self.character;
                // 角色选定 → 进选图（地图未选，completed 仍 false）
                self.selection.completed =
                    self.selection.has_character() && self.selection.has_map();
                if self.selection.completed {
                    self.enter_main_map();
                    true
                } else {
                    self.scene = Scene::SelectMap;
                    true
                }
            }
            GameInput::Back => {
                self.scene = Scene::Index;
                true
            }
            _ => false,
        }
    }

    /// `SelectMap` 场景：地图选择（`docs/gamebook.md` §3.2，`rules::selection` 行为）。
    fn handle_select_map(&mut self, input: GameInput) -> bool {
        match input {
            GameInput::Prev => {
                let cur = self.map_slot.map_or(0, MapSlot::zero_index);
                let n = if cur == 0 { 2 } else { cur - 1 };
                self.map_slot = MapSlot::from_index(n);
                self.last_sfx = Some(Sfx::Press);
                false
            }
            GameInput::Next => {
                let cur = self.map_slot.map_or(2, MapSlot::zero_index);
                let n = (cur + 1) % 3;
                self.map_slot = MapSlot::from_index(n);
                self.last_sfx = Some(Sfx::Press);
                false
            }
            GameInput::Confirm => {
                let Some(slot) = self.map_slot else {
                    self.last_sfx = Some(Sfx::Invalid);
                    return false;
                };
                if !self.unlocked_maps.contains(&slot) {
                    self.last_sfx = Some(Sfx::Invalid);
                    return false;
                }
                self.selection.map_slot = Some(slot);
                // 角色+地图齐全 → 完成（`rules::selection` 行为：completed = 双选定）
                self.selection.completed =
                    self.selection.has_character() && self.selection.has_map();
                if self.selection.completed {
                    self.enter_main_map();
                    true
                } else {
                    self.last_sfx = Some(Sfx::Press);
                    false
                }
            }
            GameInput::Back => {
                self.scene = Scene::SelectRole;
                true
            }
            _ => false,
        }
    }

    /// 进入主地图（`docs/gamebook.md` §3.3）：初始化 `MapSession` + 贸易/建造会话。
    fn enter_main_map(&mut self) {
        // 地形：按 `map_slot` 取 m01/m02/m03（m03 = 15×15，其余 20×20）
        let gw = match self.map_slot {
            Some(MapSlot::M03) => 15,
            _ => 20,
        };
        let gh = gw;
        self.map_terrain = Some(MapTerrain::open(gw, gh));
        self.map_pos = Some(GridPos::new(gh / 2, gw / 2));
        self.map_landmarks = vec![(2, 2), (gh - 3, gw - 3)];
        // 贸易/建造：开局 1000 金 + 占位价格表
        self.gold = 1000;
        self.owned = Vec::new();
        self.bank = Bank::new(1000);
        self.buildings = Vec::new();
        // 选择完成标志
        self.selection.completed = self.selection.has_character() && self.selection.has_map();
        self.scene = Scene::MainMap;
        // 场景管理器状态码同步（`rules::main_loop` 行为）
        self.scenes.set_state(0);
    }

    /// `MainMap` 场景：移动/贸易/建造/事件分派（`docs/gamebook.md` §3.3）。
    fn handle_main_map(&mut self, input: GameInput) -> bool {
        match input {
            GameInput::Walk(dir) => {
                let Some(t) = self.map_terrain.clone() else {
                    return false;
                };
                let Some(pos) = self.map_pos else {
                    return false;
                };
                // `rules::main_map::MapSession::move_to` 行为（`oracle_move`）
                let next = ti_core::oracle::main_map::oracle_move(&t, pos, dir);
                if next.is_some() {
                    self.map_pos = next;
                }
                false
            }
            GameInput::Landmark(code) => {
                // `rules::main_map::MapSession::dispatch` 行为（`oracle_dispatch_event`）
                let target = ti_core::oracle::main_map::oracle_dispatch_event(code);
                self.last_dispatch = target;
                match target {
                    Some(SceneTarget::Vs) => {
                        // 战斗：建 `BattleView`（`GameRng` 取 seed + tier）
                        let seed = self.rng.next().unsigned_abs();
                        let tier = self.rng.in_range(5);
                        let player = Unit { hp: 200, atk: 30 };
                        self.battle = Some(BattleView::new(seed, player, tier));
                        self.scene = Scene::Vs;
                        true
                    }
                    Some(SceneTarget::Msgl | SceneTarget::Msgm | SceneTarget::Msgs) => {
                        let kind = match target {
                            Some(SceneTarget::Msgl) => MsgKind::Msgl,
                            Some(SceneTarget::Msgm) => MsgKind::Msgm,
                            _ => MsgKind::Msgs,
                        };
                        self.msg_kind = Some(kind);
                        self.msg_page = 0;
                        self.msg_total = 1;
                        self.scene = Scene::Message;
                        true
                    }
                    Some(SceneTarget::Card | SceneTarget::Bigcard) => {
                        // 卡片准备（`spec/main_map.md §d` 事件码 3/5）
                        self.msg_kind = Some(MsgKind::Msgm);
                        self.msg_page = 0;
                        self.msg_total = 1;
                        self.scene = Scene::Message;
                        true
                    }
                    None => false,
                }
            }
            GameInput::TradeBuy(gid) => {
                // `rules::trade::TradeLedger::buy` 行为（`oracle_buy`）
                if let Some(price) = self.price_table.lookup(gid) {
                    if self.gold >= price {
                        self.gold -= price;
                        match self.owned.iter_mut().find(|(id, _)| *id == gid) {
                            Some(entry) => entry.1 += 1,
                            None => self.owned.push((gid, 1)),
                        }
                        self.last_sfx = Some(Sfx::Press);
                    }
                }
                false
            }
            GameInput::TradeSell(gid) => {
                // `rules::trade::TradeLedger::sell` 行为（`oracle_sell`）
                let owned_qty = self.owned.iter().find(|(id, _)| *id == gid).map_or(0, |&(_, q)| q);
                if owned_qty > 0 {
                    if let Some(price) = self.price_table.lookup(gid) {
                        let gain = price * owned_qty;
                        self.gold = self.gold.saturating_add(gain);
                        self.owned.retain(|(id, _)| *id != gid);
                        self.last_sfx = Some(Sfx::Press);
                    }
                }
                false
            }
            GameInput::Build(kind) => {
                // `rules::build::BuildManager::build` 行为（`oracle_build`）
                let cost = kind.base_cost();
                if self.bank.gold >= cost {
                    self.bank.gold -= cost;
                    match self.buildings.iter_mut().find(|(k, _)| *k == kind) {
                        Some(entry) => entry.1 += 1,
                        None => self.buildings.push((kind, 1)),
                    }
                    self.last_sfx = Some(Sfx::Press);
                }
                false
            }
            GameInput::MapBack => {
                self.scene = Scene::Index;
                true
            }
            _ => false,
        }
    }

    /// Vs 场景：战斗回合推进（`docs/gamebook.md` §3.4，`rules::battle` 行为）。
    fn handle_vs(&mut self, input: GameInput) -> bool {
        match input {
            GameInput::Attack => {
                if let Some(b) = self.battle.as_mut() {
                    b.step();
                    if b.outcome.is_some() {
                        self.battle_won = matches!(b.outcome, Some(Side::Player));
                        if self.battle_won {
                            self.last_sfx = Some(Sfx::Victory);
                        }
                    }
                }
                false
            }
            GameInput::Back => {
                // 战斗结束/中途退出 → 回主菜单（`spec/battle.md §d`，状态码 25→0）
                self.scene = Scene::Index;
                self.scenes.set_state(0);
                true
            }
            _ => false,
        }
    }

    /// Message 场景：消息/帮助翻页（`docs/gamebook.md` §3.5，`rules::messages` 行为）。
    fn handle_message(&mut self, input: GameInput) -> bool {
        match input {
            GameInput::MsgNext => {
                // `rules::messages::MessageSession::next` 行为（到末页保持）
                if self.msg_page + 1 < self.msg_total {
                    self.msg_page += 1;
                }
                false
            }
            GameInput::MsgPrev => {
                // `rules::messages::MessageSession::prev` 行为（首页回末页循环）
                if self.msg_page == 0 {
                    self.msg_page = self.msg_total.saturating_sub(1);
                } else {
                    self.msg_page -= 1;
                }
                false
            }
            GameInput::MsgClose | GameInput::Back => {
                // 关闭 → 回场景源（`Message` 来自 `Index`/`MainMap`/`Vs`）
                self.scene = if self.map_pos.is_some() && self.battle.is_none() {
                    Scene::MainMap
                } else {
                    Scene::Index
                };
                self.msg_kind = None;
                true
            }
            _ => false,
        }
    }

    /// Saveload 场景：存读档（`docs/gamebook.md` §3.6，`rules::saveload` 行为）。
    fn handle_saveload(&mut self, input: GameInput) -> bool {
        match input {
            GameInput::LoadSave(slot) => {
                // `rules::saveload::SaveManager::read_slot` 行为：按槽位恢复状态
                let Some(s) = self.save_slots.get(slot) else {
                    return false;
                };
                if s.character > 0 && s.character <= 5 {
                    self.character = Character::from_index(s.character - 1);
                    self.selection.character = self.character;
                }
                if s.map_slot < 3 {
                    self.map_slot = MapSlot::from_index(s.map_slot);
                    self.selection.map_slot = self.map_slot;
                }
                self.gold = s.gold;
                self.bank = Bank::new(s.gold);
                // 读档 → 进主地图（若角色+地图齐全）
                if self.selection.has_character() && self.selection.has_map() {
                    self.enter_main_map();
                    self.selection.completed = true;
                } else {
                    self.scene = Scene::SelectRole;
                }
                self.last_sfx = Some(Sfx::Press);
                true
            }
            GameInput::WriteSave(slot) => {
                // `rules::saveload::SaveManager::write_slot` 行为：序列化 `SaveSlot`
                let ch = self.character.map_or(1, |c| c.index() + 1);
                let m = self.map_slot.map_or(0, MapSlot::zero_index);
                if self.save_slots.len() > slot {
                    self.save_slots[slot] = SaveSlot {
                        version: 2,
                        character: ch,
                        gold: self.gold,
                        map_slot: m,
                        position: self.map_pos.map_or(0, |p| p.index(20)),
                    };
                    self.last_sfx = Some(Sfx::Press);
                }
                false
            }
            GameInput::SaveloadClose | GameInput::Back => {
                // 关闭存读档子窗口 → 回主菜单
                self.scene = Scene::Index;
                true
            }
            _ => false,
        }
    }

    /// Index 场景触摸：点菜单行 = 选中该行并立即执行（`docs/modernization.md` §5）。
    fn touch_index(&mut self, s: TouchSample) -> bool {
        if s.gesture != Gesture::Tap {
            return false;
        }
        let Some(idx) = layout::menu_row_at(s.x, s.y) else { return false };
        self.menu_selected = idx;
        self.handle(GameInput::Confirm)
    }

    /// `SelectRole` 场景触摸：点某行 = 选中该角色并确认；下滑 = 返回。
    fn touch_select_role(&mut self, s: TouchSample) -> bool {
        match s.gesture {
            Gesture::Tap => {
                let Some(i) = layout::list_index_at(s.x, s.y, Character::all().len()) else {
                    return false;
                };
                self.character = Character::from_index(u32::try_from(i).unwrap_or(0));
                self.selection.character = self.character;
                self.handle(GameInput::Confirm)
            }
            Gesture::SwipeDown => self.handle(GameInput::Back),
            _ => false,
        }
    }

    /// `SelectMap` 场景触摸：点某行 = 选中该地图并确认；下滑 = 返回。
    ///
    /// 未解锁地图点按由 `handle_select_map` 的 `Confirm` 分支判 `Sfx::Invalid`。
    fn touch_select_map(&mut self, s: TouchSample) -> bool {
        match s.gesture {
            Gesture::Tap => {
                let Some(i) = layout::list_index_at(s.x, s.y, MapSlot::all().len()) else {
                    return false;
                };
                self.map_slot = MapSlot::from_index(u32::try_from(i).unwrap_or(0));
                self.handle(GameInput::Confirm)
            }
            Gesture::SwipeDown => self.handle(GameInput::Back),
            _ => false,
        }
    }

    /// `MainMap` 场景触摸：
    /// - 点格子 = 朝该格逐格走（每次点按走到能走的最远格，不穿墙）；
    /// - 点自己所在格 = 触发据点事件（`Landmark(code)`）；
    /// - 滑动 = 朝该方向走一步。
    fn touch_main_map(&mut self, s: TouchSample) -> bool {
        let Some(cur) = self.map_pos else { return false };
        match s.gesture {
            Gesture::Tap => {
                let Some(terrain) = self.map_terrain.clone() else { return false };
                let Some((row, col)) =
                    layout::grid_cell_at(s.x, s.y, terrain.grid_w, terrain.grid_h)
                else {
                    return false;
                };
                let target = GridPos::new(row, col);
                // 点自己所在格 = 触发据点事件（等价键盘 Space）
                if target == cur {
                    return self.handle(GameInput::Landmark(6));
                }
                // 朝目标逐格走：先走差值大的轴，撞墙/越界即停
                self.walk_towards(&terrain, target)
            }
            Gesture::SwipeUp => self.handle(GameInput::Walk(Dir::North)),
            Gesture::SwipeDown => self.handle(GameInput::Walk(Dir::South)),
            Gesture::SwipeLeft => self.handle(GameInput::Walk(Dir::West)),
            Gesture::SwipeRight => self.handle(GameInput::Walk(Dir::East)),
        }
    }

    /// 从当前位置朝 `to` 逐格移动：优先走差值大的轴，遇阻挡/越界即停（不穿墙）。
    ///
    /// 每步经 [`GameApp::handle`] 的 `Walk` 分支落地（保持与键盘路径同一实现），
    /// `oracle_move` 只用于合法性预检。单次点按最多 `MAX_TOUCH_WALK` 步
    /// （避免一次触摸跨越半张图）。
    fn walk_towards(&mut self, terrain: &MapTerrain, to: GridPos) -> bool {
        /// 单次点按的最大步数。
        const MAX_TOUCH_WALK: u32 = 6;
        let mut changed = false;
        for _ in 0..MAX_TOUCH_WALK {
            let Some(pos) = self.map_pos else { break };
            let dr = i64::from(to.row) - i64::from(pos.row);
            let dc = i64::from(to.col) - i64::from(pos.col);
            if dr == 0 && dc == 0 {
                break;
            }
            // 行差绝对值 ≥ 列差 → 走纵向，否则走横向
            let dir = if dr.unsigned_abs() >= dc.unsigned_abs() {
                if dr < 0 { Dir::North } else { Dir::South }
            } else if dc < 0 {
                Dir::West
            } else {
                Dir::East
            };
            // 预检：不可通行则停下（不穿墙、不越界）
            if oracle_move(terrain, pos, dir).is_none() {
                break;
            }
            changed |= self.handle(GameInput::Walk(dir));
        }
        changed
    }

    /// `Vs` 场景触摸：点任意处 = 攻击一回合；下滑 = 退出战斗。
    fn touch_vs(&mut self, s: TouchSample) -> bool {
        match s.gesture {
            Gesture::Tap | Gesture::SwipeUp | Gesture::SwipeLeft | Gesture::SwipeRight => {
                self.handle(GameInput::Attack)
            }
            Gesture::SwipeDown => self.handle(GameInput::Back),
        }
    }

    /// `Message` 场景触摸：左半屏 = 关闭；右半屏 = 下一页；
    /// 上滑 = 下一页；下滑 = 上一页（`docs/modernization.md` §5 翻页表）。
    fn touch_message(&mut self, s: TouchSample) -> bool {
        match s.gesture {
            Gesture::Tap => {
                if s.x < layout::MSG_SPLIT_X {
                    self.handle(GameInput::MsgClose)
                } else {
                    self.handle(GameInput::MsgNext)
                }
            }
            Gesture::SwipeUp => self.handle(GameInput::MsgNext),
            Gesture::SwipeDown => self.handle(GameInput::MsgPrev),
            _ => false,
        }
    }

    /// `Saveload` 场景触摸：点有档的槽 = 读档；点空槽 = 存档；下滑 = 关闭。
    fn touch_saveload(&mut self, s: TouchSample) -> bool {
        match s.gesture {
            Gesture::Tap => {
                let hit = (0..self.save_slots.len()).find(|&i| layout::slot_at(s.x, s.y, i));
                let Some(slot) = hit else { return false };
                // 有档（version > 0）→ 读档；空槽 → 存档
                if self.save_slots[slot].version > 0 {
                    self.handle(GameInput::LoadSave(slot))
                } else {
                    self.handle(GameInput::WriteSave(slot))
                }
            }
            Gesture::SwipeDown => self.handle(GameInput::SaveloadClose),
            _ => false,
        }
    }

    /// 渲染当前场景到帧缓冲（`docs/gamebook.md` §4，分发到各 `render_*`）。
    ///
    /// 调用方随后调 [`MenuShell::present`] 上屏。
    ///
    /// 若 `self.atlas` 与 `self.scene_cache` 已注入，则先 blit 该场景的
    /// 真实精灵（`.Map` 控件记录 → PCX），再叠加文字/高亮（占位逻辑不变）。
    pub fn render(&mut self) {
        let scene_name: &'static str = match self.scene {
            Scene::Index | Scene::Intro => "index.Map",
            Scene::SelectRole => "selectrole.Map",
            Scene::SelectMap => "selectmap.Map",
            Scene::MainMap => "main.Map",
            Scene::Vs => "vs.Map",
            Scene::Message => "msgs.Map",
            Scene::Saveload => "saveload.Map",
            Scene::Exit => "",
        };
        match self.scene {
            Scene::Index | Scene::Intro => self.render_index(),
            Scene::SelectRole => self.render_select_role(),
            Scene::SelectMap => self.render_select_map(),
            Scene::MainMap => self.render_main_map(),
            Scene::Vs => self.render_vs(),
            Scene::Message => self.render_message(),
            Scene::Saveload => self.render_saveload(),
            Scene::Exit => {
                self.shell.fb.clear(0x0000);
                return;
            }
        }
        // 真实精灵 blit（atlas + 场景缓存都齐备时）
        if let (Some(atlas), Some(scene)) = (self.atlas.as_mut(), self.scene_cache.get(scene_name))
        {
            if !scene.words.is_empty() {
                // S6.4：按本帧时长推进动画（多帧精灵 num>1 才受影响）
                atlas.advance_animation(self.frame_delta_ms);
                atlas.blit_scene(scene, &mut self.shell.fb);
            }
        }
    }

    /// Index 场景渲染（复用 `MenuShell::render` 菜单 + 标题 + 状态行机制）。
    fn render_index(&mut self) {
        self.shell.selected = self.menu_selected;
        self.shell.render();
    }

    /// `SelectRole` 场景渲染：角色头像占位 + 名称 + 选中高亮。
    fn render_select_role(&mut self) {
        self.shell.fb.clear(COLOR_BG);
        self.shell.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        self.shell.draw_text_row(i18n::sget("scene.select_role.title"), 40, 4);
        for (i, c) in Character::all().iter().enumerate() {
            let y = 50 + i32::try_from(i).unwrap_or(0) * 30;
            let sel = i == self.character.map_or(0, |c| c.index() as usize);
            if sel {
                self.shell.fill_rect(20, y - 10, 136, 26, COLOR_SELECTED);
            }
            self.shell.draw_text_row(c.label(), 60, y);
        }
        self.shell.draw_text_row(i18n::sget("scene.select_role.nav_hint"), 30, 210);
    }

    /// `SelectMap` 场景渲染：地图缩略 + 名称 + 选中高亮。
    fn render_select_map(&mut self) {
        self.shell.fb.clear(COLOR_BG);
        self.shell.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        self.shell.draw_text_row(i18n::sget("scene.select_map.title"), 40, 4);
        for (i, m) in MapSlot::all().iter().enumerate() {
            let y = 50 + i32::try_from(i).unwrap_or(0) * 30;
            let sel = i == self.map_slot.map_or(0, |m| m.zero_index() as usize);
            let locked = !self.unlocked_maps.contains(m);
            if sel && !locked {
                self.shell.fill_rect(20, y - 10, 136, 26, COLOR_SELECTED);
            }
            self.shell.draw_text_row(m.label(), 60, y);
            if locked {
                self.shell.draw_text_row(i18n::sget("scene.select_map.locked"), 130, y);
            }
        }
        self.shell.draw_text_row(i18n::sget("scene.select_map.nav_hint"), 30, 210);
    }

    /// `MainMap` 场景渲染：等距网格 + 玩家棋子 + 据点 + 金/持有量 HUD。
    fn render_main_map(&mut self) {
        self.shell.fb.clear(COLOR_BG);

        // S6.3：真实地形瓦片（等距投影 + 摄像机），无地形资源时保持占位背景
        let terrain_drawn = match (self.terrain_atlas.as_mut(), self.terrain_map.as_ref()) {
            (Some(atlas), Some(terrain)) => atlas.blit_map(terrain, &mut self.shell.fb) > 0,
            _ => false,
        };

        // HUD（标题条 + 金/持有量 + 提示）
        self.shell.fill_rect(0, 0, 176, 16, COLOR_TITLE_BAR);
        self.shell.draw_text_row(i18n::sget("scene.main_map.title"), 64, 2);
        let gold_str = i18n::sgetf("scene.main_map.hud.gold_fmt", &[&self.gold]);
        self.shell.draw_text_row(&gold_str, 4, 150);
        let own_str = i18n::sgetf("scene.main_map.hud.owned_fmt", &[&self.owned.len()]);
        self.shell.draw_text_row(&own_str, 4, 162);

        // 玩家棋子 + 据点（等距投影坐标，相机与地形一致）
        let gw = self.terrain_map.as_ref().map_or(20, |t| t.grid_w);
        let camera = self
            .terrain_map
            .as_ref()
            .map_or((0, 0), crate::terrain_atlas::TerrainAtlas::center_camera);
        let cx = i32::try_from(camera.0).unwrap_or(0);
        let cy = i32::try_from(camera.1).unwrap_or(0);
        if let Some(pos) = self.map_pos {
            if terrain_drawn {
                let iso_x = crate::terrain_atlas::TerrainAtlas::iso_x(pos.row, pos.col);
                let iso_y = crate::terrain_atlas::TerrainAtlas::iso_y(pos.row, pos.col, gw);
                self.shell.fill_rect(iso_x + cx - 2, iso_y + cy - 2, 5, 5, COLOR_PLAYER);
            } else {
                let px = i32::try_from(pos.col).unwrap_or(0) * 6 + 20;
                let py = i32::try_from(pos.row).unwrap_or(0) * 6 + 30;
                self.shell.fill_rect(px - 1, py - 1, 4, 4, COLOR_PLAYER);
            }
        }
        for (r, c) in &self.map_landmarks {
            if terrain_drawn {
                let iso_x = crate::terrain_atlas::TerrainAtlas::iso_x(*r, *c);
                let iso_y = crate::terrain_atlas::TerrainAtlas::iso_y(*r, *c, gw);
                self.shell.fill_rect(iso_x + cx - 1, iso_y + cy - 1, 3, 3, COLOR_LANDMARK);
            } else {
                let px = i32::try_from(*c).unwrap_or(0) * 6 + 20;
                let py = i32::try_from(*r).unwrap_or(0) * 6 + 30;
                self.shell.fill_rect(px - 1, py - 1, 3, 3, COLOR_LANDMARK);
            }
        }
        self.shell.draw_text_row(i18n::sget("scene.main_map.hint"), 4, 210);
    }

    /// `Vs` 场景渲染：战斗面板（双方 HP + 回合 + 结果）。
    fn render_vs(&mut self) {
        self.shell.fb.clear(COLOR_BG);
        self.shell.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        self.shell.draw_text_row(i18n::sget("scene.vs.title"), 64, 4);
        if let Some(b) = self.battle.as_ref() {
            let ph = i18n::sgetf("scene.vs.player_hp_fmt", &[&b.player_hp]);
            let bh = i18n::sgetf("scene.vs.boss_hp_fmt", &[&b.boss_hp]);
            let rd = i18n::sgetf("scene.vs.rounds_fmt", &[&b.rounds]);
            self.shell.draw_text_row(&ph, 10, 50);
            self.shell.draw_text_row(&bh, 10, 80);
            self.shell.draw_text_row(&rd, 10, 110);
            match &b.outcome {
                Some(Side::Player) => {
                    self.shell.draw_text_row(i18n::sget("scene.vs.victory"), 60, 150);
                }
                Some(Side::Enemy) => {
                    self.shell.draw_text_row(i18n::sget("scene.vs.defeat"), 60, 150);
                }
                None => self.shell.draw_text_row(i18n::sget("scene.vs.awaiting"), 50, 150),
            }
        } else {
            self.shell.draw_text_row(i18n::sget("scene.vs.empty"), 60, 50);
        }
        self.shell.draw_text_row(i18n::sget("scene.vs.hint"), 60, 210);
    }

    /// `Message` 场景渲染：当前页文本 + 页码。
    fn render_message(&mut self) {
        self.shell.fb.clear(COLOR_BG);
        self.shell.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        let kind = self.msg_kind.unwrap_or(MsgKind::Help);
        self.shell.draw_text_row(kind.label(), 60, 4);
        // 当前页文本（`rules::messages::oracle_text_line` 行为，`spec/messages.md §c`）
        let text = oracle_text_line(kind, self.msg_page);
        self.shell.draw_text_row(text, 10, 40);
        let page_str = format!("{}/{}", self.msg_page + 1, self.msg_total);
        self.shell.draw_text_row(&page_str, 60, 200);
        self.shell.draw_text_row(i18n::sget("scene.message.nav_hint"), 40, 210);
    }

    /// `Saveload` 场景渲染：槽位列表 + 金/进度摘要。
    fn render_saveload(&mut self) {
        self.shell.fb.clear(COLOR_BG);
        self.shell.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        self.shell.draw_text_row(i18n::sget("scene.saveload.title"), 60, 4);
        for (i, s) in self.save_slots.iter().enumerate() {
            let y = 40 + i32::try_from(i).unwrap_or(0) * 40;
            let has = s.version > 0;
            let has_str = if has { i18n::sget("slot.has") } else { i18n::sget("slot.empty") };
            let slot_str = i18n::sgetf("slot.label_fmt", &[&(i + 1), &has_str]);
            self.shell.draw_text_row(&slot_str, 10, y);
            if has {
                let g = i18n::sgetf("hud.gold_fmt", &[&s.gold]);
                self.shell.draw_text_row(&g, 100, y);
            }
        }
        self.shell.draw_text_row(i18n::sget("scene.saveload.nav_hint"), 30, 210);
    }
}
