//! S67.4：确定性 UI 流程测试（LGUI `lgui test` 假时钟借鉴）。
//!
//! 同 seed + 同固定帧时间 + 同输入序列 ⇒ 两次运行快照逐字段一致。
//! 固定手段 = `set_rng_seed(seed)`（骰子/RNG 流）+ `set_frame_delta_ms(16)`
//! （固定帧时间 = 假时钟等价物，消除 `Time::delta()` 的不确定性）。
//! 不依赖真实时钟、不依赖渲染线程，纯 `GameApp` 状态推进。

use ti_app::game_app::{GameApp, GameInput, HitData};

/// 一次 UI 流程 = `index` → `select_role` → `select_map` → `main_map` → 摇骰×5 → 贸易买/卖。
/// 返回可逐字节比较的状态快照字符串（确定性判据）。
fn run_flow_with_seed(seed: u32) -> String {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(seed);
    app.set_rng_seed(seed);
    app.set_frame_delta_ms(16); // 假时钟：固定帧时间，杜绝 wall-clock 漂移

    let hit = |app: &mut GameApp, key: u32, index: i32| {
        app.poll_hit(Some(HitData { key, index, swipe: 0 }))
    };
    assert!(hit(&mut app, 1, 0), "index medallion 0 命中");
    assert!(app.handle(GameInput::Confirm), "选角 Confirm 消费");
    assert!(hit(&mut app, 2, 0), "select_map 卡 0 命中");
    for _ in 0..5 {
        app.handle(GameInput::Spin);
    }
    app.handle(GameInput::TradeBuy(1));
    app.handle(GameInput::TradeSell(1));

    format!(
        "scene={:?}|gold={}|bank={}|pos={:?}|owned={}|buildings={}|boat={}",
        app.scene(),
        app.gold(),
        app.bank().gold,
        app.map_pos(),
        app.owned().len(),
        app.buildings().len(),
        app.boat_level(),
    )
}

#[test]
fn deterministic_ui_flow_same_seed_same_snapshot() {
    let first = run_flow_with_seed(42);
    let second = run_flow_with_seed(42);
    assert_eq!(first, second, "同 seed + 同假时钟 + 同输入 ⇒ 快照逐字节一致");
}

#[test]
fn deterministic_ui_flow_different_seed_differs() {
    // 负例护栏：若 RNG 与流程脱钩，两个 seed 会给出同一快照（假确定性）。
    let baseline = run_flow_with_seed(42);
    let alternative = run_flow_with_seed(99999);
    assert_ne!(baseline, alternative, "不同 seed 应给出不同轨迹（防 RNG 脱钩）");
}
