//! S8.3.b 贸易系统正式实现（数值从 `assets/data/balance.json` 注入）。
//!
//! 业务规格（唯一事实源）：[`docs/business/economy.md`](../../../docs/business/economy.md)。
//! 设计要点：
//! - **不重抄数值**：价格/售价系数/数量上限全部来自 [`TradeCfg`]（由 `data::balance()` 注入）；
//! - **不做动态行情**：售价固定（`sell = buy × markup`），避免投机博弈；
//! - 纯逻辑、零 IO、确定性（同输入同输出）。
//!
//! 本模块数值全部读 balance.json，不再取逆向「推测」价格表。

use crate::data::TradeCfg;

/// 贸易错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeErr {
    /// 金不足（`gold < cost`）。
    InsufficientGold,
    /// 未知商品（`gid` 不在目录中）。
    UnknownGood,
    /// 无库存（持有量为 0）。
    NoStock,
    /// 数量越界（`qty > 有效上限 min(qoods.qty_max, cargo_cap)` 或 `qty == 0`）。
    QtyOutOfRange,
}

/// 一次买卖的结算结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeResult {
    /// 商品 id。
    pub gid: u32,
    /// 成交数量。
    pub qty: u32,
    /// 支付/获得金额。
    pub amount: u32,
    /// 结算后金币。
    pub gold_left: u32,
    /// 超出 `gold_cap` 的溢出量（进银行；`gold_cap == 0` 时恒 0）。
    pub overflow: u32,
}

/// 把非负有限 `f64` 钳到 `u32` 区间后取整（避免直接 cast 触发 clippy pedantic）。
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::trivially_copy_pass_by_ref
)]
fn f64_to_u32(v: f64) -> Option<u32> {
    if !v.is_finite() || v < 0.0 {
        return None;
    }
    Some(v.round().min(f64::from(u32::MAX)) as u32)
}

/// S10.3 改动 3：卖出单价纯函数（`round(buy × markup)`，不依赖 `TradeCfg` 结构体）。
///
/// 供 `SceneHost` 等无账本上下文的调用点使用；`markup` 走 `balance.trade.markup`。
#[must_use]
pub fn sell_price(buy: u32, markup: f32) -> u32 {
    let v = f64::from(buy) * f64::from(markup);
    f64_to_u32(v).unwrap_or(buy)
}

/// 贸易账本：持有量 + 金 + 配置 + 累计买入成本。
///
/// `gold_cap` 来自 `balance.economy.gold_max`（`0` = 无上限，卖出加金时不做钳制）。
/// 卖出加金钳到 `gold_cap` 后溢出部分由调用方经 [`BuildManager::settle_overflow`](crate::rules::build::BuildManager::settle_overflow) 处理。
///
/// **单次贸易数量上限** = `min(goods.qty_max, cargo_cap)`（`docs/business/economy.md`
/// §船系统规则 2）：`cargo_cap` 是船舱容量，由 [`crate::rules::boat::BoatSession::cargo`]
/// 注入（默认 = 1 级小舢板货舱 = 现状有效上限 ⇒ 1 级行为与旧版一致）。
#[derive(Debug, Clone)]
pub struct TradeLedger {
    /// 持有量（`goods_id → qty`）。
    goods: Vec<(u32, u32)>,
    /// 当前金。
    gold: u32,
    /// 累计买入成本（`goods_id → 累计成本`，用于均价与利润）。
    buy_cost: Vec<(u32, u32)>,
    /// 数值配置（价格表/售价系数/数量上限），由 balance 注入。
    cfg: TradeCfg,
    /// 金量上限（`balance.economy.gold_max`；`0` = 不限制）。
    gold_cap: u32,
    /// 船舱上限（单次贸易数量上限的船侧约束；`0` = 不限）。
    cargo_cap: u32,
}

impl TradeLedger {
    /// 新建贸易账本（注入配置 + 初始金 + 初始持有 + 金量上限）。
    ///
    /// `gold_cap` 来自 `balance.economy.gold_max`；`0` 表示不限制（卖出加金不钳制）。
    /// 船舱上限取 `cfg.default_cargo_cap`（1 级小舢板货舱 = 现状有效上限）。
    #[must_use]
    pub fn new(cfg: TradeCfg, gold: u32, goods: &[(u32, u32)], gold_cap: u32) -> Self {
        Self {
            goods: goods.to_vec(),
            gold,
            buy_cost: Vec::new(),
            cargo_cap: cfg.default_cargo_cap,
            cfg,
            gold_cap,
        }
    }

    /// 新建贸易账本（金量上限 = 0，不限制）。
    #[must_use]
    pub fn new_unlimited(cfg: TradeCfg, gold: u32, goods: &[(u32, u32)]) -> Self {
        Self::new(cfg, gold, goods, 0)
    }

    /// 当前金（只读）。
    #[must_use]
    pub fn gold(&self) -> u32 {
        self.gold
    }

    /// 持有量（只读）。
    #[must_use]
    pub fn goods(&self) -> &[(u32, u32)] {
        &self.goods
    }

    /// 某商品当前持有量（无记录 → 0）。
    #[must_use]
    pub fn owned(&self, gid: u32) -> u32 {
        self.goods.iter().find(|(id, _)| *id == gid).map_or(0, |&(_, q)| q)
    }

    /// 商品配置（只读；不存在 → `None`）。
    #[must_use]
    pub fn good_def(&self, gid: u32) -> Option<&crate::data::GoodsDef> {
        self.cfg.goods.iter().find(|g| g.id == gid)
    }

    /// 买入单价（`buy`；不存在 → `None`）。
    #[must_use]
    pub fn buy_price(&self, gid: u32) -> Option<u32> {
        self.good_def(gid).map(|g| g.buy)
    }

    /// 卖出单价（`round(buy × markup)`；不存在 → `None`）。
    #[must_use]
    pub fn sell_price(&self, gid: u32) -> Option<u32> {
        let g = self.good_def(gid)?;
        let v = f64::from(g.buy) * f64::from(self.cfg.markup);
        Some(f64_to_u32(v).unwrap_or(g.buy))
    }

    /// 商品目录原始持有上限（`balance.trade.goods[].qty_max`；不存在 → 0）。
    #[must_use]
    pub fn goods_qty_max(&self, gid: u32) -> u32 {
        self.good_def(gid).map_or(0, |g| g.qty_max)
    }

    /// 单次贸易数量上限（有效上限）= `min(goods.qty_max, cargo_cap)`。
    ///
    /// `cargo_cap == 0`（不限）时 = `goods.qty_max`（`docs/business/economy.md`
    /// §船系统规则 2 的 `effective = min(goods.qty_max, cargo)`）；不存在 → 0。
    #[must_use]
    pub fn qty_cap(&self, gid: u32) -> u32 {
        let goods_max = self.goods_qty_max(gid);
        if self.cargo_cap == 0 { goods_max } else { goods_max.min(self.cargo_cap) }
    }

    /// 当前船舱上限（`0` = 不限，仅受商品目录 `qty_max` 约束）。
    #[must_use]
    pub fn cargo_cap(&self) -> u32 {
        self.cargo_cap
    }

    /// 设置船舱上限（造船后由 [`crate::rules::boat::BoatSession`] 注入；`0` = 不限）。
    pub fn set_cargo_cap(&mut self, cap: u32) {
        self.cargo_cap = cap;
    }

    /// 扣金（造船等一次性支出用）。
    ///
    /// # Errors
    ///
    /// [`TradeErr::InsufficientGold`]：`gold < amount`（失败状态不变）。
    pub fn debit(&mut self, amount: u32) -> Result<(), TradeErr> {
        if self.gold < amount {
            return Err(TradeErr::InsufficientGold);
        }
        self.gold -= amount;
        Ok(())
    }

    /// 买入均价（累计成本 / 持有量；无记录 → 0）。
    #[must_use]
    pub fn avg_buy_price(&self, gid: u32) -> u32 {
        let qty = self.owned(gid);
        if qty == 0 {
            return 0;
        }
        self.buy_cost.iter().find(|(id, _)| *id == gid).map_or(0, |&(_, cost)| cost / qty)
    }

    /// 买入 `qty` 件。
    ///
    /// 成功：`gold -= buy_price × qty`，`goods[gid].qty += qty`，
    /// `buy_cost[gid] += buy_price × qty`；失败状态不变。
    ///
    /// # Errors
    ///
    /// - [`TradeErr::UnknownGood`]：`gid` 不在目录；
    /// - [`TradeErr::QtyOutOfRange`]：`qty == 0` 或 `owned + qty > 有效上限`
    ///   （`min(goods.qty_max, cargo_cap)`）；
    /// - [`TradeErr::InsufficientGold`]：`gold < cost`。
    pub fn buy(&mut self, gid: u32, qty: u32) -> Result<TradeResult, TradeErr> {
        let price = self.buy_price(gid).ok_or(TradeErr::UnknownGood)?;
        let cap = self.qty_cap(gid);
        if qty == 0 || self.owned(gid).saturating_add(qty) > cap {
            return Err(TradeErr::QtyOutOfRange);
        }
        let cost = price.saturating_mul(qty);
        if self.gold < cost {
            return Err(TradeErr::InsufficientGold);
        }
        self.gold -= cost;
        self.apply_buy(gid, qty, cost);
        Ok(TradeResult { gid, qty, amount: cost, gold_left: self.gold, overflow: 0 })
    }

    /// 卖出 `qty` 件（`qty == 0` 视为卖出全部持有，符合 UI 直觉）。
    ///
    /// 成交数量 = `min(qty 或 持有量, 持有量, 有效上限 min(qoods.qty_max, cargo_cap))`
    /// （单次贸易数量上限对买卖两侧一致生效）。
    ///
    /// 成功：`gold += sell_price × qty`，`goods[gid].qty -= qty`，
    /// 按比例扣减 `buy_cost`。若设置了 `gold_cap > 0`，结算后
    /// 金币钳制到 `gold_cap`（溢出走银行，由调用方经
    /// [`crate::rules::build::BuildManager::settle_overflow`] 处理）。
    ///
    /// # Errors
    ///
    /// - [`TradeErr::UnknownGood`]：`gid` 不在目录；
    /// - [`TradeErr::NoStock`]：持有量为 0。
    pub fn sell(&mut self, gid: u32, qty: u32) -> Result<TradeResult, TradeErr> {
        let owned = self.owned(gid);
        if owned == 0 {
            return Err(TradeErr::NoStock);
        }
        let price = self.sell_price(gid).ok_or(TradeErr::UnknownGood)?;
        let cap = self.qty_cap(gid);
        let qty = if qty == 0 { owned } else { qty.min(owned) }.min(cap);
        let gain = price.saturating_mul(qty);
        let raw = self.gold.saturating_add(gain);
        let (new_gold, overflow) = if self.gold_cap > 0 {
            let clamped = raw.min(self.gold_cap);
            (clamped, raw.saturating_sub(clamped))
        } else {
            (raw, 0)
        };
        self.gold = new_gold;
        self.apply_sell(gid, qty, price);
        Ok(TradeResult { gid, qty, amount: gain, gold_left: self.gold, overflow })
    }

    /// 当前金量上限（`0` = 不限制）。
    #[must_use]
    pub fn gold_cap(&self) -> u32 {
        self.gold_cap
    }

    /// 设置金量上限（建造完成后由调用方调用）。
    ///
    /// `0` = 解除限制。
    pub fn set_gold_cap(&mut self, cap: u32) {
        self.gold_cap = cap;
    }

    /// 某商品的累计利润（`(卖出价 - 买入均价) × 持有量`；无持有 → 0）。
    #[must_use]
    pub fn profit_since(&self, gid: u32) -> i32 {
        let owned = self.owned(gid);
        if owned == 0 {
            return 0;
        }
        let Some(price) = self.sell_price(gid) else {
            return 0;
        };
        let avg = self.avg_buy_price(gid);
        let unit = price.saturating_sub(avg);
        i32::try_from(unit.saturating_mul(owned)).unwrap_or(i32::MAX)
    }

    /// 预置买入成本（测试夹具：构造任意均价而不必先 buy）。
    pub fn set_buy_cost(&mut self, gid: u32, cost: u32) {
        match self.buy_cost.iter_mut().find(|(id, _)| *id == gid) {
            Some(entry) => entry.1 = cost,
            None => self.buy_cost.push((gid, cost)),
        }
    }

    /// 配置只读视图（表现层画价格表用）。
    #[must_use]
    pub fn cfg(&self) -> &TradeCfg {
        &self.cfg
    }

    // ── 内部辅助 ─────────────────────────────────────────────────────────────

    /// 买入后：持有量 +qty，累计成本 +cost。
    fn apply_buy(&mut self, gid: u32, qty: u32, cost: u32) {
        match self.goods.iter_mut().find(|(id, _)| *id == gid) {
            Some(entry) => entry.1 += qty,
            None => self.goods.push((gid, qty)),
        }
        match self.buy_cost.iter_mut().find(|(id, _)| *id == gid) {
            Some(entry) => entry.1 = entry.1.saturating_add(cost),
            None => self.buy_cost.push((gid, cost)),
        }
    }

    /// 卖出后：持有量 -qty，按买入价等比扣减累计成本。
    fn apply_sell(&mut self, gid: u32, qty: u32, buy_price: u32) {
        if qty == 0 {
            return;
        }
        if let Some(entry) = self.goods.iter_mut().find(|(id, _)| *id == gid) {
            entry.1 = entry.1.saturating_sub(qty);
        }
        let reduce = buy_price.saturating_mul(qty);
        if let Some(entry) = self.buy_cost.iter_mut().find(|(id, _)| *id == gid) {
            entry.1 = entry.1.saturating_sub(reduce);
        }
    }
}
