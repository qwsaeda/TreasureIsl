//! oracle：战斗系统差分基准，测试专用（S11 起降级为纯基准函数）。
//!
//! **类型已迁至 [`crate::domain::combat`]**（S11 归档改造：业务类型不该住在逆向目录）。
//! 本模块只保留从伪代码直译的**差分基准行为函数**（BOSS 档表 / 单回合 / 整场），
//! 供 `tests/diff_battle.rs` 与 `rules::battle` 对拍。
//!
//! 业务规则以 `docs/business/combat.md` 为准；本 oracle 是**历史参照**，不是规则来源。

pub use ti_core::domain::combat::{Side, Unit};

/// oracle：BOSS 档 HP 表（逆向初值，5 档；权威值在 `balance.combat.boss_tiers`）。
///
/// tier 0..=4 → 50000/100000/150000/200000/300000；越界（`tier > 4`）→ 0。
/// ⚠️ 正式实现 `rules::battle::BattleSession` 用 `balance.json` 的值（可调），
/// 本表冻结为历史基准（不改）。
#[must_use]
pub fn oracle_boss_hp(tier: u32) -> u32 {
    match tier {
        0 => 50_000,
        1 => 100_000,
        2 => 150_000,
        3 => 200_000,
        4 => 300_000,
        _ => 0,
    }
}

/// oracle：确定性伪随机伤害（spec §c 推测公式，无 rand 库）。
///
/// ```text
/// damage = (seed.wrapping_mul(2654435761)) % (atk + 1) + 1
/// ```
/// `atk = 0` → 恒 `1`（`% 1 = 0` + 1）；`seed = 0` → 恒 `1`。
#[must_use]
fn oracle_damage(seed: u32, atk: u32) -> u32 {
    seed.wrapping_mul(2_654_435_761) % (atk + 1) + 1
}

/// oracle：单回合战斗（spec §c，双方各行动一次）。
///
/// 规则：
/// - 玩家先行动：`enemy.hp -= oracle_damage(seed, player.atk)`；
/// - 敌方后行动：`player.hp -= oracle_damage(seed.wrapping_add(1), enemy.atk)`（seed+1 避免同回合双方伤害相同，**推测**）；
/// - 返回 `(player_hp, enemy_hp, winner)`；
/// - `winner`：`enemy.hp == 0` → `Player`；`player.hp == 0` → `Enemy`；双方同时 0 → `Player`（玩家优势，**推测**）；未分胜负 → `None`。
#[must_use]
pub fn oracle_battle_round(seed: u32, player: Unit, enemy: Unit) -> (u32, u32, Option<Side>) {
    let player_dmg = oracle_damage(seed, player.atk);
    let enemy_hp_after = enemy.hp.saturating_sub(player_dmg);

    let enemy_dmg = oracle_damage(seed.wrapping_add(1), enemy.atk);
    let player_hp_after = player.hp.saturating_sub(enemy_dmg);

    let winner = if enemy_hp_after == 0 {
        Some(Side::Player)
    } else if player_hp_after == 0 {
        Some(Side::Enemy)
    } else {
        None
    };

    (player_hp_after, enemy_hp_after, winner)
}

/// oracle：整场战斗（spec §c，每回合双方 HP 轨迹 + 胜方 + 回合数）。
///
/// 规则：
/// - `boss_hp = oracle_boss_hp(boss_tier)`（越界 → 0，战斗立即结束，**推测**：BOSS 无 HP 视为已死）；
/// - 每回合 `seed` 递增 1（`seed`、`seed+1`、`seed+2`...），确定性伪随机；
/// - 记录每回合结束后双方 HP（`(player_hp, enemy_hp)` 轨迹）；
/// - 任一方 HP = 0 → 结束，返回 `(轨迹, 胜方, 回合数)`；
/// - `boss_hp == 0`（越界档）→ 空轨迹 + `Player` 胜 + 0 回合。
///
/// 最大回合数上限 1000（防止无限循环，**推测**；实际 BOSS 50000 HP / 最小伤害 1 = 最多 50000 回合，
/// 但 oracle 以 1000 为界，diff 测试用有限组合保证在界内结束）。
#[must_use]
pub fn oracle_full_battle(seed: u32, player: Unit, boss_tier: u32) -> (Vec<(u32, u32)>, Side, u32) {
    let boss_hp_init = oracle_boss_hp(boss_tier);
    if boss_hp_init == 0 {
        return (Vec::new(), Side::Player, 0);
    }

    let enemy = Unit { hp: boss_hp_init, atk: 10 };
    let mut player_cur = player;
    let mut enemy_cur = enemy;
    let mut trace: Vec<(u32, u32)> = Vec::new();
    let mut round_seed = seed;
    let mut round_num: u32 = 0;

    for _ in 0..1000 {
        round_num += 1;
        let (ph, eh, outcome) = oracle_battle_round(round_seed, player_cur, enemy_cur);
        trace.push((ph, eh));
        player_cur = Unit { hp: ph, atk: player_cur.atk };
        enemy_cur = Unit { hp: eh, atk: enemy_cur.atk };
        round_seed = round_seed.wrapping_add(1);
        if let Some(winner) = outcome {
            return (trace, winner, round_num);
        }
    }
    // 1000 回合未分胜负（**推测**：极端低伤害情况），判定 Enemy 胜
    (trace, Side::Enemy, round_num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_hp_table() {
        assert_eq!(oracle_boss_hp(0), 50_000);
        assert_eq!(oracle_boss_hp(1), 100_000);
        assert_eq!(oracle_boss_hp(2), 150_000);
        assert_eq!(oracle_boss_hp(3), 200_000);
        assert_eq!(oracle_boss_hp(4), 300_000);
        assert_eq!(oracle_boss_hp(5), 0);
    }

    #[test]
    fn damage_zero_atk() {
        assert_eq!(oracle_damage(0, 0), 1);
        assert_eq!(oracle_damage(42, 0), 1);
    }

    #[test]
    fn damage_zero_seed() {
        assert_eq!(oracle_damage(0, 100), 1);
    }

    #[test]
    fn round_player_wins() {
        let player = Unit { hp: 100, atk: 1000 };
        let enemy = Unit { hp: 50, atk: 1 };
        let (_ph, eh, w) = oracle_battle_round(1, player, enemy);
        assert_eq!(w, Some(Side::Player));
        assert_eq!(eh, 0);
    }

    #[test]
    fn round_enemy_wins() {
        let player = Unit { hp: 1, atk: 1 };
        let enemy = Unit { hp: 100, atk: 1000 };
        let (ph, _eh, w) = oracle_battle_round(1, player, enemy);
        assert_eq!(w, Some(Side::Enemy));
        assert_eq!(ph, 0);
    }

    #[test]
    fn full_battle_deterministic() {
        let player = Unit { hp: 1000, atk: 50 };
        let (t1, w1, r1) = oracle_full_battle(42, player, 0);
        let (t2, w2, r2) = oracle_full_battle(42, player, 0);
        assert_eq!(t1, t2);
        assert_eq!(w1, w2);
        assert_eq!(r1, r2);
    }

    #[test]
    fn full_battle_invalid_tier() {
        let player = Unit { hp: 100, atk: 10 };
        let (trace, winner, rounds) = oracle_full_battle(0, player, 99);
        assert!(trace.is_empty());
        assert_eq!(winner, Side::Player);
        assert_eq!(rounds, 0);
    }
}
