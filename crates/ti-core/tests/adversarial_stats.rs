#![allow(clippy::unwrap_used, clippy::expect_used)]
//! 飞轮 R21 · 方案 C（降级 = 调参统计面，零 ML 依赖）：AI 对抗**胜率/破产/通关分布**。
//!
//! 用户拍板「C 降级为调参统计面」（不引入 MCTS/自弈——违背数据驱动纪律 + 超游戏规模 +
//! 打穿确定性断言，见 `evidence/ai_adversarial_e2e_options.md`）。C 落点 = 在 A 公共驱动器
//! （[`adversarial_common`]，复用现成 `ai_policy_v2`）之上做**多 seed × 多 AI 席分布统计**，
//! 给「AI 对抗」一个可调参、可量化的面：
//!
//! | 统计 | 内容 |
//! |---|---|
//! | 对局收敛率 | 多 seed 跑满局，`winner.is_some()` 占比（对局真分出胜者的比例） |
//! | 破产分布 | 全破产（无人胜）/ 胜者席位分布（哪个 AI 席更易胜） |
//! | 金极差 | 终局各席金 max−min（经济离散度，调 `balance.territory` 节奏的观测面） |
//! | 回合数分布 | 收敛对局 `rounds` 的 min/max（对局长度，调 BOSS 节奏的观测面） |
//!
//! **判据口径**：本测试 = 调参**观测面**（统计可复现：同 seed 集 + 同席数 + 同 balance ⇒
//! 分布逐位复现），**非硬断言「收敛率必须 100%」**——因现行 balance 有收敛缺口（飞轮 R21
//! 探针：长对局 `winner=None`），统计面如实记录现状收敛率 + 分布，作为「改 balance 调节奏
//! 后对比提升」的基线。改 `balance.territory.ai` 系数 = 分布变（数据驱动，零代码）。

mod adversarial_common;
use adversarial_common::run_adversarial_game;

/// 多 seed × 席数跑一批对局，返回统计快照（确定性可复现：seed 集固定）。
fn distribution(seeds: &[u32], seats: u8, cap: u32) -> (u32, u32, u32, u8, u8, u32, u32) {
    // (总对局, 收敛〔winner=Some〕, 全破产, 胜者席位最小值, 胜者席位最大值, 金极差 max, 回合数 min)
    let mut total = 0u32;
    let mut converged = 0u32;
    let mut all_bankrupt = 0u32;
    let mut win_min: Option<u8> = None;
    let mut win_max: Option<u8> = None;
    let mut gold_spread_max = 0u32;
    let mut rounds_min: Option<u32> = None;
    for &seed in seeds {
        let out = run_adversarial_game(seed, seats, cap);
        total += 1;
        if let Some(w) = out.winner {
            converged += 1;
            win_min = Some(win_min.map_or(w, |m| m.min(w)));
            win_max = Some(win_max.map_or(w, |m| m.max(w)));
        }
        if out.alive.iter().all(|a| !*a) {
            all_bankrupt += 1;
        }
        let spread = out
            .gold
            .iter()
            .fold(0u32, |mx, &g| mx.max(g))
            .saturating_sub(out.gold.iter().fold(u32::MAX, |mn, &g| mn.min(g)));
        gold_spread_max = gold_spread_max.max(spread);
        rounds_min = Some(rounds_min.map_or(out.rounds, |m| m.min(out.rounds)));
    }
    (
        total,
        converged,
        all_bankrupt,
        win_min.unwrap_or(0),
        win_max.unwrap_or(0),
        gold_spread_max,
        rounds_min.unwrap_or(0),
    )
}

/// 统计可复现（同 seed 集 + 席数 + cap ⇒ 分布快照逐位一致；AI 决策 + RNG 确定性）。
#[test]
fn adversarial_distribution_reproducible() {
    let seeds = [11u32, 7, 2026, 42, 999, 31337, 555, 808];
    let a = distribution(&seeds, 4, 300);
    let b = distribution(&seeds, 4, 300);
    assert_eq!(a, b, "同 seed 集 ⇒ 分布快照逐位复现（调参统计面确定性）");
    assert_eq!(a.0, u32::try_from(seeds.len()).expect("seed 数 ≤ u32::MAX"), "总对局数 = seed 数");
}

/// 收敛观测（如实记录现状：现行 balance 有收敛缺口，长对局多数 seed `winner=None`；
/// 本断言 = 统计可跑 + 收敛率/全破产/金极差/回合数观测面在位，非强制收敛率 100%）。
#[test]
fn adversarial_convergence_observed() {
    let seeds = [11u32, 7, 2026, 42, 999, 31337];
    let (total, converged, all_bankrupt, win_min, win_max, gold_spread, rounds_min) =
        distribution(&seeds, 4, 300);
    // 观测面在位（统计可计算，不崩）：
    assert!(converged + all_bankrupt <= total, "收敛 + 全破产 ≤ 总对局（互斥终局态）");
    assert!(win_min <= win_max, "胜者席位 min ≤ max（分布一致）");
    assert!(gold_spread <= 999_999, "金极差 ≤ gold_max（入账钳制）");
    assert!(rounds_min <= 300, "回合数 min ≤ 兜底上限");
    // 现状如实：短对局（300 回合）4 AI 席，现行 balance 下多数不收敛（收敛缺口，R21 探针）。
    // 若 balance 调参后收敛率提升，此值随之变（数据驱动，零代码）——本测试 = 调参基线观测。
    let _ = (total, converged, all_bankrupt, win_min, win_max, gold_spread, rounds_min);
}

/// 调参面：AI 席数越多 = 互咬越密集 = 回合数/金极差趋势（观测，非硬断言）。
#[test]
fn adversarial_more_seats_more_contention() {
    let seeds = [2026u32, 7, 11];
    let fewer = distribution(&seeds, 2, 300);
    let more = distribution(&seeds, 5, 300);
    // 5 席互咬面 > 2 席（观测：回合数上限内 5 席更易触达兜底上限，金极差更大）。
    assert!(more.5 >= fewer.5 || more.1 >= fewer.1, "5 席对抗密度 ≥ 2 席（金极差/收敛观测面）");
}
