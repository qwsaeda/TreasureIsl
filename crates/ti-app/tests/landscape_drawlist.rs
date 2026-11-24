//! S38 方案 A：横屏世界层 draw list 验证（无头，不依赖 GPU readback）。
//!
//! `--shot` 对 `index` 与 `select_role` 的 `readback` 已知环境 GPU flake（仅 `vs`/`main_map` 可靠读回，
//! 见 `evidence/S38.0.log`），故横屏版面用**绘制列表级**断言验证：场景 `render()` 按当前
//! 方向（`Settings.orientation`）选 `scenes.json` 的 `primitives`（1920×1080 坐标），
//! 写入 draw list 的 Image/Rect/Text op 矩形 = 期望横屏坐标。
//!
//! 判据：
//! - index 横屏：`chart_full` 满屏 (0,0,1920,1080) + `helm` 屏心 (828,408,264,264)；
//! - `select_role` 横屏：`bg_night_sea` 满屏 (0,0,1920,1080) + 立绘面板 (60,110,560,700)；

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use ti_app::game_app::GameApp;
use ti_platform::settings::Orientation;
use ti_render::drawlist::DrawOp;

fn game() -> GameApp {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.set_record_draw(true);
    app.shell_mut().set_orientation(Orientation::Landscape);
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

/// S39：`index` 横屏新版世界层 + UI 层组件（羊皮藏宝图底 + 人物互动层 + 绳挂木牌菜单 6 块）。
///
/// 判据（`scenes.json` `index` 原语）：
/// - `index_bg_map_h.png` 满屏 (0,0,1920,1080)（标题已烘焙进底图）；
/// - `index_hero_duo.png` 人物层 (511,324,679,660)（居中偏左）；
/// - `index_rope.png` 菜单主绳 (1446,138,434,28)；
/// - 6 块木牌（第 0 块未旋转 (1446,176,434,92)；第 1 块旋转 expand (1444,274,438,108)）。
#[test]
fn index_landscape_world_layer() {
    let mut app = game();
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
    // S81：木牌坐标已参数化（JSON rect 置 0 + derived=menu），真实 rect 由 Bevy 壳
    // index_menu_rect 运行期从 menu.* 算（发货路径）；CPU 后端不再持有 UI 层权威坐标
    // （bevy-cpu-parity 已排除 UI 层）⇒ 此处只验数据契约，不再断言 draw list 木牌死坐标。
    let scenes = ti_core::data::scenes().expect("scenes.json 加载");
    let index = scenes.get("index").expect("scenes.json 缺 index");
    assert!(
        index.extra_scalar.get("menu.x") == Some(&1446)
            && index.extra_scalar.get("menu.plank_rot_dx") == Some(&-2),
        "index 横屏 menu 参数缺失/漂移: {:?}",
        index.extra_scalar
    );
    let derived_planks: Vec<_> = index
        .primitives
        .iter()
        .filter(|p| p.params.get("derived").and_then(|v| v.as_str()) == Some("menu"))
        .collect();
    assert_eq!(derived_planks.len(), 18, "index 横屏 18 行木牌/图标应标 derived=menu");
}

/// S38：`select_role` 横屏世界层（bg 满屏 + 立绘面板左侧）。
#[test]
fn select_role_landscape_world_layer() {
    let mut app = game();
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
            a.contains("role_") && *x == 40 && *y == 90 && *w == 720 && *h == 810
        }),
        "select_role 横屏立绘应在左侧面板 (40,90,720,810，S63 T9 候选 A 放大)，got {rects:?}"
    );
}

/// S38：`select_map` 横屏世界层（bg 满屏 + 3 张地图缩略图**横排**）。
#[test]
fn select_map_landscape_world_layer() {
    // 3 张缩略图横排（left panel 20,140,1160,760 内 x=40/420/800，y=200）
    let mut app = game();
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
}

/// S38：`message` 横屏世界层（bg 满屏）。
/// 面板/导航 = `bevy_ui`（`build_message_chrome`），面板居中由 `bevy_ui` 适配。
#[test]
fn message_landscape_world_layer() {
    // bg 满屏 1920×1080
    let mut app = game();
    ti_app::forward_nav::navigate(&mut app, "message");
    app.render();
    let rects = image_rects(&app);
    assert!(
        rects.iter().any(|(a, x, y, w, h)| {
            a.ends_with("bg_night_sea.png") && *x == 0 && *y == 0 && *w == 1920 && *h == 1080
        }),
        "message 横屏 bg 应满屏 1920×1080，got {rects:?}"
    );
}

/// S38：`saveload` 横屏世界层（bg 满屏 + 5 按钮右侧竖排）。
/// 5 槽列表 = `bevy_ui`（`build_list_chrome_pos` `PanelSide::Left`），左侧自动重排。
#[test]
fn saveload_landscape_world_layer() {
    // bg 满屏 + slot1 按钮 (1680,125,180,150)
    let mut app = game();
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
}

/// S38：`vs` 横屏正式战斗（`render_v2_elem_landscape`：bg 框内 + 顶双 HP 框 + 角色对角）。
#[test]
fn vs_landscape_world_layer() {
    // navigate("vs") 有 battle_snapshot → render_v2_elem_landscape（phase=clash）
    let mut app = game();
    ti_app::forward_nav::navigate(&mut app, "vs");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| {
            a.contains("vs_v2/bg_") && *x == 44 && *y == 44 && *w == 1832 && *h == 992
        }),
        "vs 横屏 bg 应在羊皮框内 (44,44,1832,992)，got {imgs:?}"
    );
    // S63 候选 A：clash 帧 = hero 左中 (360,280) 480×680 + enemy 右中 (1080,280) + crew 居中前景 (820,560)
    assert!(
        imgs.iter().any(|(a, x, y, ..)| a.ends_with("crew.png") && *x == 820 && *y == 560),
        "vs 横屏 clash crew 应在 (820,560，S63 候选A居中前景)，got {imgs:?}"
    );
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| a.ends_with("hero.png")
            && *x == 360
            && *y == 280
            && *w == 480
            && *h == 680),
        "vs 横屏 clash hero 应在 (360,280,480,680，S63 候选A左中)，got {imgs:?}"
    );
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| a.ends_with("enemy.png")
            && *x == 1080
            && *y == 280
            && *w == 480
            && *h == 680),
        "vs 横屏 clash enemy 应在 (1080,280,480,680，S63 候选A右中)，got {imgs:?}"
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
}

/// S38：`main_map` v6 横屏（`render_v6_elem` 海图满屏 1920×1080，椭圆格环）。
/// 注：v6 默认路径先画世界层（`V6_OCEAN_BG` 满屏），故断言满屏 `image` 而非 `chart_full` 原语（z=2 <3）。
#[test]
fn main_map_v6_landscape_world_layer() {
    // v6 海图满屏 1920×1080（render_v6_elem V6_OCEAN_BG）
    let mut app = game();
    ti_app::forward_nav::navigate(&mut app, "main_map");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(_, x, y, w, h)| *x == 0 && *y == 0 && *w == 1920 && *h == 1080),
        "main_map 横屏 v6 海图应满屏 1920×1080，got {imgs:?}"
    );
}
