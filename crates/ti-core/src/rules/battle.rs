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
use crate::domain::combat::{CombatAction, ItemEffect, Side, Unit};
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
    player_hp_max: u32,
    player_atk: u32,
    enemy_hp: u32,
    enemy_atk: u32,
    round: u32,
    outcome: Option<Side>,
    /// 剩余生效回合数（朗姆酒增益，S27.7；`0` = 无增益）。
    atk_buff_rounds: u32,
    /// 增益百分比（`50` = ×1.5；`atk_buff_rounds == 0` 时不生效）。
    atk_buff_pct: u32,
    /// 敌方下一次反击落空（烟雾弹，S27.7；消耗一次即清）。
    enemy_staggered: bool,
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
            player_hp_max: player.hp,
            player_atk: player.atk,
            enemy_hp,
            enemy_atk,
            round,
            outcome,
            atk_buff_rounds: 0,
            atk_buff_pct: 0,
            enemy_staggered: false,
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
            player_hp_max: player.hp,
            player_atk: player.atk,
            enemy_hp: enemy.hp,
            enemy_atk: enemy.atk,
            round,
            outcome,
            atk_buff_rounds: 0,
            atk_buff_pct: 0,
            enemy_staggered: false,
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

    /// 当前玩家 HP（只读；S27.7 血量回复后调用方需要读回）。
    #[must_use]
    pub fn player_hp(&self) -> u32 {
        self.player_hp
    }

    /// 玩家初始 HP（回复上限；`S27.7` `heal_pct` 的分母）。
    #[must_use]
    pub fn player_hp_max(&self) -> u32 {
        self.player_hp_max
    }

    /// 剩余增益回合数（`0` = 无增益；测试与 HUD 用）。
    #[must_use]
    pub fn atk_buff_rounds(&self) -> u32 {
        self.atk_buff_rounds
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

    /// 单步回合（默认动作 = 攻击；等价于 `step_with(CombatAction::Attack)`）。
    ///
    /// 保留旧 API 语义：`docs/business/combat.md` 回合流程，参数全走 `cfg` + `rng`，确定性。
    #[must_use]
    pub fn step(&mut self) -> (u32, u32, Option<Side>) {
        self.step_with(CombatAction::Attack)
    }

    /// 单步回合（S27.7：动作 = 攻击 / 防御 / 道具，`docs/business/combat.md` §战斗动作）。
    ///
    /// 结算顺序（两种先手分支同构）：
    /// 1. 按 `cfg.player_first` 决定先结算谁；
    /// 2. 玩家动作（攻击 / 防御 / 道具）→ 敌 HP 归零即 `Player` 胜，不再反击；
    /// 3. 敌方反击（`Stagger` 消耗 1 次落空；`Defend` 按 `defend_reduction_pct` 减伤）
    ///    → 玩家 HP 归零即 `Enemy` 胜；
    /// 4. 回合末增益计数 −1（使用道具当回合不计）。
    ///
    /// 确定性：道具结算**不消耗随机数**（`GameRng` 状态不变），同 seed + 同动作序列 ⇒ 同轨迹。
    /// 已结束的会话 → 返回当前 HP + 已定结果，不再推进。
    #[must_use]
    pub fn step_with(&mut self, action: CombatAction) -> (u32, u32, Option<Side>) {
        if let Some(w) = self.outcome {
            return (self.player_hp, self.enemy_hp, Some(w));
        }
        self.round += 1;
        let defending = matches!(action, CombatAction::Defend);
        let buffed_now = if self.cfg.player_first {
            let applied = self.apply_player_action(action);
            if self.enemy_hp == 0 {
                self.outcome = Some(Side::Player);
                return (self.player_hp, self.enemy_hp, self.outcome);
            }
            self.enemy_strike(defending);
            if self.player_hp == 0 {
                self.outcome = Some(Side::Enemy);
            }
            applied
        } else {
            self.enemy_strike(defending);
            if self.player_hp == 0 {
                self.outcome = Some(Side::Enemy);
                return (self.player_hp, self.enemy_hp, self.outcome);
            }
            let applied = self.apply_player_action(action);
            if self.enemy_hp == 0 {
                self.outcome = Some(Side::Player);
            }
            applied
        };
        if self.outcome.is_none() && self.round >= self.cfg.max_rounds {
            // 兜底：超回合上限仍未分胜负 → `Enemy`（`docs/business/combat.md` 胜负判定表）
            self.outcome = Some(Side::Enemy);
        }
        // 回合末增益时效（使用当回合不计；`rounds` = 剩余生效攻击次数）
        if !buffed_now && self.atk_buff_rounds > 0 {
            self.atk_buff_rounds -= 1;
        }
        (self.player_hp, self.enemy_hp, self.outcome)
    }

    /// 玩家动作结算（返回「本回合是否是增益道具」，供回合末时效判定）。
    fn apply_player_action(&mut self, action: CombatAction) -> bool {
        match action {
            CombatAction::Attack => {
                let atk = self.buffed_atk();
                let pdmg = deal_damage(atk, &self.cfg.damage, &mut self.rng);
                self.enemy_hp = self.enemy_hp.saturating_sub(pdmg);
            }
            CombatAction::Defend => {}
            CombatAction::Item(effect) => match effect {
                ItemEffect::Damage(v) => {
                    // 固定伤害：不吃 atk、不吃随机（不推进 `GameRng`）
                    self.enemy_hp = self.enemy_hp.saturating_sub(v);
                }
                ItemEffect::HealPct(pct) => {
                    let heal = self.player_hp_max.saturating_mul(pct.min(100)) / 100;
                    self.player_hp = self.player_hp.saturating_add(heal).min(self.player_hp_max);
                }
                ItemEffect::AtkBuff { pct, rounds } => {
                    self.atk_buff_pct = pct;
                    self.atk_buff_rounds = rounds;
                    return true;
                }
                ItemEffect::Stagger => self.enemy_staggered = true,
            },
        }
        false
    }

    /// 敌方反击（`Stagger` 落空 / `Defend` 减伤；确定性走 `GameRng`）。
    fn enemy_strike(&mut self, defending: bool) {
        if self.enemy_staggered {
            // 烟雾弹：本次反击落空，消耗 1 次（不消耗随机数）
            self.enemy_staggered = false;
            return;
        }
        let mut edmg = deal_damage(self.enemy_atk, &self.cfg.damage, &mut self.rng);
        if defending {
            let keep = 100u32.saturating_sub(self.cfg.defend_reduction_pct.min(100));
            edmg = edmg.saturating_mul(keep) / 100;
        }
        self.player_hp = self.player_hp.saturating_sub(edmg);
    }

    /// 增益后的 atk（无增益 = 原值；`pct` 按百分比向上取整到整数伤害由伤害模型统一处理）。
    fn buffed_atk(&self) -> u32 {
        if self.atk_buff_rounds == 0 || self.atk_buff_pct == 0 {
            return self.player_atk;
        }
        self.player_atk.saturating_mul(100u32.saturating_add(self.atk_buff_pct)) / 100
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
