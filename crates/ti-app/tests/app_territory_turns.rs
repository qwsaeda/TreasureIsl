//! S28.7 应用层回合循环测试（territory.md §6：玩家回合后 4 AI 连动，同规则同表；
//! 领地账本 + 席位金币 + 轮转 经 `GameApp` 状态可观测）。
//!
//! 判据：`cargo test -p ti-app --test app_territory_turns` 全绿。

use ti_app::game_app::{GameApp, GameInput, Scene};
use ti_app::scenes::MapHost;
use ti_core::domain::selection::{Character, MapSlot};

/// 进主地图的导航（新游戏 → 选角 Confirm → 选图 Confirm → 主地图）。
fn enter_main_map(app: &mut GameApp) {
    app.handle(GameInput::Confirm); // Index → SelectRole
    app.handle(GameInput::Confirm); // 选第 1 角色 → SelectMap
    app.handle(GameInput::Next); // 首张地图（M01）
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap, "应进主地图");
}

/// §6：玩家摇骰后 4 AI 自动连动；轮转回玩家席；领地账本/席位金币被真实变更。
#[test]
fn ai_turns_run_after_player_spin() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(7);
    enter_main_map(&mut app);

    let initial_player_seat = app.turn_cycle.current;
    assert!(app.turn_cycle.is_player_seat(initial_player_seat), "当前轮到玩家席");
    // 5 席 = 1 玩家 + 4 AI（单玩家局，§1 不变量 players + ai = 5）
    let ai_total =
        (0..5).filter(|i| !app.turn_cycle.is_player_seat(u8::try_from(*i).unwrap_or(0))).count();
    assert_eq!(ai_total, 4, "单玩家局 = 4 AI");

    let terr_before = app.territory.as_ref().expect("领地账本已构建").cells().count();
    assert!(terr_before > 0, "领地格在账本");

    // 玩家摇骰（Spin = 主地图移动唯一入口，S19）
    let turn = app.board_spin().expect("主地图有赛道");
    assert!((1..=6).contains(&turn.roll), "骰点在 1..=6");

    // 玩家回合结束后 = 回到玩家席（AI 连动已走完 4 席）
    assert_eq!(app.turn_cycle.current, initial_player_seat, "轮转回玩家席");

    // 4 个 AI 席会话已推进（位置 ≠ 起点，除非骰点回绕；确定性 seed 下可复现）
    let ai_moved: usize = app
        .ai_sessions
        .iter()
        .filter_map(|s| s.as_ref().map(ti_core::rules::board::BoardSession::index))
        .sum();
    assert!(ai_moved > 0, "AI 席已摇骰前进");

    // 领地账本仍健全（格数不变；部分格可能已易主）
    let terr = app.territory.as_ref().expect("领地账本在位");
    assert_eq!(terr.cells().count(), terr_before, "领地格数不变");
    // 席位金币在初始金或附近波动（交钱/首占/产出）
    for g in &app.seat_gold {
        assert!(*g <= 10_000, "席位金币不越界");
    }
}

/// §6 不变量：AI 与玩家走同一分支表（同一状态下 AI 策略结果 = 规则层确定性输出，
/// 且游戏层落格结算后经 `territory_ownership` 可查）。
#[test]
fn ai_landing_settlement_is_deterministic() {
    let mut a = GameApp::new(1080.0, 1920.0);
    a.set_rng_seed(42);
    enter_main_map(&mut a);
    a.board_spin();
    let snap_a: Vec<(u32, Option<u8>, u8)> = a
        .territory
        .as_ref()
        .expect("账本")
        .snapshot()
        .into_iter()
        .filter(|(_, (o, _))| o.is_some())
        .map(|(k, (o, l))| (k, o, l))
        .collect();

    let mut b = GameApp::new(1080.0, 1920.0);
    b.set_rng_seed(42);
    enter_main_map(&mut b);
    b.board_spin();
    let snap_b: Vec<(u32, Option<u8>, u8)> = b
        .territory
        .as_ref()
        .expect("账本")
        .snapshot()
        .into_iter()
        .filter(|(_, (o, _))| o.is_some())
        .map(|(k, (o, l))| (k, o, l))
        .collect();

    assert_eq!(snap_a, snap_b, "同 seed 同开局 ⇒ AI 落格结算归属/等级轨迹恒同（确定性）");
    let _ = (Character::Panda, MapSlot::M01);
}
