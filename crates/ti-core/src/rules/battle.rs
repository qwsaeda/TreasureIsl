//! S3.8 / S8.3.a 战斗系统正式实现（数值数据化，S12.1 战斗域包）。
//!
//! 行为依据 `docs/business/combat.md`（唯一事实源）；数值单一事实源 =
//! `assets/data/balance.json → combat`（[`crate::data::balance`]），伤害模型/BOSS 档
//! 全由 `CombatCfg` 注入（逆向产物仅作参照，不作业务依据）。
//!
//! - 单位 [`Unit`]、阵营 [`Side`]（类型唯一家 [`crate::domain::combat`]，
//!   本模块直连 domain）；
//! - [`BattleSession`]：战斗会话（`CombatCfg` + `GameRng` + 双方 HP + 回合数 + 结果）；
//! - `from_balance` / `new_with_cfg` / `step` / `finish` / `boss_hp` 方法。
//!
//! 确定性（`docs/business/combat.md` 伤害模型）：
//! `damage = round(atk * atk_coef) + rng_offset`，`rng_offset ∈ [0, rng_span]`（走
//! [`GameRng::in_range`]，`rng_span == 0` 时无浮动），下限 `flat`；
//! 同 `seed` + 同 `CombatCfg` + 同操作序列 ⇒ 同 HP 轨迹/胜方/回合数
//! （EXP-040，可复现、可回放、可测试）。
//!
//! 回合流程（`docs/business/combat.md`）：按 `player_first` 定先手（默认 `true`）→
//! 先结算先手方伤害 → 归零即结束（后手不再反击）→ 否则结算后手方伤害 → 双方各判 HP；
//! `boss_hp==0` 立即 `Player` 胜，`player_hp==0` 判 `Enemy` 胜，超 `max_rounds` 仍平局 →
//! `Enemy`（兜底，同回合判 `Defeat`，与胜负判定表一致）。

use crate::data::{self, CombatCfg, DamageCfg};
use crate::domain::combat::{Side, Unit};
use crate::rules::boat::BoatSession;
use crate::rules::rng::GameRng;

/// 战斗加成（`docs/business/combat.md` §建造加成扩展）。
///
/// **总 atk / 总 hp = 建筑 Σ + 船累计值**：建筑加成 = `atk_bonus × level` 逐建筑求和
/// （[`crate::rules::build::BuildManager::build_atk_bonus`]），
/// 船加成 = `balance.boat.levels[level].atk/hp`（**累计值，直接加**，不经 `× level`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CombatBonus {
    /// atk 加成合计。
    pub atk: u32,
    /// hp 加成合计。
    pub hp: u32,
}

impl CombatBonus {
    /// 合成战斗加成（建筑 Σ + 船累计值；两者直接相加）。
    #[must_use]
    pub fn combine(build_atk: u32, build_hp: u32, boat: &BoatSession) -> Self {
        Self { atk: build_atk.saturating_add(boat.atk()), hp: build_hp.saturating_add(boat.hp()) }
    }

    /// 把加成叠加到基础单位（`base + bonus`；卡牌临时扣血等仍由调用方处理）。
    #[must_use]
    pub fn apply(self, base: Unit) -> Unit {
        Unit { hp: base.hp.saturating_add(self.hp), atk: base.atk.saturating_add(self.atk) }
    }
}

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

/// 从 `balance.combat` 读战斗参数（数值单一事实源；加载失败时**硬错误**，
/// 不 fallback，`docs/process/02_conventions.md` §4.4 数据失败=硬错误）。
fn load_cfg() -> Result<CombatCfg, data::LoadError> {
    Ok(data::balance()?.combat.clone())
}

/// 伤害结算（`docs/business/combat.md`）：`round(atk * atk_coef) + rng_offset`，下限 `flat`。
/// `rng_offset ∈ [0, rng_span]`（[`GameRng::in_range(rng_span + 1)`]，`rng_span == 0`
/// → 无浮动，纯 `round(atk * coef)`；数值单一事实源 `balance.combat.damage`）。
fn deal_damage(atk: u32, dmg: &DamageCfg, rng: &mut GameRng) -> u32 {
    let base = cast_atk_base(atk, dmg.atk_coef);
    let offset = if dmg.rng_span == 0 { 0 } else { rng.in_range(dmg.rng_span.saturating_add(1)) };
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
    ///
    /// `balance()` 加载失败返回 `Err`（硬错误，不 fallback，`02_conventions.md` §4.4）。
    ///
    /// # Errors
    ///
    /// [`crate::data::LoadError`]：`balance.json` 缺失 / 解析失败。
    pub fn from_balance(
        seed: u32,
        player: Unit,
        boss_tier: u32,
    ) -> Result<Self, crate::data::LoadError> {
        let cfg = load_cfg()?;
        Ok(Self::new_with_cfg(cfg, seed, player, boss_tier))
    }

    /// 新建会话并叠加战斗加成（建筑 Σ + 船累计值 → 玩家单位）。
    ///
    /// 等价于 `new_with_cfg(cfg, seed, bonus.apply(base), boss_tier)`：
    /// 玩家 atk/hp = 基础值 + [`CombatBonus`]（`docs/business/combat.md` §建造加成）。
    #[must_use]
    pub fn new_with_bonus(
        cfg: CombatCfg,
        seed: u32,
        base: Unit,
        boss_tier: u32,
        bonus: CombatBonus,
    ) -> Self {
        Self::new_with_cfg(cfg, seed, bonus.apply(base), boss_tier)
    }

    /// 显式 `CombatCfg` 注入（测试 / 自定义战斗参数，不读 `balance`）。
    #[must_use]
    pub fn new_with_cfg(cfg: CombatCfg, seed: u32, player: Unit, boss_tier: u32) -> Self {
        let boss = cfg.boss(boss_tier);
        let enemy_hp = boss.map_or(0, |b| b.hp);
        let enemy_atk = boss.map_or(0, |b| b.atk);
        // 越界档 → `enemy_hp == 0`：无效战斗，会话立即 `Player` 胜、`round = 0`
        // （`docs/business/combat.md` 可测试规则表「越界 tier ⇒ 无效战斗」，可回归测）
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

    /// 自定义敌方单位（S28.6 副本守军战 / 攻占战：敌方 hp/atk 由调用方注入，
    /// 数值走 `rules::territory::defender_unit`，不硬编码；确定性同 `from_balance`）。
    #[must_use]
    pub fn new_with_enemy(cfg: CombatCfg, seed: u32, player: Unit, enemy: Unit) -> Self {
        let (outcome, round) = if enemy.hp == 0 { (Some(Side::Player), 0) } else { (None, 0) };
        Self {
            cfg,
            rng: GameRng::new(seed),
            player_hp: player.hp,
            player_atk: player.atk,
            enemy_hp: enemy.hp,
            enemy_atk: enemy.atk,
            round,
            outcome,
        }
    }

    /// 副本守军战（`balance.combat` + 自定义敌方；数值单一事实源，加载失败 = 硬错误）。
    ///
    /// # Errors
    ///
    /// [`crate::data::LoadError`]：`balance.json` 缺失 / 解析失败。
    pub fn dungeon_from_balance(
        seed: u32,
        player: Unit,
        enemy: Unit,
    ) -> Result<Self, crate::data::LoadError> {
        Ok(Self::new_with_enemy(load_cfg()?, seed, player, enemy))
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
    /// 按 `cfg.player_first` 定顺序（默认 `true`）：
    /// - `true`：先结算玩家对 BOSS 伤害，`boss_hp==0` → `Player` 胜（不再反击）；否则 BOSS 反击结算玩家；
    /// - `false`：先结算 BOSS 对玩家伤害，`player_hp==0` → `Enemy` 胜（不再反击）；否则玩家结算 BOSS。
    ///
    /// 兜底规则：`player_hp==0` 判 `Enemy` 胜；`max_rounds` 仍平局 → `Enemy`（与胜负判定表一致）。
    /// 已结束的会话 → 返回当前 HP + 已定结果，不再推进。
    #[must_use]
    pub fn step(&mut self) -> (u32, u32, Option<Side>) {
        if let Some(w) = self.outcome {
            return (self.player_hp, self.enemy_hp, Some(w));
        }
        self.round += 1;
        if self.cfg.player_first {
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
        } else {
            // BOSS 先手：结算对玩家伤害
            let edmg = deal_damage(self.enemy_atk, &self.cfg.damage, &mut self.rng);
            self.player_hp = self.player_hp.saturating_sub(edmg);
            if self.player_hp == 0 {
                self.outcome = Some(Side::Enemy);
                return (self.player_hp, self.enemy_hp, self.outcome);
            }
            // 玩家反击
            let pdmg = deal_damage(self.player_atk, &self.cfg.damage, &mut self.rng);
            self.enemy_hp = self.enemy_hp.saturating_sub(pdmg);
            if self.enemy_hp == 0 {
                self.outcome = Some(Side::Player);
            } else if self.round >= self.cfg.max_rounds {
                // 兜底：超回合上限仍未分胜负 → `Enemy`（`docs/business/combat.md` 胜负判定表）
                self.outcome = Some(Side::Enemy);
            }
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
        // `step` 保证 `max_rounds` 内 outcome 必为 `Some`（`round >= max_rounds` 兜底），
        // 此处为类型完备的防御性分支（不可达；若未来引入 `max_rounds == 0` 边界再处理）。
        let winner = self.outcome.unwrap_or(Side::Enemy);
        (trace, winner, self.round)
    }
}
