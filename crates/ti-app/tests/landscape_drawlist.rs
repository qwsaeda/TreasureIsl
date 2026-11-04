//! S38 方案 A：横屏世界层 draw list 验证（无头，不依赖 GPU readback）。
//!
//! `--shot` 对 `index` 与 `select_role` 的 `readback` 已知环境 GPU flake（仅 `vs`/`main_map` 可靠读回，
//! 见 `evidence/S38.0.log`），故横屏版面用**绘制列表级**断言验证：场景 `render()` 按当前
//! 方向（`Settings.orientation`）选 `scenes.json` 的 `landscape.primitives`（1920×1080 坐标），
//! 写入 draw list 的 Image/Rect/Text op 矩形 = 期望横屏坐标。
//!
//! 判据：
//! - index 横屏：`chart_full` 满屏 (0,0,1920,1080) + `helm` 屏心 (828,408,264,264)；
//! - `select_role` 横屏：`bg_night_sea` 满屏 (0,0,1920,1080) + 立绘面板 (60,110,560,700)；
//! - 竖屏对照：`chart_full` (0,0,1080,1920) + `helm` (408,838)（0 行为变化 = 竖屏原坐标）。

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use ti_app::game_app::GameApp;
use ti_platform::settings::Orientation;
use ti_render::drawlist::DrawOp;

fn game(landscape: bool) -> GameApp {
    let (w, h) = if landscape { (1920.0, 1080.0) } else { (1080.0, 1920.0) };
    let mut app = GameApp::new(w, h);
    app.set_record_draw(true);
    app.shell_mut().set_orientation(if landscape {
        Orientation::Landscape
    } else {
        Orientation::Portrait
    });
    app
}

/// 收集 draw list 中的 `Image` op 目标矩形（asset, x, y, w, h）。
fn image_rects(app: &GameApp) -> Vec<(String, i32, i32, i32, i32)> {
    app.draw_list()
        .ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::Image { asset, x, y, w, h, .. } => Some((asset.clone(), *x, *y, *w, *h)),
            _ => None,
        })
        .collect()
}

/// S38：`index` 横屏世界层（`chart_full` 满屏 + `helm` 屏心），竖屏对照（原坐标）。
#[test]
fn index_landscape_world_layer() {
    // 横屏
    let mut app = game(true);
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("chart_full.png") && *x == 0 && *y == 0 && *w == 1920 && *h == 1080
        }),
        "index 横屏 chart_full 应满屏 1920×1080，got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("helm.png") && *x == 828 && *y == 408 && *w == 264 && *h == 264
        }),
        "index 横屏 helm 应在屏心 (828,408)，got {rects:?}"
    );

    // 竖屏对照（0 行为变化）
    let mut app = game(false);
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("chart_full.png") && *x == 0 && *y == 0 && *w == 1080 && *h == 1920
        }),
        "index 竖屏 chart_full 应 1080×1920（0 行为变化），got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, _w, _h)| { a.ends_with("helm.png") && *x == 408 && *y == 838 }),
        "index 竖屏 helm 应在 (408,838)，got {rects:?}"
    );
}

/// S38：`select_role` 横屏世界层（bg 满屏 + 立绘面板左侧），竖屏对照。
#[test]
fn select_role_landscape_world_layer() {
    // 横屏
    let mut app = game(true);
    ti_app::forward_nav::navigate(&mut app, "select_role");
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1920 && *h == 1080
        }),
        "select_role 横屏 bg 应满屏 1920×1080，got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.contains("role_") && *x == 60 && *y == 110 && *w == 560 && *h == 700
        }),
        "select_role 横屏立绘应在左侧面板 (60,110,560,700)，got {rects:?}"
    );

    // 竖屏对照
    let mut app = game(false);
    ti_app::forward_nav::navigate(&mut app, "select_role");
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1080 && *h == 1920
        }),
        "select_role 竖屏 bg 应 1080×1920（0 行为变化），got {rects:?}"
    );
}

/// S38：`select_map` 横屏世界层（bg 满屏 + 3 张地图缩略图**横排**），竖屏对照（竖排原坐标）。
#[test]
fn select_map_landscape_world_layer() {
    // 横屏：3 张缩略图横排（left panel 20,140,1160,760 内 x=40/420/800，y=200）
    let mut app = game(true);
    ti_app::forward_nav::navigate(&mut app, "select_map");
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1920 && *h == 1080
        }),
        "select_map 横屏 bg 应满屏 1920×1080，got {rects:?}"
    );
    let expect: &[(&str, i32, i32)] =
        &[("map_harbour.png", 40, 200), ("map_open_sea.png", 420, 200), ("map_reef.png", 800, 200)];
    for (asset, ex, ey) in expect {
        assert!(
            rects.iter().any(|(a, x, y, w, h)| {
                a.ends_with(asset) && *x == *ex && *y == *ey && *w == 360 && *h == 280
            }),
            "select_map 横屏 {asset} 应在 ({ex},{ey},360,280)，got {rects:?}"
        );
    }

    // 竖屏对照（0 行为变化 = 竖排原坐标）
    let mut app = game(false);
    ti_app::forward_nav::navigate(&mut app, "select_map");
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1080 && *h == 1920
        }),
        "select_map 竖屏 bg 应 1080×1920（0 行为变化），got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("map_harbour.png") && *x == 96 && *y == 240 && *w == 400 && *h == 280
        }),
        "select_map 竖屏 map_harbour 应在 (96,240,400,280)，got {rects:?}"
    );
}
