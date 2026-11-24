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

/// A-06 玩家落**自己**领地格 = 玩家可选（§4「T 属于你 → 玩家可选 ①升级 ②领产出」），不自动。
///
/// 置 `pending_own` 待决：`confirm_own`（主地图 `Confirm`）= 升级加固（金足）；`abandon_own`
/// （主地图 `Back`）= 领取产出（金币守恒：产出入账自己）。旧实现落自己格自动升级/领产出（无选择权）。
#[test]
fn own_landing_gives_player_choice() {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.set_rng_seed(11);
    enter_main_map(&mut app);
    let me = app.turn_cycle.current;
    // 让玩家占 harbour 首格（构造"自己格"）。
    let cell = app
        .territory
        .as_ref()
        .expect("领地账本")
        .isle_cells("harbour")
        .first()
        .copied()
        .expect("m01 有 harbour");
    {
        let mut seat = ti_core::rules::territory_v2::SeatState::new(1000, 30, 20, 0);
        if let Some(terr) = app.territory.as_mut() {
            let _ = ti_core::rules::territory_v2::capture_cell(&mut seat, terr, cell, me, 5);
        }
    }
    // 摇骰直到落回自己格（`pending_own` 在位 = A-06 待决升级/领产出）。
    let mut saw_own = false;
    for _ in 0..80 {
        app.handle(GameInput::Spin);
        if app.pending_own.is_some() {
            saw_own = true;
            break;
        }
    }
    assert!(saw_own, "玩家摇骰应落回自己领地格（`pending_own` 在位，A-06 玩家可选）");
    let pending_cell = app.pending_own.expect("待决自己格应在位");
    // 领取产出前记录自己金币（守恒基准）。
    let gold_before = app.seat_gold[me as usize];
    // 领产出（`Back` = abandon_own）：产出入账自己（金币守恒，gold 不减少）。
    let consumed = app.abandon_own();
    assert!(consumed, "abandon_own 应消费待决");
    assert!(app.pending_own.is_none(), "领产出后待决应清");
    assert!(app.seat_gold[me as usize] >= gold_before, "领产出 = 产出入账自己（金币守恒，不减少）");
    let _ = pending_cell;
}

/// §6：玩家摇骰后 4 AI 自动连动；轮转回玩家席；领地账本/席位金币被真实变更。
#[test]
fn ai_turns_run_after_player_spin() {
    let mut app = GameApp::new(1920.0, 1080.0);
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

    // 非当前席会话已推进（位置 ≠ 起点，除非骰点回绕；确定性 seed 下可复现）
    let ai_moved: usize = app
        .seat_sessions
        .iter()
        .filter_map(|s| s.as_ref().map(ti_core::rules::board::BoardSession::index))
        .sum();
    assert!(ai_moved > 0, "非当前席已摇骰前进");

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
    let mut a = GameApp::new(1920.0, 1080.0);
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

    let mut b = GameApp::new(1920.0, 1080.0);
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
