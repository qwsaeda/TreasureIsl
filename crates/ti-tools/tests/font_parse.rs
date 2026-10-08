//! S1.5 验收：`GoldLand.Fnt` 结构完整 + 0 未知字节 + 往返逐字节无损。
//!
//! S1.5b 修正：文件**无头部**，24384B = 1016 字形 × 24B（旧记录「头部 24B 字宽表」
//! 实为第 0 个字形，对应 `font.dat` 首字符 `！`）。位图解码见 `font_parity`。

use std::path::{Path, PathBuf};

use ti_formats::font::{Font, GLYPH_BYTES, GLYPH_ROWS, GLYPH_WIDTH};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[test]
fn real_font_parses_with_zero_unknown_bytes() {
    let raw = std::fs::read(repo_root().join("data").join("GoldLand.Fnt")).expect("字体存在");
    assert_eq!(raw.len(), 24_384, "F-303：GoldLand.Fnt 应为 24384 字节");
    let f = Font::parse("GoldLand.Fnt", &raw).expect("解析成功");
    assert_eq!(raw.len() % GLYPH_BYTES, 0, "24384 必须整除 24");
    assert_eq!(f.glyph_count(), 1016, "1016 个字形（无头部）");
    // 往返逐字节无损 = 0 未知字节
    assert_eq!(f.to_bytes(), raw, "往返必须逐字节一致");
    eprintln!("[s1.5] {} 个字形，往返无损", f.glyph_count());
}

#[test]
fn glyph_bits_use_low_byte_plus_high_nibble() {
    // 实测：u16 LE 的 bit8~bit11 恒为 0 → 每行 12 像素（低 8 位 + 高 4 位）
    let raw = std::fs::read(repo_root().join("data").join("GoldLand.Fnt")).expect("字体存在");
    let f = Font::parse("GoldLand.Fnt", &raw).expect("解析成功");
    let mut checked = 0usize;
    for g in &f.glyphs {
        assert_eq!(g.len(), GLYPH_BYTES, "字形长度应为 {GLYPH_BYTES}");
        for row in 0..GLYPH_ROWS as usize {
            let w = u16::from_le_bytes([g[row * 2], g[row * 2 + 1]]);
            assert_eq!(w & 0x0F00, 0, "bit8~bit11 应恒为 0");
            checked += 1;
        }
    }
    assert_eq!(checked, f.glyph_count() * GLYPH_ROWS as usize);
    assert_eq!(GLYPH_WIDTH, 12);
}

#[test]
fn json_export_matches_structure() {
    let raw = std::fs::read(repo_root().join("data").join("GoldLand.Fnt")).expect("字体存在");
    let f = Font::parse("GoldLand.Fnt", &raw).expect("解析成功");
    let j = f.to_json();
    assert!(j.contains("\"glyph_width\": 12"), "{j}");
    assert!(j.contains("\"glyph_rows\": 12"), "{j}");
    assert!(j.contains("\"glyph_count\": 1016"), "{j}");
}

#[test]
fn font_dat_starts_with_utf16_bom() {
    // F-303（S1.5b 修正）：font.dat = UTF-16LE+BOM 的字符集清单（非字宽表）
    let raw = std::fs::read(repo_root().join("data").join("font.dat")).expect("font.dat 存在");
    assert_eq!(raw.len(), 2034, "F-303：font.dat 应为 2034 字节");
    assert_eq!(&raw[..2], &[0xFF, 0xFE], "应以 UTF-16 BOM 开头");
}
