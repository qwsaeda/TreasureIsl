//! S3.9 卡牌/事件系统性质测试。
//!
//! 遍历（catalog 5 张卡 × `event_id` 0..=25 × 各 state `hp`/`gold` 组合），
//! 验证 `CardEngine` 的映射 / 效果应用 / 全事件清单行为；
//! + 语义测试（全部 `event_id` 都有处理路径 → 覆盖率 100% 证明：
//!   对 `all_event_ids` 的每个 id，`apply` 都返回 `Applied` 而非 `UnknownEvent`，
//!   找不到的才 `UnknownEvent`，且 `UnknownEvent` 的 id 在 spec 里标注为「无路径」）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。
//! 用 `.expect("...")`（测试代码允许，门禁第 6 项豁免 tests/）。

use ti_core::rules::cards::{
    CARD_CATALOG, CardDef, CardEffect, CardEngine, CardResult, CardState, all_event_ids,
    apply_card, event_to_card,
};

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

/// 断言 `apply` 结果与手算的饱和运算一致。
fn assert_apply_correct(catalog: &[CardDef], event_id: u32, initial_hp: u32, initial_gold: u32) {
    let card = event_to_card(event_id, catalog);
    let engine = CardEngine::new(catalog.to_vec());
    let mut state = CardState { hp: initial_hp, gold: initial_gold };
    let result = engine.apply(event_id, &mut state);

    match card {
        Some(c) => {
            let mut expected_state = CardState { hp: initial_hp, gold: initial_gold };
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
    let catalog = &CARD_CATALOG;
    for event_id in 0..=10u32 {
        assert_event_to_card_correct(catalog, event_id);
    }
}

/// 遍历 5 张卡 × `event_id` 1..=25 × `hp` {0, 50, 100, `999_999`} × `gold` {0, 100, `999_999`} 的应用路径。
#[test]
fn diff_apply_all_combos() {
    let catalog = &CARD_CATALOG;
    let hps = [0u32, 50, 100, 999_999];
    let golds = [0u32, 100, 999_999];
    for event_id in 1..=25u32 {
        for &hp in &hps {
            for &gold in &golds {
                assert_apply_correct(catalog, event_id, hp, gold);
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
/// （`CARD_CATALOG` 非空时），且 `apply_card` 都返回 `Applied` 而非 `UnknownEvent`。
#[test]
fn semantic_all_event_ids_have_processing_path() {
    let catalog = &CARD_CATALOG;
    let ids = all_event_ids(catalog);
    assert_eq!(ids.len(), 25, "25 个 event_id（1..=25），覆盖率 100%");
    for id in &ids {
        let card = event_to_card(*id, catalog);
        assert!(card.is_some(), "event_id {id} 应有处理路径（spec §b）");
        let card = card.expect("已断言 Some");
        let mut state = CardState { hp: 100, gold: 0 };
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
    let catalog = &CARD_CATALOG;

    // Damage
    let card_dmg = event_to_card(1, catalog).expect("event_id=1 应映射到卡 1（Damage）");
    assert!(matches!(card_dmg.effect, CardEffect::Damage(10)), "卡 1 应为 Damage(10)");
    let mut state = CardState { hp: 100, gold: 0 };
    let r = apply_card(card_dmg, &mut state);
    assert_eq!(r, CardResult::Applied(CardEffect::Damage(10)));
    assert_eq!(state.hp, 90, "HP 100 - 10 = 90");

    // Heal
    let card_heal = event_to_card(2, catalog).expect("event_id=2 应映射到卡 2（Heal）");
    assert!(matches!(card_heal.effect, CardEffect::Heal(15)), "卡 2 应为 Heal(15)");
    let mut state2 = CardState { hp: 80, gold: 0 };
    let r2 = apply_card(card_heal, &mut state2);
    assert_eq!(r2, CardResult::Applied(CardEffect::Heal(15)));
    assert_eq!(state2.hp, 95, "HP 80 + 15 = 95");

    // Gold
    let card_gold = event_to_card(3, catalog).expect("event_id=3 应映射到卡 3（Gold）");
    let mut state3 = CardState { hp: 0, gold: 0 };
    let _ = apply_card(card_gold, &mut state3);
    assert_eq!(state3.gold, 20, "金 0 + 20 = 20");
}

/// 语义测试 (c)：`Move` 不改 hp/gold，`Unknown` 不改 hp/gold。
#[test]
fn semantic_move_and_unknown_no_state_change() {
    let catalog = &CARD_CATALOG;

    let card_move = event_to_card(4, catalog).expect("event_id=4 应映射到卡 4（Move）");
    let mut state = CardState { hp: 100, gold: 50 };
    let r = apply_card(card_move, &mut state);
    assert_eq!(r, CardResult::Applied(CardEffect::Move));
    assert_eq!(state.hp, 100, "Move 不改 HP");
    assert_eq!(state.gold, 50, "Move 不改金");

    let card_unknown = event_to_card(5, catalog).expect("event_id=5 应映射到卡 5（Unknown）");
    let mut state2 = CardState { hp: 100, gold: 50 };
    let r2 = apply_card(card_unknown, &mut state2);
    assert_eq!(r2, CardResult::Applied(CardEffect::Unknown));
    assert_eq!(state2.hp, 100, "Unknown 不改 HP");
    assert_eq!(state2.gold, 50, "Unknown 不改金");
}

/// 语义测试 (d)：`CardEngine::all_event_ids` 与 spec 规则一致（1..=25 共 25 个 id）。
#[test]
fn semantic_all_event_ids_match() {
    let engine = CardEngine::new(CARD_CATALOG.to_vec());
    let i = engine.all_event_ids();
    assert_eq!(i.len(), 25, "25 个 event_id");
    assert_eq!(i, (1..=25u32).collect::<Vec<_>>(), "应为 1..=25 连续序列");
}

/// 语义测试 (e)：`CardEngine::card_by_id` 对 5 张卡逐一命中，对 id=99 → None。
#[test]
fn semantic_card_by_id() {
    let engine = CardEngine::placeholder();
    for card in &CARD_CATALOG {
        let found = engine.card_by_id(card.id);
        assert!(found.is_some(), "card id={} 应命中", card.id);
        assert_eq!(found.expect("已断言 Some").id, card.id);
    }
    assert!(engine.card_by_id(99).is_none(), "id=99 → None");
}
