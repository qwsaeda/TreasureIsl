//! 贸易台账集成测试（S52.1；`docs/business/economy.md` 可测规则表）。
//!
//! S52.1 补测：`trade.rs` `good_def` / `set_buy_cost` 此前 0 集成测试（飞轮 R2 找问题 P1-6）。
//! oracle = `economy.md`：售价 = round(buy × markup)；有效上限 = `min(goods.qty_max, cargo_cap)`。
//! 判据：`cargo test -p ti-core --test trade_rules` 全绿。

use ti_core::data::{GoodsDef, TradeCfg};
use ti_core::rules::trade::{TradeErr, TradeLedger};

fn cfg() -> TradeCfg {
    TradeCfg {
        markup: 1.5,
        goods: vec![
            GoodsDef { id: 1, name_key: "goods.1".into(), buy: 20, qty_max: 10 },
            GoodsDef { id: 2, name_key: "goods.2".into(), buy: 50, qty_max: 5 },
        ],
        default_cargo_cap: 0,
    }
}

fn ledger(gold: u32) -> TradeLedger {
    TradeLedger::new_unlimited(cfg(), gold, &[(1, 0), (2, 0)])
}

/// `good_def`：目录内 = Some（值对齐），目录外 = None。
#[test]
fn good_def_some_in_dir_none_out() {
    let led = ledger(1000);
    assert_eq!(led.good_def(1).map(|g| g.buy), Some(20), "目录内 buy = 20");
    assert_eq!(led.good_def(2).map(|g| g.qty_max), Some(5), "目录内 qty_max = 5");
    assert_eq!(led.good_def(99), None, "目录外 = None");
}

/// `set_buy_cost`（商人抬价面）：写入 `buy_cost` 记账表（`avg_buy_price` 均价 = 累计成本/持有量）。
///
/// S52.1 审计注：`buy_price` 跟随 `cfg.goods[].buy`（目录价，§商人定价），
/// `set_buy_cost` 只影响 `avg_buy_price` 记账面（盈亏统计），不改变成交单价。
/// 两语义分离 = 成交价锚目录、均价记实际（商人抬价后均价 ≠ 目录价）。
#[test]
fn set_buy_cost_updates_avg_buy_price() {
    let mut led = ledger(1000);
    // 目录价 buy = 20（`buy_price` 跟随 cfg）。
    led.buy(1, 2).expect("买 2 件 @20");
    assert_eq!(led.buy_price(1), Some(20), "成交单价 = 目录 buy（不随 set_buy_cost 变）");
    // 预置均价 30（均价 = buy_cost/qty = 60/2 = 30；`set_buy_cost` 写的是总价非单价）。
    led.set_buy_cost(1, 60);
    assert_eq!(led.avg_buy_price(1), 30, "均价 = 60/2 = 30");
    // 无持有商品 = 均价 0（源 doc「无记录 → 0」；记账表有但 0 持有不除零）。
    led.set_buy_cost(99, 40);
    assert_eq!(led.owned(99), 0, "追加商品 0 持有");
    assert_eq!(led.avg_buy_price(99), 0, "0 持有 = 均价 0（不除零）");
    assert_eq!(led.good_def(99), None, "追加商品不在目录（good_def 仍 None）");
}

/// 买入超有效上限 ⇒ `QtyOutOfRange`（`economy.md`：0 ≤ qty ≤ `min(qty_max, cargo_cap)`）。
#[test]
fn buy_over_cap_rejected() {
    let mut led = ledger(1000);
    let r = led.buy(2, 6); // qty_max = 5
    assert!(matches!(r, Err(TradeErr::QtyOutOfRange)), "超 qty_max = 拒");
    led.set_cargo_cap(3); // 船舱上限收紧
    let r2 = led.buy(1, 4); // min(10, 3) = 3
    assert!(matches!(r2, Err(TradeErr::QtyOutOfRange)), "超船舱上限 = 拒");
}

/// 卖出 = round(buy × markup) × qty（`economy.md` 售价公式；金币随卖出增加）。
#[test]
fn sell_uses_markup_price() {
    let mut led = ledger(1000);
    led.buy(1, 4).expect("买 4 件 @20 = 80 金");
    assert_eq!(led.gold(), 920, "买后金 = 1000 - 80");
    let res = led.sell(1, 4).expect("卖 4 件");
    assert_eq!(res.amount, 30 * 4, "售价 = round(20 × 1.5) × 4 = 30 × 4");
    assert_eq!(led.gold(), 1040, "卖后金 = 920 + 120（售价 round(20×1.5)=30/件 × 4 = 120）");
}
