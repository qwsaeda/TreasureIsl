#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S65 领地升级面板 e2e：落自己格（`pending_own`）→ H 键开面板 → 展示等级/造价/产出
//! → Confirm 走 `confirm_own`（金足升级、金不足退化领产出）/ Back 走 `abandon_own`（领产出）。
//! 判据 = ① 仅 `pending_own` 可开 ② 只读查询与 rules 一致 ③ H 键经 `poll()` 真路由
//! ④ Confirm 升级金守恒 / 退化领产出 ⑤ Back 领产出 ⑥ 与驻军+5 面板互斥。

use ti_app::{GameApp, GameInput, KeyState, MapHost, Scene};
use ti_core::rules::territory::Ownership;

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
    }
}

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

/// 造一个自己格待决场景：在已有 territory 账本取第一个领地格，设等级/驻军，给足金币。
fn enter_with_own_cell(gold: u32) -> GameApp {
    let mut app = enter_main_map();
    let seat = app.turn_cycle.current;
    let cell = (0..40u32)
        .find(|&i| app.territory.as_ref().is_some_and(|t| t.ownership(i).is_some()))
        .expect("测试地图须有领地格");
    app.territory
        .as_mut()
        .unwrap()
        .mutate(cell, Ownership { owner: Some(seat), level: 1, garrison: 0 });
    app.seat_gold[seat as usize] = gold;
    app.pending_own = Some(cell);
    app
}

#[test]
fn upgrade_panel_open_only_with_pending_own() {
    let mut app = enter_main_map();
    app.handle(GameInput::OpenUpgradePanel);
    assert!(!app.upgrade_panel_open, "无自己格待决时 H 键不应开升级面板");
    let mut app = enter_with_own_cell(50_000);
    app.handle(GameInput::OpenUpgradePanel);
    assert!(app.upgrade_panel_open, "落自己格后 H 键开升级面板");
}

#[test]
fn upgrade_panel_queries_match_rules() {
    let mut app = enter_with_own_cell(50_000);
    app.handle(GameInput::OpenUpgradePanel);
    assert_eq!(app.own_level(), 1, "当前等级 = 1");
    assert!(app.own_upgrade_cost() > 0, "升级造价 > 0（与 balance 一致）");
    assert!(app.own_production() > 0, "本圈产出 > 0（与 balance 一致）");
    assert!(!app.own_isle_name().is_empty(), "岛名文案非空");
}

#[test]
fn upgrade_panel_confirm_upgrades_and_conserves_gold() {
    let mut app = enter_with_own_cell(50_000);
    let cell = app.pending_own.unwrap();
    let seat = app.turn_cycle.current;
    let cost = app.own_upgrade_cost();
    let gold0 = app.seat_gold[seat as usize];
    app.handle(GameInput::OpenUpgradePanel);
    app.handle(GameInput::Confirm);
    assert!(!app.upgrade_panel_open, "Confirm 后面板关闭");
    let terr = app.territory.as_ref().unwrap();
    assert_eq!(terr.ownership(cell).map_or(0, |o| o.level), 2, "Confirm 金足 ⇒ 等级 1→2");
    assert_eq!(app.seat_gold[seat as usize], gold0 - cost, "金守恒：扣升级造价");
}

#[test]
fn upgrade_panel_back_collects_production() {
    let mut app = enter_with_own_cell(100);
    let seat = app.turn_cycle.current;
    let prod = app.own_production();
    let gold0 = app.seat_gold[seat as usize];
    app.handle(GameInput::OpenUpgradePanel);
    app.handle(GameInput::Back);
    assert!(!app.upgrade_panel_open, "Back 后面板关闭");
    assert_eq!(app.seat_gold[seat as usize], gold0 + prod, "Back ⇒ 领产出入账（金守恒）");
}

#[test]
fn upgrade_panel_insufficient_gold_degrades_to_collect() {
    let mut app = enter_with_own_cell(0); // 金不足
    let seat = app.turn_cycle.current;
    let prod = app.own_production();
    app.handle(GameInput::OpenUpgradePanel);
    app.handle(GameInput::Confirm);
    assert_eq!(app.seat_gold[seat as usize], prod, "金不足 Confirm ⇒ 退化领产出");
}

#[test]
fn upgrade_panel_mutual_exclusion() {
    let mut app = enter_with_own_cell(50_000);
    app.handle(GameInput::OpenUpgradePanel);
    assert!(app.upgrade_panel_open);
    app.handle(GameInput::OpenGarrisonPanel);
    assert!(app.garrison_panel_open && !app.upgrade_panel_open, "开驻军关升级（互斥）");
    app.handle(GameInput::OpenUpgradePanel);
    assert!(app.upgrade_panel_open && !app.garrison_panel_open, "重开升级关驻军");
    app.handle(GameInput::Back);
    assert!(!app.upgrade_panel_open, "Back 关升级面板");
    assert_eq!(app.scene(), Scene::MainMap, "Back 仅关面板不退场景");
}

/// H 键经 `poll()` 真路由（R22-010 同型覆盖）。
#[test]
fn upgrade_panel_via_poll_keyboard() {
    let mut app = enter_with_own_cell(50_000);
    let mut k = keys();
    k.upgrade = true;
    app.poll(&k);
    assert!(app.upgrade_panel_open, "poll H 键应开升级面板");
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.upgrade_panel_open, "poll Back 应关升级面板");
    assert_eq!(app.scene(), Scene::MainMap);
}
