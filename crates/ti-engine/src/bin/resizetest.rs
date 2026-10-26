//! 2026-10-26 窗口缩放跟随（用户报告「桌面版不随窗口缩放」）的**机器判定**模式。
//!
//! `--resizetest` 启动真窗口 → 读初值相机正交 scale（s1）→ 程序化把窗口物理分辨率翻倍 →
//! 读回相机 scale（s2）→ 断言 `s2 ≈ 2·s1`（`window_resize_system` 重建 `vp` + `camera_system`
//! 读 `vp.scale()` 链路生效）。PASS = 退出码 0，FAIL = 非 0（与 `--selftest` 同判据口径）。
//!
//! 从 `treasure_bevy.rs` 拆出（该文件有 900 行棘轮上限，`audit_code` 对账）；本模块只含
//! `--resizetest` 的资源 + 系统，经 `mod resizetest;` 挂进壳。

use bevy::camera::{Camera2d, Projection};
use bevy::prelude::*;
use bevy::window::Window;

#[derive(Resource)]
pub struct ResizeTest {
    /// 已运行帧（门控用，容忍首帧 shader 编译）。
    pub frame: u32,
    /// 初值相机正交 scale。
    pub s1: f32,
    /// 翻倍后相机正交 scale。
    pub s2: f32,
    /// 是否已触发 resize。
    pub resized: bool,
    /// 是否已判定（防重复发 `AppExit`）。
    pub done: bool,
}

/// `--resizetest` 系统：帧 20 记 s1，帧 25 翻倍窗口，帧 45 记 s2 并判定退出。
#[allow(clippy::needless_pass_by_value)]
pub fn resizetest_system(
    mut test: ResMut<ResizeTest>,
    mut windows: Query<&mut Window>,
    cam: Query<&Projection, With<Camera2d>>,
    mut exits: MessageWriter<AppExit>,
) {
    test.frame = test.frame.saturating_add(1);
    let cscale = |c: &Projection| match c {
        Projection::Orthographic(o) => o.scale,
        _ => 0.0,
    };
    match test.frame {
        20 => {
            if let Ok(c) = cam.single() {
                test.s1 = cscale(c);
            }
        }
        25 => {
            if let Ok(mut win) = windows.single_mut() {
                // 物理分辨率翻倍（逻辑尺寸随 scale_factor 同比，判定与 DPI 无关）。
                let (w, h) = (win.physical_width() * 2, win.physical_height() * 2);
                win.resolution.set_physical_resolution(w, h);
                test.resized = true;
            }
        }
        45 => {
            if let Ok(c) = cam.single() {
                test.s2 = cscale(c);
            }
            test.done = true;
            // s1>0（初值有效）且翻倍后 scale ≈ 2×（±10%）。
            let pass = test.s1 > 0.0 && test.s2 > test.s1 * 1.8 && test.s2 < test.s1 * 2.2;
            if pass {
                println!(
                    "[resizetest] PASS (s1={:.3} -> s2={:.3}; camera ortho scale doubled with window)",
                    test.s1, test.s2
                );
                exits.write(AppExit::Success);
            } else {
                eprintln!(
                    "[resizetest] FAIL (s1={:.3} s2={:.3} resized={}; window resize did not drive camera)",
                    test.s1, test.s2, test.resized
                );
                exits.write(AppExit::error());
            }
        }
        _ => {}
    }
}
