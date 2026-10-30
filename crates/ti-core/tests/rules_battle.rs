//! S8.3.a / S12.1 战斗域包：战斗系统（`rules/battle`）测试。
//!
//! 依据 `docs/business/combat.md` 可测试规则表（每条 → 单测名）：
//!
//! | 规则 | 单测 |
//! |---|---|
//! | 同 seed + 同操作序列 ⇒ 同结果 | `combat_same_seed_same_result` / `combat_same_seed_same_result_from_balance` |
//! | 玩家先手，伤害顺序固定 | `combat_player_first`（`combat_victory_no_counterattack` 覆盖先手顺序） |
//! | 每回合玩家至少掉 `flat` 血 | `combat_min_damage_floor` |
//! | BOSS.hp==0 立即 Victory，不再反击 | `combat_victory_no_counterattack` |
//! | player.hp==0 判 Defeat | `combat_defeat_when_player_hp_zero` |
//! | `max_rounds` 兜底判胜负 | `combat_max_rounds_tiebreak` / `combat_max_rounds_tiebreak_same_hp` |
//! | 越界 tier ⇒ 无效战斗 | `combat_invalid_tier_rejected`（`session_invalid_tier`） |
//! | 火药桶伤害固定 300，不吃 atk/随机 | `combat_item_damage_is_fixed` |
//! | 医疗包回 25% 上限、不超上限 | `combat_item_heal_capped` |
//! | 朗姆酒 3 回合 atk×1.5，到期失效 | `combat_item_atk_buff_expires` |
//! | 烟雾弹只挡 1 次反击 | `combat_item_stagger_once` |
//! | 防御当回合反击减半 | `combat_defend_halves_counter` |
//! | 道具不消耗随机数（同 seed 同轨迹） | `combat_item_keeps_rng_state` |
//!
//! 置于集成测试目录，豁免门禁的 `unwrap`/`expect` 扫描（EXP-052）；
//! `cfg.boss(0)` 在 `test_cfg()` 恒有档 0，`expect` 在此安全。
//!
//! 类型路径走 `ti_core::domain::combat`（c1 契约 §1，`Unit`/`Side` 类型唯一家），
//! 不经逆向参照路径重导出（S12.1 消除战斗域对逆向路径的依赖）。

use ti_core::data::{BossTier, CombatCfg, DamageCfg};
use ti_core::domain::combat::{CombatAction, ItemEffect, Side, Unit};
use ti_core::rules::battle::BattleSession;

/// 测试用战斗参数（不读 `balance`，隔离 RNG 行为；BOSS 档 0 与 `balance.json` 同值）。
fn test_cfg() -> CombatCfg {
    CombatCfg {
        player_first: true,
        max_rounds: 30,
        player: None,
        damage: DamageCfg { flat: 1, atk_coef: 1.0, rng_span: 3 },
        boss_tiers: vec![BossTier { tier: 0, hp: 50_000, atk: 10 }],
        defend_reduction_pct: 50,
        items: Vec::new(),
    }
}

/// 无随机浮动的战斗参数（S27.7 道具/防御用例夹具：伤害可精确断言）。
fn deterministic_cfg() -> CombatCfg {
    let mut cfg = test_cfg();
    cfg.damage.rng_span = 0;
    cfg.boss_tiers = vec![BossTier { tier: 0, hp: 50_000, atk: 10 }];
    cfg
}

#[test]
fn session_from_balance_new_and_step() {
    let player = Unit { hp: 100, atk: 10 };
    let mut s = BattleSession::from_balance(1, player, 0).expect("balance.json 有效");
    assert_eq!(s.boss_hp(), 2_400, "BOSS 档 0 初 HP 走 balance.combat");
    assert_eq!(s.round(), 0);
    assert_eq!(s.outcome(), None);
    let (ph, eh, _) = s.step();
    assert_eq!(s.round(), 1);
    assert!(ph <= 100, "玩家 HP 只减不增");
    assert!(eh < 2_400, "BOSS 被扣血（档 0 hp=2400）");
}

#[test]
fn session_invalid_tier() {
    let player = Unit { hp: 100, atk: 10 };
    let mut s = BattleSession::new_with_cfg(test_cfg(), 0, player, 99);
    assert_eq!(s.boss_hp(), 0, "越界档 → 无效战斗");
    assert_eq!(s.outcome(), Some(Side::Player), "无效战斗判玩家胜（无 BOSS 可打）");
    let (trace, w, r) = s.finish();
    assert_eq!(trace, [] as [(u32, u32); 0]);
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
    assert!(eh <= 50_000 - 5, "BOSS 每回合至少掉 flat=5 血（test_cfg 档 0 夹具 50k，非 balance）");
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
    let mut a = BattleSession::from_balance(7, player, 1).expect("balance.json 有效");
    let mut b = BattleSession::from_balance(7, player, 1).expect("balance.json 有效");
    assert_eq!(a.finish(), b.finish(), "from_balance 同 seed 同结果");
}

/// 同 seed + 同操作序列 ⇒ 同结果（`docs/business/combat.md` 可测试规则表第 1 条）：
/// 显式 `GameRng` 播种 → 同战斗轨迹/胜方/回合数（可复现、可回放、可测试）。
#[test]
fn combat_same_seed_same_result() {
    let player = Unit { hp: 300, atk: 20 };
    let mut s1 = BattleSession::new_with_cfg(test_cfg(), 1234, player, 0);
    let mut s2 = BattleSession::new_with_cfg(test_cfg(), 1234, player, 0);
    let r1 = s1.finish();
    let r2 = s2.finish();
    assert_eq!(r1, r2, "同 seed 同参数 → 同轨迹/胜方/回合数");
}

/// 玩家先手，伤害顺序固定（`docs/business/combat.md` 可测试规则表第 2 条）：
/// `player_first == true`（默认档）→ 一击秒 BOSS 的回合内，BOSS 不反击 → 玩家 HP 保持初始值不变
/// （顺序即「先结算玩家伤害、归零即结束、不再轮到 BOSS」的固定流程；若顺序颠倒，先手优势即被破坏）。
#[test]
fn combat_player_first() {
    let cfg = test_cfg();
    let boss = cfg.boss(0).expect("test_cfg 恒有档 0");
    let player = Unit { hp: 7, atk: boss.hp };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (ph, eh, outcome) = s.step();
    assert_eq!(eh, 0, "玩家先手：先结算对 BOSS 伤害，BOSS 归零");
    assert_eq!(outcome, Some(Side::Player), "同回合 BOSS 归零即判玩家胜");
    assert_eq!(ph, 7, "BOSS 归零不再反击，玩家 HP 不变（顺序固定的可观测判据）");
}

/// `player_first == false`（配置可翻转）→ 先结算 BOSS 伤害：
/// BOSS 一击秒玩家的回合内，玩家不反击 → BOSS HP 保持初始值不变。
#[test]
fn combat_boss_first_when_disabled() {
    let mut cfg = test_cfg();
    cfg.player_first = false;
    let boss = cfg.boss(0).expect("test_cfg 恒有档 0");
    let player = Unit { hp: 1, atk: boss.hp };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (ph, eh, outcome) = s.step();
    assert_eq!(ph, 0, "BOSS 先手：先结算对玩家伤害，玩家归零");
    assert_eq!(outcome, Some(Side::Enemy), "同回合玩家归零即判 Defeat");
    assert_eq!(eh, boss.hp, "玩家归零不再反击，BOSS HP 不变（顺序翻转可观测）");
}

/// 越界 tier ⇒ 无效战斗（`docs/business/combat.md` 可测试规则表第 7 条，
/// `from_balance` 路径：真实 `balance.combat` 档表之外的 tier 同样判无效）。
#[test]
fn combat_invalid_tier_rejected() {
    let player = Unit { hp: 100, atk: 10 };
    let mut s = BattleSession::from_balance(0, player, 99).expect("balance.json 有效");
    assert_eq!(s.boss_hp(), 0, "越界档 → 无效战斗（BOSS 无 HP）");
    assert_eq!(s.outcome(), Some(Side::Player), "无效战斗判玩家胜（无 BOSS 可打）");
    let (trace, winner, rounds) = s.finish();
    assert!(trace.is_empty(), "无效战斗无回合轨迹");
    assert_eq!(winner, Side::Player);
    assert_eq!(rounds, 0);
}

/// `max_rounds` 兜底：跑满回合上限未分胜负 → `Defeat`（`Side::Enemy`，
/// `docs/business/combat.md` 胜负判定表第 3 行「超过 `max_rounds` 仍按 HP 高者判（同回合为 Defeat）」）。
/// 本用例把 `test_cfg` 的 `max_rounds` 调到 1000 以匹配 `balance.json` 的 `combat.max_rounds`
/// （战斗平衡目标 10–40 回合，30 回合内正常分胜负，1000 是防拉锯兜底）；
/// 玩家 `atk = 0`（单回合 1–4 伤害，BOSS 50k HP 撑满 1000 回合不掉空），玩家 HP 足够存活满 1000 回合
/// ⇒ 跑满上限双方均未归零 → 未分胜负，触发兜底判 `Enemy`。
#[test]
fn combat_max_rounds_tiebreak_same_hp() {
    let mut cfg = test_cfg();
    cfg.max_rounds = 1_000;
    let player = Unit { hp: 30_000, atk: 0 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (trace, winner, rounds) = s.finish();
    assert_eq!(rounds, 1_000, "跑满 max_rounds 未分胜负");
    assert_eq!(trace.len(), 1_000, "逐回合快照条数 = 跑满回合数");
    assert_eq!(winner, Side::Enemy, "兜底判 Enemy（Defeat）");
}

/// BOSS 档表逐档走 `balance.json`（`docs/business/combat.md` 可测试规则表 + `combat.md` §参数）：
/// 档 0..=4 的 HP 与 BOSS 档表一致，越界档 5 ⇒ 无效战斗。
#[test]
fn combat_boss_tiers_from_balance() {
    let player = Unit { hp: 1, atk: 0 };
    let expect_boss_hp: &[u32] = &[2_400, 4_800, 7_200, 9_600, 12_000];
    for (tier, &init) in expect_boss_hp.iter().enumerate() {
        let tier_u32 = u32::try_from(tier).unwrap_or(u32::MAX);
        let s = BattleSession::from_balance(0, player, tier_u32).expect("balance.json 有效");
        assert_eq!(s.boss_hp(), init, "档 {tier} 初 HP 走 balance.combat.boss_tiers[{tier}]");
        assert_eq!(s.outcome(), None, "合法档 → 正常开打");
    }
    let invalid = BattleSession::from_balance(0, player, 5).expect("balance.json 有效");
    assert_eq!(invalid.boss_hp(), 0, "档 5 越界 → 无效战斗");
    assert_eq!(invalid.outcome(), Some(Side::Player), "越界档判玩家胜（无 BOSS 可打）");
}

/// S27.7：火药桶伤害固定（不吃 atk、不吃随机）——`docs/business/combat.md` §道具表。
#[test]
fn combat_item_damage_is_fixed() {
    let cfg = deterministic_cfg();
    let boss = cfg.boss(0).expect("test_cfg 恒有档 0");
    // 玩家 atk=1（普攻只掉 1 血），火药桶固定 300 ⇒ 掉 300 而非 1
    let player = Unit { hp: 1_000, atk: 1 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (_ph, eh, _) = s.step_with(CombatAction::Item(ItemEffect::Damage(300)));
    assert_eq!(eh, boss.hp - 300, "火药桶固定 300 伤害（不吃 atk/随机）");
}

/// S27.7：医疗包按上限百分比回复，且不超上限。
///
/// 夹具：BOSS atk = 100（无随机）；每回合结算顺序 = 玩家动作 → 敌方反击
/// ⇒ 每次道具回合都会在回复后再挨一下，期望值已按此顺序推算。
#[test]
fn combat_item_heal_capped() {
    let mut cfg = deterministic_cfg();
    cfg.boss_tiers = vec![BossTier { tier: 0, hp: 50_000, atk: 100 }];
    let player = Unit { hp: 1_000, atk: 1 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    // 防御回合：反击 100 → 减半 50 ⇒ 玩家 HP 1000 → 950
    let _ = s.step_with(CombatAction::Defend);
    assert_eq!(s.player_hp(), 950, "防御减半：100 → 50");
    // 医疗包 20% 上限 = 200：950 + 200 = 1150 ⇒ 钳到上限 1000 ⇒ 再挨反击 100 ⇒ 900
    let _ = s.step_with(CombatAction::Item(ItemEffect::HealPct(20)));
    assert_eq!(s.player_hp(), 900, "回复被上限钳制（封顶 1000）后被反击 100");
    assert!(s.player_hp() <= player.hp, "回复不超上限");
    // 再来一轮：850 + 500（50% 请求）⇒ 仍钳到 1000 ⇒ 反击 100 ⇒ 900（证明是封顶而非溢出）
    let _ = s.step_with(CombatAction::Defend);
    assert_eq!(s.player_hp(), 850);
    let _ = s.step_with(CombatAction::Item(ItemEffect::HealPct(50)));
    assert_eq!(s.player_hp(), 900, "50% 请求同样被上限钳制（不溢出上限）");
}

/// S27.7：朗姆酒增益覆盖 3 次攻击后失效。
#[test]
fn combat_item_atk_buff_expires() {
    let mut cfg = deterministic_cfg();
    cfg.boss_tiers = vec![BossTier { tier: 0, hp: 50_000, atk: 0 }];
    let player = Unit { hp: 10_000, atk: 100 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    // 用道具（不吃当回合攻击）⇒ 第 2 回合起第一次攻击吃到增益
    let _ = s.step_with(CombatAction::Item(ItemEffect::AtkBuff { pct: 50, rounds: 3 }));
    assert_eq!(s.atk_buff_rounds(), 3, "增益剩 3 回合（使用当回合不计）");
    // 第 2 回合：增益生效（100 × 1.5 = 150）
    let (_p, eh1, _) = s.step_with(CombatAction::Attack);
    assert_eq!(eh1, 50_000 - 150, "第 1 次攻击吃 50% 增益");
    let (_p, eh2, _) = s.step_with(CombatAction::Attack);
    assert_eq!(eh2, 50_000 - 300, "第 2 次攻击仍吃增益");
    let (_p, eh3, _) = s.step_with(CombatAction::Attack);
    assert_eq!(eh3, 50_000 - 450, "第 3 次攻击吃最后一次增益");
    let (_p, eh4, _) = s.step_with(CombatAction::Attack);
    assert_eq!(eh4, 50_000 - 450 - 100, "第 4 次攻击增益已到期（回到原 atk）");
    assert_eq!(s.atk_buff_rounds(), 0, "增益计数归零");
}

/// S27.7：烟雾弹只挡 1 次反击（下一回合恢复正常掉血）。
#[test]
fn combat_item_stagger_once() {
    let cfg = deterministic_cfg();
    let player = Unit { hp: 1_000, atk: 1 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    // 烟雾弹回合：敌方反击落空 ⇒ 玩家 HP 不变
    let (ph, _eh, _) = s.step_with(CombatAction::Item(ItemEffect::Stagger));
    assert_eq!(ph, player.hp, "烟雾弹当回合敌方反击落空");
    // 下一回合正常反击
    let (ph2, _eh2, _) = s.step_with(CombatAction::Attack);
    assert_eq!(ph2, player.hp - 10, "烟雾弹只挡 1 次，下一回合正常挨打");
}

/// S27.7：防御动作当回合反击减半（`defend_reduction_pct = 50`）。
#[test]
fn combat_defend_halves_counter() {
    let cfg = deterministic_cfg();
    let player = Unit { hp: 1_000, atk: 1 };
    let mut s = BattleSession::new_with_cfg(cfg, 1, player, 0);
    let (ph, _eh, _) = s.step_with(CombatAction::Defend);
    assert_eq!(ph, player.hp - 5, "防御当回合反击伤害减半（BOSS atk=10 → 5）");
}

/// S27.7：道具不消耗随机数 ⇒ 同 seed + 同动作序列 ⇒ 同轨迹。
#[test]
fn combat_item_keeps_rng_state() {
    let player = Unit { hp: 1_000, atk: 20 };
    let mut a = BattleSession::new_with_cfg(test_cfg(), 77, player, 0);
    let mut b = BattleSession::new_with_cfg(test_cfg(), 77, player, 0);
    let _ = a.step_with(CombatAction::Item(ItemEffect::Damage(300)));
    let _ = b.step_with(CombatAction::Item(ItemEffect::Damage(300)));
    assert_eq!(a.finish(), b.finish(), "道具回合不推进 RNG ⇒ 后续轨迹一致");
}
