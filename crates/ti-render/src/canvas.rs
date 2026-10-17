//! Canvas trait：渲染框架与宿主平台之间的最小抽象接口。
//!
//! 宿主（如 [`crate::shell::SceneShell`] 或游戏自身的渲染器）实现此 trait，
//! 渲染框架的所有绘制原语通过此 trait 操作像素与文字，不直接依赖平台 API。

/// 画布接口：渲染框架操作像素与文字的最小抽象。
///
/// 实现方负责把绘制操作落到实际帧缓冲（如 RGB565 内存缓冲），
/// 渲染框架本身不关心呈现后端细节。
pub trait Canvas {
    /// 填充矩形（RGB565，整数逻辑坐标）。
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16);

    /// 绘制一行文本（正文档字号）。
    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u16);

    /// 绘制一行标题档文本（标题字号）。
    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, color: u16);

    /// 绘制一行提示档文本（提示字号）。
    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, color: u16);

    /// 正文档文本像素宽。
    fn text_width(&self, text: &str) -> i32;

    /// 标题档文本像素宽。
    fn title_text_width(&self, text: &str) -> i32;

    /// 提示档文本像素宽。
    fn hint_text_width(&self, text: &str) -> i32;

    /// 正文档字形垂直高（px）。
    fn glyph_height(&self) -> i32;

    /// 标题档字形垂直高（px）。
    fn title_glyph_height(&self) -> i32;

    /// 提示档字形垂直高（px）。
    fn hint_glyph_height(&self) -> i32;

    /// 清屏到指定颜色。
    fn clear_frame(&mut self, color: u16);

    /// 读取帧缓冲某像素颜色。
    fn pixel_at(&self, x: i32, y: i32) -> u16;
}
