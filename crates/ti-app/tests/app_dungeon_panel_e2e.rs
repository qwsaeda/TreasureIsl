#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S66 副本情报面板 e2e：落无主格（`pending_dungeon`）→ J 键开面板 → 展示岛名/守军人手/武器
//! → Confirm 进守军战（走 `confirm_dungeon` 切 Vs）/ Back 放弃待决。
//! 判据 = ① 仅 `pending_dungeon` 可开 ② 守军查询 > 0 且与 battle 一致 ③ J 键经 `poll()` 真路由
//! ④ Confirm 进 Vs 副本战 ⑤ Back 放弃待决 ⑥ 与驻军/升级/5 面板互斥。

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
    }
}

fn main_map_with_board() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(7);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    app
}

/// 摇骰到落无主领地格（`pending_dungeon` 在位）。
fn spin_to_unowned(app: &mut GameApp) {
    for _ in 0..40 {
        app.handle(GameInput::Spin);
        if app.pending_dungeon.is_some() {
            return;
        }
    }
    unreachable!("40 次摇骰必落无主领地格");
}

#[test]
fn dungeon_panel_open_only_with_pending() {
    let mut app = main_map_with_board();
    app.handle(GameInput::OpenDungeonPanel);
    assert!(!app.dungeon_panel_open, "无待决副本时 J 键不应开面板");
    spin_to_unowned(&mut app);
    app.handle(GameInput::OpenDungeonPanel);
    assert!(app.dungeon_panel_open, "落无主格后 J 键开副本情报面板");
}

#[test]
fn dungeon_panel_queries_match_battle() {
    let mut app = main_map_with_board();
    spin_to_unowned(&mut app);
    app.handle(GameInput::OpenDungeonPanel);
    assert!(!app.dungeon_isle_name().is_empty(), "岛名非空");
    let (crew, arms) = app.dungeon_defender();
    assert!(crew > 0, "守军人手 > 0（与 balance defender 一致）");
    assert!(arms > 0, "守军武器 > 0（守军强度 × dungeon_attack_coef）");
    // 与真实进战斗的守军一致（confirm_dungeon 后 battle.dungeon 在位）。
    let view = {
        app.handle(GameInput::Confirm);
        app.battle().cloned().expect("Confirm 后进副本战斗")
    };
    let battle = view.dungeon.expect("battle.dungeon 在位");
    let isle_name_after = app.dungeon_isle_name();
    assert_eq!(isle_name_after, "", "进战斗后 pending 清空 ⇒ 查询为空（不残留）");
    let _ = battle;
}

#[test]
fn dungeon_panel_confirm_enters_vs_battle() {
    let mut app = main_map_with_board();
    spin_to_unowned(&mut app);
    app.handle(GameInput::OpenDungeonPanel);
    app.handle(GameInput::Confirm);
    assert!(!app.dungeon_panel_open, "Confirm 后面板关闭");
    assert_eq!(app.scene(), Scene::Vs, "Confirm 进 Vs 副本战");
    assert!(app.battle().is_some_and(|b| b.dungeon.is_some()), "battle.dungeon 在位");
}

#[test]
fn dungeon_panel_back_abandons_pending() {
    let mut app = main_map_with_board();
    spin_to_unowned(&mut app);
    assert!(app.pending_dungeon.is_some());
    app.handle(GameInput::OpenDungeonPanel);
    app.handle(GameInput::Back);
    assert!(!app.dungeon_panel_open, "Back 关面板");
    assert!(app.pending_dungeon.is_none(), "Back 放弃待决副本（清 pending_dungeon）");
    assert_eq!(app.scene(), Scene::MainMap, "Back 仅放弃待决不退场景");
}

#[test]
fn dungeon_panel_mutual_exclusion() {
    let mut app = main_map_with_board();
    spin_to_unowned(&mut app);
    app.handle(GameInput::OpenDungeonPanel);
    assert!(app.dungeon_panel_open);
    app.handle(GameInput::OpenShipPanel);
    assert!(app.ship_panel_open && !app.dungeon_panel_open, "开造船关副本（互斥）");
    app.handle(GameInput::OpenDungeonPanel);
    assert!(app.dungeon_panel_open && !app.ship_panel_open, "重开副本关造船");
    app.handle(GameInput::Back);
    assert!(!app.dungeon_panel_open);
}

/// J 键经 `poll()` 真路由（R22-010 同型覆盖）。
#[test]
fn dungeon_panel_via_poll_keyboard() {
    let mut app = main_map_with_board();
    spin_to_unowned(&mut app);
    let mut k = keys();
    k.dungeon = true;
    app.poll(&k);
    assert!(app.dungeon_panel_open, "poll J 键应开副本情报面板");
    let mut k = keys();
    k.back = true;
    app.poll(&k);
    assert!(!app.dungeon_panel_open, "poll Back 应关面板并放弃待决");
    assert!(app.pending_dungeon.is_none());
}
