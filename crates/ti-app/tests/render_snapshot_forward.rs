//! S14 正向渲染像素快照门禁（第三套，S15.5 重锚）：7 场景 1080 代码绘制 forward 基线 diff。
//!
//! S15.4 移除 legacy 逐像素写帧层后，forward 与常规态同为纯代码渲染（`forward_render`
//! 开关 S15.6 QA017 已退役，两态帧恒等 QA014）；本门禁从 176×220 旧基线重锚到 1080×1920
//! 代码绘制基线（`08_render_arch.md` §6 契约 5）：基线 `assets/forward_baseline/<scene>.png`
//! （scenes.json 键名），缺失 / legacy 尺寸 → 重生成，同尺寸 → 逐像素 diff=0。
//! 7 张基线两两不同（防空转）。

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

use ti_app::forward_nav::{FORWARD_SCENES, navigate};
use ti_app::game_app::GameApp;

fn render_forward_frame(scene_key: &str) -> Vec<u16> {
    let mut app = GameApp::new(1080.0, 1920.0);
    // S15.6 QA125：基线必须 TTF 在位（44px 真字形态），缺失即 panic 防豆腐基线入库
    snapshot_util::require_ttf_present(&app, "render_snapshot_forward");
    let _ = navigate(&mut app, scene_key);
    app.render_to_framebuf().to_vec()
}

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
}

#[test]
fn forward_snapshot_all_scenes() {
    for scene in FORWARD_SCENES {
        let frame = render_forward_frame(scene);
        assert_eq!(frame.len(), snapshot_util::FB_PIXELS, "{scene} 帧缓冲应 = 1080×1920");
        let dir = repo_root().join("assets").join("forward_baseline");
        match snapshot_util::baseline_gate(scene, &dir, &frame) {
            snapshot_util::BaselineResult::Match => {}
            snapshot_util::BaselineResult::Regenerated => {}
            snapshot_util::BaselineResult::Drift { diff, max_delta } => {
                panic!(
                    "{scene} forward 快照漂移：diff 像素 {diff}，最大通道偏差 {max_delta}（1080 代码绘制基线，契约 §5）"
                );
            }
        }
    }
    // 7 张基线两两不同（防空转）。
    let dir = repo_root().join("assets").join("forward_baseline");
    let mut digests: Vec<(String, u64)> = Vec::new();
    for scene in FORWARD_SCENES {
        let bytes = std::fs::read(dir.join(format!("{scene}.png")))
            .unwrap_or_else(|e| panic!("基线 {scene} 读取失败: {e}"));
        let digest = bytes
            .iter()
            .fold(0x517c_e1b7_0007u64, |h, &b| h.wrapping_mul(0x100_0000_01b3) ^ u64::from(b));
        digests.push((scene.to_owned(), digest));
    }
    for i in 0..digests.len() {
        for j in (i + 1)..digests.len() {
            assert_ne!(
                digests[i].1, digests[j].1,
                "forward 基线 {} 与 {} 字节完全相同 —— 门禁对该场景空转",
                digests[i].0, digests[j].0
            );
        }
    }
}
