//! S8.3.b 贸易系统业务规则测试（判据见 `docs/business/economy.md` 可测试规则表）。
//!
//! 数值全部由 `TradeCfg` 注入（夹具或真实 `balance.json`），不在测试里硬编码业务表。

use ti_core::data::GoodsDef;
use ti_core::rules::trade::{TradeErr, TradeLedger};

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
    TradeLedger::new(cfg(), gold, &[])
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
    let mut l = TradeLedger::new(b.trade.clone(), 1_000_000, &[]);
    let gid = b.trade.goods[0].id;
    let price = b.trade.goods[0].buy;
    let r = l.buy(gid, 1).expect("buy from real table");
    assert_eq!(r.amount, price, "成交额须等于 JSON 里的 buy");
    assert_eq!(l.owned(gid), 1);
    assert_eq!(l.qty_cap(gid), b.trade.goods[0].qty_max, "上限来自 JSON");
}
