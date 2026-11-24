#![allow(clippy::unwrap_used, clippy::expect_used, clippy::struct_field_names)]
//! R22-010 回归：面板必须经**真实 `poll()` 键盘路径**可交互（不是只靠 `handle()` 直接分发）。
//! 5 个面板 e2e 此前均绕过 `poll()`，导致 `input.rs` `MainMap` 分支漏路由 ↑↓/Confirm/Back 长期漏网。
//! 本测试构造 `KeyState` 走 `app.poll()`，断言：开面板 → ↑↓ 改选中 → Enter 执行 → Back 仅关面板（不退出 `MainMap`）。
//! R25 F-16 口径说明：5 条 `*_via_poll_keyboard` 覆盖 poll 路由；`all_five_panels_pairwise_mutual_exclusion`
//! 专验 `handle()` 层两两互斥（R22-011 遗留），两者分工不同、不重复。

use ti_app::{GameApp, GameInput, KeyState, Scene};

/// 面板 opener 与状态 getter（R22-022 参数化互斥测试用）。
type Opener = (GameInput, fn(&GameApp) -> bool);

/// 空键位快照（所有键 false）。
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
        achievement: false, // S72：成就面板（P 键） // S68：卡牌图鉴（C 键） // S66：副本情报面板（J 键） // S65：领地升级面板（H 键） // S64：驻军自定面板（G 键）
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

/// 贸易面板：T 开 → ↓ 改选中 → Enter 买 → Back 关面板（仍在 `MainMap`）。
#[test]
fn trade_panel_via_poll_keyboard() {
    let mut app = enter_main_map();
    // T 键开面板（poll）
    let mut k = keys();
    k.buy = true;
    app.poll(&k);
    assert!(app.trade_panel_open, "poll T 键应开贸易面板（R22-010）");
    // ↓ 改选中（poll）
    let mut k = keys();
    k.down = true;
    app.poll(&k);
    assert_eq!(app.trade_panel_sel, 1, "poll ↓ 应改贸易选中（R22-010）");
    // Enter 买（poll）
    let mut k = keys();
    k.confirm = true;
    app.poll(&k);
    // Back 仅关面板，不退出 MainMap（poll）
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.trade_panel_open, "poll Back 应关贸易面板（R22-010）");
    assert_eq!(
        app.scene(),
        Scene::MainMap,
        "Back 关面板后仍在 MainMap（不被发成 MapBack 退出，R22-010）"
    );
}

/// 事件面板：V 开 → ↑↓ 改选中 → Back 关（poll 路径）。
#[test]
fn event_panel_via_poll_keyboard() {
    let mut app = enter_main_map();
    let mut k = keys();
    k.event = true;
    app.poll(&k);
    assert!(app.event_panel_open, "poll V 键应开事件面板");
    let mut k = keys();
    k.down = true;
    app.poll(&k);
    assert_eq!(app.event_panel_sel, 1, "poll ↓ 应改事件选中");
    let mut k = keys();
    k.up = true;
    app.poll(&k);
    assert_eq!(app.event_panel_sel, 0, "poll ↑ 应回事件首项");
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.event_panel_open, "poll Back 应关事件面板");
    assert_eq!(app.scene(), Scene::MainMap);
}

/// 商人面板：M 开 → ↓ 改选中 → Enter 买 → Back 关（R24 FIND-007 补 poll 覆盖）。
#[test]
fn merchant_panel_via_poll_keyboard() {
    let mut app = enter_main_map();
    let mut k = keys();
    k.merchant = true;
    app.poll(&k);
    assert!(app.merchant_panel_open, "poll M 键应开商人面板");
    let mut k = keys();
    k.down = true;
    app.poll(&k);
    assert_eq!(app.merchant_panel_sel, 1, "poll ↓ 应改商人选中");
    // R25 F-14：Enter 须真消费——买到武器（sel=1）：arms +1 / gold -50（不是只断言面板仍开）。
    let seat = app.turn_cycle.current as usize;
    let arms0 = app.seat_arms[seat];
    let gold0 = app.seat_gold[seat];
    let mut k = keys();
    k.confirm = true;
    app.poll(&k);
    assert!(app.merchant_panel_open, "poll Enter 买后仍在商人面板");
    assert_eq!(app.seat_arms[seat], arms0 + 1, "poll Enter 真买武器：arms +1（R25 F-14 去弱断言）");
    assert_eq!(app.seat_gold[seat], gold0 - 50, "poll Enter 真扣金：gold -50");
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.merchant_panel_open, "poll Back 应关商人面板");
    assert_eq!(app.scene(), Scene::MainMap);
}

/// 建造面板：B 开 → ↓ 改选中 → Enter 建 → Back 关（R24 FIND-007 补 poll 覆盖）。
#[test]
fn build_panel_via_poll_keyboard() {
    let mut app = enter_main_map();
    let mut k = keys();
    k.build = true;
    app.poll(&k);
    assert!(app.build_panel_open, "poll B 键应开建造面板");
    let mut k = keys();
    k.down = true;
    app.poll(&k);
    assert_eq!(app.build_panel_sel, 1, "poll ↓ 应改建造选中");
    // Back 路由单独验证（Enter 建造成功后 confirm_build_panel 会自动关面板，不能重复 Back）。
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.build_panel_open, "poll Back 应关建造面板");
    assert_eq!(app.scene(), Scene::MainMap, "poll Back 关面板后仍在 MainMap");
    // Enter 路由单独验证：重开 → ↓ → Enter 建造成功后面板自动关。
    let mut k = keys();
    k.build = true;
    app.poll(&k);
    let mut k = keys();
    k.down = true;
    app.poll(&k);
    let mut k = keys();
    k.confirm = true;
    app.poll(&k);
    assert!(!app.build_panel_open, "poll Enter 建造成功后自动关面板");
    assert_eq!(app.scene(), Scene::MainMap);
}

/// 造船面板：N 开 → Enter 升级 → Back 关（R24 FIND-007 补 poll 覆盖；船无 sel 只有升级）。
#[test]
fn ship_panel_via_poll_keyboard() {
    let mut app = enter_main_map();
    let mut k = keys();
    k.ship = true;
    app.poll(&k);
    assert!(app.ship_panel_open, "poll N 键应开造船面板");
    // R25 F-14：Enter 须真消费——船级 +1（不是只断言面板仍开）。
    let level0 = app.boat.level();
    let mut k = keys();
    k.confirm = true;
    app.poll(&k);
    assert!(app.ship_panel_open, "poll Enter 升级后仍在造船面板；船无 sel 字段");
    assert_eq!(
        app.boat.level(),
        level0 + 1,
        "poll Enter 真升级：boat level +1（R25 F-14 去弱断言）"
    );
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.ship_panel_open, "poll Back 应关造船面板");
    assert_eq!(app.scene(), Scene::MainMap);
}

/// R22-011/R22-022：5 面板两两互斥全覆盖（含漏网的 event→trade）。
#[test]
fn all_five_panels_pairwise_mutual_exclusion() {
    let openers: [Opener; 5] = [
        (GameInput::OpenTradePanel, |a| a.trade_panel_open),
        (GameInput::OpenBuildPanel, |a| a.build_panel_open),
        (GameInput::OpenMerchantPanel, |a| a.merchant_panel_open),
        (GameInput::OpenEventPanel, |a| a.event_panel_open),
        (GameInput::OpenShipPanel, |a| a.ship_panel_open),
    ];
    for (i, (open_i, get_i)) in openers.iter().enumerate() {
        for (j, (open_j, get_j)) in openers.iter().enumerate() {
            if i == j {
                continue;
            }
            let mut app = enter_main_map();
            app.handle(*open_i);
            assert!(get_i(&app), "开面板 {i} 应生效");
            app.handle(*open_j);
            assert!(!get_i(&app), "开面板 {j} 应关面板 {i}（R22-011 互斥全覆盖）");
            assert!(get_j(&app), "开面板 {j} 应生效");
        }
    }
}

/// 面板态 Space 不摇骰（R22-010：spin 在面板打开时应被吞）。
#[test]
fn panel_swallows_spin_via_poll() {
    let mut app = enter_main_map();
    let mut k = keys();
    k.ship = true;
    app.poll(&k);
    assert!(app.ship_panel_open);
    let pos_before = app.board.as_ref().map(ti_core::rules::board::BoardSession::position_index);
    let mut k = keys();
    k.spin = true;
    app.poll(&k);
    let pos_after = app.board.as_ref().map(ti_core::rules::board::BoardSession::position_index);
    assert_eq!(pos_before, pos_after, "面板态 Space 不应摇骰改位置（R22-010）");
}
