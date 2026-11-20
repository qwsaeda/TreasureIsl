#![allow(clippy::unwrap_used, clippy::expect_used)]
//! 飞轮 R21 · 方案 B：AI 玩家自动化对抗 E2E（**app 壳级**，走真实 UI 路径 + 场景态 + AI toast）。
//!
//! 方案 A（`ti-core/tests/adversarial_e2e.rs`）= 纯规则层驱动器（对局性质判据：确定性/守恒/可达）。
//! 本文件 = 同一对抗面的**真实链路**形态：经 `GameApp` 真实 UI 路径（Index→选角→选图→主地图，
//! `app_seats_e2e` 同模式）进 2 玩家 + N AI 局，自动 `GameInput::Spin` 多回合（`turn_cycle` 自动
//! 轮转 AI 席 + `settle_ai_landing` AI 结算），断言**场景态/AI toast/终局面**真走全链路：
//!
//! | 断言 | 内容 |
//! |---|---|
//! | 真实链路 | 多回合 spin 后场景恒 `MainMap`（不崩/不跳场景，AI 自动结算不卡 UI） |
//! | AI 自动结算可见 | 有 AI 席的局，spin 后 `last_ai_toast` 在位（AI 落格结算经主地图 toast 通道） |
//! | 确定性 | 同 seed 同 spin 次数 ⇒ 同场景 + 同 toast（AI 决策 + RNG 可复现，真实链路面） |
//! | 终局读面在位 | `match_outcome` 字段可达（对局终局判定面，不强制出胜者——同 A 的收敛缺口） |
//!
//! 与 A 分工：A = 规则层纯函数驱动器（快，对局性质）；B = app 壳真链路（慢，验证 UI/场景/
//! 热座/AI toast 同帧正确 + 真实 `turn_cycle` 轮转）。两面对照 = 「AI 自动对抗真实 E2E 玩」
//! 的规则层 + 表现层双判据（硬规则 2：验收命令实跑输出 = 本测试全绿）。

use ti_app::{GameApp, GameInput, Scene};

/// 走真实 UI 路径进 2 玩家 + `ai` 个 AI 席的主地图（`app_seats_e2e` 同模式）。
fn enter_main_map_with_ai(seed: u32, ai: u32) -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(seed);
    app.set_rng_seed(seed);
    app.handle(GameInput::Confirm); // Index → SelectRole
    app.handle(GameInput::Next);
    app.handle(GameInput::MarkSeat); // 角色 0
    app.handle(GameInput::Next);
    app.handle(GameInput::MarkSeat); // 角色 1
    app.seat_ai_count = Some(ai);
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap, "真实 UI 路径进主地图");
    app
}

/// 断言 1：真实链路（多回合 spin 后场景恒 `MainMap`，AI 自动结算不崩/不跳场景/不卡 UI）。
#[test]
fn app_adversarial_stays_in_main_map() {
    let mut app = enter_main_map_with_ai(7, 3);
    // 多回合自动摇骰（`turn_cycle` 轮转 2 玩家 + 3 AI 席，AI 席经 `settle_ai_landing` 自动结算）。
    for _ in 0..24 {
        app.handle(GameInput::Spin);
        assert_eq!(app.scene(), Scene::MainMap, "AI 自动结算回合后应恒主地图（不崩不跳场景）");
    }
}

/// 断言 2：AI 自动结算可见（有 AI 席的局，spin 后 `last_ai_toast` 在位——AI 落格经主地图 toast 通道）。
#[test]
fn app_adversarial_ai_toast_visible() {
    let mut app = enter_main_map_with_ai(7, 4);
    // 足够多回合保证某 AI 席落格结算（领地/副本/产出格触发 toast）。
    let mut saw_ai_toast = false;
    for _ in 0..48 {
        app.handle(GameInput::Spin);
        if app.last_ai_toast.is_some() {
            saw_ai_toast = true;
            break;
        }
    }
    assert!(saw_ai_toast, "4 AI 局 48 回合内应有 AI 落格 toast（AI 自动结算可见反馈）");
}

/// 断言 3：确定性（同 seed 同 spin 次数 ⇒ 同场景 + 同末次 AI toast；AI 决策 + RNG 可复现）。
#[test]
fn app_adversarial_deterministic_same_seed() {
    let run = |seed: u32| -> (Scene, Option<String>) {
        let mut app = enter_main_map_with_ai(seed, 3);
        for _ in 0..12 {
            app.handle(GameInput::Spin);
        }
        (app.scene(), app.last_ai_toast)
    };
    let a = run(2026);
    let b = run(2026);
    assert_eq!(a, b, "同 seed 同 12 spin ⇒ 同场景 + 同末次 AI toast（确定性复现，真实链路面）");
}

/// 断言 4：终局读面在位（`match_outcome` 字段可达；不强制出胜者——同 A 的对局收敛缺口，
/// 此处只验真实链路终局判定面不崩）。
#[test]
fn app_adversarial_match_outcome_readable() {
    let mut app = enter_main_map_with_ai(7, 2);
    for _ in 0..12 {
        app.handle(GameInput::Spin);
    }
    // `match_outcome` 可访问（`None` = 未终局；`Some` = 已分胜负；两者都不崩 = 真实链路终局面）。
    let _outcome = &app.match_outcome;
    assert!(app.scene() == Scene::MainMap, "多 AI 局 spin 后场景面一致");
}
