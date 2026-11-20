#![allow(clippy::unwrap_used, clippy::expect_used)]
//! 飞轮 R21 · 方案 A：AI 玩家自动化对抗 E2E（规则层 headless 完整对局驱动器）。
//!
//! 驱动器抽到公共模块 [`adversarial_common`]（方案 A/C 共享，单一事实源），本文件 = 方案 A 的
//! **5 条对局性质判据**（确定性/不拉锯/金有界/多席不串/回合上限）。
//!
//! | 断言 | 内容 |
//! |---|---|
//! | 确定性 | 同 seed 同席数 ⇒ 同终局（胜者/回合数/末格位置/兵力逐位复现） |
//! | 终局可达 | 对局必达 `victory_seat_active` 有胜者 / 全破产 / 回合上限兜底之一（不卡死） |
//! | 不无限拉锯 | 回合数 ≤ 兜底上限（防 AI 互咬死循环） |
//! | 金有界 | 席位金恒 ≤ `balance.economy.gold_max`（入账钳制不溢出） |
//! | 多席不串 | 快照长度 = 席数（各席资源隔离，无越席写入） |
//!
//! 数据驱动：AI 行为由 `balance.territory.ai`（`siege_mode`/`upgrade_mode` 公式形态 + 系数，
//! EXP-196）决定，改 JSON 不改代码。本文件 = 对抗对局**正确性机器判据**（确定性/守恒/可达），
//! 非胜率统计（统计面 = 方案 C，`adversarial_stats.rs`）。

mod adversarial_common;
use adversarial_common::{GameOutcome, run_adversarial_game};

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
    let b = ti_core::data::balance().expect("balance 有效");
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

// 保留 GameOutcome 引用面（避免 `mod` 死代码告警 + 供将来断言扩展）。
#[allow(dead_code)]
fn _outcome_type_anchor(_o: &GameOutcome) {}
