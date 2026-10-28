//! 环岛赛道：进入主地图 / 摇骰前进 / 落格结算（`rules::board` 接线）。
//!
//! 拆分自 `game_app.rs`（S20.0b）：赛道玩法与场景表现分离——本模块只管状态与结算，
//! 画法在 `scenes/main_map.rs`（`docs/business/progression.md` §环岛赛道）。

use ti_core::data;
use ti_core::data::balance;
use ti_core::domain::economy::Bank;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{GridPos, MapTerrain};
use ti_core::domain::selection::MapSlot;
use ti_core::rules::board::{BoardSession, BossTrigger, Landing, Track, Turn};
use ti_core::rules::cards_draw;
use ti_core::rules::main_map::MapSession;

use super::{GameApp, Scene};
use crate::scenes::EconomyHost;

impl GameApp {
    /// 落格提示条的显示时长（帧，S20.0c 单一入口 `toast()` 引用）。
    const TOAST_TICKS: u32 = 120;
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
    pub(crate) fn enter_main_map_at(&mut self, start_override: Option<GridPos>) -> bool {
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
    pub(crate) fn enter_main_map(&mut self) -> bool {
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
        let landing = turn.landing;
        self.map_pos = Some(session.pos());
        // 绕圈补给（大富翁「经过起点」语义）
        let lap_delta = laps_after.saturating_sub(laps_before);
        if lap_delta > 0 {
            let bonus = b.board.lap_bonus_gold.saturating_mul(lap_delta);
            self.gold = self.gold.saturating_add(bonus).min(b.economy.gold_max);
        }
        self.settle_board_landing(landing, b);
        self.apply_board_boss(turn.boss, laps_after, b);
        Some(turn)
    }
    /// 落格提示条 + 音效（提示条三连的**单一入口**）。
    ///
    /// S20.0c：`board_toast` + `board_toast_ticks = 120` + `last_sfx` 三连在代码里出现 7 处
    /// （复制粘贴 ⇒ 改时长/音效必漏一处）⇒ 收敛为单一方法。
    /// 时长常量 `TOAST_TICKS` = 120 帧（出图取证用 `hold_board_toast` 顶住衰减）。
    fn toast(&mut self, text: String, sfx: Option<Sfx>) {
        self.board_toast = Some(text);
        self.board_toast_ticks = Self::TOAST_TICKS;
        if let Some(s) = sfx {
            self.last_sfx = Some(s);
        }
    }
    /// 落格结算（`Landing` → 金 / HP / 卡 / 提示条；数值全走 `balance.board`）。
    fn settle_board_landing(&mut self, landing: Landing, b: &data::Balance) {
        match landing {
            Landing::Battle => {
                self.card_state.hp = self.card_state.hp.saturating_add(b.board.battle_damage);
                self.gold = self.gold.saturating_sub(b.board.battle_gold_loss);
                self.toast(
                    i18n_fmt("scene.main_map.toast.battle_fmt", &[&b.board.battle_damage]),
                    Some(Sfx::Invalid),
                );
            }
            Landing::Treasure => {
                self.gold = self.gold.saturating_add(b.board.treasure_gold).min(b.economy.gold_max);
                self.toast(
                    i18n_fmt("scene.main_map.toast.treasure_fmt", &[&b.board.treasure_gold]),
                    Some(Sfx::Press),
                );
            }
            Landing::City => {
                self.gold = self.gold.saturating_add(b.board.city_gold).min(b.economy.gold_max);
                self.toast(
                    i18n_fmt("scene.main_map.toast.city_fmt", &[&b.board.city_gold]),
                    Some(Sfx::Press),
                );
            }
            Landing::Port => {
                self.toast(
                    String::from(crate::i18n::sget("scene.main_map.toast.port")),
                    Some(Sfx::Press),
                );
            }
            Landing::Event | Landing::Card => {
                // 事件格 / 卡牌格 = **必抽卡**（两类落格行为一致；`balance.cards.event_chance`
                // 不参与赛道结算，仅用于 legacy 移动路径 `rules::main_map`）。
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
                        self.toast(
                            i18n_fmt("scene.main_map.toast.card_fmt", &[&card_name]),
                            Some(Sfx::Press),
                        );
                    }
                }
            }
            // BOSS 格本身无金/HP 结算（提示条由 `apply_board_boss` 给出：
            // 未满圈 = 未现 / 满圈 = 现身）；起点格与普通格同样无结算。
            Landing::Boss | Landing::Start | Landing::Plain => {}
            Landing::Territory => {
                // S28 领地格（`docs/business/territory.md` §4 三支分叉）：先显领地信息签
                // （岛名 i18n `isle.<id>`）；占领/交钱/攻占/升级的结算菜单 = S28.6/7 接线。
                let sp = self.board.as_ref().and_then(BoardSession::current).cloned();
                if let Some(sp) = sp {
                    let isle_name = sp.isle.as_deref().map_or_else(
                        || crate::i18n::sget("scene.main_map.node.territory").to_string(),
                        |isle| crate::i18n::sget(&format!("isle.{isle}")).to_string(),
                    );
                    self.toast(
                        i18n_fmt("scene.main_map.toast.territory_fmt", &[&isle_name]),
                        Some(Sfx::Press),
                    );
                }
            }
        }
    }
    /// BOSS 触发提示（S19.3：`Turn::boss` 三态 = 未遇 / 未现 / 现身）。
    fn apply_board_boss(&mut self, boss: BossTrigger, laps_after: u32, b: &data::Balance) {
        match boss {
            BossTrigger::None => {}
            BossTrigger::Locked => {
                let remain = b.board.boss_min_laps.saturating_sub(laps_after).max(1);
                self.toast(
                    i18n_fmt("scene.main_map.toast.boss_locked_fmt", &[&remain]),
                    Some(Sfx::Invalid),
                );
            }
            BossTrigger::Engaged => {
                // 位置已在规则层停到 BOSS 格（`BoardSession::step` 拦停），此处只给提示条
                self.toast(String::from(crate::i18n::sget("scene.main_map.toast.boss")), None);
            }
        }
    }
    /// 保持落格提示条常驻（出图取证用）。
    ///
    /// S19.7 D5：落格提示条按帧衰减（120 帧），而离屏截图的**实际捕获帧**受贴图异步
    /// 加载影响（远大于 120 帧）⇒ 提示条在出图里已过期、无法作为验收证据。
    /// 本方法把衰减计数顶到极大值，使提示条在截图帧仍在（只影响出图模式）。
    pub fn hold_board_toast(&mut self) {
        if self.board_toast.is_some() {
            self.board_toast_ticks = u32::MAX;
        }
    }
}
/// i18n 取文案（重复模式收敛：`audit_code` 重复模式 >5 即违例；单一入口）。
fn i18n_fmt(key: &str, args: &[&dyn std::fmt::Display]) -> String {
    crate::i18n::sgetf(key, args)
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
