//! S3.7 建造/城堡系统正式实现。
//!
//! 与 `oracle::build` 行为逐位一致（同一输入产生逐位相同的
//! 建造成本/金量判定/建造/存取结果），但结构不同：用 `BuildManager`
//! 结构体 + 方法承载状态，而非自由函数。
//!
//! 依据 `docs/spec/build.md`：
//! - 建筑类型 [`Building`]、银行 [`Bank`]、错误 [`BuildErr`]（从 oracle 重导出）；
//! - [`BuildManager`]：建造管理器（银行 + 建筑列表）；
//! - `build_cost` / `can_afford` / `build` / `deposit` / `withdraw` / `level` 方法。

use crate::oracle::build::{
    Bank, BuildErr, Building, oracle_build, oracle_build_cost, oracle_can_afford, oracle_deposit,
    oracle_withdraw,
};

/// 建造管理器：银行 + 建筑列表（spec §b/c，`sub_1B9AC` 双槽位金的 oracle 化）。
#[derive(Debug, Clone)]
pub struct BuildManager {
    bank: Bank,
    buildings: Vec<(Building, u32)>,
}

impl BuildManager {
    /// 新建建造管理器（指定初始银行金量、建筑列表 `(kind, level)`）。
    #[must_use]
    pub fn new(bank: Bank, buildings: &[(Building, u32)]) -> Self {
        Self { bank, buildings: buildings.to_vec() }
    }

    /// 当前银行（只读）。
    #[must_use]
    pub fn bank(&self) -> &Bank {
        &self.bank
    }

    /// 建筑列表（只读）。
    #[must_use]
    pub fn buildings(&self) -> &[(Building, u32)] {
        &self.buildings
    }

    /// 某建筑当前等级（无记录 → 0）。
    #[must_use]
    pub fn level(&self, kind: Building) -> u32 {
        self.buildings.iter().find(|(k, _)| *k == kind).map_or(0, |&(_, l)| l)
    }

    /// 建造成本（行为与 `oracle_build_cost` 逐位一致）。
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn build_cost(&self, kind: Building, level: u32) -> u32 {
        oracle_build_cost(kind, level)
    }

    /// 金量是否足够（行为与 `oracle_can_afford` 逐位一致）。
    #[must_use]
    pub fn can_afford(&self, cost: u32) -> bool {
        oracle_can_afford(self.bank, cost)
    }

    /// 建造操作（行为与 `oracle_build` 逐位一致，成功时更新建筑列表）。
    ///
    /// 成功 → 扣金 + 建筑等级 +1；失败 → 返回 `Err`，状态不变。
    ///
    /// # Errors
    ///
    /// - [`BuildErr::InvalidLevel`]：`level > 10`；
    /// - [`BuildErr::Insufficient`]：金量不足。
    pub fn build(&mut self, kind: Building, level: u32) -> Result<(), BuildErr> {
        let new_bank = oracle_build(self.bank, kind, level)?;
        self.bank = new_bank;
        if let Some(entry) = self.buildings.iter_mut().find(|(k, _)| *k == kind) {
            entry.1 += 1;
        } else {
            self.buildings.push((kind, 1));
        }
        Ok(())
    }

    /// 存款（行为与 `oracle_deposit` 逐位一致）。
    ///
    /// # Errors
    ///
    /// - [`BuildErr::Insufficient`]：`amount > bank.gold`。
    pub fn deposit(&mut self, amount: u32) -> Result<(), BuildErr> {
        self.bank = oracle_deposit(self.bank, amount)?;
        Ok(())
    }

    /// 取款（行为与 `oracle_withdraw` 逐位一致，永不失败）。
    pub fn withdraw(&mut self, amount: u32) {
        self.bank = oracle_withdraw(self.bank, amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_build_and_level() {
        let mut m = BuildManager::new(Bank::new(10_000), &[]);
        assert_eq!(m.build(Building::Castle, 0), Ok(()), "build should succeed with 10000 gold");
        assert_eq!(m.level(Building::Castle), 1);
        assert_eq!(m.bank().gold, 9000);
    }

    #[test]
    fn manager_build_insufficient() {
        let mut m = BuildManager::new(Bank::new(500), &[]);
        assert_eq!(m.build(Building::Castle, 0), Err(BuildErr::Insufficient));
        assert_eq!(m.level(Building::Castle), 0, "失败不改变状态");
    }

    #[test]
    fn manager_deposit_withdraw() {
        let mut m = BuildManager::new(Bank::new(5000), &[]);
        assert_eq!(m.deposit(1000), Ok(()), "deposit 1000 from 5000 should succeed");
        assert_eq!(m.bank().gold, 4000);
        m.withdraw(2000);
        assert_eq!(m.bank().gold, 6000);
    }
}
