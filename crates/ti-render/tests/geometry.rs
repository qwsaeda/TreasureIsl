//! `ti-render` 几何层单测（F21 + F6）：
//!
//! - [`IsoProjection`] 正/逆变换互逆 + tile 退化的除零早退（F6：原 `screen_to_cell`
//!   分母 `2 * tile_w * tile_h`，tile 为 0 时整数除零 panic）；
//! - `title_bar` / `menu_item` / `bar_slot` / `panel` / `wood_frame` 的绘制矩形
//!   落点与层序（契约 5：门禁判据锚在代码绘制区，纯几何先提成可单测函数）。
//!
//! 判据用「录制 Canvas」：记录每次 `fill_rect` 的 (x, y, w, h, color)，
//! 断言精确矩形序列——比像素基线更早失败、且能定位到具体绘制步骤。

use ti_render::canvas::Canvas;
use ti_render::iso::IsoProjection;
use ti_render::shell::{bar_slot, menu_item, panel, title_bar, wood_frame};
use ti_render::theme::Theme;

/// 录制型画布：只记录矩形，不画像素。
#[derive(Debug, Default)]
struct Recorder {
    rects: Vec<(i32, i32, i32, i32, u16)>,
    clears: usize,
}

impl Canvas for Recorder {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.rects.push((x, y, w, h, color));
    }
    fn draw_text(&mut self, _text: &str, _x: i32, _y: i32, _color: u16) {}
    fn draw_title_text(&mut self, _text: &str, _x: i32, _y: i32, _color: u16) {}
    fn draw_hint_text(&mut self, _text: &str, _x: i32, _y: i32, _color: u16) {}
    fn text_width(&self, _text: &str) -> i32 {
        0
    }
    fn title_text_width(&self, _text: &str) -> i32 {
        0
    }
    fn hint_text_width(&self, _text: &str) -> i32 {
        0
    }
    fn glyph_height(&self) -> i32 {
        44
    }
    fn title_glyph_height(&self) -> i32 {
        96
    }
    fn hint_glyph_height(&self) -> i32 {
        28
    }
    fn clear_frame(&mut self, _color: u16) {
        self.clears += 1;
    }
    fn pixel_at(&self, _x: i32, _y: i32) -> u16 {
        0
    }
}

// === IsoProjection（F6 / EXP-067 正逆成对）===

/// 正/逆变换互逆：`screen_to_cell(cell_center(r, c))` 回到 `(r, c)`。
///
/// 逆变换是整数除法，格心恰在菱形顶点邻域时可能截断 ±1（投影本身按格心取整），
/// 故判据取 |误差| ≤ 1；触摸命中场景由 `touch.rs` 的 20×20 严格用例锁。
#[test]
fn iso_projection_round_trip() {
    let proj = IsoProjection::new(540, 520, 25, 13);
    for row in 0..8 {
        for col in 0..8 {
            let (cx, cy) = proj.cell_center(row, col);
            let (r2, c2) = proj.screen_to_cell(cx, cy);
            assert!(
                (r2 - row).abs() <= 1 && (c2 - col).abs() <= 1,
                "往返失败 ({row},{col}) -> ({r2},{c2})"
            );
        }
    }
}

/// tile 退化不得 panic：分母为 0 时早退 `(0, 0)`（原实现整数除零）。
#[test]
fn iso_projection_degenerate_tile_does_not_panic() {
    for (tw, th) in [(0, 13), (25, 0), (0, 0)] {
        let proj = IsoProjection::new(540, 520, tw, th);
        assert!(proj.is_degenerate());
        assert_eq!(proj.screen_to_cell(100, 200), (0, 0), "tile_w={tw} tile_h={th}");
    }
    assert!(!IsoProjection::new(540, 520, 25, 13).is_degenerate());
}

/// `cell` 返回菱形顶角，`cell_center` = 顶角 + (`tile_w`, `tile_h`)。
#[test]
fn iso_projection_cell_center_offset() {
    let proj = IsoProjection::new(540, 520, 25, 13);
    let (x, y) = proj.cell(0, 0);
    assert_eq!((x, y), (540, 520));
    assert_eq!(proj.cell_center(0, 0), (565, 533));
    assert_eq!(proj.tile_size(), (25, 13));
}

// === 外壳组件几何（契约 5：纯几何可单测）===

/// `title_bar` 的四个绘制层落点（顺序 = 层序，见 `08_render_arch.md` §4）。
#[test]
fn title_bar_geometry() {
    let theme = Theme::default();
    let mut rec = Recorder::default();
    title_bar(&mut rec, &theme, 90);
    let geom: Vec<(i32, i32, i32, i32)> =
        rec.rects.iter().map(|(x, y, w, h, _)| (*x, *y, *w, *h)).collect();
    assert_eq!(
        &geom[..4],
        &[
            (12, 12, 1056, 90), // 木纹外框（h 来自参数）
            (16, 16, 1048, 82), // 羊皮纸内框（h - 8）
            (20, 20, 8, 74),    // 左侧金色 accent 竖条（h - 16）
            (16, 82, 1048, 3),  // 底部细线
        ],
        "title_bar 前四层几何漂移（改 h 须同步复核）"
    );
    // 右侧罗盘装饰：横针 28×3 + 竖针 3×28 + 中心点 4×4（cx = 1080 - 48 = 1032，mid = 16+(90-8)/2 = 57）
    assert_eq!(
        &geom[4..],
        &[(1032 - 14, 57 - 1, 28, 3), (1032 - 1, 57 - 14, 3, 28), (1032 - 2, 57 - 2, 4, 4),],
        "罗盘装饰几何漂移"
    );
}

/// `menu_item`：未选中基色 + 8px 描边四边 + bevel 四条。
#[test]
fn menu_item_geometry() {
    let theme = Theme::default();
    let mut rec = Recorder::default();
    menu_item(&mut rec, &theme, 100, 200, 900, 170, false, false);
    let geom: Vec<(i32, i32, i32, i32, u16)> = rec.rects.clone();
    assert_eq!(geom[0], (100, 200, 900, 170, theme.menu_bg), "未选中基色");
    assert_eq!(geom[1], (100, 200, 900, 8, theme.menu_border), "上描边 8px");
    assert_eq!(geom[2], (100, 200 + 170 - 8, 900, 8, theme.menu_border), "下描边 8px");
    assert_eq!(geom[3], (100, 208, 8, 170 - 16, theme.menu_border), "左描边 8px");
    assert_eq!(geom[4], (100 + 900 - 8, 208, 8, 170 - 16, theme.menu_border), "右描边 8px");
    assert_eq!(geom[5], (108, 208, 900 - 16, 2, theme.bevel_hi), "bevel 凸起上");
    assert_eq!(geom[6], (108, 208, 2, 170 - 16, theme.bevel_hi), "bevel 凸起左");
    assert_eq!(geom[7], (108, 200 + 170 - 10, 900 - 16, 2, theme.bevel_shadow), "bevel 阴影下");
    assert_eq!(geom[8], (100 + 900 - 10, 208, 2, 170 - 16, theme.bevel_shadow), "bevel 阴影右");
    assert_eq!(geom.len(), 9, "未选中不应画铆钉");
}

/// `menu_item` 选中态：基色 + 铆钉（左右各 1 个 8×8）。
#[test]
fn menu_item_chosen_has_rivets() {
    let theme = Theme::default();
    let mut rec = Recorder::default();
    menu_item(&mut rec, &theme, 0, 0, 200, 100, true, false);
    assert_eq!(rec.rects.first().copied(), Some((0, 0, 200, 100, theme.gold_bright)));
    let tail: Vec<(i32, i32, i32, i32, u16)> = rec.rects[9..].to_vec();
    assert_eq!(
        tail,
        vec![
            (24, 100 / 2 - 4, 8, 8, theme.gold_bright),
            (200 - 32, 100 / 2 - 4, 8, 8, theme.gold_bright),
        ],
        "选中态铆钉落点"
    );
}

/// `pressed` 时 bevel 高光/阴影互换（凹陷）。
#[test]
fn menu_item_pressed_inverts_bevel() {
    let theme = Theme::default();
    let mut rec = Recorder::default();
    menu_item(&mut rec, &theme, 0, 0, 200, 100, false, true);
    assert_eq!(rec.rects[5].4, theme.bevel_shadow, "按下时上边取阴影");
    assert_eq!(rec.rects[7].4, theme.bevel_hi, "按下时下边取高光");
}

/// `bar_slot` / `panel` / `wood_frame` 的基础落点。
#[test]
fn slot_panel_frame_geometry() {
    let theme = Theme::default();
    let mut rec = Recorder::default();
    bar_slot(&mut rec, &theme, 60, 170, 440, 48);
    assert_eq!(rec.rects[0], (60, 170, 440, 48, theme.hilite), "血条槽底");
    assert_eq!(rec.rects[1], (64, 174, 440 - 8, 2, theme.bevel_shadow), "槽内上阴影");

    let mut rec2 = Recorder::default();
    panel(&mut rec2, &theme, 90, 220, 900, 1480);
    assert_eq!(
        (rec2.rects[0].0, rec2.rects[0].1, rec2.rects[0].2, rec2.rects[0].3),
        (90, 220, 900, 1480),
        "面板首层为整块底"
    );
    assert!(rec2.rects.len() >= 4, "面板应有木纹细边 + bevel 多层");

    let mut rec3 = Recorder::default();
    wood_frame(&mut rec3, &theme);
    let frame: Vec<(i32, i32, i32, i32)> =
        rec3.rects.iter().map(|(x, y, w, h, _)| (*x, *y, *w, *h)).collect();
    for edge in [(0, 0, 12, 1920), (1068, 0, 12, 1920), (0, 0, 1080, 12), (0, 1908, 1080, 12)] {
        assert!(frame.contains(&edge), "wood_frame 缺 12px 外框 {edge:?}，实际 {frame:?}");
    }
}
