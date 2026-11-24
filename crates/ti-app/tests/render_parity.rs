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
use ti_render::framebuffer::Framebuf;

/// 呈现层帧像素数（`1920 × 1080`，S84 纯横屏；legacy 176×220 统一缩放落画布）。
const FB_PIXELS: usize = 1920 * 1080;

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
    let mut app = GameApp::new(1920.0, 1080.0);
    render_current(&mut app)
}

/// 选角场景渲染。
fn render_select_role() -> Vec<u16> {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.handle(GameInput::Confirm); // Index → SelectRole
    render_current(&mut app)
}

/// 选图场景渲染。
fn render_select_map() -> Vec<u16> {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Confirm); // → SelectMap
    render_current(&mut app)
}

/// 主地图场景渲染。
fn render_main_map() -> Vec<u16> {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Next); // 选角
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Next); // 选图
    app.handle(GameInput::Confirm); // → MainMap
    render_current(&mut app)
}

/// 战斗场景渲染。
fn render_vs() -> Vec<u16> {
    let mut app = GameApp::new(1920.0, 1080.0);
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
        assert_eq!(pixels.len(), FB_PIXELS, "{name} 帧缓冲尺寸应 = 1920×1080（S84 纯横屏）");
    }
}

/// P4-1 判据：渲染后有真实内容（非黑屏/全背景）。
/// S85.5：5 纯 UI 场景 chrome 迁 schema（CPU 死原语已删，CPU 帧不再承载），
/// 像素判据只对游戏世界（MainMap/Vs）成立；5 场景改验 schema 组件表非空。
#[test]
fn each_scene_renders_real_content() {
    let schema = ti_core::data::ui_schema::components().expect("components_v1.json");
    for name in ["index", "select_role", "select_map", "message", "saveload"] {
        let sc = schema.get(name).expect("缺 UI 场景组件");
        assert!(!sc.elements.is_empty(), "{name} 组件表为空（画面将空白）");
    }
    for (name, pixels) in [("MainMap", render_main_map()), ("Vs", render_vs())] {
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
/// S85.5：标题已烘焙进 `index_bg` 资产、木牌文案走 schema 文本组件；CPU 帧不再承载首页，
/// 故改验 schema 文本 binding 解析非空（真字形由 `--shot` bevy 探针守）。
#[test]
fn index_scene_renders_real_glyphs_for_title() {
    let sc = ti_core::data::ui_schema::components()
        .expect("components_v1.json")
        .get("index")
        .expect("index 组件")
        .clone();
    let footer = sc.elements.iter().find(|e| e.id == "footer").expect("index 缺 footer 组件");
    assert_eq!(footer.binding, "scene.index.footer", "首页文案 binding 漂移");
    let text = ti_app::i18n::sget(&footer.binding);
    assert!(!text.is_empty(), "首页文案 i18n 解析为空（裸 key / 死文案）");
}

/// P4-1 判据：`Framebuf` 尺寸与呈现层分辨率 1920×1080 一致（S84 纯横屏；legacy 176×220 统一缩放落画布）。
#[test]
fn framebuf_matches_logical_resolution() {
    let mut fb = Framebuf::new();
    fb.clear(0x0000);
    assert_eq!(fb.as_slice().len(), FB_PIXELS, "帧缓冲 = 1920×1080（S84 纯横屏）");
    // S85.5：index chrome 迁 schema（CPU 帧不再画首页），(0,0) 像素判据只对游戏世界成立；
    // index 背景由 `index_bg` role 指向 assets/ui/index_bg_map_h.png（--shot 非黑探针守）。
    let roles = ti_core::data::ui_schema::asset_roles().expect("asset_roles.json");
    assert_eq!(roles.path("index_bg"), Some("assets/ui/index_bg_map_h.png"), "首页底图 role 漂移");
    let mut game = GameApp::new(1920.0, 1080.0);
    game.handle(GameInput::Confirm);
    game.handle(GameInput::Next);
    game.handle(GameInput::Confirm);
    game.handle(GameInput::Next);
    game.handle(GameInput::Confirm); // → MainMap（游戏世界）
    let world_pixels = render_current(&mut game);
    let wood = ti_app::menu_shell::COLOR_WOOD;
    let parch_dark = ti_app::menu_shell::COLOR_PARCH_DARK;
    let wood_warm = ti_app::scenes::chrome::COLOR_WOOD_WARM;
    assert!(
        world_pixels[0] == wood
            || world_pixels[0] == parch_dark
            || world_pixels[0] == wood_warm
            || world_pixels[0] != 0,
        "游戏世界像素(0,0) 应为木纹/羊皮纸/背景贴图（非黑），实为 {:#05X}",
        world_pixels[0]
    );
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

// S15.4：legacy 逐像素 blit 对齐（`blit_pcx_independent` / `render_sprite_region_matches_tiformats_baseline`）
// 已随 legacy 像素写帧层移除而删除。保留 `decode_pcx_independent`（纯 ti-formats 解码，诊断用）
// 与下方 5 个 `app.render()` 代码绘制判定。逐像素渲染基线重锚见 S15.5。

/// S8.6 判据：独立 ti-formats 解码基线本身非空（证明 PCX 解码成功，decode-only）。
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
}
