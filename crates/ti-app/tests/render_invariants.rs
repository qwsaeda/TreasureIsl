//! 结构不变式断言（**无像素基线**；S18.7b 取代 render_snapshot /
//! render_snapshot_forward / visual_regression 三套 golden-image diff）。
//!
//! 判据（逐场景）：
//! ① 帧尺寸 = 1080×1920；
//! ② 非黑像素覆盖率 ≥ 90%（画面确实画出来了，不是黑屏/空屏）；
//! ③ 7 场景内容签名两两不同（防「全部渲染成同一张」）；
//! ④ 暖色板在位：画面含 `gold_warm` 像素（标题/选中态金墨）；
//! ⑤ TTF 在位（QA125：豆腐字形不算数）。
//!
//! 设计取舍：开发期**不维护 golden PNG**——像素级微调（抗锯麧/字形光栅化/半像素偏移）
//! 会让基线频繁失效；改为断言**结构与语义不变式**（画了没有、是不是同一张、配色对不对）。
//! 像素级对齐由 S18.7b 的 **CPU↔Bevy 内存对拍**（无文件）承担。
#![allow(clippy::match_same_arms, clippy::many_single_char_names, clippy::items_after_statements)]
#![allow(clippy::doc_markdown, clippy::uninlined_format_args)]

#[path = "common/snapshot_util.rs"]
mod snapshot_util;

use std::collections::BTreeMap;

use ti_app::forward_nav::navigate;
use ti_app::game_app::GameApp;

/// 场景（`forward_nav` 键）。
const SCENES: [(&str, &str); 7] = [
    ("index", "index"),
    ("select_role", "select_role"),
    ("select_map", "select_map"),
    ("main_map", "main_map"),
    ("vs", "vs"),
    ("message", "message"),
    ("saveload", "saveload"),
];

/// 渲染一帧（1080×1920 RGB565）。
fn render_scene_frame(nav: &str) -> Vec<u16> {
    let mut app = GameApp::new(1080.0, 1920.0);
    snapshot_util::require_ttf_present(&app, "render_invariants");
    let _ = navigate(&mut app, nav);
    app.render_to_framebuf().to_vec()
}

/// 非黑像素覆盖率百分比（整数运算，无浮点转换）。
fn non_black_permille(px: &[u16]) -> u64 {
    let n = px.iter().filter(|c| **c != 0).count() as u64;
    let total = px.len().max(1) as u64;
    n * 1000 / total
}

/// 内容指纹（FNV-1a）。
fn signature(px: &[u16]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for c in px {
        h ^= u64::from(*c);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[test]
fn render_invariants_all_scenes() {
    let theme = ti_render::theme::Theme::default();
    let gold = theme.by_name("gold_warm").unwrap_or(0xFEA0);
    let mut sigs: BTreeMap<&str, u64> = BTreeMap::new();
    for (name, nav) in SCENES {
        let px = render_scene_frame(nav);
        assert_eq!(px.len(), 1080 * 1920, "{name} 帧尺寸应 = 1080×1920");
        let permille = non_black_permille(&px);
        assert!(permille >= 900, "{name} 非黑覆盖率 {permille}‰ < 900‰（画面没画出来）");
        assert!(px.contains(&gold), "{name} 无 gold_warm 像素（暖色板未生效）");
        let s = signature(&px);
        assert!(!sigs.values().any(|v| *v == s), "{name} 内容签名与其它场景相同（渲染成同一张）");
        sigs.insert(name, s);
    }
    assert_eq!(sigs.len(), SCENES.len(), "7 场景签名应互异");
}
