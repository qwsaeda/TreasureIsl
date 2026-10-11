//! S8.3.c 地图卡牌（数据化版）：卡池来自 `balance.cards`，抽牌走可复现 RNG。
//!
//! 设计（`docs/business/progression.md` §卡片）：
//! - **卡池单一事实源** = `assets/data/balance.json → cards`（[`crate::data::cards`]）；
//!   旧 `rules/cards.rs` 的 `CARD_CATALOG` 常量降级为参照物（S8 定位 §1），本模块不再引用；
//! - **抽牌可复现** = 用调用方注入的 [`crate::rules::rng::GameRng`]（可播种 PCG，`rand_pcg`，S8.7 落地，确定性；EXP-040），
//!   同 seed + 同操作序列 → 同卡组，可被存读档/回放验证；
//! - **无占位**：抽不到（`deck` 为空 / `hand_size == 0`）返回空手，不 panic、不 fallback。
//!
//! 与旧 `rules/cards.rs` 的边界：
//! - 旧模块的 `event_to_card`（事件 id → 卡映射）仍有效，但**卡池来源**改走本模块的
//!   [`CardPool::from_balance`]；旧 `CARD_CATALOG` 的「5 张占位卡」不再作为运行时来源。
//! - 旧 `apply_card` 的状态机（Damage/Heal/Gold/Move 对 `CardState` 的修改）保留复用，
//!   本模块只提供「抽哪些卡」，不重复实现效果应用。

use crate::data::CardDef;

use crate::rules::rng::GameRng;

/// 卡牌池：抽牌的唯一入口（卡定义来自 `balance.cards`，抽牌顺序由 RNG 决定）。
///
/// 持有 `Vec<CardDef>`（`from_balance` 读 `balance.cards.deck` 一次）；
/// `seed` 由调用方注入（存读档/新游戏入口），保证可复现。
#[derive(Debug, Clone, PartialEq)]
pub struct CardPool {
    /// 卡池（`balance.cards.deck`，按 `id` 有序）。
    deck: Vec<CardDef>,
    /// 一次抽牌张数（`balance.cards.hand_size`）。
    hand_size: u32,
    /// 事件触发概率（`balance.cards.event_chance`，`0.0..=1.0`；地图移动后触发卡牌的概率）。
    event_chance: f32,
}

impl CardPool {
    /// 从 `balance.cards` 构建卡池（卡定义单一事实源，S8 数据化）。
    ///
    /// `deck` 为空 → 抽牌恒返回空手（无卡可抽，非错误）；`hand_size` 上限取 `deck.len()`
    /// 避免越界（`hand_size > deck.len()` 时按 `deck.len()` 截断，不重抽同张）。
    #[must_use]
    pub fn from_balance() -> Self {
        let Ok(b) = crate::data::balance() else {
            return Self { deck: Vec::new(), hand_size: 0, event_chance: 0.0 };
        };
        Self {
            deck: b.cards.deck.clone(),
            hand_size: b.cards.hand_size.min(u32::try_from(b.cards.deck.len()).unwrap_or(0)),
            event_chance: b.cards.event_chance,
        }
    }

    /// 事件触发概率（`balance.cards.event_chance`，`0.0..=1.0`；地图移动后触发卡牌用）。
    /// 整表改坏 / 空池 → `0.0`（永不触发，不 panic）。
    #[must_use]
    pub fn event_chance(&self) -> f32 {
        self.event_chance
    }

    /// 一次抽牌（可复现）：用 `rng` 取 `hand_size` 张卡（不放回、按权重抽样）。
    ///
    /// 权重抽样规则（`docs/business/progression.md` §卡片「可复现抽牌」）：
    /// - 总权重 `W = Σ weight`；`rng.in_range(W)` 落在某张卡的累计权重区间 → 选中该卡；
    /// - 选中后**从本局手牌排除**（不放回），避免一次抽中同张；
    /// - `rng` 的调用次数 = `hand_size`（每张卡恰一次 `in_range`），保证同 seed 同结果。
    ///
    /// 返回按抽中顺序的 `id` 列表；`deck` 空或 `hand_size == 0` → 空列表。
    #[must_use]
    pub fn draw(&self, rng: &mut GameRng) -> Vec<u32> {
        if self.deck.is_empty() || self.hand_size == 0 {
            return Vec::new();
        }
        let total: u32 = self.deck.iter().map(|c| c.weight).sum();
        if total == 0 {
            return Vec::new();
        }
        // 不放回：用剩余卡累加权重；每张抽走一次。
        let mut remaining: Vec<bool> = vec![true; self.deck.len()];
        let mut hand: Vec<u32> = Vec::with_capacity(usize::try_from(self.hand_size).unwrap_or(0));
        for _ in 0..self.hand_size {
            let live_total: u32 = remaining
                .iter()
                .enumerate()
                .filter(|(_, in_hand)| **in_hand)
                .map(|(i, _)| self.deck[i].weight)
                .sum();
            if live_total == 0 {
                break;
            }
            let roll = rng.in_range(live_total);
            let mut acc = 0u32;
            for (i, in_hand) in remaining.iter().enumerate() {
                if !*in_hand {
                    continue;
                }
                acc += self.deck[i].weight;
                if roll < acc {
                    remaining[i] = false;
                    hand.push(self.deck[i].id);
                    break;
                }
            }
            // roll 必落在某张卡区间内（`roll < live_total`），循环必 break；
            // 理论上不会走到这里（防御性：若全排除仍 roll < acc 失败则本张跳过，下张继续）。
        }
        hand
    }

    /// 一次抽 `n` 张（地图事件抽牌用 `n=1`；`n==0` 或空池 → 空手）。
    ///
    /// 委托 [`CardPool::draw`]（权重不放回抽样）取前 `n` 张；`n > hand_size` 时按 `hand_size` 截断。
    /// 同 `rng` seed + 同 `n` → 同 `n` 张卡（可复现，EXP-040）。
    #[must_use]
    pub fn draw_one(&self, rng: &mut GameRng, n: u32) -> Vec<u32> {
        let count = usize::try_from(n).unwrap_or(0).min(self.hand_size() as usize);
        if count == 0 || self.deck.is_empty() {
            return Vec::new();
        }
        let hand = self.draw(rng);
        hand.into_iter().take(count).collect()
    }

    /// 卡牌定义（只读，`balance.cards.deck`）。
    #[must_use]
    pub fn deck(&self) -> &[CardDef] {
        &self.deck
    }

    /// 一次抽牌张数（`balance.cards.hand_size`，截断到 `deck.len()` 后）。
    #[must_use]
    pub fn hand_size(&self) -> u32 {
        self.hand_size
    }
}

impl CardPool {
    /// 直接构造（测试 / 自定义卡池用；不读 `balance`，`event_chance` 显式指定）。
    #[must_use]
    pub fn new(deck: Vec<CardDef>, hand_size: u32, event_chance: f32) -> Self {
        Self {
            hand_size: hand_size.min(u32::try_from(deck.len()).unwrap_or(0)),
            event_chance,
            deck,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CardEffectDef;

    // 测试用静态卡池（`from_balance` 读真实表；这里用 `new` 隔离 RNG 行为）。
    fn deck3() -> Vec<CardDef> {
        vec![
            CardDef {
                id: 1,
                name_key: "cards.1".into(),
                weight: 3,
                effect: CardEffectDef::Damage(10),
            },
            CardDef {
                id: 2,
                name_key: "cards.2".into(),
                weight: 2,
                effect: CardEffectDef::Heal(15),
            },
            CardDef {
                id: 3,
                name_key: "cards.3".into(),
                weight: 1,
                effect: CardEffectDef::Gold(20),
            },
        ]
    }

    #[test]
    fn empty_deck_returns_empty_hand() {
        let pool = CardPool::new(Vec::new(), 3, 0.35);
        let mut rng = GameRng::new(1);
        assert!(pool.draw(&mut rng).is_empty(), "空 deck → 空手");
    }

    #[test]
    fn zero_hand_size_returns_empty() {
        let pool = CardPool::new(deck3(), 0, 0.35);
        let mut rng = GameRng::new(1);
        assert!(pool.draw(&mut rng).is_empty(), "hand_size=0 → 空手");
    }

    #[test]
    fn reproducible_draw_same_seed() {
        let pool = CardPool::new(deck3(), 3, 0.35);
        let mut a = GameRng::new(42);
        let mut b = GameRng::new(42);
        assert_eq!(pool.draw(&mut a), pool.draw(&mut b), "同 seed 必须同卡组（可复现抽牌）");
    }

    #[test]
    fn draw_without_replacement_no_duplicate_within_hand() {
        let pool = CardPool::new(deck3(), 3, 0.35);
        let mut rng = GameRng::new(7);
        let hand = pool.draw(&mut rng);
        let mut deduped = hand.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), hand.len(), "手牌内不放回：无重复 id");
        assert!(hand.len() <= 3, "手牌数 ≤ hand_size");
    }

    #[test]
    fn draw_respects_hand_size_cap() {
        let pool = CardPool::new(deck3(), 5, 0.35);
        assert_eq!(pool.hand_size(), 3, "hand_size 截断到 deck.len()");
        let mut rng = GameRng::new(1);
        assert!(pool.draw(&mut rng).len() <= 3, "实际抽中 ≤ deck 张数（不放回）");
    }

    #[test]
    fn from_balance_uses_real_table() {
        // 读真实 `balance.cards`（编译期 `include_str!`，测试环境恒可用）。
        let pool = CardPool::from_balance();
        assert!(!pool.deck().is_empty(), "真实 balance 卡池非空");
        assert_eq!(pool.deck().len(), 5, "balance.json 5 张卡");
    }

    #[test]
    fn deck_expose_read_only() {
        let pool = CardPool::new(deck3(), 2, 0.35);
        assert_eq!(pool.deck().len(), 3);
        assert_eq!(pool.deck()[0].id, 1);
    }
}
