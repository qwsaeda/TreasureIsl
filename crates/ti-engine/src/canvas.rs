//! S18.2：`BevyCanvas` —— `ti_render::canvas::Canvas` 的 Bevy 实现
//! （`docs/process/11_bevy_migration.md` §4 架构形状）。
//!
//! 设计口径：
//! - **op 记录制**：Canvas 方法只记录绘制 op（纯、可测）；`flush` 把 op 落 ECS
//!   （`fill_rect` → 定尺寸色块 Sprite；三档文本 → `Text2d`；`clear_frame` → `ClearColor`）。
//! - **1080 坐标契约**（见 `docs/process/08_render_arch.md` 单一坐标系）：逻辑坐标
//!   `(0,0)` = 左上、`y` 向下；Bevy 2D 世界 `y` 向上 → `世界 = (cx - 540, 960 - cy)`，
//!   `Camera2d` 原点 + `viewport_scale` letterbox 缩放（`min(win_w/1080, win_h/1920)`，
//!   富余边距 = 清屏色条）。
//! - **层序**：Bevy 0.20 的 2D 精灵排序键 = `Transform.translation.z`
//!   （`bevy_sprite_render` 用 `FloatOrd(transform.translation().z)`，0.20 无独立
//!   `ZIndex` 组件）；`canvas.set_z` 决定本帧 op 层，文本层 = `z + 1`
//!   （与「调用顺序即层序」旧口径同义：后调用的原语 z 更大）。
//! - **文本指标同源**：宽度/行高公式与 `ti-app MenuShell`（`tier_text_width_px`）
//!   逐式对齐（TTF `advance` 求和 ceil / 位图回退 `char_count`×advance），
//!   保证 chrome 布局计算在两个后端结果一致；像素级字形对齐在 S18.3（TTF GPU）。
//! - `pixel_at`：Bevy 后端无 CPU 帧缓冲，返回最近一次读回的像素（无读回 = 0）；
//!   读回走 `Screenshot` 读回（`readback_screenshot`），像素对照测试见 `tests/`。

use bevy::asset::{AssetServer, Handle};
use bevy::prelude::*;
use bevy::text::{Font, FontSize, TextColor};

use ti_render::canvas::Canvas;
use ti_render::font::TtfFont;
use ti_render::framebuffer::{LOGICAL_H_I32, LOGICAL_W_I32};
use ti_render::theme::Theme;

/// 画布实体标记（下一帧 `flush` 时整体 despawn，帧间无残留实体）。
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CanvasEntity;

/// 逻辑坐标中心点换算（(x,y,w,h) 左上 → 世界坐标，y 翻转）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
#[inline]
fn logical_center_to_world(x: i32, y: i32, w: i32, h: i32) -> Vec3 {
    let cx = (i32::min(x, LOGICAL_W_I32).saturating_add(w / 2)) as f32;
    let cy = (i32::min(y, LOGICAL_H_I32).saturating_add(h / 2)) as f32;
    Vec3::new(cx - 540.0, 960.0 - cy, 0.0)
}

/// 单点坐标换算（文本左上角 → 世界坐标，y 翻转后由调用方补半高偏移）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
#[inline]
fn logical_point_to_world(x: i32, y: i32) -> Vec2 {
    Vec2::new(x as f32 - 540.0, 960.0 - y as f32)
}

/// RGB565（`u16`）→ Bevy `Color`（5→8 位线性扩展，无抖动）。
#[must_use]
pub fn rgb565_to_color(value: u16) -> Color {
    const MAX5: u32 = 31;
    const MAX6: u32 = 63;
    let v = u32::from(value);
    let r = (v >> 11) & MAX5;
    let g = (v >> 5) & MAX6;
    let b = v & MAX5;
    #[allow(clippy::cast_precision_loss)]
    Color::srgb(
        (r * 255 / MAX5) as f32 / 255.0,
        (g * 255 / MAX6) as f32 / 255.0,
        (b * 255 / MAX5) as f32 / 255.0,
    )
}

/// 视窗逻辑尺寸 → 相机 scale（letterbox：内容恒 1080×1920 完整可见，
/// 富余边距 = 清屏色条）。
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn viewport_scale(window_w: f32, window_h: f32) -> f32 {
    f32::min(window_w / LOGICAL_W_I32 as f32, window_h / LOGICAL_H_I32 as f32)
}

/// 三档文本（与 `MenuShell::tier_*` 同源）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tier {
    /// 正文档（body）。
    Body,
    /// 标题档。
    Title,
    /// 提示档。
    Hint,
}

#[derive(Clone, Debug)]
enum Op {
    /// 清屏（RGB565）。
    Clear(u16),
    /// 定尺寸色块（已按画布裁剪；RGB565）。
    Rect { x: i32, y: i32, w: i32, h: i32, color: u16 },
    /// 文本（三档；RGB565）。
    Text { tier: Tier, text: String, x: i32, y: i32, color: u16 },
}

/// `ti_render::canvas::Canvas` 的 Bevy 后端（op 记录 + `flush` 落 ECS）。
///
/// 零游戏语义：只认识 1080 坐标系、RGB565、主题与三档字体（渲染框架层概念）。
pub struct BevyCanvas {
    ops: Vec<Op>,
    theme: Theme,
    ttf_body: Option<TtfFont>,
    ttf_title: Option<TtfFont>,
    ttf_hint: Option<TtfFont>,
    /// GPU 字体句柄（`AssetServer` 载入的 TTF，S18.3）。
    ///
    /// `None` = 用 Bevy 默认字体（无 CJK 字形 → 豆腐块，仅 spike/调试用）。
    font: Option<Handle<Font>>,
    z: i32,
    last_readback: Option<Readback>,
}

/// 最近一次 GPU 读回（`pixel_at` 的来源；未读回 = 无数据，返回 0）。
#[derive(Debug, Default)]
struct Readback {
    pixels: Vec<u16>,
}

impl BevyCanvas {
    /// 新建（默认主题，位图回退指标）。
    #[must_use]
    pub fn new(theme: Theme) -> Self {
        Self {
            ops: Vec::new(),
            ttf_body: None,
            ttf_title: None,
            ttf_hint: None,
            font: None,
            z: 0,
            last_readback: None,
            theme,
        }
    }

    /// 挂 GPU 字体（`AssetServer` 载入的 TTF 句柄；S18.3 CJK GPU 图集）。
    pub fn set_font(&mut self, font: Handle<Font>) {
        self.font = Some(font);
    }

    /// GPU 字体是否在位。
    #[must_use]
    pub fn has_font(&self) -> bool {
        self.font.is_some()
    }

    /// 从 `AssetServer` 载入项目字体资产（`assets/fonts/lxgw-wenkai-reg.ttf`）。
    ///
    /// 平台路径差异：native = 文件系统绝对路径；wasm = 相对 `assets/` 的 URL。
    /// 返回句柄由调用方在 App 生命周期内持有（`Handle` 引用计数）。
    #[must_use]
    pub fn load_project_font(server: &AssetServer, root: &str) -> Handle<Font> {
        server.load(format!("{root}fonts/lxgw-wenkai-reg.ttf"))
    }

    /// 挂 TTF 三档（与 `MenuShell::set_ttf_fonts` 同参；在位 = 指标/字号切 TTF 口径）。
    pub fn set_ttf_fonts(
        &mut self,
        body: Option<TtfFont>,
        title: Option<TtfFont>,
        hint: Option<TtfFont>,
    ) {
        self.ttf_body = body;
        self.ttf_title = title;
        self.ttf_hint = hint;
    }

    /// 本帧 op 的 z 层（文本层 = z+1）。
    pub fn set_z(&mut self, z: i32) {
        self.z = z;
    }

    /// 本帧 z 层。
    #[must_use]
    pub fn z(&self) -> i32 {
        self.z
    }

    /// 当前帧记录的 op 数（契约测试用）。
    #[must_use]
    pub fn op_count(&self) -> usize {
        self.ops.len()
    }

    /// 清空本帧 op（重复绘制同场景前先清，帧内重入安全）。
    pub fn clear_ops(&mut self) {
        self.ops.clear();
    }

    /// 把最近一次 GPU 读回写入（`readback_screenshot` 产物，1080×1920 行主序 RGB565）。
    pub fn set_readback(&mut self, pixels: Vec<u16>) {
        self.last_readback = Some(Readback { pixels });
    }

    /// 把本帧 op 落 ECS：清屏 → `ClearColor`；色块 → 定尺寸 Sprite；文本 → `Text2d`。
    ///
    /// 旧帧 `CanvasEntity` 实体整体 despawn（帧间零残留）。`world` 需注册过
    /// 相关组件（[`init_canvas_world`] 或完整 App 世界）。返回落盘实体数。
    #[allow(clippy::cast_precision_loss)]
    pub fn flush(&self, world: &mut World) -> usize {
        Self::init_canvas_world(world);
        {
            let mut q = world.query_filtered::<Entity, With<CanvasEntity>>();
            let stale: Vec<Entity> = q.iter(world).collect();
            for e in stale {
                world.despawn(e);
            }
        }
        let mut count = 0usize;
        for op in &self.ops {
            match op {
                Op::Clear(color) => {
                    let cc = ClearColor(rgb565_to_color(*color));
                    world.insert_resource(cc);
                }
                Op::Rect { x, y, w, h, color } => {
                    let mut t =
                        Transform::from_translation(logical_center_to_world(*x, *y, *w, *h));
                    t.translation.z = f32::from(i16::try_from(self.z).unwrap_or(i16::MAX));
                    world.spawn((
                        Sprite::from_color(
                            rgb565_to_color(*color),
                            Vec2::new(*w as f32, *h as f32),
                        ),
                        t,
                        CanvasEntity,
                    ));
                    count += 1;
                }
                Op::Text { tier, text, x, y, color } => {
                    let width = self.tier_width(*tier, text);
                    let glyph_h = self.tier_glyph_px(*tier);
                    let center = logical_point_to_world(*x, *y)
                        + Vec2::new(width as f32 / 2.0, -(glyph_h as f32) / 2.0);
                    let mut t = Transform::from_translation(Vec3::new(center.x, center.y, 0.0));
                    t.translation.z = f32::from(i16::try_from(self.z + 1).unwrap_or(i16::MAX));
                    #[allow(clippy::cast_precision_loss)]
                    let font_size = FontSize::Px(self.tier_font_px(*tier) as f32);
                    let mut text_font = bevy::text::TextFont { font_size, ..Default::default() };
                    if let Some(handle) = &self.font {
                        text_font.font = handle.clone().into();
                    }
                    world.spawn((
                        Text2d::new(text.clone()),
                        TextColor(rgb565_to_color(*color)),
                        text_font,
                        t,
                        CanvasEntity,
                    ));
                    count += 1;
                }
            }
        }
        count
    }

    /// 注册画布组件/资源到裸 World（幂等；完整 App 世界可跳过）。
    pub fn init_canvas_world(world: &mut World) {
        Self::register_all(world);
    }

    fn register_all(world: &mut World) {
        world.register_component::<Sprite>();
        world.register_component::<Transform>();
        world.register_component::<Text2d>();
        world.register_component::<TextColor>();
        world.register_component::<bevy::text::TextFont>();
        world.register_component::<CanvasEntity>();
        if !world.contains_resource::<ClearColor>() {
            world.insert_resource(ClearColor::default());
        }
    }

    /// 取世界里的清屏色（`flush` 后断言用）。
    #[must_use]
    pub fn world_clear_color(world: &World) -> Option<Color> {
        world.get_resource::<ClearColor>().map(|c| c.0)
    }

    // === 三档指标（与 MenuShell 逐式同源；公式变动需同步 ti-app 侧） ===

    fn tier_ttf(&self, tier: Tier) -> Option<&TtfFont> {
        match tier {
            Tier::Body => self.ttf_body.as_ref(),
            Tier::Title => self.ttf_title.as_ref(),
            Tier::Hint => self.ttf_hint.as_ref(),
        }
    }

    fn tier_bitmap_advance(&self, tier: Tier) -> i32 {
        match tier {
            Tier::Body => self.theme.bitmap_advance,
            Tier::Title => self.theme.bitmap_advance_title,
            Tier::Hint => self.theme.bitmap_advance_hint,
        }
    }

    fn tier_font_px(&self, tier: Tier) -> i32 {
        match tier {
            Tier::Body => self.theme.font_size,
            Tier::Title => self.theme.title_font_size,
            Tier::Hint => self.theme.hint_font_size,
        }
    }

    fn tier_width(&self, tier: Tier, text: &str) -> i32 {
        if let Some(t) = self.tier_ttf(tier) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let w = text.chars().map(|ch| t.advance(ch)).sum::<f32>().ceil() as i32;
            return w;
        }
        let count = i32::try_from(text.chars().count()).unwrap_or(i32::MAX);
        count.saturating_mul(self.tier_bitmap_advance(tier))
    }

    fn tier_glyph_px(&self, tier: Tier) -> i32 {
        match self.tier_ttf(tier) {
            Some(_) => self.tier_font_px(tier),
            None => self.tier_bitmap_advance(tier),
        }
    }
}

impl Canvas for BevyCanvas {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        // 与 Framebuf::fill_rect 同裁剪语义：越界部分逐像素裁剪。
        let x0 = x.clamp(0, LOGICAL_W_I32);
        let y0 = y.clamp(0, LOGICAL_H_I32);
        let x1 = x.saturating_add(w).clamp(0, LOGICAL_W_I32);
        let y1 = y.saturating_add(h).clamp(0, LOGICAL_H_I32);
        let iw = x1 - x0;
        let ih = y1 - y0;
        if iw <= 0 || ih <= 0 {
            return;
        }
        self.ops.push(Op::Rect { x: x0, y: y0, w: iw, h: ih, color });
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.ops.push(Op::Text { tier: Tier::Body, text: text.to_owned(), x, y, color });
    }

    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.ops.push(Op::Text { tier: Tier::Title, text: text.to_owned(), x, y, color });
    }

    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.ops.push(Op::Text { tier: Tier::Hint, text: text.to_owned(), x, y, color });
    }

    fn text_width(&self, text: &str) -> i32 {
        self.tier_width(Tier::Body, text)
    }

    fn title_text_width(&self, text: &str) -> i32 {
        self.tier_width(Tier::Title, text)
    }

    fn hint_text_width(&self, text: &str) -> i32 {
        self.tier_width(Tier::Hint, text)
    }

    fn glyph_height(&self) -> i32 {
        self.tier_glyph_px(Tier::Body)
    }

    fn title_glyph_height(&self) -> i32 {
        self.tier_glyph_px(Tier::Title)
    }

    fn hint_glyph_height(&self) -> i32 {
        self.tier_glyph_px(Tier::Hint)
    }

    fn clear_frame(&mut self, color: u16) {
        self.ops.push(Op::Clear(color));
    }

    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        let Some(rb) = &self.last_readback else {
            return 0;
        };
        if x < 0 || y < 0 {
            return 0;
        }
        let xi = usize::try_from(x.clamp(0, LOGICAL_W_I32 - 1)).unwrap_or(0);
        let yi = usize::try_from(y.clamp(0, LOGICAL_H_I32 - 1)).unwrap_or(0);
        #[allow(clippy::cast_precision_loss)]
        let idx = yi * LOGICAL_W_I32 as usize + xi;
        rb.pixels.get(idx).copied().unwrap_or(0)
    }
}

/// `Color`（sRGB 8bit）→ RGB565（`u16`，5/6 位量化；与 `rgb565_to_color` 互逆 ±1 LSB）。
///
/// 像素对照测试（`tests/bevy_vs_cpu_pixel.rs`）用：GPU 读回的 8bit 像素
/// 压回 565 与 CPU 帧缓冲同口径比对。
#[must_use]
pub fn color_to_rgb565(c: Color) -> u16 {
    let s = c.to_srgba();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let red = (s.red * 255.0 + 0.5).round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let green = (s.green * 255.0 + 0.5).round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let blue = (s.blue * 255.0 + 0.5).round() as u32;
    let r5 = (red * 31 / 255).min(31) & 0x1F;
    let g6 = (green * 63 / 255).min(63) & 0x3F;
    let b5 = (blue * 31 / 255).min(31) & 0x1F;
    u16::try_from((r5 << 11) | (g6 << 5) | b5).unwrap_or(0)
}
