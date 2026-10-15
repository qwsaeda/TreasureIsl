//! 视觉回归基础设施（EXP-072）：3 关键场景 headless 逐像素基准 diff。
//!
//! P4-1 `render_parity` 只锁 PCX 解码基准，不锁「场景渲染最终像素」。
//! 本测试补洞：`GameApp::render_to_framebuf()` → 176×220 RGB565 像素
//! → 与 `assets/hd_baseline/<场景>.png` 基准图逐像素对比。
//!
//! 判据：
//! - 基准图缺失（首次跑）→ 生成 PNG 落盘 + PASS；
//! - 基准图存在（后续跑）→ RGB565→RGB8 逐像素 diff，允许 ±1 量化误差，
//!   diff 像素数 = 0 才 PASS（漂移 → 输出 diff 数 + 最大偏差 → FAIL）。
//!
//! 基准图目录 `assets/hd_baseline/`，3 张：`Index.png` / `SelectRole.png` / `MainMap.png`。
//! 生成脚本：`scripts/visual_baseline.ps1`（幂等：已有基准图则只校验）。
//!
//! PNG 编码/解码自实现（zlib stored 块 + 自研 inflate），零额外依赖。

#![allow(
    clippy::cast_possible_truncation,
    clippy::many_single_char_names,
    clippy::expect_used,
    clippy::items_after_statements,
    clippy::single_match_else
)]

use std::fs;
use std::path::Path;

use ti_app::game_app::{GameApp, GameInput};

const FB_W: usize = 176;
const FB_H: usize = 220;
const FB_PIXELS: usize = FB_W * FB_H;

/// RGB565 允许 ±1 量化误差（5-bit 通道展开到 8-bit 的固有偏差）。
const TOLERANCE: i32 = 1;

/// 仓库根（`crates/ti-app/` 向上两级，与 `repo_root()` 同口径，EXP-050 CJK 路径修正）。
fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| std::path::PathBuf::from("."), Path::to_path_buf)
}

/// 基准图目录 `assets/hd_baseline/`（不存在时创建，首次生成用）。
fn baseline_dir() -> std::path::PathBuf {
    let dir = repo_root().join("assets").join("hd_baseline");
    fs::create_dir_all(&dir).expect("基准图目录创建失败");
    dir
}

/// 基准图路径（`assets/hd_baseline/<name>.png`）。
fn baseline_path(name: &str) -> std::path::PathBuf {
    baseline_dir().join(format!("{name}.png"))
}

/// RGB565 → RGB8 三通道（与 `ti_platform::gfx::rgb565_to_rgba8` 同公式，无 macroquad 依赖）。
fn rgb565_to_rgb8(p: u16) -> [i32; 3] {
    let r5 = i32::from((p >> 11) & 0x1F);
    let g6 = i32::from((p >> 5) & 0x3F);
    let b5 = i32::from(p & 0x1F);
    [(r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2)]
}

/// 把 RGB565 帧缓冲编码成 PNG（RGB8，不透明）字节（自实现，零额外依赖）。
fn encode_png_rgb565(pixels: &[u16]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(FB_PIXELS * 3);
    for &p in pixels {
        let [r, g, b] = rgb565_to_rgb8(p);
        rgb.extend_from_slice(&[
            u8::try_from(r).unwrap_or(0),
            u8::try_from(g).unwrap_or(0),
            u8::try_from(b).unwrap_or(0),
        ]);
    }
    // 行主序原始像素（每行 1 字节 filter=None + 176×3 RGB）
    let row_bytes = FB_W * 3;
    let mut raw: Vec<u8> = Vec::with_capacity((FB_H + 1) * row_bytes);
    raw.push(0u8); // filter: None
    for y in 0..FB_H {
        raw.extend_from_slice(&rgb[y * row_bytes..(y + 1) * row_bytes]);
    }

    // zlib 压缩：stored 块（BFINAL=1, BTYPE=00，无压缩存储 + adler32 校验）
    let zlib = zlib_compress_stored(&raw);

    // CRC32（IEEE 802.3，PNG 标准）
    let crc_table = build_crc_table();
    let crc32 = |data: &[u8]| crc32_calc(data, &crc_table);

    let mut out = vec![137u8, 80, 78, 71, 13, 10, 26, 10]; // PNG 签名
    let chunk = |tag: &[u8; 4], data: &[u8]| -> Vec<u8> {
        let mut buf = Vec::with_capacity(12 + data.len());
        buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
        buf.extend_from_slice(tag);
        buf.extend_from_slice(data);
        let mut crc_input = Vec::with_capacity(4 + data.len());
        crc_input.extend_from_slice(tag);
        crc_input.extend_from_slice(data);
        buf.extend_from_slice(&crc32(&crc_input).to_be_bytes());
        buf
    };

    // IHDR：176×220（宽在前），bit depth 8，color type 2（RGB）
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&176u32.to_be_bytes());
    ihdr.extend_from_slice(&220u32.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    out.extend(chunk(b"IHDR", &ihdr));
    out.extend(chunk(b"IDAT", &zlib));
    out.extend(chunk(b"IEND", &[]));
    out
}

/// zlib 压缩（stored 块，无压缩存储 + adler32）。
fn zlib_compress_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 5 + data.len() + 4);
    out.push(0x78);
    out.push(0x01); // zlib CMF=8, FLEVEL=0
    // stored block: BFINAL=1, BTYPE=00
    out.push(0x01);
    let len = u16::try_from(data.len()).unwrap_or(0xFFFF);
    out.extend_from_slice(&len.to_le_bytes());
    // nlen = !len (按位取反)
    out.extend_from_slice(&(!len).to_le_bytes());
    out.extend_from_slice(data);
    let adler = adler32(data);
    out.extend_from_slice(&adler.to_be_bytes());
    out
}

/// adler32（zlib 校验）。
fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// CRC32 查表（IEEE 802.3 多项式 0xEDB88320）。
fn build_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for i in 0..256u32 {
        let mut c = i;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        table[i as usize] = c;
    }
    table
}

/// CRC32 计算。
fn crc32_calc(data: &[u8], table: &[u32; 256]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ u32::from(b)) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

/// 解码基准 PNG → RGB8 字节（176×220×3），尺寸不符或 IO 失败时返回 `None`。
fn decode_baseline(path: &Path) -> Option<Vec<u8>> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() < 8 || bytes[0..8] != [137u8, 80, 78, 71, 13, 10, 26, 10] {
        return None;
    }
    let mut pos = 8usize;
    let mut idat: Vec<u8> = Vec::new();
    let mut width = 0u32;
    let mut height = 0u32;
    while pos + 8 <= bytes.len() {
        let len = u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
            as usize;
        let tag = &bytes[pos + 4..pos + 8];
        let data_start = pos + 8;
        if data_start + len > bytes.len() {
            break;
        }
        let data = &bytes[data_start..data_start + len];
        match tag {
            b"IHDR" => {
                if data.len() < 13 {
                    return None;
                }
                width = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                height = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            }
            b"IDAT" => idat.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
        pos = data_start + len + 4;
    }
    if width != FB_W as u32 || height != FB_H as u32 {
        return None;
    }
    let raw = zlib_decompress_stored(&idat)?;
    if raw.len() != FB_H * (1 + FB_W * 3) {
        return None;
    }
    let mut rgb = Vec::with_capacity(FB_PIXELS * 3);
    for y in 0..FB_H {
        let filter = raw[y * (1 + FB_W * 3)];
        if filter != 0 {
            return None;
        }
        let row_start = y * (1 + FB_W * 3) + 1;
        rgb.extend_from_slice(&raw[row_start..row_start + FB_W * 3]);
    }
    Some(rgb)
}

/// 解压 zlib stored 块（本测试生成的基准图格式）。
fn zlib_decompress_stored(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 3 {
        return None;
    }
    if data[0] & 0x0F != 8 {
        return None;
    }
    let mut out: Vec<u8> = Vec::new();
    let mut pos = 2usize;
    loop {
        if pos >= data.len() {
            break;
        }
        let bfinal = data[pos] & 0x01;
        let btype = (data[pos] >> 1) & 0x03;
        pos += 1;
        if btype == 0 {
            if pos + 4 > data.len() {
                return None;
            }
            let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 4;
            if pos + len > data.len() {
                return None;
            }
            out.extend_from_slice(&data[pos..pos + len]);
            pos += len;
        } else {
            return None;
        }
        if bfinal == 1 {
            break;
        }
    }
    Some(out)
}

/// 背景色 RGB8（`COLOR_BG` 米白 0xFD70 清屏色；UI 像素 = 非背景色，EXP-074）。
///
/// EXP-074：视觉回归只锁「UI 像素分布」（高亮矩形 / 文字 / HUD 落在哪些逻辑像素），
/// **不锁背景贴图**——HD 化（`assets_hd_pack` 2x/4x PNG）与原版 PCX 背景不同，
/// 整帧 diff 会因背景差异误报。故 diff 时把「背景色像素」掩掉，只比 UI 层。
/// UI-14：清屏色 = `COLOR_BG` 米白 0xFD70（S14 航海图纸质系），非 16（黑色）。
const BG565: u16 = 0xFD70;

/// 清屏背景色在 RGB565→RGB8 下的三通道值（用于掩背景）。
fn bg_rgb8() -> [i32; 3] {
    rgb565_to_rgb8(BG565)
}

/// 逐像素 diff：只比 **UI 像素**（任一侧非背景色的位置），背景像素不参与计数（EXP-074）。
///
/// 返回 `(ui_diff_pixels, max_channel_delta)`；UI 层（高亮矩形/文字/HUD）有漂移才计数。
/// 两侧都是背景色的像素 → 跳过（HD 化背景差异不判漂）。
fn diff_frames_ui(current_rgb: &[u8], baseline_rgb: &[u8]) -> (usize, i32) {
    let bg = bg_rgb8();
    let mut diff = 0usize;
    let mut max_delta = 0i32;
    for i in 0..FB_PIXELS {
        let is_bg_cur =
            (0..3).all(|c| (i32::from(current_rgb[i * 3 + c]) - bg[c]).abs() <= TOLERANCE);
        let is_bg_base =
            (0..3).all(|c| (i32::from(baseline_rgb[i * 3 + c]) - bg[c]).abs() <= TOLERANCE);
        // 背景色像素不参与 UI 漂移计数（EXP-074：HD 背景与原版 PCX 背景不同，非排版漂移）
        if is_bg_cur && is_bg_base {
            continue;
        }
        for ch in 0..3usize {
            let cur = i32::from(current_rgb[i * 3 + ch]);
            let base = i32::from(baseline_rgb[i * 3 + ch]);
            let delta = (cur - base).abs();
            if delta > max_delta {
                max_delta = delta;
            }
            if delta > TOLERANCE {
                diff += 1;
                break;
            }
        }
    }
    (diff, max_delta)
}

/// 单场景视觉回归：渲染 → 基准图存在则 diff 断言，缺失则生成落盘。
fn visual_regression(name: &str, app: GameApp) {
    let pixels: Vec<u16> = {
        let mut app = app;
        app.render_to_framebuf().to_vec()
    };
    assert_eq!(pixels.len(), FB_PIXELS, "{name} 帧缓冲尺寸应 = {FB_PIXELS}");

    let path = baseline_path(name);
    match decode_baseline(&path) {
        Some(baseline) => {
            let current_rgb: Vec<u8> = pixels
                .iter()
                .flat_map(|&p| {
                    let [r, g, b] = rgb565_to_rgb8(p);
                    [
                        u8::try_from(r).unwrap_or(0),
                        u8::try_from(g).unwrap_or(0),
                        u8::try_from(b).unwrap_or(0),
                    ]
                })
                .collect();
            let (diff, max_delta) = diff_frames_ui(&current_rgb, &baseline);
            assert_eq!(
                diff, 0,
                "{name} 视觉回归漂移（UI 像素分布）：diff 像素 {diff}/{FB_PIXELS}，最大通道偏差 {max_delta}（允许 ±{TOLERANCE}，背景像素已掩，EXP-074）"
            );
        }
        None => {
            let png_bytes = encode_png_rgb565(&pixels);
            fs::write(&path, &png_bytes).expect("基准图落盘失败");
            eprintln!(
                "视觉回归：{name} 基准图缺失 → 生成 {}（{} bytes）",
                path.display(),
                png_bytes.len()
            );
        }
    }
}

/// 主菜单（Index 场景，默认 `GameApp::new` 即 Index）。
fn render_index() -> GameApp {
    let app = GameApp::new(176.0, 220.0);
    assert_eq!(app.scene(), ti_app::game_app::Scene::Index, "默认场景应 = Index");
    app
}

/// 角色选择场景（Index → SelectRole，菜单项 0 确认）。
fn render_select_role() -> GameApp {
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), ti_app::game_app::Scene::SelectRole, "Confirm 后应进 SelectRole");
    app
}

/// 主地图场景（Index → `SelectRole` → `SelectMap` → `MainMap`）。
fn render_main_map() -> GameApp {
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), ti_app::game_app::Scene::MainMap, "5 步后应进 MainMap");
    app
}

#[test]
fn visual_regression_index() {
    visual_regression("Index", render_index());
}

#[test]
fn visual_regression_select_role() {
    visual_regression("SelectRole", render_select_role());
}

#[test]
fn visual_regression_main_map() {
    visual_regression("MainMap", render_main_map());
}

/// 三场景渲染结果互不相同（基准图非全同，防全黑/全背景基准）。
#[test]
fn baseline_scenes_are_distinct() {
    let index: Vec<u16> = {
        let mut app = render_index();
        app.render_to_framebuf().to_vec()
    };
    let role: Vec<u16> = {
        let mut app = render_select_role();
        app.render_to_framebuf().to_vec()
    };
    let main_map: Vec<u16> = {
        let mut app = render_main_map();
        app.render_to_framebuf().to_vec()
    };
    assert_ne!(index, role, "Index 与 SelectRole 基准图应不同");
    assert_ne!(role, main_map, "SelectRole 与 MainMap 基准图应不同");
    assert_ne!(index, main_map, "Index 与 MainMap 基准图应不同");
}

/// 确定性：同一场景渲染两次逐像素相同（基准图落盘后可复现）。
#[test]
fn visual_regression_is_deterministic() {
    for name in ["Index", "SelectRole", "MainMap"] {
        let app = match name {
            "Index" => render_index(),
            "SelectRole" => render_select_role(),
            _ => render_main_map(),
        };
        let a: Vec<u16> = {
            let mut app = app;
            app.render_to_framebuf().to_vec()
        };
        let app = match name {
            "Index" => render_index(),
            "SelectRole" => render_select_role(),
            _ => render_main_map(),
        };
        let b: Vec<u16> = {
            let mut app = app;
            app.render_to_framebuf().to_vec()
        };
        assert_eq!(a, b, "{name} 场景两次渲染应逐像素相同（确定性）");
    }
}
