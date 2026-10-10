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
use crate::i18n::sget;
use crate::shell_ui::{MENU, status_texts};

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

/// 场景布局色（8-bit 调色板索引；S8.4 起由 `scenes.json` 统一供数，过渡期保留 const）。
pub const COLOR_TITLE_BAR: u16 = 0x01E0;
/// 选中高亮布局色（调色板索引 `0x03E0`）。
pub const COLOR_SELECTED: u16 = 0x03E0;
/// 据点标记布局色（调色板索引 `0x07C0`）。
pub const COLOR_LANDMARK: u16 = 0x07C0;
/// 背景布局色（调色板索引 `0x0010`，深蓝）。
pub const COLOR_BG: u16 = 0x0010;
/// 玩家棋子布局色（调色板索引 `0xFFFF`，白）。
pub const COLOR_PLAYER: u16 = 0xFFFF;
/// 标题条布局色 RGBA 版（`0xAARRGGBB`，HD 8-bit 帧缓冲路径用）。
#[cfg(feature = "hd-8bit")]
pub const COLOR_TITLE_BAR_RGBA: u32 = 0xFF00_01E0;
/// 选中高亮布局色 RGBA 版（`0xAARRGGBB`，HD 8-bit 帧缓冲路径用）。
#[cfg(feature = "hd-8bit")]
pub const COLOR_SELECTED_RGBA: u32 = 0xFF00_03E0;
/// 菜单行高（逻辑像素）。
pub const ROW_H: i32 = 20;
/// 菜单起始 y（逻辑像素）。
pub const MENU_TOP: i32 = 96;
/// 底部状态行起点 x（逻辑像素）。
pub const STATUS_X: i32 = 4;
/// 底部状态行 y（贴最后一行菜单之下）。
pub const STATUS_Y: i32 = LOGICAL_H_I32 - GLYPH_ADVANCE;

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
    /// [`mod_count`](MenuShell::mod_count) 再调 [`render`](MenuShell::render)。
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

    /// 菜单确认：记录选择（写存储由入口负责）并返回当前选中项。
    #[must_use]
    pub fn selected_item(&self) -> usize {
        self.selected
    }

    /// 菜单上下移动（循环）。
    pub fn move_selection(&mut self, up: bool) {
        let n = MENU.len();
        self.selected = if up { (self.selected + n - 1) % n } else { (self.selected + 1) % n };
    }

    /// 绘制主菜单到逻辑帧缓冲（再经 [`FramePresenter`] 按脏行带上屏）。
    pub fn render(&mut self) {
        // 背景：原版主色（深蓝）+ 顶部标题条
        self.fb.clear(COLOR_BG);
        self.fill_rect(0, 0, LOGICAL_W_I32, 40, COLOR_TITLE_BAR);

        // 中文标题（真字形；字体整体缺失时全是豆腐块，绝不静默空白）
        self.draw_text_row(sget("scene.index.title"), 58, 12);

        // 菜单条目（带选中高亮；文案标签运行时取 i18n 表）
        let labels = crate::shell_ui::menu_labels();
        for (i, _item) in MENU.iter().enumerate() {
            let y = MENU_TOP + i32::try_from(i).unwrap_or(0) * ROW_H;
            if i == self.selected {
                self.fill_rect(18, y - 12, LOGICAL_W_I32 - 36, ROW_H - 2, COLOR_SELECTED);
            }
            self.draw_text_row(labels[i], 56, y);
        }

        // 状态行：真实资源计数 + 画质模式（S7.6）
        let [glyphs, sounds, modules] =
            status_texts(self.font_glyphs, self.wav_count, self.mod_count);
        let quality_tag = if cfg!(feature = "hd-8bit") {
            format!("{} F4", self.settings.quality.label())
        } else {
            crate::i18n::sget("menu.origin_label").to_owned()
        };
        let mut cursor = STATUS_X;
        for text in [&glyphs, &sounds, &modules, &quality_tag] {
            self.draw_text_row(text, cursor, STATUS_Y);
            let span =
                i32::try_from(text.chars().count()).unwrap_or(0).saturating_mul(GLYPH_ADVANCE);
            cursor = cursor.saturating_add(span);
        }
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
