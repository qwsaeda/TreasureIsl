//! 绘制列表（S18.7a）：**后端无关**的绘制命令记录器。
//!
//! 用途：让「场景绘制代码」与「渲染后端」解耦 ——
//! - CPU 模式：绘制列表 → `ti-render` 帧缓冲（macroquad 壳，S18.8 退役后删）；
//! - Bevy 模式：绘制列表 → Bevy 实体（`Sprite` / `Text2d` / 烘焙 `Image`）。
//!
//! 场景代码**一行不改**：`SceneHost` 的绘制目标可插拔（`RenderHost::draw_target`），
//! 同一份绘制代码产出同一份命令序列 → 两后端同画面。
//!
//! 本模块**不依赖 Bevy**（`ti-render` 是渲染框架层，零引擎依赖）。

use crate::canvas::Canvas;
use crate::font::TtfFont;
use crate::theme::Theme;

/// 绘制命令（后端无关）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DrawOp {
    /// 清屏（RGB565）。
    Clear(u16),
    /// 实心矩形。
    Rect {
        /// 左缘 x。
        x: i32,
        /// 上缘 y。
        y: i32,
        /// 宽。
        w: i32,
        /// 高。
        h: i32,
        /// 颜色（RGB565）。
        color: u16,
    },
    /// 文本（三档）。
    Text {
        /// 文本内容。
        text: String,
        /// 左缘 x。
        x: i32,
        /// 顶缘 y。
        y: i32,
        /// 字号档（title/body/hint）。
        tier: Tier,
        /// 颜色（RGB565）。
        color: u16,
    },
    /// 贴图（资源路径 + 目标矩形 + 色调；Bevy 侧走资产服务器，CPU 侧读文件解码）。
    Image {
        /// 资源路径（相对 `assets/` 根，如 `ui/board.png`）。
        asset: String,
        /// 左缘 x。
        x: i32,
        /// 顶缘 y。
        y: i32,
        /// 目标宽。
        w: i32,
        /// 目标高。
        h: i32,
        /// 色调（RGB565；`0xFFFF` 白 = 无色调，S21.14 图标图片化跨层 tint）。
        tint: u16,
        /// 绕目标矩形中心顺时针旋转（**毫度**；`0` = 不旋转，S27.5 环形卡径向）。
        rot_mdeg: i32,
    },
}

/// 文本档位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// 正文档。
    Body,
    /// 标题档。
    Title,
    /// 提示档。
    Hint,
}

/// 绘制列表：按顺序记录绘制命令（供任意后端执行）。
#[derive(Debug, Clone, Default)]
pub struct DrawList {
    /// 命令序列（顺序 = 绘制顺序 = 层序）。
    pub ops: Vec<DrawOp>,
    /// 主题（执行端度量用：文本宽度/字高）。
    pub theme: Theme,
    /// 三档 TTF（度量用；与 CPU 路径同源 → 版面一致）。
    pub ttf: TtfSet,
}

/// 三档 TTF 句柄集合（度量用）。
#[derive(Debug, Clone, Default)]
pub struct TtfSet {
    /// 正文档 TTF。
    pub body: Option<TtfFont>,
    /// 标题档 TTF。
    pub title: Option<TtfFont>,
    /// 提示档 TTF。
    pub hint: Option<TtfFont>,
}

impl DrawList {
    /// 新建（默认主题）。
    #[must_use]
    pub fn new(theme: Theme) -> Self {
        Self { ops: Vec::new(), theme, ttf: TtfSet::default() }
    }

    /// 清空命令（下一帧重录）。
    pub fn clear(&mut self) {
        self.ops.clear();
    }

    /// 命令数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

impl Canvas for DrawList {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.ops.push(DrawOp::Rect { x, y, w, h, color });
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.ops.push(DrawOp::Text { text: text.to_owned(), x, y, tier: Tier::Body, color });
    }

    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.ops.push(DrawOp::Text { text: text.to_owned(), x, y, tier: Tier::Title, color });
    }

    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.ops.push(DrawOp::Text { text: text.to_owned(), x, y, tier: Tier::Hint, color });
    }

    fn text_width(&self, text: &str) -> i32 {
        crate::text::tier_text_width(text, self.ttf.body.as_ref(), self.theme.bitmap_advance)
    }

    fn title_text_width(&self, text: &str) -> i32 {
        crate::text::tier_text_width(text, self.ttf.title.as_ref(), self.theme.bitmap_advance_title)
    }

    fn hint_text_width(&self, text: &str) -> i32 {
        crate::text::tier_text_width(text, self.ttf.hint.as_ref(), self.theme.bitmap_advance_hint)
    }

    fn glyph_height(&self) -> i32 {
        self.theme.font_size
    }

    fn title_glyph_height(&self) -> i32 {
        self.theme.title_font_size
    }

    fn hint_glyph_height(&self) -> i32 {
        self.theme.hint_font_size
    }

    fn clear_frame(&mut self, color: u16) {
        self.ops.push(DrawOp::Clear(color));
    }

    fn blit_image(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32) {
        self.ops.push(DrawOp::Image {
            asset: asset.to_owned(),
            x,
            y,
            w,
            h,
            tint: 0xFFFF,
            rot_mdeg: 0,
        });
    }

    fn blit_image_tint(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32, tint: u16) {
        self.ops.push(DrawOp::Image { asset: asset.to_owned(), x, y, w, h, tint, rot_mdeg: 0 });
    }

    #[allow(clippy::too_many_arguments)]
    fn blit_image_rot(
        &mut self,
        asset: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        tint: u16,
        rot_mdeg: i32,
    ) {
        self.ops.push(DrawOp::Image { asset: asset.to_owned(), x, y, w, h, tint, rot_mdeg });
    }

    fn pixel_at(&self, _x: i32, _y: i32) -> u16 {
        0
    }
}
