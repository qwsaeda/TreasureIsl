//! oracle：建造/银行差分基准，测试专用（S11 起降级为纯基准函数）。
//!
//! 业务规则以 `docs/business/economy.md` 为准；造价以 `balance.json` 为准（可调）。
//!
//! 类型分工（P2 经济域收口）：`Building` 域类型来自 `ti_core::domain::economy`；
//! 银行与错误类型是 oracle 差分基准专用形态（含 `deposit` 追踪与
//! `UnknownBuilding` 变体），正式实现用 `rules::build::{Bank, BuildErr}`。

pub use ti_core::domain::economy::Building;

/// 银行（oracle 差分基准形态：流动金 + 已存金）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bank {
    /// 流动金。
    pub gold: u32,
    /// 银行累计。
    pub deposit: u32,
}

impl Bank {
    /// 新建银行（流动金 `gold`，存款 0）。
    #[must_use]
    pub const fn new(gold: u32) -> Self {
        Self { gold, deposit: 0 }
    }
}

/// 建造/银行操作错误（oracle 差分基准专用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildErr {
    /// 金量不足。
    Insufficient,
    /// 等级无效。
    InvalidLevel,
    /// 未知建筑 id（oracle `oracle_build` 按字符串 id 校验路径）。
    UnknownBuilding,
}

/// oracle：建筑基础造价（逆向初值，冻结为历史基准）。
///
/// ⚠️ 正式实现 `rules::build::cost_of` 用 `balance.build.buildings[i].base_cost`（可调），
/// 本表冻结不改（S11 起不再挂在 `Building` 类型上，避免业务类型携带逆向数值）。
#[must_use]
pub const fn oracle_base_cost(kind: Building) -> u32 {
    match kind {
        Building::Castle => 1000,
        Building::Investment => 500,
        Building::Bank => 200,
        Building::Fort => 800,
    }
}

/// oracle：建造成本（`oracle_base_cost(kind) × (level + 1)`，逆向推测公式）。
///
/// `level = 0`（首次建造）→ `base × 1`；`level = 1`（升级）→ `base × 2`；以此类推。
#[must_use]
pub fn oracle_build_cost(kind: Building, level: u32) -> u32 {
    oracle_base_cost(kind) * (level + 1)
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
    Ok(Bank { gold: bank.gold - amount, deposit: bank.deposit.saturating_add(amount) })
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
    Bank { gold: new_gold, deposit: bank.deposit }
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
        let Bank { gold, .. } = result.unwrap_or_default();
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
        let Bank { gold, deposit } = result.unwrap_or_default();
        assert_eq!(gold, 4000);
        assert_eq!(deposit, 1000, "存款累计到 deposit");
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
        assert_eq!(result.deposit, 0, "oracle 取款不触碰 deposit（原版语义）");
    }
}
