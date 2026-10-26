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
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot, SelectionState};
use ti_core::rules::board::{BoardSession, TrackSpace, Turn};
use ti_core::rules::cards;
use ti_core::rules::messages::total_pages;
use ti_core::rules::progression::{CLEARED_BIT, ProgressionState};
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
