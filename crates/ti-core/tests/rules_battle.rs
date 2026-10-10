//! S8.3.a 战斗系统（`rules/battle`）测试。
//!
//! 依据 `docs/business/combat.md` 的可测试规则（每条 → 单测名）：
//! 同 seed 同结果 / 玩家先手 / min floor / 越界档无效 / 超回合兜底。
//!
//! 置于集成测试目录，豁免门禁的 `unwrap`/`expect` 扫描（EXP-052）；
//! `cfg.boss(0)` 在 `test_cfg()` 恒有档 0，`unwrap` 在此安全。

use ti_core::oracle::battle::{Side, Unit};
use ti_core::rules::battle::BattleSession;
use ti_core::rules::rng::GameRng;

/// 测试用战斗参数（不读 `balance`，隔离 RNG 行为；BOSS 档 0 与 `balance.json` 同值）。
fn test_cfg() -> ti_core::data::CombatCfg {
    ti_core::data::CombatCfg {
        player_first: true,
        max_rounds: 30,
        damage: ti_core::data::DamageCfg { flat: 1, atk_coef: 1.0, rng_span: 3 },
        boss_tiers: vec![ti_core::data::BossTier { tier: 0, hp: 50_000, atk: 10 }],
    }
}

#[test]
fn session_from_balance_new_and_step() {
    let player = Unit { hp: 100, atk: 10 };
    let mut s = BattleSession::from_balance(1, player, 0);
    assert_eq!(s.boss_hp(), 50_000, "BOSS 档 0 初 HP 走 balance.combat");
    assert_eq!(s.round(), 0);
    assert_eq!(s.outcome(), None);
    let (ph, eh, _) = s.step();
    assert_eq!(s.round(), 1);
    assert!(ph <= 100, "玩家 HP 只减不增");
    assert!(eh < 50_000, "BOSS 被扣血");
}

#[test]
fn session_invalid_tier() {
    let player = Unit { hp: 100, atk: 10 };
    let mut s = BattleSession::new_with_cfg(test_cfg(), 0, player, 99);
    assert_eq!(s.boss_hp(), 0, "越界档 → 无效战斗");
    assert_eq!(s.outcome(), Some(Side::Player), "无效战斗判玩家胜（无 BOSS 可打）");
    let (trace, w, r) = s.finish();
    assert!(trace.is_empty());
    assert_eq!(w, Side::Player);
    assert_eq!(r, 0);
}

#[test]
fn session_deterministic() {
    let player = Unit { hp: 1000, atk: 50 };
    let mut s1 = BattleSession::new_with_cfg(test_cfg(), 42, player, 0);
    let mut s2 = BattleSession::new_with_cfg(test_cfg(), 42, player, 0);
    assert_eq!(s1.finish(), s2.finish(), "同 seed + 同参数 → 同结果（可复现战斗）");
}

#[test]
fn combat_victory_no_counterattack() {
    // 玩家一击秒 BOSS → 立即 Player 胜，BOSS 不反击
    let cfg = test_cfg();
    let boss = cfg.boss(0).expect("test_cfg 恒有档 0");
    let player = Unit { hp: 1, atk: boss.hp };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (ph, eh, outcome) = s.step();
    assert_eq!(eh, 0, "玩家打空 BOSS HP");
    assert_eq!(outcome, Some(Side::Player));
    assert_eq!(ph, 1, "BOSS 未反击，玩家 HP 不变");
}

#[test]
fn combat_defeat_when_player_hp_zero() {
    let cfg = test_cfg();
    let player = Unit { hp: 1, atk: 1 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (_, eh, _) = s.step();
    assert!(eh > 0, "BOSS 仍存活");
    assert_eq!(s.outcome().unwrap_or(Side::Player), Side::Enemy, "玩家 HP 归零判 Defeat");
}

#[test]
fn combat_min_damage_floor() {
    // atk=0 → 伤害 = 0 + offset，但下限 flat，每回合至少掉 flat 血
    let mut cfg = test_cfg();
    cfg.damage.rng_span = 0; // 无浮动 → 伤害 = round(0*1.0) = 0 → 下限 flat
    cfg.damage.flat = 5;
    let player = Unit { hp: 100, atk: 0 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (_ph, eh, _) = s.step();
    assert!(eh <= 50_000 - 5, "BOSS 每回合至少掉 flat=5 血");
}

#[test]
fn combat_max_rounds_tiebreak() {
    // 双方 atk 极低 + max_rounds 小 → 兜底判胜负（`docs/business/combat.md` 胜负判定表）
    let mut cfg = test_cfg();
    cfg.max_rounds = 3;
    let player = Unit { hp: 10_000, atk: 1 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (_trace, w, r) = s.finish();
    assert_eq!(r, 3, "跑满 max_rounds");
    assert!(matches!(w, Side::Enemy), "超回合上限未分胜负 → 兜底 Enemy");
}

#[test]
fn combat_same_seed_same_result_from_balance() {
    // 走 `from_balance`（真实 `balance.combat`）验证同 seed 同结果
    let player = Unit { hp: 500, atk: 40 };
    let mut a = BattleSession::from_balance(7, player, 1);
    let mut b = BattleSession::from_balance(7, player, 1);
    assert_eq!(a.finish(), b.finish(), "from_balance 同 seed 同结果");
}

/// 可复现性：显式 `GameRng` 播种同 seed → 同战斗轨迹（`docs/business/combat.md` §伤害模型）。
#[test]
fn combat_reproducible_with_explicit_seed() {
    let player = Unit { hp: 300, atk: 20 };
    let mut s1 = BattleSession::new_with_cfg(test_cfg(), 1234, player, 0);
    let mut s2 = BattleSession::new_with_cfg(test_cfg(), 1234, player, 0);
    let r1 = s1.finish();
    let r2 = s2.finish();
    assert_eq!(r1, r2, "同 seed 同参数 → 同轨迹/胜方/回合数（可复现、可回放）");
    let _ = GameRng::new(0); // 占位：保持 RNG 依赖路径编译（战斗内部走 GameRng）
}
