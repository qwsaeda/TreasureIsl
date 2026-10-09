//! S3.9 卡牌/事件系统正式实现。
//!
//! 依据 `docs/spec/cards.md`：
//! - 卡牌效果 [`CardEffect`]（Damage/Heal/Gold/Move/Unknown，spec §c `sub_3D1B4` 4 种效果码）；
//! - 卡牌定义 [`CardDef`]（`id` + `effect`）；
//! - 卡牌状态 [`CardState`]（`hp` + `gold`）；
//! - 应用结果 [`CardResult`]（`Applied(CardEffect)` / `UnknownEvent`）；
//! - 占位卡牌目录 [`CARD_CATALOG`]（5 张卡，spec §a）；
//! - [`CardEngine`]：卡牌引擎（`catalog: Vec<CardDef>`）；
//! - `event_to_card` / `apply` / `all_event_ids` / `card_by_id` 方法。
//!
//! 行为确定（同输入逐位相同），不允许 rand。
//! 随机分支（`sub_2101C` case 3 的 `rand()%25+1` 事件生成）不在此建模，
//! 随机序列对齐归 S3.12。

/// 卡牌效果（spec §c，`sub_3D1B4` case 1..4 → 4 种效果码，**推测**语义映射）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardEffect {
    /// 扣 HP（效果码 1，**推测**）。
    Damage(u32),
    /// 加 HP（效果码 2，**推测**）。
    Heal(u32),
    /// 加减金（效果码 3，**推测**）。
    Gold(u32),
    /// 地图移动（效果码 4，**推测**）。
    Move,
    /// 未知效果（`sub_3D1B4` default → 0，spec §c）。
    Unknown,
}

/// 卡牌定义（spec §a，`cards\cards.ini` `card%d` 记录的 oracle 化）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDef {
    /// 卡牌 id（`u32`，对应 `cards\cards.ini` 的 `card%d` 序号，1-based）。
    pub id: u32,
    /// 卡牌效果（spec §c，`sub_3D1B4` 效果码 1..4 的 oracle 化）。
    pub effect: CardEffect,
}

/// 卡牌状态（spec §b，`sub_32960` 存储的卡牌对象 + 金量的 oracle 化）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardState {
    /// 当前 HP（`Damage` 扣减 / `Heal` 加成的目标字段）。
    pub hp: u32,
    /// 当前金（`Gold` 加减的目标字段）。
    pub gold: u32,
}

/// 卡牌应用结果（spec §c）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardResult {
    /// 效果已应用（返回所应用的效果，供测试断言）。
    Applied(CardEffect),
    /// 未知事件（`event_id` 在 catalog 中无对应卡牌 → 无处理路径，spec §d）。
    UnknownEvent,
}

/// 占位卡牌目录（5 张卡，spec §a；效果 **推测** 映射，对照 `sub_19F9C` 初值价格表）。
pub const CARD_CATALOG: [CardDef; 5] = [
    CardDef { id: 1, effect: CardEffect::Damage(10) },
    CardDef { id: 2, effect: CardEffect::Heal(15) },
    CardDef { id: 3, effect: CardEffect::Gold(20) },
    CardDef { id: 4, effect: CardEffect::Move },
    CardDef { id: 5, effect: CardEffect::Unknown },
];

/// `event_id` → 卡牌定义映射（spec §b，`sub_32960` 统一存储语义）。
///
/// 规则（spec §b 重映射表 + `sub_32960` 行 14-15）：
/// - `event_id` 1..=5：直接映射到 `catalog[i-1]`（卡牌 id = `event_id`）；
/// - `event_id` 6..=25：映射到 `catalog[(event_id - 1) % 5]`（**推测**：事件 id 循环对应 5 张卡）；
/// - `catalog` 为空 → 返回 `None`（无处理路径，spec §d）。
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn event_to_card(event_id: u32, catalog: &[CardDef]) -> Option<&CardDef> {
    if catalog.is_empty() {
        return None;
    }
    if event_id == 0 {
        return None;
    }
    let idx = if event_id <= catalog.len() as u32 {
        event_id - 1
    } else {
        (event_id - 1) % catalog.len() as u32
    };
    catalog.get(idx as usize)
}

/// 卡牌效果应用（spec §c，确定性，无 rand）。
///
/// 规则（spec §c 直译）：
/// - `Damage(n)`：`state.hp = hp.saturating_sub(n)`；
/// - `Heal(n)`：`state.hp += n`（u32 饱和加）；
/// - `Gold(n)`：`state.gold += n`（**推测**：仅加，exe 侧扣金由贸易系统 `sub_54D1C` 处理）；
/// - `Move`：不改 `state`（地图移动归 S3.5，oracle 层仅记录效果）；
/// - `Unknown`：不改 `state`（`sub_3D1B4` default → 0，无效果）。
///
/// 返回 `Applied(effect)`；`event_id` 无对应卡牌时返回 `UnknownEvent`。
#[must_use]
pub fn apply_card(card: &CardDef, state: &mut CardState) -> CardResult {
    match card.effect {
        CardEffect::Damage(n) => {
            state.hp = state.hp.saturating_sub(n);
            CardResult::Applied(CardEffect::Damage(n))
        }
        CardEffect::Heal(n) => {
            state.hp = state.hp.saturating_add(n);
            CardResult::Applied(CardEffect::Heal(n))
        }
        CardEffect::Gold(n) => {
            state.gold = state.gold.saturating_add(n);
            CardResult::Applied(CardEffect::Gold(n))
        }
        CardEffect::Move => CardResult::Applied(CardEffect::Move),
        CardEffect::Unknown => CardResult::Applied(CardEffect::Unknown),
    }
}

/// 全部有处理路径的 `event_id` 清单（spec §b，25 个 id，100% 覆盖）。
///
/// 规则（spec §b `sub_2101C` case 3 重映射表）：
/// - `event_id` 1..=25 全部有处理路径（`sub_32960` 统一存储）；
/// - 重映射 {4→8, 13→17, 14→18, 15→19} 为 exe 侧 id 修正（`rand()%25+1` 生成后）；
/// - 返回 1..=25 共 25 个 id（`catalog` 为空时返回空列表，表示无处理路径）。
#[must_use]
pub fn all_event_ids(catalog: &[CardDef]) -> Vec<u32> {
    if catalog.is_empty() {
        return Vec::new();
    }
    (1..=25).collect()
}

/// 卡牌引擎：持有卡牌目录，提供事件映射/效果应用/全事件清单。
#[derive(Debug, Clone)]
pub struct CardEngine {
    catalog: Vec<CardDef>,
}

impl CardEngine {
    /// 新建卡牌引擎（指定卡牌目录）。
    #[must_use]
    pub fn new(catalog: Vec<CardDef>) -> Self {
        Self { catalog }
    }

    /// 占位卡牌目录（`CARD_CATALOG` 重导出，spec §a）。
    #[must_use]
    pub fn placeholder() -> Self {
        Self::new(CARD_CATALOG.to_vec())
    }

    /// 卡牌目录（只读，spec §a）。
    #[must_use]
    pub fn catalog(&self) -> &[CardDef] {
        &self.catalog
    }

    /// `event_id` → 卡牌定义映射（spec §b）。
    #[must_use]
    pub fn event_to_card(&self, event_id: u32) -> Option<&CardDef> {
        event_to_card(event_id, &self.catalog)
    }

    /// 卡牌效果应用（spec §c）。
    ///
    /// `event_id` 无对应卡牌时 → `UnknownEvent`（spec §d 无路径标注）。
    #[must_use]
    pub fn apply(&self, event_id: u32, state: &mut CardState) -> CardResult {
        match event_to_card(event_id, &self.catalog) {
            Some(card) => apply_card(card, state),
            None => CardResult::UnknownEvent,
        }
    }

    /// 全部有处理路径的 `event_id` 清单（spec §b）。
    #[must_use]
    pub fn all_event_ids(&self) -> Vec<u32> {
        all_event_ids(&self.catalog)
    }

    /// 按卡牌 id 查定义（`catalog` 内线性查找，无 → `None`）。
    #[must_use]
    pub fn card_by_id(&self, id: u32) -> Option<&CardDef> {
        self.catalog.iter().find(|c| c.id == id)
    }
}

impl Default for CardEngine {
    fn default() -> Self {
        Self::placeholder()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_event_to_card_direct() {
        let engine = CardEngine::placeholder();
        for (i, card) in CARD_CATALOG.iter().enumerate() {
            let r = engine.event_to_card(u32::try_from(i + 1).unwrap_or(0));
            assert!(r.is_some(), "event_id {} 应有对应卡牌", i + 1);
            if let Some(c) = r {
                assert_eq!(c.id, card.id);
            }
        }
    }

    #[test]
    fn engine_event_to_card_wraps() {
        let engine = CardEngine::placeholder();
        let r = engine.event_to_card(6);
        assert!(r.is_some(), "event_id 6 应循环映射到 catalog[0]");
        if let Some(c) = r {
            assert_eq!(c.id, 1, "6 % 5 = 1 → id=1");
        }

        let r2 = engine.event_to_card(25);
        assert!(r2.is_some(), "event_id 25 应循环映射");
        if let Some(c) = r2 {
            assert_eq!(c.id, 5, "25 % 5 = 0 → id=5");
        }
    }

    #[test]
    fn engine_event_to_card_empty_catalog_none() {
        let engine = CardEngine::new(Vec::new());
        assert_eq!(engine.event_to_card(1), None, "空 catalog → None");
        assert_eq!(engine.event_to_card(0), None, "event_id=0 → None");
    }

    #[test]
    fn engine_apply_damage() {
        let engine = CardEngine::placeholder();
        let mut state = CardState { hp: 100, gold: 0 };
        let r = engine.apply(1, &mut state);
        assert_eq!(r, CardResult::Applied(CardEffect::Damage(10)));
        assert_eq!(state.hp, 90);
    }

    #[test]
    fn engine_apply_heal() {
        let engine = CardEngine::placeholder();
        let mut state = CardState { hp: 80, gold: 0 };
        let r = engine.apply(2, &mut state);
        assert_eq!(r, CardResult::Applied(CardEffect::Heal(15)));
        assert_eq!(state.hp, 95);
    }

    #[test]
    fn engine_apply_gold() {
        let engine = CardEngine::placeholder();
        let mut state = CardState { hp: 0, gold: 0 };
        let r = engine.apply(3, &mut state);
        assert_eq!(r, CardResult::Applied(CardEffect::Gold(20)));
        assert_eq!(state.gold, 20);
    }

    #[test]
    fn engine_apply_move_no_state_change() {
        let engine = CardEngine::placeholder();
        let mut state = CardState { hp: 100, gold: 50 };
        let r = engine.apply(4, &mut state);
        assert_eq!(r, CardResult::Applied(CardEffect::Move));
        assert_eq!(state.hp, 100);
        assert_eq!(state.gold, 50);
    }

    #[test]
    fn engine_all_event_ids() {
        let engine = CardEngine::placeholder();
        let ids = engine.all_event_ids();
        assert_eq!(ids.len(), 25, "25 个 event_id");
        assert_eq!(ids[0], 1);
        assert_eq!(ids[24], 25);
        let empty = CardEngine::new(Vec::new()).all_event_ids();
        assert!(empty.is_empty(), "空 catalog → 空列表");
    }

    #[test]
    fn engine_all_event_ids_have_processing_path() {
        let engine = CardEngine::placeholder();
        for id in engine.all_event_ids() {
            assert!(engine.event_to_card(id).is_some(), "event_id {id} 无处理路径");
        }
    }

    #[test]
    fn engine_card_by_id() {
        let engine = CardEngine::placeholder();
        assert_eq!(engine.card_by_id(1).map(|c| c.id), Some(1));
        assert_eq!(engine.card_by_id(99), None, "不存在 → None");
    }

    #[test]
    fn engine_empty_catalog_no_path() {
        let engine = CardEngine::new(Vec::new());
        assert_eq!(engine.event_to_card(1), None, "空 catalog → None");
        assert_eq!(engine.all_event_ids(), Vec::<u32>::new(), "空 catalog → 空列表");
        let mut state = CardState::default();
        assert_eq!(
            engine.apply(1, &mut state),
            CardResult::UnknownEvent,
            "空 catalog → UnknownEvent"
        );
    }
}
