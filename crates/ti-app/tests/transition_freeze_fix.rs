//! S17.9 场景切换过渡修复回归：
//!
//! 根因（EXP）：S4.3 静态帧跳渲染与 S17.4 时间制交叉淡化冲突——过渡推进只在
//! `GameApp::render()` 内，主循环静止帧（无输入沿）跳过 render → 240ms 交叉淡化
//! 冻结在首帧 ~7% 旧画面 → 切屏残影「没有清屏」。修复：主循环 render 门控加
//! `transition_active()`（过渡窗口强制逐帧渲染）+ 交叉淡化套 ease-in-out 曲线。
//!
//! 本测试锁三条判据：
//! 1. 稳态（无切换）`transition_active() == false`（不误触发逐帧渲染，S4.3 优化存活）；
//! 2. 切场景后 `transition_active() == true`，逐帧 `render()` 16ms 推进可在
//!    240ms/16ms≈15 帧内走完（≤64 帧上限防死循环）；
//! 3. 过渡终帧像素 == 纯新场景渲染（无旧画面残色）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use ti_app::game_app::{GameApp, GameInput};

/// 判据 1：稳态无过渡。
#[test]
fn steady_state_has_no_active_transition() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_frame_delta_ms(16);
    app.render();
    app.render();
    assert!(!app.transition_active(), "稳态（无场景切换）不应有过渡");
}

/// 判据 2 + 3：切场景后过渡逐帧走完，终帧 = 纯新场景（无旧画面残留）。
#[test]
fn transition_completes_within_window_and_ends_on_pure_new_scene() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_frame_delta_ms(16);
    app.render(); // 首帧渲染（index 稳态），rendered_once = true
    assert!(!app.transition_active(), "切换前无过渡");

    app.handle(GameInput::Confirm); // Index → SelectRole（enter 内 start_transition）
    assert!(app.transition_active(), "场景切换应立即启动 240ms 过渡");

    // 逐帧推进（主循环修复后：transition_active 门控下每帧都会 render）。
    // 240ms / 16ms ≈ 15 帧；上限 64 帧 = 4 倍窗口，防回归后死循环。
    let mut frames = 0usize;
    while app.transition_active() && frames < 64 {
        app.render();
        frames += 1;
    }
    assert!(
        !app.transition_active(),
        "240ms 窗口（15 帧@16ms）内过渡应走完，实际 {frames} 帧仍未结束"
    );
    assert!(frames <= 20, "16ms 帧步长下应在 ~15 帧内完成（ease 不影响进度推进），实际 {frames}");

    // 终帧 == 纯新场景渲染（transition 已 None，再渲一帧像素应逐位不变）。
    let final_frame = app.shell().fb.as_slice().to_vec();
    app.render();
    let clean_frame = app.shell().fb.as_slice().to_vec();
    assert_eq!(final_frame, clean_frame, "过渡终帧应等于纯新场景渲染（无旧画面残色）");

    // 新旧场景帧互异（证明过渡确实发生过混合，非空转）。
    let mut ref_app = GameApp::new(1080.0, 1920.0);
    ref_app.set_frame_delta_ms(16);
    ref_app.render();
    let index_frame = ref_app.shell().fb.as_slice();
    assert_ne!(
        clean_frame,
        index_frame.to_vec(),
        "SelectRole 终帧应与 Index 帧互异（场景确实切走了）"
    );
}
