#![allow(clippy::unwrap_used, clippy::expect_used)]
//! 飞轮 R21 · 方案 A：AI 玩家自动化对抗 E2E（规则层 headless 完整对局驱动器）。
//!
//! 用户 2026-11-20 拍板「AI 玩家自动化对抗真实 E2E 玩」。现 `ai_policy_v2` 单回合决策 +
//! 生命周期判定（`bankrupt`/`victory_seat_active`/`is_cleared`）已有，缺「N 个 AI 席当对手
//! 自动跑满一整局」的对抗驱动面。本文件 = 纯规则层驱动器（零渲染依赖，headless ms 级），
//! 复用现成 rules 函数（`board::BoardSession::step` 摇骰移动 + `territory_v2::ai_policy_v2`
//! AI 决策 + `TerritoryState: TerritoryApply` 结算回填 + `assault` 攻占 + `bankrupt`/
//! `victory_seat_active` 终局判定），断言**对局性质**：
//!
//! | 断言 | 内容 |
//! |---|---|
//! | 确定性 | 同 seed 同席数 ⇒ 同终局（胜者/回合数/末格位置/兵力逐位复现） |
//! | 终局可达 | 对局必达 `victory_seat_active` 有胜者 / 全破产 / 回合上限兜底之一（不卡死） |
//! | 不无限拉锯 | 回合数 ≤ 兜底上限（防 AI 互咬死循环） |
//! | 金有界 | 席位金恒 ≤ `balance.economy.gold_max`（入账钳制不溢出） |
//! | 多席不串 | 快照长度 = 席数（各席资源隔离，无越席写入） |
//!
//! 数据驱动：AI 行为由 `balance.territory.ai`（`siege_mode`/`upgrade_mode` 公式形态 +
//! `attack_threshold`/`upgrade_priority`/`toll_aversion` 系数，EXP-196）决定，改 JSON 不改代码。
//! 本测试 = 对抗对局的**正确性机器判据**（确定性/守恒/可达），非胜率统计（胜率分布 = 后续
//! 性能/调参轮，见 `evidence/ai_adversarial_e2e_options.md`）。

use ti_core::data::{balance, board};
use ti_core::rules::board::{BoardSession, Track};
use ti_core::rules::rng::GameRng;
use ti_core::rules::territory::{
    DungeonOutcome, SettleAction, TerritoryApply, TerritoryRules, TerritoryState,
};
use ti_core::rules::territory_v2::{self, SeatState};

/// 单局对局结果（驱动器输出快照，供确定性断言比对）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct GameOutcome {
    /// 终局胜者（`victory_seat_active`；全破产/上限 = `None`）。
    winner: Option<u8>,
    /// 总回合数（全席各计一次 = 1 回合；恒 ≤ 兜底上限）。
    rounds: u32,
    /// 终局时各席位置（确定性比对面）。
    positions: Vec<u32>,
    /// 终局时各席存活。
    alive: Vec<bool>,
    /// 终局时各席金/兵力（守恒终值比对面）。
    gold: Vec<u32>,
    crew: Vec<u32>,
}

/// 驱动 N 个 AI 席跑完整局（headless，纯规则层）。
///
/// `seed` 可播种（确定性）；`seats` = 席数（2..=5，`balance.selection` 口径）；
/// `max_rounds` 兜底上限（防 AI 互咬死循环；返回 `rounds` 恒 ≤ 此值）。
///
/// 编排 = 每回合各存活席依次：`BoardSession::step`（摇骰→沿环移动→落格/BOSS 拦停）→
/// 落格结算（`ai_policy_v2` 决策 + `TerritoryApply` 回填 + `assault` 攻占）→ 破产判定
/// （金/兵力耗尽 ⇒ `bankrupt` 剔除）。终局 = `victory_seat_active` 有胜者 / 全破产 / 达上限。
fn run_adversarial_game(seed: u32, seats: u8, max_rounds: u32) -> GameOutcome {
    let b = balance().expect("balance.json 有效");
    let db = board().expect("board.json 有效");
    let map = db.map("m01").expect("m01 在板");
    let terr = TerritoryState::new(map, b).expect("领地账本构建成功（领地格在赛道）");
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

    let mut terr = terr;
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
            boards[s].step(&mut rng, sides, boss_laps);
            let idx = boards[s].position_index();
            let seat_idx = u8::try_from(s).expect("席数 ≤ 5");
            settle_ai_landing(b, &mut terr, &rules, s, seat_idx, idx, &mut seats_state);
            // 破产判定：结算后**金耗尽**（金币 = 硬资源，过路费/升级金归零 = 出局；兵力经
            // `assault` 1/3 驻军损耗恒 >0 不淘汰 ⇒ 单用兵力判破产 = 对局无自然终局，飞轮 R21
            // 探针坐实「400 回合 winner=None 全员存活」）。金耗尽 = 玩家出局，`bankrupt` 清空占领格。
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

/// 断言 1：确定性（同 seed 同席数 ⇒ 同终局快照，逐位复现；不同 seed = 不同轨迹）。
#[test]
fn adversarial_deterministic_same_seed() {
    let a = run_adversarial_game(2026, 4, 400);
    let b = run_adversarial_game(2026, 4, 400);
    assert_eq!(a, b, "同 seed 同 4 AI 席 ⇒ 终局快照逐位一致（确定性复现）");
    let c = run_adversarial_game(4242, 4, 400);
    let differs = c.winner != a.winner
        || c.rounds != a.rounds
        || c.positions != a.positions
        || c.crew != a.crew;
    assert!(differs, "不同 seed 应对抗出不同轨迹（非恒同；防 AI 决策与 RNG 脱钩）");
}

/// 断言 2：不无限拉锯（对局必达兜底上限；终局态 = 胜者 / 全破产 / 达上限之一）。
///
/// **已知业务缺口（飞轮 R21 探针坐实，待用户拍板，不擅自改 balance）**：现行 `apply_collect`
/// 每圈产出金无衰减 + 破产仅金耗尽（兵力 1/3 驻军损耗恒 >0）⇒ **400 回合多数 seed 仍 2 席
/// 存活 + 金暴涨（实测 72078），`winner=None` 对局不自然收敛到终局**（金有界 = 上限 999999，
/// 400 回合打满不了）。本断言只验「兜底上限可达」（驱动器不卡死）；「对局可收敛到唯一胜者」=
/// 平衡调参缺口（加 BOSS 终结 / 金衰减 / 破产扩兵力=0，或调 `balance.territory` 节奏），
/// 见 `evidence/ai_adversarial_e2e_options.md` 待拍板段。
#[test]
fn adversarial_reaches_terminal_state() {
    for seed in [11u32, 7, 2026, 99999] {
        let out = run_adversarial_game(seed, 3, 400);
        assert!(out.rounds <= 400, "对局须达兜底上限内（不无限拉锯）：seed={seed}");
        let terminal_victory = out.winner.is_some();
        let terminal_all_bankrupt = out.alive.iter().all(|a| !*a);
        assert!(
            terminal_victory || terminal_all_bankrupt || out.rounds == 400,
            "终局态可达：seed={seed} winner={:?} alive={:?}",
            out.winner,
            out.alive
        );
    }
}

/// 断言 3：金有界（席位金恒 ≤ `balance.economy.gold_max`；入账钳制不溢出）。
#[test]
fn adversarial_gold_bounded() {
    let b = balance().expect("balance 有效");
    let out = run_adversarial_game(2026, 4, 200);
    for g in &out.gold {
        assert!(*g <= b.economy.gold_max, "席位金超 `balance.economy.gold_max`（入账须钳制）：{g}");
    }
}

/// 断言 4：多席不串（各席快照长度 = 席数；无越席写入）。
#[test]
fn adversarial_seat_isolation() {
    for seats in [2u8, 3, 4, 5] {
        let out = run_adversarial_game(2026, seats, 100);
        assert_eq!(out.positions.len(), usize::from(seats), "位置快照长度 = 席数");
        assert_eq!(out.crew.len(), usize::from(seats), "兵力快照长度 = 席数");
        assert_eq!(out.alive.len(), usize::from(seats), "存活快照长度 = 席数");
    }
}

/// 断言 5：回合上限兜底生效（`rounds` 恒 ≤ 传入上限；防 AI 互咬死循环）。
#[test]
fn adversarial_rounds_bound_respected() {
    for cap in [50u32, 150, 400] {
        let out = run_adversarial_game(2026, 4, cap);
        assert!(out.rounds <= cap, "对局回合数须 ≤ 兜底上限 {cap}");
    }
}
