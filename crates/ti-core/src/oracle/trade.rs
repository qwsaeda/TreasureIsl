//! oracle：S3.6 贸易系统（固定价格表买卖结算）差分基准，测试专用。
//!
//! 直译 `docs/spec/trade.md` 的买卖/价格/利润规则：
//! - 商品 [`Goods`]（`id` + `name` + `base_price`，占位目录，spec §a）；
//! - 价格表 [`PriceTable`]（`entries: Vec<(goods_id, fixed_price)>`，spec §b）；
//! - [`oracle_price`]：固定价格查表（无行情波动，spec §b 推测：`rand()%22+1` = 商品选择而非价格波动）；
//! - [`oracle_buy`]：买入结算（`paid = price`，`profit = 0`，`gold_left = gold - paid`，spec §c）；
//! - [`oracle_sell`]：卖出结算（`gain = price`，`profit = price - avg_buy_price`，`gold_left = gold + gain`，spec §c）；
//! - 事件码 [`oracle_trade_event`]：trade 相关事件（spec §d，`场景管理器帧更新总控` case 22/23 + `贸易场景帧处理` a3 码）。
//!
//! 行为确定（同输入逐位相同），不允许 rand。
//! 随机分支（`主地图帧更新` case 4 的 `rand()%22+1` 商品选择）不在此 oracle 建模，
//! diff 测试以固定输入做 5 商品 × buy/sell × 各金量/持有量组合 × 错误路径对拍
//! （随机序列对齐归 S3.12）。

/// 商品（占位目录，spec §a；具体名称对照 S1.6 goods.lpk 解析结果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Goods {
    /// 商品 id（`u32`，对应 `资源查表` 的 key 值域）。
    pub id: u32,
    /// 商品名称（占位，**推测**对应 goods.lpk 记录）。
    pub name: &'static str,
    /// 基础价格（`贸易场景对象` 初值 / `资源查表` 查表值的占位，spec §b）。
    pub base_price: u32,
}

/// 价格表（固定价格，spec §b；`单件买入` 行 49-67/146-164 查表语义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceTable {
    /// 价格条目（`goods_id → fixed_price`，5 商品占位表，spec §a 5 槽）。
    pub entries: Vec<(u32, u32)>,
}

impl PriceTable {
    /// 新建固定价格表（5 商品占位：id 1..5，价格 [20, 76, 95, 114, 133]，来自 `贸易场景对象` 初值）。
    #[must_use]
    pub fn placeholder() -> Self {
        Self { entries: vec![(1, 20), (2, 76), (3, 95), (4, 114), (5, 133)] }
    }

    /// 查固定价格（`oracle_price` 的无 seed 版本，spec §b 固定价格表）。
    ///
    /// 查不到 → `None`（对应 `资源查表` 遍历无匹配返回 0 的语义）。
    #[must_use]
    pub fn lookup(&self, gid: u32) -> Option<u32> {
        self.entries.iter().find(|&&(id, _)| id == gid).map(|&(_, p)| p)
    }

    /// oracle：确定性价格查询（`base_price` 或 `base_price ± seed%10`，spec §b 行情波动占位）。
    ///
    /// 固定价格表（`seed == 0`）直接返回 `base_price`；
    /// `seed != 0` 时 `price = base_price - (seed % 10)`（**推测**：价格只降不涨的波动公式，
    /// oracle 层用确定性公式替代 exe 的 `rand()%22+1` 哈希，S3.12 RNG 对齐后再校准）。
    /// 查不到 → `None`。
    #[must_use]
    pub fn oracle_price(&self, gid: u32, seed: u32) -> Option<u32> {
        let base = self.lookup(gid)?;
        Some(if seed == 0 { base } else { base.saturating_sub(seed % 10) })
    }
}

/// 贸易错误（`贸易操作验证` 返回码映射，spec §c）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeErr {
    /// 金不足（`贸易操作验证` case 0/2 行 31-38：`资源集字段读取器(+136,0)==0` → 结果码 2）。
    InsufficientGold,
    /// 未知商品（`gid` 不在价格表中，`资源查表` 遍历无匹配）。
    UnknownGood,
    /// 无库存（`贸易操作验证` case 1/5 行 41-49/73-89：持有列表无匹配 → 结果码 1/15）。
    NoStock,
}

/// 贸易结算结果（`单件买入` 行 99/118 金累加 + 行 128-135 均价计算，spec §c）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TradeResult {
    /// 商品 id。
    pub gid: u32,
    /// 支付/获得金额（买入 = `price × qty`，卖出 = `price × qty`）。
    pub paid: u32,
    /// 结算后剩余金（买入 = `gold - paid`，卖出 = `gold + paid`）。
    pub gold_left: u32,
    /// 利润（买入 = 0，卖出 = `price - avg_buy_price`，spec §c；负值钳 0）。
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

/// 贸易事件码（spec §d，`场景管理器帧更新总控` case 22/23 + `贸易场景帧处理` a3 码）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeEvent {
    /// 进入贸易场景（买入方向，`场景管理器帧更新总控` case 22，`a1+716=22`）。
    TradeBuy,
    /// 进入贸易场景（卖出方向，`场景管理器帧更新总控` case 23，`a1+716=23`）。
    TradeSell,
    /// 买入/卖出确认（`贸易场景帧处理` a3=13，`+1020==6` 买入 / `+1020==7` 卖出）。
    Confirm,
    /// 取消（`贸易场景帧处理` a3=37，撤销当前操作）。
    Cancel,
    /// 翻上页（`贸易场景帧处理` a3=38，商品索引递减）。
    PageUp,
    /// 翻页确认（`贸易场景帧处理` a3=39，`+1020` 6→7）。
    PageConfirm,
    /// 翻页（`贸易场景帧处理` a3=40，商品索引递增）。
    PageDown,
}

/// oracle：贸易事件码判定（spec §d，给定事件码返回对应的 [`TradeEvent`]）。
///
/// 未知事件码 → `None`（与 `oracle_dispatch_event` 的 `None` 语义一致）。
#[must_use]
pub fn oracle_trade_event(code: u32) -> Option<TradeEvent> {
    match code {
        22 => Some(TradeEvent::TradeBuy),
        23 => Some(TradeEvent::TradeSell),
        13 => Some(TradeEvent::Confirm),
        37 => Some(TradeEvent::Cancel),
        38 => Some(TradeEvent::PageUp),
        39 => Some(TradeEvent::PageConfirm),
        40 => Some(TradeEvent::PageDown),
        _ => None,
    }
}

/// 占位商品目录（5 商品，spec §a；名称 **推测**，价格来自 `贸易场景对象` 初值）。
pub const GOODS_CATALOG: [Goods; 5] = [
    Goods { id: 1, name: "good_01", base_price: 20 },
    Goods { id: 2, name: "good_02", base_price: 76 },
    Goods { id: 3, name: "good_03", base_price: 95 },
    Goods { id: 4, name: "good_04", base_price: 114 },
    Goods { id: 5, name: "good_05", base_price: 133 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_table_lookup() {
        let t = PriceTable::placeholder();
        assert_eq!(t.lookup(1), Some(20));
        assert_eq!(t.lookup(2), Some(76));
        assert_eq!(t.lookup(3), Some(95));
        assert_eq!(t.lookup(4), Some(114));
        assert_eq!(t.lookup(5), Some(133));
        assert_eq!(t.lookup(6), None);
        assert_eq!(t.lookup(0), None);
    }

    #[test]
    fn oracle_price_fixed_and_seeded() {
        let t = PriceTable::placeholder();
        // seed=0 → 固定价格
        assert_eq!(t.oracle_price(1, 0), Some(20));
        assert_eq!(t.oracle_price(5, 0), Some(133));
        // seed=3 → price = base - 3
        assert_eq!(t.oracle_price(1, 3), Some(17));
        assert_eq!(t.oracle_price(5, 3), Some(130));
        // seed=15 → seed%10=5，price = base - 5
        assert_eq!(t.oracle_price(2, 15), Some(71));
        // 未知商品 → None
        assert_eq!(t.oracle_price(99, 0), None);
        // seed=100 → seed%10=0，等价固定价格
        assert_eq!(t.oracle_price(1, 100), Some(20));
    }

    #[test]
    fn oracle_buy_success() {
        let t = PriceTable::placeholder();
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
        let t = PriceTable::placeholder();
        // 金 10 < 商品 2 价格 76
        assert_eq!(oracle_buy(&t, 2, 10), Err(TradeErr::InsufficientGold));
        // 金 0
        assert_eq!(oracle_buy(&t, 1, 0), Err(TradeErr::InsufficientGold));
    }

    #[test]
    fn oracle_buy_unknown_good() {
        let t = PriceTable::placeholder();
        assert_eq!(oracle_buy(&t, 99, 1000), Err(TradeErr::UnknownGood));
    }

    #[test]
    fn oracle_sell_success_with_avg() {
        let t = PriceTable::placeholder();
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
        let t = PriceTable::placeholder();
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
        let t = PriceTable::placeholder();
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
        let t = PriceTable::placeholder();
        assert_eq!(oracle_sell(&t, 1, 0, 100, 10), Err(TradeErr::NoStock), "持有 0 → NoStock");
    }

    #[test]
    fn oracle_sell_unknown_good() {
        let t = PriceTable::placeholder();
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
            assert!(!g.name.is_empty(), "商品名非空");
            assert!(g.base_price > 0, "基础价格 > 0");
        }
    }
}
