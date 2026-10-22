//! 存读档：存档槽读写 + 状态恢复（`rules::progression` / `persistence` 接线）。
//!
//! 拆分自 `game_app.rs`（S20.0b）。

use ti_core::data::balance;
use ti_core::domain::economy::Bank;
use ti_core::domain::map::GridPos;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::progression::{CLEARED_BIT, ProgressionState};
use ti_core::rules::rng;

use super::{GameApp, Scene};

impl GameApp {
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
}
