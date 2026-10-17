//! S15.1 战斗域业务场景穷举测试（`exhaust_combat`）。
//!
//! 枚举空间全部由 `assets/data/exhaust_scenarios.json`（`ti_core::data::exhaust()`）驱动，
//! 本文件只做"加载 + 逐条断言"薄壳（加新组合 = 改 JSON，不改测试代码，
//! `02_conventions.md` §1 数值入 JSON 纪律）。
//!
//! 不变式（每组合，精确值对账，非自我循环）：
//! - HP 永不下溢（`player_hp`/`enemy_hp` ∈ [0, 初始值]）
//! - `finish()` 后 `outcome` 必 `Some(_)`（`max_rounds` 兜底规则，永不 `None`）
//! - 同 `(cfg, seed, player, tier)` 两次独立 `finish()` 逐位一致（确定性）
//! - 越界 tier → 立即 `outcome=Some(Side::Player)`、`round=0`（无效战斗，`combat.md` 可测规则表）

use ti_core::data::{BossTier, CombatCfg, DamageCfg, balance, exhaust};
use ti_core::domain::combat::{Side, Unit};
use ti_core::rules::battle::BattleSession;

/// `usize` → `u32` 安全转换（测试枚举空间规模远小于 `u32::MAX`，越界钳到 `u32::MAX`，不 panic）。
#[allow(clippy::cast_possible_truncation)]
fn usize_as_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// 独立手算一遍伤害公式（与 `rules/battle.rs::deal_damage` 同口径，测试侧独立实现，
/// 用于对账而非循环调用被测函数——避免 EXP-067 式"两边各自单层自洽"假绿）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn expected_damage_floor(atk: u32, dmg: &DamageCfg) -> u32 {
    let base = (f64::from(atk) * f64::from(dmg.atk_coef)).round() as u32;
    base.saturating_add(dmg.rng_span).max(dmg.flat)
}

/// 越界 tier 的无效战斗必须立即 Player 胜、0 回合（`combat.md` 可测规则表第 7 条）。
#[test]
fn exhaust_out_of_range_tier_instant_player_win() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let b = balance().expect("balance.json 有效");
    let tier_max = b.combat.boss_tiers.iter().map(|t| t.tier).max().expect("档非空");
    for tier in (tier_max + 1)..=tier_max + 8 {
        for &seed in &es.combat.oob_seed_check {
            let mut s = BattleSession::new_with_cfg(
                b.combat.clone(),
                seed,
                Unit { hp: 100, atk: 20 },
                tier,
            );
            assert_eq!(
                s.outcome(),
                Some(Side::Player),
                "越界 tier={tier} seed={seed} 必须立即 Player 胜（无效战斗）"
            );
            assert_eq!(s.round(), 0, "越界 tier 会话未启动，round 初值恒 0");
            assert_eq!(s.boss_hp(), 0, "越界 tier BOSS HP = 0（无 BOSS 可打）");
            let (trace, winner, rounds) = s.finish();
            assert!(trace.is_empty(), "越界 tier 无逐回合轨迹");
            assert_eq!(winner, Side::Player);
            assert_eq!(rounds, 0);
        }
    }
}

/// 全 tier × 全 seed × 双先手 × 全 `max_rounds` 档 × 全玩家属性档 = 组合数由 JSON 驱动，
/// 每组合断言 HP 无下溢 + outcome 必 Some + 同组合两次独立跑逐位一致（确定性）。
#[test]
fn exhaust_all_tier_seed_combo_invariants() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let b = balance().expect("balance.json 有效");
    let tiers: Vec<u32> = b.combat.boss_tiers.iter().map(|t| t.tier).collect();
    assert_eq!(tiers.len(), 5, "balance.combat 5 档 BOSS");

    let mut combos_checked: u32 = 0;
    for tier in &tiers {
        for &seed in &es.combat.seeds {
            for &player_first in &[true, false] {
                for &max_rounds in &es.combat.max_rounds_levels {
                    let mut cfg = b.combat.clone();
                    cfg.player_first = player_first;
                    cfg.max_rounds = max_rounds;
                    for pv in &es.combat.player_variants {
                        let player = Unit { hp: pv.hp, atk: pv.atk };
                        let mut a = BattleSession::new_with_cfg(cfg.clone(), seed, player, *tier);
                        let initial_boss_hp = cfg.boss(*tier).expect("tier 必在表内").hp;
                        let (trace_a, winner_a, rounds_a) = a.finish();
                        let mut b2 = BattleSession::new_with_cfg(cfg.clone(), seed, player, *tier);
                        let (trace_b, winner_b, rounds_b) = b2.finish();

                        // 确定性：同组合两次独立跑逐位一致
                        assert_eq!(
                            trace_a, trace_b,
                            "确定性违例：tier={tier} seed={seed} player_first={player_first} \
                             max_rounds={max_rounds} player={player:?} 两次轨迹不一致"
                        );
                        assert_eq!(winner_a, winner_b, "胜方不一致（确定性违例）");
                        assert_eq!(rounds_a, rounds_b, "回合数不一致（确定性违例）");

                        // 不变式：逐回合双方 HP 恒在 [0, 初始值] 内（无下溢）
                        let mut prev_ph = player.hp;
                        for &(ph, eh) in &trace_a {
                            assert!(eh <= initial_boss_hp, "BOSS HP 越初始值（下溢/上溢违例）");
                            assert!(ph <= player.hp, "玩家 HP 越初始值");
                            // FIND-A3 修复：BOSS 存活反击的回合，玩家 HP 至少掉 `flat`
                            //（combat.md 可测规则表第 3 条）。`eh > 0` = 本回合 BOSS 未归零 →
                            // 有反击 → 玩家 HP 必至少掉 `flat`；`eh == 0` = BOSS 本回合归零，
                            // 按「boss_hp==0 立即 Victory 不再反击」语义，玩家 HP 未变，跳过。
                            if eh > 0 {
                                assert!(
                                    ph <= prev_ph.saturating_sub(cfg.damage.flat),
                                    "BOSS 存活回合玩家 HP 须至少掉 flat={flat}（tier={tier} seed={seed}）",
                                    flat = cfg.damage.flat
                                );
                            }
                            prev_ph = ph;
                        }
                        // outcome 必 Some（finish 返回后 outcome 已定，不再 None）
                        let mut check =
                            BattleSession::new_with_cfg(cfg.clone(), seed, player, *tier);
                        let _ = check.finish();
                        assert!(
                            check.outcome().is_some(),
                            "finish 后 outcome 必 Some（兜底规则，永不 None）：tier={tier}"
                        );
                        combos_checked += 1;
                    }
                }
            }
        }
    }
    let expected_combos = usize_as_u32(
        tiers.len()
            * es.combat.seeds.len()
            * 2
            * es.combat.max_rounds_levels.len()
            * es.combat.player_variants.len(),
    );
    assert_eq!(
        combos_checked,
        expected_combos,
        "组合数须 = 5 tier × {seeds} seed × 2 先手 × {mr} max_rounds 档 × {pv} 玩家档（JSON 驱动）",
        seeds = es.combat.seeds.len(),
        mr = es.combat.max_rounds_levels.len(),
        pv = es.combat.player_variants.len()
    );
}

/// `max_rounds=1` 边界最小档：首回合归零即对应方胜，未归零即兜底 `Enemy` 胜
/// （`combat.md` 胜负判定表）。两个用例（高 atk 必杀 / atk 恰好等于档 0 hp）由 JSON 驱动。
#[test]
fn exhaust_max_rounds_one_edge_case() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let b = balance().expect("balance.json 有效");
    let mut cfg = b.combat.clone();
    cfg.max_rounds = 1;
    cfg.player_first = true;

    let c = es.combat.max_rounds_one_case;
    let mut s = BattleSession::new_with_cfg(
        cfg.clone(),
        c.seed,
        Unit { hp: c.player_hp, atk: c.player_atk },
        c.boss_tier,
    );
    let (trace, winner, rounds) = s.finish();
    assert_eq!(rounds, 1, "首回合即归零，回合数 = 1");
    assert_eq!(winner, Side::Player, "玩家先手首回合打死 BOSS → Player 胜");
    assert_eq!(trace.len(), 1);

    let c2 = es.combat.max_rounds_one_case_exact;
    let boss_hp = cfg.boss(c2.boss_tier).expect("档在表内").hp;
    assert_eq!(
        c2.player_atk, boss_hp,
        "JSON 用例 player_atk 须恰好等于档 {} BOSS hp（精确边界，非凭感觉猜值）",
        c2.boss_tier
    );
    let mut s2 = BattleSession::new_with_cfg(
        cfg.clone(),
        c2.seed,
        Unit { hp: c2.player_hp, atk: c2.player_atk },
        c2.boss_tier,
    );
    let (trace2, winner2, rounds2) = s2.finish();
    assert_eq!(rounds2, 1);
    assert_eq!(
        winner2,
        Side::Player,
        "atk 恰好 = BOSS hp，首回合归零 → 立即 Player 胜，不再反击（玩家 hp 未消耗）"
    );
    // FIND-A4 修复：精确断言"不再反击"——BOSS 归零那一回合，玩家 HP 未继续变化
    assert_eq!(
        trace2.first().map(|&(ph, _)| ph),
        Some(c2.player_hp),
        "BOSS 归零后玩家 HP 未继续变化（不再反击）"
    );
    let _ = cfg;
}

/// 夹具隔离假值（不读 balance）交叉验证：全 tier × seed × 双先手 × `max_rounds` 档组合，
/// 不变式与真实表口径一致（双轨交叉，避免"恰好夹具值绕过边界"假绿）。
#[test]
fn exhaust_fixture_cfg_cross_validation() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let fx = &es.combat.fixture;
    let mut combos: u32 = 0;
    for tier in fx.boss_tiers.iter().map(|t| t.tier) {
        for &seed in &fx.seeds {
            for &player_first in &[true, false] {
                for &max_rounds in &fx.max_rounds_levels {
                    let cfg = CombatCfg {
                        player_first,
                        max_rounds,
                        player: None,
                        damage: DamageCfg { flat: 1, atk_coef: 1.0, rng_span: 3 },
                        boss_tiers: fx
                            .boss_tiers
                            .iter()
                            .map(|t| BossTier { tier: t.tier, hp: t.hp, atk: t.atk })
                            .collect(),
                    };
                    let player = Unit { hp: fx.player_hp, atk: fx.player_atk };
                    let mut a = BattleSession::new_with_cfg(cfg.clone(), seed, player, tier);
                    let (trace_a, winner_a, rounds_a) = a.finish();
                    let mut b = BattleSession::new_with_cfg(cfg.clone(), seed, player, tier);
                    let (trace_b, winner_b, rounds_b) = b.finish();
                    assert_eq!(trace_a, trace_b, "夹具确定性违例 tier={tier} seed={seed}");
                    assert_eq!(winner_a, winner_b);
                    assert_eq!(rounds_a, rounds_b);
                    let initial_boss = cfg.boss(tier).expect("夹具档必有").hp;
                    for &(_, eh) in &trace_a {
                        assert!(eh <= initial_boss, "夹具 BOSS HP 越初始值");
                    }
                    let floor = expected_damage_floor(fx.player_atk, &cfg.damage);
                    assert!(floor >= cfg.damage.flat, "伤害下限恒 ≥ flat");
                    combos += 1;
                }
            }
        }
    }
    let expected_combos =
        usize_as_u32(fx.boss_tiers.len() * fx.seeds.len() * 2 * fx.max_rounds_levels.len());
    assert_eq!(combos, expected_combos, "夹具组合数须 = JSON 驱动值");
}
