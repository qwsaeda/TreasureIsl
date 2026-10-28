//! oracle 贸易差分基准（S11 降级为纯基准函数；类型在 domain::economy）。
//!
//! 差分基准 = 从 `legacy/docs/spec/*.md` 伪代码直译的纯函数，供 `tests/diff_*.rs` 对账
//! 正向实现（`rules::`）；本模块**不参与运行时**，S11 起物理归档 `legacy/` 前保留为对照基准。


pub use crate::domain::economy::{
    GOODS_CATALOG, Goods, PriceTable, TRADE_EVENT_CODES, TradeEvent, trade_event_from_code,
};

/// 贸易事件码判定（spec §d，给定事件码返回对应的 [`TradeEvent`]；未知码 → `None`）。
/// 转发到 `domain::economy::trade_event_from_code`（域内唯一事实源）。
#[must_use]
pub fn oracle_trade_event(code: u32) -> Option<TradeEvent> {
    trade_event_from_code(code)
}

/// 贸易错误（oracle 差分基准专用；正式实现用 `rules::trade::TradeErr`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeErr {
    /// 金不足（`gold < price`）。
    InsufficientGold,
    /// 未知商品（`gid` 不在价格表中）。
    UnknownGood,
    /// 无库存（`owned == 0`）。
    NoStock,
}

/// 贸易结算结果（oracle 差分基准专用；正式实现用 `rules::trade::TradeResult`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeResult {
    /// 商品 id。
    pub gid: u32,
    /// 支付/获得金额。
    pub paid: u32,
    /// 结算后剩余金。
    pub gold_left: u32,
    /// 利润（买入 = 0；负值钳 0）。
    pub profit: i32,
}

/// oracle：买入结算（`单件买入` a2=1，spec §c 买入公式）。
///
/// 规则（spec §c 直译）：
/// - 查不到商品 → `Err(UnknownGood)`；
/// - 金不足（`gold < price`）→ `Err(InsufficientGold)`；
/// - 否则：`paid = price`，`gold_left = gold - price`，`profit = 0`（买入不产生利润）。
///
/// # Errors
///
/// - [`TradeErr::UnknownGood`]：`gid` 不在价格表中；
/// - [`TradeErr::InsufficientGold`]：`gold < price`。
pub fn oracle_buy(table: &PriceTable, gid: u32, gold: u32) -> Result<TradeResult, TradeErr> {
    let price = table.lookup(gid).ok_or(TradeErr::UnknownGood)?;
    if gold < price {
        return Err(TradeErr::InsufficientGold);
    }
    Ok(TradeResult { gid, paid: price, gold_left: gold - price, profit: 0 })
}

/// oracle：卖出结算（`单件买入` a2=0，spec §c 卖出公式）。
///
/// 规则（spec §c 直译）：
/// - 查不到商品 → `Err(UnknownGood)`；
/// - `owned == 0` → `Err(NoStock)`；
/// - `gold` 为卖出前金（不影响利润计算，仅影响 `gold_left`）；
/// - `avg_buy_price` = 历史买入均价（`单件买入` 行 128-135 公式的 oracle 简化：
///   由调用方提供，若 `avg == 0` 则视为无历史买入 → `profit = price`）；
/// - `paid = price × owned`（卖出获得），`gold_left = gold + paid`，
///   `profit = (price - avg_buy_price) × owned`（负值钳 0，`单件买入` 行 134/231）。
///
/// # Errors
///
/// - [`TradeErr::UnknownGood`]：`gid` 不在价格表中；
/// - [`TradeErr::NoStock`]：`owned == 0`。
pub fn oracle_sell(
    table: &PriceTable,
    gid: u32,
    owned: u32,
    gold: u32,
    avg_buy_price: u32,
) -> Result<TradeResult, TradeErr> {
    let price = table.lookup(gid).ok_or(TradeErr::UnknownGood)?;
    if owned == 0 {
        return Err(TradeErr::NoStock);
    }
    let gain = price.saturating_mul(owned);
    let unit_profit = if avg_buy_price > 0 { price.saturating_sub(avg_buy_price) } else { price };
    let profit = i32::try_from(unit_profit.saturating_mul(owned)).unwrap_or(i32::MAX);
    Ok(TradeResult { gid, paid: gain, gold_left: gold.saturating_add(gain), profit })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> PriceTable {
        PriceTable::new(GOODS_CATALOG.iter().map(|g| (g.id, g.base_price)).collect())
    }

    #[test]
    fn price_table_lookup() {
        let t = table();
        assert_eq!(t.lookup(1), Some(20));
        assert_eq!(t.lookup(2), Some(76));
        assert_eq!(t.lookup(3), Some(95));
        assert_eq!(t.lookup(4), Some(114));
        assert_eq!(t.lookup(5), Some(133));
        assert_eq!(t.lookup(6), None);
        assert_eq!(t.lookup(0), None);
    }

    #[test]
    fn price_table_placeholder_matches_balance() {
        // placeholder（冻结差分基准）与 balance 表当前值一致
        let t = PriceTable::placeholder();
        let Ok(b) = crate::data::balance() else {
            return; // balance 加载失败：差分基准无锚，跳过
        };
        for g in &b.trade.goods {
            assert_eq!(t.lookup(g.id), Some(g.buy), "条目须与 balance 表一致");
        }
        assert_eq!(t.lookup(99), None, "目录外商品 → None");
    }

    #[test]
    fn oracle_buy_success() {
        let t = table();
        // 金够买 1 件商品 1（价格 20）
        let r = oracle_buy(&t, 1, 100);
        assert!(r.is_ok(), "金 100 够买商品 1（价格 20）");
        let r = r.unwrap_or(TradeResult { gid: 0, paid: 0, gold_left: 0, profit: 0 });
        assert_eq!(r.gid, 1);
        assert_eq!(r.paid, 20);
        assert_eq!(r.gold_left, 80);
        assert_eq!(r.profit, 0, "买入不产生利润");
    }

    #[test]
    fn oracle_buy_insufficient_gold() {
        let t = table();
        // 金 10 < 商品 2 价格 76
        assert_eq!(oracle_buy(&t, 2, 10), Err(TradeErr::InsufficientGold));
        // 金 0
        assert_eq!(oracle_buy(&t, 1, 0), Err(TradeErr::InsufficientGold));
    }

    #[test]
    fn oracle_buy_unknown_good() {
        let t = table();
        assert_eq!(oracle_buy(&t, 99, 1000), Err(TradeErr::UnknownGood));
    }

    #[test]
    fn oracle_sell_success_with_avg() {
        let t = table();
        // 商品 1 价格 20，均价 15，持有 3 件，金 0
        let r = oracle_sell(&t, 1, 3, 0, 15);
        assert!(r.is_ok(), "卖出 3 件商品 1 应成功");
        let r = r.unwrap_or(TradeResult { gid: 0, paid: 0, gold_left: 0, profit: 0 });
        assert_eq!(r.gid, 1);
        assert_eq!(r.paid, 60, "3 × 20 = 60");
        assert_eq!(r.gold_left, 60, "0 + 60 = 60");
        assert_eq!(r.profit, 15, "3 × (20 - 15) = 15");
    }

    #[test]
    fn oracle_sell_profit_zero_when_avg_exceeds_price() {
        let t = table();
        // 均价 30 > 价格 20 → unit_profit = 0（saturating_sub），profit = 0
        let r = oracle_sell(&t, 1, 2, 0, 30);
        assert!(r.is_ok(), "卖出 2 件商品 1 应成功");
        let r = r.unwrap_or(TradeResult { gid: 0, paid: 0, gold_left: 0, profit: 0 });
        assert_eq!(r.paid, 40, "2 × 20 = 40");
        assert_eq!(r.gold_left, 40);
        assert_eq!(r.profit, 0, "均价 30 > 价格 20，利润钳 0");
    }

    #[test]
    fn oracle_sell_no_avg_means_full_price_profit() {
        let t = table();
        // avg=0 → profit = price × owned（无历史买入）
        let r = oracle_sell(&t, 2, 2, 0, 0);
        assert!(r.is_ok(), "卖出 2 件商品 2（无均价）应成功");
        let r = r.unwrap_or(TradeResult { gid: 0, paid: 0, gold_left: 0, profit: 0 });
        assert_eq!(r.paid, 152, "2 × 76");
        assert_eq!(r.gold_left, 152);
        assert_eq!(r.profit, 152, "无均价 → profit = 全价");
    }

    #[test]
    fn oracle_sell_no_stock() {
        let t = table();
        assert_eq!(oracle_sell(&t, 1, 0, 100, 10), Err(TradeErr::NoStock), "持有 0 → NoStock");
    }

    #[test]
    fn oracle_sell_unknown_good() {
        let t = table();
        assert_eq!(
            oracle_sell(&t, 99, 5, 100, 10),
            Err(TradeErr::UnknownGood),
            "未知商品 → UnknownGood"
        );
    }

    #[test]
    fn trade_event_codes() {
        assert_eq!(oracle_trade_event(22), Some(TradeEvent::TradeBuy));
        assert_eq!(oracle_trade_event(23), Some(TradeEvent::TradeSell));
        assert_eq!(oracle_trade_event(13), Some(TradeEvent::Confirm));
        assert_eq!(oracle_trade_event(37), Some(TradeEvent::Cancel));
        assert_eq!(oracle_trade_event(38), Some(TradeEvent::PageUp));
        assert_eq!(oracle_trade_event(39), Some(TradeEvent::PageConfirm));
        assert_eq!(oracle_trade_event(40), Some(TradeEvent::PageDown));
        // 未知码
        assert_eq!(oracle_trade_event(0), None);
        assert_eq!(oracle_trade_event(1), None);
        assert_eq!(oracle_trade_event(14), None);
        assert_eq!(oracle_trade_event(21), None);
        assert_eq!(oracle_trade_event(100), None);
    }

    #[test]
    fn goods_catalog_all_5() {
        for g in GOODS_CATALOG {
            assert!(g.id >= 1 && g.id <= 5, "商品 id 应在 1..=5");
            assert!(!g.name_key.is_empty(), "商品名 key 非空");
            assert!(g.base_price > 0, "基础价格 > 0");
        }
    }
}
