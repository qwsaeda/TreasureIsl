//! 2026-10-26 窗口缩放跟随（用户报告「桌面版不随窗口缩放」）的**机器判定**模式。
//!
//! `--resizetest` 启动真窗口 → 读初值**实际可见世界区域**（Bevy 0.20
//! `OrthographicProjection.area`，pub 字段）→ 程序化把窗口物理分辨率翻倍 → 再读一次 →
//! 断言两条不变量：
//!
//! 1. **画布恒完整可见**：`area ≈ 1080×1920`（letterbox，内容不裁切）；
//! 2. **内容随窗口等比放大**：`px/unit = 窗口逻辑宽 / area 宽` 翻倍
//!    （窗口变大 ⇒ 1 逻辑单位占更多物理像素 ⇒ 画面跟着缩放）。
//!
//! 判据取 `area`（渲染真值）而非 `o.scale`（内部量），故对 `o.scale` 口径写反**必然 FAIL**
//! （EXP-115：曾把 `o.scale` 误取为 `vp.scale()`，540×960 下 `area=270×480` = 4× 放大）。
//!
//! PASS = 退出码 0，FAIL = 非 0（与 `--selftest` 同判据口径）。
//! 从 `treasure_bevy.rs` 拆出（该文件有 900 行棘轮上限，`audit_code` 对账）；本模块只含
//! `--resizetest` 的资源 + 系统，经 `mod resizetest;` 挂进壳。

use bevy::camera::{Camera2d, Projection};
use bevy::prelude::*;
use bevy::window::Window;

use super::{camera_system, LOGICAL_H, LOGICAL_W};

/// 接线：插入判据资源 + 判据系统（`main` 只留一行调用，避免 `main()` 触
/// clippy `too_many_lines`）。
pub fn attach_resizetest(app: &mut App) {
    app.insert_resource(ResizeTest {
        frame: 0,
        area1: [0.0, 0.0],
        area2: [0.0, 0.0],
        win1: [0.0, 0.0],
        win2: [0.0, 0.0],
        resized: false,
        done: false,
    })
    .add_systems(Update, resizetest_system.after(camera_system));
}

#[derive(Resource)]
pub struct ResizeTest {
    /// 已运行帧（门控用，容忍首帧 shader 编译）。
    pub frame: u32,
    /// 初值可见世界区域（宽, 高）。
    pub area1: [f32; 2],
    /// 窗口翻倍后可见世界区域（宽, 高）。
    pub area2: [f32; 2],
    /// 初值窗口逻辑尺寸（宽, 高）。
    pub win1: [f32; 2],
    /// 窗口翻倍后逻辑尺寸（宽, 高）。
    pub win2: [f32; 2],
    /// 是否已触发 resize。
    pub resized: bool,
    /// 是否已判定（防重复发 `AppExit`）。
    pub done: bool,
}

/// `--resizetest` 系统：帧 20 记初值，帧 25 翻倍窗口，帧 45 记后值并判定退出。
#[allow(clippy::needless_pass_by_value)]
pub fn resizetest_system(
    mut test: ResMut<ResizeTest>,
    mut windows: Query<&mut Window>,
    cam: Query<&Projection, With<Camera2d>>,
    mut exits: MessageWriter<AppExit>,
) {
    test.frame = test.frame.saturating_add(1);
    let area_of = |c: &Projection| match c {
        Projection::Orthographic(o) => [o.area.max.x - o.area.min.x, o.area.max.y - o.area.min.y],
        _ => [0.0, 0.0],
    };
    match test.frame {
        20 => {
            if let Ok(c) = cam.single() {
                test.area1 = area_of(c);
            }
            if let Ok(win) = windows.single() {
                test.win1 = [win.width(), win.height()];
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
                test.area2 = area_of(c);
            }
            if let Ok(win) = windows.single() {
                test.win2 = [win.width(), win.height()];
            }
            test.done = true;
            let canvas = [LOGICAL_W as f32, LOGICAL_H as f32];
            let near = |a: f32, b: f32| (a - b).abs() <= b * 0.02;
            // 不变量 1：画布恒完整可见（可见世界 ≈ 1080×1920）。
            let fit1 = near(test.area1[0], canvas[0]) && near(test.area1[1], canvas[1]);
            let fit2 = near(test.area2[0], canvas[0]) && near(test.area2[1], canvas[1]);
            // 不变量 2：内容随窗口等比放大（px/unit = 窗口逻辑宽 / 可见世界宽，翻倍）。
            let ppu1 = if test.area1[0] > 0.0 {
                test.win1[0] / test.area1[0]
            } else {
                0.0
            };
            let ppu2 = if test.area2[0] > 0.0 {
                test.win2[0] / test.area2[0]
            } else {
                0.0
            };
            let scaled = ppu1 > 0.0 && ppu2 > ppu1 * 1.8 && ppu2 < ppu1 * 2.2;
            let pass = fit1 && fit2 && scaled;
            if pass {
                println!(
                    "[resizetest] PASS (area {:.0}x{:.0} -> {:.0}x{:.0}; px/unit {:.3} -> {:.3}; canvas fully visible and content scales with window)",
                    test.area1[0],
                    test.area1[1],
                    test.area2[0],
                    test.area2[1],
                    ppu1,
                    ppu2
                );
                exits.write(AppExit::Success);
            } else {
                eprintln!(
                    "[resizetest] FAIL (fit1={fit1} fit2={fit2} scaled={scaled}; area {:.0}x{:.0} -> {:.0}x{:.0}; win {:.0}x{:.0} -> {:.0}x{:.0}; expected area=1080x1920 and px/unit doubled)",
                    test.area1[0],
                    test.area1[1],
                    test.area2[0],
                    test.area2[1],
                    test.win1[0],
                    test.win1[1],
                    test.win2[0],
                    test.win2[1]
                );
                exits.write(AppExit::error());
            }
        }
        _ => {}
    }
}
