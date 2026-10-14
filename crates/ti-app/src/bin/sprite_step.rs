//! 逐精灵渲染验证 bin（EXP-076 §7 用户逐步确认流）：
//! 每次只 blit 指定记录号的单个精灵到干净 176×220 帧缓冲，输出可看的 PNG。
//! 用法：
//!   cargo run -p ti-app --features native-shell --bin sprite-step -- index <记录号>
//! 输出：evidence/sprite_step/index_<记录号>.png（放大 4x 便于查看）+ 该精灵元信息。
//! 渲染当前记录号及其之前的所有精灵（累积层，z-order 文件顺序），PNG 为累积态。

use ti_app::render_pipeline::{FB_H, FB_W, clip_rect};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scene_name = args.get(1).map(|s| s.as_str()).unwrap_or("index");
    let Some(target_s) = args.get(2) else {
        eprintln!("用法: sprite-step <scene> <record#>");
        return;
    };
    let target: usize = match target_s.parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("record# must be a number");
            return;
        }
    };

    let pkg = match std::fs::read("legacy/data/ui.pak") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ui.pak 读取失败: {e}");
            return;
        }
    };
    let mut atlas = match ti_app::sprite_atlas::SpriteAtlas::new(&pkg) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("atlas 构建失败: {e}");
            return;
        }
    };
    let map_path = format!("legacy/data/{scene_name}.Map");
    let map_bytes = match std::fs::read(&map_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{map_path} 读取失败: {e}");
            return;
        }
    };
    let scene = match ti_formats::mapfile::MapScene::parse(scene_name, &map_bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("scene 解析失败: {e}");
            return;
        }
    };
    let controls = match ti_formats::mapfile::MapControl::parse_words(scene_name, &scene.words) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("controls 解析失败: {e}");
            return;
        }
    };
    if target >= controls.len() {
        eprintln!("记录号 {target} 超出范围（共 {0} 条）", controls.len());
        return;
    }

    // 累积 blit 记录 0..=target（z-order 文件顺序正序，与 blit_scene_frame 一致：
    // record 0 先画=底层，record N 后画=顶层，对齐原版 sub_443FC next 遍历）
    let mut fb = ti_platform::gfx::Framebuf::new();
    fb.clear(0);
    let mut blitted: Vec<(usize, u32)> = Vec::new();
    for (i, ctrl) in controls.iter().enumerate() {
        if i > target {
            break;
        }
        if ctrl.f1 == 0 {
            continue;
        }
        let Some(sprite) = atlas.sprite(ctrl.f1) else {
            continue;
        };
        let sw = sprite.width as usize;
        let sh = sprite.height as usize;
        let Some(_clip) = clip_rect(ctrl.f2 as usize, ctrl.f3 as usize, sw, sh) else {
            continue; // 越界早退（契约 §3）
        };
        ti_app::sprite_atlas::SpriteAtlas::blit_sprite(
            &sprite, ctrl.f2, ctrl.f3, &mut fb, 0, ctrl.f4,
        );
        blitted.push((i, ctrl.f1));
    }

    // 输出 4x 放大的 PNG（176x220 → 704x880），便于肉眼查看
    let scale = 4u32;
    let out_w = FB_W as u32 * scale;
    let out_h = FB_H as u32 * scale;
    let rgb = upscale_rgb565(fb.as_slice(), scale, out_w, out_h);

    let out_dir = "evidence/sprite_step";
    let _ = std::fs::create_dir_all(out_dir);
    let out_path = format!("{out_dir}/{scene_name}_{target}.png");
    let png = encode_png_rgb(&rgb, out_w, out_h);
    match std::fs::write(&out_path, png) {
        Ok(_) => println!("已生成: {out_path}"),
        Err(e) => {
            eprintln!("PNG 写入失败: {e}");
            return;
        }
    }

    // 打印该记录元信息
    let ctrl = &controls[target];
    let sprite_info = match atlas.sprite(ctrl.f1) {
        Some(s) => format!("{}x{}", s.width, s.height),
        None => "none".to_owned(),
    };
    println!(
        "record {target}: f1={} origin({},{}) sprite_size={} => cumulative blit {} sprites",
        ctrl.f1,
        ctrl.f2,
        ctrl.f3,
        sprite_info,
        blitted.len()
    );
    println!(
        "累积层(0..={target}): {}",
        blitted.iter().map(|(i, f1)| format!("[{i}]=f1:{f1}")).collect::<Vec<_>>().join(" ")
    );
}

/// RGB565 → 放大 RGB8（nearest-neighbor，便于肉眼查看）。
fn upscale_rgb565(fb: &[u16], scale: u32, out_w: u32, out_h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (out_w * out_h * 3) as usize];
    for oy in 0..out_h {
        let sy = oy / scale;
        for ox in 0..out_w {
            let sx = ox / scale;
            let p = fb[(sy as usize) * FB_W + sx as usize];
            let r5 = (p >> 11) & 0x1F;
            let g6 = (p >> 5) & 0x3F;
            let b5 = p & 0x1F;
            let r = ((r5 as u32) << 3 | (r5 as u32) >> 2) & 0xFF;
            let g = ((g6 as u32) << 2 | (g6 as u32) >> 4) & 0xFF;
            let b = ((b5 as u32) << 3 | (b5 as u32) >> 2) & 0xFF;
            let idx = ((oy as usize) * out_w as usize + ox as usize) * 3;
            out[idx] = r as u8;
            out[idx + 1] = g as u8;
            out[idx + 2] = b as u8;
        }
    }
    out
}

/// 自实现 PNG 编码（RGB8，stored zlib 块 + CRC32，零依赖，与 render_snapshot 同口径）。
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
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
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
    // 按 65535 块切分（stored 块单块 len <= 65535）
    let mut out: Vec<u8> = Vec::new();
    out.push(0x78);
    out.push(0x01);
    let mut off = 0usize;
    let mut last = false;
    while !last {
        let chunk = &data[off..];
        last = chunk.len() <= 65535;
        let n = if last { chunk.len() } else { 65535 };
        let len = n as u16;
        out.push(if last { 0x01 } else { 0x00 });
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&chunk[..n]);
        off += n;
    }
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
