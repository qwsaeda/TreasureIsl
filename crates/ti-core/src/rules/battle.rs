//! S3.8 / S8.3.a 战斗系统正式实现（数值数据化）。
//!
//! 行为依据 `docs/business/combat.md`（唯一事实源）；数值单一事实源 =
//! `assets/data/balance.json → combat`（[`crate::data::balance`]），本模块**不再**
//! 读 `oracle::battle` 的 `oracle_boss_hp`/`oracle_battle_round`（S8 定位 §1，
//! 逆向产物降级参照物，伤害模型/BOSS 档全由 `CombatCfg` 注入）。
//!
//! - 单位 [`Unit`]、阵营 [`Side`]（仍从 oracle 重导出，S8.3.a 保留）；
//! - [`BattleSession`]：战斗会话（`CombatCfg` + `GameRng` + 双方 HP + 回合数 + 结果）；
//! - `from_balance` / `new_with_cfg` / `step` / `finish` / `boss_hp` 方法。
//!
//! 确定性（`docs/business/combat.md` 伤害模型）：
//! `damage = round(atk * atk_coef) + rng_offset`，`rng_offset ∈ [0, rng_span)`（走
//! [`GameRng::in_range`]），下限 `flat`；同 `seed` + 同 `CombatCfg` + 同操作序列
//! ⇒ 同 HP 轨迹/胜方/回合数（EXP-040，可复现、可回放、可测试）。
//!
//! 回合流程（`docs/business/combat.md`）：玩家先手 → 结算对 BOSS 伤害 →
//! BOSS 存活则反击 → 双方各判 HP；`boss_hp==0` 立即 `Player` 胜（不再反击），
//! `player_hp==0` 判 `Enemy` 胜，超 `max_rounds` 仍平局 → `Enemy`（兜底）。

use crate::data::{self, CombatCfg, DamageCfg};
use crate::oracle::battle::{Side, Unit};
use crate::rules::rng::GameRng;

/// 战斗会话：`CombatCfg` + `GameRng` + 双方 HP + 回合数 + 结果。
///
/// 数值（BOSS HP/atk、伤害浮动、回合上限）全由 [`CombatCfg`] 注入，本结构体
/// 不硬编码任何业务数值（S8.3.a 数据化判据，`docs/business/combat.md`）。
#[derive(Debug, Clone)]
pub struct BattleSession {
    cfg: CombatCfg,
    rng: GameRng,
    player_hp: u32,
    player_atk: u32,
    enemy_hp: u32,
    enemy_atk: u32,
    round: u32,
    outcome: Option<Side>,
}

/// 读 `balance.combat`（成功用真实表；失败回退显式默认档 0，不 panic，门禁红线）。
fn load_cfg() -> CombatCfg {
    match data::balance() {
        Ok(b) => b.combat.clone(),
        Err(_) => fallback_cfg(),
    }
}

/// 整表改坏时的回退战斗参数（显式默认，非静默假数据；BOSS 档 0 同 `balance.json`）。
fn fallback_cfg() -> CombatCfg {
    CombatCfg {
        player_first: true,
        max_rounds: 30,
        damage: DamageCfg { flat: 1, atk_coef: 1.0, rng_span: 3 },
        boss_tiers: vec![crate::data::BossTier { tier: 0, hp: 50_000, atk: 10 }],
    }
}

/// 伤害结算（`docs/business/combat.md`）：`round(atk * atk_coef) + rng_offset`，下限 `flat`。
/// `rng_offset ∈ [0, rng_span)`（`rng_span==0` → 无浮动，纯 `round(atk*coef)`）。
fn deal_damage(atk: u32, dmg: &DamageCfg, rng: &mut GameRng) -> u32 {
    let base = cast_atk_base(atk, dmg.atk_coef);
    let offset = if dmg.rng_span == 0 { 0 } else { rng.in_range(dmg.rng_span) };
    base.saturating_add(offset).max(dmg.flat)
}

/// `atk * atk_coef` 四舍五入取整（非负有限值钳到 `u32::MAX`；clippy pedantic 的 f64 转整在此统一挂 allow）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn cast_atk_base(atk: u32, coef: f32) -> u32 {
    let v = (f64::from(atk) * f64::from(coef)).round();
    if !v.is_finite() || v < 0.0 {
        return 0;
    }
    u32::try_from(v.min(f64::from(u32::MAX)) as u64).unwrap_or(u32::MAX)
}

impl BattleSession {
    /// 从 `balance.combat` 新建会话（数值单一事实源，`docs/business/combat.md`）。
    ///
    /// `boss_tier` 越界（`cfg.boss_tier(tier)` → `None`）→ `enemy_hp = 0`，会话立即
    /// `outcome = Some(Player)`、`round = 0`（无效战斗，与 `docs/business/combat.md`
    /// 「越界 tier ⇒ 无效战斗」一致，可回归测）。
    #[must_use]
    pub fn from_balance(seed: u32, player: Unit, boss_tier: u32) -> Self {
        Self::new_with_cfg(load_cfg(), seed, player, boss_tier)
    }

    /// 显式 `CombatCfg` 注入（测试 / 自定义战斗参数，不读 `balance`）。
    #[must_use]
    pub fn new_with_cfg(cfg: CombatCfg, seed: u32, player: Unit, boss_tier: u32) -> Self {
        let boss = cfg.boss(boss_tier);
        let enemy_hp = boss.map_or(0, |b| b.hp);
        let enemy_atk = boss.map_or(0, |b| b.atk);
        let (outcome, round) = if enemy_hp == 0 { (Some(Side::Player), 0) } else { (None, 0) };
        Self {
            cfg,
            rng: GameRng::new(seed),
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

    /// 单步回合（`docs/business/combat.md` 回合流程，参数全走 `cfg` + `rng`，确定性）。
    ///
    /// 玩家先手：先结算玩家对 BOSS 的伤害，`boss_hp==0` → `Player` 胜（不再反击）；
    /// 否则 BOSS 反击结算玩家，`player_hp==0` → `Enemy` 胜；到 `max_rounds` 仍平局
    /// → `Enemy`（兜底判胜负，与 `docs/business/combat.md` 胜负判定表一致）。
    /// 已结束的会话 → 返回当前 HP + 已定结果，不再推进。
    #[must_use]
    pub fn step(&mut self) -> (u32, u32, Option<Side>) {
        if let Some(w) = self.outcome {
            return (self.player_hp, self.enemy_hp, Some(w));
        }
        self.round += 1;
        // 玩家先手：结算对 BOSS 伤害
        let pdmg = deal_damage(self.player_atk, &self.cfg.damage, &mut self.rng);
        self.enemy_hp = self.enemy_hp.saturating_sub(pdmg);
        if self.enemy_hp == 0 {
            self.outcome = Some(Side::Player);
            return (self.player_hp, self.enemy_hp, self.outcome);
        }
        // BOSS 反击
        let edmg = deal_damage(self.enemy_atk, &self.cfg.damage, &mut self.rng);
        self.player_hp = self.player_hp.saturating_sub(edmg);
        if self.player_hp == 0 {
            self.outcome = Some(Side::Enemy);
        } else if self.round >= self.cfg.max_rounds {
            // 兜底：超回合上限仍未分胜负 → `Enemy`（`docs/business/combat.md` 胜负判定表）
            self.outcome = Some(Side::Enemy);
        }
        (self.player_hp, self.enemy_hp, self.outcome)
    }

    /// 整场战斗（从当前状态跑完全部回合，返回 `(每回合 HP 轨迹, 胜方, 回合数)`）。
    ///
    /// 轨迹记录每步 `step` 后的双方 HP（`docs/business/combat.md`「逐回合快照」，
    /// 不含初始 HP 快照，只含每回合结束后的 HP）。
    #[must_use]
    pub fn finish(&mut self) -> (Vec<(u32, u32)>, Side, u32) {
        if let Some(w) = self.outcome {
            // 已立即结束（BOSS HP=0 / 越界档 → 无效战斗）
            return (Vec::new(), w, 0);
        }
        let mut trace: Vec<(u32, u32)> = Vec::new();
        for _ in 0..self.cfg.max_rounds {
            let (ph, eh, outcome) = self.step();
            trace.push((ph, eh));
            if let Some(w) = outcome {
                return (trace, w, self.round);
            }
        }
        (trace, self.outcome.unwrap_or(Side::Enemy), self.round)
    }
}
