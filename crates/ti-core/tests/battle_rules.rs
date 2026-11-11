//! 战斗会话构造集成测试（S52.1；`docs/business/combat.md` 可测规则表）。
//!
//! S52.1 补测：`battle.rs` `dungeon_from_balance` / `player_hp_max` 此前 0 集成测试
//! （飞轮 R2 找问题 P1-5）。判据：`cargo test -p ti-core --test battle_rules` 全绿。

use ti_core::domain::combat::Unit;
use ti_core::rules::battle::BattleSession;

/// 测试用战斗配置（与 `rules_battle.rs` 的 `test_cfg` 同口径：确定性）。
fn cfg() -> ti_core::data::CombatCfg {
    ti_core::data::CombatCfg {
        player_first: true,
        max_rounds: 30,
        player: None,
        damage: ti_core::data::DamageCfg { flat: 1, atk_coef: 1.0, rng_span: 3 },
        boss_tiers: vec![ti_core::data::BossTier { tier: 0, hp: 50_000, atk: 10 }],
        defend_reduction_pct: 50,
        items: Vec::new(),
        anim: ti_core::data::CombatAnimCfg::default(),
    }
}

/// `dungeon_from_balance`（§副本 = 兵力模型，U3 定案）：同 `from_balance` 走真实 balance 表。
#[test]
fn dungeon_from_balance_builds_session() {
    let s = BattleSession::dungeon_from_balance(
        42,
        Unit { hp: 100, atk: 20 },
        Unit { hp: 80, atk: 15 },
    )
    .expect("balance 表加载");
    assert_eq!(s.player_hp(), 100, "玩家 HP = 传入值");
    assert_eq!(s.boss_hp(), 80, "敌方 HP = 传入值（dungeon 走 enemy 参数）");
}

/// `player_hp_max`（S27.7 回复上限）：= 开局玩家 HP（`heal_pct` 分母）。
#[test]
fn player_hp_max_is_initial_hp() {
    let mut s = BattleSession::new_with_cfg(cfg(), 7, Unit { hp: 50, atk: 5 }, 0);
    assert_eq!(s.player_hp_max(), 50, "回复上限 = 初始 HP");
    assert_eq!(s.player_hp(), 50, "开局玩家 HP = 初始");
    // 回合后玩家 HP 降，但上限不变（回复分母锚定初始值）。
    let (_round, _ph, _outcome) = s.step();
}

/// 同 seed 同敌人配置 ⇒ 同战斗轨迹（确定性，`dungeon_from_balance` 走 balance 表）。
#[test]
fn dungeon_same_seed_same_trajectory() {
    let mk = || {
        BattleSession::dungeon_from_balance(
            1234,
            Unit { hp: 200, atk: 30 },
            Unit { hp: 200, atk: 30 },
        )
        .expect("balance 表")
    };
    let mut a = mk();
    let mut b = mk();
    for _ in 0..3 {
        let sa = a.step();
        let sb = b.step();
        assert_eq!(sa, sb, "同 seed 同配置 = 同回合轨迹");
    }
}
