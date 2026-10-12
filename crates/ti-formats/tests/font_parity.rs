//! P4-3 / S1.5b 验收：12×12 字库位图可渲染为可读汉字/字母。
//!
//! 实测结论（`data/GoldLand.Fnt` + `data/font.dat`）：
//! - 行序**自上而下**、位序 **MSB-first 在左**（几何断言用 `b/d/p/L/A` 的结构特征验证）；
//! - `font.dat` = UTF-16 字符集清单，与字形下标 1:1（1016 项，`记` 重复 @981/@996）。

use std::fs;
use std::path::{Path, PathBuf};

use ti_formats::font::{Charset, Font};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load_font() -> Font {
    let raw =
        fs::read(repo_root().join("legacy").join("data").join("GoldLand.Fnt")).expect("字体存在");
    Font::parse("GoldLand.Fnt", &raw).expect("字体解析成功")
}

fn load_charset() -> Charset {
    let raw =
        fs::read(repo_root().join("legacy").join("data").join("font.dat")).expect("font.dat 存在");
    Charset::parse("font.dat", &raw).expect("字符集解析成功")
}

fn art(font: &Font, cs: &Charset, ch: char) -> Option<Vec<String>> {
    let g = font.glyph_for(cs, ch)?;
    Some(
        g.rows
            .iter()
            .map(|r| (0..12).map(|x| if (r >> (11 - x)) & 1 == 1 { '#' } else { '.' }).collect())
            .collect(),
    )
}

#[test]
fn charset_and_glyphs_are_one_to_one() {
    let font = load_font();
    let cs = load_charset();
    assert_eq!(cs.len(), 1016, "font.dat 应为 1016 个字符");
    assert_eq!(font.glyph_count(), cs.len(), "字形与字符必须 1:1");

    let unique: std::collections::HashSet<char> = cs.chars.iter().copied().collect();
    assert_eq!(unique.len(), 1015, "唯一字符应为 1015");
    assert_eq!(cs.chars.iter().filter(|&&c| c == '记').count(), 2, "'记' 应出现 2 次");
    assert_eq!(cs.index_of('记'), Some(981), "首个 '记' 在 981");
    assert_eq!(cs.chars[996], '记', "次个 '记' 在 996");
    // 两处字形字节完全相同 → 首现位查表无歧义
    assert_eq!(font.glyphs[981], font.glyphs[996], "重复 '记' 的字形应全同");

    // 关键字符下标（实测）
    assert_eq!(cs.index_of('！'), Some(0), "'！' 是首个字符");
    assert_eq!(cs.index_of('A'), Some(317));
    assert_eq!(cs.index_of('金'), Some(714));
    assert_eq!(cs.index_of('银'), Some(800));
    assert_eq!(cs.index_of('岛'), Some(241));
    assert_eq!(cs.index_of('记'), Some(981));
    eprintln!("[p4-3] charset 1016 项与字形 1:1，唯一 1015，'记'@981/996 字形全同");
}

#[test]
fn glyph_bit_and_row_order_render_readable_shapes() {
    let font = load_font();
    let cs = load_charset();

    // 第 0 个字形（'！'）：旧「字宽表」值实为该字形的 12 行位图
    let g0 = font.glyph(0).expect("字形 0 存在");
    assert_eq!(
        g0.rows,
        [0x000, 0x180, 0x3C0, 0x3C0, 0x3C0, 0x180, 0x180, 0x000, 0x180, 0x180, 0x000, 0x000],
        "'！' 位图行（上留白 + 横杠 + 断口 + 点 + 下留白）"
    );

    // 行序自上而下 + MSB 在左：'b' 竖杆在左、bowl 在右、上伸部在顶、下缘留白
    let glyph_b = font.glyph_for(&cs, 'b').expect("'b' 存在");
    assert!(glyph_b.pixel(2, 2) && glyph_b.pixel(3, 2), "b 竖杆应在左（第 2 行）");
    assert!(glyph_b.pixel(6, 6) && glyph_b.pixel(7, 6), "b bowl 应在右侧下部");
    assert!(!glyph_b.pixel(5, 2), "b 顶部右侧不应有着墨");
    assert!(glyph_b.pixel(2, 1), "b 应有上伸部（顶部竖杆）");
    assert!(!glyph_b.pixel(2, 10), "b 下缘应留白（防垂直翻转）");

    // 'd' = 'b' 的水平镜像：竖杆在右、bowl 在左
    let glyph_d = font.glyph_for(&cs, 'd').expect("'d' 存在");
    assert!(glyph_d.pixel(5, 2) && glyph_d.pixel(6, 2), "d 竖杆应在右");
    assert!(glyph_d.pixel(1, 6), "d bowl 应在左侧下部");
    assert!(!glyph_d.pixel(1, 2), "d 顶部左侧不应有着墨");

    // 'p' = 下伸部在底（防垂直翻转把 p 变 b）
    let glyph_p = font.glyph_for(&cs, 'p').expect("'p' 存在");
    assert!(glyph_p.pixel(2, 10), "p 应有下伸部（底部竖杆）");
    assert!(!glyph_p.pixel(2, 1), "p 顶部不应有竖杆");
    assert!(glyph_p.pixel(6, 4), "p bowl 应在右侧");

    // 'L'：脚衬线向右伸出、顶行仅竖杆（防水平翻转/垂直翻转）
    let glyph_l = font.glyph_for(&cs, 'L').expect("'L' 存在");
    assert!(glyph_l.pixel(4, 1), "L 顶行应有衬线（限竖杆附近）");
    assert!(!glyph_l.pixel(6, 1), "L 顶行右侧不应有着墨（防水平翻转）");
    assert!(glyph_l.pixel(6, 9), "L 脚衬线应向右伸出");
    assert!(glyph_l.pixel(2, 9), "L 底行竖杆处应有着墨");

    // 'A'：尖顶在第 1 行（行序）、中横梁、下部两腿、内部留空
    let glyph_a = font.glyph_for(&cs, 'A').expect("'A' 存在");
    assert!(glyph_a.pixel(4, 1), "A 尖顶应在第 1 行");
    assert!(glyph_a.pixel(6, 6), "A 应有中横梁");
    assert!(!glyph_a.pixel(4, 4), "A 内部应留空");
    assert!(glyph_a.pixel(1, 9) && glyph_a.pixel(6, 9), "A 下部两腿应着墨");

    eprintln!("[p4-3] 行序/位序几何断言全部通过（b/d/p/L/A + '！' 行向量）");
}

#[test]
fn render_sample_text_writes_evidence() {
    let font = load_font();
    let cs = load_charset();
    let sample = "金银岛 TreasureIsl ABLP bdip 0123";
    let missing: Vec<char> =
        sample.chars().filter(|c| !c.is_whitespace() && cs.index_of(*c).is_none()).collect();
    assert!(missing.is_empty(), "样例字符必须都在字符集内: {missing:?}");

    let mut lines: Vec<String> = Vec::new();
    for ch in sample.chars() {
        if ch.is_whitespace() {
            continue;
        }
        let rows = art(&font, &cs, ch).expect("样例字符必在字符集内");
        if lines.is_empty() {
            lines = rows;
        } else {
            for (dst, src) in lines.iter_mut().zip(rows) {
                dst.push(' ');
                dst.push_str(&src);
            }
        }
    }
    assert_eq!(lines.len(), 12, "渲染结果应为 12 行");

    // 落盘留证：逐行人工可读的 ASCII 点阵
    let out = repo_root().join("evidence").join("font_render.txt");
    let mut text = String::from("# S1.5b font render evidence (12x12, '#'=ink)\n");
    for line in &lines {
        text.push_str(line);
        text.push('\n');
    }
    fs::write(&out, text.as_bytes()).expect("证据文件可写");
    eprintln!("[p4-3] 样例渲染 12 行 -> {}", out.display());
}
