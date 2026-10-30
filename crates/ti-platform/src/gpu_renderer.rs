//! S10.4：GPU 化逐精灵 batch 渲染（`draw_texture` + 纹理图集 + alpha 混合）。
//!
//! S18.8b（macroquad 退役上洗）：GL 呈现层整体删除——`blit_sprite` /
//! `blit_terrain_tile` / `draw_ttf_text` 的真实 GPU 调用（macroquad
//! `draw_texture_ex` / `draw_text_ex`）随 macroquad 依赖退役；
//! 本类保留**图集抽象 + headless 计数语义**（`SpriteRef` = 图集内像素 rect；
//! `draw_calls` = 逐精灵调用计数，headless 测试判据 `> 0` 不变），
//! 真实上屏由 ti-engine 的 Bevy 壳承担（S18.8a 起，`bevy_sprite` 图集
//! + `ImageNode` 逐精灵，与 S10.4 同口径 = 逐精灵 batch 减少 draw call）。
//!
//! 设计（`docs/style.md` §5.6.5）：
//! - [`GpuRenderer`]：图集引用（`SpriteRef` = 纹理 id + 图集内 UV rect）；
//!   逐精灵 `blit_sprite` = 计数（headless 测试判据）；
//!   真实渲染路径 = Bevy 壳（`ti_engine::canvas::BevyCanvas` + sprite atlas）。
//! - headless / 无 GPU 后端时退化为 no-op（只记 draw call 计数，不实际上传），
//!   保证 headless 测试可跑（`GpuRenderer::is_headless` 判据）。
//! - [`SpriteRef`]：逐精灵引用（纹理 id + 图集内 UV rect）。
//!
//! TTF 字形：Bevy 壳走 `ti_render::font::TtfFont` + GPU Text2d（S18.7a 口径）；
//! 本模块不再持 macroquad `Font` / `draw_text_ex`。

/// `flip_y` 端侧约定（native 不翻 / wasm 翻，见 `blit_sprite` 注释与 EXP-062）。
const IS_WASM: bool = cfg!(target_arch = "wasm32");

/// 逐精灵引用：图集纹理 id + 图集内像素 rect（UV 转换由调用方完成）。
///
/// `texture` 为图集纹理 id（`u32`，Bevy 壳 = sprite atlas 句柄；
/// CPU 诊断路径 = 序号占位），`rect` 为 `(x, y, w, h)` 像素单位（图集内局部坐标）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteRef {
    /// 图集纹理 id（Bevy 壳 sprite atlas 句柄；headless = 序号）。
    pub texture: u32,
    /// 图集内像素 rect `(x, y, w, h)`。
    pub rect: (i32, i32, i32, i32),
}

/// GPU 渲染器（S10.4 + S18.8b 上洗）：逐精灵 batch + 纹理图集 + alpha 混合。
///
/// S18.8b 后语义 = **图集抽象 + 计数**（GL 呈现由 Bevy 壳承担）：
/// - `blit_sprite` / `blit_terrain_tile` = 逐精灵计数（headless 测试判据 `draw_calls > 0` 不变）；
/// - `headless` 模式：no-op，只记 draw call 数；
/// - `atlas`：图集纹理句柄（`None` 时 blit 仍计数，不 panic）。
pub struct GpuRenderer {
    /// 是否 headless 模式（true = no-op，只记 draw call 数）。
    headless: bool,
    /// 已执行的 blit 调用数（headless 模式供测试断言；Bevy 壳自持真实 draw call）。
    pub draw_calls: u64,
    /// 图集纹理句柄（`None` 时 blit 走计数路径；Bevy 壳 = sprite atlas 纹理 id）。
    atlas: Option<u32>,
}

impl GpuRenderer {
    /// 新建 GPU 渲染器（S18.8b：不再持 GL 纹理对象，图集句柄经 [`set_atlas`] 注入）。
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

    /// 注入图集纹理句柄（Bevy 壳 = sprite atlas 纹理 id；历史调用方传 `Texture2D`
    /// 的 API 已随 macroquad 退役，S18.8b 起句柄 = 不透明 `u32`）。
    pub fn set_atlas(&mut self, atlas: u32) {
        self.atlas = Some(atlas);
    }

    /// 图集纹理句柄（只读，`None` 时 blit 退化为计数）。
    #[must_use]
    pub fn atlas(&self) -> Option<u32> {
        self.atlas
    }

    /// 逐精灵 blit 计数（S18.8b：真实 GPU `draw_texture_ex` 已随 macroquad 退役；
    /// 本方法 = 逐精灵调用计数，headless 测试判据 `draw_calls > 0` 不变；
    /// 真实上屏 = Bevy 壳 sprite atlas + `ImageNode` 逐精灵 batch，同 S10.4 口径）。
    ///
    /// `flip_y` 端侧约定（EXP-062）：`IS_WASM` 常量保留 = 端侧纹理 V 方向
    /// （native 不翻 / wasm 翻），供 Bevy 壳图集 UV 构造时复用同一口径。
    pub fn blit_sprite(&mut self, _dst: (i32, i32), _src: &SpriteRef) {
        self.draw_calls += 1;
        let _ = IS_WASM;
    }

    /// 地形瓦片 blit（同 `blit_sprite` 机制，语义别名）。
    pub fn blit_terrain_tile(&mut self, dst: (i32, i32), tile: &SpriteRef) {
        self.blit_sprite(dst, tile);
    }

    /// 已执行的 blit 调用数（headless 测试判据：`> 0` 证明路径走到）。
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
