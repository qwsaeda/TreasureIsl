//! S3.6 贸易系统正式实现。
//!
//! 与 `oracle::trade` 行为逐位一致（同一输入产生逐位相同的
//! 买入/卖出结算/价格查询/利润计算），但结构不同：用 `TradeLedger`
//! 结构体 + 方法承载状态（持有量/金/价格表/买入成本），而非自由函数。
//!
//! 依据 `docs/spec/trade.md`：
//! - 商品目录 [`Goods`]、价格表 [`PriceTable`]、错误 [`TradeErr`]、结果 [`TradeResult`]
//!   （从 oracle 重导出）；
//! - [`TradeLedger`]：贸易账本（`goods: Vec<(gid, qty)>` + `gold` + `table` + `buy_cost`）；
//! - `buy` / `sell` / `gold()` / `profit_since(&self, gid)` 方法。

use crate::oracle::trade::{
    GOODS_CATALOG, Goods, PriceTable, TradeErr, TradeResult, oracle_buy, oracle_sell,
};

/// 贸易账本：持有量 + 金 + 价格表 + 买入成本。
#[derive(Debug, Clone)]
pub struct TradeLedger {
    /// 持有量（`goods_id → qty`，`sub_4DCD8` 8 字节记录的 oracle 化）。
    goods: Vec<(u32, u32)>,
    /// 当前金（`sub_54D1C` `+1024` 偏移的 oracle 化）。
    gold: u32,
    /// 价格表（`sub_546E0`/`sub_54D1C` 固定 5 商品占位表）。
    table: PriceTable,
    /// 历史买入成本（`goods_id → 累计成本`，用于 `avg_buy_price` 计算均价，
    /// 对应 `sub_54D1C` 行 128-135 均价公式的 oracle 简化）。
    buy_cost: Vec<(u32, u32)>,
}

impl TradeLedger {
    /// 新建贸易账本（指定初始金、价格表、初始持有量）。
    #[must_use]
    pub fn new(gold: u32, table: PriceTable, goods: &[(u32, u32)]) -> Self {
        Self { goods: goods.to_vec(), gold, table, buy_cost: Vec::new() }
    }

    /// 当前金（只读，对应 `sub_54D1C` `+1024` 偏移）。
    #[must_use]
    pub fn gold(&self) -> u32 {
        self.gold
    }

    /// 持有量（只读，对应 `sub_4DCD8` 链表记录）。
    #[must_use]
    pub fn goods(&self) -> &[(u32, u32)] {
        &self.goods
    }

    /// 某商品当前持有量（无记录 → 0）。
    #[must_use]
    pub fn owned(&self, gid: u32) -> u32 {
        self.goods.iter().find(|(id, _)| *id == gid).map_or(0, |&(_, q)| q)
    }

    /// 买入均价（`sub_54D1C` 行 128-135 公式的 oracle 简化：`累计成本 / 累计数量`）。
    ///
    /// 无买入记录或持有量为 0 → 0（对应 `oracle_sell` 的 `avg_buy_price == 0` → profit = 全价）。
    #[must_use]
    pub fn avg_buy_price(&self, gid: u32) -> u32 {
        let qty = self.owned(gid);
        if qty == 0 {
            return 0;
        }
        self.buy_cost.iter().find(|(id, _)| *id == gid).map_or(0, |&(_, cost)| cost / qty)
    }

    /// 买入 1 件（行为与 `oracle_buy` 逐位一致，同时更新内部状态）。
    ///
    /// 成功：`gold -= price`，`goods[gid].qty += 1`，`buy_cost[gid].cost += price`。
    /// 失败：状态不变。
    ///
    /// # Errors
    ///
    /// - [`TradeErr::UnknownGood`]：`gid` 不在价格表中；
    /// - [`TradeErr::InsufficientGold`]：`gold < price`。
    pub fn buy(&mut self, gid: u32) -> Result<TradeResult, TradeErr> {
        let result = oracle_buy(&self.table, gid, self.gold)?;
        self.gold = result.gold_left;
        self.apply_buy(gid, result.paid);
        Ok(result)
    }

    /// 卖出当前全部持有件（行为与 `oracle_sell` 逐位一致，同时更新内部状态）。
    ///
    /// 成功：`gold += gain`，`goods[gid].qty -= owned`，`buy_cost[gid].cost -= price × owned`。
    /// 失败：状态不变。
    ///
    /// # Errors
    ///
    /// - [`TradeErr::UnknownGood`]：`gid` 不在价格表中；
    /// - [`TradeErr::NoStock`]：`owned == 0`。
    pub fn sell(&mut self, gid: u32) -> Result<TradeResult, TradeErr> {
        let owned = self.owned(gid);
        let avg = self.avg_buy_price(gid);
        let result = oracle_sell(&self.table, gid, owned, self.gold, avg)?;
        self.gold = result.gold_left;
        self.apply_sell(gid, owned);
        Ok(result)
    }

    /// 某商品的累计利润（`profit_since`，对应 `sub_54D1C` 行 128-135 均价公式）。
    ///
    /// 规则：`profit = (当前价 - 买入均价) × 持有量`（负值钳 0，与 `oracle_sell` 一致）。
    /// 无持有 → 0。
    #[must_use]
    pub fn profit_since(&self, gid: u32) -> i32 {
        let owned = self.owned(gid);
        if owned == 0 {
            return 0;
        }
        let Some(price) = self.table.lookup(gid) else {
            return 0;
        };
        let avg = self.avg_buy_price(gid);
        let unit_profit = if avg > 0 { price.saturating_sub(avg) } else { price };
        i32::try_from(unit_profit.saturating_mul(owned)).unwrap_or(i32::MAX)
    }

    /// 占位商品目录（`GOODS_CATALOG` 重导出，spec §a）。
    #[must_use]
    pub fn catalog() -> &'static [Goods; 5] {
        &GOODS_CATALOG
    }

    /// 查固定价格（行为与 `oracle_price` 逐位一致，seed=0 即固定价格）。
    #[must_use]
    pub fn price(&self, gid: u32) -> Option<u32> {
        self.table.lookup(gid)
    }

    /// 预置买入成本（用于 diff 测试：构造任意 `avg_buy_price` 而不走 buy 链）。
    pub fn set_buy_cost(&mut self, gid: u32, cost: u32) {
        match self.buy_cost.iter_mut().find(|(id, _)| *id == gid) {
            Some(entry) => entry.1 = cost,
            None => self.buy_cost.push((gid, cost)),
        }
    }

    // ── 内部辅助 ─────────────────────────────────────────────────────────────

    /// 买入成功后的内部状态更新（`goods` 持有量 +1，`buy_cost` 累计 + price）。
    fn apply_buy(&mut self, gid: u32, price: u32) {
        match self.goods.iter_mut().find(|(id, _)| *id == gid) {
            Some(entry) => entry.1 += 1,
            None => self.goods.push((gid, 1)),
        }
        match self.buy_cost.iter_mut().find(|(id, _)| *id == gid) {
            Some(entry) => entry.1 = entry.1.saturating_add(price),
            None => self.buy_cost.push((gid, price)),
        }
    }

    /// 卖出成功后的内部状态更新（`goods` 持有量 - `sold_qty`，`buy_cost` 按比例扣减）。
    fn apply_sell(&mut self, gid: u32, sold_qty: u32) {
        if sold_qty == 0 {
            return;
        }
        if let Some(entry) = self.goods.iter_mut().find(|(id, _)| *id == gid) {
            entry.1 = entry.1.saturating_sub(sold_qty);
        }
        // 扣减 buy_cost（精确扣减：price × sold_qty，因 oracle 固定价格表下均价 = cost/qty）
        let price = self.table.lookup(gid).unwrap_or(0);
        let reduce = price.saturating_mul(sold_qty);
        if let Some(entry) = self.buy_cost.iter_mut().find(|(id, _)| *id == gid) {
            entry.1 = entry.1.saturating_sub(reduce);
        }
    }
}

impl Default for TradeLedger {
    fn default() -> Self {
        Self::new(0, PriceTable::placeholder(), &[])
    }
}
