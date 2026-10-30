//! 商人（v2，2026-10-30 用户定案）：每图必有**武器商人** + **奴隶商人**。
//!
//! 单一事实源 = [`docs/business/territory.md`](../../../../docs/business/territory.md) §7 +
//! [`economy.md`](../../../../docs/business/economy.md) 资源节。
//!
//! 规则：
//! - **武器**：花金币买；**一人一把**（`武器 ≤ 人手`，多余武器无效 → 购买上限 = 人手）。
//! - **人手**：花金币买；**上限 = 船容量**（船越大可带越多水手）。
//! - 金币不足 / 已达上限 ⇒ 拒绝（不扣金、不发货）。

/// 购买结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuyResult {
    /// 剩余金币。
    pub gold_left: u32,
    /// 实际购得数量（受上限/金币约束，≤ 请求 qty）。
    pub qty_bought: u32,
}

/// 购买失败原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchantError {
    /// 金币不足（连 1 件都买不起）。
    InsufficientGold,
    /// 已达上限（人手或武器已满，无法再买）。
    CapReached,
}

/// 买武器：扣金 + 得武器；**上限 = 人手**（一人一把，多余武器无效）。
///
/// `gold`/`arms`/`crew` 为当前值，`qty` 为请求数量，`price` 为单价。
#[must_use]
pub fn buy_arms(
    gold: u32,
    arms: u32,
    crew: u32,
    qty: u32,
    price: u32,
) -> Result<BuyResult, MerchantError> {
    let price = price.max(1);
    let cap = crew.saturating_sub(arms); // 一人一把：还能买 = 人手 − 现有武器
    if cap == 0 {
        return Err(MerchantError::CapReached);
    }
    let want = qty.min(cap);
    let total = want.checked_mul(price).unwrap_or(u32::MAX);
    if total > gold {
        return Err(MerchantError::InsufficientGold);
    }
    Ok(BuyResult { gold_left: gold - total, qty_bought: want })
}

/// 买人手：扣金 + 得人手；**上限 = 船容量**。
#[must_use]
pub fn buy_crew(
    gold: u32,
    crew: u32,
    boat_cap: u32,
    qty: u32,
    price: u32,
) -> Result<BuyResult, MerchantError> {
    let price = price.max(1);
    let cap = boat_cap.saturating_sub(crew);
    if cap == 0 {
        return Err(MerchantError::CapReached);
    }
    let want = qty.min(cap);
    let total = want.checked_mul(price).unwrap_or(u32::MAX);
    if total > gold {
        return Err(MerchantError::InsufficientGold);
    }
    Ok(BuyResult { gold_left: gold - total, qty_bought: want })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_arms_debits_and_adds_capped_at_crew() {
        // 人手 10、武器 5：还能买 5；请求 8 → 只买 5。
        let r = buy_arms(1000, 5, 10, 8, 50).unwrap();
        assert_eq!(r.qty_bought, 5);
        assert_eq!(r.gold_left, 1000 - 5 * 50);
    }

    #[test]
    fn buy_arms_one_per_person_cap() {
        // 人手 10、武器 10：已满（一人一把）→ 拒绝。
        assert_eq!(buy_arms(1000, 10, 10, 1, 50), Err(MerchantError::CapReached));
    }

    #[test]
    fn buy_arms_insufficient_gold() {
        // 买 3 把需 150，只有 100 → 拒绝。
        assert_eq!(buy_arms(100, 0, 10, 3, 50), Err(MerchantError::InsufficientGold));
    }

    #[test]
    fn buy_crew_capped_at_boat_capacity() {
        // 船容量 20、现 15：还能买 5；请求 10 → 只买 5。
        let r = buy_crew(1000, 15, 20, 10, 30).unwrap();
        assert_eq!(r.qty_bought, 5);
        assert_eq!(r.gold_left, 1000 - 5 * 30);
    }

    #[test]
    fn buy_crew_at_capacity_rejected() {
        assert_eq!(buy_crew(1000, 20, 20, 1, 30), Err(MerchantError::CapReached));
    }

    #[test]
    fn buy_crew_insufficient_gold() {
        assert_eq!(buy_crew(20, 0, 20, 1, 30), Err(MerchantError::InsufficientGold));
    }

    #[test]
    fn zero_price_clamped() {
        // price=0 被钳到 1，避免除零/白嫖。
        let r = buy_arms(1000, 0, 10, 1, 0).unwrap();
        assert_eq!(r.qty_bought, 1);
        assert_eq!(r.gold_left, 999);
    }
}
