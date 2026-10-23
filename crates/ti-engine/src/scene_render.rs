#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::many_single_char_names
)]
//! S18.7：**Bevy 原生场景渲染器** —— 通用原语 → Bevy 实体（Sprite / Text2d / 烘焙 Image）。
//!
//! **通用原语集**（渲染器只认这 5 种，**不认识"菜单""木框"等场景专属概念**）：
//! | kind | 落地 | 参数 |
//! |---|---|---|
//! | `rect` | `Sprite::from_color` | `rect` + `color`（theme 令牌名） |
//! | `text` | `Text2d` + `TextFont`（GPU TTF） | `rect`（居中）+ `tier` + `text`/`text_key` |
//! | `icon` | 烘焙 48×48 纹理 → `Sprite` | `params.icon`（图标 id）+ `rect`（左上 + 边长） |
//! | `grad` | N 条 `Sprite` 竖向渐变 | `params.from`/`to`/`steps`（令牌名 + 级数） |
//! | `glow` | 烘焙径向光晕纹理 → `Sprite` | `rect`（圆心 + 半径） |
//!
//! **界面 = 数据**（`scenes.json` 的 `primitives` 数组）：加/改/删任何元素 = 改 JSON，
//! 渲染器**零改动**（`docs/process/12_bevy_usage.md` §5 + §3 数据驱动边界）。
//!
//! **z 层**（Bevy 0.20 实测，EXP-103）：相机朝 -Z、默认 `near = 0` → 可见实体须 `z < 0`；
//! 绘制按 z 升序 → **z 越大越靠上**；故背景最负、前景接近 0。
//!
//! 零游戏语义：只认 1080 坐标系、theme 令牌与原语表（`ti_core::data::scenes`）。

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::text::{Font, FontSize, TextColor};

use crate::canvas::rgb565_to_color;
use ti_render::theme::Theme;

/// 逻辑画布尺寸（1080×1920 单一坐标系，`08_render_arch.md` 契约）。
const LOGICAL_W: f32 = 1080.0;
/// 逻辑画布高度。
const LOGICAL_H: f32 = 1920.0;

/// 逻辑 → Bevy 世界换算：`world = (cx - 540, 960 - cy)`（y 翻转，S18.2 契约）。
#[inline]
fn to_world(x: f32, y: f32) -> Vec2 {
    Vec2::new(x - LOGICAL_W / 2.0, LOGICAL_H / 2.0 - y)
}

/// 烘焙位图（RGBA8）—— 供 `Sprite` 采样（图标 / 光晕等像素级形状）。
struct Raster {
    width: usize,
    height: usize,
    px: Vec<u8>,
}

impl Raster {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height, px: vec![0u8; width * height * 4] }
    }

    #[allow(
        clippy::many_single_char_names,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        let (r, g, b) = rgb565_to_rgb8(color);
        let max_h = i32::try_from(self.height).unwrap_or(i32::MAX);
        let max_w = i32::try_from(self.width).unwrap_or(i32::MAX);
        for py in y.max(0)..(y + h).min(max_h) {
            for px in x.max(0)..(x + w).min(max_w) {
                let idx = (usize::try_from(py).unwrap_or(0) * self.width
                    + usize::try_from(px).unwrap_or(0))
                    * 4;
                self.px[idx] = r;
                self.px[idx + 1] = g;
                self.px[idx + 2] = b;
                self.px[idx + 3] = 255;
            }
        }
    }

    /// 实心圆盘（边缘硬，月盘用）。
    #[allow(clippy::cast_precision_loss)]
    fn disc(&mut self, cx: f32, cy: f32, radius: f32, color: u16) {
        let (r0, g0, b0) = rgb565_to_rgb8(color);
        for py in 0..self.height {
            for px in 0..self.width {
                let dx = px as f32 + 0.5 - cx;
                let dy = py as f32 + 0.5 - cy;
                if (dx * dx + dy * dy).sqrt() > radius {
                    continue;
                }
                let idx = (py * self.width + px) * 4;
                self.px[idx] = r0;
                self.px[idx + 1] = g0;
                self.px[idx + 2] = b0;
                self.px[idx + 3] = 255;
            }
        }
    }

    /// 径向渐变光晕（中心亮 → 边缘透明）。
    #[allow(clippy::cast_precision_loss)]
    fn radial_glow(&mut self, cx: f32, cy: f32, radius: f32, color: u16) {
        let (r0, g0, b0) = rgb565_to_rgb8(color);
        for py in 0..self.height {
            for px in 0..self.width {
                let dx = px as f32 + 0.5 - cx;
                let dy = py as f32 + 0.5 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > radius {
                    continue;
                }
                let t = 1.0 - dist / radius;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let a = (t * t * 255.0) as u8;
                let idx = (py * self.width + px) * 4;
                self.px[idx] = r0;
                self.px[idx + 1] = g0;
                self.px[idx + 2] = b0;
                self.px[idx + 3] = a;
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn to_image(&self) -> Image {
        Image::new(
            Extent3d {
                width: self.width as u32,
                height: self.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.px.clone(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}

/// 图标烘焙画布（把 `ti_render::icons` 的 rect 几何落到 RGBA 位图）。
struct IconCanvas<'a> {
    raster: &'a mut Raster,
}

impl ti_render::Canvas for IconCanvas<'_> {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.raster.fill(x, y, w, h, color);
    }
    fn draw_text(&mut self, _t: &str, _x: i32, _y: i32, _c: u16) {}
    fn draw_title_text(&mut self, _t: &str, _x: i32, _y: i32, _c: u16) {}
    fn draw_hint_text(&mut self, _t: &str, _x: i32, _y: i32, _c: u16) {}
    fn text_width(&self, _t: &str) -> i32 {
        0
    }
    fn title_text_width(&self, _t: &str) -> i32 {
        0
    }
    fn hint_text_width(&self, _t: &str) -> i32 {
        0
    }
    fn glyph_height(&self) -> i32 {
        0
    }
    fn title_glyph_height(&self) -> i32 {
        0
    }
    fn hint_glyph_height(&self) -> i32 {
        0
    }
    fn clear_frame(&mut self, _color: u16) {}
    fn pixel_at(&self, _x: i32, _y: i32) -> u16 {
        0
    }
}

/// ti-render 图标几何 → 烘焙位图（一次性烘焙 → GPU 采样）。
fn bake_icon(kind: u32, size: i32, color: u16) -> Image {
    #[allow(clippy::cast_sign_loss)]
    let icon_px = size.max(1) as usize;
    let mut raster = Raster::new(icon_px, icon_px);
    let mut canvas = IconCanvas { raster: &mut raster };
    ti_render::icons::icon(&mut canvas, &Theme::default(), kind, 0, 0, color);
    raster.to_image()
}

/// RGB565 → RGB8（5/6/5 位展 8 位）。
#[allow(clippy::cast_possible_truncation)]
fn rgb565_to_rgb8(c: u16) -> (u8, u8, u8) {
    let r5 = (c >> 11) & 0x1F;
    let g6 = (c >> 5) & 0x3F;
    let b5 = c & 0x1F;
    (
        (u32::from(r5) << 3 | u32::from(r5) >> 2) as u8,
        (u32::from(g6) << 2 | u32::from(g6) >> 4) as u8,
        (u32::from(b5) << 3 | u32::from(b5) >> 2) as u8,
    )
}

/// 场景实体句柄（选择态更新用）。
#[derive(Debug, Default)]
pub struct SceneEntities {
    /// 菜单卡 / 图标 / 文字（按原语 id 索引，供选择态换色）。
    pub selectable: Vec<(u64, Entity, SelectableKind)>,
}

/// 可选择元素种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectableKind {
    /// 菜单卡底。
    Card,
    /// 菜单图标。
    Icon,
    /// 菜单文字。
    Label,
}

/// 原语 → 实体（通用执行器）。
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::many_single_char_names
)]
pub fn spawn_primitives(
    world: &mut World,
    theme: &Theme,
    font: Option<Handle<Font>>,
    selected: i64,
    prims: &[ti_core::data::Primitive],
) -> SceneEntities {
    let mut out = SceneEntities::default();
    for (idx, p) in prims.iter().enumerate() {
        // `when` 过滤：仅"选中"或仅"常态"的元素按当前选择显隐。
        let visible = match p.when.as_str() {
            "selected" => p.item == selected,
            "normal" => p.item != selected,
            _ => true,
        };
        if !visible {
            if std::env::var("TI_PRIM_DEBUG").is_ok() {
                eprintln!("[prim] skip kind={} when={:?} item={}", p.kind, p.when, p.item);
            }
            continue;
        }
        if std::env::var("TI_PRIM_DEBUG").is_ok() {
            eprintln!("[prim] draw kind={} when={:?} item={} z={}", p.kind, p.when, p.item, p.z);
        }
        let color = theme.by_name(&p.color).unwrap_or(theme.text_on_parch);
        let z = f32::from(i16::try_from(p.z).unwrap_or(i16::MIN));
        let rect = p.rect;
        match p.kind.as_str() {
            "rect" => {
                let e = world.spawn((
                    Sprite::from_color(
                        rgb565_to_color(color),
                        Vec2::new(rect.w as f32, rect.h as f32),
                    ),
                    Transform::from_translation(
                        to_world(
                            rect.x as f32 + rect.w as f32 / 2.0,
                            rect.y as f32 + rect.h as f32 / 2.0,
                        )
                        .extend(z),
                    ),
                ));
                if p.item >= 0 {
                    out.selectable.push((idx as u64, e.id(), SelectableKind::Card));
                }
            }
            "grad" => {
                // 竖向渐变 = N 条 Sprite（起止色令牌 + 级数，全数据）。
                // 起止色 = 令牌名（数据侧写令牌名，渲染层只解析）。
                let from = param_str(p, "from_token")
                    .and_then(|name| theme.by_name(&name))
                    .unwrap_or(theme.bg);
                let to = param_str(p, "to_token")
                    .and_then(|name| theme.by_name(&name))
                    .unwrap_or(theme.bg);
                // 飞轮 R1 ⑥-12：级数钳制走 `generic::clamp_grad_steps`（双执行器同读，参数不漂移）。
                let steps =
                    usize::try_from(ti_render::generic::clamp_grad_steps(
                        i32::try_from(param_i64(p, "steps").unwrap_or(32)).unwrap_or(32),
                    ))
                    .unwrap_or(32);
                let band = rect.h as f32 / steps as f32;
                for i in 0..steps {
                    let t = i as f32 / (steps - 1) as f32;
                    let c = lerp_rgb565(from, to, t);
                    world.spawn((
                        Sprite::from_color(
                            rgb565_to_color(c),
                            Vec2::new(rect.w as f32, band + 1.0),
                        ),
                        Transform::from_translation(
                            to_world(
                                rect.x as f32 + rect.w as f32 / 2.0,
                                rect.y as f32 + i as f32 * band + band / 2.0,
                            )
                            .extend(z),
                        ),
                    ));
                }
            }
            "disc" => {
                // 实心圆盘 = 烘焙圆形纹理（月盘 / 圆形装饰）。
                let radius = rect.w as f32;
                let diameter = usize::try_from((radius * 2.0).round() as i64).unwrap_or(8);
                let mut raster = Raster::new(diameter, diameter);
                raster.disc(radius, radius, radius, color);
                let handle = world.resource_mut::<Assets<Image>>().add(raster.to_image());
                world.spawn((
                    Sprite {
                        image: handle,
                        custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                        ..Default::default()
                    },
                    Transform::from_translation(to_world(rect.x as f32, rect.y as f32).extend(z)),
                ));
            }
            "glow" => {
                let radius = rect.w as f32;
                let diameter = usize::try_from((radius * 2.0).round() as i64).unwrap_or(8);
                let mut raster = Raster::new(diameter, diameter);
                raster.radial_glow(radius, radius, radius, color);
                let handle = world.resource_mut::<Assets<Image>>().add(raster.to_image());
                world.spawn((
                    Sprite {
                        image: handle,
                        custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                        ..Default::default()
                    },
                    Transform::from_translation(to_world(rect.x as f32, rect.y as f32).extend(z)),
                ));
            }
            "icon" => {
                // 图标名 → id（`ui_icons.json` 注册表；未知名 = 0）。
                let kind = param_str(p, "icon")
                    .and_then(|name| ti_core::data::icon_ids().get(name.as_str()).copied())
                    .unwrap_or(0);
                // 飞轮 R1 ⑥-02：缺省边长读注册表 `default_size`（与 CPU 执行器同读，双端不漂移）；
                // 注册表整体缺失 = 48 crash-safe 兜底（同 ①-07 族口径，框架侧 `icons::ICON_SIZE` 恒等）。
                let default_size = match ti_core::data::ui_icons() {
                    Ok(reg) => i64::from(reg.default_size),
                    Err(_) => 48,
                };
                let size = i32::try_from(param_i64(p, "size").unwrap_or(default_size)).unwrap_or(48);
                let handle =
                    world.resource_mut::<Assets<Image>>().add(bake_icon(kind, size, color));
                let e = world.spawn((
                    Sprite {
                        image: handle,
                        custom_size: Some(Vec2::new(size as f32, size as f32)),
                        ..Default::default()
                    },
                    Transform::from_translation(
                        to_world(
                            rect.x as f32 + size as f32 / 2.0,
                            rect.y as f32 + size as f32 / 2.0,
                        )
                        .extend(z),
                    ),
                ));
                if p.item >= 0 {
                    out.selectable.push((idx as u64, e.id(), SelectableKind::Icon));
                }
            }
            "text" => {
                let content = if p.text_key.is_empty() {
                    p.text.clone()
                } else {
                    ti_app::i18n::sget(&p.text_key).to_owned()
                };
                if content.is_empty() {
                    continue;
                }
                let size = match p.tier.as_str() {
                    "title" => theme.title_font_size as f32,
                    "hint" => theme.hint_font_size as f32,
                    _ => theme.font_size as f32,
                };
                let mut text_font =
                    bevy::text::TextFont { font_size: FontSize::Px(size), ..Default::default() };
                if let Some(handle) = &font {
                    text_font.font = handle.clone().into();
                }
                let e = world.spawn((
                    Text2d::new(content),
                    TextColor(rgb565_to_color(color)),
                    text_font,
                    Transform::from_translation(
                        to_world(
                            rect.x as f32 + rect.w as f32 / 2.0,
                            rect.y as f32 + rect.h as f32 / 2.0,
                        )
                        .extend(z),
                    ),
                ));
                if p.item >= 0 {
                    out.selectable.push((idx as u64, e.id(), SelectableKind::Label));
                }
            }
            _ => {}
        }
    }
    out
}

/// 原语参数（字符串）。
fn param_str(p: &ti_core::data::Primitive, key: &str) -> Option<String> {
    p.params.get(key).and_then(|v| v.as_str()).map(str::to_owned)
}

/// 原语参数（数值）。
fn param_i64(p: &ti_core::data::Primitive, key: &str) -> Option<i64> {
    p.params.get(key).and_then(serde_json::Value::as_i64)
}

/// RGB565 线性插值（`t` = 0..=1；起止色均 RGB565）。
#[allow(clippy::many_single_char_names)]
fn lerp_rgb565(from: u16, to: u16, t: f32) -> u16 {
    let t = t.clamp(0.0, 1.0);
    let f = u32::from(from);
    let g = u32::from(to);
    let mix = |a: u32, b: u32| -> u32 {
        let d = i64::from(b) - i64::from(a);
        let v = i64::from(a) + (d * (t * 1000.0) as i64) / 1000;
        u32::try_from(v.max(0)).unwrap_or(0)
    };
    let r = mix((f >> 11) & 0x1F, (g >> 11) & 0x1F).min(31);
    let gg = mix((f >> 5) & 0x3F, (g >> 5) & 0x3F).min(63);
    let b = mix(f & 0x1F, g & 0x1F).min(31);
    u16::try_from((r << 11) | (gg << 5) | b).unwrap_or(0)
}

// 飞轮 R1 ⑥-04：`known_tokens`/`token_color` 已删（全仓零消费方死导出；
// 令牌不存在 = `theme.by_name` 返 None → 原语跳过，不 panic，数据侧无需名表）。

/// 绘制列表 → Bevy 实体（**通用执行器**：`DrawList` = 场景绘制命令序列）。
///
/// 覆盖**全部 7 场景**（场景代码零改动：绘制目标可插拔 = CPU 帧缓冲 / 绘制列表）。
/// - `Clear` → `ClearColor` 资源；
/// - `Rect` → `Sprite::from_color`；
/// - `Text` → `Text2d` + `TextFont`（GPU TTF）+ `TextColor`（位置：CPU 侧左上角 → Bevy 居中）。
///
/// **z 层**：命令序 = 绘制序 → z = `-(总数 - 序)`（负值可见；**越大越靠上** = 后画的在上，
/// 与 CPU 侧「调用序即层序」同义）。
/// 画制列表实体标记（S18.7a）：由 `DrawList` 生成的 sprite/text 实体。
///
/// 用途：每帧重建前按该标记清理旧实体（否则切场景留鬼影）。
#[derive(Component)]
pub struct DrawListEntity;

/// 画制列表 -> Bevy 实体（S18.7a）。
///
/// 每个 `DrawOp` -> 一个实体：`Rect` -> `Sprite`（实心定位），`Text` -> `Text2d`（字号档 = CPU 路径同源）；
/// `Clear` -> `ClearColor` 资源。层序 z 见函数内说明。
///
/// 返回生成的实体数。
// S18.8a：门禁新增 wasm 交付壳 clippy 覆盖（`cargo clippy -D warnings` 覆盖到本 crate），
// `Handle<Font>` 按值传入但函数体只读（`Text2d` 构造需要 owned clone）→ 显式豁免该 lint。
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_draw_list(
    world: &mut World,
    font: Option<Handle<Font>>,
    list: &ti_render::drawlist::DrawList,
) -> usize {
    let total = list.ops.len();
    let mut count = 0usize;
    for (idx, op) in list.ops.iter().enumerate() {
        // 层序 = 绘制顺序（z 越大越靠上 = EXP-103）；
        // **有界区间 [-12, 0]**：
        // - 上界 0 → 始终在背景 sprite（z=-13）之前，背景不遮挡；
        // - 步长 12/total → 与层数无关，深度精度稳定（不超 far=1000）。
        let z = -12.0 * ((total - idx) as f32 / total as f32);
        match op {
            ti_render::drawlist::DrawOp::Clear(color) => {
                world.insert_resource(ClearColor(rgb565_to_color(*color)));
            }
            ti_render::drawlist::DrawOp::Rect { x, y, w, h, color } => {
                world.spawn((
                    DrawListEntity,
                    Sprite::from_color(rgb565_to_color(*color), Vec2::new(*w as f32, *h as f32)),
                    Transform::from_translation(
                        to_world(*x as f32 + *w as f32 / 2.0, *y as f32 + *h as f32 / 2.0)
                            .extend(z),
                    ),
                ));
                count += 1;
            }
            ti_render::drawlist::DrawOp::Text { text, x, y, tier, color } => {
                // 文本宽度 = 与 CPU 路径同源度量（列表自带的 TTF）→ 两后端版面一致。
                let (width, height) = match tier {
                    ti_render::drawlist::Tier::Title => (
                        ti_render::text::tier_text_width(
                            text,
                            list.ttf.title.as_ref(),
                            list.theme.bitmap_advance_title,
                        ),
                        list.theme.title_font_size,
                    ),
                    ti_render::drawlist::Tier::Hint => (
                        ti_render::text::tier_text_width(
                            text,
                            list.ttf.hint.as_ref(),
                            list.theme.bitmap_advance_hint,
                        ),
                        list.theme.hint_font_size,
                    ),
                    ti_render::drawlist::Tier::Body => (
                        ti_render::text::tier_text_width(
                            text,
                            list.ttf.body.as_ref(),
                            list.theme.bitmap_advance,
                        ),
                        list.theme.font_size,
                    ),
                };
                let mut text_font = bevy::text::TextFont {
                    font_size: FontSize::Px(height as f32),
                    ..Default::default()
                };
                if let Some(handle) = &font {
                    text_font.font = handle.clone().into();
                }
                world.spawn((
                    DrawListEntity,
                    Text2d::new(text.clone()),
                    TextColor(rgb565_to_color(*color)),
                    text_font,
                    Transform::from_translation(
                        to_world(*x as f32 + width as f32 / 2.0, *y as f32 + height as f32 / 2.0)
                            .extend(z),
                    ),
                ));
                count += 1;
            }
            ti_render::drawlist::DrawOp::Image { asset, x, y, w, h } => {
                // S18.8b：贴图原语 → 资产服务器（路径相对 `assets/` 根）。
                let handle: Handle<Image> = world.resource::<AssetServer>().load(asset.clone());
                world.spawn((
                    DrawListEntity,
                    Sprite {
                        image: handle,
                        custom_size: Some(Vec2::new(*w as f32, *h as f32)),
                        ..Default::default()
                    },
                    Transform::from_translation(
                        to_world(*x as f32 + *w as f32 / 2.0, *y as f32 + *h as f32 / 2.0).extend(z),
                    ),
                ));
                count += 1;
            }
        }
    }
    count
}
