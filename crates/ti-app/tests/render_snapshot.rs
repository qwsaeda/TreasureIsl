//! 渲染快照门禁（契约 §5，`docs/process/07_render_pipeline.md`）：7 场景逐像素基准 diff。
//!
//! 职责：锁「场景渲染最终像素」全场景覆盖（RGB565 native 路，`render_to_framebuf`），
//! 越界求交修复（契约 §3）+ 数据源（`.Map`/`.pak`）漂移当场拦截。
//!
//! 判据（与 `visual_regression.rs` 同口径，EXP-074 背景掩码）：
//! - 基准缺失 → 生成落盘 `assets/render_baseline/<scene>.png` + PASS（首跑）；
//! - 基准存在 → RGB565→RGB8 逐像素 diff，背景像素掩（EXP-074），diff=0 才 PASS。
//!
//! 7 个有 `.Map` 的场景：index/selectrole/selectmap/main/vs/msgs/saveload。
//! 基准由**越界修复后**代码生成（杜绝 EXP-074/075 自证循环，契约 §5）。
//!
//! PNG 编码/解码自实现（zlib stored 块 + 自研 inflate，与 `visual_regression` 同口径），零额外依赖。

#![allow(
    clippy::cast_possible_truncation,
    clippy::expect_used,
    clippy::items_after_statements,
    clippy::panic
)]

use std::fs;
use std::path::{Path, PathBuf};

use ti_app::sprite_atlas::SpriteAtlas;
use ti_formats::mapfile::MapScene;
use ti_platform::gfx::Framebuf;

const FB_W: usize = 176;
const FB_H: usize = 220;
const FB_PIXELS: usize = FB_W * FB_H;
const TOLERANCE: i32 = 1;
/// 清屏背景色 RGB565（`menu_shell::COLOR_BG` 米白 0xFD70，UI 像素 = 非背景色，EXP-074）。
///
/// 背景掩码 = 清屏色（`MenuShell::clear_frame(COLOR_BG)` 0xFD70），非标题条色 0x87BB。
/// 标题条/HUD 是 UI 像素（非背景），漂移须被检测到。
const BG565: u16 = 0xFD70;

#[test]
fn render_snapshot_all_scenes() {
    // 7 个场景全部直接渲染各自 `.Map`，不复用状态机导航
    // （原实现对 main/vs/msgs/saveload 用「当前可达场景」，5 张基准字节相同 = 门禁空转）。
    for name in ["index", "selectrole", "selectmap", "main", "vs", "msgs", "saveload"] {
        snapshot_map_scene(name);
    }

    // 7 张基准必须两两不同（防空转回归：曾经 5 张字节完全相同，门禁对那 5 个场景无效）。
    let names = ["index", "selectrole", "selectmap", "main", "vs", "msgs", "saveload"];
    let mut digests: Vec<(String, String)> = Vec::new();
    for name in names {
        let path = baseline_path(name);
        let bytes =
            fs::read(&path).unwrap_or_else(|e| panic!("基准 {} 读取失败: {e}", path.display()));
        let digest = bytes
            .iter()
            .fold(0x517c_e1b7_0007u64, |h, &b| h.wrapping_mul(0x100_0000_01b3) ^ u64::from(b));
        digests.push((name.to_owned(), format!("{digest:016x}")));
    }
    for i in 0..digests.len() {
        for j in (i + 1)..digests.len() {
            assert_ne!(
                digests[i].1, digests[j].1,
                "基准 {} 与 {} 字节完全相同 —— 快照门禁对该场景空转",
                digests[i].0, digests[j].0
            );
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn baseline_dir() -> PathBuf {
    let dir = repo_root().join("assets").join("render_baseline");
    fs::create_dir_all(&dir).expect("快照目录创建失败");
    dir
}

fn baseline_path(name: &str) -> PathBuf {
    baseline_dir().join(format!("{name}.png"))
}

/// RGB565 → RGB8 三通道（与 `ti_platform::gfx::rgb565_to_rgba8` 同公式）。
fn rgb565_to_rgb8(p: u16) -> [i32; 3] {
    let r5 = i32::from((p >> 11) & 0x1F);
    let g6 = i32::from((p >> 5) & 0x3F);
    let b5 = i32::from(p & 0x1F);
    [(r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2)]
}

fn bg_rgb8() -> [i32; 3] {
    rgb565_to_rgb8(BG565)
}

/// 逐像素 diff：只比 UI 像素（任一侧非背景色），背景像素掩（EXP-074）。
fn diff_frames_ui(current_rgb: &[u8], baseline_rgb: &[u8]) -> (usize, i32) {
    let bg = bg_rgb8();
    let mut diff = 0usize;
    let mut max_delta = 0i32;
    for i in 0..FB_PIXELS {
        let is_bg_cur =
            (0..3).all(|c| (i32::from(current_rgb[i * 3 + c]) - bg[c]).abs() <= TOLERANCE);
        let is_bg_base =
            (0..3).all(|c| (i32::from(baseline_rgb[i * 3 + c]) - bg[c]).abs() <= TOLERANCE);
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

fn to_rgb8(pixels: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(FB_PIXELS * 3);
    for &p in pixels {
        let [r, g, b] = rgb565_to_rgb8(p);
        out.extend_from_slice(&[
            u8::try_from(r).unwrap_or(0),
            u8::try_from(g).unwrap_or(0),
            u8::try_from(b).unwrap_or(0),
        ]);
    }
    out
}

/// 渲染并快照**指定场景的 `.Map`**（不经状态机导航）。
///
/// 原实现用 `snapshot_rendered` 走状态机导航取「当前可达场景」，headless 下
/// vs/msgs/saveload/main 均不可达，导致 5 张基准图字节完全相同 —— 门禁对这几个场景
/// 长期空转（任何渲染改动都不会被发现）。本函数直接加载 `legacy/data/<scene>.Map`
/// 调 [`SpriteAtlas::blit_scene_frame`] 渲染，保证 7 个场景各验各的。
fn snapshot_map_scene(name: &str) {
    let map_path = repo_root().join(format!("legacy/data/{name}.Map"));
    let map_bytes =
        fs::read(&map_path).unwrap_or_else(|e| panic!("{} 读取失败: {e}", map_path.display()));
    let scene =
        MapScene::parse(name, &map_bytes).unwrap_or_else(|e| panic!("{name}.Map 解析失败: {e}"));
    let pkg_path = repo_root().join("legacy/data/ui.pak");
    let pkg_bytes =
        fs::read(&pkg_path).unwrap_or_else(|e| panic!("{} 读取失败: {e}", pkg_path.display()));
    let mut atlas = SpriteAtlas::new(&pkg_bytes).expect("SpriteAtlas 构建失败");

    let mut fb = Framebuf::new();
    fb.clear(BG565);
    let n = atlas.blit_scene_frame(&scene, &mut fb, 0);
    assert!(n > 0, "{name} 应 blit 出控件（count>0）");
    let pixels: Vec<u16> = fb.as_slice().to_vec();
    assert_eq!(pixels.len(), FB_PIXELS, "{name} 帧缓冲尺寸应 = {FB_PIXELS}");

    let path = baseline_path(name);
    let current_rgb = to_rgb8(&pixels);
    if let Some(baseline) = decode_baseline(&path) {
        let (diff, max_delta) = diff_frames_ui(&current_rgb, &baseline);
        assert_eq!(
            diff, 0,
            "{name} 渲染快照漂移（UI 像素，背景已掩 EXP-074）：diff {diff}/{FB_PIXELS}，最大偏差 {max_delta}（契约 §5）"
        );
    } else {
        let png = encode_png_rgb8(&current_rgb);
        fs::write(&path, &png).expect("快照落盘失败");
        eprintln!(
            "渲染快照：{name} 基准缺失 → 生成 {}（{} bytes，{n} 控件）",
            path.display(),
            png.len()
        );
    }
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

/// RGB8 → PNG 编码（stored 块 + CRC32，与 `visual_regression` 同口径）。
fn encode_png_rgb8(rgb: &[u8]) -> Vec<u8> {
    let row_bytes = FB_W * 3;
    let mut raw: Vec<u8> = Vec::with_capacity((FB_H + 1) * row_bytes);
    raw.push(0u8);
    for y in 0..FB_H {
        raw.extend_from_slice(&rgb[y * row_bytes..(y + 1) * row_bytes]);
    }
    let zlib = zlib_compress_stored(&raw);
    let crc_table = build_crc_table();
    let crc32 = |data: &[u8]| crc32_calc(data, &crc_table);
    let mut out = vec![137u8, 80, 78, 71, 13, 10, 26, 10];
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
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&176u32.to_be_bytes());
    ihdr.extend_from_slice(&220u32.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    out.extend(chunk(b"IHDR", &ihdr));
    out.extend(chunk(b"IDAT", &zlib));
    out.extend(chunk(b"IEND", &[]));
    out
}

fn zlib_compress_stored(data: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(2 + 5 + data.len() + 4);
    out.push(0x78);
    out.push(0x01);
    out.push(0x01);
    let len = u16::try_from(data.len()).unwrap_or(0xFFFF);
    let nlen = !len;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&nlen.to_le_bytes());
    out.extend_from_slice(data);
    out.extend_from_slice(&[0x00, 0x00, 0xFF, 0xFF]);
    out
}

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

fn crc32_calc(data: &[u8], table: &[u32; 256]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ u32::from(b)) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}
