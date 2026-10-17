//! 视觉回归门禁（EXP-072，S15.5 重锚）：3 关键场景 1080 代码绘制基线逐像素 diff。
//!
//! S15.4 移除 legacy 176×220 逐像素写帧层 + 旧基线后，本门禁重锚到 1080×1920 代码绘制基线
//! （`08_render_arch.md` §6 契约 5）：基线 `assets/hd_baseline/<Scene>.png`（RGB8 DEFLATE），
//! 缺失 / legacy 尺寸 → 重生成；同尺寸 → 逐像素 diff=0。与 `scripts/visual_baseline.ps1` 同口径。

#![allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::match_same_arms,
    clippy::many_single_char_names,
    clippy::items_after_statements,
    clippy::doc_markdown,
    clippy::uninlined_format_args
)]

#[path = "common/snapshot_util.rs"]
mod snapshot_util;

use std::path::PathBuf;

use ti_app::forward_nav::navigate;
use ti_app::game_app::GameApp;

/// 3 关键场景：hd_baseline 文件名（PascalCase，`visual_baseline.ps1` 口径）→ `forward_nav` scenes.json 键。
const SCENES: [(&str, &str); 3] =
    [("Index", "index"), ("SelectRole", "select_role"), ("MainMap", "main_map")];

fn render_scene_frame(nav_key: &str) -> Vec<u16> {
    let mut app = GameApp::new(1080.0, 1920.0);
    // S15.6 QA125：基线必须 TTF 在位（44px 真字形态），缺失即 panic 防豆腐基线入库
    snapshot_util::require_ttf_present(&app, "visual_regression");
    let _ = navigate(&mut app, nav_key);
    app.render_to_framebuf().to_vec()
}

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
}

#[test]
fn visual_regression_3_scenes() {
    let dir = repo_root().join("assets").join("hd_baseline");
    for (name, nav_key) in SCENES {
        let frame = render_scene_frame(nav_key);
        assert_eq!(frame.len(), snapshot_util::FB_PIXELS, "{name} 帧缓冲应 = 1080×1920");
        match snapshot_util::baseline_gate(name, &dir, &frame) {
            snapshot_util::BaselineResult::Match => {}
            snapshot_util::BaselineResult::Regenerated => {}
            snapshot_util::BaselineResult::Drift { diff, max_delta } => {
                panic!(
                    "{name} 视觉回归漂移：diff 像素 {diff}，最大通道偏差 {max_delta}（1080 代码绘制基线，EXP-074 掩背景已废弃，整帧 diff=0）"
                );
            }
        }
    }
}
