#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S64 驻军自定面板 e2e：落自己格（`pending_own`）→ G 键开面板 → ↑ 加驻军 / ↓ 减驻军
//! → Enter 确认走 `rules::territory_v2::set_garrison`（人手守恒钳位）→ Back 关。
//! 判据 = ① 仅 `pending_own` 在位可开 ② 驻军±N ⇒ 舰队人手∓N（守恒）
//! ③ 钳到 [0, 驻军+舰队] ④ 非所有者不变 ⑤ 5 面板互斥 ⑥ G 键经 `poll()` 真路由。

use ti_app::{GameApp, GameInput, KeyState, Scene};
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
        dungeon: false,
        card: false, // S68：卡牌图鉴（C 键） // S66：副本情报面板（J 键） // S65：领地升级面板（H 键）
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

/// 造一个自己格待决场景：在已有 territory 账本里取第一个领地格归席位，设驻军/舰队人手。
fn enter_with_own_cell(garrison: u32, crew: u32) -> GameApp {
    let mut app = enter_main_map();
    let seat = app.turn_cycle.current;
    // 领地格 index 从 board.json 来（territory 格），用 ownership() 探测第一个在账本里的格。
    let cell = (0..40u32)
        .find(|&i| app.territory.as_ref().is_some_and(|t| t.ownership(i).is_some()))
        .expect("测试地图须有领地格");
    let terr = app.territory.as_mut().unwrap();
    terr.mutate(cell, Ownership { owner: Some(seat), level: 1, garrison });
    app.seat_crew[seat as usize] = crew;
    app.pending_own = Some(cell);
    app
}

#[test]
fn garrison_panel_open_only_with_pending_own() {
    let mut app = enter_main_map();
    // 无 pending_own：G 键不打开面板。
    app.handle(GameInput::OpenGarrisonPanel);
    assert!(!app.garrison_panel_open, "无自己格待决时 G 键不应开驻军面板");
    // 有 pending_own：打开。
    let mut app = enter_with_own_cell(2, 10);
    app.handle(GameInput::OpenGarrisonPanel);
    assert!(app.garrison_panel_open, "落自己格后 G 键开驻军面板");
}

#[test]
fn garrison_increase_conserves_crew() {
    let mut app = enter_with_own_cell(2, 10);
    let cell = app.pending_own.unwrap();
    app.handle(GameInput::OpenGarrisonPanel);
    let seat = app.turn_cycle.current;
    let crew0 = app.seat_crew[seat as usize];
    // 目标设为 5（garrison_sel 初始 = 当前驻军 2，↑×3 ⇒ 5）。
    app.handle(GameInput::MoveUp);
    app.handle(GameInput::MoveUp);
    app.handle(GameInput::MoveUp);
    assert_eq!(app.garrison_sel, 5, "↑×3 目标驻军 2→5");
    app.handle(GameInput::Confirm);
    assert!(!app.garrison_panel_open, "Enter 确认后面板关闭");
    let terr = app.territory.as_ref().unwrap();
    assert_eq!(terr.ownership(cell).map_or(0, |o| o.garrison), 5, "驻军 2→5");
    assert_eq!(app.seat_crew[seat as usize], crew0 - 3, "守恒：舰队人手 10→7（分出 3）");
}

#[test]
fn garrison_decrease_conserves_crew() {
    let mut app = enter_with_own_cell(5, 4);
    let cell = app.pending_own.unwrap();
    app.handle(GameInput::OpenGarrisonPanel);
    let seat = app.turn_cycle.current;
    let crew0 = app.seat_crew[seat as usize];
    app.handle(GameInput::MoveDown);
    app.handle(GameInput::MoveDown);
    assert_eq!(app.garrison_sel, 3, "↓×2 目标驻军 5→3");
    app.handle(GameInput::Confirm);
    let terr = app.territory.as_ref().unwrap();
    assert_eq!(terr.ownership(cell).map_or(0, |o| o.garrison), 3, "驻军 5→3");
    assert_eq!(app.seat_crew[seat as usize], crew0 + 2, "守恒：舰队人手 4→6（调回 2）");
}

#[test]
fn garrison_clamped_to_available() {
    let mut app = enter_with_own_cell(2, 3); // 可用上限 = 2+3 = 5
    let cell = app.pending_own.unwrap();
    app.handle(GameInput::OpenGarrisonPanel);
    for _ in 0..20 {
        app.handle(GameInput::MoveUp);
    }
    assert_eq!(app.garrison_sel, 5, "↑ 多次钳到可用上限 5（驻军2+舰队3）");
    app.handle(GameInput::Confirm);
    let seat = app.turn_cycle.current;
    let terr = app.territory.as_ref().unwrap();
    assert_eq!(terr.ownership(cell).map_or(0, |o| o.garrison), 5, "驻军钳到 5");
    assert_eq!(app.seat_crew[seat as usize], 0, "舰队人手全部调出 = 0");
}

#[test]
fn garrison_panel_mutual_exclusion() {
    let mut app = enter_with_own_cell(2, 10);
    let _cell = app.pending_own.unwrap();
    app.handle(GameInput::OpenGarrisonPanel);
    assert!(app.garrison_panel_open);
    app.handle(GameInput::OpenShipPanel);
    assert!(app.ship_panel_open && !app.garrison_panel_open, "开造船关驻军（互斥）");
    app.handle(GameInput::OpenGarrisonPanel);
    assert!(app.garrison_panel_open && !app.ship_panel_open, "重开驻军关造船");
    app.handle(GameInput::Back);
    assert!(!app.garrison_panel_open, "Back 关驻军面板");
    assert_eq!(app.scene(), Scene::MainMap, "Back 仅关面板不退场景");
}

/// G 键经 `poll()` 真路由（R22-010 同型覆盖）：keymap `garrison` ⇒ `OpenGarrisonPanel`。
#[test]
fn garrison_panel_via_poll_keyboard() {
    let mut app = enter_with_own_cell(2, 10);
    let cell = app.pending_own.unwrap();
    let mut k = keys();
    k.garrison = true;
    app.poll(&k);
    assert!(app.garrison_panel_open, "poll G 键应开驻军面板");
    let mut k = keys();
    k.up = true;
    app.poll(&k);
    assert_eq!(app.garrison_sel, 3, "poll ↑ 应加驻军目标 2→3");
    let mut k = keys();
    k.confirm = true;
    app.poll(&k);
    let terr = app.territory.as_ref().unwrap();
    assert_eq!(terr.ownership(cell).map_or(0, |o| o.garrison), 3, "poll Enter 真设驻军 3");
}
