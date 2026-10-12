//! S8.3.b 建造 / 银行业务规则测试（判据见 `docs/business/economy.md` 可测试规则表）。

use ti_core::data::BuildingDef;
use ti_core::rules::build::{Bank, BuildErr, BuildManager};

fn cfg() -> ti_core::data::BuildCfg {
    ti_core::data::BuildCfg {
        max_level: 3,
        buildings: vec![
            BuildingDef {
                id: "Bank".into(),
                name_key: "build.bank".into(),
                base_cost: 100,
                gold_max: 1_000,
            },
            BuildingDef {
                id: "Castle".into(),
                name_key: "build.castle".into(),
                base_cost: 500,
                gold_max: 5_000,
            },
        ],
    }
}

fn mgr(gold: u32) -> BuildManager {
    BuildManager::new(cfg(), Bank::new(gold), &[])
}

#[test]
fn build_debits_and_raises_cap() {
    let mut m = mgr(10_000);
    assert_eq!(m.build("Bank"), Ok(()), "affordable");
    assert_eq!(m.level("Bank"), 1);
    assert_eq!(m.bank().gold, 9_900, "cost = 100 x 1");
    assert_eq!(m.gold_cap(), 1_000, "建造抬升上限");
}

#[test]
fn build_cost_scales_with_level() {
    let m = mgr(0);
    assert_eq!(m.build_cost("Castle", 1), Some(500));
    assert_eq!(m.build_cost("Castle", 2), Some(1_000), "base x level");
    assert_eq!(m.build_cost("Castle", 3), Some(1_500));
    assert_eq!(m.build_cost("Nope", 1), None);
}

#[test]
fn build_unknown_building_rejected() {
    let mut m = mgr(10_000);
    assert_eq!(m.build("Nope"), Err(BuildErr::UnknownBuilding));
}

#[test]
fn build_insufficient_no_state_change() {
    let mut m = mgr(50);
    assert_eq!(m.build("Castle"), Err(BuildErr::Insufficient), "500 > 50");
    assert_eq!(m.level("Castle"), 0, "失败不改状态");
    assert_eq!(m.bank().gold, 50);
}

#[test]
fn build_unlocks_monotonic_cap() {
    let mut m = mgr(10_000);
    assert_eq!(m.gold_cap(), 0, "无建筑 → 无上限抬升");
    m.build("Bank").expect("bank");
    assert_eq!(m.gold_cap(), 1_000);
    m.build("Castle").expect("castle");
    assert_eq!(m.gold_cap(), 5_000, "取最大");
    m.build("Bank").expect("bank+1");
    assert_eq!(m.gold_cap(), 5_000, "低阶建筑不降低上限");
}

#[test]
fn build_respects_max_level() {
    let mut m = mgr(100_000);
    for _ in 0..3 {
        m.build("Bank").expect("level up");
    }
    assert_eq!(m.level("Bank"), 3);
    assert_eq!(m.build("Bank"), Err(BuildErr::InvalidLevel), "max_level = 3");
}

#[test]
fn deposit_withdraw_move_gold() {
    let mut m = mgr(5_000);
    assert_eq!(m.deposit(1_000), Ok(()));
    assert_eq!(m.bank().gold, 4_000);
    assert_eq!(m.bank().deposit, 1_000);
    m.withdraw(500);
    assert_eq!(m.bank().gold, 4_500);
    assert_eq!(m.bank().deposit, 500);
    assert_eq!(m.bank().total(), 5_000, "总额守恒");
}

#[test]
fn deposit_insufficient() {
    let mut m = mgr(100);
    assert_eq!(m.deposit(200), Err(BuildErr::Insufficient));
    assert_eq!(m.bank().gold, 100, "失败不改状态");
}

#[test]
fn withdraw_clamps_to_deposit() {
    let mut m = mgr(100);
    m.deposit(100).expect("deposit all");
    m.withdraw(999);
    assert_eq!(m.bank().deposit, 0, "取款不超过存款");
    assert_eq!(m.bank().gold, 100, "取回原流动金");
    assert_eq!(m.bank().total(), 100, "存取只是搬移，总额守恒");
}

#[test]
fn settle_overflow_moves_excess_to_deposit() {
    let mut m = mgr(2_000);
    m.build("Bank").expect("raise cap to 1000");
    // 2000 - 100 = 1900 流动，超过 cap 1000，溢出 900 全进银行
    let moved = m.settle_overflow(1.0);
    assert_eq!(moved, 900);
    assert_eq!(m.bank().gold, 1_000);
    assert_eq!(m.bank().deposit, 900);
}

#[test]
fn settle_overflow_rate_scales() {
    let mut m = mgr(2_000);
    m.build("Bank").expect("cap");
    let moved = m.settle_overflow(0.5);
    assert_eq!(moved, 450, "900 x 0.5");
    assert_eq!(m.bank().gold, 1_450, "剩余仍超 cap，下轮继续结算");
}

#[test]
fn economy_gold_never_exceeds_cap() {
    // 建造 Castle 抬上限到 5000，初始金 6000 → 结算后金币永远 ≤ 上限
    let mut m = BuildManager::new(
        ti_core::data::BuildCfg {
            max_level: 10,
            buildings: vec![BuildingDef {
                id: "Castle".into(),
                name_key: "build.castle".into(),
                base_cost: 500,
                gold_max: 5_000,
            }],
        },
        Bank::new(6_000),
        &[],
    );
    assert_eq!(m.gold_cap(), 0, "无建筑 → 无上限");
    m.build("Castle").expect("affordable: 500*1 <= 6000");
    let cap = m.gold_cap();
    assert_eq!(cap, 5_000, "上限须等于建筑 gold_max");
    // 结算溢出：6000 - 500 = 5500 流动 > cap 5000 → 溢出进银行
    let moved = m.settle_overflow(1.0);
    assert_eq!(moved, 500, "5500 - 5000 = 500 进银行");
    assert!(m.bank().gold <= cap, "结算后流动金须 ≤ 上限");
    assert_eq!(m.bank().gold, 5_000);
    assert_eq!(m.bank().deposit, 500);
}

#[test]
fn build_leveraged_from_balance_table() {
    // 真实 balance 表：成本/上限来自 JSON
    let b = ti_core::data::balance().expect("balance should load");
    let castle = b.build.buildings.iter().find(|x| x.id == "Castle").expect("Castle in balance");
    let mut m = BuildManager::new(b.build.clone(), Bank::new(castle.base_cost * 10), &[]);
    assert_eq!(m.build("Castle"), Ok(()));
    assert_eq!(m.gold_cap(), castle.gold_max, "上限须等于 JSON 里的 gold_max");
    assert_eq!(m.bank().gold, castle.base_cost * 9);
}
