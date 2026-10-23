//! 主菜单壳的**表现层**共享逻辑（平台无关）。
//!
//! 渲染走 [`ti_render::Canvas`] trait（纯内存 RGB565 帧缓冲）+
//! [`FramePresenter`]（脏行带呈现），**不直接碰 macroquad**。
//!
//! - 字形走 [`ti_render::font`]（真 `GoldLand.Fnt` 墨点 + TTF 光栅化）；
//! - 菜单表 / 状态行文本取自 [`crate::shell_ui`]（壳与测试同源）。

use ti_formats::font::{Charset, Font};
use ti_platform::gfx::{FramePresenter, UploadMode, Viewport};
use ti_platform::settings::{BorderStyle, FilterMode, QualityMode, ScaleMode, Settings};
use ti_render::framebuffer::{Framebuf, FramebufRgba};

use ti_render::canvas::Canvas;
use ti_render::font::{FontSet, TtfFont};
use ti_render::theme::Theme;

// 颜色常量重导出（从 chrome 模块，兼容旧导入路径）
pub use crate::scenes::chrome::{
    COLOR_BEVEL_HI, COLOR_BEVEL_SHADOW, COLOR_BG, COLOR_BLACK, COLOR_DANGER, COLOR_GOLD,
    COLOR_GOLD_BRIGHT, COLOR_GOLD_GLOW, COLOR_HILITE, COLOR_LANDMARK, COLOR_MENU_BG,
    COLOR_MENU_BORDER, COLOR_PARCH_DARK, COLOR_PARCH_LIGHT, COLOR_PLAYER, COLOR_SECONDARY,
    COLOR_SELECTED, COLOR_TEXT_ON_PARCH, COLOR_TEXT_ON_WOOD, COLOR_TITLE_BAR, COLOR_WOOD,
};

// 逻辑尺寸常量（从 ti_render::framebuffer 重导出）
pub use ti_render::framebuffer::{LOGICAL_H_I32, LOGICAL_W_I32, LOGICAL_W_USIZE};

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
    /// TTF 软件光栅化字体——**正文档**（UI-21，`TtfFont` @ 44px）；
    /// `Some` 时优先于 `font` 使用。
    pub ttf_font: Option<TtfFont>,
    /// QB115：TTF **标题档**（`TtfFont` @ 96px）。
    pub ttf_title: Option<TtfFont>,
    /// QB115：TTF **提示档**（`TtfFont` @ 28px）。
    pub ttf_hint: Option<TtfFont>,
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
    /// S5.1 场景级静态帧优化：上次已渲染的场景键（`i32`，0..=7）。
    pub last_rendered_scene: Option<i32>,
    /// 用户设置（缩放模式/方向/按键，S4.1；`refit` 与呈现按此设置）。
    pub settings: Settings,
    /// 渲染主题（颜色 + 字号，数据驱动）。
    pub theme: Theme,
    /// S7.5（feature `hd-8bit`）：RGBA8 HD 帧缓冲（与 `fb` 并行，默认构建不存在）。
    #[cfg(feature = "hd-8bit")]
    pub fb_rgba: FramebufRgba,
}

/// QA128：内置 TTF 三档的**进程级**缓存（同字节只读盘 + 只解析一次；模块级 static，
/// 关联 `static` 不允许进 impl 块）。
static BUILTIN_TTF_SET: std::sync::OnceLock<(Option<TtfFont>, Option<TtfFont>, Option<TtfFont>)> =
    std::sync::OnceLock::new();

impl MenuShell {
    /// 构建壳：清空帧缓冲、按当前窗口尺寸建立视口。
    #[must_use]
    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        Self::with_upload_mode(screen_w, screen_h, UploadMode::Partial)
    }

    /// 指定上传策略构建壳（native 桌面用 [`UploadMode::FullRebuild`]）。
    #[must_use]
    pub fn with_upload_mode(screen_w: f32, screen_h: f32, mode: UploadMode) -> Self {
        let mut fb = Framebuf::new();
        fb.clear(0x0000);
        let settings = Settings::default_settings();
        let vp = settings.viewport(screen_w, screen_h);
        let (ttf_title, ttf_font, ttf_hint) = Self::builtin_ttf_set();
        Self {
            fb,
            vp,
            presenter: FramePresenter::with_mode(mode),
            font: None,
            ttf_font,
            ttf_title,
            ttf_hint,
            font_glyphs: 0,
            wav_count: 0,
            mod_count: 0,
            selected: 0,
            last_rendered: None,
            last_rendered_scene: None,
            settings,
            theme: Self::load_theme(),
            #[cfg(feature = "hd-8bit")]
            fb_rgba: FramebufRgba::new(),
        }
    }

    /// 进程级 TTF 三档（标题 96 / 正文 44 / 提示 28）。
    #[must_use]
    fn builtin_ttf_set() -> (Option<TtfFont>, Option<TtfFont>, Option<TtfFont>) {
        let cached = BUILTIN_TTF_SET.get_or_init(Self::load_builtin_ttf);
        (cached.0.clone(), cached.1.clone(), cached.2.clone())
    }

    /// 内置 TTF 自动加载（`assets/fonts/lxgw-wenkai-reg.ttf` → 三档）。
    fn load_builtin_ttf() -> (Option<TtfFont>, Option<TtfFont>, Option<TtfFont>) {
        let path = Self::repo_root().map(|root| root.join("assets/fonts/lxgw-wenkai-reg.ttf"));
        let Some(p) = path else {
            eprintln!(
                "[font] assets/fonts/lxgw-wenkai-reg.ttf not found; TTF missing, fall back to 12px bitmap"
            );
            return (None, None, None);
        };
        let Ok(bytes) = std::fs::read(&p) else {
            eprintln!(
                "[font] assets/fonts/lxgw-wenkai-reg.ttf read failed; fall back to 12px bitmap"
            );
            return (None, None, None);
        };
        #[allow(clippy::cast_precision_loss)]
        let title_size = 96.0f32;
        #[allow(clippy::cast_precision_loss)]
        let body_size = 44.0f32;
        #[allow(clippy::cast_precision_loss)]
        let hint_size = 28.0f32;
        let title = TtfFont::from_bytes(&bytes, title_size);
        let body = TtfFont::from_bytes(&bytes, body_size);
        let hint = TtfFont::from_bytes(&bytes, hint_size);
        if title.is_none() || body.is_none() || hint.is_none() {
            eprintln!(
                "[font] assets/fonts/lxgw-wenkai-reg.ttf parse failed; fall back to 12px bitmap"
            );
        }
        (title, body, hint)
    }

    /// 仓库根定位：`current_dir()` 向上最多 8 级找 `assets/fonts/` 目录。
    fn repo_root() -> Option<std::path::PathBuf> {
        let mut dir = std::env::current_dir().ok()?;
        for _ in 0..8 {
            if dir.join("assets").join("fonts").is_dir() {
                return Some(dir);
            }
            dir = dir.parent()?.to_path_buf();
        }
        None
    }

    /// 加载渲染主题（S15.7 数据分离）：优先 `assets/data/theme.json`，缺失/坏则回落
    /// `Theme::default()`。**色板与字号的唯一事实源 = JSON**，Rust 侧只留兜底值。
    fn load_theme() -> Theme {
        let Some(root) = Self::repo_root() else {
            return Theme::default();
        };
        let path = root.join("assets").join("data").join("theme.json");
        match std::fs::read_to_string(&path) {
            Ok(text) => Theme::from_json_str(&text),
            Err(_) => Theme::default(),
        }
    }

    /// 注入字体（解析成功的 `GoldLand.Fnt` + `font.dat`）。
    pub fn set_font(&mut self, font: Option<FontSet>) {
        let glyph_count = font.as_ref().map_or(0, |f| f.glyphs.len());
        self.font = font;
        self.font_glyphs = glyph_count;
    }

    /// 注入 TTF 字体三档。
    pub fn set_ttf_fonts(
        &mut self,
        title: Option<TtfFont>,
        body: Option<TtfFont>,
        hint: Option<TtfFont>,
    ) {
        self.ttf_title = title;
        self.ttf_font = body;
        self.ttf_hint = hint;
    }

    /// 三档 TTF 在位与否。
    #[must_use]
    pub fn ttf_loaded(&self) -> bool {
        self.ttf_font.is_some() || self.ttf_title.is_some() || self.ttf_hint.is_some()
    }

    /// 文本像素宽（正文档口径）。
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn text_width_px(&self, text: &str) -> i32 {
        Self::tier_text_width_px(text, self.ttf_font.as_ref(), 12)
    }

    /// 标题档文本宽。
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn title_text_width_px(&self, text: &str) -> i32 {
        Self::tier_text_width_px(text, self.ttf_title.as_ref(), 26)
    }

    /// 提示档文本宽。
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn hint_text_width_px(&self, text: &str) -> i32 {
        Self::tier_text_width_px(text, self.ttf_hint.as_ref(), 8)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn tier_text_width_px(text: &str, ttf: Option<&TtfFont>, bitmap_advance: i32) -> i32 {
        if let Some(t) = ttf {
            let width = text.chars().map(|ch| t.advance(ch)).sum::<f32>();
            width.ceil() as i32
        } else {
            let count = i32::try_from(text.chars().count()).unwrap_or(i32::MAX);
            count.saturating_mul(bitmap_advance)
        }
    }

    /// 字形垂直高（正文档）。
    #[must_use]
    pub fn glyph_px(&self) -> i32 {
        if self.ttf_font.is_some() { self.theme.font_size } else { self.theme.bitmap_advance }
    }

    /// 标题档字形垂直高。
    #[must_use]
    pub fn title_glyph_px(&self) -> i32 {
        if self.ttf_title.is_some() {
            self.theme.title_font_size
        } else {
            self.theme.bitmap_advance
        }
    }

    /// 提示档字形垂直高。
    #[must_use]
    pub fn hint_glyph_px(&self) -> i32 {
        if self.ttf_hint.is_some() { self.theme.hint_font_size } else { self.theme.bitmap_advance }
    }

    /// 行高（正文档）。
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn line_height_px(&self) -> i32 {
        match &self.ttf_font {
            Some(ttf) => ttf.line_height().ceil() as i32,
            None => self.theme.bitmap_advance,
        }
    }

    /// 单字符推进宽（正文档口径）。
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn char_advance_px(&self, ch: char) -> i32 {
        match &self.ttf_font {
            Some(ttf) => ttf.advance(ch).ceil() as i32,
            None => self.theme.bitmap_advance,
        }
    }

    /// 注入音效统计。
    pub fn set_audio(&mut self, wav_count: usize, mod_count: usize) {
        self.wav_count = wav_count;
        self.mod_count = mod_count;
    }

    /// 帧缓冲上屏（S4.3：脏行带部分上传，静止帧零 GPU 工作）。
    pub fn present(&mut self) {
        self.presenter.present(&self.fb, &self.vp);
    }

    /// 在帧缓冲上填充矩形（RGB565，逻辑像素整数坐标）。
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        // S15.10：像素写入单源 = 渲染框架 `Framebuf::fill_rect`（本方法不再自带像素循环）；
        // 脏行标记属呈现层，转 `presenter.mark_dirty`。
        self.fb.fill_rect(x, y, w, h, color);
        self.presenter.mark_dirty(x, y, w, h);
    }

    /// 整帧调亮/压暗（场景切换淡入、消息页压暗）并标全帧脏。
    ///
    /// S15.10：像素操作在渲染框架帧缓冲，脏行跟踪在呈现层，故整帧像素操作后
    /// 必须显式标全帧脏（旧实现在帧缓冲内部 `dirty_rows.fill(true)`）。
    pub fn apply_brightness(&mut self, factor: f32) {
        self.fb.apply_brightness(factor);
        self.presenter.mark_all_dirty();
    }

    /// 从上一帧缓冲渐变到当前内容（场景切换淡入，EXP-081）并标全帧脏。
    pub fn crossfade_from(&mut self, prev: &[u16], t: f32) {
        self.fb.crossfade_from(prev, t);
        self.presenter.mark_all_dirty();
    }

    /// 绘制一行文本（正文档，默认墨色 = 深底白字 [`COLOR_TEXT_ON_PARCH`]）。
    pub fn draw_text_row(&mut self, text: &str, x: i32, y: i32) {
        self.draw_text_row_colored(text, x, y, COLOR_TEXT_ON_PARCH);
    }

    /// 绘制一行**指定墨色**的文本（正文档）。
    pub fn draw_text_row_colored(&mut self, text: &str, x: i32, y: i32, color: u16) {
        let ttf = self.ttf_font.clone();
        match ttf {
            Some(f) => self.draw_ttf_row(text, x, y, color, &f),
            None => self.draw_bitmap_row(text, x, y, color, self.theme.bitmap_advance),
        }
    }

    /// 标题档文本行。
    pub fn draw_title_row(&mut self, text: &str, x: i32, y: i32, color: u16) {
        let ttf = self.ttf_title.clone();
        match ttf {
            Some(f) => self.draw_ttf_row(text, x, y, color, &f),
            None => self.draw_bitmap_row(text, x, y, color, self.theme.bitmap_advance_title),
        }
    }

    /// 提示档文本行。
    pub fn draw_hint_row(&mut self, text: &str, x: i32, y: i32, color: u16) {
        let ttf = self.ttf_hint.clone();
        match ttf {
            Some(f) => self.draw_ttf_row(text, x, y, color, &f),
            None => self.draw_bitmap_row(text, x, y, color, self.theme.bitmap_advance_hint),
        }
    }

    fn draw_ttf_row(&mut self, text: &str, x: i32, y: i32, color: u16, ttf: &TtfFont) {
        use ab_glyph::{Font as AbGlyphFont, PxScale, ScaleFont};
        #[allow(clippy::cast_precision_loss)]
        let size_px = ttf.size_px();
        let scaled = ttf.font().as_scaled(PxScale::from(size_px));
        #[allow(clippy::cast_precision_loss)]
        let mut cursor = x as f32;
        for ch in text.chars() {
            let raster = ttf.rasterize(ch);
            if raster.ink_count() > 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let y_offset = match scaled.outline_glyph(scaled.scaled_glyph(ch)) {
                    Some(outlined) => -outlined.px_bounds().min.y.ceil() as i32,
                    None => 0,
                };
                #[allow(clippy::cast_possible_truncation)]
                raster.blit(&mut self.fb, cursor.round() as i32, y + y_offset, color);
            }
            cursor += ttf.advance(ch);
        }
    }

    fn draw_bitmap_row(&mut self, text: &str, x: i32, y: i32, color: u16, advance: i32) {
        let mut cursor = x;
        for ch in text.chars() {
            self.draw_glyph_colored(ch, cursor, y, color);
            cursor = cursor.saturating_add(advance);
        }
    }

    /// 光栅化单个字符。
    pub fn draw_glyph(&mut self, ch: char, x: i32, y: i32) {
        let raster = ti_render::font::bitmap::Raster::for_char(self.font.as_ref(), ch);
        raster.blit(&mut self.fb, x, y, COLOR_TEXT_ON_PARCH);
    }

    /// 光栅化单个字符到**指定墨色**。
    pub fn draw_glyph_colored(&mut self, ch: char, x: i32, y: i32, color: u16) {
        let raster = ti_render::font::bitmap::Raster::for_char(self.font.as_ref(), ch);
        raster.blit(&mut self.fb, x, y, color);
    }

    /// 视口跟随窗口尺寸（letterbox / pillarbox，P4-2）。
    pub fn refit(&mut self, screen_w: f32, screen_h: f32) {
        self.vp = self.settings.viewport(screen_w, screen_h);
    }

    /// 用户可配置项。
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// 可变访问用户设置。
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// 当前缩放模式。
    #[must_use]
    pub fn scale_mode(&self) -> ScaleMode {
        self.settings.scale_mode
    }

    /// 切换缩放模式并立即按当前窗口尺寸重建视口。
    pub fn set_scale_mode(&mut self, mode: ScaleMode, screen_w: f32, screen_h: f32) {
        self.settings.scale_mode = mode;
        self.vp = self.settings.viewport(screen_w, screen_h);
    }

    /// 当前上采样滤波模式。
    #[must_use]
    pub fn filter_mode(&self) -> FilterMode {
        self.settings.filter_mode
    }

    /// 切换上采样滤波。
    pub fn set_filter_mode(&mut self, mode: FilterMode) {
        self.settings.filter_mode = mode;
        self.presenter.set_filter(mode);
    }

    /// 循环切换滤波模式并返回新值。
    pub fn cycle_filter_mode(&mut self) -> FilterMode {
        let next = self.settings.filter_mode.next();
        self.set_filter_mode(next);
        next
    }

    /// 当前边框装饰样式。
    #[must_use]
    pub fn border_style(&self) -> BorderStyle {
        self.settings.border_style
    }

    /// 切换边框装饰样式。
    pub fn set_border_style(&mut self, style: BorderStyle) {
        self.settings.border_style = style;
        ti_platform::gfx::set_border_style(style);
    }

    /// 循环切换边框样式并返回新值。
    pub fn cycle_border_style(&mut self) -> BorderStyle {
        let next = self.settings.border_style.next();
        self.set_border_style(next);
        next
    }

    /// S7.6：循环切换画质模式。
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

    /// S7.5（feature `hd-8bit`）：RGBA8 HD 帧缓冲上屏。
    #[cfg(feature = "hd-8bit")]
    pub fn present_rgba(&mut self) {
        self.presenter.present_rgba(&self.fb_rgba, &self.vp);
    }

    /// S7.5（feature `hd-8bit`）：把 RGB565 帧缓冲整体转成 RGBA8。
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
        self.presenter.mark_all_dirty();
    }
}

/// 把「字体字节 + 字符表字节」解析成渲染框架的**中性** [`FontSet`]（纯字节，无 IO）。
///
/// S15.10（可复用性）：`ti_formats::font` 是**本游戏的字体载体格式**，渲染框架不认它。
/// 本函数是「格式层 → 框架层」的唯一转换点：逐字形把原始字节行解成 `u16` 行位图，
/// 产出 `ti_render::font::FontSet`。换游戏只需换本函数 + `ti-formats` 侧解析器。
#[must_use]
pub fn parse_font_bytes(font_bytes: &[u8], charset_bytes: &[u8]) -> Option<FontSet> {
    let parsed = Font::parse("GoldLand.Fnt", font_bytes).ok()?;
    let charset = Charset::parse("font.dat", charset_bytes).ok()?;
    let count = parsed.glyph_count();
    let mut glyphs = Vec::with_capacity(count);
    for i in 0..count {
        glyphs.push(ti_render::font::Glyph { rows: parsed.glyph(i)?.rows });
    }
    Some(FontSet { glyphs, chars: charset.chars })
}

// === ti_render::Canvas trait 实现 ===

impl Canvas for MenuShell {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.fill_rect(x, y, w, h, color);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.draw_text_row_colored(text, x, y, color);
    }

    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.draw_title_row(text, x, y, color);
    }

    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        self.draw_hint_row(text, x, y, color);
    }

    fn text_width(&self, text: &str) -> i32 {
        self.text_width_px(text)
    }

    fn title_text_width(&self, text: &str) -> i32 {
        self.title_text_width_px(text)
    }

    fn hint_text_width(&self, text: &str) -> i32 {
        self.hint_text_width_px(text)
    }

    fn glyph_height(&self) -> i32 {
        self.glyph_px()
    }

    fn title_glyph_height(&self) -> i32 {
        self.title_glyph_px()
    }

    fn hint_glyph_height(&self) -> i32 {
        self.hint_glyph_px()
    }

    fn clear_frame(&mut self, color: u16) {
        self.fb.clear(color);
    }

    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        let w = usize::try_from(LOGICAL_W_I32).unwrap_or(0);
        if x < 0 || y < 0 {
            return 0;
        }
        let xi = usize::try_from(x).unwrap_or(0);
        let yi = usize::try_from(y).unwrap_or(0);
        self.fb.as_slice().get(yi.wrapping_mul(w).wrapping_add(xi)).copied().unwrap_or(0)
    }

    /// 贴图（S18.8b `image` 原语）：RGBA8 资源 → 最近邻缩放 + alpha 混合进 RGB565 帧缓冲。
    ///
    /// 资源路径 = 相对 `assets/` 根（如 `ui/board.png`）；缺资源 = 不画（不 panic）。
    /// S21.14：无色调 = 委托 [`Self::blit_image_tint`] 传白（`0xFFFF` 恒等）。
    #[cfg(feature = "hd-8bit")]
    fn blit_image(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32) {
        self.blit_image_tint(asset, x, y, w, h, 0xFFFF);
    }

    /// 贴图 + 色调（S21.14 图标图片化跨层 tint）：源像素 RGB 按 `tint` multiply（`0xFFFF` 白 = 恒等），
    /// alpha 不变，再 alpha 混合进 RGB565 帧缓冲 —— 与 Bevy 侧 `Sprite.color` 乘纹理同口径。
    #[cfg(feature = "hd-8bit")]
    #[allow(clippy::many_single_char_names)]
    fn blit_image_tint(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32, tint: u16) {
        let Some(img) = crate::image_cache::get(asset) else { return };
        if w <= 0 || h <= 0 {
            return;
        }
        let iw = i32::try_from(img.width).unwrap_or(0);
        let ih = i32::try_from(img.height).unwrap_or(0);
        if iw <= 0 || ih <= 0 {
            return;
        }
        let (tr, tg, tb) = rgb565_to_rgb8(tint);
        let stride = usize::try_from(LOGICAL_W_I32).unwrap_or(0);
        let fb = self.fb.as_mut_slice();
        for dy in 0..h {
            let sy = (dy * ih / h).clamp(0, ih - 1);
            let row = y.saturating_add(dy);
            if !(0..LOGICAL_H_I32).contains(&row) {
                continue;
            }
            for dx in 0..w {
                let sx = (dx * iw / w).clamp(0, iw - 1);
                let col = x.saturating_add(dx);
                if !(0..LOGICAL_W_I32).contains(&col) {
                    continue;
                }
                let src_idx = (usize::try_from(sy).unwrap_or(0) * usize::try_from(iw).unwrap_or(0)
                    + usize::try_from(sx).unwrap_or(0))
                    * 4;
                let Some(px) = img.rgba.get(src_idx..src_idx + 4) else { continue };
                let dst_idx =
                    usize::try_from(row).unwrap_or(0) * stride + usize::try_from(col).unwrap_or(0);
                let Some(dst) = fb.get_mut(dst_idx) else { continue };
                // 源 RGB × tint（u16 累加防溢出；白 tint = 恒等），alpha 通道不变
                let r = u8::try_from((u16::from(px[0]) * u16::from(tr)) / 255).unwrap_or(0);
                let g = u8::try_from((u16::from(px[1]) * u16::from(tg)) / 255).unwrap_or(0);
                let b = u8::try_from((u16::from(px[2]) * u16::from(tb)) / 255).unwrap_or(0);
                *dst = ti_render::framebuffer::blend_rgba_over(*dst, r, g, b, px[3]);
            }
        }
    }
}

/// RGB565 → RGB8（S21.14 CPU 侧贴图 tint multiply 用；`0xFFFF` → (255,255,255) 恒等）。
#[cfg(feature = "hd-8bit")]
fn rgb565_to_rgb8(c: u16) -> (u8, u8, u8) {
    let r = u8::try_from(((c >> 11) & 0x1F) * 255 / 31).unwrap_or(0);
    let g = u8::try_from(((c >> 5) & 0x3F) * 255 / 63).unwrap_or(0);
    let b = u8::try_from((c & 0x1F) * 255 / 31).unwrap_or(0);
    (r, g, b)
}
