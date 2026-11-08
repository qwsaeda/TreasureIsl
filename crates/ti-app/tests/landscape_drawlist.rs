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

/// 收集 draw list 中的 `Rect` op（x, y, w, h）。
fn rect_rects(app: &GameApp) -> Vec<(i32, i32, i32, i32)> {
    app.draw_list()
        .ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::Rect { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .collect()
}

/// S39：`index` 横屏新版世界层 + UI 层组件（羊皮藏宝图底 + 人物互动层 + 绳挂木牌菜单 6 块），
/// 竖屏对照（S38.1 罗盘环，0 行为变化）。
///
/// 横屏判据（`scenes.json` `index.landscape` 原语）：
/// - `index_bg_map_h.png` 满屏 (0,0,1920,1080)（标题已烘焙进底图）；
/// - `index_hero_duo.png` 人物层 (511,324,679,660)（居中偏左）；
/// - `index_rope.png` 菜单主绳 (1446,138,434,28)；
/// - 6 块木牌（第 0 块未旋转 (1446,176,434,92)；第 1 块旋转 expand (1444,274,438,108)）。
#[test]
fn index_landscape_world_layer() {
    // 横屏
    let mut app = game(true);
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("index_bg_map_h.png") && *x == 0 && *y == 0 && *w == 1920 && *h == 1080
        }),
        "index 横屏 羊皮底图应满屏 1920×1080，got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("index_hero_duo.png") && *x == 511 && *y == 324 && *w == 679 && *h == 660
        }),
        "index 横屏 人物互动层应在 (511,324,679,660)，got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("index_rope.png") && *x == 1446 && *y == 138 && *w == 434 && *h == 28
        }),
        "index 横屏 菜单主绳应在 (1446,138,434,28)，got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("index_plank_0.png") && *x == 1446 && *y == 176 && *w == 434 && *h == 92
        }),
        "index 横屏 木牌 0 应在 (1446,176,434,92)，got {rects:?}"
    );
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("index_plank_1.png") && *x == 1444 && *y == 274 && *w == 438 && *h == 108
        }),
        "index 横屏 木牌 1 应在 (1444,274,438,108)（旋转 expand），got {rects:?}"
    );

    // 竖屏对照（0 行为变化 = S38.1 罗盘环仍由数据还原）
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
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("medallion_ring.png") && *x == 468 && *y == 498 && *w == 144 && *h == 144
        }),
        "index 竖屏 罗盘环徽章 0 应在 (468,498,144,144)（Rust 环退役后由数据还原），got {rects:?}"
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

/// S38：`message` 横屏世界层（bg 满屏），竖屏对照（0 行为变化）。
/// 面板/导航 = `bevy_ui`（`build_message_chrome`），横屏面板居中由 `bevy_ui` 适配。
#[test]
fn message_landscape_world_layer() {
    // 横屏：bg 满屏 1920×1080
    let mut app = game(true);
    ti_app::forward_nav::navigate(&mut app, "message");
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1920 && *h == 1080
        }),
        "message 横屏 bg 应满屏 1920×1080，got {rects:?}"
    );

    // 竖屏对照
    let mut app = game(false);
    ti_app::forward_nav::navigate(&mut app, "message");
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1080 && *h == 1920
        }),
        "message 竖屏 bg 应 1080×1920（0 行为变化），got {rects:?}"
    );
}

/// S38：`saveload` 横屏世界层（bg 满屏 + 5 按钮右侧竖排），竖屏对照（右侧按钮原坐标）。
/// 5 槽列表 = `bevy_ui`（`build_list_chrome_pos` `PanelSide::Left`），横屏左侧自动重排。
#[test]
fn saveload_landscape_world_layer() {
    // 横屏：bg 满屏 + slot1 按钮 (1680,125,180,150)
    let mut app = game(true);
    ti_app::forward_nav::navigate(&mut app, "saveload");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1920 && *h == 1080
        }),
        "saveload 横屏 bg 应满屏 1920×1080，got {imgs:?}"
    );
    let rects = rect_rects(&app);
    assert!(
        rects.iter().any(|(x, y, w, h)| *x == 1680 && *y == 125 && *w == 180 && *h == 150),
        "saveload 横屏 slot1 按钮应在 (1680,125,180,150)，got {rects:?}"
    );

    // 竖屏对照（0 行为变化）
    let mut app = game(false);
    ti_app::forward_nav::navigate(&mut app, "saveload");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1080 && *h == 1920
        }),
        "saveload 竖屏 bg 应 1080×1920（0 行为变化），got {imgs:?}"
    );
    let rects = rect_rects(&app);
    assert!(
        rects.iter().any(|(x, y, w, h)| *x == 760 && *y == 240 && *w == 160 && *h == 224),
        "saveload 竖屏 slot1 按钮应在 (760,240,160,224)，got {rects:?}"
    );
}

/// S38：`vs` 横屏正式战斗（`render_v2_elem_landscape`：bg 框内 + 顶双 HP 框 + 角色对角），
/// 竖屏对照（`render_v2_elem` 原式 0 行为变化）。
#[test]
fn vs_landscape_world_layer() {
    // 横屏：navigate("vs") 有 battle_snapshot → render_v2_elem_landscape（phase=clash）
    let mut app = game(true);
    ti_app::forward_nav::navigate(&mut app, "vs");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| {
            a.contains("vs_v2/bg_") && *x == 44 && *y == 44 && *w == 1832 && *h == 992
        }),
        "vs 横屏 bg 应在羊皮框内 (44,44,1832,992)，got {imgs:?}"
    );
    // clash 帧：crew 冲锋 + hero 左下 + enemy 右上（对角）
    assert!(
        imgs.iter().any(|(a, x, y, ..)| a.ends_with("crew.png") && *x == 120 && *y == 420),
        "vs 横屏 clash crew 应在 (120,420)，got {imgs:?}"
    );
    assert!(
        imgs.iter().any(|(a, x, y, ..)| a.ends_with("hero.png") && *x == 60 && *y == 600),
        "vs 横屏 clash hero 应在 (60,600)，got {imgs:?}"
    );
    let rects = rect_rects(&app);
    // 顶双 HP 状态框（左攻 64,40 / 右守 976,40，880×110）
    assert!(
        rects.iter().any(|(x, y, w, h)| *x == 64 && *y == 40 && *w == 880 && *h == 110),
        "vs 横屏 attacker HP 框应在 (64,40,880,110)，got {rects:?}"
    );
    assert!(
        rects.iter().any(|(x, y, w, h)| *x == 976 && *y == 40 && *w == 880 && *h == 110),
        "vs 横屏 defender HP 框应在 (976,40,880,110)，got {rects:?}"
    );

    // 竖屏对照（S49.4 v6 转默认：格子场景满屏 + 顶面板，非 v2 羊皮框 44,44）
    let mut app = game(false);
    ti_app::forward_nav::navigate(&mut app, "vs");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| {
            (a.contains("vs_v2/bg_") || a.contains("vs_v2/scene_"))
                && *x == 0
                && *y == 0
                && *w == 1080
                && *h == 1920
        }),
        "vs 竖屏 v6 格子场景 bg 应满屏 (0,0,1080,1920)，got {imgs:?}"
    );
    let rects = rect_rects(&app);
    assert!(
        rects.iter().any(|(x, y, w, h)| *x == 40 && *y == 50 && *w == 400 && *h == 200),
        "vs 竖屏 v6 顶面板应在 (40,50,400,200)，got {rects:?}"
    );
}

/// S38：`main_map` v6 横屏（`render_v6_elem` 海图满屏 1920×1080，椭圆格环），竖屏对照（0 行为变化）。
/// 注：v6 默认路径先画世界层（`V6_OCEAN_BG` 满屏），故断言满屏 `image` 而非 `chart_full` 原语（z=2 <3）。
#[test]
fn main_map_v6_landscape_world_layer() {
    // 横屏：v6 海图满屏 1920×1080（render_v6_elem V6_OCEAN_BG）
    let mut app = game(true);
    ti_app::forward_nav::navigate(&mut app, "main_map");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(_, x, y, w, h)| *x == 0 && *y == 0 && *w == 1920 && *h == 1080),
        "main_map 横屏 v6 海图应满屏 1920×1080，got {imgs:?}"
    );

    // 竖屏对照（0 行为变化）
    let mut app = game(false);
    ti_app::forward_nav::navigate(&mut app, "main_map");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(_, x, y, w, h)| *x == 0 && *y == 0 && *w == 1080 && *h == 1920),
        "main_map 竖屏 v6 海图应满屏 1080×1920（0 行为变化），got {imgs:?}"
    );
}
