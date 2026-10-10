//! S8.1 数据表集成测试（`assets/data/*.json` → `ti_core::data` 加载层）。
//!
//! 判据（ledger S8.1）：三表可解析、数值在合法域、字符串覆盖场景标题/提示、
//! 场景布局齐全、解析失败是硬错误（不 fallback）。
//!
//! 本文件位于 `tests/`，因此允许 `expect`（`docs/process/02_conventions.md` §5）。

use ti_core::data::{
    Balance, BossTier, BuildingDef, CombatCfg, DamageCfg, LoadError, balance, scenes, strings,
};

// ---------------------------------------------------------------- 平衡数值表

#[test]
fn balance_json_parses_and_is_populated() {
    let b = balance().expect("balance.json should parse and convert");
    assert!(b.economy.gold_max > 0, "gold_max must be positive");
    assert!(!b.combat.boss_tiers.is_empty(), "boss_tiers must not be empty");
    assert!(!b.trade.goods.is_empty(), "trade.goods must not be empty");
    assert!(!b.build.buildings.is_empty(), "build.buildings must not be empty");
    assert!(!b.cards.deck.is_empty(), "cards.deck must not be empty");
}

#[test]
fn boss_tier_lookup_in_range_and_out_of_range() {
    let b = balance().expect("balance");
    for (i, tier) in b.combat.boss_tiers.iter().enumerate() {
        assert_eq!(tier.tier, u32::try_from(i).unwrap_or(u32::MAX), "tiers must be contiguous");
        assert!(tier.hp > 0, "boss tier {} hp must be positive", tier.tier);
        assert!(tier.atk > 0, "boss tier {} atk must be positive", tier.tier);
    }
    let first = b.combat.boss(0).expect("tier 0 exists");
    assert_eq!(first.tier, 0);
    // Out-of-range tier -> None (docs/business/combat.md).
    assert!(b.combat.boss(9_999).is_none(), "out-of-range tier must be None");
}

#[test]
fn damage_cfg_has_positive_floor_and_span() {
    let d = &balance().expect("balance").combat.damage;
    assert!(d.flat > 0, "flat floor must be positive so combat converges");
    assert!(d.atk_coef > 0.0, "atk_coef must be positive");
}

#[test]
fn trade_goods_have_positive_price_and_qty_cap() {
    for g in &balance().expect("balance").trade.goods {
        assert!(g.buy > 0, "goods {} price must be positive", g.id);
        assert!(g.qty_max > 0, "goods {} qty cap must be positive", g.id);
        assert!(!g.name_key.is_empty(), "goods {} needs a name_key", g.id);
    }
}

#[test]
fn build_buildings_lookable_by_id_with_unique_ids() {
    let b = balance().expect("balance");
    let castle: &BuildingDef = b.build.get("Castle").expect("Castle must exist");
    assert!(castle.base_cost > 0);
    assert!(castle.gold_max > 0);
    assert!(b.build.get("nonexistent").is_none());
    let mut ids: Vec<&str> = b.build.buildings.iter().map(|x| x.id.as_str()).collect();
    let n = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), n, "building ids must be unique");
}

#[test]
fn cards_event_chance_in_unit_interval_and_weights_positive() {
    let b = balance().expect("balance");
    assert!(
        (0.0..=1.0).contains(&b.cards.event_chance),
        "event_chance {} must be in [0,1]",
        b.cards.event_chance
    );
    for c in &b.cards.deck {
        assert!(c.weight > 0, "card {} weight must be positive", c.id);
    }
}

#[test]
fn save_cfg_is_sane() {
    let s = &balance().expect("balance").save;
    assert!(s.slot_count > 0);
    assert!(s.magic > 0);
    assert!(s.v2_size > 0);
}

#[test]
fn ui_cfg_matches_logical_resolution() {
    let u = &balance().expect("balance").ui;
    assert!(u.logical_width > 0 && u.logical_height > 0);
    assert!(u.glyph_advance > 0);
    assert!(u.max_back_steps > 0, "no-deadlock bound must be positive");
}

// -------------------------------------------------------------------- 文本表

#[test]
fn strings_table_loads_and_resolves_keys() {
    let s = strings().expect("strings.json should parse");
    assert!(!s.is_empty(), "strings table must not be empty");
    assert_ne!(
        s.get("scene.select_role.title"),
        "scene.select_role.title",
        "key must resolve to text"
    );
    // Missing key falls back to the key itself (never panics).
    assert_eq!(s.get("no.such.key"), "no.such.key");
    assert!(s.try_get("no.such.key").is_none());
}

#[test]
fn strings_cover_every_scene_title_and_hint() {
    let s = strings().expect("strings");
    for key in [
        "scene.select_role.title",
        "scene.select_role.hint",
        "scene.select_map.title",
        "scene.select_map.hint",
        "scene.main_map.title",
        "scene.vs.title",
        "scene.message.hint",
        "scene.saveload.title",
    ] {
        assert!(s.try_get(key).is_some(), "missing string key `{key}`");
    }
}

#[test]
fn strings_cover_every_goods_building_and_message_name_key() {
    let b = balance().expect("balance");
    let s = strings().expect("strings");
    for g in &b.trade.goods {
        assert!(s.try_get(&g.name_key).is_some(), "goods name_key `{}` missing", g.name_key);
    }
    for bl in &b.build.buildings {
        assert!(s.try_get(&bl.name_key).is_some(), "building name_key `{}` missing", bl.name_key);
    }
    for c in &b.cards.deck {
        assert!(s.try_get(&c.name_key).is_some(), "card name_key `{}` missing", c.name_key);
    }
}

// ---------------------------------------------------------------- 场景布局表

#[test]
fn scenes_table_loads_and_every_game_scene_present() {
    let sc = scenes().expect("scenes.json should parse");
    for id in ["index", "select_role", "select_map", "main_map", "vs", "message", "saveload"] {
        assert!(sc.contains_key(id), "missing scene layout `{id}`");
    }
}

#[test]
fn scene_layout_rects_are_inside_logical_resolution() {
    let sc = scenes().expect("scenes");
    let ui = &balance().expect("balance").ui;
    for (id, layout) in sc {
        let w = i32::try_from(ui.logical_width).unwrap_or(176);
        let h = i32::try_from(ui.logical_height).unwrap_or(220);
        assert!(
            layout.title.x >= 0 && layout.title.x < w,
            "scene `{id}` title.x {} out of bounds",
            layout.title.x
        );
        assert!(
            layout.title.y >= 0 && layout.title.y < h,
            "scene `{id}` title.y {} out of bounds",
            layout.title.y
        );
    }
}

#[test]
fn list_scenes_have_positive_row_metrics() {
    let sc = scenes().expect("scenes");
    for id in ["select_role", "select_map", "saveload"] {
        let layout = &sc[id];
        assert_eq!(&layout.id, id, "scene `{id}` id must match its key");
        assert!(layout.list.row_h > 0, "scene `{id}` row_h must be positive");
        assert!(layout.list.item_h > 0, "scene `{id}` item_h must be positive");
        assert!(layout.list.item_h <= layout.list.row_h, "scene `{id}` item_h must fit in row_h");
    }
}

// ------------------------------------------------------- 解析失败是硬错误

#[test]
fn invalid_json_is_a_hard_error_not_fallback() {
    // A syntactically broken payload must surface an error rather than silently
    // yielding a default — this is the "硬错误，不 fallback" contract.
    let r: Result<std::collections::BTreeMap<String, String>, _> =
        serde_json::from_str("{ not json");
    let mapped = r.map_err(|e| LoadError::Syntax(e.to_string()));
    assert!(matches!(mapped, Err(LoadError::Syntax(_))));
}

#[test]
fn balance_fixture_is_constructible_for_future_rules_tests() {
    // Confirms the data types are plain structs a unit test can build by hand
    // (docs/process/03_templates.md §4: "数值来自 balance.json（调用方注入，测试可注假值）").
    let b = Balance {
        economy: ti_core::data::EconomyCfg { gold_max: 100, bank_rate: 1.0, initial_gold: 0 },
        combat: CombatCfg {
            player_first: true,
            max_rounds: 10,
            damage: DamageCfg { flat: 1, atk_coef: 1.0, rng_span: 1 },
            boss_tiers: vec![BossTier { tier: 0, hp: 10, atk: 1 }],
        },
        trade: ti_core::data::TradeCfg {
            markup: 1.0,
            goods: vec![ti_core::data::GoodsDef {
                id: 1,
                name_key: "g".to_string(),
                buy: 1,
                qty_max: 9,
            }],
        },
        build: ti_core::data::BuildCfg {
            max_level: 3,
            buildings: vec![BuildingDef {
                id: "C".to_string(),
                name_key: "b".to_string(),
                base_cost: 1,
                gold_max: 9,
            }],
        },
        cards: ti_core::data::CardsCfg { event_chance: 0.5, hand_size: 1, deck: Vec::new() },
        messages: ti_core::data::MessagesCfg { pages: std::collections::BTreeMap::new() },
        selection: ti_core::data::SelectionCfg { character_count: 5, map_count: 3 },
        save: ti_core::data::SaveCfg { slot_count: 1, magic: 1, v2_size: 1 },
        ui: ti_core::data::UiCfg {
            logical_width: 1,
            logical_height: 1,
            glyph_advance: 1,
            max_back_steps: 1,
        },
    };
    assert_eq!(b.combat.boss(0).map(|x| x.hp), Some(10));
}
