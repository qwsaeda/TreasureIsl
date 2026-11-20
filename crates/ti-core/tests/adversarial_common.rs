//! 飞轮 R21 · AI 玩家自动化对抗 E2E · 公共驱动器模块（方案 A/C 共享，单一事实源）。
//!
//! `adversarial_e2e.rs`（方案 A：对局性质判据）与 `adversarial_stats.rs`（方案 C：调参统计面）
//! 经 `mod adversarial_common;` 共享本驱动器，避免「同一驱动器两份实现漂移」。
//! 驱动器 = 纯规则层（零渲染依赖，headless），复用现成 rules 函数：
//! `BoardSession::step`（摇骰→沿环移动→落格/BOSS 拦停）+ `territory_v2::ai_policy_v2`
//!（AI 决策，数据驱动 `balance.territory.ai`，EXP-196）+ `TerritoryApply`（结算回填）+
//! `assault`（攻占一次性确定性结算）+ `bankrupt`/`victory_seat_active`（终局判定）。
//!
//! **已知收敛缺口**（飞轮 R21 探针坐实，待用户拍板）：`apply_collect` 每圈产出金无衰减 +
//! 破产仅金耗尽（兵力 1/3 驻军损耗恒 >0）⇒ 长对局多数 seed 不出唯一胜者（`winner=None`）。
//! 驱动器如实暴露此缺口（`winner` 多为 `None`），收敛处置（A1 BOSS 终结/A2 金衰减/A3 破产扩
//! 兵力）= 平衡调参面，不擅自改 balance。

#![allow(dead_code)]

use ti_core::data::{balance, board};
use ti_core::rules::board::{BoardSession, Track};
use ti_core::rules::rng::GameRng;
use ti_core::rules::territory::{
    DungeonOutcome, SettleAction, TerritoryApply, TerritoryRules, TerritoryState,
};
use ti_core::rules::territory_v2::{self, SeatState};

/// 单局对局结果（驱动器输出快照，供确定性/统计断言比对）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOutcome {
    /// 终局胜者（`victory_seat_active`；全破产/上限/未收敛 = `None`）。
    pub winner: Option<u8>,
    /// 总回合数（全席各计一次 = 1 回合；恒 ≤ 兜底上限）。
    pub rounds: u32,
    /// 终局时各席位置（确定性比对面）。
    pub positions: Vec<u32>,
    /// 终局时各席存活。
    pub alive: Vec<bool>,
    /// 终局时各席金（守恒终值比对面）。
    pub gold: Vec<u32>,
    /// 终局时各席兵力（`assault` 1/3 驻军损耗后净值）。
    pub crew: Vec<u32>,
}

/// 驱动 N 个 AI 席跑完整局（headless，纯规则层）。
///
/// `seed` 可播种（确定性）；`seats` = 席数（2..=5，`balance.selection` 口径）；
/// `max_rounds` 兜底上限（防 AI 互咬死循环；返回 `rounds` 恒 ≤ 此值）。
///
/// # Panics
///
/// `balance()`/`board()` 加载失败 / `m01` 不在板 / 领地账本构建失败（测试用数据硬失败，`expect` 安全）。
pub fn run_adversarial_game(seed: u32, seats: u8, max_rounds: u32) -> GameOutcome {
    let b = balance().expect("balance.json 有效");
    let db = board().expect("board.json 有效");
    let map = db.map("m01").expect("m01 在板");
    let mut terr = TerritoryState::new(map, b).expect("领地账本构建成功（领地格在赛道）");
    let rules = TerritoryRules::new(b);
    let trk = Track::from_map(map).expect("m01 赛道构建");
    let mut rng = GameRng::new(seed);
    let sides = b.board.dice_sides.max(1);
    let boss_laps = b.board.boss_min_laps;

    let seat_cnt = usize::from(seats);
    let init_gold = b.economy.initial_gold;
    let init_crew = b.manpower.initial_crew;
    let init_arms = b.manpower.initial_arms;
    // 各席独立赛道会话（位置/绕圈独立）+ 席位资源（金/兵力/武器）。
    let mut boards: Vec<BoardSession> =
        (0..seat_cnt).map(|_| BoardSession::new(trk.clone(), 0)).collect();
    let mut seats_state: Vec<SeatState> =
        (0..seat_cnt).map(|_| SeatState::new(init_gold, init_crew, init_arms, 0)).collect();
    let mut active: Vec<bool> = vec![true; seat_cnt];

    let mut rounds = 0u32;
    loop {
        let active_mask = active.clone();
        // 终局判定（回合初）：在局掩码内唯一存活者 = 终局胜。
        if territory_v2::victory_seat_active(&seats_state, &active_mask).is_some() {
            break;
        }
        if active_mask.iter().all(|a| !a) {
            break; // 全破产
        }
        if rounds >= max_rounds {
            break; // 兜底上限（不无限拉锯）
        }
        rounds += 1;
        for s in 0..seat_cnt {
            if !active[s] {
                continue;
            }
            let laps_before = boards[s].laps();
            let turn = boards[s].step(&mut rng, sides, boss_laps);
            let laps_after = boards[s].laps();
            let idx = boards[s].position_index();
            let seat_idx = u8::try_from(s).expect("席数 ≤ 5");
            // A1（BOSS 终结）：绕满 `boss_min_laps` 圈 BOSS 现身 + 当前席「击败 BOSS」⇒ 该席胜本图
            // （大富翁式「绕满 N 圈打 BOSS 通关」，规则层终局；app 壳 = `host` BOSS 战胜 → `match_outcome`）。
            if turn.boss.engaged() {
                return boss_victory(s, &boards, &seats_state, rounds, &active);
            }
            // A2（金衰减）：落圈扣领地维护费（拥有格数 × `maintenance_per_lap`，金可耗尽 ⇒ 破产可触发）。
            let lap_delta = laps_after.saturating_sub(laps_before);
            if lap_delta > 0 {
                let maintenance = b
                    .territory
                    .maintenance_per_lap
                    .saturating_mul(terr.owned_count(seat_idx))
                    .saturating_mul(lap_delta);
                if maintenance > 0 {
                    seats_state[s].gold = seats_state[s].gold.saturating_sub(maintenance);
                }
            }
            settle_ai_landing(b, &mut terr, &rules, s, seat_idx, idx, &mut seats_state);
            // 破产判定：结算后**金耗尽**（金币 = 硬资源；兵力经 `assault` 1/3 驻军损耗恒 >0
            // 不淘汰 ⇒ 单用兵力判破产 = 对局无自然终局）。金耗尽 = 出局，`bankrupt` 清空占领格。
            if seats_state[s].gold == 0 {
                territory_v2::bankrupt(&mut seats_state[s], &mut terr, seat_idx);
                active[s] = false;
            }
        }
    }
    let active_mask = active.clone();
    GameOutcome {
        winner: territory_v2::victory_seat_active(&seats_state, &active_mask),
        rounds,
        positions: boards.iter().map(BoardSession::position_index).collect(),
        alive: active,
        gold: seats_state.iter().map(|st| st.gold).collect(),
        crew: seats_state.iter().map(|st| st.crew).collect(),
    }
}

/// A1（BOSS 终结）：当前席击败 BOSS ⇒ 该席胜本图，对局终局（大富翁式「绕满 N 圈打 BOSS 通关」）。
/// 终局快照 = 胜者席位 + 当前回合数 + 各席位置/兵力（确定性可比对；金在 BOSS 战胜瞬间定格）。
fn boss_victory(
    winner_seat: usize,
    boards: &[BoardSession],
    seats_state: &[SeatState],
    rounds: u32,
    active: &[bool],
) -> GameOutcome {
    // BOSS 通关 = 唯一胜者：胜者存活，其余席全部出局（大富翁式「先打 BOSS 者通本图」，
    // 终局快照「恰 1 存活席」自洽，供快照断言 + 出图标「唯一胜者」）。
    let alive: Vec<bool> = active.iter().enumerate().map(|(i, a)| *a && i == winner_seat).collect();
    GameOutcome {
        winner: Some(u8::try_from(winner_seat).expect("席数 ≤ 5")),
        rounds,
        positions: boards.iter().map(BoardSession::position_index).collect(),
        alive,
        gold: seats_state.iter().map(|st| st.gold).collect(),
        crew: seats_state.iter().map(|st| st.crew).collect(),
    }
}

/// 单席 AI 落格结算（`ai_policy_v2` 决策 + `TerritoryApply`/`assault` 回填；headless 版
/// `board.rs::settle_ai_landing` 的纯规则等价物——无 toast/i18n，只改领地账本 + 席位资源）。
fn settle_ai_landing(
    b: &ti_core::data::Balance,
    terr: &mut TerritoryState,
    rules: &TerritoryRules<'_>,
    s: usize,
    seat_idx: u8,
    idx: u32,
    seats_state: &mut [SeatState],
) {
    let action = territory_v2::ai_policy_v2(
        b,
        terr,
        seat_idx,
        idx,
        seats_state[s].gold,
        seats_state[s].crew,
        seats_state[s].arms,
    );
    let isle = terr.isle_cells_by_index(idx).unwrap_or("");
    let level = terr.ownership(idx).map_or(1, |o| o.level.max(1));
    match action {
        SettleAction::EnterDungeon => {
            let bonus = terr.apply_dungeon(rules, idx, seat_idx, DungeonOutcome::Win);
            seats_state[s].gold = seats_state[s].gold.saturating_add(bonus).min(b.economy.gold_max);
        }
        SettleAction::PayToll => {
            let owner = terr.ownership(idx).and_then(|o| o.owner).unwrap_or(u8::MAX);
            if owner == seat_idx || owner == u8::MAX {
                return;
            }
            let toll = terr.apply_pay_toll(rules, idx, seat_idx);
            let paid = toll.min(seats_state[s].gold);
            seats_state[s].gold = seats_state[s].gold.saturating_sub(paid);
            seats_state[owner as usize].gold =
                seats_state[owner as usize].gold.saturating_add(paid).min(b.economy.gold_max);
        }
        SettleAction::Siege => {
            let owner = terr.ownership(idx).and_then(|o| o.owner).unwrap_or(u8::MAX);
            if owner == seat_idx || owner == u8::MAX {
                return;
            }
            // 攻占 = 一次性 `assault`（兵力模型，重定驻军 1/3 守恒）；攻方席位资源走 `SeatState`。
            let new_garrison = seats_state[s].crew / 3;
            let mut attacker = seats_state[s].clone();
            attacker.position = idx;
            let _ = territory_v2::assault(b, &mut attacker, terr, idx, seat_idx, new_garrison);
            seats_state[s] = attacker;
        }
        SettleAction::Upgrade => {
            let cost = rules.upgrade_cost_at(isle, level);
            if seats_state[s].gold >= cost && terr.apply_upgrade(rules, idx, seat_idx) {
                seats_state[s].gold = seats_state[s].gold.saturating_sub(cost);
            }
        }
        SettleAction::Collect | SettleAction::Maxed => {
            let amt = terr.apply_collect(rules, idx, seat_idx);
            seats_state[s].gold = seats_state[s].gold.saturating_add(amt).min(b.economy.gold_max);
        }
    }
}
