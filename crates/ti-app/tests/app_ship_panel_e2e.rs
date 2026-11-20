#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S63 T3 造船面板 e2e：N 键开 → 展示当前船级/货舱/下一级/造价 → Enter 升级（走 `rules::boat`
//! `boat_upgrade`：扣金 + 船级 +1）→ 满级自动关 / Back 关。判据 = ① 开关 ② 升级扣金船级+1 ③ 面板互斥。

use ti_app::{EconomyHost, GameApp, GameInput, Scene};

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
fn ship_panel_open_and_mutual_exclusion() {
    let mut app = enter_main_map();
    assert!(!app.ship_panel_open);
    app.handle(GameInput::OpenShipPanel);
    assert!(app.ship_panel_open, "N 键开造船面板");
    app.handle(GameInput::OpenMerchantPanel);
    assert!(app.merchant_panel_open && !app.ship_panel_open, "面板互斥");
    app.handle(GameInput::OpenShipPanel);
    app.handle(GameInput::Back);
    assert!(!app.ship_panel_open, "Back 关");
}

#[test]
fn ship_panel_upgrade_via_rules() {
    let mut app = enter_main_map();
    let level0 = app.boat_level();
    let bank0 = app.bank_gold();
    app.handle(GameInput::OpenShipPanel);
    app.handle(GameInput::Confirm); // 升级
    assert_eq!(app.boat_level(), level0 + 1, "船级 +1（rules::boat upgrade）");
    assert!(app.bank_gold() < bank0, "升级扣银行金（rules::boat 走 bank_gold，800）");
}
