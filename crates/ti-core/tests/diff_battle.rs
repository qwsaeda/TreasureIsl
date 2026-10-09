//! S3.8 战斗系统差分测试。
//!
//! 遍历 `seed` 0..=10 × `tier` 0..=4 × 玩家 HP/atk 组合，
//! `oracle_full_battle` 与 impl `finish` 逐位一致（每回合 HP 轨迹 + 胜方 + 回合数全等）；
//! 另加 2 个语义测试（(a) 固定种子同输入整场输出确定；(b) tier4=300000 HP 正确）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::battle::{Side, Unit, oracle_boss_hp, oracle_full_battle};
use ti_core::rules::battle::BattleSession;

/// 断言 oracle 与 impl 的整场战斗结果逐位一致。
fn assert_full_battle_equal(seed: u32, player_hp: u32, player_atk: u32, boss_tier: u32) {
    let player = Unit { hp: player_hp, atk: player_atk };
    let o = oracle_full_battle(seed, player, boss_tier);
    let mut s = BattleSession::new(seed, player, boss_tier);
    let i = s.finish();
    assert_eq!(
        o.0, i.0,
        "HP 轨迹不一致 (seed={seed} hp={player_hp} atk={player_atk} tier={boss_tier})"
    );
    assert_eq!(
        o.1, i.1,
        "胜方不一致 (seed={seed} hp={player_hp} atk={player_atk} tier={boss_tier})"
    );
    assert_eq!(
        o.2, i.2,
        "回合数不一致 (seed={seed} hp={player_hp} atk={player_atk} tier={boss_tier})"
    );
}

/// 断言 oracle 与 impl 的 BOSS HP 逐位一致。
fn assert_boss_hp_equal(tier: u32) {
    let o = oracle_boss_hp(tier);
    let player = Unit { hp: 100, atk: 10 };
    let s = BattleSession::new(0, player, tier);
    let i = s.boss_hp();
    assert_eq!(o, i, "oracle_boss_hp vs boss_hp (tier={tier}) 不一致");
}

/// 遍历 `seed` 0..=10 × `tier` 0..=4 × 玩家 HP/atk 组合：
/// `oracle_full_battle` vs `BattleSession::finish`。
#[test]
fn diff_full_battle_all_combos() {
    let hp_values = [10u32, 100, 500, 1000, 5000, 99_999];
    let atk_values = [1u32, 10, 50, 100, 500];

    for seed in 0..=10u32 {
        for tier in 0..=4u32 {
            for &hp in &hp_values {
                for &atk in &atk_values {
                    assert_full_battle_equal(seed, hp, atk, tier);
                }
            }
        }
    }
}

/// 遍历 `tier` 0..=4（含越界）：`oracle_boss_hp` vs `BattleSession::boss_hp`。
#[test]
fn diff_boss_hp_all_tiers() {
    for tier in 0..=4u32 {
        assert_boss_hp_equal(tier);
    }
    // 越界
    assert_boss_hp_equal(5);
    assert_boss_hp_equal(100);
}

/// 语义测试 (a)：固定种子同输入整场输出确定（两次调用结果完全一致）。
#[test]
fn semantic_deterministic_output() {
    let player = Unit { hp: 1000, atk: 50 };
    let seed = 42u32;
    let tier = 2u32; // 150000 HP

    let r1 = oracle_full_battle(seed, player, tier);
    let mut s1 = BattleSession::new(seed, player, tier);
    let r2 = s1.finish();

    assert_eq!(r1.0, r2.0, "两次 HP 轨迹应一致");
    assert_eq!(r1.1, r2.1, "两次胜方应一致");
    assert_eq!(r1.2, r2.2, "两次回合数应一致");

    // 再跑一次（独立调用）
    let player2 = Unit { hp: 1000, atk: 50 };
    let r3 = oracle_full_battle(seed, player2, tier);
    assert_eq!(r1, r3, "同 seed 同输入应逐位相同");
}

/// 语义测试 (b)：tier4 = 300000 HP 正确。
#[test]
fn semantic_tier4_boss_hp() {
    assert_eq!(oracle_boss_hp(4), 300_000, "tier4 应为 300000");

    let player = Unit { hp: 1, atk: 1 };
    let mut s = BattleSession::new(0, player, 4);
    assert_eq!(s.boss_hp(), 300_000, "BattleSession 初始 BOSS HP 应为 300000");

    let (_trace, winner, _rounds) = s.finish();
    // 玩家 HP=1，BOSS atk=10 → 第一回合玩家必死
    assert_eq!(winner, Side::Enemy, "HP=1 的玩家 vs 300000 HP BOSS 应判 Enemy 胜");
}
