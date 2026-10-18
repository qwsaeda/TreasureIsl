//! S2.5 补实验收：wasm 壳的字形渲染路径 = **真实 `GoldLand.Fnt` 字形**（非伪随机噪点）。
//!
//! 断言四层（对应「假 1」的修复证据）：
//! 1. **逐位相等**：`Raster::for_char` / `Raster::blit` 与 `Glyph::pixel` 完全一致；
//! 2. **几何结构**：`b/d/p/L/A/！` 的笔画位置（与 `font_parity.rs` 同口径复核）；
//! 3. **非噪点**：墨点率落在真实字形区间，且与旧版 seed 伪随机位图**不相等**；
//! 4. **可见回退**：字表缺字（`继`/`续` 不在 1016 字表）与字体整体缺失都必须
//!    画豆腐块，空白字符则必须留空。

use std::fs;
use std::path::{Path, PathBuf};

use ti_app::MENU;
use ti_app::glyph_raster::{FontSet, Raster};
use ti_formats::font::{Charset, Font, GLYPH_ROWS, GLYPH_WIDTH};
use ti_render::framebuffer::Framebuf;

/// 壳里画的中文标题。
const TITLE: &str = "金银岛";
/// 字形墨色（与 `treasure_wasm` 的 `INK_COLOR` 一致）。
const INK_COLOR: u16 = 0xFFFF;
/// 旧版伪随机光栅的种子乘数（S2.5 之前的占位实现）。
const LEGACY_SEED_MUL: u32 = 2_654_435_761;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn font_set() -> FontSet {
    let font_raw =
        fs::read(repo_root().join("legacy").join("data").join("GoldLand.Fnt")).expect("字体存在");
    let parsed = Font::parse("GoldLand.Fnt", &font_raw).expect("字体解析成功");
    let charset_raw =
        fs::read(repo_root().join("legacy").join("data").join("font.dat")).expect("字符表存在");
    let charset = Charset::parse("font.dat", &charset_raw).expect("字符表解析成功");
    let count = parsed.glyph_count();
    let mut glyphs = Vec::with_capacity(count);
    for i in 0..count {
        glyphs
            .push(ti_render::font::Glyph { rows: parsed.glyph(i).expect("字形在表内").rows });
    }
    FontSet { glyphs, chars: charset.chars }
}

/// 旧版占位光栅（`seed = ch * 2_654_435_761`，12 行 × 8 列 MSB-first）。
///
/// 旧实现的 `shift = row * 8 + col` 最大 95，超出 `u32` 位宽；release 构建把位移
/// 掩码成 `& 31`，于是第 4..8 行与第 0..4 行**逐位重复**（周期 4 行）—— 这正是
/// 它一眼就不是字形的原因。此处按 release 语义复刻，用于「噪点已被清除」断言。
fn legacy_noise(ch: char) -> Vec<bool> {
    let seed = (ch as u32).wrapping_mul(LEGACY_SEED_MUL);
    let mut noise = vec![false; GLYPH_WIDTH * usize::from(GLYPH_ROWS)];
    for row in 0..usize::from(GLYPH_ROWS) {
        for col in 0..8usize {
            let shift = u32::try_from((row * 8 + col) & 31).expect("掩码后 < 32");
            if let Some(slot) = noise.get_mut(row * GLYPH_WIDTH + col) {
                *slot = ((seed >> shift) & 1) == 1;
            }
        }
    }
    noise
}

#[test]
fn draw_path_is_bit_identical_to_glyph_pixel() {
    let fs = font_set();
    let mut ink_report = Vec::new();
    for ch in ['A', 'b', 'd', 'p', 'L', '金', '岛', '！'] {
        let glyph = fs.glyph_for(ch).expect("样例字符在字表内");
        let raster = Raster::for_char(Some(&fs), ch);
        assert_eq!(raster, Raster::from_glyph(&glyph), "'{ch}' 光栅应等于真字形");
        for y in 0..usize::from(GLYPH_ROWS) {
            for x in 0..GLYPH_WIDTH {
                assert_eq!(
                    raster.at(x, y),
                    glyph.pixel(x, y),
                    "'{ch}' 像素 ({x},{y}) 应与 Glyph::pixel 一致"
                );
            }
        }
        assert_eq!(raster, Raster::for_char(Some(&fs), ch), "'{ch}' 同参两次调用逐位相同");
        ink_report.push(format!("{ch}:{}", raster.ink_count()));
    }
    eprintln!("[s2.5] 墨点数 {}", ink_report.join(" "));
}

#[test]
fn blit_writes_exactly_the_glyph_pixels_to_the_framebuffer() {
    let fs = font_set();
    let stride = usize::try_from(ti_platform::gfx::CANVAS_W).expect("帧宽(CANVAS)可转 usize");
    let origin_x = 30usize;
    let origin_y = 40usize;
    for ch in ['A', '金'] {
        let glyph = fs.glyph_for(ch).expect("样例字符在字表内");
        let mut fb = Framebuf::new();
        Raster::for_char(Some(&fs), ch).blit(&mut fb, 30, 40, INK_COLOR);
        let pixels = fb.as_slice();
        for y in 0..usize::from(GLYPH_ROWS) {
            for x in 0..GLYPH_WIDTH {
                let actual = pixels[(origin_y + y) * stride + origin_x + x];
                let expected = if glyph.pixel(x, y) { INK_COLOR } else { 0 };
                assert_eq!(actual, expected, "'{ch}' 帧缓冲 ({x},{y}) 应等于字形像素");
            }
        }
        let painted = pixels.iter().filter(|&&p| p == INK_COLOR).count();
        assert_eq!(painted, Raster::from_glyph(&glyph).ink_count(), "'{ch}' 不得多画/少画一个点");
    }
}

#[test]
fn geometric_shapes_stem_from_real_glyphs() {
    let fs = font_set();

    // 'b'：竖杆在左、bowl 在右下、上伸部在顶、下缘留白（防垂直翻转）
    let glyph_b = Raster::for_char(Some(&fs), 'b');
    assert!(glyph_b.at(2, 2) && glyph_b.at(3, 2), "b 竖杆应在左（第 2 行）");
    assert!(glyph_b.at(6, 6) && glyph_b.at(7, 6), "b bowl 应在右侧下部");
    assert!(!glyph_b.at(5, 2), "b 顶部右侧不应有着墨");
    assert!(glyph_b.at(2, 1), "b 应有上伸部");
    assert!(!glyph_b.at(2, 10), "b 下缘应留白");

    // 'd' = 'b' 水平镜像：竖杆在右、bowl 在左
    let glyph_d = Raster::for_char(Some(&fs), 'd');
    assert!(glyph_d.at(5, 2) && glyph_d.at(6, 2), "d 竖杆应在右");
    assert!(glyph_d.at(1, 6), "d bowl 应在左侧下部");
    assert!(!glyph_d.at(1, 2), "d 顶部左侧不应有着墨");

    // 'p'：下伸部在底（防垂直翻转把 p 变 b）
    let glyph_p = Raster::for_char(Some(&fs), 'p');
    assert!(glyph_p.at(2, 10), "p 应有下伸部");
    assert!(!glyph_p.at(2, 1), "p 顶部不应有竖杆");
    assert!(glyph_p.at(6, 4), "p bowl 应在右侧");

    // 'L'：脚衬线向右伸出、顶行无横臂（防水平翻转）
    let glyph_l = Raster::for_char(Some(&fs), 'L');
    assert!(glyph_l.at(4, 1), "L 顶行竖杆处应有着墨");
    assert!(!glyph_l.at(6, 1), "L 顶行右侧不应有着墨");
    assert!(glyph_l.at(6, 9), "L 脚衬线应向右伸出");
    assert!(glyph_l.at(2, 9), "L 底行竖杆处应有着墨");

    // 'A'：尖顶在第 1 行、中横梁、内部留空、下部两腿
    let glyph_a = Raster::for_char(Some(&fs), 'A');
    assert!(glyph_a.at(4, 1), "A 尖顶应在第 1 行");
    assert!(glyph_a.at(6, 6), "A 应有中横梁");
    assert!(!glyph_a.at(4, 4), "A 内部应留空");
    assert!(glyph_a.at(1, 9) && glyph_a.at(6, 9), "A 下部两腿应着墨");

    // '！'：上留白 + 横杠 + 断口 + 点（位图行 [0x000,0x180,0x3C0,...] 的几何含义）
    // 0x180 = bit8|bit7 → x=3,4；0x3C0 = bit9..bit6 → x=2..5
    let glyph_bang = Raster::for_char(Some(&fs), '！');
    assert!(glyph_bang.at(3, 1) && glyph_bang.at(4, 1), "'！' 第 1 行横杠应在中偏右");
    assert!(!glyph_bang.at(5, 1) && !glyph_bang.at(2, 1), "'！' 第 1 行比第 2 行窄");
    assert!(!glyph_bang.at(4, 0) && !glyph_bang.at(4, 11), "'！' 首末行应留白");
    assert!(!glyph_bang.at(0, 1), "'！' 左缘不应有着墨");
    assert!(glyph_bang.at(4, 8), "'！' 下方点应在中列");

    let counts = ['A', 'b', 'd', 'p', 'L', '！', '金', '岛'].map(|ch| {
        let raster = Raster::for_char(Some(&fs), ch);
        (ch, raster.ink_count())
    });
    eprintln!("[s2.5] 几何断言通过，墨点数 {counts:?}");
}

#[test]
fn ink_density_is_plausible_and_not_legacy_noise() {
    let fs = font_set();
    for ch in ['A', 'b', 'd', 'p', 'L'] {
        let raster = Raster::for_char(Some(&fs), ch);
        let ink = raster.ink_count();
        assert!((10..=90).contains(&ink), "'{ch}' 墨点率 {ink}/144 应落在真实字形区间");
        assert_ne!(
            raster.as_slice(),
            legacy_noise(ch).as_slice(),
            "'{ch}' 必须与旧版伪随机位图不同（噪点实现未清除）"
        );
    }
    for ch in ['金', '岛'] {
        let raster = Raster::for_char(Some(&fs), ch);
        let ink = raster.ink_count();
        assert!((20..=130).contains(&ink), "汉字 '{ch}' 墨点率 {ink}/144 应落在合理区间");
        assert_ne!(raster.as_slice(), legacy_noise(ch).as_slice(), "汉字 '{ch}' 不是噪点");
    }
    // '！' 的真实位图有 22 个墨点（font_parity 行向量 2+4+4+4+2+2+2+2）
    let bang = Raster::for_char(Some(&fs), '！');
    assert_eq!(bang.ink_count(), 22, "'！' 墨点数应为实测值 22");
}

#[test]
fn missing_chars_fall_back_to_visible_tofu() {
    let fs = font_set();
    let tofu = Raster::tofu();

    // 豆腐块必须「可见」：四边着墨 + 内部留空
    assert_eq!(tofu.ink_count(), 4 * GLYPH_WIDTH - 4, "豆腐块应是 12×12 空心框");
    assert!(tofu.at(0, 0) && tofu.at(GLYPH_WIDTH - 1, 0), "豆腐块上边应有着墨");
    assert!(
        tofu.at(0, usize::from(GLYPH_ROWS) - 1)
            && tofu.at(GLYPH_WIDTH - 1, usize::from(GLYPH_ROWS) - 1),
        "豆腐块下边应有着墨"
    );
    assert!(!tofu.at(6, 6), "豆腐块内部应留空");

    // 字表缺字 → 豆腐块（实测：'继'/'续' 不在 font.dat 的 1016 字表内）
    assert!(fs.index_of('继').is_none(), "'继' 实测不在字表内");
    assert!(fs.index_of('续').is_none(), "'续' 实测不在字表内");
    for ch in ['继', '续'] {
        let raster = Raster::for_char(Some(&fs), ch);
        assert_eq!(raster, tofu, "'{ch}' 不在字表，必须回退豆腐块而不是空白");
        assert!(raster.ink_count() > 0, "'{ch}' 回退结果必须可见");
    }

    // 字体整体缺失 → 仍然可见
    assert_eq!(Raster::for_char(None, '金'), tofu, "字体未载入时也必须画豆腐块");

    // 空白字符留空（ASCII 空格不在字表内，但不能画豆腐块）
    let space = Raster::for_char(Some(&fs), ' ');
    assert_eq!(space, Raster::blank(), "空白字符应留空");
    assert_eq!(space.ink_count(), 0, "空白字符不应有着墨");
}

#[test]
fn shell_strings_render_with_known_fallback_only() {
    let fs = font_set();
    let mut tofu_seen: Vec<char> = Vec::new();
    let mut check = |text: &str, where_: &str| {
        for ch in text.chars() {
            if ch.is_whitespace() {
                continue;
            }
            if Raster::for_char(Some(&fs), ch) == Raster::tofu() {
                tofu_seen.push(ch);
                // 显式回退白名单：见文件末尾 `KNOWN_BITMAP_FALLBACK`（legacy 位图字表覆盖边界记录，
                // 运行时 TTF 正常渲染）。S15.6 QB150 新增 `scene.index.footer`
                // =「金银岛 v1 · 桌面版」带出 桌/面/版 三字。
                assert!(
                    KNOWN_BITMAP_FALLBACK.contains(&ch),
                    "{where_} 的 '{ch}' 既不在字表也没有显式回退预期（新增 UI 文案带出的新字必须 \
                     先进 KNOWN_BITMAP_FALLBACK，或改用字表已覆盖的表述）"
                );
            }
        }
    };

    check(TITLE, "标题");
    // 菜单标签：全部来自 strings.json 的 menu.* key（S8.3 数据化，EXP-052）。
    // 缺失 key 会回 key 本身（ASCII 裸 key），不触发豆腐块，但仍跑字表覆盖校验。
    let labels = ti_app::shell_ui::menu_labels();
    for (i, item) in MENU.iter().enumerate() {
        check(labels[i], labels[i]);
        let _ = item;
    }
    // S15.6 QB150：index 尾行 = 用户版 i18n `scene.index.footer`（旧开发自检
    // status_texts 已删，诊断走 `stats` feature eprintln，不进 UI 字表覆盖）。
    let footer = ti_app::i18n::sget("scene.index.footer");
    check(footer, "尾行(i18n)");
    // 标题（scene.index.title）现在也来自 strings.json，单独覆盖校验
    let title = ti_app::i18n::sget("scene.index.title");
    check(title, "标题(i18n)");
    tofu_seen.sort_unstable();
    tofu_seen.dedup();
    // S11/S10.4：TTF 字形并入 GPU 阶段后，字表缺字集合以 strings.json 实际文本为准；
    // 缺字只允许落在已知回退字符集内，其余须命中真字形。
    // 白名单单一来源 = `KNOWN_BITMAP_FALLBACK`（上方 check 闭包内定义，两处断言共用，
    // 避免两处硬编码漂移 = EXP-048 重复模式固化）。
    assert!(
        tofu_seen.iter().all(|ch| KNOWN_BITMAP_FALLBACK.contains(ch)),
        "缺字只允许在已知回退字符集内（{KNOWN_BITMAP_FALLBACK:?}），实际 {tofu_seen:?}"
    );
    eprintln!("[s2.5] 菜单/标题/状态行缺字 = {tofu_seen:?}（其余均为真字形）");
}

/// 已知 **legacy 位图字表**（`GoldLand.Fnt` + `font.dat`）覆盖边界外的 UI 用字。
///
/// 语义澄清（避免误读为「已知缺陷」）：这些字在运行时由 `lxgw-wenkai-reg.ttf`
/// TTF 正常光栅渲染（44/96/28 三档）；仅当 TTF 不可用时才回退位图态，此时它们
/// 落在字表外 → 豆腐块。本清单 = **legacy 字表覆盖边界的显式记录**，新增 UI 文案
/// 带出表外新字时必须先进此清单（或改用已覆盖表述），否则本测试 FAIL 报警。
const KNOWN_BITMAP_FALLBACK: [char; 8] = ['继', '续', '新', '游', '戏', '桌', '面', '版'];
