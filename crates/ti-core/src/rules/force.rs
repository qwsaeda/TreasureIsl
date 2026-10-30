//! 兵力模型（v2，2026-10-30 用户定案）：人手 + 武器 **确定性 VS**。
//!
//! 单一事实源 = [`docs/business/combat.md`](../../../../docs/business/combat.md) §兵力模型；
//! 接线 = [`territory.md`](../../../../docs/business/territory.md) §4（攻占 / 多人同格）。
//!
//! 规则：
//! - **一人一把武器**：`武装 = min(人手, 武器)`（武器 > 人手 = 多余武器无效，瓶颈 = 人手）；
//!   人手可 > 或 < 武器（多余人手 = 徒手，战力折减）。
//! - **战力 = 武装 × weapon_coef + 徒手 × barehand_coef**（weapon_coef > barehand_coef）。
//! - **胜负确定性**：比较战力，高者胜；同战力 = 守方胜（防守优势）。
//! - **双方必损**：人手按 `loss_pct`、武器按 `weapon_loss_pct` 掉（胜者也损）。

use crate::data::ManpowerCfg;

/// 一方兵力（人手 + 武器）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Force {
    /// 人手（水手）。
    pub crew: u32,
    /// 武器。
    pub arms: u32,
}

impl Force {
    /// 构造兵力。
    #[must_use]
    pub const fn new(crew: u32, arms: u32) -> Self {
        Self { crew, arms }
    }

    /// 武装人手 = min(人手, 武器)（一人一把；武器多余 = 没人用，无效）。
    #[must_use]
    pub fn armed(&self) -> u32 {
        self.crew.min(self.arms)
    }

    /// 徒手人手 = 人手 − 武装（人手 > 武器时才有）。
    #[must_use]
    pub fn unarmed(&self) -> u32 {
        self.crew.saturating_sub(self.armed())
    }

    /// 战力 = 武装 × weapon_coef + 徒手 × barehand_coef。
    #[must_use]
    pub fn power(&self, cfg: &ManpowerCfg) -> u64 {
        u64::from(self.armed()) * u64::from(cfg.weapon_coef)
            + u64::from(self.unarmed()) * u64::from(cfg.barehand_coef)
    }

    /// 战后剩余（人手 / 武器都按 pct 损耗；胜者也损，`saturating_sub` 防下溢）。
    #[must_use]
    pub fn attrition(&self, cfg: &ManpowerCfg) -> Force {
        let crew_loss = apply_pct(self.crew, cfg.loss_pct);
        let arms_loss = apply_pct(self.arms, cfg.weapon_loss_pct);
        Force::new(self.crew.saturating_sub(crew_loss), self.arms.saturating_sub(arms_loss))
    }
}

/// 胜负方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceSide {
    /// 进攻方胜（战力 > 守方）。
    Attacker,
    /// 守方胜（战力 ≥ 攻方；防守优势）。
    Defender,
}

/// 兵力结算结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForceResult {
    /// 胜方。
    pub winner: ForceSide,
    /// 攻方战后剩余（必损后）。
    pub attacker_after: Force,
    /// 守方战后剩余（必损后）。
    pub defender_after: Force,
}

/// 确定性 VS：比较战力 → 判胜 → 双方必损。
///
/// - 攻方战力 **>** 守方 ⇒ 攻方胜；否则守方胜（防守优势）。
/// - 双方都按 `attrition` 损耗（人手 + 武器）。
#[must_use]
pub fn resolve(attack: &Force, defense: &Force, cfg: &ManpowerCfg) -> ForceResult {
    let a_pow = attack.power(cfg);
    let d_pow = defense.power(cfg);
    let winner = if a_pow > d_pow { ForceSide::Attacker } else { ForceSide::Defender };
    ForceResult {
        winner,
        attacker_after: attack.attrition(cfg),
        defender_after: defense.attrition(cfg),
    }
}

/// `v × pct / 10000`（pct 已钳到 0..=10000）。
fn apply_pct(v: u32, pct: u32) -> u32 {
    (u64::from(v) * u64::from(pct.min(10000)) / 10000) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ManpowerCfg {
        ManpowerCfg {
            weapon_coef: 2,
            barehand_coef: 1,
            loss_pct: 1000,        // 10%
            weapon_loss_pct: 2000, // 20%
            initial_crew: 20,
            initial_arms: 5,
        }
    }

    #[test]
    fn armed_caps_at_crew() {
        // 武器 > 人手：武装 = 人手，多余武器无效（一人一把）。
        let f = Force::new(5, 20);
        assert_eq!(f.armed(), 5);
        assert_eq!(f.unarmed(), 0);
        // 人手 > 武器：武装 = 武器，多余徒手。
        let g = Force::new(20, 5);
        assert_eq!(g.armed(), 5);
        assert_eq!(g.unarmed(), 15);
    }

    #[test]
    fn power_weights_armed_over_barehand() {
        let c = cfg();
        // 武装 5 → 5×2=10；徒手 15 → 15×1=15；总 25。
        let g = Force::new(20, 5);
        assert_eq!(g.power(&c), 25);
        // 全武装 5 → 5×2=10。
        let h = Force::new(5, 5);
        assert_eq!(h.power(&c), 10);
    }

    #[test]
    fn attacker_power_wins() {
        let c = cfg();
        let a = Force::new(30, 30); // 30×2=60
        let d = Force::new(20, 20); // 20×2=40
        assert_eq!(resolve(&a, &d, &c).winner, ForceSide::Attacker);
    }

    #[test]
    fn defender_wins_on_tie_or_higher() {
        let c = cfg();
        let a = Force::new(20, 20);
        let d = Force::new(20, 20);
        // 同战力 → 守方胜（防守优势）。
        assert_eq!(resolve(&a, &d, &c).winner, ForceSide::Defender);
        // 守方更强 → 守方胜。
        let d2 = Force::new(30, 30);
        assert_eq!(resolve(&a, &d2, &c).winner, ForceSide::Defender);
    }

    #[test]
    fn both_sides_attrition() {
        let c = cfg();
        let a = Force::new(20, 10);
        let d = Force::new(15, 8);
        let r = resolve(&a, &d, &c);
        // 攻方 20 人手损 10% = 2 → 18；10 武器损 20% = 2 → 8。
        assert_eq!(r.attacker_after, Force::new(18, 8));
        // 守方 15 人手损 10% = 1 → 14；8 武器损 20% = 1 → 7。
        assert_eq!(r.defender_after, Force::new(14, 7));
    }

    #[test]
    fn attrition_caps_at_zero() {
        // 损耗 100%：人手/武器清零，不 panic（saturating）。
        let full = ManpowerCfg { loss_pct: 10000, weapon_loss_pct: 10000, ..cfg() };
        let f = Force::new(5, 3);
        assert_eq!(f.attrition(&full), Force::new(0, 0));
    }

    #[test]
    fn deterministic_same_input_same_result() {
        let c = cfg();
        let a = Force::new(25, 25);
        let d = Force::new(18, 18);
        assert_eq!(resolve(&a, &d, &c), resolve(&a, &d, &c));
    }
}
