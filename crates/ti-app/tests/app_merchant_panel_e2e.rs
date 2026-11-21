#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S63 T7 商人面板 e2e：M 键开 → ↑↓选买人手/买武器 → Enter 买（走 `rules::merchant` 写回
//! `seat_crew`/`seat_arms`/gold）→ Back 关。判据 = ① 开关+选择 ② 买人手：人手+1、金-30、不超船容量
//! ③ 买武器：武器+1、金-50、不超人手 ④ 与贸易/建造面板互斥。

use ti_app::{GameApp, GameInput, Scene};

fn enter_main_map() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(42);
    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Confirm); // → SelectMap（旧单选流）
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap);
    app
}

#[test]
fn merchant_panel_open_select_and_mutual_exclusion() {
    let mut app = enter_main_map();
    assert!(!app.merchant_panel_open);
    app.handle(GameInput::OpenMerchantPanel);
    assert!(app.merchant_panel_open, "M 键开商人面板");
    assert_eq!(app.merchant_panel_sel, 0, "初始选买人手");
    app.handle(GameInput::MoveDown);
    assert_eq!(app.merchant_panel_sel, 1, "↓ 选买武器");
    app.handle(GameInput::MoveUp);
    assert_eq!(app.merchant_panel_sel, 0, "↑ 回买人手");
    // 互斥：开贸易面板自动关商人
    app.handle(GameInput::OpenTradePanel);
    assert!(app.trade_panel_open && !app.merchant_panel_open, "面板互斥");
    app.handle(GameInput::OpenMerchantPanel);
    assert!(app.merchant_panel_open && !app.trade_panel_open);
    app.handle(GameInput::Back);
    assert!(!app.merchant_panel_open, "Back 关");
}

#[test]
fn merchant_buy_crew_via_rules() {
    let mut app = enter_main_map();
    let seat = app.turn_cycle.current as usize;
    let crew0 = app.seat_crew[seat];
    let gold0 = app.seat_gold[seat];
    app.handle(GameInput::OpenMerchantPanel);
    app.handle(GameInput::Confirm); // 买人手（sel=0）
    assert_eq!(app.seat_crew[seat], crew0 + 1, "人手 +1（rules::buy_crew）");
    assert_eq!(app.seat_gold[seat], gold0 - 30, "金 -30（slave_price）");
}

/// R22-024：金不足时买人被拒（状态不变，走 `Sfx::Invalid`）。
#[test]
fn merchant_buy_rejected_when_insufficient_gold() {
    let mut app = enter_main_map();
    let seat = app.turn_cycle.current as usize;
    // 压金到 0（< slave_price 30）
    app.seat_gold[seat] = 0;
    let crew0 = app.seat_crew[seat];
    app.handle(GameInput::OpenMerchantPanel);
    app.handle(GameInput::Confirm); // 买人手（应被拒）
    assert_eq!(app.seat_crew[seat], crew0, "金不足：人手不变（buy_crew Err）");
    assert_eq!(app.seat_gold[seat], 0, "金不足：金不变");
}

#[test]
fn merchant_buy_arms_via_rules() {
    let mut app = enter_main_map();
    let seat = app.turn_cycle.current as usize;
    let arms0 = app.seat_arms[seat];
    let gold0 = app.seat_gold[seat];
    app.handle(GameInput::OpenMerchantPanel);
    app.handle(GameInput::MoveDown); // 选买武器
    app.handle(GameInput::Confirm);
    assert_eq!(app.seat_arms[seat], arms0 + 1, "武器 +1（rules::buy_arms）");
    assert_eq!(app.seat_gold[seat], gold0 - 50, "金 -50（weapon_price）");
}
