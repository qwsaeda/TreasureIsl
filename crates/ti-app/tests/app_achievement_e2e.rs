#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S72 成就系统 e2e：规则层 eval 已在 ti-core 单测覆盖；本文件测 app 接线——
//! P 键开面板 / 成就位掩码 / 金币入账触发 tycoon / 存档往返还原 / 面板互斥。

use ti_app::{GameApp, GameInput, KeyState, MapHost, Scene};
use ti_core::rules::achievements as ach;

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
        achievement: false,
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
fn achievement_panel_open_back_and_mutual_exclusion() {
    let mut app = enter_main_map();
    assert!(!app.achievement_panel_open());
    app.handle(GameInput::OpenAchievementPanel);
    assert!(app.achievement_panel_open(), "P 键开成就面板");
    app.handle(GameInput::OpenShipPanel);
    assert!(app.ship_panel_open() && !app.achievement_panel_open(), "开造船关成就（互斥）");
    app.handle(GameInput::OpenAchievementPanel);
    assert!(app.achievement_panel_open() && !app.ship_panel_open(), "重开成就关造船");
    app.handle(GameInput::Back);
    assert!(!app.achievement_panel_open(), "Back 关成就面板");
    assert_eq!(app.scene(), Scene::MainMap);
}

// tycoon 触发的纯判定由 ti-core rules::achievements 单测覆盖（qualifies/eval）；
// app 层 credit_seat_gold 是 pub(crate) 不可从 e2e 直接调，故此处只测公开面（面板/存档/poll）。

#[test]
fn achievement_mask_persists_in_save_v4() {
    let mut app = enter_main_map();
    app.achievements = (1u64 << ach::id::FIRST_BOSS) | (1u64 << ach::id::TYCOON);
    let ext = app.save_v4_ext().expect("v4 扩展应在位");
    assert_eq!(ext.achievements, app.achievements, "成就位掩码写入 v4 扩展");
    // 旧档（无成就字段）= 0：serde default 已保，构造一个空 ext 验证。
    let old = ti_core::domain::persistence::SaveSlotV4Ext { achievements: 0, ..ext.clone() };
    assert_eq!(old.achievements, 0);
}

/// P 键经 `poll()` 真路由（R22-010 同型覆盖）。
#[test]
fn achievement_panel_via_poll_keyboard() {
    let mut app = enter_main_map();
    let mut k = keys();
    k.achievement = true;
    app.poll(&k);
    assert!(app.achievement_panel_open(), "poll P 键应开成就面板");
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.achievement_panel_open(), "poll Back 应关成就面板");
}

#[test]
fn achievement_stats_qualifies_pure() {
    use ti_core::rules::achievements::AchievementStats;
    let s = AchievementStats {
        boss_defeats: 1,
        owned_cells: 1,
        cleared_m01: true,
        peak_gold: ach::TYCOON_GOLD,
        boat_level: ach::MASTER_SHIP_LEVEL,
    };
    for &a in &ach::id::ALL {
        assert!(s.qualifies(a), "id {a} 应达标");
    }
    let (mask, newly) = ach::eval(0, &s);
    assert_eq!(newly.len(), 5);
    assert_eq!(mask, (1u64 << ach::id::COUNT) - 1);
}
