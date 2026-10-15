//! S8.3.b 建造 / 银行系统正式实现（数值从 `assets/data/balance.json` 注入）。
//!
//! 业务规格（唯一事实源）：[`docs/business/economy.md`](../../../docs/business/economy.md)。
//! 设计要点：
//! - **单调升级链**：每种建筑可反复升级（`base_cost × (level+1)` 递增）；
//! - **建造抬升金量上限**：`gold_cap = max(已建建筑的 gold_max)`；
//! - **银行承接溢出**：金币超过上限时溢出部分按 `bank_rate` 进银行；
//! - 纯逻辑、零 IO、确定性。
//!
//! S8 定位变更：不再取逆向「推测」成本表，改为读 balance.json。

use crate::data::BuildCfg;
use crate::domain::economy::Building;

/// S10.3 改动 4：建造造价纯函数（`balance.build.buildings` 表，不依赖逆向成本表）。
///
/// 查 `cfg.get(kind_id(kind)).base_cost`；查不到 → 0（调用方应据 0 拒建，不 panic）。
/// `kind_id` = `Building` 枚举 → 字符串 id（与 `balance.build.buildings[].id` 对齐）。
#[must_use]
pub fn cost_of(cfg: &BuildCfg, kind: Building) -> u32 {
    cfg.get(kind_id(kind)).map_or(0, |b| b.base_cost)
}

/// `Building` 枚举 → `balance.build.buildings[].id` 字符串（与 JSON 对齐）。
#[must_use]
pub fn kind_id(kind: Building) -> &'static str {
    match kind {
        Building::Castle => "Castle",
        Building::Investment => "Investment",
        Building::Bank => "Bank",
        Building::Fort => "Fort",
    }
}

/// 把非负有限 `f64` 钳到 `u32` 区间后取整（避免直接 cast 触发 clippy pedantic）。
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::trivially_copy_pass_by_ref
)]
fn f64_to_u32(v: f64) -> Option<u32> {
    if !v.is_finite() || v < 0.0 {
        return None;
    }
    Some(v.round().min(f64::from(u32::MAX)) as u32)
}

/// 建造错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildErr {
    /// 未知建筑 id。
    UnknownBuilding,
    /// 金不足。
    Insufficient,
    /// 等级越界（`level >= max_level`）。
    InvalidLevel,
}

/// 银行：流动金 + 已存金。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bank {
    /// 流动金。
    pub gold: u32,
    /// 银行累计（承接溢出）。
    pub deposit: u32,
}

impl Bank {
    /// 新建银行（流动金 `gold`，存款 0）。
    #[must_use]
    pub fn new(gold: u32) -> Self {
        Self { gold, deposit: 0 }
    }

    /// 总资产（流动 + 存款）。
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn total(&self) -> u32 {
        self.gold.saturating_add(self.deposit)
    }
}

/// 建造管理器：银行 + 建筑等级表 + 配置。
#[derive(Debug, Clone)]
pub struct BuildManager {
    bank: Bank,
    buildings: Vec<(String, u32)>,
    cfg: BuildCfg,
}

impl BuildManager {
    /// 新建建造管理器（注入配置 + 初始银行 + 已建建筑 `(id, level)`）。
    #[must_use]
    pub fn new(cfg: BuildCfg, bank: Bank, buildings: &[(String, u32)]) -> Self {
        Self { bank, buildings: buildings.to_vec(), cfg }
    }

    /// 银行只读视图。
    #[must_use]
    pub fn bank(&self) -> &Bank {
        &self.bank
    }

    /// 建筑列表只读视图（`(id, level)`）。
    #[must_use]
    pub fn buildings(&self) -> &[(String, u32)] {
        &self.buildings
    }

    /// 某建筑当前等级（无记录 → 0）。
    #[must_use]
    pub fn level(&self, id: &str) -> u32 {
        self.buildings.iter().find(|(k, _)| k == id).map_or(0, |&(_, l)| l)
    }

    /// 建造 `id` 到目标等级 `level`（`level` 为**目标等级**，从当前 +1 逐级建）。
    ///
    /// 造价 = `base_cost × (level)`（第 `level` 级花费 `base_cost × level`），
    /// 与 `docs/business/economy.md` 的 `base_cost × (level+1)` 对齐（`level` 从 0 起算到 `level`）。
    #[must_use]
    pub fn build_cost(&self, id: &str, level: u32) -> Option<u32> {
        self.cfg.get(id).map(|b| b.base_cost.saturating_mul(level.max(1)))
    }

    /// 当前金量上限（`max(已建建筑的 gold_max)`；无建筑 → 基础上限 0）。
    #[must_use]
    pub fn gold_cap(&self) -> u32 {
        self.buildings
            .iter()
            .filter(|(_, lvl)| *lvl > 0)
            .filter_map(|(id, _)| self.cfg.get(id))
            .map(|b| b.gold_max)
            .max()
            .unwrap_or(0)
    }

    /// 全部已建建筑提供的战斗 atk 总加成（`Σ atk_bonus × level`，`docs/business/combat.md` §建造加成）。
    ///
    /// 无建筑 → 0。成长机制入口：`start_battle` 用 `player_atk + build_atk_bonus()`。
    #[must_use]
    pub fn build_atk_bonus(&self) -> u32 {
        self.buildings.iter().map(|(id, lvl)| self.cfg.atk_bonus_at(id, *lvl)).sum()
    }

    /// 全部已建建筑提供的战斗 hp 总加成（`Σ hp_bonus × level`，同上）。
    ///
    /// 无建筑 → 0。成长机制入口：`start_battle` 用 `player.hp + build_hp_bonus()`。
    #[must_use]
    pub fn build_hp_bonus(&self) -> u32 {
        self.buildings.iter().map(|(id, lvl)| self.cfg.hp_bonus_at(id, *lvl)).sum()
    }

    /// 金量是否足够支付 `cost`。
    #[must_use]
    pub fn can_afford(&self, cost: u32) -> bool {
        self.bank.gold >= cost
    }

    /// 建造/升级 `id` 一级（等级 +1）。
    ///
    /// 成功 → 扣金 + 等级 +1 + 金量上限抬升；失败状态不变。
    ///
    /// # Errors
    ///
    /// - [`BuildErr::UnknownBuilding`]：`id` 不在目录；
    /// - [`BuildErr::InvalidLevel`：已达 `max_level`；
    /// - [`BuildErr::Insufficient`]：金不足。
    pub fn build(&mut self, id: &str) -> Result<(), BuildErr> {
        let Some(def) = self.cfg.get(id) else {
            return Err(BuildErr::UnknownBuilding);
        };
        let base_cost = def.base_cost;
        let cur = self.level(id);
        if cur >= self.cfg.max_level {
            return Err(BuildErr::InvalidLevel);
        }
        let next = cur + 1;
        let cost = base_cost.saturating_mul(next);
        if !self.can_afford(cost) {
            return Err(BuildErr::Insufficient);
        }
        self.bank.gold -= cost;
        match self.buildings.iter_mut().find(|(k, _)| k == id) {
            Some(entry) => entry.1 = next,
            None => self.buildings.push((id.to_string(), next)),
        }
        Ok(())
    }

    /// 存款（流动金 → 银行）。
    ///
    /// # Errors
    ///
    /// - [`BuildErr::Insufficient`]：`amount > bank.gold`。
    pub fn deposit(&mut self, amount: u32) -> Result<(), BuildErr> {
        if amount > self.bank.gold {
            return Err(BuildErr::Insufficient);
        }
        self.bank.gold -= amount;
        self.bank.deposit = self.bank.deposit.saturating_add(amount);
        Ok(())
    }

    /// 设置银行流动金（测试/存档恢复用；正式流程经 `deposit`/`withdraw`/
    /// `settle_overflow` 搬移，不直接改）。
    pub fn set_bank_gold(&mut self, gold: u32) {
        self.bank.gold = gold;
    }

    /// 取款（银行 → 流动金，永不失败）。
    pub fn withdraw(&mut self, amount: u32) {
        let a = amount.min(self.bank.deposit);
        self.bank.deposit -= a;
        self.bank.gold = self.bank.gold.saturating_add(a);
    }

    /// 结算溢出：若 `gold > gold_cap`，超出部分按 `bank_rate` 进银行。
    ///
    /// 判据：`docs/business/economy.md`「金币达到上限，溢出部分进银行累计」。
    /// 返回本次进银行的金量。
    pub fn settle_overflow(&mut self, rate: f32) -> u32 {
        let cap = self.gold_cap();
        if cap == 0 || self.bank.gold <= cap {
            return 0;
        }
        let excess = self.bank.gold - cap;
        let raw = f64::from(excess) * f64::from(rate);
        let to_bank = f64_to_u32(raw);
        let Some(to_bank) = to_bank else {
            return 0;
        };
        if to_bank == 0 {
            return 0;
        }
        self.bank.gold -= to_bank;
        self.bank.deposit = self.bank.deposit.saturating_add(to_bank);
        to_bank
    }
}
