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

    /// 贴图绘制（**资源路径** + 目标矩形；后端各自解析资源）。
    ///
    /// 资源路径 = 相对 `assets/` 根的路径（如 `ui/board.png`）：
    /// - CPU 后端（内存帧缓冲）自行读文件 + 解码 + alpha 混合；
    /// - 绘制列表后端（[`crate::drawlist::DrawList`]）只**记录**该命令，由 Bevy 侧走资产服务器；
    /// - 缺资源 / 解码失败 = **不画**（不 panic、不留坑）。
    ///
    /// 默认实现为 no-op（仅需像素绘制的轻量画布可继承）。
    fn blit_image(&mut self, _asset: &str, _x: i32, _y: i32, _w: i32, _h: i32) {}

    /// 贴图 + 色调（**RGB565 multiply**）：`tint = 0xFFFF`（白）= 无色调（同 [`blit_image`]）。
    ///
    /// S21.14 图标图片化：白蒙版贴图在 blit 时乘上色，使同一张 PNG 可切「默认金 / 选中白 /
    /// 危险红」等墨色（跨层 tint，`09_render_framework.md` §5.1）。默认 no-op（轻量画布继承）。
    fn blit_image_tint(&mut self, _asset: &str, _x: i32, _y: i32, _w: i32, _h: i32, _tint: u16) {}

    /// 贴图 + 色调 + **绕自身中心旋转**（S27.5：`main_map` v5 环形卡径向旋转）。
    ///
    /// - `tint` 语义同 [`blit_image_tint`]（`0xFFFF` = 无色调）；
    /// - `rot_mdeg` = 屏幕坐标系（y 向下）**顺时针**旋转，单位**毫度**（`90_000` = 90°）；
    ///   `0` = 不旋转；
    /// - 旋转围绕**目标矩形中心**（`x + w/2, y + h/2`），矩形本身仍按未旋转口径给出。
    ///
    /// 默认实现退化为不旋转的 [`Self::blit_image_tint`]（无旋转能力的后端 = 视觉降级，
    /// 不 panic、不缺图；`ti-engine` 的 Bevy 后端用 `Sprite` 旋转变换实现真旋转）。
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
        let _ = rot_mdeg;
        self.blit_image_tint(asset, x, y, w, h, tint);
    }
}
