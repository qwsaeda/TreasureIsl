//! 宿主 trait 接线：`SceneHost` / `MapHost` / `MenuHost` / ... 的实现。
//!
//! 拆分自 `game_app.rs`（S20.0b）：场景模块只经这些 trait 读写状态
//!（`docs/process/01_architecture.md` §2 依赖倒置）。

use ti_core::data;
use ti_core::data::balance;
use ti_core::domain::combat::{Side, Unit};
use ti_core::domain::economy::Building;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{GridPos, MapTerrain, SceneTarget};
use ti_core::domain::messages::MsgKind;
use ti_core::domain::persistence::{SaveSlot, SaveSlotV4Ext, SeatSave, TerritorySave};
use ti_core::domain::selection::{Character, MapSlot, SelectionState};
use ti_core::rules::board::{BoardSession, TrackSpace, Turn};
use ti_core::rules::cards;
use ti_core::rules::messages::total_pages;
use ti_core::rules::progression::{CLEARED_BIT, ProgressionState};
use ti_core::rules::territory::{
    DungeonOutcome, Ownership, TerritoryApply, TerritoryRules, TerritoryState,
};
use ti_core::rules::trade;

use super::{BattleSnapshot, BattleView, GameApp, GameInput, Scene};
use crate::menu_shell::MenuShell;
use crate::scenes::SceneKind;
use crate::scenes::{
    EconomyHost, MapHost, MenuHost, NavHost, RenderHost, SceneHost, SelectionHost, SessionHost,
    SpriteHost,
};

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

    /// S29.3（B1）：UI chrome 是否已迁 Bevy `bevy_ui`（Bevy 壳 = true ⇒ 场景层跳过
    /// 已迁 chrome 原语，避免 1080 与 `bevy_ui` 双画；CPU 路径 = false 照旧）。
    fn chrome_in_bevy_ui(&self) -> bool {
        self.chrome_in_bevy_ui
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

    // S31.2b：包清单 / 注入源 = `ti_core::data::table_source` 注入面（壳层首访前采集）。
    fn map_pack_ids(&self) -> Option<Vec<String>> {
        ti_core::data::table_source::available_ids()
    }

    fn active_map_pack(&self) -> Option<String> {
        ti_core::data::table_source::source_id().map(str::to_owned)
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

    fn territory_ownership(&self, idx: u32) -> Option<Ownership> {
        self.territory.as_ref().and_then(|t| t.ownership(idx)).copied()
    }

    fn turn_seat(&self) -> u8 {
        self.turn_cycle.current
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

    fn track_node_sel(&self) -> Option<usize> {
        self.track_node_sel
    }

    fn set_track_node_sel(&mut self, v: Option<usize>) {
        self.track_node_sel = v;
    }

    fn boat_level(&self) -> u8 {
        GameApp::boat_level(self)
    }

    fn build_panel_open(&self) -> bool {
        self.build_panel_open
    }

    fn set_build_panel_open(&mut self, open: bool) {
        self.build_panel_open = open;
    }

    fn build_panel_sel(&self) -> u8 {
        self.build_panel_sel
    }

    fn set_build_panel_sel(&mut self, sel: u8) {
        self.build_panel_sel = sel;
    }

    /// S21.12 造船：走 `rules::boat::BoatSession::upgrade`（单向升级链 + 扣金 + 抬加成），
    /// 成功后把差额从钱包扣掉（与建筑建造同一钱包口径）。
    fn boat_upgrade(&mut self) -> bool {
        let before = self.bank_gold();
        let mut gold = before;
        let res = self.boat.upgrade(&mut gold);
        match res {
            Ok(_) => {
                let spent = before.saturating_sub(gold);
                self.spend_bank_gold(spent);
                true
            }
            Err(_) => false,
        }
    }

    fn boat_next_cost(&self) -> Option<u32> {
        self.boat.next_cost()
    }

    fn set_board_toast(&mut self, msg: String) {
        self.board_toast = Some(msg);
        self.board_toast_ticks = 120; // = board.rs TOAST_TICKS 口径（120 帧寿命，S21.7）
    }

    /// S28.6 是否正在领地主副本战斗（`vs` 场景 Back 路由：副本 = 回主地图，BOSS 战 = 回主菜单）。
    fn in_dungeon_battle(&self) -> bool {
        self.battle.as_ref().is_some_and(|b| b.dungeon.is_some() || b.dungeon_battle)
    }

    /// S28.6 确认进入待决副本（`pending_dungeon` 在位时切守军战；缺省 `false`）。
    fn confirm_dungeon(&mut self) -> bool {
        let Some(cell) = self.pending_dungeon.take() else {
            return false;
        };
        let Some(sess) = self.board.as_ref() else {
            return false;
        };
        let Some(sp) = sess.current().cloned() else {
            return false;
        };
        let Some(isle) = sp.isle.as_deref() else {
            return false;
        };
        self.start_dungeon_battle(cell, isle, 1);
        // S28.6：副本战与 BOSS 战同走 `vs` 场景驱动（`battle_step` 钩在 vs `on_input`）；
        // 漏切 = 战斗挂起不推进（`dungeon` 在位但无输入面）。
        self.scene = Scene::Vs;
        true
    }

    /// S28.2 存档 v4 扩展捕获（`territory.md` §8）：5 席（角色 index 升序，主玩家 = 已选角色）
    /// 金/位置/船 + 当前图领地图（无主格不写）+ 当前席位。
    fn save_v4_ext(&self) -> Option<SaveSlotV4Ext> {
        let b = data::balance().ok()?;
        let player_idx =
            u8::try_from(self.selection.character.map_or(0, Character::index).min(4)).unwrap_or(0);
        let seats = (0u8..5)
            .map(|i| {
                let position = if i == player_idx {
                    self.board.as_ref().map_or(0, BoardSession::position_index)
                } else {
                    self.ai_sessions[i as usize].as_ref().map_or(0, BoardSession::position_index)
                };
                SeatSave {
                    character: b
                        .selection
                        .characters
                        .get(i as usize)
                        .map_or(String::new(), |ch| ch.id.clone()),
                    is_player: i == player_idx,
                    gold: self.seat_gold[i as usize],
                    position,
                    boat_level: if i == player_idx { self.boat_level() } else { 1 },
                    buildings: Vec::new(),
                }
            })
            .collect();
        let territories = self
            .territory
            .as_ref()
            .map(|t| {
                let map_id = self.map_slot.map_or(String::new(), |ms| ms.id().to_string());
                let cells = t
                    .snapshot()
                    .into_iter()
                    .filter_map(|(cell, (owner, level))| owner.map(|o| (cell, (o, level))))
                    .collect();
                vec![TerritorySave { map: map_id, cells }]
            })
            .unwrap_or_default();
        Some(SaveSlotV4Ext { seats, territories, current_seat: self.turn_cycle.current })
    }

    /// S28.2 存档 v4 扩展还原（§8）：席位金 + 当前席 + 当前图领地账本（越界/不匹配图 = 宽松丢弃）。
    fn restore_v4_ext(&mut self, ext: &SaveSlotV4Ext) {
        for (i, seat) in ext.seats.iter().enumerate().take(5) {
            self.seat_gold[i] = seat.gold;
        }
        if ext.current_seat < 5 {
            self.turn_cycle.current = ext.current_seat;
        }
        let map_id = self.map_slot.map_or(String::new(), |ms| ms.id().to_string());
        for terr in &ext.territories {
            if terr.map != map_id {
                continue;
            }
            let Ok(bal) = data::balance() else {
                continue;
            };
            let Ok(db) = data::board() else {
                continue;
            };
            let Some(map_def) = db.map(&terr.map) else {
                continue;
            };
            let snapshot = terr
                .cells
                .iter()
                .map(|(cell, (owner, level))| (*cell, (Some(*owner), *level)))
                .collect();
            match TerritoryState::restore(map_def, bal, snapshot) {
                Ok(state) => self.territory = Some(state),
                Err(_e) => eprintln!("restore_v4_ext: 领地账本还原失败（宽松丢弃）"),
            }
            break;
        }
    }

    /// S28.2 某席位金币（越界 = 0）。
    fn seat_gold_of(&self, seat: u8) -> u32 {
        self.seat_gold.get(seat as usize).copied().unwrap_or(0)
    }

    /// S28.4 玩家席位标记（空 = 单选旧流）。
    fn player_seats(&self) -> Vec<Character> {
        self.seat_marks.clone()
    }

    /// S28.4 玩家席位标记写入（触摸行点选 toggle）。
    fn set_player_seats(&mut self, seats: Vec<Character>) {
        self.seat_marks = seats;
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
                let dungeon = b.dungeon.clone();
                b.dungeon = None;
                if dungeon.is_some() {
                    b.dungeon_battle = true;
                }
                if let Some((cell, isle, _level)) = dungeon {
                    // S28.6 副本战（territory.md §4/§9）：胜利 = 占领（两格 + 首占奖励）；
                    // 失败 = 现状不变。BOSS 推进钩与副本结算互斥（`b.dungeon` 在位 = 副本战）。
                    if self.battle_won {
                        self.last_sfx = Some(Sfx::Victory);
                        self.settle_dungeon_win(cell, &isle);
                    } else {
                        self.toast_dungeon_fail(&isle);
                    }
                } else if self.battle_won {
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
impl GameApp {
    /// S28.6 副本胜利结算（territory.md §4）：占领该岛两格（level = 1）+ 首占奖励入玩家席金币。
    fn settle_dungeon_win(&mut self, cell: u32, isle: &str) {
        let Ok(b) = data::balance() else {
            eprintln!("settle_dungeon_win: balance.json 加载失败（硬错误，不 fallback）");
            return;
        };
        let Some(mut terr) = self.territory.take() else {
            return;
        };
        let rules = TerritoryRules::new(b);
        let seat = self.turn_cycle.current;
        let bonus = terr.apply_dungeon(&rules, cell, seat, DungeonOutcome::Win);
        if bonus > 0 {
            self.seat_gold[seat as usize] =
                self.seat_gold[seat as usize].saturating_add(bonus).min(b.economy.gold_max);
        }
        self.territory = Some(terr);
        let isle_name = crate::i18n::sget(&format!("isle.{isle}")).to_string();
        self.board_toast = Some(format!("占领 {isle_name}（首占 +{bonus}）"));
        self.board_toast_ticks = 240;
    }

    /// S28.6 副本失败提示（§4：失败 = 现状不变，无账本变更）。
    fn toast_dungeon_fail(&mut self, isle: &str) {
        let isle_name = crate::i18n::sget(&format!("isle.{isle}")).to_string();
        self.board_toast = Some(format!("副本失败：{isle_name} 现状不变"));
        self.board_toast_ticks = 240;
    }
}
