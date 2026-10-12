//! 经济域类型（业务文档：`docs/business/economy.md`）。
//!
//! 贸易（商品/价格/结算）+ 建造（建筑/银行）两块的类型定义。
//! 价格与造价数值**不在此硬编码**，走 `assets/data/balance.json`
//! （`balance.trade` / `balance.build`），由 `rules::trade` / `rules::build` 注入。
//!
//! 类型分工（P2 经济域收口，c1 §1 类型唯一家）：
//! - 纯数据域类型（`Goods`/`PriceTable`/`Building`/`Bank`/`TradeEvent`）在本模块，
//!   跨包经 `ti_core::domain` 重导出；
//! - 错误/结算结果类型是**行为契约**（`rules` 层结算语义），不在域模型重复定义：
//!   正式实现用 `rules::trade::{TradeErr, TradeResult}` /
//!   `rules::build::{BuildErr, Bank, BuildManager}`。
//!
//! 域最小可用类型（P2 正向补）：[`GOODS_CATALOG`] 冻结初值商品目录 +
//! [`PriceTable::seed_price`]（带 seed 折扣价格）+ 事件码分派基准
//! [`TRADE_EVENT_CODES`]。

// ── 贸易 ────────────────────────────────────────────────────────────────────

/// 商品（纯数据，数值权威在 `balance.trade.goods`）。
///
/// 业务语义（`docs/business/economy.md` §商品）：
/// - `id`：商品 id（存档与价格表的键）；
/// - `name_key`：玩家可见文本 key（走 `strings.json`，非中文面量）；
/// - `base_price`：基础买入价（调试占位，权威值在 `balance.trade.goods[i].buy`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Goods {
    /// 商品 id。
    pub id: u32,
    /// 玩家可见文本 key（`strings.json`，如 `goods.1`）。
    pub name_key: &'static str,
    /// 基础买入价（调试占位，权威值在 `balance.json`）。
    pub base_price: u32,
}

/// 价格表（`goods_id → 固定买入价`）。
///
/// 业务语义（`docs/business/economy.md` §价格）：**无动态行情**，固定价表；
/// 条目由 `balance.trade.goods` 构建（`rules` 层），卖出价 =
/// `买入价 × balance.trade.markup`（`rules::trade::sell_price`）。
/// 本结构只承载数据与只读查询；带 seed 的伪随机折扣作为差分基准保留为
/// 只读查询方法（[`PriceTable::seed_price`]，`[P2]` 正向化，原逆向公式）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceTable {
    /// 价格条目（`goods_id → 买入价`）。
    pub entries: Vec<(u32, u32)>,
}

impl PriceTable {
    /// 从条目直接构造（`rules` 层从 `balance.trade.goods` 构建）。
    #[must_use]
    pub fn new(entries: Vec<(u32, u32)>) -> Self {
        Self { entries }
    }

    /// 查固定买入价（查不到 → `None`，对应「未知商品」）。
    #[must_use]
    pub fn lookup(&self, gid: u32) -> Option<u32> {
        self.entries.iter().find(|&&(id, _)| id == gid).map(|&(_, p)| p)
    }

    /// 占位价格表（冻结初值 5 商品；权威值在 `balance.trade.goods`）。
    ///
    /// 既有 API（ti-app 组合根构造 + 差分基准用）；数值与 balance.json
    /// 当前值一致（`[P2]` 单测 `placeholder_matches_balance` 校验），
    /// 调平衡改 JSON 后须同步此常量（`check_hardcode` 门禁第 12 项）。
    #[must_use]
    pub fn placeholder() -> Self {
        Self { entries: vec![(1, 20), (2, 76), (3, 95), (4, 114), (5, 133)] }
    }

    /// 带 seed 的价格（`[P2]` 差分基准语义，原逆向公式正向化：seed=0 固定，seed>0 减 `seed % 10`）。
    ///
    /// 查不到商品 → `None`。仅供差分基准/测试使用；
    /// 正式结算走 `rules::trade`（无动态行情，`docs/business/economy.md`）。
    #[must_use]
    pub fn seed_price(&self, gid: u32, seed: u32) -> Option<u32> {
        let base = self.lookup(gid)?;
        Some(if seed == 0 { base } else { base.saturating_sub(seed % 10) })
    }
}

// ── 建造 ────────────────────────────────────────────────────────────────────

/// 建筑类型。
///
/// 业务语义（`docs/business/economy.md` §建造）：4 种建筑，**单调升级链**
/// （等级只增不减），造价 = `balance.build.buildings[i].base_cost × level`
/// （由 `rules::build::cost_of` 计算，不在此硬编码）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Building {
    /// 城堡（核心据点）。
    Castle,
    /// 投资设施（提升收益）。
    Investment,
    /// 银行（金上限提升 + 存取）。
    Bank,
    /// 堡垒（防御）。
    Fort,
}

impl Building {
    /// 全部建筑类型（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [Building; 4] {
        [Self::Castle, Self::Investment, Self::Bank, Self::Fort]
    }
}

/// 银行（金量容器，业务文档 `docs/business/economy.md` §银行）。
///
/// 纯数据：流动金 + 已存金（承接上限溢出）。存取/结算溢出行为归
/// `rules::build::Bank`（c1 契约中「`Bank`」的实例形态以 `rules::build` 为准；
/// 本域类型供存档序列化（`rules::saveload`）与跨包只读视图使用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bank {
    /// 流动金。
    pub gold: u32,
    /// 银行累计（承接溢出）。
    pub deposit: u32,
}

impl Bank {
    /// 新建银行（流动金 `gold`，存款 0）。
    #[must_use]
    pub const fn new(gold: u32) -> Self {
        Self { gold, deposit: 0 }
    }

    /// 总资产（流动 + 存款）。
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn total(&self) -> u32 {
        self.gold.saturating_add(self.deposit)
    }
}

/// 贸易事件（业务文档 `docs/business/economy.md` §交互；
/// 事件码分派基准见 [`TRADE_EVENT_CODES`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeEvent {
    /// 进入贸易（买入方向）。
    TradeBuy,
    /// 进入贸易（卖出方向）。
    TradeSell,
    /// 买入/卖出确认。
    Confirm,
    /// 取消当前操作。
    Cancel,
    /// 翻上页（商品索引递减）。
    PageUp,
    /// 翻页确认（切换买/卖方向）。
    PageConfirm,
    /// 翻下页（商品索引递增）。
    PageDown,
}

// ── 差分基准冻结值（[P2] 正向化，原逆向初值）────────────────────────────────

/// 冻结初值商品目录（5 商品；权威值在 `balance.trade.goods`）。
pub const GOODS_CATALOG: [Goods; 5] = [
    Goods { id: 1, name_key: "good_01", base_price: 20 },
    Goods { id: 2, name_key: "good_02", base_price: 76 },
    Goods { id: 3, name_key: "good_03", base_price: 95 },
    Goods { id: 4, name_key: "good_04", base_price: 114 },
    Goods { id: 5, name_key: "good_05", base_price: 133 },
];

/// 贸易事件码分派基准（`code → TradeEvent`；未知码 → `None`）。
/// `[P2]` 正向化：原逆向事件码表冻结为差分基准。
pub const TRADE_EVENT_CODES: [(u32, TradeEvent); 7] = [
    (22, TradeEvent::TradeBuy),
    (23, TradeEvent::TradeSell),
    (13, TradeEvent::Confirm),
    (37, TradeEvent::Cancel),
    (38, TradeEvent::PageUp),
    (39, TradeEvent::PageConfirm),
    (40, TradeEvent::PageDown),
];

/// 事件码 → 事件（分派基准；未知码 → `None`）。
#[must_use]
pub fn trade_event_from_code(code: u32) -> Option<TradeEvent> {
    TRADE_EVENT_CODES.iter().find(|&&(c, _)| c == code).map(|&(_, e)| e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goods_catalog_invariants() {
        for g in GOODS_CATALOG {
            assert!(g.id >= 1 && g.id <= 5, "商品 id 应在 1..=5");
            assert!(!g.name_key.is_empty(), "商品名 key 非空");
            assert!(g.base_price > 0, "基础价格 > 0");
        }
    }

    #[test]
    fn placeholder_matches_balance() {
        let Ok(b) = crate::data::balance() else {
            return;
        };
        let t = PriceTable::placeholder();
        for g in &b.trade.goods {
            assert_eq!(t.lookup(g.id), Some(g.buy), "条目须与 balance 表一致");
        }
        assert_eq!(t.lookup(99), None, "目录外商品 → None");
    }

    #[test]
    fn seed_price_zero_seed_is_fixed() {
        let t = PriceTable::placeholder();
        assert_eq!(t.seed_price(1, 0), Some(20), "seed=0 固定价");
        assert_eq!(t.seed_price(99, 0), None, "未知商品 → None");
    }

    #[test]
    fn seed_price_positive_seed_discounts() {
        let t = PriceTable::placeholder();
        assert_eq!(t.seed_price(2, 3), Some(73), "76 - 3%10=3 → 73");
        assert_eq!(t.seed_price(1, 15), Some(15), "20 - 15%10=5 → 15");
    }

    #[test]
    fn trade_event_codes_dispatch() {
        assert_eq!(trade_event_from_code(22), Some(TradeEvent::TradeBuy));
        assert_eq!(trade_event_from_code(40), Some(TradeEvent::PageDown));
        assert_eq!(trade_event_from_code(0), None, "未知码 → None");
        assert_eq!(trade_event_from_code(100), None, "未知码 → None");
    }
}
