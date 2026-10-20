//! PNG 编码（无依赖实现：zlib stored + CRC32）—— 渲染框架的像素编解码归属。
//!
//! 供 `ti-app` headless 导出与 `ti-engine` Bevy 原生渲染截图共用（单一实现）。

/// RGB8 → PNG（`color_type=2` 真彩）。
#[must_use]
pub fn encode_png_rgb(rgb: &[u8], width: u32, height: u32) -> Vec<u8> {
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

/// RGBA8 → PNG（`color_type=6` 真彩带 alpha；Bevy GPU 读回用）。
/// RGBA8 → PNG（`color_type=6` 真彩带 alpha；Bevy GPU 读回用）。
#[must_use]
pub fn encode_png_rgba(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row_bytes = width as usize * 4;
    let mut raw: Vec<u8> = Vec::with_capacity(((height as usize) + 1) * row_bytes);
    for y in 0..height as usize {
        raw.push(0u8);
        raw.extend_from_slice(&rgba[y * row_bytes..(y + 1) * row_bytes]);
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
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(b"IHDR", &ihdr, &mut out);
    chunk(b"IDAT", &zlib, &mut out);
    chunk(b"IEND", &[], &mut out);
    out
}
