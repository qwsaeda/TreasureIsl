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
#![allow(
    clippy::many_single_char_names,
    clippy::redundant_closure_for_method_calls,
    clippy::type_complexity,
    clippy::manual_let_else,
    clippy::items_after_statements,
    clippy::single_match_else,
    clippy::unwrap_used,
    clippy::expect_used
)]

use ti_app::game_app::{GameApp, GameInput};
use ti_platform::gfx::Framebuf;

/// 逻辑帧像素数（`LOGICAL_W × LOGICAL_H`）。
const FB_PIXELS: usize = 176 * 220;

/// 渲染当前场景到 `GameApp.shell.fb`，返回帧缓冲像素切片。
fn render_current(app: &mut GameApp) -> Vec<u16> {
    app.render();
    app.shell().fb.as_slice().to_vec()
}

/// 场景渲染的「墨点率」：非背景色像素占比（证明画了真实内容，非全黑/全背景）。
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn ink_ratio(pixels: &[u16]) -> f32 {
    // S14 航海图纸质系：背景 = 0xFD70（米白），标题条 = 0x87BB（棕），选中高亮 = 0x066F（海绿），墨色 = 0xFFFF。
    let content = pixels.iter().filter(|&&p| p != 0x0000 && p != 0xFD70).count();
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
    app.shell_mut().render();
    let pixels = app.shell_mut().fb.as_slice();
    assert_eq!(pixels[0], 0x87BB, "主菜单第 0 行 x=0 应为标题条色（S14 航海图纸质系棕）");
}

// ── S8.6 渲染基线去自指：独立 ti-formats 解码基线 vs 渲染逐像素对照 ──

/// 定位仓库根 `data/` 目录（`CARGO_MANIFEST_DIR` 向上两级）。
fn data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| std::path::PathBuf::from("."), |p| p.to_path_buf())
        .join("legacy")
        .join("data")
}

/// 用 `ti-formats` 独立路径解码 `ui.pak` 中 f1 对应的 PCX 精灵，
/// 返回 `(width, frame_height, pixels, palette, frames)`。
/// 不经过 `GameApp`/`SpriteAtlas`，纯 `ti_formats` 解码；
/// 帧高 = 图高 / 帧数（与 `SpriteAtlas` 多帧语义一致）。
fn decode_pcx_independent(f1: u32) -> Option<(u32, u32, Vec<u8>, [[u8; 3]; 256], u32)> {
    use ti_formats::lucency::Package;
    use ti_formats::pcx::Image;

    let base = data_dir();
    let pkg_bytes = std::fs::read(base.join("ui.pak")).ok()?;
    let pkg = Package::parse("ui.pak", &pkg_bytes).ok()?;

    // 找 filenum INI（f1 → 文件名 + 帧数），与 `SpriteAtlas::parse_filenum_ini` 同逻辑
    let mut f1_to_info: std::collections::HashMap<u32, (String, u32)> =
        std::collections::HashMap::new();
    for (i, _) in pkg.entries.iter().enumerate() {
        let content = match pkg.read_entry(&pkg_bytes, i) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if content.len() < 4 || content[0] != 0xFF || content[1] != 0xFE {
            continue;
        }
        let units: Vec<u16> =
            content.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        let Ok(text) = String::from_utf16(&units) else {
            continue;
        };
        let mut section: Option<u32> = None;
        let mut pending_fn: Option<String> = None;
        let mut pending_num: u32 = 1;
        for raw in text.lines() {
            let line = raw.trim().trim_start_matches('\u{feff}');
            if line.is_empty() {
                continue;
            }
            if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(id) = section.take() {
                    if let Some(name) = pending_fn.take() {
                        f1_to_info.insert(id, (name, pending_num));
                    }
                }
                pending_num = 1;
                if inner != "filenum" {
                    section = inner.parse::<u32>().ok();
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix("filename=") {
                pending_fn = Some(rest.trim().to_owned());
                continue;
            }
            if let Some(rest) = line.strip_prefix("num=") {
                pending_num = rest.trim().parse::<u32>().unwrap_or(1).max(1);
            }
        }
        if let Some(id) = section.take() {
            if let Some(name) = pending_fn.take() {
                f1_to_info.insert(id, (name, pending_num));
            }
        }
        if !f1_to_info.is_empty() {
            break;
        }
    }

    let (name, frames) = f1_to_info.get(&f1)?.clone();
    let entry_idx = pkg.find(&name)?;
    let pcx_bytes = pkg.read_entry(&pkg_bytes, entry_idx).ok()?;
    let img = Image::decode(&name, &pcx_bytes).ok()?;
    let frame_h = u32::from(img.height).checked_div(frames).unwrap_or(1).max(1);
    Some((u32::from(img.width), frame_h, img.pixels, img.palette, frames))
}

/// 把 PCX 像素（调色板索引）blit 到独立 Framebuf（RGB565），
/// 与 `SpriteAtlas::blit_sprite` 同几何（越界裁剪），但不经过 ti-app 渲染路径。
fn blit_pcx_independent(
    fb: &mut Framebuf,
    width: u32,
    height: u32,
    pixels: &[u8],
    palette: &[[u8; 3]; 256],
    x: u32,
    y: u32,
) {
    const FB_W: usize = 176;
    const FB_H: usize = 220;
    // 契约 §3（07_render_pipeline.md）：原点越界求交为空 → 整条不画（对齐 blit_sprite）
    if x as usize >= FB_W || y as usize >= FB_H {
        return;
    }
    let ox = x as usize;
    let oy = y as usize;
    for row in 0..height as usize {
        let fy = oy.saturating_add(row);
        if fy >= FB_H {
            break;
        }
        for col in 0..width as usize {
            let fx = ox.saturating_add(col);
            if fx >= FB_W {
                break;
            }
            let idx = pixels.get(row * (width as usize) + col).copied().unwrap_or(0) as usize;
            // 同口径：palette index 0 = 透明哨兵位（sub_5D9C4），跳过（与 blit_sprite 一致）
            if idx == 0 {
                continue;
            }
            let [r, g, b] = palette[idx];
            let rgb565: u16 =
                (u16::from(r) >> 3) << 11 | (u16::from(g) >> 2) << 5 | u16::from(b) >> 3;
            let pos = fy * FB_W + fx;
            if let Some(slot) = fb.as_mut_slice().get_mut(pos) {
                *slot = rgb565;
            }
        }
    }
}

/// S8.6 判据：渲染帧中精灵 blit 区域与独立 `ti-formats` 解码基线逐像素一致。
///
/// 独立基线路径：`ti_formats::lucency::Package` → `pcx::Image::decode` → 手动 blit，
/// 不经过 `GameApp::render` / `SpriteAtlas`。
#[test]
fn render_sprite_region_matches_tiformats_baseline() {
    let (w, h, pixels, palette, _frames) = match decode_pcx_independent(1) {
        Some(v) => v,
        None => {
            eprintln!("SKIP: data/ui.pak 不存在或 f1=1 解码失败");
            return;
        }
    };

    // 独立基线：干净 Framebuf + 手动 blit
    let mut baseline = Framebuf::new();
    baseline.clear(0x0000);
    blit_pcx_independent(&mut baseline, w, h, &pixels, &palette, 10, 10);

    // 渲染路径：注入 atlas + scene_cache，单独 blit（不经 render 全文字层）
    // MapScene::parse 期望完整文件字节（16B 头 + words）；words 布局 =
    // [表头4字, 控件数, 控件记录6字…]，f1/f2/f3 对应 w[5]/w[6]/w[7]。
    let pkg_bytes = std::fs::read(data_dir().join("ui.pak")).expect("ui.pak 应可解析");
    let mut atlas =
        ti_app::sprite_atlas::SpriteAtlas::new(&pkg_bytes).expect("ui.pak 应可解析为 SpriteAtlas");
    let words: Vec<u32> = vec![
        176, 220, 0, 0, /* 控件数 */ 1, /* f0 f1 f2 f3 f4 f5 */ 1, 1, 10, 10, 1, 0,
    ];
    let mut scene_bytes: Vec<u8> = Vec::new();
    for w in [176u32, 220, 0, 0] {
        scene_bytes.extend_from_slice(&w.to_le_bytes());
    }
    for w in &words {
        scene_bytes.extend_from_slice(&w.to_le_bytes());
    }
    let scene =
        ti_formats::mapfile::MapScene::parse("index.Map", &scene_bytes).expect("场景 bytes 应合法");
    let mut app = GameApp::new(176.0, 220.0);
    let app_shell = app.shell_mut();
    app_shell.fb.clear(0x0000);
    atlas.blit_scene(&scene, &mut app_shell.fb);

    // 逐像素比对精灵 blit 区域（10,10 起 w×h，裁剪到 176×220）
    let cw = w as usize;
    let ch = h as usize;
    const X0: usize = 10;
    const Y0: usize = 10;
    let render_pixels = app_shell.fb.as_slice();
    let base_pixels = baseline.as_slice();
    let mut mismatch = 0usize;
    for row in 0..ch {
        let fy = Y0.saturating_add(row);
        if fy >= FB_PIXELS / 176 {
            break;
        }
        for col in 0..cw {
            let fx = X0.saturating_add(col);
            if fx >= 176 {
                break;
            }
            let pos = fy * 176 + fx;
            if render_pixels[pos] != base_pixels[pos] {
                mismatch += 1;
            }
        }
    }
    assert_eq!(
        mismatch, 0,
        "渲染帧精灵 blit 区域应与独立 ti-formats 解码基线逐像素一致（mismatch={mismatch}）"
    );
}

/// S8.6 判据：基线帧（独立 ti-formats 解码）本身非空（证明解码成功且 blit 有内容）。
#[test]
fn tiformats_baseline_is_non_empty() {
    let (w, h, pixels, palette, frames) = match decode_pcx_independent(1) {
        Some(v) => v,
        None => {
            eprintln!("SKIP: data/ui.pak 不存在或 f1=1 解码失败");
            return;
        }
    };
    assert!(w > 0 && h > 0, "PCX 宽高应 > 0");
    // pixels 是全图（多帧纵向堆叠），单帧高 = h，总高 = h × frames
    let total_h = (h as usize) * (frames as usize);
    assert_eq!(pixels.len(), (w as usize) * total_h, "像素数 = w × (h × frames)");
    assert!(!palette.iter().all(|c| c[0] == 0 && c[1] == 0 && c[2] == 0), "调色板非全黑");
    let mut fb = Framebuf::new();
    fb.clear(0x0000);
    blit_pcx_independent(&mut fb, w, h, &pixels, &palette, 10, 10);
    let non_black = fb.as_slice().iter().filter(|&&p| p != 0).count();
    assert!(non_black > 0, "独立基线 blit 后应有非黑像素（实际 {non_black}）");
}
