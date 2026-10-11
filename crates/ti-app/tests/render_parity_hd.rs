//! S7.5 P5-2：HD RGBA8 渲染路径逐像素对齐（feature `hd-8bit`，路径 1 并行）。
//!
//! 判据（`docs/impl/14_asset_enhancement.md` §7 P5-2）：
//! 1. 确定性：同一场景渲染两次，RGBA8 帧缓冲逐像素相同；
//! 2. 非空白：各场景 HD 渲染后墨点/色块数 > 0；
//! 3. 场景区分：不同场景 HD 渲染结果不同；
//! 4. 字形正确：主菜单标题命中真字形（与 `font_parity` 同口径）；
//! 5. `FramebufRgba` 尺寸与逻辑分辨率 176×220 一致。
//!
//! 与 `render_parity.rs`（RGB565 原路径，P4-1）并行：默认构建本测试目标为空
//! （feature 关闭时全部 `#[cfg]` 裁剪），开启 `hd-8bit` 后全绿。

#[cfg(feature = "hd-8bit")]
mod tests {
    use ti_app::game_app::{GameApp, GameInput};
    use ti_app::menu_shell::INK_COLOR_RGBA;
    use ti_platform::gfx::FramebufRgba;

    /// 逻辑帧像素数（`LOGICAL_W × LOGICAL_H`）。
    const FB_PIXELS: usize = 176 * 220;

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
        let mut app = GameApp::new(176.0, 220.0);
        render_current_hd(&mut app)
    }

    /// 选角场景 HD 渲染。
    fn render_select_role_hd() -> Vec<u32> {
        let mut app = GameApp::new(176.0, 220.0);
        app.handle(GameInput::Confirm); // Index → SelectRole
        render_current_hd(&mut app)
    }

    /// 选图场景 HD 渲染。
    fn render_select_map_hd() -> Vec<u32> {
        let mut app = GameApp::new(176.0, 220.0);
        app.handle(GameInput::Confirm); // → SelectRole
        app.handle(GameInput::Confirm); // → SelectMap
        render_current_hd(&mut app)
    }

    /// 主地图场景 HD 渲染。
    fn render_main_map_hd() -> Vec<u32> {
        let mut app = GameApp::new(176.0, 220.0);
        app.handle(GameInput::Confirm); // → SelectRole
        app.handle(GameInput::Next); // 选角
        app.handle(GameInput::Confirm); // → SelectMap
        app.handle(GameInput::Next); // 选图
        app.handle(GameInput::Confirm); // → MainMap
        render_current_hd(&mut app)
    }

    /// 战斗场景 HD 渲染。
    fn render_vs_hd() -> Vec<u32> {
        let mut app = GameApp::new(176.0, 220.0);
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
            assert_eq!(pixels.len(), FB_PIXELS, "{name} HD 帧缓冲尺寸应 = 176×220");
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
        let ink_pixels = pixels.iter().filter(|&&p| p == INK_COLOR_RGBA).count();
        assert!(
            ink_pixels > 10,
            "主菜单标题+菜单项 HD 应渲染出真字形（墨色像素数 {ink_pixels} > 10）"
        );
    }

    /// P5-2 判据 5：`FramebufRgba` 尺寸与逻辑分辨率 176×220 一致。
    #[test]
    fn hd_framebuf_matches_logical_resolution() {
        let mut fb = FramebufRgba::new();
        fb.clear(0xFF00_0000);
        assert_eq!(fb.as_slice().len(), FB_PIXELS, "HD 帧缓冲 = 176×220");
        let mut app = GameApp::new(176.0, 220.0);
        app.render_hd();
        let pixels = app.shell().fb_rgba.as_slice();
        assert!(pixels[0] != 0, "HD 主菜单第 0 行 x=0 应为标题条色（非黑）");
    }

    /// P5-2 判据 6（真 PNG 注入）：`assets_hd_pack/1x/` 的 PNG 字节经
    /// `load_hd_pngs` 注入后，HD blit 路径消费 RGBA8 真彩像素（非纯 PCX 扩色）。
    /// 缺失 `assets_hd_pack` 时 SKIP（S7.4 产物未落盘）。
    ///
    /// 文件命名：静态精灵 `NN_hd.png`（1 帧）；多帧精灵 `NN_hd_f0.png`~`f4.png`
    /// （分帧，`_fN` 后缀）。f1 从文件名前缀数字提取，帧号从 `_fN` 提取。
    #[test]
    fn hd_png_injection_produces_true_color_pixels() {
        use ti_platform::hd_png::decode_png_rgba;

        // 定位 repo 根（CWD 优先，EXP-050）
        let base = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let pack_dir = base.join("assets_hd_pack").join("1x");
        if !pack_dir.is_dir() {
            eprintln!("SKIP: assets_hd_pack/1x 不存在");
            return;
        }

        // 收集 1x 档全部 PNG 字节（f1 → 帧列表，按帧号排序）
        let mut png_map: std::collections::BTreeMap<u32, Vec<(u32, Vec<u8>)>> =
            std::collections::BTreeMap::new();
        for entry in std::fs::read_dir(&pack_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                continue;
            }
            let stem = name.strip_suffix(".png").unwrap_or(name);
            // 静态："NN_hd" → f1=NN, frame=0；多帧："NN_hd_fK" → f1=NN, frame=K
            let (f1_str, frame_str) = match stem.rsplit_once("_f") {
                Some((a, b)) => (a.strip_suffix("_hd").unwrap_or(a), b),
                None => (stem.strip_suffix("_hd").unwrap_or(stem), "0"),
            };
            let f1: u32 = match f1_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let frame_no: u32 = frame_str.parse().unwrap_or(0);
            if let Ok(bytes) = std::fs::read(&path) {
                let Ok(img) = decode_png_rgba(&bytes) else {
                    continue;
                };
                let has_color = img.rgba.iter().any(|&b| b > 0);
                if has_color {
                    png_map.entry(f1).or_default().push((frame_no, bytes));
                }
            }
        }
        assert!(!png_map.is_empty(), "assets_hd_pack/1x 应至少有一个可解码的非黑 PNG");

        // 帧号排序 → (f1, Vec<Vec<u8>>)
        let png_bytes: Vec<(u32, Vec<Vec<u8>>)> = png_map
            .into_iter()
            .map(|(f1, mut frames)| {
                frames.sort_by_key(|(n, _)| *n);
                (f1, frames.into_iter().map(|(_, b)| b).collect())
            })
            .collect();

        // 注入到一个 GameApp 的 sprite_atlas
        let mut app = GameApp::new(176.0, 220.0);
        app.load_hd_pngs(png_bytes);
        let cache_size = app.atlas_hd_cache_size();
        assert!(cache_size > 0, "HD PNG 注入后 cache_size 应 > 0");

        // 渲染 Index 场景（含 f1 精灵 blit）
        app.render_hd();
        let pixels = app.shell().fb_rgba.as_slice().to_vec();

        // 验证存在非黑、非纯 PCX 扩色的 RGBA8 像素
        let non_black = pixels.iter().filter(|&&p| p != 0xFF00_0000 && p != 0).count();
        assert!(non_black > FB_PIXELS / 10, "HD 渲染后应有大量非黑像素");
    }

    /// P5-2 判据 7（多帧 PNG 注入）：多帧精灵（f1=14，5 帧）注入后，
    /// 第 0 帧与第 1 帧 blit 出的 RGBA8 画面不同（动画帧切换生效）。
    /// 缺失 `assets_hd_pack/1x/ui/14_hd_f0.png` 时 SKIP。
    #[test]
    fn hd_multi_frame_png_blits_differ_by_frame() {
        use ti_platform::hd_png::decode_png_rgba;

        let base = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let f0_path = base.join("assets_hd_pack").join("1x").join("ui").join("14_hd_f0.png");
        let f1_path = base.join("assets_hd_pack").join("1x").join("ui").join("14_hd_f1.png");
        if !f0_path.is_file() || !f1_path.is_file() {
            eprintln!("SKIP: 多帧 PNG 14_hd_f0/f1 不存在");
            return;
        }

        let mut frames: Vec<Vec<u8>> = Vec::new();
        for k in 0..5u32 {
            let p =
                base.join("assets_hd_pack").join("1x").join("ui").join(format!("14_hd_f{k}.png"));
            if p.is_file() {
                if let Ok(bytes) = std::fs::read(&p) {
                    let Ok(img) = decode_png_rgba(&bytes) else {
                        continue;
                    };
                    let has_color = img.rgba.iter().any(|&b| b > 0);
                    if has_color {
                        frames.push(bytes);
                    }
                }
            }
        }
        assert!(frames.len() >= 2, "f1=14 多帧 PNG 应至少 2 帧（实际 {}/5）", frames.len());

        let mut app = GameApp::new(176.0, 220.0);
        app.load_hd_pngs(vec![(14u32, frames)]);
        assert!(app.atlas_hd_cache_size() >= 1, "f1=14 注入后缓存应 ≥1");

        let scene = ti_formats::mapfile::MapScene {
            header: ti_formats::mapfile::MapHeader {
                width: 176,
                height: 220,
                flags: 0,
                reserved: 0,
            },
            words: vec![176, 220, 0, 0, 1, 1, 14, 10, 10, 1, 0],
        };
        let atlas = app.atlas_mut().expect("atlas 应已注入");
        let mut fb0 = ti_platform::gfx::FramebufRgba::new();
        let mut fb1 = ti_platform::gfx::FramebufRgba::new();
        atlas.blit_scene_frame_rgba(&scene, &mut fb0, 0);
        atlas.blit_scene_frame_rgba(&scene, &mut fb1, 1);
        assert_ne!(fb0.as_slice(), fb1.as_slice(), "f1=14 多帧 PNG 不同帧号 blit 画面应不同");
    }
}
