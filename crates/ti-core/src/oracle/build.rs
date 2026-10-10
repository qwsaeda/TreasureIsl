//! oracle：S3.7 建造/城堡系统差分基准，测试专用。
//!
//! 直译 `docs/spec/build.md` 的建造/银行规则：
//! - 建筑类型 [`Building`]（4 种：Castle/Investment/Bank/Fort，spec §a 推测）；
//! - 银行 [`Bank`]（金量，spec §c）；
//! - [`oracle_build_cost`]：建造成本公式（`base × level`，spec §b 推测占位）；
//! - [`oracle_can_afford`]：金量是否足够（spec §b）；
//! - [`oracle_build`]：建造操作（扣金 + 升级，spec §b/c）；
//! - [`oracle_deposit`] / [`oracle_withdraw`]：银行存取（spec §c，`银行` 双槽位）；
//! - 错误 [`BuildErr`]（`Insufficient` / `InvalidLevel`）。
//!
//! 行为确定（同输入逐位相同），不允许 rand。
//!
//! 建造成本公式（**推测**，spec §b）：
//! ```text
//! cost = base_cost(kind) × (level + 1)
//! ```
//! 各建筑基础成本（spec §a 占位，对照 `投资` 奖励阶梯）：
//! - Castle = 1000
//! - Investment = 500
//! - Bank = 200
//! - Fort = 800

/// 建筑类型（spec §a，4 种，**推测** 由控件组合推断）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Building {
    /// 城堡（`button_aimcastle` + `button_build`）。
    Castle,
    /// 投资设施（`button_invest`）。
    Investment,
    /// 银行（`button_bank`）。
    Bank,
    /// 据点/堡垒（`button_capture` + `button_build`，**推测**）。
    Fort,
}

impl Building {
    /// 全部建筑类型（按枚举声明顺序，diff 测试遍历用）。
    #[must_use]
    pub const fn all() -> [Building; 4] {
        [Self::Castle, Self::Investment, Self::Bank, Self::Fort]
    }

    /// 基础建造成本（spec §a 占位，对照 `投资` 奖励阶梯量级）。
    #[must_use]
    pub const fn base_cost(self) -> u32 {
        match self {
            Self::Castle => 1000,
            Self::Investment => 500,
            Self::Bank => 200,
            Self::Fort => 800,
        }
    }
}

/// 银行（金量，spec §c，`银行` 双槽位金 `+1032/+1040` 的 oracle 化）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bank {
    /// 当前金量（上限 999999，`投资` 行 114/116）。
    pub gold: u32,
}

impl Bank {
    /// 新建银行（指定金量）。
    #[must_use]
    pub const fn new(gold: u32) -> Self {
        Self { gold }
    }
}

/// 建造/银行操作错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildErr {
    /// 金量不足（`bank.gold < cost`，spec §b）。
    Insufficient,
    /// 等级无效（`level` 超出范围，spec §b）。
    InvalidLevel,
}

/// oracle：建造成本（`base_cost(kind) × (level + 1)`，spec §b 推测公式）。
///
/// `level = 0`（首次建造）→ `base × 1`；`level = 1`（升级）→ `base × 2`；以此类推。
#[must_use]
pub fn oracle_build_cost(kind: Building, level: u32) -> u32 {
    kind.base_cost() * (level + 1)
}

/// oracle：金量是否足够支付建造成本（spec §b）。
#[must_use]
pub fn oracle_can_afford(bank: Bank, cost: u32) -> bool {
    bank.gold >= cost
}

/// oracle：建造操作（spec §b/c，`银行` 存款/取款 + `投资` 奖励的 oracle 化）。
///
/// 规则：
/// - `level > 10` → `InvalidLevel`（上限 10 级，**推测**）；
/// - `bank.gold < cost` → `Insufficient`（`oracle_can_afford` 等价）；
/// - 成功 → 扣金，返回新 `Bank`。
///
/// # Errors
///
/// - [`BuildErr::InvalidLevel`]：`level > 10`；
/// - [`BuildErr::Insufficient`]：`bank.gold < oracle_build_cost(kind, level)`。
pub fn oracle_build(bank: Bank, kind: Building, level: u32) -> Result<Bank, BuildErr> {
    if level > 10 {
        return Err(BuildErr::InvalidLevel);
    }
    let cost = oracle_build_cost(kind, level);
    if !oracle_can_afford(bank, cost) {
        return Err(BuildErr::Insufficient);
    }
    Ok(Bank::new(bank.gold - cost))
}

/// oracle：存款（spec §c，`银行` `a2==0, mode==2` 方向，金从流动→银行）。
///
/// `amount > bank.gold` → `Insufficient`（不能存超过持有的金）；
/// 成功 → 金量减少 `amount`（银行金增加，oracle 只跟踪流动金）。
///
/// # Errors
///
/// - [`BuildErr::Insufficient`]：`amount > bank.gold`。
pub fn oracle_deposit(bank: Bank, amount: u32) -> Result<Bank, BuildErr> {
    if amount > bank.gold {
        return Err(BuildErr::Insufficient);
    }
    Ok(Bank::new(bank.gold - amount))
}

/// oracle：取款（spec §c，`银行` `a2!=0, mode==2` 方向，金从银行→流动）。
///
/// 金量上限 999999（`投资` 行 114/116，`0xF423F`）；
/// 成功 → 金量增加 `amount`（截断至 999999）。
///
/// 永不失败（金量只增不减，上限截断），返回新 `Bank`。
#[must_use]
pub fn oracle_withdraw(bank: Bank, amount: u32) -> Bank {
    let new_gold = bank.gold.saturating_add(amount).min(999_999);
    Bank::new(new_gold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_cost_levels() {
        assert_eq!(oracle_build_cost(Building::Castle, 0), 1000);
        assert_eq!(oracle_build_cost(Building::Castle, 1), 2000);
        assert_eq!(oracle_build_cost(Building::Castle, 5), 6000);
        assert_eq!(oracle_build_cost(Building::Bank, 0), 200);
    }

    #[test]
    fn build_insufficient() {
        let bank = Bank::new(500);
        assert_eq!(oracle_build(bank, Building::Castle, 0), Err(BuildErr::Insufficient));
    }

    #[test]
    fn build_success() {
        let bank = Bank::new(5000);
        let result = oracle_build(bank, Building::Castle, 0);
        assert!(result.is_ok(), "build 5000 gold for Castle L0 should succeed");
        let Bank { gold } = result.unwrap_or_default();
        assert_eq!(gold, 4000);
    }

    #[test]
    fn build_invalid_level() {
        let bank = Bank::new(999_999);
        assert_eq!(oracle_build(bank, Building::Castle, 11), Err(BuildErr::InvalidLevel));
    }

    #[test]
    fn deposit_success() {
        let bank = Bank::new(5000);
        let result = oracle_deposit(bank, 1000);
        assert!(result.is_ok(), "deposit 1000 from 5000 should succeed");
        let Bank { gold } = result.unwrap_or_default();
        assert_eq!(gold, 4000);
    }

    #[test]
    fn deposit_insufficient() {
        let bank = Bank::new(500);
        assert_eq!(oracle_deposit(bank, 1000), Err(BuildErr::Insufficient));
    }

    #[test]
    fn withdraw_cap() {
        let bank = Bank::new(999_000);
        let result = oracle_withdraw(bank, 2000);
        assert_eq!(result.gold, 999_999, "should cap at 999_999");
    }

    #[test]
    fn withdraw_normal() {
        let bank = Bank::new(1000);
        let result = oracle_withdraw(bank, 500);
        assert_eq!(result.gold, 1500);
    }
}
