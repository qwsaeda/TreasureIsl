//! 1080×1920 呈现层帧缓冲与屏幕坐标变换（P4-2，`08_render_arch.md` 契约 1）。
//!
//! 呈现坐标系唯一 = 1080×1920（legacy 176×220 已退出运行时）；本模块提供：
//! - 帧缓冲本体在渲染框架 crate（`ti_render::framebuffer::{Framebuf, FramebufRgba}`，S15.10 单源）；
//! - [`FramePresenter`]：持续纹理呈现器（S4.3 脏行带部分上传，静止帧零 GPU）；
//! - [`Viewport`]：屏幕 ↔ 逻辑坐标的等比缩放（letterbox/pillarbox），
//!   保证任意窗口尺寸下坐标互逆（P4-2 命中判定不偏移）。

use crate::settings::{FilterMode, ScaleMode};

/// UI 呈现画布宽（1080×1920 单一权威呈现坐标系，`08_render_arch.md` 契约 1）。
pub const CANVAS_W: u32 = 1080;
/// UI 呈现画布高。
pub const CANVAS_H: u32 = 1920;

/// 帧缓冲像素数 = `CANVAS_W × CANVAS_H`（UI 呈现层）。
pub const FB_PIXELS: usize = (CANVAS_W as usize) * (CANVAS_H as usize);

/// 画布宽的 f32 值（= `CANVAS_W`，避免 `u32 as f32` 精度告警）。
const LW: f32 = 1080.0;
/// 画布高的 f32 值（= `CANVAS_H`）。
const LH: f32 = 1920.0;
/// 画布高的 i32 版（供坐标运算）。
const LH_I32: i32 = 1920;

// 帧缓冲类型归属（S15.10 单源）：帧缓冲本体在渲染框架 crate
// （`ti_render::framebuffer::Framebuf` / `FramebufRgba`），本模块只做**呈现层**：
// 脏行跟踪 + 纹理上传 + 屏幕变换。渲染框架不依赖本 crate（可复用到其他游戏）。

/// S7.5（feature `hd-8bit`）：RGBA8 像素（`0xAARRGGBB`）→ RGBA8 四元组（供纹理上传）。
///
/// 与 [`rgb565_to_rgba8`] 同用途；HD 帧缓冲已是 8 位真彩，直接解包即可。
#[cfg(feature = "hd-8bit")]
#[must_use]
pub fn rgba8_to_rgba8_quad(p: u32) -> [u8; 4] {
    [
        u8::try_from((p >> 16) & 0xFF).unwrap_or(0),
        u8::try_from((p >> 8) & 0xFF).unwrap_or(0),
        u8::try_from(p & 0xFF).unwrap_or(0),
        0xFF,
    ]
}

/// S7.5（feature `hd-8bit`）：PNG 解码后的 RGBA 四元组 → 帧缓冲值（`0xAARRGGBB`）。
///
/// 透明像素（A=0）不参与合成（调用方在 blit 时判 A=0 跳过写帧缓冲）；
/// 此处仅做通道重排，不预乘 alpha（原版 blit 为不透明覆盖语义，F-311）。
#[cfg(feature = "hd-8bit")]
#[must_use]
pub fn png_rgba_to_framebuf_u32(rgba: [u8; 4]) -> u32 {
    0xFF00_0000u32 | u32::from(rgba[0]) << 16 | u32::from(rgba[1]) << 8 | u32::from(rgba[2])
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

/// 用 `ti_render::framebuffer::Framebuf` 的像素绘制到 macroquad 窗口（无状态单帧版）。
///
/// 每帧新建纹理并整帧上传；[`FramePresenter`] 提供复用纹理 + 脏行带
/// 部分上传的高性能版本（S4.3）。本函数保留用于不需要持续呈现器的
/// 一次性场景（测试 / 截帧）。
///
/// 逻辑分辨率 `LOGICAL_W × LOGICAL_H` 与窗口尺寸解耦，四个角点经
/// [`Viewport::logical_to_screen`] 映射，窗口比例差异体现为
/// letterbox / pillarbox。**必须传入视口**：浏览器侧 canvas 会被拉伸到窗口
/// 尺寸，若仍按 176×220 直接绘制，画面只会缩在左上角。
pub fn present_framebuffer(fb: &ti_render::framebuffer::Framebuf, vp: &Viewport) {
    use macroquad::texture::Texture2D;

    let mut bytes: Vec<u8> = Vec::with_capacity(FB_PIXELS * 4);
    for &p in fb.as_slice() {
        bytes.extend_from_slice(&rgb565_to_rgba8(p));
    }
    let width_u16 = u16::try_from(CANVAS_W).unwrap_or(0);
    let height_u16 = u16::try_from(CANVAS_H).unwrap_or(0);
    let texture = Texture2D::from_rgba8(width_u16, height_u16, &bytes);
    draw_quad(&texture, vp);
}

/// 持续呈现器（S4.3 纹理批处理）：单张纹理 + 逐帧按脏行带部分上传。
///
/// 与逐帧 `from_rgba8` 相比：
/// - 零分配（RGBA8 缓存与纹理在构造时一次分配）；
/// - 帧无脏行 → 整帧跳过上传与绘制（静止画面零 GPU 工作）；
/// - 帧有脏行 → 只转换脏行带内的像素，`update_part` 一次部分上传，
///   绘制一次全屏 quad（纹理未更新部分保留 GPU 旧值）。
///
/// 典型用法：shell 主循环每帧 `presenter.present(&fb, &vp)`。
///
/// **上传策略**（[`UploadMode`]，构造时选定）：
/// - [`UploadMode::Partial`]（默认，wasm 路径）：脏行带 `update_part`/`update_from_bytes`
///   部分上传，复用首帧创建的纹理。
/// - [`UploadMode::FullRebuild`]（native 桌面路径）：每个脏帧**整帧 `from_rgba8` 重建纹理**。
///   native macroquad 桌面下复用纹理的 `update_*` 不会重新进本帧批次（`from_rgba8`
///   内部 `texture_batcher.add_unbatched` 而 `update_*` 不重标记），部分上传不反映到
///   屏幕（EXP-045）；重建纹理虽放弃"未更新行保留 GPU 旧值"的部分上传优化，但
///   [`FramePresenter::has_dirty`] 的**静止帧整帧 skip 仍成立**（菜单静止时 0 GPU 工作），
///   脏帧 ≤220 行全帧上传代价可忽略。
pub struct FramePresenter {
    /// 纹理槽（`None` = 尚未创建，首次 present 惰性创建）。
    texture: Option<macroquad::texture::Texture2D>,
    /// 全帧 RGBA8 缓存（未脏行保留上次转换值，仅脏行带重写）。
    cache: Vec<u8>,
    /// 上传策略（构造时选定）。
    mode: UploadMode,
    /// 已执行 GPU 上传的帧数（脏帧计数）。
    uploads: u64,
    /// 纹理采样滤波（S6.1：最近邻=像素锐利 / 线性=平滑）。
    filter: FilterMode,
    /// 因无脏行而整帧跳过的帧数。
    skipped: u64,
    /// 逐行脏标记（1920 行；S15.10 从帧缓冲迁入 —— 脏行跟踪是**上传带宽优化**，
    /// 属呈现层职责，帧缓冲本体只管像素，在 `ti_render::framebuffer`）。
    dirty_rows: [bool; LH_I32 as usize],
}

/// 脏帧的纹理上传策略（见 [`FramePresenter`] 文档）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadMode {
    /// 部分上传（复用首帧纹理）；wasm 路径。
    Partial,
    /// 整帧重建（每脏帧 `from_rgba8` 新建纹理）；native 桌面路径（EXP-045）。
    FullRebuild,
}

#[allow(clippy::derivable_impls)] // 有意选 `Partial`（wasm 默认），非 derive 的第一个变体语义
impl Default for UploadMode {
    fn default() -> Self {
        Self::Partial
    }
}

impl FramePresenter {
    /// 新建呈现器（无 GPU 工作，可离屏构造；纹理在首次 present 时惰性创建）。
    /// 默认 [`UploadMode::Partial`]（wasm 路径）。
    #[must_use]
    pub fn new() -> Self {
        Self::with_mode(UploadMode::Partial)
    }

    /// 新建指定上传策略的呈现器（native 桌面用 [`UploadMode::FullRebuild`]，EXP-045）。
    #[must_use]
    pub fn with_mode(mode: UploadMode) -> Self {
        Self {
            texture: None,
            cache: vec![0u8; FB_PIXELS * 4],
            mode,
            uploads: 0,
            skipped: 0,
            filter: FilterMode::default(),
            dirty_rows: [true; LH_I32 as usize],
        }
    }

    /// 标记矩形 `[x, x+w) × [y, y+h)` 所在行脏（越界裁到逻辑屏内；全裁空 = no-op）。
    ///
    /// 调用方在写像素后调用；`mark_dirty` 只记行，不写像素。
    /// S15.10：从帧缓冲迁入（帧缓冲不再持有平台层的脏行状态）。
    pub fn mark_dirty(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let y0 = y.max(0);
        let y1 = (y.saturating_add(h).saturating_sub(1)).min(LH_I32 - 1);
        let w_i32 = i32::try_from(CANVAS_W).unwrap_or(0);
        if y0 > y1 || x.saturating_add(w) <= 0 || x >= w_i32 {
            return;
        }
        for r in y0..=y1 {
            let Ok(rr) = usize::try_from(r) else { break };
            if let Some(slot) = self.dirty_rows.get_mut(rr) {
                *slot = true;
            }
        }
    }

    /// 是否存在脏行（无脏行时 `present` 可整帧跳过，静止帧 0 GPU 工作）。
    #[must_use]
    pub fn has_dirty(&self) -> bool {
        self.dirty_rows.iter().any(|&d| d)
    }

    /// 脏行带 `[first, last]`（无脏行时返回 `(0, 0)`，调用方先查 [`has_dirty`]）。
    #[must_use]
    pub fn dirty_row_range(&self) -> (i32, i32) {
        let mut first: Option<i32> = None;
        let mut last: Option<i32> = None;
        for (i, &d) in self.dirty_rows.iter().enumerate() {
            if d {
                let i = i32::try_from(i).unwrap_or(0);
                first = Some(match first {
                    None => i,
                    Some(f) => f.min(i),
                });
                last = Some(match last {
                    None => i,
                    Some(l) => l.max(i),
                });
            }
        }
        match (first, last) {
            (Some(f), Some(l)) => (f, l),
            _ => (0, 0),
        }
    }

    /// 清零所有脏标记（一次上传完成后调用）。
    pub fn reset_dirty(&mut self) {
        self.dirty_rows.fill(false);
    }

    /// 标记全帧脏（S15.10：整帧像素操作如渐变/调亮之后调用）。
    pub fn mark_all_dirty(&mut self) {
        self.dirty_rows.fill(true);
    }

    /// 上传策略（只读）。
    #[must_use]
    pub fn upload_mode(&self) -> UploadMode {
        self.mode
    }

    /// 呈现一帧：按脏行带转换 + 部分上传，绘制全屏 quad，清脏标记。
    ///
    /// - `Partial`（wasm）：无脏行时直接返回（不调任何 GPU 接口）。
    /// - `FullRebuild`（native）：无脏行时**仍 `draw_quad` 复用上次纹理**（不重建、
    ///   不转像素）——native macroquad 纹理批处理每帧重组，不重绘就露出黑底（EXP-045）；
    ///   仅保留「无脏帧不转像素」的优化，`skipped` 仍计数用于诊断。
    ///
    /// 有脏行时按 mode 转换 + 上传，成功后清本呈现器的脏行标记（[`FramePresenter::reset_dirty`]。
    pub fn present(&mut self, fb: &ti_render::framebuffer::Framebuf, vp: &Viewport) {
        if !self.has_dirty() {
            self.skipped += 1;
            if self.mode == UploadMode::FullRebuild {
                // native：静止帧也重绘 quad（复用上次纹理），否则黑底。
                // 纹理须已存在（首帧脏时已建）；从未有过脏帧则无纹理可画。
                if let Some(tex) = self.texture.as_ref() {
                    draw_quad(tex, vp);
                }
            }
            return;
        }
        let width_u16 = u16::try_from(CANVAS_W).unwrap_or(0);
        let height_u16 = u16::try_from(CANVAS_H).unwrap_or(0);
        let (first, last) = self.dirty_row_range();
        let first = first.clamp(0, LH_I32);
        let last = last.min(LH_I32 - 1).max(first);
        let stride = usize::try_from(CANVAS_W).unwrap_or(1) * 4;
        let px = fb.as_slice();
        for row in first..=last {
            let Ok(r) = usize::try_from(row) else {
                continue;
            };
            let base = r * stride;
            for (i, &p) in px[r * CANVAS_W as usize..(r + 1) * CANVAS_W as usize].iter().enumerate()
            {
                let b = rgb565_to_rgba8(p);
                self.cache[base + i * 4..base + i * 4 + 4].copy_from_slice(&b);
            }
        }
        if self.mode == UploadMode::FullRebuild {
            // native 桌面：每脏帧整帧 from_rgba8 重建纹理（EXP-045，update_* 不重进批次）。
            // 行主序 bytes 前部 = 逻辑顶（y=0），与 draw_quad 的 V=0 在顶映射一致 → 不翻转。
            let texture =
                macroquad::texture::Texture2D::from_rgba8(width_u16, height_u16, &self.cache);
            texture.set_filter(self.filter.into());
            self.texture = Some(texture);
            if let Some(tex) = self.texture.as_ref() {
                draw_quad(tex, vp);
            }
        } else {
            let texture = self.texture.get_or_insert_with(|| {
                macroquad::texture::Texture2D::from_rgba8(width_u16, height_u16, &self.cache)
            });
            texture.set_filter(self.filter.into());
            if first == 0 && last == LH_I32 - 1 {
                texture.update_from_bytes(u32::from(width_u16), u32::from(height_u16), &self.cache);
            } else {
                let band_h = u16::try_from(last - first + 1).unwrap_or(height_u16);
                let band_start = usize::try_from(first).map_or(0, |f| f.saturating_mul(stride));
                let band_end = usize::try_from(last)
                    .map_or(0, |l| (l.saturating_add(1)).saturating_mul(stride));
                let img = macroquad::texture::Image {
                    bytes: self.cache[band_start..band_end].to_vec(),
                    width: width_u16,
                    height: band_h,
                };
                texture.update_part(&img, 0, first, i32::from(width_u16), last - first + 1);
            }
            draw_quad(texture, vp);
        }
        self.reset_dirty();
        self.uploads += 1;
    }

    /// 切换纹理采样滤波（S6.1）。下一次 present 生效。
    pub fn set_filter(&mut self, filter: FilterMode) {
        self.filter = filter;
    }

    /// S7.5（feature `hd-8bit`）：RGBA8 帧缓冲版的呈现（`FramebufRgba` → 纹理上传）。
    ///
    /// 与 [`present`](FramePresenter::present) 同语义（脏行带部分上传 / 整帧重建），
    /// 差异只在像素源：`FramebufRgba` 已是 RGBA8（`0xAARRGGBB`），直接解包写缓存，
    /// 无需 565→888 转换。
    #[cfg(feature = "hd-8bit")]
    pub fn present_rgba(&mut self, fb: &ti_render::framebuffer::FramebufRgba, vp: &Viewport) {
        if !self.has_dirty() {
            self.skipped += 1;
            if self.mode == UploadMode::FullRebuild {
                // native：静止帧也重绘 quad（复用上次纹理），否则黑底（EXP-045）。
                if let Some(tex) = self.texture.as_ref() {
                    draw_quad(tex, vp);
                }
            }
            return;
        }
        let width_u16 = u16::try_from(CANVAS_W).unwrap_or(0);
        let height_u16 = u16::try_from(CANVAS_H).unwrap_or(0);
        let (first, last) = self.dirty_row_range();
        let first = first.clamp(0, LH_I32);
        let last = last.min(LH_I32 - 1).max(first);
        let stride = usize::try_from(CANVAS_W).unwrap_or(1) * 4;
        let px = fb.as_slice();
        for row in first..=last {
            let Ok(r) = usize::try_from(row) else {
                continue;
            };
            let base = r * stride;
            for (i, &p) in px[r * CANVAS_W as usize..(r + 1) * CANVAS_W as usize].iter().enumerate()
            {
                let b = rgba8_to_rgba8_quad(p);
                self.cache[base + i * 4..base + i * 4 + 4].copy_from_slice(&b);
            }
        }
        if self.mode == UploadMode::FullRebuild {
            let texture =
                macroquad::texture::Texture2D::from_rgba8(width_u16, height_u16, &self.cache);
            texture.set_filter(self.filter.into());
            self.texture = Some(texture);
            if let Some(tex) = self.texture.as_ref() {
                draw_quad(tex, vp);
            }
        } else {
            let texture = self.texture.get_or_insert_with(|| {
                macroquad::texture::Texture2D::from_rgba8(width_u16, height_u16, &self.cache)
            });
            texture.set_filter(self.filter.into());
            if first == 0 && last == LH_I32 - 1 {
                texture.update_from_bytes(u32::from(width_u16), u32::from(height_u16), &self.cache);
            } else {
                let band_h = u16::try_from(last - first + 1).unwrap_or(height_u16);
                let band_start = usize::try_from(first).map_or(0, |f| f.saturating_mul(stride));
                let band_end = usize::try_from(last)
                    .map_or(0, |l| (l.saturating_add(1)).saturating_mul(stride));
                let img = macroquad::texture::Image {
                    bytes: self.cache[band_start..band_end].to_vec(),
                    width: width_u16,
                    height: band_h,
                };
                texture.update_part(&img, 0, first, i32::from(width_u16), last - first + 1);
            }
            draw_quad(texture, vp);
        }
        self.reset_dirty();
        self.uploads += 1;
    }

    /// 当前滤波模式。
    #[must_use]
    pub fn filter(&self) -> FilterMode {
        self.filter
    }

    /// 已执行 GPU 上传的帧数（脏帧计数，供 stats / 诊断）。
    #[must_use]
    pub fn uploads(&self) -> u64 {
        self.uploads
    }

    /// 因无脏行整帧跳过的帧数（供 stats / 诊断）。
    #[must_use]
    pub fn skipped(&self) -> u64 {
        self.skipped
    }
}

impl Default for FramePresenter {
    fn default() -> Self {
        Self::new()
    }
}

/// 用给定纹理绘制全屏 quad（`present_framebuffer` 与 `FramePresenter` 共用）。
///
/// UV 映射（EXP-045）：native macroquad 桌面 GL 后端 `from_rgba8` 把行主序 bytes 前部
/// （逻辑顶 y=0）作纹理 V=1（顶），故屏顶配 V=1、屏底配 V=0，逻辑顶在屏顶、逻辑底在屏底。
/// 填充 letterbox / pillarbox 边框区（S6.2）。
///
/// 边框区 = 整屏减去游戏区。做法：先把**整屏**填成边框色，再用游戏区
/// quad 覆盖中间部分 —— 顺序上等价于「边框在底、画面在上」，且不需要
/// 分 4 段（上/下/左/右）分别算矩形，天然处理 Stretch/ZoomCrop 裁剪模式
/// （此时游戏区可能溢出屏幕，整屏填充即为正确的边框表现）。
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn fill_border(x0: f32, y0: f32, x1: f32, y1: f32) {
    let sw = macroquad::window::screen_width();
    let sh = macroquad::window::screen_height();
    // 游戏区完全在屏外（理论上不会发生，ZoomCrop 至少露出一边）→ 不画边框
    if x1 <= 0.0 || y1 <= 0.0 || x0 >= sw || y0 >= sh {
        return;
    }
    // 全屏填边框色；随后 draw_quad 的画面 quad 覆盖其上
    macroquad::shapes::draw_rectangle(0.0, 0.0, sw, sh, border_color());
}

/// 当前边框色（RGB565 → macroquad Color）。
///
/// 由 [`set_border_style`] 设置；默认深蓝（与游戏主背景一致）。
fn border_color() -> macroquad::color::Color {
    let rgb = BORDER_STYLE.with(std::cell::Cell::get).rgb565();
    let [r, g, b, _a] = rgb565_to_rgba8(rgb);
    // u8 0..=255 -> f32 0.0..=1.0
    macroquad::color::Color::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        1.0,
    )
}

thread_local! {
    /// 当前边框样式（BorderStyle，线程局部；GUI 主循环单线程）。
    static BORDER_STYLE: std::cell::Cell<crate::settings::BorderStyle> =
        const { std::cell::Cell::new(crate::settings::BorderStyle::DeepBlue) };
}

/// 设置边框装饰样式（S6.2，下次 present 生效）。
pub fn set_border_style(style: crate::settings::BorderStyle) {
    BORDER_STYLE.with(|s| s.set(style));
}

/// 当前边框装饰样式。
#[must_use]
pub fn border_style() -> crate::settings::BorderStyle {
    BORDER_STYLE.with(std::cell::Cell::get)
}

fn draw_quad(texture: &macroquad::texture::Texture2D, vp: &Viewport) {
    // 行主序 bytes 前部 = 逻辑顶（y=0）；`flip_y` 处理 native 与 wasm 纹理 V 语义
    // 相反（EXP-045/047）。`const` 须置于函数首，避免 clippy items_after_statements。
    const IS_WASM: bool = cfg!(target_arch = "wasm32");
    let (x0, y0) = vp.logical_to_screen(0.0, 0.0);
    let (x1, y1) = vp.logical_to_screen(LW, LH);
    // S6.2：先把 letterbox / pillarbox 边框区填成主题色，再画画面。
    // 边框区 = 游戏区外扩 1px 的整屏矩形（避免与画面边缘产生接缝）。
    fill_border(x0, y0, x1, y1);
    // 用 `draw_texture_ex`（宏观 API，自带正确纹理 V 方向 + 批处理）替代手工
    // `Mesh`+`draw_mesh`：手工 mesh 的弱引用纹理在 native 桌面 GL 下不进批次、
    // 不渲染，整屏只剩深蓝边框 → 看起来黑屏（EXP-047）。
    let params = macroquad::texture::DrawTextureParams {
        dest_size: Some(macroquad::prelude::vec2(x1 - x0, y1 - y0)),
        flip_y: IS_WASM,
        ..Default::default()
    };
    macroquad::texture::draw_texture_ex(texture, x0, y0, macroquad::color::WHITE, params);
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
    /// 横向缩放比（= `scale`，仅 `Stretch` 模式与 `scale` 不同）。
    scale_x: f32,
    /// 纵向缩放比（= `scale`，仅 `Stretch` 模式与 `scale` 不同）。
    scale_y: f32,
}

impl Viewport {
    /// 根据屏幕尺寸计算等比缩放与居中偏移。
    #[must_use]
    pub fn fit(screen_w: f32, screen_h: f32) -> Self {
        let scale = (screen_w / LW).min(screen_h / LH);
        let offset_x = (screen_w - LW * scale) * 0.5;
        let offset_y = (screen_h - LH * scale) * 0.5;
        Self { scale, offset_x, offset_y, scale_x: scale, scale_y: scale }
    }

    /// 缩放比（等比模式下有效；`Stretch` 下为横向值）。
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// 横向缩放比。
    #[must_use]
    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }

    /// 纵向缩放比。
    #[must_use]
    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }

    /// 非等比拉伸铺满屏幕（`Stretch` 模式）：横纵各自缩放填满，居中定位。
    ///
    /// 屏幕利用率 100%，代价是像素非方形（画面轻微变形）。
    #[must_use]
    pub fn stretch(screen_w: f32, screen_h: f32) -> Self {
        let sx = screen_w / LW;
        let sy = screen_h / LH;
        Self { scale: sx, offset_x: 0.0, offset_y: 0.0, scale_x: sx, scale_y: sy }
    }

    /// 等比放大铺满并居中裁剪（`ZoomCrop` 模式）：按较大的等比比例铺满，
    /// 溢出的部分从对称两侧裁掉。
    ///
    /// 屏幕利用率 100% 且像素保持方形，代价是裁掉部分内容
    /// （176×220 的 4:5 在 1080×2400 上会裁掉约 44% 宽度）。
    #[must_use]
    pub fn zoom_crop(screen_w: f32, screen_h: f32) -> Self {
        let scale = (screen_w / LW).max(screen_h / LH);
        let offset_x = (screen_w - LW * scale) * 0.5;
        let offset_y = (screen_h - LH * scale) * 0.5;
        Self { scale, offset_x, offset_y, scale_x: scale, scale_y: scale }
    }

    /// 按模式构造视口（设置层入口）。
    ///
    /// - [`ScaleMode::Smooth`](crate::settings::ScaleMode::Smooth) / `Integer`
    ///   → 等比铺满（不裁剪），黑边由调用方装饰；
    /// - `Stretch` → 非等比铺满；
    /// - `ZoomCrop` → 等比铺满并居中裁剪。
    ///
    /// `integer` 为真时把等比比例向下取整到 ≥1×（像素锐利）。
    #[must_use]
    pub fn with_mode(screen_w: f32, screen_h: f32, mode: ScaleMode, integer: bool) -> Self {
        match mode {
            ScaleMode::Stretch => Self::stretch(screen_w, screen_h),
            ScaleMode::ZoomCrop => Self::zoom_crop(screen_w, screen_h),
            ScaleMode::Smooth | ScaleMode::Integer => {
                let raw = (screen_w / LW).min(screen_h / LH);
                if integer {
                    // 向下取整后重新居中，保证整数倍不产生半像素偏移
                    let scale = raw.floor().max(1.0);
                    let offset_x = (screen_w - LW * scale) * 0.5;
                    let offset_y = (screen_h - LH * scale) * 0.5;
                    Self { scale, offset_x, offset_y, scale_x: scale, scale_y: scale }
                } else {
                    Self::fit(screen_w, screen_h)
                }
            }
        }
    }

    /// 用外部计算好的缩放比和偏移直接构造视口（设置层使用，P4-2 整数/横竖屏）。
    #[must_use]
    pub fn from_scale(scale: f32, offset_x: f32, offset_y: f32, _lw: f32, _lh: f32) -> Self {
        Self { scale, offset_x, offset_y, scale_x: scale, scale_y: scale }
    }

    /// 逻辑坐标 → 屏幕坐标。
    ///
    /// 横纵分别用 `scale_x` / `scale_y`（仅 `Stretch` 模式两者不同）。
    #[must_use]
    pub fn logical_to_screen(&self, lx: f32, ly: f32) -> (f32, f32) {
        (self.offset_x + lx * self.scale_x, self.offset_y + ly * self.scale_y)
    }

    /// 屏幕坐标 → 逻辑坐标（命中判定用）。
    #[must_use]
    pub fn screen_to_logical(&self, sx: f32, sy: f32) -> (f32, f32) {
        ((sx - self.offset_x) / self.scale_x, (sy - self.offset_y) / self.scale_y)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::fit(LW, LH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::BorderStyle;

    use crate::settings::ScaleMode;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn coordinate_roundtrip_integer_scale_2x() {
        // 呈现坐标系 1080×1920（契约 1）：2× 整数缩放 = 屏幕 2160×3840
        let vp = Viewport::fit(2160.0, 3840.0);
        assert!(approx_eq(vp.scale(), 2.0));
        let points = [(0.0, 0.0), (540.0, 960.0), (1080.0, 1920.0), (1.0, 1.0)];
        for (lx, ly) in points {
            let (sx, sy) = vp.logical_to_screen(lx, ly);
            let (rx, ry) = vp.screen_to_logical(sx, sy);
            assert!(approx_eq(rx, lx), "x: {rx} != {lx}");
            assert!(approx_eq(ry, ly), "y: {ry} != {ly}");
        }
    }

    #[test]
    fn coordinate_roundtrip_non_integer_scale() {
        let vp = Viewport::fit(1800.0, 3200.0);
        let points = [(0.0, 0.0), (540.0, 960.0), (1080.0, 1920.0), (505.0, 755.0)];
        for (lx, ly) in points {
            let (sx, sy) = vp.logical_to_screen(lx, ly);
            let (rx, ry) = vp.screen_to_logical(sx, sy);
            assert!(approx_eq(rx, lx), "x: {rx} != {lx}");
            assert!(approx_eq(ry, ly), "y: {ry} != {ly}");
        }
    }

    #[test]
    fn coordinate_roundtrip_wide_screen_letterbox() {
        let vp = Viewport::fit(2400.0, 1920.0);
        assert!(vp.scale() > 0.0);
        let (sx, sy) = vp.logical_to_screen(540.0, 960.0);
        let (rx, ry) = vp.screen_to_logical(sx, sy);
        assert!(approx_eq(rx, 540.0));
        assert!(approx_eq(ry, 960.0));
    }

    #[test]
    fn coordinate_roundtrip_tall_screen_pillarbox() {
        let vp = Viewport::fit(1080.0, 2400.0);
        let (sx, sy) = vp.logical_to_screen(540.0, 960.0);
        let (rx, ry) = vp.screen_to_logical(sx, sy);
        assert!(approx_eq(rx, 540.0));
        assert!(approx_eq(ry, 960.0));
    }

    #[test]
    fn coordinate_roundtrip_screen_to_logical_then_back() {
        let vp = Viewport::fit(1440.0, 2560.0);
        let screen_points = [(0.0, 0.0), (600.0, 1000.0), (1439.0, 2559.0), (605.5, 1005.5)];
        for (sx, sy) in screen_points {
            let (lx, ly) = vp.screen_to_logical(sx, sy);
            let (rx, ry) = vp.logical_to_screen(lx, ly);
            assert!(approx_eq(rx, sx), "sx: {rx} != {sx}");
            assert!(approx_eq(ry, sy), "sy: {ry} != {sy}");
        }
    }

    #[test]
    fn framebuf_size_matches_logical_resolution() {
        assert_eq!(FB_PIXELS, (CANVAS_W * CANVAS_H) as usize);
        assert_eq!(FB_PIXELS, 1080 * 1920);
    }

    #[test]
    fn framebuf_clear_sets_all_pixels() {
        let mut fb = ti_render::framebuffer::Framebuf::new();
        fb.clear(0xF800);
        assert!(fb.as_slice().iter().all(|&p| p == 0xF800));
    }

    #[test]
    fn framebuf_default_is_black() {
        let fb = ti_render::framebuffer::Framebuf::default();
        assert!(fb.as_slice().iter().all(|&p| p == 0));
    }

    // ── 缩放模式（docs/modernization.md §3：屏幕利用率）────────────────────────

    /// 屏幕利用率 = 游戏区面积 / 屏幕面积。
    fn screen_fill(vp: &Viewport, sw: f32, sh: f32) -> f32 {
        let (lw, lh) = (LW, LH);
        let w = lw * vp.scale_x();
        let h = lh * vp.scale_y();
        (w * h) / (sw * sh)
    }

    #[test]
    fn smooth_leaves_letterbox_but_keeps_square_pixels() {
        let vp = Viewport::with_mode(1080.0, 2400.0, ScaleMode::Smooth, false);
        assert!((vp.scale_x() - vp.scale_y()).abs() < 1e-4, "等比模式横纵缩放应相等");
        assert!(vp.offset_y > 1.0, "竖屏应留上下黑边");
        // 1080×1920 呈现区放进 1080×2400 屏：等比 1× → 利用率 = 1920/2400 = 80%
        let fill = screen_fill(&vp, 1080.0, 2400.0);
        assert!(fill < 0.85 && fill > 0.75, "Smooth 利用率约 80%，实际 {fill}");
    }

    #[test]
    fn integer_scale_is_whole_number_and_centered() {
        let vp = Viewport::with_mode(1080.0, 2400.0, ScaleMode::Integer, true);
        let s = vp.scale();
        assert!((s - s.floor()).abs() < 1e-4, "Integer 缩放应为整数，实际 {s}");
        assert!(s >= 1.0, "缩放不小于 1×");
        // 居中：左右留边相等
        assert!((vp.offset_x - (1080.0 - LW * s) * 0.5).abs() < 1e-3);
        assert!(vp.scale() <= 1080.0 / LW, "向下取整后不放大");
    }

    #[test]
    fn stretch_fills_screen_entirely() {
        let vp = Viewport::with_mode(1080.0, 2400.0, ScaleMode::Stretch, false);
        assert!((vp.offset_x - 0.0).abs() < 1e-4, "拉伸应无横向偏移");
        assert!((vp.offset_y - 0.0).abs() < 1e-4, "拉伸应无纵向偏移");
        let fill = screen_fill(&vp, 1080.0, 2400.0);
        assert!((fill - 1.0).abs() < 1e-4, "Stretch 利用率应为 100%，实际 {fill}");
        assert!(vp.scale_x() < vp.scale_y(), "4:5 内容拉伸到 20:9 屏 → 横向比例更小");
    }

    #[test]
    fn zoom_crop_fills_screen_keeping_square_pixels() {
        let vp = Viewport::with_mode(1080.0, 2400.0, ScaleMode::ZoomCrop, false);
        assert!((vp.scale_x() - vp.scale_y()).abs() < 1e-4, "裁剪模式像素仍方形");
        let fill = screen_fill(&vp, 1080.0, 2400.0);
        assert!(fill > 1.0, "裁剪模式下游戏区应超出屏幕（溢出部分被裁），实际 {fill}");
        assert!(vp.offset_x < 0.0, "横向溢出 → 偏移为负（左右各裁一半）");
    }

    #[test]
    fn all_modes_roundtrip_coordinates() {
        // P4-2 不变式：任何模式下 屏幕→逻辑→屏幕 都必须回到原点
        for mode in ScaleMode::ALL {
            let vp = Viewport::with_mode(1080.0, 2400.0, mode, mode == ScaleMode::Integer);
            for (lx, ly) in [(0.0_f32, 0.0_f32), (88.0, 110.0), (176.0, 220.0)] {
                let (sx, sy) = vp.logical_to_screen(lx, ly);
                let (rx, ry) = vp.screen_to_logical(sx, sy);
                assert!(
                    (rx - lx).abs() < 1e-2 && (ry - ly).abs() < 1e-2,
                    "{mode:?} ({lx},{ly}) 往返失真: ->({sx},{sy})->({rx},{ry})"
                );
            }
        }
    }

    #[test]
    fn stretch_touch_hit_testing_maps_back_to_logical() {
        // 拉伸模式下命中判定必须仍落到点击的逻辑位置（触屏正确性）
        let vp = Viewport::with_mode(1080.0, 2400.0, ScaleMode::Stretch, false);
        let (sx, sy) = vp.logical_to_screen(44.0, 110.0); // 主菜单行 x 中点
        let (lx, ly) = vp.screen_to_logical(sx, sy);
        assert!((lx - 44.0).abs() < 1e-2 && (ly - 110.0).abs() < 1e-2, "拉伸下触摸反变换失真");
    }

    /// S6.1：滤波模式不参与坐标变换——两种模式下 P4-2 往返不变式都成立。
    #[test]
    fn filter_mode_does_not_affect_coordinates() {
        let vp = Viewport::with_mode(1080.0, 2400.0, ScaleMode::Smooth, false);
        for f in FilterMode::ALL {
            // 滤波只影响纹理采样器，不进入坐标变换；此处显式断言往返仍成立
            for (lx, ly) in [(0.0_f32, 0.0_f32), (88.0, 110.0), (176.0, 220.0)] {
                let (sx, sy) = vp.logical_to_screen(lx, ly);
                let (rx, ry) = vp.screen_to_logical(sx, sy);
                assert!(
                    (rx - lx).abs() < 1e-2 && (ry - ly).abs() < 1e-2,
                    "{f:?} 滤波不应影响坐标往返"
                );
            }
        }
    }

    #[test]
    fn filter_mode_properties_are_consistent() {
        for f in FilterMode::ALL {
            assert!(!f.label().is_empty(), "{f:?} 缺标签");
        }
        assert_eq!(FilterMode::Nearest.next(), FilterMode::Linear);
        assert_eq!(FilterMode::Linear.next(), FilterMode::Nearest, "滤波应循环");
        assert_eq!(FilterMode::default(), FilterMode::Nearest, "默认像素锐利");
    }

    #[test]
    fn presenter_filter_setter_roundtrips() {
        let mut p = FramePresenter::new();
        assert_eq!(p.filter(), FilterMode::default());
        p.set_filter(FilterMode::Linear);
        assert_eq!(p.filter(), FilterMode::Linear);
        p.set_filter(FilterMode::Nearest);
        assert_eq!(p.filter(), FilterMode::Nearest);
    }

    /// S6.2：边框样式枚举行为（标签/循环/色值/resolve）。
    #[test]
    fn border_style_properties_are_consistent() {
        for s in BorderStyle::ALL {
            assert!(!s.label().is_empty(), "{s:?} 缺标签");
        }
        assert_eq!(BorderStyle::Black.next(), BorderStyle::DeepBlue);
        assert_eq!(BorderStyle::DeepBlue.next(), BorderStyle::MatchBackdrop);
        assert_eq!(BorderStyle::MatchBackdrop.next(), BorderStyle::Black, "边框应循环");
        assert_eq!(BorderStyle::default(), BorderStyle::DeepBlue);
        // Black 与 MatchBackdrop 的静态色都是纯黑（后者运行期按背景替换）
        assert_eq!(BorderStyle::Black.rgb565(), 0x0000);
        assert_eq!(BorderStyle::MatchBackdrop.rgb565(), 0x0000);
        assert_ne!(BorderStyle::DeepBlue.rgb565(), 0x0000, "深蓝应区别于纯黑");
    }

    /// `MatchBackdrop` 取帧缓冲首像素，其余用静态色。
    #[test]
    fn border_resolve_uses_backdrop_pixel() {
        assert_eq!(BorderStyle::MatchBackdrop.resolve(0x001F), 0x001F);
        assert_eq!(BorderStyle::Black.resolve(0x001F), 0x0000, "Black 不跟随背景");
        assert_eq!(BorderStyle::DeepBlue.resolve(0x001F), 0x0010, "DeepBlue 用固定色");
    }

    /// 边框填充不得污染游戏区：边框矩形（整屏）先画，游戏区 quad 后画并覆盖。
    /// 本测试验证 `fill_border` 的前置判定——游戏区完全出屏时不画边框。
    #[test]
    fn border_skipped_when_game_area_offscreen() {
        // 游戏区完全在屏幕左侧之外 → offset 使 x1 <= 0（呈现区宽 LW=1080，offset 取 -2000）
        let vp = Viewport::from_scale(1.0, -2000.0, 0.0, LW, LH);
        let (_x0, _y0) = vp.logical_to_screen(0.0, 0.0);
        let (x1, _y1) = vp.logical_to_screen(LW, LH);
        assert!(x1 <= 0.0, "构造的游戏区应在屏外，实际 x1={x1}");
        // fill_border 在此情形直接 return（不画），游戏区像素不受影响
        assert_eq!(BorderStyle::DeepBlue.resolve(0x1234), 0x0010, "边框色解析正常");
    }

    /// 边框填充后游戏区仍能正确反变换（P4-2 不变式不受边框影响）。
    #[test]
    fn border_does_not_affect_viewport_math() {
        for s in BorderStyle::ALL {
            let vp = Viewport::with_mode(1080.0, 2400.0, ScaleMode::Smooth, false);
            let (sx, sy) = vp.logical_to_screen(88.0, 110.0);
            let (lx, ly) = vp.screen_to_logical(sx, sy);
            assert!(
                (lx - 88.0).abs() < 1e-2 && (ly - 110.0).abs() < 1e-2,
                "{s:?} 边框不应影响坐标往返"
            );
        }
    }

    #[test]
    fn scale_mode_properties_are_consistent() {
        for mode in ScaleMode::ALL {
            assert!(!mode.label().is_empty(), "{mode:?} 缺标签");
            assert_eq!(
                mode.fills_screen(),
                matches!(mode, ScaleMode::Stretch | ScaleMode::ZoomCrop)
            );
            assert_eq!(mode.keeps_square_pixels(), !matches!(mode, ScaleMode::Stretch));
        }
    }
}
