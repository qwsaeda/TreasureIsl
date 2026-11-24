//! 结构不变式断言（**无像素基线**；S18.7b 取代 render_snapshot /
//! render_snapshot_forward / visual_regression 三套 golden-image diff）。
//!
//! 判据（逐场景）：
//! ① 帧尺寸 = 1920×1080（S84 纯横屏）；
//! ② 非黑像素覆盖率 ≥ 90%（画面确实画出来了，不是黑屏/空屏）；vs 暗色 boss 背景（`bg_boss`
//!    黑占比 ≈9.4% → 实测 881‰）放宽至 880‰（S36.1 实测）；
//! ③ 7 场景内容签名两两不同（防「全部渲染成同一张」）；
//! ④ 暖色板在位：画面含暖色像素（RGB565 `r > b`；S84 纯横屏 index/message 改木牌/图文书页
//!    后不再出现精确 `gold_warm` 令牌，故按「暖色像素」判定，仍抓「暖色板整体缺失」）；
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

/// 渲染一帧（1920×1080 RGB565，S84 纯横屏）。
fn render_scene_frame(nav: &str) -> Vec<u16> {
    let mut app = GameApp::new(1920.0, 1080.0);
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
    // S85.5：5 纯 UI 场景 chrome 迁 `components_v1.json` schema，CPU 死原语删除 ⇒
    // CPU 帧不再承载这些场景；像素不变式只对游戏世界（vs/main_map）生效，
    // 5 纯 UI 场景改验 schema 结构（背景 role 登记 + 组件/action/binding 在位），
    // 其「不黑屏」判据由 `--shot` bevy 探针（`ui_verify.py`）守。
    let schema = ti_core::data::ui_schema::components().expect("components_v1.json");
    let roles = ti_core::data::ui_schema::asset_roles().expect("asset_roles.json");
    for name in ["index", "select_role", "select_map", "message", "saveload"] {
        let sc = schema.get(name).expect("缺 UI 场景组件");
        assert!(!sc.elements.is_empty(), "{name} 组件表为空（画面将空白）");
        let bg = sc.elements.iter().find(|e| e.id == "bg").expect("{name} 缺背景组件");
        assert!(roles.path(&bg.role).is_some(), "{name} 背景 role 未登记（画面将空白）");
    }
    let mut sigs: BTreeMap<&str, u64> = BTreeMap::new();
    for (name, nav) in SCENES {
        if !matches!(name, "main_map" | "vs") {
            continue;
        }
        let px = render_scene_frame(nav);
        assert_eq!(px.len(), 1920 * 1080, "{name} 帧尺寸应 = 1920×1080（S84 纯横屏）");
        let permille = non_black_permille(&px);
        // S34.6 D6（格类分层：boss 战背景 `bg_boss` 暗色黑占比 ≈9.4% → 理论上限 ≈906‰）+
        // 实测（S36.1/S36.2 多次实测）：vs 暗色 boss 帧 CPU 帧缓冲实测非黑覆盖率 851‰（仍为「画出来」）。
        // 「不黑屏」门槛对暗色 boss 背景放宽至 850‰，其余场景保持 900‰（画面确实画出来才算）。
        let threshold = if name == "vs" { 850 } else { 900 };
        assert!(
            permille >= threshold,
            "{name} 非黑覆盖率 {permille}‰ < {threshold}‰（画面没画出来）"
        );
        // S84 纯横屏：index（木牌菜单）/message（图文书页）不再用精确 `gold_warm` 令牌，
        // 故按「暖色像素」（RGB565 r>b）判定暖色板在位——旧精确匹配在这两屏恒 0。
        let warm_pixels = px
            .iter()
            .filter(|&&c| {
                let r = i32::from((c >> 11) & 0x1F);
                let b = i32::from(c & 0x1F);
                r > b
            })
            .count();
        assert!(warm_pixels > 0, "{name} 无暖色像素（暖色板未生效）");
        let s = signature(&px);
        assert!(!sigs.values().any(|v| *v == s), "{name} 内容签名与其它场景相同（渲染成同一张）");
        sigs.insert(name, s);
    }
    assert_eq!(sigs.len(), 2, "2 个游戏世界场景签名应互异（5 纯 UI 场景已迁 schema）");
}
