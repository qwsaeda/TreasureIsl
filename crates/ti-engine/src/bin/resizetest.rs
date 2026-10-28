//! 2026-10-26 窗口缩放跟随（用户报告「桌面版不随窗口缩放」）的**机器判定**模式。
//!
//! `--resizetest` 启动真窗口 → 读初值**实际可见世界区域**（Bevy 0.20
//! `OrthographicProjection.area`，pub 字段）→ 程序化把窗口物理分辨率翻倍 → 再读一次 →
//! 断言两条不变量：
//!
//! 1. **画布恒完整可见**：`area ≈ 1080×1920`（letterbox，内容不裁切）；
//! 2. **内容随窗口等比放大**：`px/unit = 窗口逻辑宽 / area 宽` 翻倍
//!    （窗口变大 ⇒ 1 逻辑单位占更多物理像素 ⇒ 画面跟着缩放）。
//! 3. **（S29.5）`bevy_ui` chrome 真重排**：UI 布局根 `ComputedNode.size` = 视口尺寸
//!    （初值 = win1、翻倍后 = win2）⇒ taffy 随新视口重算（不是位图缩放）；
//!    且布局树节点数不变（只重排，不重建）。
//!
//! 判据取 `area`（渲染真值）而非 `o.scale`（内部量），故对 `o.scale` 口径写反**必然 FAIL**
//! （EXP-115：曾把 `o.scale` 误取为 `vp.scale()`，540×960 下 `area=270×480` = 4× 放大）。
//!
//! PASS = 退出码 0，FAIL = 非 0（与 `--selftest` 同判据口径）。
//! 从 `treasure_bevy.rs` 拆出（该文件有 900 行棘轮上限，`audit_code` 对账）；本模块只含
//! `--resizetest` 的资源 + 系统，经 `mod resizetest;` 挂进壳。

use bevy::camera::Projection;
use bevy::prelude::*;
use bevy::ui::ComputedNode;
use bevy::window::Window;

use super::{camera_system, GameStore, WorldCam, LOGICAL_H, LOGICAL_W};

/// 接线：插入判据资源 + 判据系统（`main` 只留一行调用，避免 `main()` 触
/// clippy `too_many_lines`）。
pub fn attach_resizetest(app: &mut App) {
    app.insert_resource(ResizeTest {
        frame: 0,
        area1: [0.0, 0.0],
        area2: [0.0, 0.0],
        win1: [0.0, 0.0],
        win2: [0.0, 0.0],
        ui1: [0.0, 0.0],
        ui2: [0.0, 0.0],
        vp1: 0.0,
        vp2: 0.0,
        ui_nodes1: 0,
        ui_nodes2: 0,
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
    /// 初值 `bevy_ui` 布局根尺寸（宽, 高；= 视口尺寸 ⇒ UI 层真重排判据）。
    pub ui1: [f32; 2],
    /// 窗口翻倍后 `bevy_ui` 布局根尺寸。
    pub ui2: [f32; 2],
    /// 初值 `bevy_ui` 节点数（布局树规模）。
    pub ui_nodes1: usize,
    /// 窗口翻倍后 `bevy_ui` 节点数（断言 = 规模不变，只重排不重建）。
    pub ui_nodes2: usize,
    /// 壳视口比例（px/逻辑单位）初值 / 翻倍后（诊断 `vp` 是否跟随窗口）。
    pub vp1: f32,
    pub vp2: f32,
    /// 是否已触发 resize。
    pub resized: bool,
    /// 是否已判定（防重复发 `AppExit`）。
    pub done: bool,
}

/// `bevy_ui` 布局根尺寸（`ComputedNode.size` 最大者 = 全屏根 = 视口）+ 节点数。
fn ui_layout_size(nodes: &Query<&ComputedNode>) -> ([f32; 2], usize) {
    let mut max_w = 0.0f32;
    let mut max_h = 0.0f32;
    let mut count = 0usize;
    for cn in nodes.iter() {
        count += 1;
        max_w = max_w.max(cn.size.x);
        max_h = max_h.max(cn.size.y);
    }
    ([max_w, max_h], count)
}

/// `--resizetest` 系统：帧 20 记初值，帧 25 翻倍窗口，帧 45 记后值并判定退出。
#[allow(clippy::needless_pass_by_value)]
pub fn resizetest_system(
    mut test: ResMut<ResizeTest>,
    mut windows: Query<&mut Window>,
    // S29.5：只取**游戏世界相机**（`WorldCam` 标记；UI 相机与离屏截图相机都不参与）。
    cam: Query<&Projection, With<WorldCam>>,
    ui_nodes: Query<&ComputedNode>,
    store: Res<GameStore>,
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
            let (size, count) = ui_layout_size(&ui_nodes);
            test.ui1 = size;
            test.ui_nodes1 = count;
            if let Ok(g) = store.0.lock() {
                test.vp1 = g.shell().vp.scale();
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
            let (size, count) = ui_layout_size(&ui_nodes);
            test.ui2 = size;
            test.ui_nodes2 = count;
            if let Ok(g) = store.0.lock() {
                test.vp2 = g.shell().vp.scale();
            }
            test.done = true;
            let (fit1, fit2, scaled, ui_fits, ui_reflow, ui_same_tree, ppu1, ppu2) = verdict(&test);
            let pass = fit1 && fit2 && scaled && ui_fits && ui_reflow && ui_same_tree;
            if pass {
                println!(
                    "[resizetest] PASS (area {:.0}x{:.0} -> {:.0}x{:.0}; px/unit {:.3} -> {:.3}; \
                     bevy_ui layout root {:.0}x{:.0} -> {:.0}x{:.0} = viewport; ui nodes {} -> {}; vp {:.3} -> {:.3})",
                    test.area1[0],
                    test.area1[1],
                    test.area2[0],
                    test.area2[1],
                    ppu1,
                    ppu2,
                    test.ui1[0],
                    test.ui1[1],
                    test.ui2[0],
                    test.ui2[1],
                    test.ui_nodes1,
                    test.ui_nodes2,
                    test.vp1,
                    test.vp2
                );
                exits.write(AppExit::Success);
            } else {
                eprintln!(
                    "[resizetest] FAIL (fit1={fit1} fit2={fit2} scaled={scaled} ui_fits={ui_fits} \
                     ui_reflow={ui_reflow} ui_same_tree={ui_same_tree}; area {:.0}x{:.0} -> {:.0}x{:.0}; \
                     win {:.0}x{:.0} -> {:.0}x{:.0}; ui root {:.0}x{:.0} -> {:.0}x{:.0}; ui nodes {} -> {}; vp {:.3} -> {:.3})",
                    test.area1[0],
                    test.area1[1],
                    test.area2[0],
                    test.area2[1],
                    test.win1[0],
                    test.win1[1],
                    test.win2[0],
                    test.win2[1],
                    test.ui1[0],
                    test.ui1[1],
                    test.ui2[0],
                    test.ui2[1],
                    test.ui_nodes1,
                    test.ui_nodes2,
                    test.vp1,
                    test.vp2
                );
                exits.write(AppExit::error());
            }
        }
        _ => {}
    }
}

/// 三不变量判定（见模块文档）：①画布恒完整可见 ②内容等比放大 ③`bevy_ui` chrome 真重排。
fn verdict(test: &ResizeTest) -> (bool, bool, bool, bool, bool, bool, f32, f32) {
    let canvas = [LOGICAL_W as f32, LOGICAL_H as f32];
    let near = |a: f32, b: f32| (a - b).abs() <= b * 0.02;
    let fit1 = near(test.area1[0], canvas[0]) && near(test.area1[1], canvas[1]);
    let fit2 = near(test.area2[0], canvas[0]) && near(test.area2[1], canvas[1]);
    let ppu1 = if test.area1[0] > 0.0 { test.win1[0] / test.area1[0] } else { 0.0 };
    let ppu2 = if test.area2[0] > 0.0 { test.win2[0] / test.area2[0] } else { 0.0 };
    let scaled = ppu1 > 0.0 && ppu2 > ppu1 * 1.8 && ppu2 < ppu1 * 2.2;
    let ui_fits = near(test.ui1[0], test.win1[0])
        && near(test.ui1[1], test.win1[1])
        && near(test.ui2[0], test.win2[0])
        && near(test.ui2[1], test.win2[1]);
    let ui_reflow = test.ui1[0] > 0.0 && test.ui2[0] > test.ui1[0] * 1.8;
    let ui_same_tree = test.ui_nodes1 > 0 && test.ui_nodes1 == test.ui_nodes2;
    (fit1, fit2, scaled, ui_fits, ui_reflow, ui_same_tree, ppu1, ppu2)
}
