#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S63 T6 事件面板 e2e：V 键开 → ↑↓选 7 种事件 → Enter 应用（`rules::events::effect` 算效果 +
//! `apply_event_effect` 写回 `seat_gold`/`crew`/`arms`）→ Back 关。判据 = ① 开关+选择环形 ② 宝藏沉船
//! (index2) 应用：金 +200、武器 +3 ③ 流浪儿(index5) 应用：人手 +4 ④ 面板互斥。

use ti_app::{GameApp, GameInput, Scene};

fn enter_main_map() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(42);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap);
    app
}

#[test]
fn event_panel_open_and_wrap_selection() {
    let mut app = enter_main_map();
    assert!(!app.event_panel_open);
    app.handle(GameInput::OpenEventPanel);
    assert!(app.event_panel_open, "V 键开事件面板");
    assert_eq!(app.event_panel_sel, 0);
    app.handle(GameInput::MoveUp);
    assert_eq!(app.event_panel_sel, 6, "↑ 从 0 环绕到末项 6");
    app.handle(GameInput::MoveDown);
    assert_eq!(app.event_panel_sel, 0, "↓ 回 0");
    app.handle(GameInput::OpenMerchantPanel);
    assert!(app.merchant_panel_open && !app.event_panel_open, "面板互斥");
    app.handle(GameInput::OpenEventPanel);
    app.handle(GameInput::Back);
    assert!(!app.event_panel_open, "Back 关");
}

#[test]
fn event_treasure_wreck_applies_gold_and_arms() {
    let mut app = enter_main_map();
    let seat = app.turn_cycle.current as usize;
    let gold0 = app.seat_gold[seat];
    let arms0 = app.seat_arms[seat];
    app.handle(GameInput::OpenEventPanel);
    app.handle(GameInput::MoveDown); // 0→1
    app.handle(GameInput::MoveDown); // 1→2 (宝藏沉船)
    assert_eq!(app.event_panel_sel, 2);
    app.handle(GameInput::Confirm);
    assert!(app.seat_gold[seat] > gold0, "宝藏沉船：金 +200");
    assert!(app.seat_arms[seat] > arms0, "宝藏沉船：武器 +3");
}

#[test]
fn event_urchin_applies_crew() {
    let mut app = enter_main_map();
    let seat = app.turn_cycle.current as usize;
    let crew0 = app.seat_crew[seat];
    app.handle(GameInput::OpenEventPanel);
    for _ in 0..5 {
        app.handle(GameInput::MoveDown); // 0→5 (流浪儿)
    }
    assert_eq!(app.event_panel_sel, 5);
    app.handle(GameInput::Confirm);
    assert_eq!(app.seat_crew[seat], crew0 + 4, "流浪儿：人手 +4");
}
