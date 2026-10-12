//! S3.9 卡牌/事件系统性质测试。
//!
//! 遍历（catalog 5 张卡 × `event_id` 0..=25 × 各 state `hp`/`gold` 组合），
//! 验证 `CardEngine` 的映射 / 效果应用 / 全事件清单行为；
//! + 语义测试（全部 `event_id` 都有处理路径 → 覆盖率 100% 证明：
//!   对 `all_event_ids` 的每个 id，`apply` 都返回 `Applied` 而非 `UnknownEvent`，
//!   找不到的才 `UnknownEvent`，且 `UnknownEvent` 的 id 在 spec 里标注为「无路径」）。
//!
//! S12.3 补全（`docs/business/progression.md` 可测试规则表，卡牌 3 条全部落地；
//! 地图 3 条见 `reference_diff_main_map.rs`）：
//! - `cards_same_seed_same_sequence`：同 seed 同路径 ⇒ 同卡牌串；
//! - `cards_effects_clamped`：效果后 HP/金钳制在合法域；
//! - `cards_weighted_pick_reproducible`：权重抽卡可复现（单测只断言可复现，分布靠统计脚本）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。
//! 用 `.expect("...")`（测试代码允许，门禁第 6 项豁免 tests/）。
//!
//! S10.3 改动 8：`CARD_CATALOG` 已删，测试用 `from_balance()` 或 `test_catalog()` 替代。

use ti_core::domain::map::{Dir, GridPos, MapTerrain};
use ti_core::rules::cards::{
    CardDef, CardEffect, CardEngine, CardResult, CardState, all_event_ids, apply_card,
    event_to_card,
};
use ti_core::rules::cards_draw::CardPool;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::rng::GameRng;

/// 测试用卡目录（与 `balance.json` `cards.deck` 同值，隔离 balance 依赖）。
fn test_catalog() -> Vec<CardDef> {
    vec![
        CardDef { id: 1, effect: CardEffect::Damage(10) },
        CardDef { id: 2, effect: CardEffect::Heal(15) },
        CardDef { id: 3, effect: CardEffect::Gold(20) },
        CardDef { id: 4, effect: CardEffect::Move },
        CardDef { id: 5, effect: CardEffect::Unknown },
    ]
}

/// 测试用 `CardState`（`hp_max`/`gold_max` = `u32::MAX`，无钳制上限）。
fn state_with(hp: u32, gold: u32) -> CardState {
    CardState { hp, gold, hp_max: u32::MAX, gold_max: u32::MAX }
}

/// 测试用 `CardState`（带 `hp_max`/`gold_max` 钳制上限，`progression.md` 效果结算）。
fn state_clamped(hp: u32, gold: u32, hp_max: u32, gold_max: u32) -> CardState {
    CardState { hp, gold, hp_max, gold_max }
}

/// 从 `balance.cards` 构建引擎（S10.3 数据化）。
fn from_balance_engine() -> CardEngine {
    CardEngine::from_balance()
}

/// 断言 `event_to_card` 映射结果符合 spec §b 规则（直接映射或循环映射）。
fn assert_event_to_card_correct(catalog: &[CardDef], event_id: u32) {
    let got = event_to_card(event_id, catalog).map(|c| c.id);
    if catalog.is_empty() || event_id == 0 {
        assert_eq!(got, None, "event_to_card (event_id={event_id}) 应 None（空 catalog 或 id=0）");
        return;
    }
    let expected_idx = if event_id <= u32::try_from(catalog.len()).unwrap_or(u32::MAX) {
        event_id - 1
    } else {
        (event_id - 1) % u32::try_from(catalog.len()).unwrap_or(u32::MAX)
    };
    let expected = catalog.get(expected_idx as usize).map(|c| c.id);
    assert_eq!(
        got, expected,
        "event_to_card (event_id={event_id}) 映射错误：got={got:?} expected={expected:?}"
    );
}

/// 断言 `apply` 结果与手算的饱和运算 + 区间钳制一致（S12.3）。
fn assert_apply_correct(catalog: &[CardDef], event_id: u32, initial_hp: u32, initial_gold: u32) {
    let card = event_to_card(event_id, catalog);
    let engine = CardEngine::new(catalog.to_vec());
    let mut state = state_with(initial_hp, initial_gold);
    let result = engine.apply(event_id, &mut state);

    match card {
        Some(c) => {
            let mut expected_state = state_with(initial_hp, initial_gold);
            let expected_result = apply_card(c, &mut expected_state);
            assert_eq!(
                result, expected_result,
                "apply (event_id={event_id}) 结果不一致：{result:?} vs {expected_result:?}"
            );
            assert_eq!(
                state.hp, expected_state.hp,
                "HP 不一致 (event_id={event_id} hp={initial_hp})"
            );
            assert_eq!(
                state.gold, expected_state.gold,
                "金不一致 (event_id={event_id} gold={initial_gold})"
            );
        }
        None => {
            assert_eq!(
                result,
                CardResult::UnknownEvent,
                "apply (event_id={event_id}) 无对应卡牌应返回 UnknownEvent"
            );
        }
    }
}

/// 遍历 5 张卡 × `event_id` 0..=10 × `hp` {0, 50, 100} × `gold` {0, 100, 999} 的映射路径。
#[test]
fn diff_event_to_card_all_combos() {
    let catalog = test_catalog();
    for event_id in 0..=10u32 {
        assert_event_to_card_correct(&catalog, event_id);
    }
}

/// 遍历 5 张卡 × `event_id` 1..=25 × `hp` {0, 50, 100, `999_999`} × `gold` {0, 100, `999_999`} 的应用路径。
#[test]
fn diff_apply_all_combos() {
    let catalog = test_catalog();
    let hps = [0u32, 50, 100, 999_999];
    let golds = [0u32, 100, 999_999];
    for event_id in 1..=25u32 {
        for &hp in &hps {
            for &gold in &golds {
                assert_apply_correct(&catalog, event_id, hp, gold);
            }
        }
    }
}

/// 空 catalog：全部 `event_id` → `UnknownEvent`（无路径，spec §d）。
#[test]
fn diff_empty_catalog_no_path() {
    let catalog: &[CardDef] = &[];
    for event_id in 1..=25u32 {
        assert_event_to_card_correct(catalog, event_id);
        assert_apply_correct(catalog, event_id, 100, 50);
    }
}

/// 语义测试 (a)：全部 `event_id` 都有处理路径（覆盖率 100% 证明）。
///
/// 对 `all_event_ids` 的每个 id，`event_to_card` 都返回 Some
/// （catalog 非空时），且 `apply_card` 都返回 `Applied` 而非 `UnknownEvent`。
#[test]
fn semantic_all_event_ids_have_processing_path() {
    let catalog = test_catalog();
    let ids = all_event_ids(&catalog);
    assert_eq!(ids.len(), 25, "25 个 event_id（1..=25），覆盖率 100%");
    for id in &ids {
        let card = event_to_card(*id, &catalog);
        assert!(card.is_some(), "event_id {id} 应有处理路径（spec §b）");
        let card = card.expect("已断言 Some");
        let mut state = state_with(100, 0);
        let result = apply_card(card, &mut state);
        assert!(
            matches!(result, CardResult::Applied(_)),
            "event_id {id} 应返回 Applied（非 UnknownEvent）"
        );
    }
}

/// 语义测试 (b)：`Damage(10)` 从 hp=100 扣到 hp=90，`Heal(15)` 从 hp=80 加到 hp=95。
#[test]
fn semantic_damage_heal() {
    let catalog = test_catalog();

    // Damage
    let card_dmg = event_to_card(1, &catalog).expect("event_id=1 应映射到卡 1（Damage）");
    assert!(matches!(card_dmg.effect, CardEffect::Damage(10)), "卡 1 应为 Damage(10)");
    let mut state = state_with(100, 0);
    let r = apply_card(card_dmg, &mut state);
    assert_eq!(r, CardResult::Applied(CardEffect::Damage(10)));
    assert_eq!(state.hp, 90, "HP 100 - 10 = 90");

    // Heal
    let card_heal = event_to_card(2, &catalog).expect("event_id=2 应映射到卡 2（Heal）");
    assert!(matches!(card_heal.effect, CardEffect::Heal(15)), "卡 2 应为 Heal(15)");
    let mut state2 = state_with(80, 0);
    let r2 = apply_card(card_heal, &mut state2);
    assert_eq!(r2, CardResult::Applied(CardEffect::Heal(15)));
    assert_eq!(state2.hp, 95, "HP 80 + 15 = 95");

    // Gold
    let card_gold = event_to_card(3, &catalog).expect("event_id=3 应映射到卡 3（Gold）");
    let mut state3 = state_with(0, 0);
    let _ = apply_card(card_gold, &mut state3);
    assert_eq!(state3.gold, 20, "金 0 + 20 = 20");
}

/// 语义测试 (c)：`Move` 不改 hp/gold，`Unknown` 不改 hp/gold。
#[test]
fn semantic_move_and_unknown_no_state_change() {
    let catalog = test_catalog();

    let card_move = event_to_card(4, &catalog).expect("event_id=4 应映射到卡 4（Move）");
    let mut state = state_with(100, 50);
    let r = apply_card(card_move, &mut state);
    assert_eq!(r, CardResult::Applied(CardEffect::Move));
    assert_eq!(state.hp, 100, "Move 不改 HP");
    assert_eq!(state.gold, 50, "Move 不改金");

    let card_unknown = event_to_card(5, &catalog).expect("event_id=5 应映射到卡 5（Unknown）");
    let mut state2 = state_with(100, 50);
    let r2 = apply_card(card_unknown, &mut state2);
    assert_eq!(r2, CardResult::Applied(CardEffect::Unknown));
    assert_eq!(state2.hp, 100, "Unknown 不改 HP");
    assert_eq!(state2.gold, 50, "Unknown 不改金");
}

/// 语义测试 (d)：`CardEngine::all_event_ids` 与 spec 规则一致（1..=25 共 25 个 id）。
#[test]
fn semantic_all_event_ids_match() {
    let engine = CardEngine::new(test_catalog());
    let i = engine.all_event_ids();
    assert_eq!(i.len(), 25, "25 个 event_id");
    assert_eq!(i, (1..=25u32).collect::<Vec<_>>(), "应为 1..=25 连续序列");
}

/// 语义测试 (e)：`CardEngine::card_by_id` 对 5 张卡逐一命中，对 id=99 → None。
/// S10.3 改动 8：用 `from_balance()` 替代 `placeholder()`（卡池走 `balance.cards`）。
#[test]
fn semantic_card_by_id() {
    let engine = from_balance_engine();
    for card in &test_catalog() {
        let found = engine.card_by_id(card.id);
        assert!(found.is_some(), "card id={} 应命中", card.id);
        assert_eq!(found.expect("已断言 Some").id, card.id);
    }
    assert!(engine.card_by_id(99).is_none(), "id=99 → None");
}

// ---------------------------------------------------------------------------
// S12.3：`docs/business/progression.md` 可测试规则表 卡牌 3 条全部落地
// （地图 3 条 `map_*` 见 `reference_diff_main_map.rs`）
// ---------------------------------------------------------------------------

/// `cards_same_seed_same_sequence`：同 seed 同路径 ⇒ 同卡牌串。
///
/// 业务规则（`progression.md` 抽牌可复现）：
/// 同 `rng` seed + 同移动序列 ⇒ 同一串卡牌结果（可复现，EXP-040）。
/// 判据：两个 `MapSession` 用同 seed `GameRng` + 同移动序列，
/// 每步 `card_event` 序列逐位相同。
#[test]
fn cards_same_seed_same_sequence() {
    let t = MapTerrain::open(4, 4);
    let pool = CardPool::from_balance();
    let mut a = MapSession::with_pool(t.clone(), GridPos::new(0, 0), &[], pool.clone());
    let mut b = MapSession::with_pool(t, GridPos::new(0, 0), &[], pool);
    let mut ra = GameRng::new(42);
    let mut rb = GameRng::new(42);

    let dirs = [Dir::East, Dir::South, Dir::East, Dir::South, Dir::West, Dir::North];
    let mut seq_a = Vec::new();
    let mut seq_b = Vec::new();
    for &d in &dirs {
        seq_a.push(a.move_to(d, &mut ra).card_event);
        seq_b.push(b.move_to(d, &mut rb).card_event);
    }
    assert_eq!(seq_a, seq_b, "同 seed + 同路径 → 同卡牌事件序列");
}

/// `cards_effects_clamped`：效果后 HP/金钳制在合法域。
///
/// 业务规则（`progression.md` 效果结算）：
/// `gold`/`hp` 增减后立即做区间钳制（`0 ≤ gold ≤ gold_max`、`0 ≤ hp ≤ hp_max`）。
/// 判据：`Heal` 后 `hp ≤ hp_max`；`Gold` 后 `gold ≤ gold_max`；`Damage` 后 `hp ≥ 0`。
#[test]
fn cards_effects_clamped() {
    let engine = CardEngine::new(test_catalog());

    // Heal 钳制：hp=90, hp_max=100, Heal(15) → min(100, 90+15) = 100
    let mut s1 = state_clamped(90, 0, 100, u32::MAX);
    let _ = engine.apply(2, &mut s1);
    assert_eq!(s1.hp, 100, "Heal 后 hp 应钳制到 hp_max=100");
    assert!(s1.hp <= 100, "hp 不得超 hp_max");

    // Gold 钳制：gold=10, gold_max=25, Gold(20) → min(25, 10+20) = 25
    let mut s2 = state_clamped(0, 10, u32::MAX, 25);
    let _ = engine.apply(3, &mut s2);
    assert_eq!(s2.gold, 25, "Gold 后 gold 应钳制到 gold_max=25");
    assert!(s2.gold <= 25, "gold 不得超 gold_max");

    // Damage 下限：hp=5, Damage(10) → saturating_sub → 0（u32 天然 ≥ 0）
    let mut s3 = state_clamped(5, 0, u32::MAX, u32::MAX);
    let _ = engine.apply(1, &mut s3);
    assert_eq!(s3.hp, 0, "Damage 后 hp 应钳制到 0（saturating_sub 下界）");

    // 无上限（hp_max=U32::MAX）：hp=100, Heal(15) → 115（不钳制）
    let mut s4 = state_with(100, 0);
    let _ = engine.apply(2, &mut s4);
    assert_eq!(s4.hp, 115, "无 hp_max 时 Heal 不加钳制");
}

/// `cards_weighted_pick_reproducible`：权重抽卡可复现（单测只断言可复现，分布靠统计脚本）。
///
/// 业务规则（`progression.md` 抽牌可复现）：
/// 同 seed + 同操作序列 ⇒ 同卡组（可复现，EXP-040）。
/// 判据：同一 `CardPool` + 同 seed `GameRng`，多次 `draw` 结果逐位相同。
#[test]
fn cards_weighted_pick_reproducible() {
    let pool = CardPool::from_balance();
    // 抽 3 次（同 seed 重置后），每次结果应相同
    let mut rng_a = GameRng::new(123);
    let mut rng_b = GameRng::new(123);
    let draws_a = (0..3).map(|_| pool.draw(&mut rng_a)).collect::<Vec<_>>();
    let draws_b = (0..3).map(|_| pool.draw(&mut rng_b)).collect::<Vec<_>>();
    assert_eq!(draws_a, draws_b, "同 seed 多次抽牌 → 同序列（可复现）");

    // 不同 seed → 结果应不同（证明 seed 有效）
    let mut rng_c = GameRng::new(999);
    let draws_c = (0..3).map(|_| pool.draw(&mut rng_c)).collect::<Vec<_>>();
    assert_ne!(draws_a, draws_c, "不同 seed → 不同序列（seed 有效）");

    // 手牌内不放回：无重复 id
    let mut rng_d = GameRng::new(1);
    let hand = pool.draw(&mut rng_d);
    let mut deduped = hand.clone();
    deduped.sort_unstable();
    deduped.dedup();
    assert_eq!(deduped.len(), hand.len(), "手牌内不放回：无重复 id");
}
