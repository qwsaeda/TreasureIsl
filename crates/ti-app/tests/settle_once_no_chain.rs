//! S74.7 · R-loopcore-1 补测：单次操作不产生多步连锁（app 层）。
//!
//! 业务规则（`docs/business/loop_core.md` §设计约束，`<!-- R-loopcore-1 -->`）：
//! 「一帧一次玩家操作 ⇒ 至多一次结算流水线」。
//!
//! 现有 `map_step_triggers_settle_once`（`reference_diff_main_map.rs`）只覆盖 `rules` 层
//! `MapSession::move_to` 每次移动触发一次落格结算，未覆盖 app 层「一次 `handle(Spin)`
//! 只推进一次棋盘」。本测试补 app 层判据：每次摇骰后棋盘位移恰好等于本次骰点
//! （环岛赛道 N 格，`(after - before) mod 31 == last_roll`）——若一次 Spin 触发多步连锁，
//! 位移会大于骰点 ⇒ 失败。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use ti_app::{GameApp, GameInput, Scene};
use ti_core::rules::board::BoardSession;

/// 进主地图（复用 `fuzz_playtest`/`e2e_playability` 的公开 API 路径）。
fn enter_main_map(seed: u32) -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(seed);
    app.handle(GameInput::Confirm); // Index → SelectRole
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Next); // 选第 0 张地图
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap);
    app
}

/// 一次 Spin 的棋盘位移恰好等于本次骰点（不多步连锁）。
#[test]
fn spin_advances_exactly_one_roll_no_chain() {
    let mut checked = 0u32;
    for seed in 0..24u32 {
        let mut app = enter_main_map(seed);
        let pos_before = app.board.as_ref().map_or(0, BoardSession::position_index);

        app.handle(GameInput::Spin);

        // 落功能格/BOSS 可能切场景，这类 seed 跳过（本测试只验普通格的单步推进）
        if app.scene() != Scene::MainMap {
            continue;
        }
        let pos_after = app.board.as_ref().map_or(0, BoardSession::position_index);
        let roll = app.board.as_ref().map_or(0, BoardSession::last_roll);
        assert!(roll > 0, "Spin 后应有有效骰点 (seed={seed})");

        let spaces =
            app.board.as_ref().map_or(31, |b| u32::try_from(b.track().len()).unwrap_or(31));
        let advanced = (pos_after + spaces - pos_before) % spaces;
        assert_eq!(
            advanced, roll,
            "seed={seed}: 一次 Spin 位移 {advanced} 应恰好等于骰点 {roll}（不多步连锁）"
        );
        checked += 1;
    }
    assert!(checked >= 12, "至少应有 12 个 seed 落在普通格供校验（实际 {checked}）");
}

/// 连续两次 Spin 各自独立结算一次（第二次位移仍等于第二次骰点）。
#[test]
fn consecutive_spins_each_settle_once() {
    let mut app = enter_main_map(7);
    for round in 0..3 {
        if app.scene() != Scene::MainMap {
            break;
        }
        let before = app.board.as_ref().map_or(0, BoardSession::position_index);
        app.handle(GameInput::Spin);
        if app.scene() != Scene::MainMap {
            break;
        }
        let after = app.board.as_ref().map_or(0, BoardSession::position_index);
        let roll = app.board.as_ref().map_or(0, BoardSession::last_roll);
        let spaces =
            app.board.as_ref().map_or(31, |b| u32::try_from(b.track().len()).unwrap_or(31));
        let advanced = (after + spaces - before) % spaces;
        assert_eq!(
            advanced, roll,
            "第 {round} 次 Spin 位移 {advanced} 应等于骰点 {roll}（每次独立结算一次）"
        );
    }
}
