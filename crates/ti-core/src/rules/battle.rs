//! S3.8 战斗系统正式实现。
//!
//! 与 `oracle::battle` 行为逐位一致（同 seed 同输入产生逐位相同的
//! 每回合 HP 轨迹/胜方/回合数），但结构不同：用 `BattleSession`
//! 状态机结构体 + 方法承载状态，而非自由函数。
//!
//! 依据 `docs/spec/battle.md`：
//! - 单位 [`Unit`]、阵营 [`Side`]（从 oracle 重导出）；
//! - [`BattleSession`]：战斗会话（seed + 双方 HP + 回合数 + 结果）；
//! - `new` / `step` / `finish` / `boss_hp` 方法。

use crate::oracle::battle::{Side, Unit, oracle_battle_round, oracle_boss_hp};

/// 战斗会话：seed + 双方 HP + 回合数 + 结果（spec §c 状态机）。
#[derive(Debug, Clone)]
pub struct BattleSession {
    seed: u32,
    player_hp: u32,
    player_atk: u32,
    enemy_hp: u32,
    enemy_atk: u32,
    round: u32,
    outcome: Option<Side>,
}

impl BattleSession {
    /// 新建战斗会话（spec §b，`sub_572E4` ctor 等价：BOSS 用 `oracle_boss_hp` 初始化 HP）。
    ///
    /// `boss_tier` 越界 → `enemy_hp = 0`，会话立即结束（`outcome = Some(Player)`，round = 0）。
    #[must_use]
    pub fn new(seed: u32, player: Unit, boss_tier: u32) -> Self {
        let enemy_hp = oracle_boss_hp(boss_tier);
        let enemy_atk = 10;
        let (outcome, round) = if enemy_hp == 0 { (Some(Side::Player), 0) } else { (None, 0) };
        Self {
            seed,
            player_hp: player.hp,
            player_atk: player.atk,
            enemy_hp,
            enemy_atk,
            round,
            outcome,
        }
    }

    /// 当前 BOSS HP（只读）。
    #[must_use]
    pub fn boss_hp(&self) -> u32 {
        self.enemy_hp
    }

    /// 当前回合数（只读）。
    #[must_use]
    pub fn round(&self) -> u32 {
        self.round
    }

    /// 战斗结果（只读，未结束 → `None`）。
    #[must_use]
    pub fn outcome(&self) -> Option<Side> {
        self.outcome
    }

    /// 单步回合（行为与 `oracle_battle_round` 逐位一致，更新内部状态）。
    ///
    /// 已结束的会话（`outcome` 已定）→ 返回当前 HP + 已定结果，不再推进。
    /// 未结束 → 执行 1 回合，更新 HP/seed/round/outcome。
    #[must_use]
    pub fn step(&mut self) -> (u32, u32, Option<Side>) {
        if let Some(w) = self.outcome {
            return (self.player_hp, self.enemy_hp, Some(w));
        }
        let player = Unit { hp: self.player_hp, atk: self.player_atk };
        let enemy = Unit { hp: self.enemy_hp, atk: self.enemy_atk };
        let (ph, eh, winner) = oracle_battle_round(self.seed, player, enemy);
        self.player_hp = ph;
        self.enemy_hp = eh;
        self.round += 1;
        self.seed = self.seed.wrapping_add(1);
        self.outcome = winner;
        (ph, eh, winner)
    }

    /// 整场战斗（行为与 `oracle_full_battle` 逐位一致）。
    ///
    /// 从当前状态开始（通常 `new` 后立即调用），跑完全部回合，
    /// 返回 `(每回合 HP 轨迹, 胜方, 回合数)`。
    ///
    /// 轨迹记录每步 `step` 后的双方 HP（与 `oracle_full_battle` 一致，
    /// 不含初始 HP 快照，只含每回合结束后的 HP）。
    #[must_use]
    pub fn finish(&mut self) -> (Vec<(u32, u32)>, Side, u32) {
        if let Some(w) = self.outcome {
            // 已立即结束（BOSS HP=0）
            return (Vec::new(), w, 0);
        }
        let mut trace: Vec<(u32, u32)> = Vec::new();
        for _ in 0..1000 {
            let (ph, eh, outcome) = self.step();
            trace.push((ph, eh));
            if let Some(w) = outcome {
                return (trace, w, self.round);
            }
        }
        (trace, Side::Enemy, self.round)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_new_and_step() {
        let player = Unit { hp: 100, atk: 10 };
        let mut s = BattleSession::new(1, player, 0);
        assert_eq!(s.boss_hp(), 50_000);
        assert_eq!(s.round(), 0);
        assert_eq!(s.outcome(), None);
        let (ph, eh, _) = s.step();
        assert_eq!(s.round(), 1);
        assert!(ph <= 100);
        assert!(eh < 50_000);
    }

    #[test]
    fn session_invalid_tier() {
        let player = Unit { hp: 100, atk: 10 };
        let mut s = BattleSession::new(0, player, 99);
        assert_eq!(s.boss_hp(), 0);
        assert_eq!(s.outcome(), Some(Side::Player));
        let (trace, w, r) = s.finish();
        assert!(trace.is_empty());
        assert_eq!(w, Side::Player);
        assert_eq!(r, 0);
    }

    #[test]
    fn session_deterministic() {
        let player = Unit { hp: 1000, atk: 50 };
        let mut s1 = BattleSession::new(42, player, 0);
        let mut s2 = BattleSession::new(42, player, 0);
        let r1 = s1.finish();
        let r2 = s2.finish();
        assert_eq!(r1, r2);
    }
}
