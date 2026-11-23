//! S74.5 稳定性验证：同进程内跑 3 次「轻量但覆盖关键路径」的流程，断言逐字节一致（0 flaky）。
//!
//! 借鉴 `e2e_ui_flow_deterministic.rs`（S67.4）的成功模式：固定 seed + 假时钟
//! （`set_frame_delta_ms(16)` = 用固定帧时间取代 wall-clock），消除时间漂移与 RNG 不确定性。
//! 关键路径 = `index` → `select_role` → `select_map` → `main_map` → 摇骰 ×N → 贸易买/卖 → 建造。
//! 每次运行独立 `GameApp::new`（无跨运行全局残留），快照字符串逐字节比较。

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ti_app::game_app::{GameApp, GameInput, HitData};

/// 固定 seed（夹具来源 = S67.4 `e2e_ui_flow_deterministic.rs` 同口径，纯测试夹具非业务数值）。
const STABILITY_SEED: u32 = 20_241_123;
/// 假时钟帧时间（ms；夹具来源同上，16ms ≈ 60FPS 固定步长）。
const FIXED_FRAME_MS: u32 = 16;
/// 重复次数（本任务判据 = 3 次一致）。
const REPEATS: u32 = 3;

/// 一次流程 = 关键路径推进，返回可逐字节比较的状态快照字符串。
fn run_key_path(seed: u32) -> String {
    let mut app = GameApp::new(1080.0, 1920.0);
    // 固定 RNG 流（骰子/抽卡/AI 全部由 seed 决定）。
    app.rng_mut().set_seed(seed);
    app.set_rng_seed(seed);
    // 假时钟：固定帧时间，杜绝 wall-clock 漂移。
    app.set_frame_delta_ms(FIXED_FRAME_MS);

    let hit = |app: &mut GameApp, key: u32, index: i32| {
        app.poll_hit(Some(HitData { key, index, swipe: 0 }))
    };

    // index → select_role → select_map → main_map。
    assert!(hit(&mut app, 1, 0), "index medallion 0 命中");
    assert!(app.handle(GameInput::Confirm), "选角 Confirm 消费");
    assert!(hit(&mut app, 2, 0), "select_map 卡 0 命中");

    // 主地图摇骰 ×3，落格结算（骰子走固定 seed）。
    for _ in 0..3 {
        app.handle(GameInput::Spin);
    }
    // 贸易买/卖（经济键路径）。
    app.handle(GameInput::TradeBuy(1));
    app.handle(GameInput::TradeSell(1));

    format!(
        "scene={:?}|seed={seed}|gold={}|bank={}|pos={:?}|owned={}|buildings={}|boat={}",
        app.scene(),
        app.gold(),
        app.bank().gold,
        app.map_pos(),
        app.owned().len(),
        app.buildings().len(),
        app.boat_level(),
    )
}

/// 判据①：同进程 3 次同 seed + 假时钟运行 ⇒ 3 份快照逐字节一致（0 flaky）。
#[test]
fn stability_repeat_three_runs_identical() {
    let mut snapshots: Vec<String> = Vec::new();
    for run in 1..=REPEATS {
        let snap = run_key_path(STABILITY_SEED);
        println!("[stability] run {run}/{REPEATS}: {snap}");
        snapshots.push(snap);
    }
    for run in 2..=REPEATS {
        assert_eq!(
            snapshots[0],
            snapshots[(run - 1) as usize],
            "第 {run} 次与第 1 次快照不一致（flaky 风险）"
        );
    }
    println!("[stability] 3/3 快照逐字节一致（0 flaky）：{}", snapshots[0]);
}

/// 判据②（负例护栏）：不同 seed ⇒ 快照不同，防「RNG 脱钩 = 假确定性」。
#[test]
fn stability_different_seed_differs() {
    let baseline = run_key_path(STABILITY_SEED);
    let alternative = run_key_path(STABILITY_SEED.wrapping_add(1));
    println!("[stability] seed={STABILITY_SEED}: {baseline}");
    println!("[stability] seed={}: {alternative}", STABILITY_SEED.wrapping_add(1));
    assert_ne!(baseline, alternative, "不同 seed 应给出不同轨迹（防 RNG 脱钩）");
}
