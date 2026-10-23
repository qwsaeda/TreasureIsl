//! 7 场景渲染 PNG 导出 bin：走 `GameApp` 真实状态机导航到各场景，
//! `render_to_framebuf` 出完整帧，导出 4x 放大 RGB8 PNG 供肉眼过「画面观感」。
//!
//! 用法：
//!   cargo run -q -p ti-app --features native-shell --bin render-png
//!
//! 输出：`evidence/render_png/<场景>.png`（7 张，704×880，nearest-neighbor 4x）。
//! S15.6 QA017：`forward_render` 开关已退役（S15.4 删 legacy 像素写帧层后渲染
//! **只有**纯代码绘制一态，`route_render` 走 §4 版面，不叠 legacy 素材；两态帧恒等
//! QA014 实证，开关无任何消费方）。出图反映**业务/设计意图**而非逆向数据产物。
//!
//! 与 `sprite_step`（只渲染 .Map 精灵层）互补：本 bin 出**纯代码渲染完整帧**，
//! 回答「立绘/血条/瓦片/文字是否按 §4 版面画对」这类业务观感问题。
//! 场景导航走 `ti_app::forward_nav::navigate`（与 `tests/render_snapshot_forward.rs` 同源）。

use ti_app::forward_nav::{FORWARD_SCENES, navigate};
use ti_app::game_app::GameApp;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf)
}

fn read_local(rel: &str) -> Option<Vec<u8>> {
    std::fs::read(repo_root().join("legacy/data").join(rel)).ok()
}

/// 构建 GameApp（字体 + 精灵 + 地形），与 `treasure_native.rs` 同源。
/// S15.6 QA017：`forward_render` 开关退役后，纯代码渲染是唯一渲染态（注入的
/// 图集/地形仅存诊断用途，不参与写帧）。飞轮 R1 ⑦-11：`.Map` 场景注入已删
/// （`scene_cache` 死字段，legacy `.Map` 归 `legacy/` 参照）。
fn build_app() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    // 真实字体（GoldLand.Fnt + font.dat）
    if let (Some(f), Some(c)) = (read_local("GoldLand.Fnt"), read_local("font.dat")) {
        app.shell_mut().set_font(ti_app::menu_shell::parse_font_bytes(&f, &c));
    }
    // S15.6 QA107：TTF 显式注入块已删——`GameApp::new` 构造壳时经
    // `MenuShell::builtin_ttf_set`（QA128 进程级缓存，EXP-050 回溯定位）自动加载
    // 内置 lxgw-wenkai-reg 三档（96/44/28，QB115），快照与实机/浏览器同源字形。
    // 精灵图集 / 场景 .Map / 地形 注入仅保留诊断用途（S15.4 起不参与写帧）
    if let Some(ui_pak) = read_local("ui.pak") {
        app.set_atlas(&ui_pak);
    }
    if let (Some(pak), Some(map_bytes)) = (read_local("glmap_m01.pak"), read_local("glmapm01.Map"))
    {
        app.set_terrain(&pak, &map_bytes, "glmapm01.Map");
    }
    // 飞轮 R1 ⑦-11：legacy `.Map` 场景注入循环已删（`scene_cache` 零读方死字段；
    // legacy `.Map` 解析归 `legacy/` 参照，契约 1.3）。
    app
}

fn main() {
    let out_dir = repo_root().join("evidence").join("render_png");
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("output dir 创建失败: {e}");
        std::process::exit(1);
    }
    let scale = 1u32;
    let mut exported = 0usize;
    for target in FORWARD_SCENES {
        let mut app = build_app();
        let _actual = navigate(&mut app, target);
        let px = app.render_to_framebuf().to_vec();
        if px.len() != 1080 * 1920 {
            eprintln!("{target} 帧缓冲尺寸异常: {} != 1080*1920", px.len());
            std::process::exit(1);
        }
        let rgb = upscale_rgb565(&px, scale);
        let png = encode_png_rgb(&rgb, 1080 * scale, 1920 * scale);
        let path = out_dir.join(format!("{target}.png"));
        if let Err(e) = std::fs::write(&path, &png) {
            eprintln!("png write 失败 ({}): {e}", path.display());
            std::process::exit(1);
        }
        exported += 1;
        println!("{target:>12}  已导出 {}", path.display());
    }
    println!(
        "\n已导出 {exported}/7 场景 PNG 到 {}（4x 放大，forward 纯代码渲染，供肉眼过画面）",
        out_dir.display()
    );
}

/// RGB565 → 放大 RGB8（nearest-neighbor，与 `sprite_step.rs` 同口径）。
fn upscale_rgb565(fb: &[u16], scale: u32) -> Vec<u8> {
    let out_w = 1080u32 * scale;
    let out_h = 1920u32 * scale;
    let mut out = vec![0u8; (out_w * out_h * 3) as usize];
    for oy in 0..out_h {
        let sy = (oy / scale) as usize;
        for ox in 0..out_w {
            let sx = (ox / scale) as usize;
            let p = fb[sy * 1080 + sx];
            let r5 = (p >> 11) & 0x1F;
            let g6 = (p >> 5) & 0x3F;
            let b5 = p & 0x1F;
            let r = (u32::from(r5) << 3 | u32::from(r5) >> 2) & 0xFF;
            let g = (u32::from(g6) << 2 | u32::from(g6) >> 4) & 0xFF;
            let b = (u32::from(b5) << 3 | u32::from(b5) >> 2) & 0xFF;
            let idx = ((oy as usize) * out_w as usize + ox as usize) * 3;
            out[idx] = r as u8;
            out[idx + 1] = g as u8;
            out[idx + 2] = b as u8;
        }
    }
    out
}

/// 自实现 PNG 编码（RGB8，stored zlib 块 + CRC32，零依赖，与 `render_snapshot` 同口径）。
fn encode_png_rgb(rgb: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row_bytes = width as usize * 3;
    let mut raw: Vec<u8> = Vec::with_capacity(((height as usize) + 1) * row_bytes);
    for y in 0..height as usize {
        raw.push(0u8);
        raw.extend_from_slice(&rgb[y * row_bytes..(y + 1) * row_bytes]);
    }
    let zlib = zlib_compress_stored(&raw);
    let crc_table = build_crc_table();
    let mut out = vec![137u8, 80, 78, 71, 13, 10, 26, 10];
    let chunk = |tag: &[u8; 4], data: &[u8], out: &mut Vec<u8>| {
        out.extend_from_slice(&u32::try_from(data.len()).unwrap_or(u32::MAX).to_be_bytes());
        out.extend_from_slice(tag);
        out.extend_from_slice(data);
        let mut crc_input = Vec::with_capacity(4 + data.len());
        crc_input.extend_from_slice(tag);
        crc_input.extend_from_slice(data);
        out.extend_from_slice(&crc32_calc(&crc_input, &crc_table).to_be_bytes());
    };
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(b"IHDR", &ihdr, &mut out);
    chunk(b"IDAT", &zlib, &mut out);
    chunk(b"IEND", &[], &mut out);
    out
}

fn zlib_compress_stored(data: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.push(0x78);
    out.push(0x01);
    let mut off = 0usize;
    let mut last = false;
    while !last {
        let chunk = &data[off..];
        last = chunk.len() <= 65535;
        let n = if last { chunk.len() } else { 65535 };
        let len = u16::try_from(n).unwrap_or(u16::MAX);
        out.push(u8::from(last));
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&chunk[..n]);
        off += n;
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn build_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut c = u32::from(u8::try_from(i).unwrap_or(u8::MAX));
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }
    table
}

fn crc32_calc(data: &[u8], table: &[u32; 256]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &byte in data {
        let idx = ((c ^ u32::from(byte)) & 0xFF) as usize;
        c = table[idx] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}
