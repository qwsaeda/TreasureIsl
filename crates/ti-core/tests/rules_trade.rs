//! S8.3.b 贸易系统业务规则测试（判据见 `docs/business/economy.md` 可测试规则表）。
//!
//! 数值全部由 `TradeCfg` 注入（夹具或真实 `balance.json`），不在测试里硬编码业务表。

use ti_core::data::{BuildCfg, BuildingDef, GoodsDef};
use ti_core::rules::build::{Bank, BuildManager};
use ti_core::rules::trade::{TradeErr, TradeLedger};

fn cfg_build() -> BuildCfg {
    BuildCfg {
        max_level: 3,
        buildings: vec![
            BuildingDef {
                id: "Bank".into(),
                name_key: "build.bank".into(),
                base_cost: 100,
                gold_max: 1_000,
                atk_bonus: 0,
                hp_bonus: 0,
            },
            BuildingDef {
                id: "Castle".into(),
                name_key: "build.castle".into(),
                base_cost: 500,
                gold_max: 5_000,
                atk_bonus: 0,
                hp_bonus: 0,
            },
        ],
    }
}

fn cfg() -> ti_core::data::TradeCfg {
    ti_core::data::TradeCfg {
        markup: 1.5,
        goods: vec![
            GoodsDef { id: 1, name_key: "goods.1".into(), buy: 20, qty_max: 10 },
            GoodsDef { id: 2, name_key: "goods.2".into(), buy: 50, qty_max: 5 },
        ],
    }
}

fn ledger(gold: u32) -> TradeLedger {
    TradeLedger::new_unlimited(cfg(), gold, &[])
}

#[test]
fn trade_buy_debits_gold_adds_goods() {
    let mut l = ledger(1000);
    let r = l.buy(1, 3).expect("buy should succeed");
    assert_eq!(r.amount, 60, "3 x 20");
    assert_eq!(l.gold(), 940);
    assert_eq!(l.owned(1), 3);
}

#[test]
fn trade_sell_credits_gold_removes_goods() {
    let mut l = ledger(1000);
    l.buy(1, 3).expect("buy");
    let r = l.sell(1, 2).expect("sell");
    assert_eq!(r.amount, 60, "2 x round(20*1.5)");
    assert_eq!(l.gold(), 1000);
    assert_eq!(l.owned(1), 1);
}

#[test]
fn trade_sell_price_from_markup() {
    let l = ledger(0);
    assert_eq!(l.sell_price(1), Some(30), "round(20 * 1.5)");
    assert_eq!(l.buy_price(2), Some(50));
}

#[test]
fn trade_qty_bounds_enforced() {
    let mut l = ledger(10_000);
    assert_eq!(l.buy(1, 0), Err(TradeErr::QtyOutOfRange));
    assert_eq!(l.buy(1, 11), Err(TradeErr::QtyOutOfRange), "exceeds qty_max 10");
    l.buy(1, 10).expect("fill up");
    assert_eq!(l.buy(1, 1), Err(TradeErr::QtyOutOfRange));
}

#[test]
fn trade_unknown_good_rejected() {
    let mut l = ledger(1000);
    assert_eq!(l.buy(999, 1), Err(TradeErr::UnknownGood));
}

#[test]
fn trade_insufficient_gold() {
    let mut l = ledger(10);
    assert_eq!(l.buy(2, 1), Err(TradeErr::InsufficientGold), "50 > 10");
    assert_eq!(l.gold(), 10, "失败不改状态");
}

#[test]
fn trade_sell_no_stock() {
    let mut l = ledger(1000);
    assert_eq!(l.sell(1, 1), Err(TradeErr::NoStock));
}

#[test]
fn trade_sell_zero_qty_sells_all() {
    let mut l = ledger(1000);
    l.buy(1, 4).expect("buy");
    let r = l.sell(1, 0).expect("sell all");
    assert_eq!(r.qty, 4);
    assert_eq!(l.owned(1), 0);
}

#[test]
fn trade_avg_buy_price_is_cost_over_qty() {
    let mut l = ledger(10_000);
    l.buy(2, 2).expect("buy 2 x 50");
    assert_eq!(l.avg_buy_price(2), 50);
    assert_eq!(l.avg_buy_price(1), 0, "未持有 → 0");
}

#[test]
fn trade_profit_uses_markup_margin() {
    let mut l = ledger(10_000);
    l.buy(1, 2).expect("buy 2 x 20 = 40");
    assert_eq!(l.profit_since(1), 20, "(30-20) x 2");
}

#[test]
fn trade_leveraged_from_balance_table() {
    // 真实 balance 表：价格与上限来自 JSON，不是硬编码
    let b = ti_core::data::balance().expect("balance should load");
    let mut l = TradeLedger::new(b.trade.clone(), 1_000_000, &[], b.economy.gold_max);
    let gid = b.trade.goods[0].id;
    let price = b.trade.goods[0].buy;
    let r = l.buy(gid, 1).expect("buy from real table");
    assert_eq!(r.amount, price, "成交额须等于 JSON 里的 buy");
    assert_eq!(l.owned(gid), 1);
    assert_eq!(l.qty_cap(gid), b.trade.goods[0].qty_max, "上限来自 JSON");
}

#[test]
fn trade_sell_clamped_to_gold_cap() {
    // 金量上限 100，卖出一件得 30 金（markup 1.5），初始金 90
    // 结算 90 + 30 = 120，钳到 100，溢出 20
    let mut l = TradeLedger::new(cfg(), 90, &[(1, 1)], 100);
    let r = l.sell(1, 1).expect("sell");
    assert_eq!(r.gold_left, 100, "卖出加金须钳到 gold_cap");
    assert_eq!(r.overflow, 20, "溢出 = 120 - 100 = 20");
    assert_eq!(l.gold(), 100);
}

#[test]
fn trade_sell_no_cap_when_zero() {
    // gold_cap = 0 表示不限制，卖出加金不钳制
    let mut l = TradeLedger::new_unlimited(cfg(), 90, &[(1, 1)]);
    let r = l.sell(1, 1).expect("sell");
    assert_eq!(r.gold_left, 120, "无上限时 90+30=120");
    assert_eq!(r.overflow, 0, "无上限时不产生溢出");
}

#[test]
fn trade_set_gold_cap_midway() {
    // 运行中通过建造抬升上限（set_gold_cap）：
    // 初始 50 金，持有 10 件，gold_cap=0（无限制）
    // 卖掉 5 件得 150 金 → 金 200
    // 然后设 gold_cap=180（模拟建造抬上限但未到位）
    // 再卖 5 件 → 200+150=350，钳到 180，溢出 170
    let mut l = TradeLedger::new_unlimited(cfg(), 50, &[(1, 10)]);
    let r = l.sell(1, 5).expect("sell 5");
    assert_eq!(r.gold_left, 200, "50 + 150 = 200");
    l.set_gold_cap(180);
    let r = l.sell(1, 5).expect("sell 5 again");
    assert_eq!(r.gold_left, 180, "结算 200+150=350 钳到 180");
    assert_eq!(r.overflow, 170, "350 - 180 = 170 进银行");
}

/// 规则表「建筑单调解锁」对应用例名（economy.md §可测试规则表第 7 条；
/// 行为本体在 `rules_build::build_unlocks_monotonic_cap`）。
#[test]
fn build_unlocks_monotonic() {
    let mut m = BuildManager::new(cfg_build(), Bank::new(10_000), &[]);
    m.build("Bank").expect("affordable");
    assert_eq!(m.level("Bank"), 1);
    m.build("Castle").expect("affordable");
    assert_eq!(m.level("Castle"), 1);
    assert_eq!(m.gold_cap(), 5_000, "上限取已建建筑 gold_max 最大值");
}

/// 规则表「金币恒 ≤ 上限（银行承接溢出）」的贸易侧衔接：
/// 卖出钳制产生的 overflow 由调用方经 `BuildManager::settle_overflow` 入银行。
#[test]
fn trade_overflow_handoff_to_bank_settle() {
    let b = ti_core::data::balance().expect("balance should load");
    let mut m = BuildManager::new(
        b.build.clone(),
        Bank::new(b.economy.initial_gold.saturating_add(10_000)),
        &[],
    );
    // 首建 Castle 抬上限
    m.build("Castle").expect("affordable");
    let cap = m.gold_cap();
    assert!(cap > 0, "Castle 建成后须抬升上限");
    // 模拟贸易侧结算后流动金超上限：卖出加金钳 cap，overflow 返回
    let surplus = cap.saturating_add(300);
    m.set_bank_gold(surplus);
    let moved = m.settle_overflow(b.economy.bank_rate);
    assert!(moved > 0, "超额部分须按 bank_rate 进银行");
    assert!(m.bank().gold <= cap, "结算后流动金恒 ≤ 上限");
    // balance.json 当前 bank_rate = 1.0（c2 §4：断言读 JSON 实际值，不抄公式）
    assert!(
        (b.economy.bank_rate - 1.0).abs() < f32::EPSILON,
        "bank_rate 须为 1.0（当前 balance.json 值）"
    );
    assert_eq!(moved, surplus - cap, "rate=1.0 时溢出全额进银行");
}
