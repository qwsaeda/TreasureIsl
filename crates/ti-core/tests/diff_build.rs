//! S3.7 建造/城堡系统差分测试。
//!
//! 遍历 4 建筑类型 × level 0..=5 × 各 bank 金量 × 存款/取款 金额组合，
//! oracle 与 impl 逐位一致；另加 2 个语义测试（金够建造→新 level、金不够→Err(Insufficient)）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

#![allow(clippy::unwrap_used, clippy::unreachable, clippy::missing_docs_in_private_items)]

use ti_core::oracle::build::{
    Bank, BuildErr, Building, oracle_build, oracle_build_cost, oracle_can_afford, oracle_deposit,
    oracle_withdraw,
};
use ti_core::rules::build::BuildManager;

/// 断言 oracle 与 impl 的 `build_cost` 逐位一致。
fn assert_cost_equal(kind: Building, level: u32) {
    let o = oracle_build_cost(kind, level);
    let m = BuildManager::new(Bank::new(0), &[]);
    let i = m.build_cost(kind, level);
    assert_eq!(o, i, "oracle_build_cost vs build_cost ({kind:?} L{level}) 不一致");
}

/// 断言 oracle 与 impl 的 `can_afford` 逐位一致。
fn assert_afford_equal(gold: u32, cost: u32) {
    let bank = Bank::new(gold);
    let o = oracle_can_afford(bank, cost);
    let m = BuildManager::new(bank, &[]);
    let i = m.can_afford(cost);
    assert_eq!(o, i, "oracle_can_afford vs can_afford (gold={gold} cost={cost}) 不一致");
}

/// 断言 oracle 与 impl 的 build 结果逐位一致。
fn assert_build_equal(gold: u32, kind: Building, level: u32) {
    let bank = Bank::new(gold);
    let o = oracle_build(bank, kind, level);
    let mut m = BuildManager::new(bank, &[]);
    let i = m.build(kind, level);
    let o_kind = o.is_ok();
    let i_kind = i.is_ok();
    assert_eq!(o_kind, i_kind, "build 结果类型不一致 (gold={gold} {kind:?} L{level})");
    if o_kind {
        assert_eq!(o.expect("ok").gold, m.bank().gold, "build 成功金量不一致");
    } else {
        assert_eq!(o.unwrap_err(), i.unwrap_err(), "build 错误不一致");
    }
}

/// 断言 oracle 与 impl 的 deposit 结果逐位一致。
fn assert_deposit_equal(gold: u32, amount: u32) {
    let bank = Bank::new(gold);
    let o = oracle_deposit(bank, amount);
    let mut m = BuildManager::new(bank, &[]);
    let i = m.deposit(amount);
    let o_kind = o.is_ok();
    let i_kind = i.is_ok();
    assert_eq!(o_kind, i_kind, "deposit 结果类型不一致 (gold={gold} amount={amount})");
    if o_kind {
        assert_eq!(o.expect("ok").gold, m.bank().gold, "deposit 成功金量不一致");
    } else {
        assert_eq!(o.unwrap_err(), i.unwrap_err(), "deposit 错误不一致");
    }
}

/// 断言 oracle 与 impl 的 withdraw 结果逐位一致（永不失败）。
fn assert_withdraw_equal(gold: u32, amount: u32) {
    let bank = Bank::new(gold);
    let o = oracle_withdraw(bank, amount);
    let mut m = BuildManager::new(bank, &[]);
    m.withdraw(amount);
    assert_eq!(o.gold, m.bank().gold, "withdraw 金量不一致 (gold={gold} amount={amount})");
}

/// 遍历 4 建筑类型 × level 0..=5：`oracle_build_cost` vs `build_cost`。
#[test]
fn diff_build_cost_all_levels() {
    for kind in Building::all() {
        for level in 0..=5u32 {
            assert_cost_equal(kind, level);
        }
    }
}

/// 遍历 4 建筑类型 × level 0..=5 × 各金量：`oracle_can_afford` vs `can_afford`。
#[test]
fn diff_can_afford_all_gold() {
    for kind in Building::all() {
        for level in 0..=5u32 {
            let cost = oracle_build_cost(kind, level);
            for gold in [0u32, 1, 100, 500, 1000, 5000, 10_000, 50_000, 999_999] {
                assert_afford_equal(gold, cost);
            }
        }
    }
}

/// 遍历 4 建筑类型 × level 0..=5 × 各金量：`oracle_build` vs `build`。
#[test]
fn diff_build_all_combos() {
    for kind in Building::all() {
        for level in 0..=5u32 {
            for gold in [0u32, 100, 1000, 5000, 10_000, 50_000, 999_999] {
                assert_build_equal(gold, kind, level);
            }
        }
    }
}

/// 遍历各金量 × 各存款金额：`oracle_deposit` vs `deposit`。
#[test]
fn diff_deposit_all_amounts() {
    for gold in [0u32, 100, 1000, 5000, 999_999] {
        for amount in [0u32, 1, 100, 1000, 5000, 999_999] {
            assert_deposit_equal(gold, amount);
        }
    }
}

/// 遍历各金量 × 各取款金额：`oracle_withdraw` vs `withdraw`。
#[test]
fn diff_withdraw_all_amounts() {
    for gold in [0u32, 100, 1000, 5000, 999_000, 999_999] {
        for amount in [0u32, 1, 100, 1000, 5000, 999_999] {
            assert_withdraw_equal(gold, amount);
        }
    }
}

/// 语义测试 (a)：金够建造 → 成功 + 新 level。
#[test]
fn semantic_build_success_increments_level() {
    let mut m = BuildManager::new(Bank::new(10_000), &[]);
    assert_eq!(m.level(Building::Castle), 0, "初始 level=0");
    m.build(Building::Castle, 0).expect("10000 >= 1000");
    assert_eq!(m.level(Building::Castle), 1, "建造后 level=1");
    assert_eq!(m.bank().gold, 9000, "金扣 1000");

    // 再次建造（level=1 升级，成本 = 1000×2 = 2000）
    m.build(Building::Castle, 1).expect("9000 >= 2000");
    assert_eq!(m.level(Building::Castle), 2, "升级后 level=2");
    assert_eq!(m.bank().gold, 7000, "金扣 2000");
}

/// 语义测试 (b)：金不够 → Err(Insufficient)。
#[test]
fn semantic_build_insufficient_gold() {
    let mut m = BuildManager::new(Bank::new(500), &[]);
    let result = m.build(Building::Castle, 0);
    assert_eq!(result, Err(BuildErr::Insufficient), "500 < 1000 应失败");
    assert_eq!(m.level(Building::Castle), 0, "失败后 level 不变");
    assert_eq!(m.bank().gold, 500, "失败后金量不变");

    // 不同建筑同样失败（oracle 直验）
    let bank = Bank::new(100);
    assert_eq!(oracle_build(bank, Building::Fort, 0), Err(BuildErr::Insufficient));
}
