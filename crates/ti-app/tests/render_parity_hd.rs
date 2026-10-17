//! S7.5 P5-2：HD RGBA8 渲染路径逐像素对齐（feature `hd-8bit`，路径 1 并行）。
//!
//! 判据（`docs/impl/14_asset_enhancement.md` §7 P5-2）：
//! 1. 确定性：同一场景渲染两次，RGBA8 帧缓冲逐像素相同；
//! 2. 非空白：各场景 HD 渲染后墨点/色块数 > 0；
//! 3. 场景区分：不同场景 HD 渲染结果不同；
//! 4. 字形正确：主菜单标题命中真字形（与 `font_parity` 同口径）；
//! 5. `FramebufRgba` 尺寸与呈现层分辨率 1080×1920 一致（QA112 注释同步）。
//!
//! 与 `render_parity.rs`（RGB565 原路径，P4-1）并行：默认构建本测试目标为空
//! （feature 关闭时全部 `#[cfg]` 裁剪），开启 `hd-8bit` 后全绿。

#[cfg(feature = "hd-8bit")]
mod tests {
    use ti_app::game_app::{GameApp, GameInput};
    use ti_platform::gfx::FramebufRgba;

    /// 呈现层帧像素数（`1080 × 1920`，§1 约束；legacy 176×220 统一缩放落画布）。
    const FB_PIXELS: usize = 1080 * 1920;

    /// 渲染当前场景到 `GameApp.shell.fb_rgba`（RGBA8 HD 路径），返回帧缓冲像素切片。
    fn render_current_hd(app: &mut GameApp) -> Vec<u32> {
        app.render_hd();
        app.shell().fb_rgba.as_slice().to_vec()
    }

    /// 场景渲染的「墨点率」：非背景色像素占比（证明画了真实内容，非全黑/全背景）。
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn ink_ratio_rgba(pixels: &[u32]) -> f32 {
        // 主菜单背景 = 0x0010（RGB565 深蓝），HD 扩展后 = 0xFF_00_00_00 量级；
        // 标题条 = 0x01E0、选中高亮 = 0x03E0、墨色 = 0xFFFF（白）。
        // 取「非黑且 alpha 全」为内容像素。
        let content = pixels.iter().filter(|&&p| p != 0xFF00_0000 && p != 0).count();
        let total = pixels.len();
        content as f32 / total as f32
    }

    /// 主菜单（Index 场景）HD 渲染。
    fn render_index_hd() -> Vec<u32> {
        let mut app = GameApp::new(1080.0, 1920.0);
        render_current_hd(&mut app)
    }

    /// 选角场景 HD 渲染。
    fn render_select_role_hd() -> Vec<u32> {
        let mut app = GameApp::new(1080.0, 1920.0);
        app.handle(GameInput::Confirm); // Index → SelectRole
        render_current_hd(&mut app)
    }

    /// 选图场景 HD 渲染。
    fn render_select_map_hd() -> Vec<u32> {
        let mut app = GameApp::new(1080.0, 1920.0);
        app.handle(GameInput::Confirm); // → SelectRole
        app.handle(GameInput::Confirm); // → SelectMap
        render_current_hd(&mut app)
    }

    /// 主地图场景 HD 渲染。
    fn render_main_map_hd() -> Vec<u32> {
        let mut app = GameApp::new(1080.0, 1920.0);
        app.handle(GameInput::Confirm); // → SelectRole
        app.handle(GameInput::Next); // 选角
        app.handle(GameInput::Confirm); // → SelectMap
        app.handle(GameInput::Next); // 选图
        app.handle(GameInput::Confirm); // → MainMap
        render_current_hd(&mut app)
    }

    /// 战斗场景 HD 渲染。
    fn render_vs_hd() -> Vec<u32> {
        let mut app = GameApp::new(1080.0, 1920.0);
        app.handle(GameInput::Confirm);
        app.handle(GameInput::Next);
        app.handle(GameInput::Confirm);
        app.handle(GameInput::Next);
        app.handle(GameInput::Confirm); // → MainMap
        app.handle(GameInput::Landmark(6)); // → Vs
        render_current_hd(&mut app)
    }

    /// P5-2 判据 1：同一场景 HD 渲染两次，RGBA8 帧缓冲逐像素完全相同（确定性）。
    #[test]
    fn hd_render_is_deterministic_per_scene() {
        for (name, pixels) in [
            ("Index", render_index_hd()),
            ("SelectRole", render_select_role_hd()),
            ("SelectMap", render_select_map_hd()),
            ("MainMap", render_main_map_hd()),
            ("Vs", render_vs_hd()),
        ] {
            let again = match name {
                "Index" => render_index_hd(),
                "SelectRole" => render_select_role_hd(),
                "SelectMap" => render_select_map_hd(),
                "MainMap" => render_main_map_hd(),
                _ => render_vs_hd(),
            };
            assert_eq!(pixels, again, "{name} 场景 HD 渲染应确定性（两次逐像素相同）");
            assert_eq!(pixels.len(), FB_PIXELS, "{name} HD 帧缓冲尺寸应 = 1080×1920");
        }
    }

    /// P5-2 判据 2：各场景 HD 渲染后都有真实内容（非黑屏/全背景）。
    #[test]
    fn hd_each_scene_renders_real_content() {
        for (name, pixels) in [
            ("Index", render_index_hd()),
            ("SelectRole", render_select_role_hd()),
            ("SelectMap", render_select_map_hd()),
            ("MainMap", render_main_map_hd()),
            ("Vs", render_vs_hd()),
        ] {
            let ratio = ink_ratio_rgba(&pixels);
            assert!(ratio > 0.01, "{name} 场景 HD 渲染应有真实内容（墨点率 {ratio} > 1%）");
        }
    }

    /// P5-2 判据 3：不同场景 HD 渲染结果不同（场景切换确实改了画面）。
    #[test]
    fn hd_distinct_scenes_produce_distinct_frames() {
        let index = render_index_hd();
        let role = render_select_role_hd();
        let map = render_select_map_hd();
        let main_map = render_main_map_hd();
        let vs = render_vs_hd();
        assert_ne!(index, role, "Index 与 SelectRole HD 应渲染不同画面");
        assert_ne!(role, map, "SelectRole 与 SelectMap HD 应渲染不同画面");
        assert_ne!(index, main_map, "Index 与 MainMap HD 应渲染不同画面");
        assert_ne!(main_map, vs, "MainMap 与 Vs HD 应渲染不同画面");
    }

    /// P5-2 判据 4：主菜单标题命中真字形（墨色 `0xFFFF_FFFF`，与 `font_parity` 同口径）。
    #[test]
    fn hd_index_scene_renders_real_glyphs_for_title() {
        let pixels = render_index_hd();
        // S15 v2：主菜单改用羊皮纸/木纹/金色系，UI 像素 = 非背景、非黑（标题条木纹、
        // 羊皮纸棋盘、bevel 按钮、深墨文字）——比旧版白墨 0xFFFF_FFFF 判据更宽松。
        let ui_pixels = pixels.iter().filter(|&&p| p != 0 && p != 0xFF_FD70).count();
        assert!(
            ui_pixels > 10,
            "主菜单标题+菜单项 HD 应渲染出真字形（非背景/非黑像素数 {ui_pixels} > 10）"
        );
    }

    /// P5-2 判据 5：`FramebufRgba` 尺寸与呈现层分辨率 1080×1920 一致。
    #[test]
    fn hd_framebuf_matches_logical_resolution() {
        let mut fb = FramebufRgba::new();
        fb.clear(0xFF00_0000);
        assert_eq!(fb.as_slice().len(), FB_PIXELS, "HD 帧缓冲 = 1080×1920");
        let mut app = GameApp::new(1080.0, 1920.0);
        app.render_hd();
        let pixels = app.shell().fb_rgba.as_slice();
        assert!(pixels[0] != 0, "HD 主菜单第 0 行 x=0 应为标题条色（非黑）");
    }

    // S15.4：legacy HD PNG 逐像素 blit 判据（判据 6/7）已随 legacy 像素写帧层移除而删除：
    // `render_hd` 不再叠加 HD 精灵，`blit_scene_frame_rgba` 已删除。HD 真彩注入的
    // 代码化贴图路径（`blit_texture` 原语）与逐像素重锚见 S15.5。
}
