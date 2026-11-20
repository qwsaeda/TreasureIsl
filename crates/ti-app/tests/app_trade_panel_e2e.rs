#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S63 T1 · 贸易面板 e2e：T 键开面板 → ↑↓选商品 → Enter 买 / Y 卖 → Back 关，
//! 走真实 `GameInput` 分发 + `rules::TradeLedger` 账本（面板 = 入口，结算 = rules）。
//!
//! 判据：① 面板开关状态机（OpenTradePanel 开 / Back 关 + 买入后保持打开）
//! ② ↑↓ 选中域钳制（0..=4 环形）③ 买扣金 + 持有 +1（与 `TradeLedger` 一致）
//! ④ 卖加金 + 持有清空 ⑤ 面板打开时吞掉 Spin（不会误摇骰）。

use ti_app::{GameApp, GameInput, Scene};

fn enter_main_map() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(42);
    app.set_rng_seed(42);
    app.handle(GameInput::Confirm); // Index → SelectRole
    app.handle(GameInput::Confirm); // 无 marks = 旧单选流 → SelectMap
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap);
    app
}

#[test]
fn trade_panel_open_close_and_selection_clamp() {
    let mut app = enter_main_map();
    assert!(!app.trade_panel_open, "初始关闭");
    app.handle(GameInput::OpenTradePanel);
    assert!(app.trade_panel_open, "T 键打开面板");
    assert_eq!(app.trade_panel_sel, 0, "初始选中商品 0");
    // ↑ 在 0 → 环绕到末项 4
    app.handle(GameInput::MoveUp);
    assert_eq!(app.trade_panel_sel, 4, "↑ 环绕到末项");
    // ↓ 在 4 → 回 0
    app.handle(GameInput::MoveDown);
    assert_eq!(app.trade_panel_sel, 0, "↓ 回首项");
    // ↓ 到 1
    app.handle(GameInput::MoveDown);
    assert_eq!(app.trade_panel_sel, 1);
    app.handle(GameInput::Back);
    assert!(!app.trade_panel_open, "Back 关闭面板");
}

#[test]
fn trade_panel_buy_and_sell_via_rules() {
    let mut app = enter_main_map();
    app.handle(GameInput::OpenTradePanel);
    let gold_before = app.gold();
    // 选商品 1（gid=1，香料）+ 买入
    app.handle(GameInput::MoveDown); // sel 0 → 1
    app.handle(GameInput::Confirm); // 买 gid=2
    assert!(app.trade_panel_open, "买入后面板保持打开");
    assert!(app.gold() < gold_before, "买入扣金（rules::TradeLedger）");
    let owned_after_buy = app.owned.iter().find(|(g, _)| *g == 2).map_or(0, |(_, q)| *q);
    assert!(owned_after_buy > 0, "持有 +1（rules 账本）");
    // 卖同商品
    app.handle(GameInput::TradeSell(2));
    let owned_after_sell = app.owned.iter().find(|(g, _)| *g == 2).map_or(0, |(_, q)| *q);
    assert_eq!(owned_after_sell, 0, "卖清空持有");
    assert!(
        app.gold() >= gold_before.saturating_sub(app.gold().saturating_sub(gold_before)),
        "卖出有金入账"
    );
}

#[test]
fn trade_panel_swallows_spin_while_open() {
    let mut app = enter_main_map();
    app.handle(GameInput::OpenTradePanel);
    let pos_before = app.board.as_ref().map(ti_core::rules::board::BoardSession::position_index);
    let roll_before = app.board.as_ref().map(ti_core::rules::board::BoardSession::last_roll);
    app.handle(GameInput::Spin); // 面板打开时应被吞掉
    let pos_after = app.board.as_ref().map(ti_core::rules::board::BoardSession::position_index);
    assert_eq!(pos_before, pos_after, "面板打开时 Spin 不改位置（不误摇骰）");
    let _ = roll_before;
}

#[test]
fn build_panel_entry_restored() {
    let mut app = enter_main_map();
    assert!(!app.build_panel_open, "初始关闭");
    app.handle(GameInput::OpenBuildPanel);
    assert!(app.build_panel_open, "B 键打开建造面板（S34 删入口后由 S63 恢复）");
    assert!(!app.trade_panel_open, "建造与贸易面板互斥");
    app.handle(GameInput::Back);
    assert!(!app.build_panel_open);
}
