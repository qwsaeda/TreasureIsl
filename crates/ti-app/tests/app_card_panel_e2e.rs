#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S68 卡牌图鉴面板 e2e：C 键开面板 → 展示卡池（名称/效果/权重）→ ↑↓ 翻页 → Back 关。
//! 判据 = ① C 键开 ② 卡牌数/名称/效果与 `balance.cards` 一致 ③ ↑↓ 翻页钳位
//! ④ 与其余面板互斥 ⑤ C 键经 `poll()` 真路由。纯读数据不改规则。

use ti_app::{GameApp, GameInput, KeyState, MapHost, Scene};

fn keys() -> KeyState {
    KeyState {
        up: false,
        down: false,
        left: false,
        right: false,
        confirm: false,
        back: false,
        landmark: false,
        spin: false,
        buy: false,
        sell: false,
        build: false,
        merchant: false,
        event: false,
        ship: false,
        garrison: false,
        upgrade: false,
        dungeon: false,
        card: false,
        achievement: false, // S72：成就面板（P 键）
    }
}

fn enter_main_map() -> GameApp {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.set_rng_seed(42);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap);
    app
}

#[test]
fn card_panel_open_and_data() {
    let mut app = enter_main_map();
    assert!(!app.card_panel_open);
    app.handle(GameInput::OpenCardPanel);
    assert!(app.card_panel_open, "C 键开卡牌图鉴面板");
    assert_eq!(app.card_count(), 5, "卡池 5 张（balance.cards.deck）");
    assert_eq!(app.card_sel(), 0, "打开时翻页归零");
    assert!(!app.card_name(0).is_empty(), "卡名非空（cards.1）");
    assert!(!app.card_effect_text(0).is_empty(), "效果描述非空");
}

#[test]
fn card_panel_paging_clamped() {
    let mut app = enter_main_map();
    app.handle(GameInput::OpenCardPanel);
    // ↓ 多次钳到末张（card_count-1 = 4）。
    for _ in 0..10 {
        app.handle(GameInput::MoveDown);
    }
    assert_eq!(app.card_sel(), 4, "↓ 多次钳到末张 4");
    // ↑ 多次钳回首张 0。
    for _ in 0..10 {
        app.handle(GameInput::MoveUp);
    }
    assert_eq!(app.card_sel(), 0, "↑ 多次钳回首张 0");
}

#[test]
fn card_panel_back_closes() {
    let mut app = enter_main_map();
    app.handle(GameInput::OpenCardPanel);
    app.handle(GameInput::Back);
    assert!(!app.card_panel_open, "Back 关面板");
    assert_eq!(app.scene(), Scene::MainMap, "Back 仅关面板不退场景");
}

#[test]
fn card_panel_mutual_exclusion() {
    let mut app = enter_main_map();
    app.handle(GameInput::OpenCardPanel);
    assert!(app.card_panel_open);
    app.handle(GameInput::OpenShipPanel);
    assert!(app.ship_panel_open && !app.card_panel_open, "开造船关图鉴（互斥）");
    app.handle(GameInput::OpenCardPanel);
    assert!(app.card_panel_open && !app.ship_panel_open, "重开图鉴关造船");
    app.handle(GameInput::Back);
    assert!(!app.card_panel_open);
}

/// C 键经 `poll()` 真路由（R22-010 同型覆盖）。
#[test]
fn card_panel_via_poll_keyboard() {
    let mut app = enter_main_map();
    let mut k = keys();
    k.card = true;
    app.poll(&k);
    assert!(app.card_panel_open, "poll C 键应开卡牌图鉴面板");
    let mut k = keys();
    k.down = true;
    app.poll(&k);
    assert_eq!(app.card_sel(), 1, "poll ↓ 应翻到第 2 张");
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.card_panel_open, "poll Back 应关面板");
    assert_eq!(app.scene(), Scene::MainMap);
}
