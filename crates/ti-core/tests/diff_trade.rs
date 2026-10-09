//! S3.6 贸易系统（固定价格表买卖结算）差分测试。
//!
//! 遍历（固定价格表 5 商品 × buy/sell × 各金量/持有量组合 × 错误路径），
//! oracle 与 impl 逐位一致（Result 的 Ok(TradeResult)/Err(TradeErr) 全等）；
//! 另加语义测试（金够买入 1 件 `gold_left` 正确、金不够 → `Err(InsufficientGold)`、
//! 卖出 3 件 profit = 3×(卖价-均价)）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::trade::{GOODS_CATALOG, PriceTable, TradeErr, oracle_buy, oracle_sell};
use ti_core::rules::trade::TradeLedger;

/// 5 商品占位价格表（spec §a，来自 `sub_546E0` 初值）。
fn placeholder_table() -> PriceTable {
    PriceTable::placeholder()
}

/// 断言 `oracle_buy` 与 `TradeLedger::buy` 逐位一致（同输入同输出）。
///
/// 注意：`oracle_buy` 是纯函数（不改状态），`TradeLedger::buy` 改内部状态，
/// 因此比对的是 `Result<TradeResult, TradeErr>` 全等（含 `gid`/`paid`/`gold_left`/`profit` 四字段）。
fn assert_buy_equal(table: &PriceTable, gid: u32, gold: u32) {
    let o = oracle_buy(table, gid, gold);
    let mut ledger = TradeLedger::new(gold, table.clone(), &[]);
    let i = ledger.buy(gid);
    assert_eq!(o, i, "oracle_buy vs TradeLedger::buy (gid={gid}, gold={gold}) 不一致");
}

/// 断言 `oracle_sell` 与 `TradeLedger::sell` 逐位一致。
///
/// 为保证 `avg_buy_price` 匹配 `oracle_sell` 的 `avg` 参数，
/// 当 `avg > 0 && owned > 0` 时，通过 `avg × owned` 的买入成本预置 `buy_cost`：
/// 直接构造 `TradeLedger` 并用 `owned` 持有量 + `buy_cost = avg × owned` 使
/// `avg_buy_price = (avg × owned) / owned = avg`。
fn assert_sell_equal(table: &PriceTable, gid: u32, owned: u32, gold: u32, avg: u32) {
    let o = oracle_sell(table, gid, owned, gold, avg);
    let goods: Vec<(u32, u32)> = if owned > 0 { vec![(gid, owned)] } else { vec![] };
    let mut ledger = TradeLedger::new(gold, table.clone(), &goods);
    // 预置 buy_cost 使 avg_buy_price = avg（当 avg > 0 时）
    if avg > 0 && owned > 0 {
        let cost = avg.saturating_mul(owned);
        ledger.set_buy_cost(gid, cost);
    }
    let i = ledger.sell(gid);
    assert_eq!(
        o, i,
        "oracle_sell vs TradeLedger::sell (gid={gid}, owned={owned}, gold={gold}, avg={avg}) 不一致"
    );
}

/// 遍历 5 商品 × 金量 {0, 10, 20, 75, 100, 200} 的买入路径。
#[test]
fn diff_buy_all_goods_gold_combos() {
    let table = placeholder_table();
    let golds = [0u32, 10, 20, 75, 100, 200];
    for g in GOODS_CATALOG {
        for &gold in &golds {
            assert_buy_equal(&table, g.id, gold);
        }
    }
}

/// 遍历 5 商品 × 持有量 {0, 1, 2, 3, 5} × 金量 {0, 50, 100} × 均价 {0, 5, 20, 100} 的卖出路径。
#[test]
fn diff_sell_all_goods_combos() {
    let table = placeholder_table();
    let owneds = [0u32, 1, 2, 3, 5];
    let golds = [0u32, 50, 100];
    let avgs = [0u32, 5, 20, 100];
    for g in GOODS_CATALOG {
        for &owned in &owneds {
            for &gold in &golds {
                for &avg in &avgs {
                    assert_sell_equal(&table, g.id, owned, gold, avg);
                }
            }
        }
    }
}

/// 未知商品（gid=0 和 gid=99）的买入/卖出错误路径。
#[test]
fn diff_unknown_good_error_paths() {
    let table = placeholder_table();
    for &gid in &[0u32, 99] {
        assert_buy_equal(&table, gid, 1000);
        assert_sell_equal(&table, gid, 3, 100, 0);
    }
}

/// 语义测试 (a)：金够买入 1 件商品 1（价格 20），`gold_left` = 100-20 = 80。
#[test]
fn semantic_buy_one_gold_left() {
    let table = placeholder_table();
    let r = oracle_buy(&table, 1, 100).expect("金 100 够买商品 1（价格 20）");
    assert_eq!(r.paid, 20, "买入 1 件商品 1，paid = 20");
    assert_eq!(r.gold_left, 80, "gold_left = 100 - 20 = 80");
    assert_eq!(r.profit, 0, "买入不产生利润");

    // impl 一致
    let mut ledger = TradeLedger::new(100, table.clone(), &[]);
    let i = ledger.buy(1).expect("impl buy 商品 1");
    assert_eq!(r, i, "oracle 与 impl 买入结果应逐位一致");
    assert_eq!(ledger.gold(), 80, "impl gold() 应为 80");
}

/// 语义测试 (b)：金不够（10 < 76）→ `Err(InsufficientGold)`。
#[test]
fn semantic_insufficient_gold() {
    let table = placeholder_table();
    let o = oracle_buy(&table, 2, 10);
    assert!(
        matches!(o, Err(TradeErr::InsufficientGold)),
        "金 10 < 商品 2 价格 76 → InsufficientGold"
    );

    let mut ledger = TradeLedger::new(10, table.clone(), &[]);
    let i = ledger.buy(2);
    assert_eq!(o, i, "oracle 与 impl 金不足错误应一致");
    assert!(matches!(i, Err(TradeErr::InsufficientGold)), "impl 也应为 InsufficientGold");
}

/// 语义测试 (c)：卖出 3 件商品 1（价格 20，均价 15）→ profit = 3×(20-15) = 15。
#[test]
fn semantic_sell_3_profit() {
    let table = placeholder_table();
    let r = oracle_sell(&table, 1, 3, 0, 15).expect("卖出 3 件商品 1");
    assert_eq!(r.paid, 60, "paid = 3 × 20 = 60");
    assert_eq!(r.gold_left, 60, "gold_left = 0 + 60 = 60");
    assert_eq!(r.profit, 15, "profit = 3 × (20 - 15) = 15");

    // impl 一致（持有 3 件，均价 15）：通过预置 buy_cost 使 avg_buy_price = 15
    let mut ledger = TradeLedger::new(0, table.clone(), &[(1, 3)]);
    ledger.set_buy_cost(1, 15 * 3); // cost = 45，qty = 3，avg = 45/3 = 15
    let i = ledger.sell(1).expect("impl 卖出 3 件商品 1");
    assert_eq!(r, i, "oracle 与 impl 卖出结果应逐位一致");
    assert_eq!(i.profit, 15, "profit = 3 × (20 - 15) = 15");

    // impl 独立验证：profit_since（无买入成本 → avg=0 → profit = 全价 × 持有）
    let ledger2 = TradeLedger::new(0, table.clone(), &[(1, 3)]);
    let ps = ledger2.profit_since(1);
    assert_eq!(ps, 60, "无买入成本 → profit_since = 价格 20 × 3 = 60");

    // 有买入成本时 profit_since = (价格 - 均价) × 持有
    let mut ledger3 = TradeLedger::new(100, table.clone(), &[]);
    // 买入 3 件商品 1（价格 20，金 100 足够）
    for _ in 0..3 {
        ledger3.buy(1).expect("买入 3 件商品 1");
    }
    assert_eq!(ledger3.owned(1), 3, "持有 3 件商品 1");
    assert_eq!(ledger3.gold(), 40, "金 = 100 - 3×20 = 40");
    assert_eq!(ledger3.avg_buy_price(1), 20, "均价 = 3×20 / 3 = 20");
    // profit_since = (20 - 20) × 3 = 0（均价 = 价格，无利润）
    assert_eq!(ledger3.profit_since(1), 0, "均价 = 价格 → profit = 0");
}

/// 遍历事件码 0..=50：`oracle_trade_event` 与 spec §d 表对照。
#[test]
fn diff_trade_event_codes() {
    use ti_core::oracle::trade::oracle_trade_event;
    // 已知事件码
    let expected = [
        (22u32, true),
        (23, true),
        (13, true),
        (37, true),
        (38, true),
        (39, true),
        (40, true),
        (0, false),
        (1, false),
        (14, false),
        (21, false),
        (100, false),
    ];
    for &(code, should_be_some) in &expected {
        let r = oracle_trade_event(code);
        assert_eq!(
            r.is_some(),
            should_be_some,
            "事件码 {code} 应为 {}，实际 {:?}",
            if should_be_some { "Some" } else { "None" },
            r
        );
    }
}

/// 遍历 5 商品 × `oracle_price` seed {0, 1, 5, 10, 15, 100} 的价格查询。
#[test]
fn diff_oracle_price_all_seeds() {
    let table = placeholder_table();
    for g in GOODS_CATALOG {
        for &seed in &[0u32, 1, 5, 10, 15, 100] {
            let p = table.oracle_price(g.id, seed);
            assert!(p.is_some(), "商品 {} 价格查询应 Some（seed={seed}）", g.id);
            let p = p.expect("已断言 Some");
            if seed == 0 || seed % 10 == 0 {
                assert_eq!(p, g.base_price, "seed % 10 == 0 → 固定价格");
            } else {
                assert_eq!(p, g.base_price.saturating_sub(seed % 10), "seed != 0 → base - seed%10");
            }
        }
    }
}
