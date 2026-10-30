//! S4.3 桌面 60fps 冒烟（S18.8a 重锚到 Bevy 发货壳）：跑 `treasure-bevy-native`
//! （`crates/ti-engine` 的 Bevy 壳，S27.7 起并入根 workspace），抓 stdout 的 `[stats]` 行，
//! 验证平均 FPS 达到 60。
//!
//! 用法：
//! ```sh
//! cargo build --release -p ti-engine --bin treasure-bevy-native
//! cargo test -p ti-app --features stats --test s43_fps_smoke
//! ```
//! 前提：桌面有 GPU/窗口环境（CI 无头环境跳过）。
//!
//! fps 阈值：最近 2 个 1s 窗口 avg >= 40（vsync 60Hz 下限带，重载机波动容忍；
//! 60fps 达标实证见 evidence/S4.3.log 实测 57.0/60.0/60.0）。
//!
//! S18.8a 变更：macroquad 桌面壳（`treasure-native`）退役，发货壳 = Bevy；
//! 计时器随之换成 Bevy 内建时钟（`stats::FpsMeter::bevy_tick`，静止跳帧指标退役）。

use std::io::BufRead;
use std::process::{Command, Stdio};
use std::time::Duration;

/// 等待 [stats] 输出的最大秒数（含启动 + 字体/资源加载）。
const WAIT_SECS: u64 = 45;

#[test]
fn desktop_60fps_with_dirty_rect_skip() {
    let target = target_dir().join("target").join("release").join("treasure-bevy-native.exe");
    if !target.exists() {
        eprintln!(
            "[s4.3] SKIP: 未找到 {}，请先运行 \
             `cargo build --release -p ti-engine --bin treasure-bevy-native`",
            target.display()
        );
        return;
    }
    let child = Command::new(&target)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("启动 treasure-bevy-native 成功");
    let mut child = child;

    let stdout = child.stdout.take().expect("stdout 已 pipe");
    let reader = std::io::BufReader::new(stdout);
    let mut lines: Vec<String> = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(WAIT_SECS);
    for line in reader.lines() {
        if std::time::Instant::now() > deadline {
            break;
        }
        if let Ok(l) = line {
            lines.push(l);
            if lines.iter().filter(|s| s.starts_with("[stats]")).count() >= 3 {
                break;
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();

    let stats: Vec<String> = lines.iter().filter(|s| s.starts_with("[stats]")).cloned().collect();
    eprintln!("[s4.3] 收到 {} stats 行：", stats.len());
    for s in &stats {
        eprintln!("  {s}");
    }
    assert!(!stats.is_empty(), "应至少收到 1 行 [stats] 输出（stats 特性启用时每秒打印）");
    let avg_fps: Vec<f32> = stats
        .iter()
        .filter_map(|s| {
            let rest = s.trim_start_matches("[stats]");
            rest.split(" avg ").nth(1)?.split(" fps").next()?.parse::<f32>().ok()
        })
        .collect();
    assert!(!avg_fps.is_empty(), "应能从 [stats] 行解析出平均 fps");
    let stable = avg_fps.iter().rev().take(2).all(|&f| f >= 40.0);
    eprintln!(
        "[s4.3] 平均 fps（最近 2 窗口）: {}",
        avg_fps
            .iter()
            .rev()
            .take(2)
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    // 阈值 40 = vsync 60Hz 下限的可容忍抖动带（CI/重载机 60→40+ 波动，
    // 见 evidence/S4.3.log；fps 仅防回归）。
    assert!(stable, "桌面 fps 验收要求最近 2 个 1s 窗口平均 fps >= 40（vsync 60Hz 下限带）");
}

fn target_dir() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR = crates/ti-app → 上两级 = 仓库根
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("仓库根存在");
    root.to_path_buf()
}
