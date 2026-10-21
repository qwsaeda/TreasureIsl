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

    /// 全帧提亮/压暗（场景切换淡入，`factor` = 1.0 原值 / 0.0 全黑）。
    ///
    /// `factor` = 1.0 = 原样，0.0 = 全黑；逐像素对 RGB565 三通道做 per-channel 乘法。
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn apply_brightness(&mut self, factor: f32) {
        let f = factor.clamp(0.0, 1.0);
        for p in &mut self.pixels {
            let r = f32::from((*p >> 11) & 0x1F);
            let g = f32::from((*p >> 5) & 0x3F);
            let b = f32::from(*p & 0x1F);
            let rr = (r * f).round().min(31.0) as u16;
            let gg = (g * f).round().min(63.0) as u16;
            let bb = (b * f).round().min(31.0) as u16;
            *p = (rr << 11) | (gg << 5) | bb;
        }
    }

    /// 当前画面快照（定长 `FB_PIXELS` 的 RGB565），供场景切换后渐变用。
    #[must_use]
    pub fn snapshot(&self) -> Box<[u16]> {
        self.pixels.clone()
    }

    /// 从上一帧缓冲 `prev` 渐变到当前内容，`t` = 0.0 = 全上一帧，1.0 = 全当前帧不渐变。
    ///
    /// 用于场景切换淡入（EXP-081）：`enter()` 前先快照，进入后渐变到新画面。
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn crossfade_from(&mut self, prev: &[u16], t: f32) {
        let ratio = (t.clamp(0.0, 1.0) * 255.0).round() as u32;
        for (i, slot) in self.pixels.iter_mut().enumerate() {
            let Some(&old) = prev.get(i) else { break };
            if ratio == 0 {
                *slot = old;
            } else if ratio >= 255 {
                // 全当前帧，不渐变
            } else {
                let cur = *slot;
                let mix = |a: u16, b: u16, shift: u32, mask: u16| -> u16 {
                    let av = u32::from((a >> shift) & mask);
                    let bv = u32::from((b >> shift) & mask);
                    let m = u32::from(mask);
                    (((av * ratio + bv * (255 - ratio) + 127) / 255) & m) as u16
                };
                let r = mix(old, cur, 11, 0x1F);
                let g = mix(old, cur, 5, 0x3F);
                let b = mix(old, cur, 0, 0x1F);
                *slot = (r << 11) | (g << 5) | b;
            }
        }
    }
}

impl Default for Framebuf {
    fn default() -> Self {
        Self::new()
    }
}

/// RGB8 → RGB565（位截断口径：与 `assets/data/theme.json` 的 8bit→565 转换脚本同规约）。
#[must_use]
pub fn rgb8_to_565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = u16::from(r >> 3);
    let g6 = u16::from(g >> 2);
    let b5 = u16::from(b >> 3);
    (r5 << 11) | (g6 << 5) | b5
}

/// RGB565 → RGB8（位扩展口径：与 `rgb8_to_565` 互逆到 5/6/5 精度）。
#[must_use]
pub fn rgb565_to_rgb8(c: u16) -> (u8, u8, u8) {
    let r5 = ((c >> 11) & 0x1F) as u8;
    let g6 = ((c >> 5) & 0x3F) as u8;
    let b5 = (c & 0x1F) as u8;
    ((r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2))
}

/// RGBA8 贴图像素**混合**到 RGB565 目标像素（S18.8b `image` 原语）。
///
/// `a = 0` = 全透明（目标不变）；`a = 255` = 全覆盖；中间值按 `a/255` 线性插值
/// （逐通道，量化到 RGB565 的 5/6/5 精度）。
#[must_use]
#[allow(clippy::many_single_char_names)] // 通道名 r/g/b/a 是像素语义的标准缩写
pub fn blend_rgba_over(dst: u16, r: u8, g: u8, b: u8, a: u8) -> u16 {
    if a == 0 {
        return dst;
    }
    if a == 255 {
        return rgb8_to_565(r, g, b);
    }
    let (dr, dg, db) = rgb565_to_rgb8(dst);
    let mix = |old: u8, new: u8| -> u8 {
        let v = u16::from(old) * (255 - u16::from(a)) + u16::from(new) * u16::from(a);
        u8::try_from((v + 127) / 255).unwrap_or(0)
    };
    rgb8_to_565(mix(dr, r), mix(dg, g), mix(db, b))
}

/// 借用视图（§5.6.6 选用矩阵「缓冲/容器借用视图 → `AsRef/AsMut`」）：
/// 泛型消费方可把 `Framebuf` 当作 `&[u16]` 直接读。
/// 手写 `as_slice`/`as_mut_slice` 保留（既有调用点），本 trait 为纯增量。
impl AsRef<[u16]> for Framebuf {
    fn as_ref(&self) -> &[u16] {
        &self.pixels
    }
}

impl AsMut<[u16]> for Framebuf {
    fn as_mut(&mut self) -> &mut [u16] {
        &mut self.pixels
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

/// RGBA8 帧缓冲（`0xAARRGGBB`，1080×1920）。
///
/// HD 资产路径用（真彩色 + alpha 混合）；与 [`Framebuf`]（RGB565）同构，
/// 差别只在像素宽度与初始色。**不含脏行跟踪** —— 上传带宽优化属平台呈现层
/// （`ti_platform::gfx::FramePresenter`）职责，渲染框架只管像素（S15.10 单源）。
pub struct FramebufRgba {
    pixels: Box<[u32]>,
}

impl FramebufRgba {
    /// 新建全帧缓冲（不透明黑）。
    #[must_use]
    pub fn new() -> Self {
        Self { pixels: vec![0xFF00_0000u32; FB_PIXELS].into_boxed_slice() }
    }

    /// 用指定 RGBA8 值清空全部像素。
    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    /// 只读访问全部像素（行主序，1080×1920）。
    #[must_use]
    pub fn as_slice(&self) -> &[u32] {
        &self.pixels
    }

    /// 可变访问全部像素。
    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut self.pixels
    }
}

impl Default for FramebufRgba {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u32]> for FramebufRgba {
    fn as_ref(&self) -> &[u32] {
        &self.pixels
    }
}

impl AsMut<[u32]> for FramebufRgba {
    fn as_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }
}
