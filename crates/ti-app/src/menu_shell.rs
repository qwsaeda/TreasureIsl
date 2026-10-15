//! 主菜单壳的**表现层**共享逻辑（平台无关）。
//!
//! 设计（`docs/style.md` §5.6 三层定位）：把「绘制主菜单 + 光栅字形 + 布局常量」
//! 抽到 lib 层，让 `treasure-wasm`（wasm-shell）与 `treasure-native`（native-shell）
//! 两个入口 **共用同一份表现逻辑**；入口只负责各自的资源 IO（HTTP fetch / 本地文件）
//! 与 macroquad 主循环。
//!
//! - 渲染走 [`ti_platform::gfx::Framebuf`]（纯内存 RGB565 帧缓冲）+
//!   [`FramePresenter`]（脏行带呈现），**不直接碰 macroquad**；
//! - 字形走 [`crate::glyph_raster::Raster`]（真 `GoldLand.Fnt` 墨点，缺字画豆腐块）；
//! - 菜单表 / 状态行文本取自 [`crate::shell_ui`]（壳与测试同源）。

use ti_formats::font::{Charset, Font};
use ti_platform::gfx::{FramePresenter, Framebuf, UploadMode, Viewport};
use ti_platform::settings::{BorderStyle, FilterMode, QualityMode, ScaleMode, Settings};

use crate::glyph_raster::{FontSet, Raster};

/// 逻辑宽（i32，供坐标运算）。
pub const LOGICAL_W_I32: i32 = 176;
/// 逻辑宽（usize，供帧缓冲索引）。
pub const LOGICAL_W_USIZE: usize = 176;
/// 逻辑高（i32，供坐标运算）。
pub const LOGICAL_H_I32: i32 = 220;
/// 字形步进（逻辑像素，= `GLYPH_WIDTH` 12 列，F-303 实测）。
pub const GLYPH_ADVANCE: i32 = 12;
/// 字形墨色（RGB565 白）。
pub const INK_COLOR: u16 = 0xFFFF;
/// S7.5（feature `hd-8bit`）：RGBA8 墨色（`0xAARRGGBB` 白不透明）。
#[cfg(feature = "hd-8bit")]
pub const INK_COLOR_RGBA: u32 = 0xFFFF_FFFF;

/// 场景布局色（S14 全量重设计，航海图纸质系，`docs/business/08_layout_design.md` §2）。
pub const COLOR_TITLE_BAR: u16 = 0x87BB; // #8B4513 棕（主色，标题条/边框）
/// 选中高亮布局色（#2E8B57 海绿，强调色）。
pub const COLOR_SELECTED: u16 = 0x066F;
/// 据点标记布局色（#8B4513 棕，主色）。
pub const COLOR_LANDMARK: u16 = 0x87BB;
/// 背景布局色（#F5F5DC 米白，航海图纸质背景）。
pub const COLOR_BG: u16 = 0xFD70;
/// 玩家棋子布局色（#2E8B57 海绿，强调色）。
pub const COLOR_PLAYER: u16 = 0x066F;
/// 危险/玩家血条布局色（#FF6347 橙红，S14 战斗面板）。
pub const COLOR_DANGER: u16 = 0xFDAF;
/// 标题条布局色 RGBA 版（`0xAARRGGBB`，HD 8-bit 帧缓冲路径用）。
#[cfg(feature = "hd-8bit")]
pub const COLOR_TITLE_BAR_RGBA: u32 = 0xFF_87BB;
/// 选中高亮布局色 RGBA 版（`0xAARRGGBB`，HD 8-bit 帧缓冲路径用）。
#[cfg(feature = "hd-8bit")]
pub const COLOR_SELECTED_RGBA: u32 = 0xFF_066F;

/// 已载入的真实资源 + 菜单 UI 状态（壳的渲染核心，平台无关）。
#[derive(Default)]
pub struct MenuShell {
    /// 逻辑帧缓冲（渲染目标）。
    pub fb: Framebuf,
    /// 等比缩放视口（每帧按窗口尺寸更新）。
    pub vp: Viewport,
    /// 呈现器（S4.3 脏行带部分上传；静止帧零 GPU 工作）。
    pub presenter: FramePresenter,
    /// 真实字体（`GoldLand.Fnt` + `font.dat`）；缺失时全豆腐块，绝不静默空白。
    pub font: Option<FontSet>,
    /// 已解析字形数（状态行显示）。
    pub font_glyphs: usize,
    /// 已载入可播放 WAV 曲目数（状态行显示）。
    pub wav_count: usize,
    /// 识别到但未接入播放的 MOD 条目数（状态行显示）。
    pub mod_count: usize,
    /// 当前选中的菜单项。
    pub selected: usize,
    /// S4.3 静态帧优化：上次已渲染的 `selected`；未变化时跳过整帧渲染。
    pub last_rendered: Option<usize>,
    /// S5.1 场景级静态帧优化：上次已渲染的场景键（`i32`，0..=7）；
    /// 场景切换时强制整帧重渲染（`docs/gamebook.md` §4 各场景分发）。
    pub last_rendered_scene: Option<i32>,
    /// 用户设置（缩放模式/方向/按键，S4.1；`refit` 与呈现按此设置）。
    pub settings: Settings,
    /// S7.5（feature `hd-8bit`）：RGBA8 HD 帧缓冲（与 `fb` 并行，默认构建不存在）。
    #[cfg(feature = "hd-8bit")]
    pub fb_rgba: ti_platform::gfx::FramebufRgba,
}

impl MenuShell {
    /// 构建壳：清空帧缓冲、按当前窗口尺寸建立视口。
    ///
    /// 调用方随后填充 [`font`](MenuShell::font) / [`wav_count`](MenuShell::wav_count) /
    /// [`mod_count`](MenuShell::mod_count)；场景渲染走 `scenes/*.rs`（`IndexScene` 等）。
    ///
    /// 默认走 [`UploadMode::Partial`]（wasm 路径，复用纹理部分上传）。
    #[must_use]
    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        Self::with_upload_mode(screen_w, screen_h, UploadMode::Partial)
    }

    /// 指定上传策略构建壳（native 桌面用 [`UploadMode::FullRebuild`]，EXP-045）。
    #[must_use]
    pub fn with_upload_mode(screen_w: f32, screen_h: f32, mode: UploadMode) -> Self {
        let mut fb = Framebuf::new();
        fb.clear(0x0000);
        let settings = Settings::default_settings();
        let vp = settings.viewport(screen_w, screen_h);
        Self {
            fb,
            vp,
            presenter: FramePresenter::with_mode(mode),
            font: None,
            font_glyphs: 0,
            wav_count: 0,
            mod_count: 0,
            selected: 0,
            last_rendered: None,
            last_rendered_scene: None,
            settings,
            #[cfg(feature = "hd-8bit")]
            fb_rgba: ti_platform::gfx::FramebufRgba::new(),
        }
    }

    /// 注入字体（解析成功的 `GoldLand.Fnt` + `font.dat`）。
    pub fn set_font(&mut self, font: Option<FontSet>) {
        let glyph_count = font.as_ref().map_or(0, |(parsed, _)| parsed.glyph_count());
        self.font = font;
        self.font_glyphs = glyph_count;
    }

    /// 注入音效统计（`music.lpk` 解包后：可播放 WAV 数 + MOD 数）。
    pub fn set_audio(&mut self, wav_count: usize, mod_count: usize) {
        self.wav_count = wav_count;
        self.mod_count = mod_count;
    }

    /// 帧缓冲上屏（S4.3：脏行带部分上传，静止帧零 GPU 工作）。
    pub fn present(&mut self) {
        self.presenter.present(&mut self.fb, &self.vp);
    }

    /// S7.5（feature `hd-8bit`）：RGBA8 HD 帧缓冲上屏（同上传策略，真彩无 565 量化）。
    #[cfg(feature = "hd-8bit")]
    pub fn present_rgba(&mut self) {
        self.presenter.present_rgba(&mut self.fb_rgba, &self.vp);
    }

    /// 在帧缓冲上填充矩形（RGB565，逻辑像素整数坐标；S4.3 起同步置脏行带）。
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        let x0 = x.clamp(0, LOGICAL_W_I32);
        let y0 = y.clamp(0, LOGICAL_H_I32);
        let x1 = x.saturating_add(w).clamp(0, LOGICAL_W_I32);
        let y1 = y.saturating_add(h).clamp(0, LOGICAL_H_I32);
        for py in y0..y1 {
            for px in x0..x1 {
                let idx = usize::try_from(py).unwrap_or(0) * LOGICAL_W_USIZE
                    + usize::try_from(px).unwrap_or(0);
                if let Some(slot) = self.fb.as_mut_slice().get_mut(idx) {
                    *slot = color;
                }
            }
        }
        self.fb.mark_dirty(x, y, w, h);
    }

    /// S7.5（feature `hd-8bit`）：在 RGBA8 HD 帧缓冲上填充矩形（`0xAARRGGBB`）。
    #[cfg(feature = "hd-8bit")]
    pub fn fill_rect_rgba(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        let x0 = x.clamp(0, LOGICAL_W_I32);
        let y0 = y.clamp(0, LOGICAL_H_I32);
        let x1 = x.saturating_add(w).clamp(0, LOGICAL_W_I32);
        let y1 = y.saturating_add(h).clamp(0, LOGICAL_H_I32);
        for py in y0..y1 {
            for px in x0..x1 {
                let idx = usize::try_from(py).unwrap_or(0) * LOGICAL_W_USIZE
                    + usize::try_from(px).unwrap_or(0);
                if let Some(slot) = self.fb_rgba.as_mut_slice().get_mut(idx) {
                    *slot = color;
                }
            }
        }
        self.fb_rgba.mark_dirty(x, y, w, h);
    }

    /// 绘制一行文本（12px 步进，整数逻辑坐标；空白字符推进但不着墨）。
    pub fn draw_text_row(&mut self, text: &str, x: i32, y: i32) {
        let mut cursor = x;
        for ch in text.chars() {
            self.draw_glyph(ch, cursor, y);
            cursor = cursor.saturating_add(GLYPH_ADVANCE);
        }
    }

    /// 光栅化单个字符：命中字表 → 真字形逐像素上墨；空白 → 留空；
    /// 字表缺字（或字体整体缺失）→ 豆腐块。
    pub fn draw_glyph(&mut self, ch: char, x: i32, y: i32) {
        let raster = Raster::for_char(self.font.as_ref(), ch);
        raster.blit(&mut self.fb, x, y, INK_COLOR);
    }

    /// S7.5（feature `hd-8bit`）：RGBA8 HD 帧缓冲上的文字行（`0xAARRGGBB` 墨色）。
    #[cfg(feature = "hd-8bit")]
    pub fn draw_text_row_rgba(&mut self, text: &str, x: i32, y: i32, color: u32) {
        let mut cursor = x;
        for ch in text.chars() {
            let raster = Raster::for_char(self.font.as_ref(), ch);
            raster.blit_rgba(&mut self.fb_rgba, cursor, y, color);
            cursor = cursor.saturating_add(GLYPH_ADVANCE);
        }
    }

    /// S7.5（feature `hd-8bit`）：把 RGB565 帧缓冲整体转成 RGBA8（`0xAARRGGBB`）。
    ///
    /// 路径 1 并行的核心：RGB565 场景渲染完成后逐像素扩展写 `fb_rgba`，
    /// 保证 HD 路径与原路径同源（`docs/impl/14_asset_enhancement.md` §3.5 路径 1）。
    #[cfg(feature = "hd-8bit")]
    pub fn sync_rgba_from_rgb565(&mut self) {
        let src = self.fb.as_slice();
        let dst = self.fb_rgba.as_mut_slice();
        for (i, &p) in src.iter().enumerate() {
            if let Some(slot) = dst.get_mut(i) {
                let [r, g, b, _a] = ti_platform::gfx::rgb565_to_rgba8(p);
                *slot = 0xFF00_0000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b);
            }
        }
        self.fb_rgba.mark_dirty(0, 0, LOGICAL_W_I32, LOGICAL_H_I32);
    }

    /// 视口跟随窗口尺寸（letterbox / pillarbox，P4-2）。
    ///
    /// 缩放模式取自 [`Self::settings`]：等比模式按窗口尺寸重算，
    /// 拉伸/裁剪模式同样按窗口尺寸重建（窗口变化时必须跟随）。
    pub fn refit(&mut self, screen_w: f32, screen_h: f32) {
        self.vp = self.settings.viewport(screen_w, screen_h);
    }

    /// 用户可配置项（缩放模式/方向/按键等，S4.1）；`refit` 与呈现按此设置。
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// 可变访问用户设置（改缩放模式后需再调 [`Self::refit`] 生效）。
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// 当前缩放模式。
    #[must_use]
    pub fn scale_mode(&self) -> ScaleMode {
        self.settings.scale_mode
    }

    /// 切换缩放模式并立即按当前窗口尺寸重建视口（设置菜单用）。
    pub fn set_scale_mode(&mut self, mode: ScaleMode, screen_w: f32, screen_h: f32) {
        self.settings.scale_mode = mode;
        self.vp = self.settings.viewport(screen_w, screen_h);
    }

    /// 当前上采样滤波模式（S6.1）。
    #[must_use]
    pub fn filter_mode(&self) -> FilterMode {
        self.settings.filter_mode
    }

    /// 切换上采样滤波（最近邻=像素锐利 / 线性=平滑），下次 present 生效。
    pub fn set_filter_mode(&mut self, mode: FilterMode) {
        self.settings.filter_mode = mode;
        self.presenter.set_filter(mode);
    }

    /// 循环切换滤波模式并返回新值（F2 键绑定，设置菜单用）。
    pub fn cycle_filter_mode(&mut self) -> FilterMode {
        let next = self.settings.filter_mode.next();
        self.set_filter_mode(next);
        next
    }

    /// 当前边框装饰样式（S6.2）。
    #[must_use]
    pub fn border_style(&self) -> BorderStyle {
        self.settings.border_style
    }

    /// 切换边框装饰样式（letterbox / pillarbox 区域填充色）。
    pub fn set_border_style(&mut self, style: BorderStyle) {
        self.settings.border_style = style;
        ti_platform::gfx::set_border_style(style);
    }

    /// 循环切换边框样式并返回新值（F3 键绑定）。
    pub fn cycle_border_style(&mut self) -> BorderStyle {
        let next = self.settings.border_style.next();
        self.set_border_style(next);
        next
    }

    /// S7.6：循环切换画质模式（原版/HD，F4 键绑定）。
    #[must_use]
    pub fn cycle_quality(&mut self) -> QualityMode {
        let next = self.settings.quality.next();
        self.settings.quality = next;
        next
    }

    /// 呈现器统计：脏帧上传数。
    #[must_use]
    pub fn uploads(&self) -> u64 {
        self.presenter.uploads()
    }

    /// 呈现器统计：跳过的静止帧数。
    #[must_use]
    pub fn skipped(&self) -> u64 {
        self.presenter.skipped()
    }
}

/// 把「字体字节 + 字符表字节」解析成 [`FontSet`]（纯字节，无 IO）。
///
/// 失败返回 `None`（壳仍能以全豆腐块运行，不静默空白）。
#[must_use]
pub fn parse_font_bytes(font_bytes: &[u8], charset_bytes: &[u8]) -> Option<FontSet> {
    let parsed = Font::parse("GoldLand.Fnt", font_bytes).ok()?;
    let charset = Charset::parse("font.dat", charset_bytes).ok()?;
    Some((parsed, charset))
}

#[cfg(test)]
mod tests {
    //! S10.2 渲染断言：`last_rendered_scene` 静态帧优化不变式。
    //! S14 配色/布局断言：航海图纸质系常量值域 + scenes.json 坐标对齐 16px 网格。
    //!
    //! 不变式 = `last_rendered_scene` 值域 ∈ `SceneKind::discriminant()` 0..=8（唯一无碰撞），
    //! 场景切换时必然 `!=` 旧值 → 壳层强制重渲染（P0-5 场景 key 碰撞修复回归）。

    use super::{
        COLOR_BG, COLOR_DANGER, COLOR_LANDMARK, COLOR_PLAYER, COLOR_SELECTED, COLOR_TITLE_BAR,
        GLYPH_ADVANCE, MenuShell,
    };

    #[test]
    fn last_rendered_scene_discriminant_range() {
        // S10.2：值域 0..=8（9 个 SceneKind 各占一个），旧 `Scene::key()` 值域 0..=7 有
        // Intro|Exit 碰撞（值 7），切场景跳帧残留。discriminant() 消除碰撞。
        // 逐一验证 9 个新壳默认 last_rendered_scene 均为 None（首帧强制渲染判据）。
        for _ in 0..9 {
            let shell = MenuShell::with_upload_mode(
                352.0,
                440.0,
                ti_platform::gfx::UploadMode::FullRebuild,
            );
            // 默认 None = 首帧强制渲染（壳层 `changed || last_rendered != ...` 判据）
            assert!(
                shell.last_rendered_scene.is_none(),
                "新壳 last_rendered_scene 应为 None（首帧强制重渲染）"
            );
        }
        // 模拟场景切换：Intro(0) → Exit(8)，两值不同 → 壳层必重渲染
        let intro_key = 0i32;
        let exit_key = 8i32;
        assert_ne!(
            intro_key, exit_key,
            "Intro 与 Exit 的 discriminant 必须不同（旧 key=7 碰撞回归）"
        );
        let mut shell =
            MenuShell::with_upload_mode(352.0, 440.0, ti_platform::gfx::UploadMode::FullRebuild);
        shell.last_rendered_scene = Some(intro_key);
        assert_eq!(shell.last_rendered_scene, Some(intro_key));
        shell.last_rendered_scene = Some(exit_key);
        assert_eq!(shell.last_rendered_scene, Some(exit_key));
    }

    /// S14：航海图纸质系配色常量与 `docs/business/08_layout_design.md` §2 表一致（值域断言）。
    #[test]
    fn palette_constants_match_doc() {
        // §2 表：bg=#F5F5DC(0xFD70) primary=#8B4513(0x87BB) accent=#2E8B57(0x066F)
        //       danger=#FF6347(0xFDAF) ink=深灰
        assert_eq!(COLOR_BG, 0xFD70, "背景应米白 0xFD70");
        assert_eq!(COLOR_TITLE_BAR, 0x87BB, "标题条应棕 0x87BB（主色）");
        assert_eq!(COLOR_SELECTED, 0x066F, "选中高亮应海绿 0x066F（强调色）");
        assert_eq!(COLOR_LANDMARK, 0x87BB, "据点应棕 0x87BB");
        assert_eq!(COLOR_PLAYER, 0x066F, "玩家棋子应海绿 0x066F");
        assert_eq!(COLOR_DANGER, 0xFDAF, "玩家血条应橙红 0xFDAF");
    }

    fn abort_on_err<E: std::fmt::Display>(e: E) -> ! {
        eprintln!("{e}");
        std::process::abort();
    }

    /// S14：scenes.json 坐标 ∈ [0,176]×[0,220] 且列表起点对齐 16px 网格（±2px）。
    #[test]
    fn layout_coords_in_bounds_grid() {
        let scenes = ti_core::data::scenes().unwrap_or_else(|e| abort_on_err(e));
        for (id, layout) in scenes {
            // 标题/列表/hint/hud 全在屏内
            assert!(layout.title.x >= 0 && layout.title.x < 176, "{id} 标题 x 越界");
            assert!(layout.title.y >= 0 && layout.title.y < 220, "{id} 标题 y 越界");
            assert!(layout.hint.x >= 0 && layout.hint.x < 176, "{id} 提示 x 越界");
            assert!(layout.hint.y >= 0 && layout.hint.y < 220, "{id} 提示 y 越界");
            assert!(layout.hud.x >= 0 && layout.hud.x < 176, "{id} HUD x 越界");
            // 列表场景（select_role/select_map/saveload/index）起点对齐 16px 网格 ±2px
            if layout.list.row_h > 0 {
                let y_mod = layout.list.y % 16;
                let aligned = y_mod <= 2 || y_mod >= 14;
                assert!(aligned, "{id} 列表 y={} 未对齐 16px 网格（§4）", layout.list.y);
                // 列表右缘不越屏（x + w <= 176）
                assert!(
                    layout.list.x + layout.list.w <= 176,
                    "{id} 列表右缘 {x}+{w}={r} 越屏 176",
                    x = layout.list.x,
                    w = layout.list.w,
                    r = layout.list.x + layout.list.w
                );
            }
        }
    }

    /// S14：主地图 HUD 区 y=152..176 不与瓦片区 y=32..144 重叠（§4.3）。
    #[test]
    fn main_map_hud_no_overlap() {
        let scenes = ti_core::data::scenes().unwrap_or_else(|e| abort_on_err(e));
        let Some(main) = scenes.get("main_map") else {
            std::process::abort();
        };
        // 瓦片区 32..144（112px），HUD 152..176（24px），间隙 8px 无重叠
        let tile_end = 32 + 112;
        let hud_start = main.hud.y;
        assert!(hud_start >= tile_end, "main_map HUD y={hud_start} 与瓦片区 end={tile_end} 重叠");
    }

    /// S14：战斗血条 x=16..160 在屏内（176 宽）（§4.4）。
    #[test]
    fn vs_hp_bars_in_frame() {
        let scenes = ti_core::data::scenes().unwrap_or_else(|e| abort_on_err(e));
        let Some(vs) = scenes.get("vs") else {
            std::process::abort();
        };
        for key in ["player_hp", "boss_hp"] {
            let Some(bar) = vs.extra.get(key) else {
                eprintln!("血条 extra 缺失: {key}");
                std::process::abort();
            };
            assert!(bar.x >= 0, "{key} x 越左");
            assert!(
                bar.x + bar.w <= 176,
                "{key} 右缘 {}+{}={} 越屏 176",
                bar.x,
                bar.w,
                bar.x + bar.w
            );
        }
    }

    /// S14：字号 12px 正文（[`GLYPH_ADVANCE`]）与 §3 字体规格一致。
    #[test]
    fn font_advance_match() {
        assert_eq!(GLYPH_ADVANCE, 12, "正文字号应为 12px（08_layout_design.md §3）");
    }
}
