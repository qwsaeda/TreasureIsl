//! S2.5 浏览器入口：macroquad wasm 壳（`<canvas>` + miniquad JS 宿主）。
//!
//! 验收口径（`docs/plan-s0-s2.md` S2.5）：浏览器显示**主菜单 + 真实资源 + 中文 + 音效**，
//! 并解锁 `scripts/gate.ps1` 第 4 项的 wasm 行。
//!
//! 本文件只做三件事：
//! 1. `#[macroquad::main]` 异步主循环（wasm32 目标）；
//! 2. 把窗口强制为 `LOGICAL_W × LOGICAL_H` 逻辑分辨率（`docs/reverse-facts.md` F-101）；
//! 3. 主菜单绘制 —— 中文文本走**真实 `GoldLand.Fnt` 字形**（`ti_app::glyph_raster`，
//!    缺字画豆腐块），音效走**真实 `music.lpk` 曲目**（`ti_app::audio_prep` 的
//!    解包 → `parse_wav` → `Mixer` → `macroquad::audio` 输出链）。
//!
//! ⚠️ 玩法场景（S3.x）接入前，此处只呈现主菜单骨架，不伪造任何玩法逻辑；
//! 菜单 `event_id` 为**占位猜测值**（真实事件 id 属 S3.3，见 `ti_app::shell_ui`）。

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use macroquad::prelude::*;
use ti_app::audio_prep::{MusicBundle, prepare_music, wav_pcm16};
use ti_app::glyph_raster::{FontSet, Raster};
use ti_app::{MENU, status_texts};
use ti_formats::font::{Charset, Font};
use ti_platform::gfx::{Framebuf, Viewport};
use ti_platform::storage::{Backend, MemoryStore};

/// 逻辑宽（= 原版 176，`docs/reverse-facts.md` F-101）。
const LOGICAL_W: i32 = 176;
/// 逻辑宽（usize 形式，供帧缓冲索引）。
const LOGICAL_W_USIZE: usize = 176;
/// 逻辑高（= 原版 220）。
const LOGICAL_H: i32 = 220;
/// 字形步进（逻辑像素，= `ti_formats::font::GLYPH_WIDTH`，F-303 实测 12 列）。
const GLYPH_ADVANCE: i32 = 12;
/// 字形墨色（RGB565 白）。
const INK_COLOR: u16 = 0xFFFF;
/// 菜单行高（逻辑像素）。
const ROW_H: i32 = 20;
/// 菜单起始 y（逻辑像素）。
const MENU_TOP: i32 = 96;
/// 底部状态行起点 x（逻辑像素）。
const STATUS_X: i32 = 4;
/// 底部状态行 y（贴在最后一行菜单之下 = `LOGICAL_H - GLYPH_ADVANCE`）。
const STATUS_Y: i32 = LOGICAL_H - GLYPH_ADVANCE;
/// BGM 音量（`macroquad::audio::PlaySoundParams::volume`，0.0..=1.0）。
const BGM_VOLUME: f32 = 0.5;
/// 单次音效（SE）音量。
const SE_VOLUME: f32 = 1.0;

/// 真实音效装载结果（`music.lpk` → 混音 → macroquad 音频后端）。
#[derive(Default)]
struct AudioBank {
    /// 循环 BGM（`music.lpk` 里最长的 WAV 曲目）。
    bgm: Option<Sound>,
    /// 单次音效（最短的 WAV 曲目，菜单确认时触发）。
    se: Option<Sound>,
    /// 已交给音频后端的曲目数。
    wav_count: usize,
    /// 识别到但未接入播放的 MOD 条目数（播放器属 S1.7b，当前阻塞）。
    mod_count: usize,
}

/// 已载入的真实资源（启动时一次性构建，wasm 单线程安全）。
struct Shell {
    fb: Framebuf,
    vp: Viewport,
    font: Option<FontSet>,
    font_glyphs: usize,
    audio: AudioBank,
    store: MemoryStore,
    selected: usize,
}

impl Shell {
    /// 异步构建壳：加载真实字体（`GoldLand.Fnt` + `font.dat`）与真实音效
    /// （`music.lpk`），并立刻起循环 BGM。
    async fn new() -> Self {
        let mut fb = Framebuf::new();
        fb.clear(0x0000);
        let vp = Viewport::fit(screen_width(), screen_height());
        let font = load_font_set().await;
        let font_glyphs = font.as_ref().map_or(0, |(parsed, _)| parsed.glyph_count());
        let audio = load_audio_bank().await;
        let shell =
            Self { fb, vp, font, font_glyphs, audio, store: MemoryStore::new(), selected: 0 };
        shell.start_bgm();
        shell
    }

    /// 起循环 BGM（浏览器自动播放策略下可能要等首次交互才出声，见 evidence）。
    fn start_bgm(&self) {
        if let Some(sound) = self.audio.bgm.as_ref() {
            play_sound(sound, PlaySoundParams { looped: true, volume: BGM_VOLUME });
        }
    }

    /// 菜单确认：记录选择 + 触发一次真实音效。
    fn on_confirm(&mut self) {
        let event_id = MENU[self.selected].event_id;
        let _ = self.store.store("last_menu", &event_id.to_le_bytes());
        if let Some(sound) = self.audio.se.as_ref() {
            play_sound(sound, PlaySoundParams { looped: false, volume: SE_VOLUME });
        }
    }

    /// 绘制主菜单到逻辑帧缓冲（再由 `present_framebuffer` 上屏）。
    fn render(&mut self) {
        // 背景：原版主色（深蓝）+ 顶部标题条
        self.fb.clear(0x0010);
        self.fill_rect(0, 0, LOGICAL_W, 40, 0x01E0);

        // 中文标题（真字形；字体整体缺失时全是豆腐块，绝不静默空白）
        self.draw_text_row("金银岛", 58, 12);

        // 菜单条目（带选中高亮）
        for (i, item) in MENU.iter().enumerate() {
            let y = MENU_TOP + i32::try_from(i).unwrap_or(0) * ROW_H;
            if i == self.selected {
                self.fill_rect(18, y - 12, LOGICAL_W - 36, ROW_H - 2, 0x03E0);
            }
            self.draw_text_row(item.label, 56, y);
        }

        // 状态行：真实资源计数（DoD：真实资源 + 音效可证）
        let [glyphs, sounds, modules] =
            status_texts(self.font_glyphs, self.audio.wav_count, self.audio.mod_count);
        let mut cursor = STATUS_X;
        for text in [&glyphs, &sounds, &modules] {
            self.draw_text_row(text, cursor, STATUS_Y);
            let span =
                i32::try_from(text.chars().count()).unwrap_or(0).saturating_mul(GLYPH_ADVANCE);
            cursor = cursor.saturating_add(span);
        }
    }

    /// 在帧缓冲上填充矩形（RGB565，逻辑像素整数坐标）。
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        let x0 = x.clamp(0, LOGICAL_W);
        let y0 = y.clamp(0, LOGICAL_H);
        let x1 = x.saturating_add(w).clamp(0, LOGICAL_W);
        let y1 = y.saturating_add(h).clamp(0, LOGICAL_H);
        for py in y0..y1 {
            for px in x0..x1 {
                let idx = usize::try_from(py).unwrap_or(0) * LOGICAL_W_USIZE
                    + usize::try_from(px).unwrap_or(0);
                if let Some(slot) = self.fb.as_mut_slice().get_mut(idx) {
                    *slot = color;
                }
            }
        }
    }

    /// 绘制一行文本（12px 步进，整数逻辑坐标；空白字符推进但不着墨）。
    fn draw_text_row(&mut self, text: &str, x: i32, y: i32) {
        let mut cursor = x;
        for ch in text.chars() {
            self.draw_glyph(ch, cursor, y);
            cursor = cursor.saturating_add(GLYPH_ADVANCE);
        }
    }

    /// 光栅化单个字符：命中字表 → 真字形逐像素上墨；空白 → 留空；
    /// 字表缺字（或字体整体缺失）→ 豆腐块。
    fn draw_glyph(&mut self, ch: char, x: i32, y: i32) {
        let raster = Raster::for_char(self.font.as_ref(), ch);
        raster.blit(&mut self.fb, x, y, INK_COLOR);
    }

    /// 帧缓冲上屏。
    fn present(&self) {
        ti_platform::gfx::present_framebuffer(&self.fb, &self.vp);
    }
}

/// 载入真实字体资源（wasm 侧走 macroquad 的 `load_file` → HTTP fetch）。
async fn load_font_set() -> Option<FontSet> {
    let font_bytes = macroquad::file::load_file("assets/GoldLand.Fnt").await.ok()?;
    let charset_bytes = macroquad::file::load_file("assets/font.dat").await.ok()?;
    let parsed = Font::parse("GoldLand.Fnt", &font_bytes).ok()?;
    let charset = Charset::parse("font.dat", &charset_bytes).ok()?;
    Some((parsed, charset))
}

/// 载入 `music.lpk` → 解包 → 混音 → 交给 macroquad 音频后端。
///
/// 任何一步失败都只让曲目数归 0（状态行可见），不在启动路径上失败。
async fn load_audio_bank() -> AudioBank {
    let mut bank = AudioBank::default();
    let Ok(bytes) = macroquad::file::load_file("assets/music.lpk").await else {
        return bank;
    };
    let Ok(bundle) = prepare_music(&bytes) else {
        return bank;
    };
    bank.mod_count = bundle.mod_count;
    let mut loaded: Vec<(usize, Sound)> = Vec::new();
    for (index, track) in bundle.tracks.iter().enumerate() {
        let wav = wav_pcm16(track.sample_rate, &track.pcm);
        if let Ok(sound) = load_sound_from_bytes(&wav).await {
            loaded.push((index, sound));
        }
    }
    bank.wav_count = loaded.len();
    bank.bgm = pick_track(&loaded, &bundle, true);
    bank.se = pick_track(&loaded, &bundle, false);
    bank
}

/// 选曲：`longest` 取混音帧数最多的当 BGM，否则取最少的当单次音效。
fn pick_track(loaded: &[(usize, Sound)], bundle: &MusicBundle, longest: bool) -> Option<Sound> {
    let mut best: Option<(usize, &Sound)> = None;
    for (index, sound) in loaded {
        let Some(track) = bundle.tracks.get(*index) else {
            continue;
        };
        let better = match best {
            None => true,
            Some((frames, _)) if longest => track.frames > frames,
            Some((frames, _)) => track.frames < frames,
        };
        if better {
            best = Some((track.frames, sound));
        }
    }
    best.map(|(_, sound)| sound.clone())
}

/// 窗口配置：强制原版逻辑分辨率 176×220（`docs/reverse-facts.md` F-101）。
fn window_conf() -> Conf {
    Conf {
        window_title: "金银岛 TreasureIsl".to_owned(),
        window_width: 176,
        window_height: 220,
        ..Default::default()
    }
}

/// 主循环。
#[macroquad::main(window_conf)]
async fn main() {
    let mut shell = Shell::new().await;

    loop {
        // 键鼠交互（菜单上下 + 确认）
        if is_key_pressed(KeyCode::Up) {
            shell.selected = (shell.selected + MENU.len() - 1) % MENU.len();
        }
        if is_key_pressed(KeyCode::Down) {
            shell.selected = (shell.selected + 1) % MENU.len();
        }
        if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
            shell.on_confirm();
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // 视口跟随窗口尺寸（letterbox / pillarbox，P4-2）
        shell.vp = Viewport::fit(screen_width(), screen_height());
        shell.render();
        shell.present();
        next_frame().await;
    }
}
