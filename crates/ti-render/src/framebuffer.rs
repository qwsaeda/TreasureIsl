//! 帧缓冲抽象：固定大小 RGB565 像素缓冲。
//!
//! 纯内存渲染目标，不依赖任何平台 API。平台层（macroquad/miniquad）
//! 负责把帧缓冲内容缩放到实际窗口并呈现。

/// UI 呈现画布宽（1080×1920 单一权威呈现坐标系）。
pub const CANVAS_W: u32 = 1080;
/// UI 呈现画布高。
pub const CANVAS_H: u32 = 1920;

/// 帧缓冲像素数 = `CANVAS_W × CANVAS_H`。
pub const FB_PIXELS: usize = (CANVAS_W as usize) * (CANVAS_H as usize);

/// 逻辑宽（i32，供坐标运算）。
pub const LOGICAL_W_I32: i32 = 1080;
/// 逻辑宽（usize，供帧缓冲索引）。
pub const LOGICAL_W_USIZE: usize = 1080;
/// 逻辑高（i32，供坐标运算）。
pub const LOGICAL_H_I32: i32 = 1920;

/// RGB565 呈现层帧缓冲。
///
/// 渲染后端（macroquad/miniquad）负责把它缩放到实际窗口并 present。
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

    /// 只读访问全部像素（行主序，1080×1920）。
    #[must_use]
    pub fn as_slice(&self) -> &[u16] {
        &self.pixels
    }

    /// 可变访问全部像素。
    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.pixels
    }

    /// 填充矩形（RGB565，整数逻辑坐标；越界部分逐像素裁剪）。
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        let x0 = x.clamp(0, LOGICAL_W_I32);
        let y0 = y.clamp(0, LOGICAL_H_I32);
        let x1 = x.saturating_add(w).clamp(0, LOGICAL_W_I32);
        let y1 = y.saturating_add(h).clamp(0, LOGICAL_H_I32);
        for py in y0..y1 {
            for px in x0..x1 {
                let idx = usize::try_from(py).unwrap_or(0) * LOGICAL_W_USIZE
                    + usize::try_from(px).unwrap_or(0);
                if let Some(slot) = self.pixels.get_mut(idx) {
                    *slot = color;
                }
            }
        }
    }
}

impl Default for Framebuf {
    fn default() -> Self {
        Self::new()
    }
}

/// 像素写入目标：渲染框架操作像素的最小 trait。
///
/// 任何帧缓冲（[`Framebuf`]、平台 GPU 缓冲等）实现此 trait，即可作为
/// [`crate::font::bitmap::Raster::blit`] 与 [`crate::font::ttf::TtfFont`] 的输出目标。
/// 渲染框架不依赖具体帧缓冲类型，只依赖此 trait —— 可复用到其他游戏/平台。
pub trait PixelSink {
    /// 写单像素（RGB565，整数逻辑坐标；越界 no-op）。
    fn set_pixel(&mut self, x: i32, y: i32, color: u16);
}

impl PixelSink for Framebuf {
    fn set_pixel(&mut self, x: i32, y: i32, color: u16) {
        if (0..LOGICAL_W_I32).contains(&x) && (0..LOGICAL_H_I32).contains(&y) {
            let idx =
                usize::try_from(y).unwrap_or(0) * LOGICAL_W_USIZE + usize::try_from(x).unwrap_or(0);
            if let Some(slot) = self.pixels.get_mut(idx) {
                *slot = color;
            }
        }
    }
}
