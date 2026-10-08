//! 176×220 逻辑帧缓冲与屏幕坐标变换（P4-2）。
//!
//! 原版 gx.dll 以 176×220 整屏位块绘制；本模块提供：
//! - [`Framebuf`]：固定大小的 RGB565 像素缓冲；
//! - [`Viewport`]：屏幕 ↔ 逻辑坐标的等比缩放（letterbox/pillarbox），
//!   保证任意窗口尺寸下坐标互逆（P4-2 命中判定不偏移）。

use ti_core::{LOGICAL_HEIGHT, LOGICAL_WIDTH};

/// 帧缓冲像素数 = `LOGICAL_WIDTH × LOGICAL_HEIGHT`。
pub const FB_PIXELS: usize = (LOGICAL_WIDTH as usize) * (LOGICAL_HEIGHT as usize);

/// 逻辑宽的 f32 值（= `LOGICAL_WIDTH`，避免 `u32 as f32` 精度告警）。
const LW: f32 = 176.0;
/// 逻辑高的 f32 值（= `LOGICAL_HEIGHT`）。
const LH: f32 = 220.0;

/// 176×220 RGB565 逻辑帧缓冲。
///
/// 原版 gx.dll 的 `GXBeginDraw`/`GXEndDraw` 返回的正是这样的整屏缓冲；
/// 渲染后端（macroquad）负责把它缩放到实际窗口并 present。
pub struct Framebuf {
    pixels: Box<[u16]>,
}

impl Framebuf {
    /// 新建全黑帧缓冲。
    #[must_use]
    pub fn new() -> Self {
        Self { pixels: vec![0u16; FB_PIXELS].into_boxed_slice() }
    }

    /// 用指定 RGB565 值清空全部像素。
    pub fn clear(&mut self, color: u16) {
        self.pixels.fill(color);
    }

    /// 只读访问全部像素（行主序，176×220）。
    #[must_use]
    pub fn as_slice(&self) -> &[u16] {
        &self.pixels
    }

    /// 可变访问全部像素。
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.pixels
    }
}

impl Default for Framebuf {
    fn default() -> Self {
        Self::new()
    }
}

/// RGB565 → `Color`（macroquad 用 f32 RGBA，通道范围 0.0~1.0）。
///
/// 原版帧缓冲是 RGB565，这里做标准位扩展（`r5<<3|r5>>2`），可视作无损。
#[must_use]
pub fn rgb565_to_color(p: u16) -> macroquad::color::Color {
    let r5 = f32::from((p >> 11) & 0x1F);
    let g6 = f32::from((p >> 5) & 0x3F);
    let b5 = f32::from(p & 0x1F);
    macroquad::color::Color::new(
        (r5 * (255.0 / 31.0)) / 255.0,
        (g6 * (255.0 / 63.0)) / 255.0,
        (b5 * (255.0 / 31.0)) / 255.0,
        1.0,
    )
}

/// RGB565 → RGBA8 四元组（供纹理上传）。
#[must_use]
pub fn rgb565_to_rgba8(p: u16) -> [u8; 4] {
    let r5 = ((p >> 11) & 0x1F) as u8;
    let g6 = ((p >> 5) & 0x3F) as u8;
    let b5 = (p & 0x1F) as u8;
    [(r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2), 255]
}

/// 用 [`Framebuf`] 的像素绘制到 macroquad 窗口。
///
/// 每帧一次纹理上传；逻辑分辨率 `LOGICAL_W × LOGICAL_H` 与窗口尺寸解耦，
/// 四个角点经 [`Viewport::logical_to_screen`] 映射，窗口比例差异体现为
/// letterbox / pillarbox。**必须传入视口**：浏览器侧 canvas 会被拉伸到窗口
/// 尺寸，若仍按 176×220 直接绘制，画面只会缩在左上角。
pub fn present_framebuffer(fb: &Framebuf, vp: &Viewport) {
    use macroquad::models::{Vertex, draw_mesh};
    use macroquad::texture::Texture2D;

    let mut bytes: Vec<u8> = Vec::with_capacity(FB_PIXELS * 4);
    for &p in fb.as_slice() {
        bytes.extend_from_slice(&rgb565_to_rgba8(p));
    }
    let white = macroquad::color::WHITE;
    let width_u16 = u16::try_from(LOGICAL_WIDTH).unwrap_or(0);
    let height_u16 = u16::try_from(LOGICAL_HEIGHT).unwrap_or(0);
    let texture = Texture2D::from_rgba8(width_u16, height_u16, &bytes);
    let (x0, y0) = vp.logical_to_screen(0.0, 0.0);
    let (x1, y1) = vp.logical_to_screen(LW, LH);
    let vertices = vec![
        Vertex::new(x0, y0, 0.0, 0.0, 1.0, white),
        Vertex::new(x1, y0, 0.0, 1.0, 1.0, white),
        Vertex::new(x1, y1, 0.0, 1.0, 0.0, white),
        Vertex::new(x0, y1, 0.0, 0.0, 0.0, white),
    ];
    let mesh = macroquad::models::Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(texture),
    };
    draw_mesh(&mesh);
}

/// 屏幕坐标 ↔ 逻辑坐标(176×220)的等比缩放变换。
///
/// 保持 176:220 宽高比，不足部分居中留黑（letterbox / pillarbox）。
/// 逆变换严格互逆：`screen_to_logical(logical_to_screen(p)) == p`。
pub struct Viewport {
    /// 逻辑→屏幕的缩放比。
    scale: f32,
    /// 逻辑原点在屏幕上的 x 偏移（居中留黑）。
    offset_x: f32,
    /// 逻辑原点在屏幕上的 y 偏移。
    offset_y: f32,
}

impl Viewport {
    /// 根据屏幕尺寸计算等比缩放与居中偏移。
    #[must_use]
    pub fn fit(screen_w: f32, screen_h: f32) -> Self {
        let scale = (screen_w / LW).min(screen_h / LH);
        let offset_x = (screen_w - LW * scale) * 0.5;
        let offset_y = (screen_h - LH * scale) * 0.5;
        Self { scale, offset_x, offset_y }
    }

    /// 缩放比。
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// 逻辑坐标 → 屏幕坐标。
    #[must_use]
    pub fn logical_to_screen(&self, lx: f32, ly: f32) -> (f32, f32) {
        (self.offset_x + lx * self.scale, self.offset_y + ly * self.scale)
    }

    /// 屏幕坐标 → 逻辑坐标（命中判定用）。
    #[must_use]
    pub fn screen_to_logical(&self, sx: f32, sy: f32) -> (f32, f32) {
        ((sx - self.offset_x) / self.scale, (sy - self.offset_y) / self.scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ti_core::{LOGICAL_HEIGHT, LOGICAL_WIDTH};

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn coordinate_roundtrip_integer_scale_2x() {
        let vp = Viewport::fit(352.0, 440.0);
        assert!(approx_eq(vp.scale(), 2.0));
        let points = [(0.0, 0.0), (88.0, 110.0), (176.0, 220.0), (1.0, 1.0)];
        for (lx, ly) in points {
            let (sx, sy) = vp.logical_to_screen(lx, ly);
            let (rx, ry) = vp.screen_to_logical(sx, sy);
            assert!(approx_eq(rx, lx), "x: {rx} != {lx}");
            assert!(approx_eq(ry, ly), "y: {ry} != {ly}");
        }
    }

    #[test]
    fn coordinate_roundtrip_non_integer_scale() {
        let vp = Viewport::fit(300.0, 375.0);
        let points = [(0.0, 0.0), (88.0, 110.0), (176.0, 220.0), (50.5, 75.5)];
        for (lx, ly) in points {
            let (sx, sy) = vp.logical_to_screen(lx, ly);
            let (rx, ry) = vp.screen_to_logical(sx, sy);
            assert!(approx_eq(rx, lx), "x: {rx} != {lx}");
            assert!(approx_eq(ry, ly), "y: {ry} != {ly}");
        }
    }

    #[test]
    fn coordinate_roundtrip_wide_screen_letterbox() {
        let vp = Viewport::fit(800.0, 220.0);
        assert!(vp.scale() > 0.0);
        let (sx, sy) = vp.logical_to_screen(88.0, 110.0);
        let (rx, ry) = vp.screen_to_logical(sx, sy);
        assert!(approx_eq(rx, 88.0));
        assert!(approx_eq(ry, 110.0));
    }

    #[test]
    fn coordinate_roundtrip_tall_screen_pillarbox() {
        let vp = Viewport::fit(176.0, 440.0);
        let (sx, sy) = vp.logical_to_screen(88.0, 110.0);
        let (rx, ry) = vp.screen_to_logical(sx, sy);
        assert!(approx_eq(rx, 88.0));
        assert!(approx_eq(ry, 110.0));
    }

    #[test]
    fn coordinate_roundtrip_screen_to_logical_then_back() {
        let vp = Viewport::fit(463.0, 578.0);
        let screen_points = [(0.0, 0.0), (200.0, 300.0), (462.0, 577.0), (100.5, 200.5)];
        for (sx, sy) in screen_points {
            let (lx, ly) = vp.screen_to_logical(sx, sy);
            let (rx, ry) = vp.logical_to_screen(lx, ly);
            assert!(approx_eq(rx, sx), "sx: {rx} != {sx}");
            assert!(approx_eq(ry, sy), "sy: {ry} != {sy}");
        }
    }

    #[test]
    fn framebuf_size_matches_logical_resolution() {
        assert_eq!(FB_PIXELS, (LOGICAL_WIDTH * LOGICAL_HEIGHT) as usize);
        assert_eq!(FB_PIXELS, 176 * 220);
    }

    #[test]
    fn framebuf_clear_sets_all_pixels() {
        let mut fb = Framebuf::new();
        fb.clear(0xF800);
        assert!(fb.as_slice().iter().all(|&p| p == 0xF800));
    }

    #[test]
    fn framebuf_default_is_black() {
        let fb = Framebuf::default();
        assert!(fb.as_slice().iter().all(|&p| p == 0));
    }
}
