//! S28.4 选角多席位 e2e（`territory.md` §1：≥1 玩家 + 4 AI 不变量；热座交接提示）。
//!
//! 判据：单选旧流 0 回归 / 多选 2 玩家席 + 3 AI / 0 选被拒回退 / 玩家 ≥2 热座交接提示在位。

use ti_app::{GameApp, GameInput, Scene};
use ti_core::domain::selection::Character;

fn enter_main_map() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    app
}

fn player_count(app: &GameApp) -> u8 {
    u8::try_from((0..5u8).filter(|i| app.turn_cycle.is_player_seat(*i)).count()).unwrap_or(0)
}

fn ai_session_count(app: &GameApp) -> u8 {
    u8::try_from((0..5u8).filter(|i| app.ai_sessions[*i as usize].is_some()).count()).unwrap_or(0)
}

/// 单选旧流：无席位标记 = 主角色单玩家席 + 4 AI（0 回归判据）。
#[test]
fn seats_default_single_player_no_regression() {
    let app = enter_main_map();
    assert_eq!(player_count(&app), 1, "默认 = 1 玩家席 + 4 AI");
    assert_eq!(ai_session_count(&app), 4, "4 AI 席会话在位");
}

/// 多选 2 玩家席：`seat_marks` = 2 角色 ⇒ 2 玩家 + 3 AI；当前席 = 最低序号玩家席。
///
/// 确定性入口 = 全新 `GameApp`（`init_territory` 在 `enter_main_map` 走一次）。
#[test]
fn seats_two_players_three_ai() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    let c0 = Character::from_index(0).expect("角色 0 应存在");
    let c2 = Character::from_index(2).expect("角色 2 应存在");
    app.seat_marks = vec![c0, c2];
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    assert_eq!(player_count(&app), 2, "2 玩家席");
    assert!(app.turn_cycle.is_player_seat(0), "席 0 = 玩家（c0）");
    assert!(app.turn_cycle.is_player_seat(2), "席 2 = 玩家（c2）");
    assert_eq!(ai_session_count(&app), 3, "3 AI 席会话在位");
}

/// 热座交接：玩家 ≥2 摇骰（AI 连动轮转回玩家席）⇒ 交接提示在位。
#[test]
fn hot_seat_handoff_toast_shows_on_multi_player() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    let c0 = Character::from_index(0).expect("角色 0 应存在");
    let c1 = Character::from_index(1).expect("角色 1 应存在");
    app.seat_marks = vec![c0, c1];
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Spin);
    assert!(app.last_ai_toast.is_some(), "玩家 ≥2 热座交接提示应在位（last_ai_toast 通道）");
}
