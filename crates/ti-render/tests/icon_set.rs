//! S17.3 图标集测试：14 枚逐枚真画出墨点（0 空图标）+ tint 真生效 + 越界 id 静默。
//!
//! 注册表完备性（14 条 / id 连续 / 绑定引用无悬空）在 `ti-core` 与 doccheck2 第 24 项断言；
//! 本文件只管**渲染事实**：每枚 id 都得有墨点，否则 = 注册了但没画图形。
//!
//! 判据（`docs/process/10_ui_redesign.md` §5）：
//! 1. 14 枚 id 全部在集内（`is_valid`）且逐枚墨点 > 40（阈值 = 最小图标 rect 组合面积下界）；
//! 2. 不同 tint 形状一致（同墨点数）且新 tint 颜色真实上屏；
//! 3. 越界 id 画 0 墨点（不 panic、不留坑）。

use ti_render::canvas::Canvas;
use ti_render::framebuffer::{Framebuf, LOGICAL_W_I32};
use ti_render::icons;
use ti_render::theme::Theme;

/// 录一次绘制的最小 Canvas（图标只用 `fill_rect`；文本方法 no-op 计 0 宽）。
struct Counting {
    fb: Framebuf,
}

impl Counting {
    fn new() -> Self {
        let mut c = Self { fb: Framebuf::new() };
        c.fb.clear(0x0000);
        c
    }
    fn count(&self, color: u16) -> usize {
        self.fb.as_slice().iter().filter(|&&p| p == color).count()
    }
}

impl Canvas for Counting {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.fb.fill_rect(x, y, w, h, color);
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
    fn clear_frame(&mut self, color: u16) {
        self.fb.clear(color);
    }
    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        let idx = usize::try_from(y * LOGICAL_W_I32 + x).unwrap_or(0);
        self.fb.as_slice().get(idx).copied().unwrap_or(0)
    }
}

fn draw_icon(id: u32, tint: u16) -> Counting {
    let mut c = Counting::new();
    icons::icon(&mut c, &Theme::DEFAULT, id, 100, 100, tint);
    c
}

#[test]
fn all_fourteen_ids_are_valid_and_draw_ink() {
    for id in 0..icons::ICON_COUNT {
        assert!(icons::is_valid(id), "id {id} 应属集内");
        let n = draw_icon(id, Theme::DEFAULT.gold).count(Theme::DEFAULT.gold);
        assert!(n > 40, "图标 id {id} 墨点 {n} 过少（空图标：注册了但没画图形）");
    }
}

#[test]
fn tint_actually_applies() {
    let theme = Theme::DEFAULT;
    let gold = draw_icon(icons::ICON_CASTLE, theme.gold).count(theme.gold);
    let wood = draw_icon(icons::ICON_CASTLE, theme.text_on_wood).count(theme.text_on_wood);
    assert_eq!(gold, wood, "同一图标不同 tint 墨点数应一致（形状相同，仅换色）");
    assert!(wood > 40, "选中 tint（text_on_wood）墨点 {wood} 过少");
}

#[test]
fn out_of_range_id_draws_nothing() {
    let theme = Theme::DEFAULT;
    let c = draw_icon(icons::ICON_COUNT, theme.gold);
    assert_eq!(c.count(theme.gold), 0, "越界 id 应画 0 墨点（不 panic、不留坑）");
}
