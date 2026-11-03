//! 环岛赛道：进入主地图 / 摇骰前进 / 落格结算（`rules::board` 接线）。
//!
//! 拆分自 `game_app.rs`（S20.0b）：赛道玩法与场景表现分离——本模块只管状态与结算，
//! 画法在 `scenes/main_map.rs`（`docs/business/progression.md` §环岛赛道）。

use ti_core::data;
use ti_core::data::balance;
use ti_core::domain::economy::Bank;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{GridPos, MapTerrain};
use ti_core::domain::selection::Character;
use ti_core::domain::selection::MapSlot;
use ti_core::rules::board::{BoardSession, BossTrigger, Landing, Track, Turn};
use ti_core::rules::cards_draw;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::selection;
use ti_core::rules::territory::{
    DungeonOutcome, SeatIdx, SeatPool, SettleAction, TerritoryApply, TerritoryRules,
    TerritoryState, TurnCycle,
};
use ti_core::rules::territory_v2::{self, SeatState};

use super::{GameApp, MatchOutcome, Scene};
use crate::scenes::EconomyHost;

impl GameApp {
    /// 落格提示条的显示时长（帧，S20.0c 单一入口 `toast()` 引用）。
    const TOAST_TICKS: u32 = 120;

    /// v3：某席的赛道会话（只读）。
    ///
    /// v3 每席独立会话（§1 独立性 / D2 修复）：人类席不再共用一个会话。
    ///
    /// **存储约定**：`board` 字段持有**当前行动席**的会话（v2 沿用，渲染层零改动）；
    /// `seat_sessions[i]` 持有**非当前席**的会话。切换席位时二者互换（不克隆 ⇒ 无漂移）。
    #[must_use]
    pub fn seat_session(&self, seat: u8) -> Option<&BoardSession> {
        let cur = self.turn_cycle.current;
        if seat == cur {
            self.board.as_ref()
        } else {
            self.seat_sessions.get(seat as usize).and_then(|s| s.as_ref())
        }
    }

    /// v3：切换到某席（热座交接 / 回合推进）——当前席会话存入 `seat_sessions`，
    /// 目标席会话载入 `board`。
    ///
    /// **互换而非克隆** ⇒ `board` 与 `seat_sessions` 永不漂移（D2 的根因级修复）。
    pub(crate) fn switch_to_seat(&mut self, seat: u8) {
        let cur = self.turn_cycle.current;
        if cur == seat {
            return;
        }
        // 当前席落盘
        if let Some(sess) = self.board.take() {
            if let Some(slot) = self.seat_sessions.get_mut(cur as usize) {
                *slot = Some(sess);
            }
        }
        // 目标席载入
        self.board = self.seat_sessions.get_mut(seat as usize).and_then(std::option::Option::take);
        self.turn_cycle.current = seat;
    }

    /// v3：当前行动席的金币（**唯一金币账本** `seat_gold[]` 的一次读取）。
    ///
    /// D3 修复：v2 有 `self.gold`（主席镜像）与 `seat_gold[5]` 两套账本 ⇒
    /// AI 收过路费只写 `seat_gold`，玩家 HUD 的 `self.gold` 不动（实测 §1.1 D3）。
    /// v3 起 `self.gold` **只作当前行动席的镜像**，所有改动走 `seat_gold[]`。
    #[must_use]
    pub fn current_seat_gold(&self) -> u32 {
        self.seat_gold.get(self.turn_cycle.current as usize).copied().unwrap_or(0)
    }

    /// v3：给当前行动席加/减金币（写唯一账本 + 同步镜像）。
    pub(crate) fn add_current_seat_gold(&mut self, delta: u32) {
        let seat = self.turn_cycle.current as usize;
        if let Some(g) = self.seat_gold.get_mut(seat) {
            *g = g.saturating_add(delta);
        }
        self.sync_gold_mirror();
    }

    /// v3：把 `self.gold` 镜像同步为当前行动席的账本值（唯一写入口）。
    ///
    /// 单一同步点 ⇒ 镜像永不漂移（D3 根因消除）。
    pub(crate) fn sync_gold_mirror(&mut self) {
        self.gold = self.current_seat_gold();
    }

    /// v3：破产检查 + 终局判定（§1.2 / §4.4）。
    ///
    /// - 席位金币不足以支付 ⇒ 破产（`territory_v2::bankrupt`：金币归零 + 占领格清空）；
    /// - 破产席从 `TurnCycle` 标记出局（后续轮转跳过）；
    /// - 只剩一个存活席 ⇒ 胜利；全部人类席破产 ⇒ 失败；无人存活 ⇒ 平局。
    ///
    /// 返回 `true` = 对局已结束（`match_outcome` 已置位）。
    pub(crate) fn check_match_end(&mut self, bal: &data::Balance) -> bool {
        if self.match_outcome.is_some() {
            return true;
        }
        let Some(mut terr) = self.territory.take() else {
            return false;
        };
        // ① 破产判定：金币不足以支付当前应付 ⇒ 出局。
        let mut bankrupt_seats: Vec<u8> = Vec::new();
        for seat in 0..5u8 {
            if !self.turn_cycle.is_playing(seat) {
                continue;
            }
            let gold = self.seat_gold.get(seat as usize).copied().unwrap_or(0);
            // 判定口径：金币为 0 且无舰队人手（无法再生产）⇒ 破产出局。
            // （领地过路费按 `settle` 已扣到 0 为止；此处判「归零 + 无舰队」= 彻底出局。）
            let crew = self.seat_crew.get(seat as usize).copied().unwrap_or(0);
            if gold == 0 && crew == 0 {
                bankrupt_seats.push(seat);
            }
        }
        for seat in &bankrupt_seats {
            let mut sstate = territory_v2::SeatState::new(
                self.seat_gold[*seat as usize],
                self.seat_crew[*seat as usize],
                self.seat_arms[*seat as usize],
                0,
            );
            territory_v2::bankrupt(&mut sstate, &mut terr, *seat);
            self.seat_gold[*seat as usize] = 0;
            self.seat_crew[*seat as usize] = 0;
            self.turn_cycle.mark_bankrupt(*seat);
            let name = ai_seat_name(*seat, bal);
            self.toast(i18n_fmt("scene.main_map.toast.bankrupt_fmt", &[&name]), Some(Sfx::Invalid));
        }
        self.territory = Some(terr);

        // ② 终局判定（只看**在局**席位）。
        let active: Vec<bool> = (0..5u8).map(|s| self.turn_cycle.is_active_seat(s)).collect();

        // S36.3 飞轮 R2-F1：标记破产后若当前行动席已出局，推进 `current` 到下一存活在局席
        // （防下一轮 `spin_board` 用死席会话操作——`mark_bankrupt` 不清会话，`board` 仍指向死席）。
        if !self.turn_cycle.is_alive(self.turn_cycle.current) {
            self.turn_cycle.advance();
        }
        let states: Vec<territory_v2::SeatState> = (0..5u8)
            .map(|s| {
                let mut st = territory_v2::SeatState::new(
                    self.seat_gold[s as usize],
                    self.seat_crew[s as usize],
                    self.seat_arms[s as usize],
                    0,
                );
                // 同步 `turn_cycle.alive`（`mark_bankrupt` 已修改）→ `SeatState.alive`
                st.alive = self.turn_cycle.is_alive(s);
                st
            })
            .collect();
        let human_seats: Vec<u8> = (0..5u8)
            .filter(|s| self.turn_cycle.is_player_seat(*s) && self.turn_cycle.is_active_seat(*s))
            .collect();
        // S36.3 飞轮 F-3.1/F-9.2：胜负判定顺序对齐 `territory.md` §4.4 规格——
        // **先判 Defeat（全部人类席破产 ⇒ 玩家失败）**，再判 Victory（存活席 = 1 且为在局席）。
        // 原实现先判 Victory 导致「全人类破产 + 1 存活 AI 席」被误判为 AI 胜（对玩家 = 败）。
        if territory_v2::players_defeated(&states, &human_seats) {
            self.match_outcome = Some(MatchOutcome::Defeat);
            // S36.3 飞轮 R2-F3：终局置位 ⇒ 清热座交接待确认（对局已结束，交接无意义；
            // 不清则 Confirm 需按两次——第一次清 handoff 第二次才触发终局面板回主菜单）。
            self.handoff_pending = None;
            return true;
        }
        if let Some(winner) = territory_v2::victory_seat_active(&states, &active) {
            self.match_outcome = Some(MatchOutcome::Victory { seat: winner });
            self.handoff_pending = None;
            return true;
        }
        // 全部在局席位都破产 ⇒ 平局（无胜者）。
        let any_alive = states
            .iter()
            .enumerate()
            .any(|(i, s)| s.alive && active.get(i).copied().unwrap_or(false));
        if !any_alive && !active.is_empty() && active.iter().any(|a| *a) {
            self.match_outcome = Some(MatchOutcome::Draw);
            return true;
        }
        false
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
        // S27.5 D1：首次进场 map_slot=None（shot 导航/新开局路径），兜底设 M01；
        // 否则 main_map 渲染侧 host.map_slot() 永远 None ⇒ 中央据点岛不画。
        if self.map_slot.is_none() {
            self.map_slot = MapSlot::from_index(0);
        }
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
        // S28.7 领地系统初始化（territory.md §1：主玩家 = 已选角色，其余自动 AI 席；
        // 领地账本 = 该图赛道全无主；AI 席各持独立赛道会话，位置 = 起点格）
        self.init_territory(&map_def.id, b);
        // 玩家位置 = 当前赛道格（S19 环岛赛道）：无赛道时回退 balance `player_start`。
        self.map_pos =
            self.board.as_ref().map(BoardSession::pos).or(Some(GridPos::new(start[0], start[1])));
        // v3：`seat_gold[]` 已是唯一金币账本（`init_territory` 铺好）⇒ 这里只同步镜像。
        self.sync_gold_mirror();
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
        self.pending_dungeon = None; // S28.6：新摇骰 = 旧副本入口失效（落格后按新落格重设）
        self.pending_siege = None; // S33.5：新摇骰 = 旧攻占选择失效（同上）
        // v3：热座交接未确认 ⇒ 不允许摇骰（§1 热座 + §1.2 隐藏信息）。
        if self.handoff_pending.is_some() {
            return None;
        }
        // 对局已结束（`match_outcome` 在位）⇒ 摇骰 no-op（§4.4 终局后不再行动，飞轮 F-6.3）。
        if self.match_outcome.is_some() {
            return None;
        }
        let b = match balance() {
            Ok(b) => b,
            Err(_e) => {
                #[cfg(feature = "stats")]
                eprintln!("[stats] spin_board: balance.json 加载失败（硬错误，不 fallback）");
                return None;
            }
        };
        // v3：摇骰作用于**当前行动席**的独立会话（`board` 持有当前席会话；人类席不再共用）。
        let seat = self.turn_cycle.current;
        // S36.3 飞轮 R2-F1：当前席已出局（破产）且 `advance` 后仍无存活在局席 ⇒ 无会话可摇，
        // 返回 `None`（对局实际已终局，`match_outcome` 由 `check_match_end` 在下一结算点置位）。
        if !self.turn_cycle.is_playing(seat) {
            return None;
        }
        let sides = b.board.dice_sides.min(b.board.max_steps);
        let (turn, laps_before, laps_after, pos) = {
            let session = self.board.as_mut()?;
            let laps_before = session.laps();
            let turn = session.step(&mut self.rng, sides, b.board.boss_min_laps);
            let laps_after = session.laps();
            let pos = session.pos();
            (turn, laps_before, laps_after, pos)
        };
        let landing = turn.landing;
        self.map_pos = Some(pos);
        // 绕圈补给（大富翁「经过起点」语义）——走唯一金币账本 `seat_gold[]`。
        let lap_delta = laps_after.saturating_sub(laps_before);
        if lap_delta > 0 {
            let bonus = b.board.lap_bonus_gold.saturating_mul(lap_delta);
            self.add_current_seat_gold(bonus);
            let cap = b.economy.gold_max;
            if let Some(g) = self.seat_gold.get_mut(seat as usize) {
                *g = (*g).min(cap);
            }
            self.sync_gold_mirror();
        }
        self.settle_board_landing(landing, b);
        self.apply_board_boss(turn.boss, laps_after, b);
        // S28.7：玩家回合结束 ⇒ 轮转到后续 AI 席自动行动（§6），回到玩家席
        self.run_ai_turns(b);
        // v3：破产/胜负判定（§1.2）——接线后对局才可能结束（D4）。
        self.check_match_end(b);
        Some(turn)
    }

    /// S28.7 领地/席位初始化（§1：席位池 = 已选人类席 + 可配 AI 席；账本全无主；
    /// **每席独立赛道会话**（v3 D2 修复）；金币 = 初始金）。`enter_main_map_at` / 读档共用。
    fn init_territory(&mut self, map_id: &str, bal: &data::Balance) {
        let db = match data::board() {
            Ok(db) => db,
            Err(_e) => {
                self.territory = None;
                return;
            }
        };
        let Some(map_def) = db.map(map_id) else {
            self.territory = None;
            return;
        };
        let Ok(terr) = TerritoryState::new(map_def, bal) else {
            eprintln!("enter_main_map: 领地账本构建失败（硬错误，不 fallback）");
            self.territory = None;
            return;
        };
        self.territory = Some(terr);
        // 席位池：人类席 = 已勾选角色（`seat_marks`）；AI 席数可配（`seat_ai_count`）。
        // 空 `seat_marks` = 旧单选流回退（1 人类 + 4 AI，§1 v2 兼容路径）。
        let player_idx = self.selection.character.map_or(0, Character::index);
        let pool = if self.seat_marks.is_empty() {
            SeatPool::build_with_ai(&[player_idx.min(4)], self.seat_ai_count.unwrap_or(4))
        } else {
            let max_ai = bal.selection.max_ai_seats;
            selection::seat_pool_with_ai(&self.seat_marks, self.seat_ai_count.unwrap_or(0), max_ai)
        };
        let mut cycle = TurnCycle::new(&pool);
        // 当前席 = 最低序号**人类席**（确定性开局，§1 顺序 = index 升序）
        cycle.current = pool
            .player_seats()
            .first()
            .copied()
            .unwrap_or(u8::try_from(player_idx.min(4)).unwrap_or(0));
        self.turn_cycle = cycle;
        // v3：金币 = **唯一账本** `seat_gold[]`（每席独立，D3）；`self.gold` 只是当前席镜像。
        self.seat_gold = [bal.economy.initial_gold; 5];
        // S33.4 席位舰队人手/武器初始（`territory.md` §11 U4：船 1 级 ≈ 20 水手 + 5 武器；
        // `balance.manpower.initial_crew/arms` 单一事实源；驻军初始 0）。
        self.seat_crew = [bal.manpower.initial_crew; 5];
        self.seat_arms = [bal.manpower.initial_arms; 5];
        // v3：**每席独立赛道会话**（含人类席）——v2 只有 AI 席有会话，人类席共用 `board`
        // ⇒ 两个人类席走同一条路（实测 D2）。各席同起点、同赛道，独立位置/圈数。
        //
        // 存储约定：`board` 持有**当前行动席**的会话；`seat_sessions[i]` 存其余席。
        let Some(bd) = self.board.as_ref() else {
            self.seat_sessions = core::array::from_fn(|_| None);
            return;
        };
        let track = bd.track().clone();
        let start_index = bd.index();
        let cur = self.turn_cycle.current;
        self.seat_sessions = core::array::from_fn(|i| {
            let seat = u8::try_from(i).unwrap_or(0);
            if seat == cur || !pool.is_active(seat) {
                None
            } else {
                Some(BoardSession::new(track.clone(), start_index))
            }
        });
        // 当前席会话 = 全席同起点的独立会话（`board` 已由 `enter_main_map_at` 建好，位置一致）。
        self.board = Some(BoardSession::new(track, start_index));
    }

    /// S28.7 AI 连续段（§6）：从当前席位轮转，AI 席自动行动（摇骰 + 前进 + 落格结算），
    /// 直到下一个玩家席（单玩家局 = 4 个 AI 连动后回玩家）。
    ///
    /// v3：AI 席用**自己的**赛道会话（`seat_sessions[seat]`），不再与人类席共用；
    /// 破产出局席被 `TurnCycle` 跳过；轮到下一人类席 ⇒ 置 `handoff_pending`（须显式确认）。
    fn run_ai_turns(&mut self, bal: &data::Balance) {
        let Some(mut terr) = self.territory.take() else {
            return;
        };
        let rules = TerritoryRules::new(bal);
        let sides = bal.board.dice_sides.min(bal.board.max_steps);
        // 玩家回合刚结束 ⇒ 先轮转离开当前（玩家）席；多玩家局 = 停在下一玩家席（热座交接）
        self.turn_cycle.advance();
        for _ in 0..5 {
            let seat = self.turn_cycle.current;
            if self.turn_cycle.is_player_seat(seat) {
                break;
            }
            // v3：AI 席用**自己的**赛道会话（人类席不再共用 `board`）。
            let Some(sess) = self.seat_sessions[seat as usize].as_mut() else {
                self.turn_cycle.advance();
                continue;
            };
            let roll = sess.spin(&mut self.rng, sides);
            sess.advance(roll);
            let idx = u32::try_from(sess.index()).unwrap_or(0);
            self.settle_ai_landing(bal, &mut terr, &rules, seat, idx);
            self.turn_cycle.advance();
        }
        self.territory = Some(terr);

        // v3 热座交接（§1 + §1.2）：轮到人类席 ⇒ 切到该席会话 + 置 `handoff_pending`，
        // **必须显式确认**才允许该席操作（防误操作 + 遮蔽他人情报）。
        // 单人类席（v2 兼容流）不置 handoff_pending，保持 0 行为变化。
        let seat = self.turn_cycle.current;
        if self.turn_cycle.is_playing(seat) && self.turn_cycle.is_player_seat(seat) {
            // S36.3 飞轮 F-6.1/F-10.1：交接判定用 `is_playing`（在局 ∧ 存活）而非
            // `is_player_seat`（只判席位类型）——已破产玩家席不再触发交接（防死锁）。
            let active_players = (0..5u8)
                .filter(|s| self.turn_cycle.is_player_seat(*s) && self.turn_cycle.is_alive(*s))
                .count();
            self.switch_to_seat(seat);
            if active_players >= 2 {
                self.handoff_pending = Some(seat);
                let hint = crate::i18n::sget("scene.main_map.toast.handoff");
                self.last_ai_toast = if hint.contains("scene.main_map") {
                    Some("热座交接：轮到玩家席（按确认摇骰开始回合）".to_string())
                } else {
                    Some(hint.to_string())
                };
            } else {
                self.handoff_pending = None;
            }
        } else {
            self.handoff_pending = None;
        }
    }

    /// S28.7 席位金币入账（钳制 `balance.economy.gold_max`；§1 席位独立经济）。
    pub(crate) fn credit_seat_gold(&mut self, seat: SeatIdx, amount: u32, bal: &data::Balance) {
        let idx = seat as usize;
        self.seat_gold[idx] = self.seat_gold[idx].saturating_add(amount).min(bal.economy.gold_max);
    }

    /// S28.7 AI 落格结算（§6：与玩家同一分支表，策略只选分支；战斗类 = 阈值确定性：
    /// 无主 = 副本自动胜利（U1 副本场景未建，AI 走确定性占领）；攻占 = 策略阈值通过 ⇒ 胜利；
    /// 交钱 / 产出 / 加固 = 直接账本变更 + 提示签条（主地图可见反馈））。
    #[allow(clippy::too_many_lines)] // S28.7 AI 落格分支（战场/副本/交钱/加固）同表，拆分会割裂结算语义
    fn settle_ai_landing(
        &mut self,
        b: &data::Balance,
        terr: &mut TerritoryState,
        rules: &TerritoryRules<'_>,
        seat: u8,
        idx: u32,
    ) {
        let gold = self.seat_gold[seat as usize];
        let crew = self.seat_crew[seat as usize];
        let arms = self.seat_arms[seat as usize];
        // S33.6：v2 AI 决策用**兵力模型**（`ai_policy_v2`：强舰队 → Siege / 弱 → PayToll），
        // 非旧 `attack_threshold × base` 纯阈值（攻守都耗兵力，须真实比较）。
        let action = territory_v2::ai_policy_v2(b, terr, seat, idx, gold, crew, arms);
        let name = ai_seat_name(seat, b);
        let isle_name = terr
            .isle_cells_by_index(idx)
            .map_or(String::new(), |isle| crate::i18n::sget(&format!("isle.{isle}")).to_string());
        match action {
            SettleAction::EnterDungeon => {
                let bonus = terr.apply_dungeon(rules, idx, seat, DungeonOutcome::Win);
                self.credit_seat_gold(seat, bonus, b);
                self.last_ai_toast = Some(i18n_fmt(
                    "scene.main_map.toast.ai_dungeon_fmt",
                    &[
                        &name as &dyn std::fmt::Display,
                        &isle_name as &dyn std::fmt::Display,
                        &bonus as &dyn std::fmt::Display,
                    ],
                ));
            }
            SettleAction::PayToll => {
                let Some(owner) = terr.ownership(idx).and_then(|o| o.owner) else {
                    return;
                };
                if owner == seat {
                    return;
                }
                let toll = terr.apply_pay_toll(rules, idx, seat);
                self.seat_gold[seat as usize] = self.seat_gold[seat as usize].saturating_sub(toll);
                self.credit_seat_gold(owner, toll, b);
                let owner_name = ai_seat_name(owner, b);
                self.last_ai_toast = Some(i18n_fmt(
                    "scene.main_map.toast.ai_toll_fmt",
                    &[
                        &name as &dyn std::fmt::Display,
                        &owner_name as &dyn std::fmt::Display,
                        &toll as &dyn std::fmt::Display,
                    ],
                ));
            }
            SettleAction::Siege => {
                // S33.6：v2 攻占 = 一次性确定性结算（`territory_v2::assault`，兵力模型），
                // 非旧 `apply_siege(..., Win)` 纯胜利（不耗兵力）。AI 战后驻军自定 = 战后舰队 1/3 守。
                let prev = terr.ownership(idx).and_then(|o| o.owner);
                let new_garrison = crew / 3;
                let mut sstate = SeatState::new(gold, crew, arms, u32::from(self.boat_level()));
                sstate.position = idx;
                let outcome = territory_v2::assault(b, &mut sstate, terr, idx, seat, new_garrison);
                self.seat_gold[seat as usize] = sstate.gold;
                self.seat_crew[seat as usize] = sstate.crew;
                self.seat_arms[seat as usize] = sstate.arms;
                let won = matches!(outcome, territory_v2::AssaultOutcome::Win { .. });
                let owner_txt = prev.map_or_else(|| String::from("无主"), |o| ai_seat_name(o, b));
                self.last_ai_toast = Some(i18n_fmt(
                    if won {
                        "scene.main_map.toast.ai_siege_fmt"
                    } else {
                        "scene.main_map.toast.ai_siege_fail_fmt"
                    },
                    &[
                        &name as &dyn std::fmt::Display,
                        &owner_txt as &dyn std::fmt::Display,
                        &isle_name as &dyn std::fmt::Display,
                    ],
                ));
            }
            SettleAction::Collect => {
                let amt = terr.apply_collect(rules, idx, seat);
                self.credit_seat_gold(seat, amt, b);
                self.last_ai_toast = Some(i18n_fmt(
                    "scene.main_map.toast.ai_collect_fmt",
                    &[
                        &name as &dyn std::fmt::Display,
                        &isle_name as &dyn std::fmt::Display,
                        &amt as &dyn std::fmt::Display,
                    ],
                ));
            }
            SettleAction::Upgrade => {
                let level_before = terr.ownership(idx).map_or(1, |o| o.level.max(1));
                if terr.apply_upgrade(rules, idx, seat) {
                    let cost = rules
                        .upgrade_cost_at(terr.isle_cells_by_index(idx).unwrap_or(""), level_before);
                    self.seat_gold[seat as usize] =
                        self.seat_gold[seat as usize].saturating_sub(cost);
                    let level_after = terr.ownership(idx).map_or(level_before, |o| o.level);
                    self.last_ai_toast = Some(i18n_fmt(
                        "scene.main_map.toast.ai_upgrade_fmt",
                        &[
                            &name as &dyn std::fmt::Display,
                            &isle_name as &dyn std::fmt::Display,
                            &level_after as &dyn std::fmt::Display,
                        ],
                    ));
                }
            }
            SettleAction::Maxed => {}
        }
    }

    /// 落格提示条 + 音效（提示条三连的**单一入口**）。
    ///
    /// S20.0c：`board_toast` + `board_toast_ticks = 120` + `last_sfx` 三连在代码里出现 7 处
    /// （复制粘贴 ⇒ 改时长/音效必漏一处）⇒ 收敛为单一方法。
    /// 时长常量 `TOAST_TICKS` = 120 帧（出图取证用 `hold_board_toast` 顶住衰减）。
    pub(crate) fn toast(&mut self, text: String, sfx: Option<Sfx>) {
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
                // v3：金币走唯一账本 `seat_gold[]`（D3）。
                let loss = b.board.battle_gold_loss;
                let seat = self.turn_cycle.current as usize;
                if let Some(g) = self.seat_gold.get_mut(seat) {
                    *g = g.saturating_sub(loss);
                }
                self.sync_gold_mirror();
                self.toast(
                    i18n_fmt("scene.main_map.toast.battle_fmt", &[&b.board.battle_damage]),
                    Some(Sfx::Invalid),
                );
            }
            Landing::Treasure => {
                self.add_current_seat_gold(b.board.treasure_gold);
                let cap = b.economy.gold_max;
                let seat = self.turn_cycle.current as usize;
                if let Some(g) = self.seat_gold.get_mut(seat) {
                    *g = (*g).min(cap);
                }
                self.sync_gold_mirror();
                self.toast(
                    i18n_fmt("scene.main_map.toast.treasure_fmt", &[&b.board.treasure_gold]),
                    Some(Sfx::Press),
                );
            }
            Landing::City => {
                self.add_current_seat_gold(b.board.city_gold);
                let cap = b.economy.gold_max;
                let seat = self.turn_cycle.current as usize;
                if let Some(g) = self.seat_gold.get_mut(seat) {
                    *g = (*g).min(cap);
                }
                self.sync_gold_mirror();
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

            Landing::Boss | Landing::Start | Landing::Plain => {}
            Landing::Territory => {
                self.settle_territory_landing(b);
            }
        }
    }

    /// S28.6 领地格落格三支分叉（territory.md §4）：无主 → 副本守军战（切 vs，胜利占领在
    /// 战斗钩结算）；他人 → 交过路费；自己 → 升级加固（金足）/ 产出领取（金不足或满级）。
    fn settle_territory_landing(&mut self, balance: &data::Balance) {
        let Some(sess) = self.board.as_ref() else {
            return;
        };
        let Some(sp) = sess.current().cloned() else {
            return;
        };
        let Some(isle) = sp.isle.as_deref() else {
            return;
        };
        let cell = u32::try_from(sess.index()).unwrap_or(0);
        let Some(mut terr) = self.territory.take() else {
            return;
        };
        let rules = TerritoryRules::new(balance);
        let seat = self.turn_cycle.current;
        let my_name = self.selection.character.map_or_else(String::new, |ch| {
            crate::i18n::sget(&format!("role.name.{}", ch.label())).to_string()
        });
        let isle_name = crate::i18n::sget(&format!("isle.{isle}")).to_string();
        match terr.ownership(cell).and_then(|o| o.owner) {
            None => {
                // 无主 → 副本入口待确认（§4 玩家选择权：提示签条 + `Confirm` 进副本战；
                // 胜利占领 + 首占奖励在战斗钩 `battle_step` 结算）
                self.territory = Some(terr);
                self.pending_dungeon = Some(cell);
                self.toast(
                    i18n_fmt("scene.main_map.toast.dungeon_fmt", &[&isle_name]),
                    Some(Sfx::Press),
                );
            }
            Some(owner) if owner != seat => {
                // S33.5 玩家攻占 UI：落他人领地格 = 二选一（交钱 / 攻占），不自动交。
                // 主地图 `Confirm` = 攻占（进 VS 攻占战报），`Back` = 交过路费（§4.2 产出归所有方）。
                // AI 落他人格仍自动交（`settle_ai_landing` 分支，S28.7）。
                self.territory = Some(terr);
                self.pending_siege = Some((cell, owner));
                let owner_name = ai_seat_name(owner, balance);
                self.toast(
                    i18n_fmt(
                        "scene.main_map.toast.siege_choice_fmt",
                        &[
                            &owner_name as &dyn std::fmt::Display,
                            &isle_name as &dyn std::fmt::Display,
                        ],
                    ),
                    Some(Sfx::Press),
                );
            }
            Some(owner) => {
                // 自己 → 升级加固（金足）/ 产出领取（金不足或满级）（§4.1/§4.3）
                let level_before = terr.ownership(cell).map_or(1, |o| o.level.max(1));
                let cost = rules.upgrade_cost_at(isle, level_before);
                let afford = self.seat_gold[owner as usize] >= cost;
                // 先判金再应用（`apply_upgrade` 有状态变更，金不足不得先改账本）
                if afford && terr.apply_upgrade(&rules, cell, owner) {
                    self.seat_gold[owner as usize] =
                        self.seat_gold[owner as usize].saturating_sub(cost);
                    let level_after = terr.ownership(cell).map_or(level_before, |o| o.level);
                    self.toast(
                        i18n_fmt(
                            "scene.main_map.toast.ai_upgrade_fmt",
                            &[
                                &my_name as &dyn std::fmt::Display,
                                &isle_name as &dyn std::fmt::Display,
                                &level_after as &dyn std::fmt::Display,
                            ],
                        ),
                        Some(Sfx::Press),
                    );
                } else {
                    let amt = terr.apply_collect(&rules, cell, owner);
                    self.credit_seat_gold(owner, amt, balance);
                    self.toast(
                        i18n_fmt(
                            "scene.main_map.toast.ai_collect_fmt",
                            &[
                                &my_name as &dyn std::fmt::Display,
                                &isle_name as &dyn std::fmt::Display,
                                &amt as &dyn std::fmt::Display,
                            ],
                        ),
                        Some(Sfx::Press),
                    );
                }
                self.territory = Some(terr);
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
pub(super) fn i18n_fmt(key: &str, args: &[&dyn std::fmt::Display]) -> String {
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

/// S28.7 AI 席位名（`role.name.<character>` i18n；越界 = 空串）。
pub(super) fn ai_seat_name(seat: u8, _b: &data::Balance) -> String {
    Character::from_index(u32::from(seat)).map_or_else(String::new, |ch| {
        crate::i18n::sget(&format!("role.name.{}", ch.label())).to_string()
    })
}
