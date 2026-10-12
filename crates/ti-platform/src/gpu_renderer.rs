//! S10.4：GPU 化逐精灵 batch 渲染（macroquad `draw_texture` + 纹理图集 + alpha 混合）。
//!
//! 设计（`docs/style.md` §5.6.5）：
//! - [`GpuRenderer`]：持有 macroquad `Texture2D` 图集（`from_rgba8` / `load_texture_from_image`）；
//!   逐精灵 `blit_sprite` 走 `draw_texture_ex`（macroquad 内部自动 batch，减少 draw call）；
//!   alpha 混合：macroquad 默认 `Blend::Alpha`（SrcAlpha/DstAlpha），HD PNG 透明通道自动混合
//!   （不再 `png_rgba_to_framebuf_u32` 丢 A）。
//! - headless / 无 GPU 后端时退化为 no-op（只记 draw call 计数，不实际上传），
//!   保证 headless 测试可跑（`GpuRenderer::is_headless` 判据）。
//! - [`SpriteRef`]：逐精灵引用（纹理 id + 图集内 UV rect）。
//!
//! TTF 字形（`draw_ttf_text`）：macroquad `draw_text_ex` + `TextParams { font, font_size }`。
//! 字体子集文件放 `assets/fonts/NotoSansCJK-subset.ttf`（占位：S10.4 暂用 system TTF 或空，
//! 子集生成脚本待补）；若 macroquad `load_ttf` API 不确定，用 `Font::from_bytes` 备选。
//!
//! 不引入新 crate 依赖（macroquad 已有）。

use macroquad::prelude::Rect;
use macroquad::text::Font;
use macroquad::texture::Texture2D;

/// `flip_y` 端侧约定（native 不翻 / wasm 翻，见 `blit_sprite` 注释与 EXP-062）。
const IS_WASM: bool = cfg!(target_arch = "wasm32");

/// 逐精灵引用：图集纹理 id + 图集内像素 rect（UV 转换由调用方完成）。
///
/// `texture` 为 macroquad `Texture2D` 的 `id()`（`u32`），
/// `rect` 为 `(x, y, w, h)` 像素单位（图集内局部坐标）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteRef {
    /// macroquad 纹理 id（`Texture2D::id()`）。
    pub texture: u32,
    /// 图集内像素 rect `(x, y, w, h)`。
    pub rect: (i32, i32, i32, i32),
}

/// GPU 渲染器（S10.4）：逐精灵 batch + 纹理图集 + alpha 混合。
///
/// headless 模式（`new_headless`）：不实际调用 macroquad GPU API，
/// 只累加 `draw_calls` 计数，供 headless 测试断言 `draw_calls > 0`。
pub struct GpuRenderer {
    /// 是否 headless 模式（true = no-op，只记 draw call 数）。
    headless: bool,
    /// 已执行的 draw call 数（headless 模式供测试断言）。
    pub draw_calls: u64,
    /// 图集纹理（可选；`None` 时 blit 走 `headless` no-op 路径）。
    atlas: Option<Texture2D>,
}

impl GpuRenderer {
    /// 新建 GPU 渲染器（默认 headless = false，可实际调 macroquad GPU API）。
    #[must_use]
    pub fn new() -> Self {
        Self { headless: false, draw_calls: 0, atlas: None }
    }

    /// 新建 headless GPU 渲染器（no-op，只记 draw call 计数，供测试用）。
    #[must_use]
    pub fn new_headless() -> Self {
        Self { headless: true, draw_calls: 0, atlas: None }
    }

    /// 是否 headless 模式（测试判据）。
    #[must_use]
    pub fn is_headless(&self) -> bool {
        self.headless
    }

    /// 注入图集纹理（`Texture2D`，`from_rgba8` / `load_texture_from_image` 产出）。
    pub fn set_atlas(&mut self, atlas: Texture2D) {
        self.atlas = Some(atlas);
    }

    /// 图集纹理（只读，`None` 时 blit 退化为 no-op）。
    #[must_use]
    pub fn atlas(&self) -> Option<&Texture2D> {
        self.atlas.as_ref()
    }

    /// 逐精灵 blit：`dst` 为逻辑屏坐标，`src` 为图集内 rect。
    ///
    /// 走 macroquad `draw_texture_ex`（内部自动 batch，减少 draw call）；
    /// alpha 混合由 macroquad 默认 `Blend::Alpha`（SrcAlpha/DstAlpha）处理，
    /// HD PNG 透明通道自动混合（不再丢 A）。
    ///
    /// headless 模式：只累加 `draw_calls`，不调 GPU API。
    #[allow(clippy::cast_precision_loss)]
    pub fn blit_sprite(&mut self, dst: (i32, i32), src: &SpriteRef) {
        if self.headless {
            self.draw_calls += 1;
            return;
        }
        let Some(atlas) = self.atlas.as_ref() else {
            self.draw_calls += 1;
            return;
        };
        let (sx, sy, sw, sh) = src.rect;
        let (dx, dy) = dst;
        // Q022/EXP-062：flip_y 端侧固定约定（native 不翻 / wasm 翻），与 gfx.rs FramePresenter 同机制。
        // native（cfg!(target_arch != "wasm32")）flip_y=false；wasm（target_arch="wasm32"）flip_y=true。
        // 禁止因「画面反了」手动翻转，否则反复横跳（EXP-062）。
        let params = macroquad::texture::DrawTextureParams {
            source: Some(Rect {
                x: sx as f32,
                y: sy as f32,
                w: (sw as f32).max(1.0),
                h: (sh as f32).max(1.0),
            }),
            dest_size: Some(macroquad::prelude::vec2((sw as f32).max(1.0), (sh as f32).max(1.0))),
            flip_y: IS_WASM,
            ..Default::default()
        };
        macroquad::texture::draw_texture_ex(
            atlas,
            dx as f32,
            dy as f32,
            macroquad::color::WHITE,
            params,
        );
        self.draw_calls += 1;
    }

    /// 地形瓦片 blit（同 `blit_sprite` 机制，语义别名）。
    pub fn blit_terrain_tile(&mut self, dst: (i32, i32), tile: &SpriteRef) {
        self.blit_sprite(dst, tile);
    }

    /// TTF 矢量字形：`draw_text_ex` + `TextParams { font, font_size }`。
    ///
    /// 字体子集文件 `assets/fonts/NotoSansCJK-subset.ttf`（占位，子集生成脚本待补）；
    /// 若 macroquad `load_ttf` API 不确定，用 `Font::from_bytes` 备选。
    ///
    /// headless 模式：只累加 `draw_calls`，不调 GPU API。
    pub fn draw_ttf_text(&mut self, text: &str, pos: (f32, f32), font: &Font, size: u16) {
        if self.headless {
            self.draw_calls += 1;
            return;
        }
        let params =
            macroquad::text::TextParams { font: Some(font), font_size: size, ..Default::default() };
        macroquad::text::draw_text_ex(text, pos.0, pos.1, params);
        self.draw_calls += 1;
    }

    /// 已执行的 draw call 数（headless 测试判据：`> 0` 证明路径走到）。
    #[must_use]
    pub fn draw_call_count(&self) -> u64 {
        self.draw_calls
    }
}

impl Default for GpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}
