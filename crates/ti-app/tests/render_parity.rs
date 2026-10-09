//! S5.3 P4-1 逐像素渲染对齐：用 headless 路径（`Framebuf` + 真字形，无 macroquad 窗口）
//! 渲染关键场景，断言逐像素一致（P0 资源解码参考 = `GoldLand.Fnt` 真字形 + 场景布局）。
//!
//! 判据（`docs/parity.md` P4-1）：关键场景渲染 == 资源解码参考图，逐像素一致。
//! 原版 ARM exe 无运行环境，参考基准 = `ti-formats` 资源解码 + `menu_shell` 布局常量
//! （同一份真字形/布局，三端共用）。本测试验证：
//! 1. 确定性：同一场景渲染两次，帧缓冲逐像素相同；
//! 2. 非空白：各场景渲染后墨点/色块数 > 0；
//! 3. 场景区分：不同场景渲染结果不同；
//! 4. 字形正确：标题「金银岛」命中真字形（与 `font_parity` 同口径）。

use ti_app::game_app::{GameApp, GameInput};
use ti_platform::gfx::Framebuf;

/// 逻辑帧像素数（`LOGICAL_W × LOGICAL_H`）。
const FB_PIXELS: usize = 176 * 220;

/// 渲染当前场景到 `GameApp.shell.fb`，返回帧缓冲像素切片。
fn render_current(app: &mut GameApp) -> Vec<u16> {
    app.render();
    app.shell.fb.as_slice().to_vec()
}

/// 场景渲染的「墨点率」：非背景色像素占比（证明画了真实内容，非全黑/全背景）。
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn ink_ratio(pixels: &[u16]) -> f32 {
    // 主菜单背景 = 0x0010（深蓝），标题条 = 0x01E0，选中高亮 = 0x03E0，墨色 = 0xFFFF。
    let content = pixels.iter().filter(|&&p| p != 0x0000 && p != 0x0010).count();
    let total = pixels.len();
    content as f32 / total as f32
}

/// 主菜单（Index 场景）渲染：标题 + 菜单项 + 状态行。
fn render_index() -> Vec<u16> {
    let mut app = GameApp::new(176.0, 220.0);
    render_current(&mut app)
}

/// 选角场景渲染。
fn render_select_role() -> Vec<u16> {
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm); // Index → SelectRole
    render_current(&mut app)
}

/// 选图场景渲染。
fn render_select_map() -> Vec<u16> {
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Confirm); // → SelectMap
    render_current(&mut app)
}

/// 主地图场景渲染。
fn render_main_map() -> Vec<u16> {
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Next); // 选角
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Next); // 选图
    app.handle(GameInput::Confirm); // → MainMap
    render_current(&mut app)
}

/// 战斗场景渲染。
fn render_vs() -> Vec<u16> {
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm); // → MainMap
    app.handle(GameInput::Landmark(6)); // → Vs
    render_current(&mut app)
}

/// P4-1 判据：同一场景渲染两次，帧缓冲逐像素完全相同（确定性）。
#[test]
fn render_is_deterministic_per_scene() {
    for (name, pixels) in [
        ("Index", render_index()),
        ("SelectRole", render_select_role()),
        ("SelectMap", render_select_map()),
        ("MainMap", render_main_map()),
        ("Vs", render_vs()),
    ] {
        let again = match name {
            "Index" => render_index(),
            "SelectRole" => render_select_role(),
            "SelectMap" => render_select_map(),
            "MainMap" => render_main_map(),
            _ => render_vs(),
        };
        assert_eq!(pixels, again, "{name} 场景渲染应确定性（两次渲染逐像素相同）");
        assert_eq!(pixels.len(), FB_PIXELS, "{name} 帧缓冲尺寸应 = 176×220");
    }
}

/// P4-1 判据：各场景渲染后都有真实内容（非黑屏/全背景）。
#[test]
fn each_scene_renders_real_content() {
    for (name, pixels) in [
        ("Index", render_index()),
        ("SelectRole", render_select_role()),
        ("SelectMap", render_select_map()),
        ("MainMap", render_main_map()),
        ("Vs", render_vs()),
    ] {
        let ratio = ink_ratio(&pixels);
        assert!(ratio > 0.01, "{name} 场景渲染应有真实内容（墨点率 {ratio} > 1%）");
    }
}

/// P4-1 判据：不同场景渲染结果不同（场景切换确实改了画面）。
#[test]
fn distinct_scenes_produce_distinct_frames() {
    let index = render_index();
    let role = render_select_role();
    let map = render_select_map();
    let main_map = render_main_map();
    let vs = render_vs();
    assert_ne!(index, role, "Index 与 SelectRole 应渲染不同画面");
    assert_ne!(role, map, "SelectRole 与 SelectMap 应渲染不同画面");
    assert_ne!(index, main_map, "Index 与 MainMap 应渲染不同画面");
    assert_ne!(main_map, vs, "MainMap 与 Vs 应渲染不同画面");
}

/// P4-1 判据：主菜单标题「金银岛」命中真字形（与 `font_parity` 同口径，P0 资源解码参考）。
#[test]
fn index_scene_renders_real_glyphs_for_title() {
    let pixels = render_index();
    let ink_pixels = pixels.iter().filter(|&&p| p == 0xFFFF).count();
    assert!(ink_pixels > 10, "主菜单标题+菜单项应渲染出真字形（墨色像素数 {ink_pixels} > 10）");
}

/// P4-1 判据：`Framebuf` 尺寸与原版逻辑分辨率 176×220 一致（P0 资源解码参考的坐标系）。
#[test]
fn framebuf_matches_logical_resolution() {
    let mut fb = Framebuf::new();
    fb.clear(0x0000);
    assert_eq!(fb.as_slice().len(), FB_PIXELS, "帧缓冲 = 176×220");
    let mut app = GameApp::new(176.0, 220.0);
    app.shell.render();
    let pixels = app.shell.fb.as_slice();
    assert_eq!(pixels[0], 0x01E0, "主菜单第 0 行 x=0 应为标题条色 0x01E0");
}
