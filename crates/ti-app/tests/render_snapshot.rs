//! 渲染快照门禁（契约 §5，S15.5 重锚）：7 场景 1080 代码绘制全帧基线 diff。
//!
//! S15.4 移除 legacy 逐像素写帧层后，本门禁从 176×220 legacy 精灵层基线重锚到
//! 1080×1920 代码绘制基线（`08_render_arch.md` §6 契约 5）：
//! - 帧来源 `GameApp::render_to_framebuf()`（RGB565 1080×1920，确定性，契约 3）。
//! - 基线 `assets/render_baseline/<scene>.png`（RGB8 DEFLATE，`snapshot_util` 共用机制）。
//! - 基线缺失 / legacy 176×220 尺寸 → 重生成；同尺寸 → 逐像素 diff=0。
//! - 7 张基线两两不同（防空转）。

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

/// legacy `.Map` 场景名 → `forward_nav` 的 scenes.json 键。
fn nav_key(legacy: &str) -> &str {
    match legacy {
        "index" => "index",
        "selectrole" => "select_role",
        "selectmap" => "select_map",
        "main" => "main_map",
        "vs" => "vs",
        "msgs" => "message",
        "saveload" => "saveload",
        _ => legacy,
    }
}

const SCENES: [&str; 7] = ["index", "selectrole", "selectmap", "main", "vs", "msgs", "saveload"];

fn render_scene_frame(legacy_name: &str) -> Vec<u16> {
    let mut app = GameApp::new(1080.0, 1920.0);
    // S15.6 QA125：基线必须 TTF 在位（44px 真字形态），缺失即 panic 防豆腐基线入库
    snapshot_util::require_ttf_present(&app, "render_snapshot");
    let _ = navigate(&mut app, nav_key(legacy_name));
    app.render_to_framebuf().to_vec()
}

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
}

fn baseline_path(name: &str) -> PathBuf {
    repo_root().join("assets").join("render_baseline").join(format!("{name}.png"))
}

#[test]
fn render_snapshot_all_scenes() {
    for name in SCENES {
        let frame = render_scene_frame(name);
        assert_eq!(frame.len(), snapshot_util::FB_PIXELS, "{name} 帧缓冲应 = 1080×1920");
        let dir = repo_root().join("assets").join("render_baseline");
        match snapshot_util::baseline_gate(name, &dir, &frame) {
            snapshot_util::BaselineResult::Match => {} // diff=0（1080 代码绘制基线一致）
            snapshot_util::BaselineResult::Regenerated => {} // 首跑 / S15.5 重锚
            snapshot_util::BaselineResult::Drift { diff, max_delta } => {
                panic!(
                    "{name} 渲染快照漂移：diff 像素 {diff}（>0），最大通道偏差 {max_delta}（1080 代码绘制基线，契约 §5）"
                );
            }
        }
    }
    // 7 张基线两两不同（防全同 = 门禁空转；历史曾 5 张字节相同）。
    let mut digests: Vec<(String, u64)> = Vec::new();
    for name in SCENES {
        let bytes = std::fs::read(baseline_path(name))
            .unwrap_or_else(|e| panic!("基线 {name} 读取失败: {e}"));
        let digest = bytes
            .iter()
            .fold(0x517c_e1b7_0007u64, |h, &b| h.wrapping_mul(0x100_0000_01b3) ^ u64::from(b));
        digests.push((name.to_owned(), digest));
    }
    for i in 0..digests.len() {
        for j in (i + 1)..digests.len() {
            assert_ne!(
                digests[i].1, digests[j].1,
                "基线 {} 与 {} 字节完全相同 —— 快照门禁对该场景空转",
                digests[i].0, digests[j].0
            );
        }
    }
}
