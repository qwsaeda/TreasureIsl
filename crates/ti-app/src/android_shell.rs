//! Android 壳桥接：把 `treasure-android` bin 的 macroquad 主循环抽成可被
//! `android-activity` JNI 入口（`miniquad::run(game_loop)`）调用的纯函数。
//!
//! 设计（`docs/plan-s5.md` §S5.5）：
//! - `run` 是游戏主循环本体（构建共享 `GameApp` + 触屏输入 + 逐帧渲染 176×220 帧缓冲），
//!   与 `treasure-android` bin 的 `#[macroquad::main]` 同源逻辑（单渲染线程，避免双开后端）；
//! - `android-activity` crate 的 `game_loop` 经本模块桥接，miniquad 的 android 入口
//!   线程直接调用，不 spawn 额外渲染线程。
//!
//! 资源 IO 走 `macroquad::file::load_file` 的 android 后端（APK assets），与 wasm 壳的
//! HTTP fetch 同源机制。横竖屏自适应（P4-2 letterbox/pillarbox 由 `FramePresenter` 跟随
//! `screen_width/height` 每帧 refit）。

use crate::audio_prep::{MusicBundle, prepare_music, wav_pcm16};
use crate::game_app::GameApp;
use crate::menu_shell::parse_font_bytes;
use crate::scenes::SceneKind;
use crate::touch::TouchPad;
use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use macroquad::prelude::*;

/// BGM 音量（0.0..=1.0）。
const BGM_VOLUME: f32 = 0.5;
/// 单次音效（SE）音量。
const SE_VOLUME: f32 = 1.0;

/// 读 APK assets 资源文件；失败返回 `None`（壳以全豆腐块 / 0 曲目运行，不静默空白）。
async fn read_asset(name: &str) -> Option<Vec<u8>> {
    match macroquad::file::load_file(name).await {
        Ok(bytes) => Some(bytes),
        Err(_) => None,
    }
}

/// 把 `music.lpk` 解包 → 混音 → OGG 转码 → 逐条交给 macroquad 音频后端。
async fn load_audio_bank() -> (Option<MusicBundle>, Vec<(usize, Sound)>) {
    let Some(bytes) = read_asset("assets/music.lpk").await else {
        return (None, Vec::new());
    };
    let Ok(bundle) = prepare_music(&bytes) else {
        return (None, Vec::new());
    };
    let mut loaded = Vec::new();
    for (index, track) in bundle.tracks.iter().enumerate() {
        let wav = wav_pcm16(track.sample_rate, &track.pcm);
        if let Ok(sound) = load_sound_from_bytes(&wav).await {
            loaded.push((index, sound));
        }
    }
    let base = loaded.len();
    for ogg_track in &bundle.ogg_tracks {
        if let Ok(sound) = load_sound_from_bytes(&ogg_track.ogg).await {
            loaded.push((base, sound));
        }
    }
    (Some(bundle), loaded)
}

/// 选 BGM：优先取 OGG 曲目（索引 ≥ tracks.len()），其次取 WAV 最长。
fn pick_bgm(loaded: &[(usize, Sound)], bundle: Option<&MusicBundle>) -> Option<Sound> {
    let bundle = bundle?;
    let ogg_start = bundle.tracks.len();
    if let Some((_, sound)) = loaded.iter().find(|(i, _)| *i >= ogg_start) {
        return Some(sound.clone());
    }
    pick_sound(loaded, Some(bundle), true)
}

/// 选曲：`longest` 取混音帧数最多的当 BGM，否则取最少的当单次音效。
fn pick_sound(
    loaded: &[(usize, Sound)],
    bundle: Option<&MusicBundle>,
    longest: bool,
) -> Option<Sound> {
    let bundle = bundle?;
    let mut best: Option<(usize, &Sound)> = None;
    for (index, sound) in loaded {
        if let Some(track) = bundle.tracks.get(*index) {
            let better = match best {
                None => true,
                Some((frames, _)) if longest => track.frames > frames,
                Some((frames, _)) => track.frames < frames,
            };
            if better {
                best = Some((track.frames, sound));
            }
        }
    }
    best.map(|(_, sound)| sound.clone())
}

/// 游戏主循环本体（`android-activity` JNI 入口经本函数桥接）。
///
/// 构建共享 `GameApp` 交互核心（13 玩法模块接线，`docs/gamebook.md`），加载 APK assets
/// 的字体/音效，起循环 BGM，逐帧触屏输入 + 渲染 176×220 逻辑帧缓冲。
/// 单渲染线程（`miniquad::run` 的 android 入口线程直接调用，不 spawn 额外线程）。
pub async fn run() {
    let mut game = GameApp::new(screen_width(), screen_height());

    // 真实字体（GoldLand.Fnt + font.dat，从 APK assets）
    if let (Some(font_bytes), Some(charset_bytes)) =
        (read_asset("assets/GoldLand.Fnt").await, read_asset("assets/font.dat").await)
    {
        game.shell_mut().set_font(parse_font_bytes(&font_bytes, &charset_bytes));
    }

    // 真实音效（music.lpk → 解包 → 混音 → OGG 转码 → macroquad 音频后端）
    let (bundle, loaded) = load_audio_bank().await;
    let total_count = loaded.len();
    let mod_count = bundle.as_ref().map_or(0, |b| b.mod_count);
    game.shell_mut().set_audio(total_count, mod_count);

    // 精灵图集（ui.pak → SpriteAtlas）+ 场景 .Map 文件（blit 真实贴图）
    if let Some(ui_pak) = read_asset("assets/ui.pak").await {
        game.set_atlas(&ui_pak);
    }
    let map_names = [
        "index.Map",
        "intro.Map",
        "selectrole.Map",
        "selectmap.Map",
        "main.Map",
        "vs.Map",
        "msgs.Map",
        "saveload.Map",
    ];
    for name in map_names {
        if let Some(bytes) = read_asset(&format!("assets/{name}")).await {
            game.add_scene(name, &bytes);
        }
    }

    // S7.5/S8.9 + N7：HD PNG 档（`assets_hd_pack/1x/ui/`）。注入清单来自
    // `assets_hd_pack/hd_f1_manifest.json`（EXP-048/060：注入数 = 目录全量，不硬编码）。
    #[cfg(feature = "hd-8bit")]
    {
        use crate::hd_manifest::{
            HdF1Manifest, load_hd_pngs_from_manifest, manifest_expected_count,
        };
        let manifest: HdF1Manifest = match read_asset("assets_hd_pack/hd_f1_manifest.json").await {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            None => HdF1Manifest::default(),
        };
        if !manifest.static_f1.is_empty() || !manifest.multiframe_f1.is_empty() {
            let mut fetch = |name: &str| async move { read_asset(name).await };
            let mapping = load_hd_pngs_from_manifest(&manifest, &mut fetch).await;
            let expected = manifest_expected_count(&manifest);
            if mapping.len() < expected {
                eprintln!(
                    "[hd] N7 注入 {}/{} f1（manifest 全量 {expected}；缺失 = PCX 回退，EXP-048/060 告警）",
                    mapping.len(),
                    expected
                );
            }
            if !mapping.is_empty() {
                game.load_hd_pngs(mapping);
            }
        }
    }

    // BGM 优先取 OGG 曲目（MOD 转码），SE 取 WAV 最短
    let bgm = pick_bgm(&loaded, bundle.as_ref());
    let se = pick_sound(&loaded, bundle.as_ref(), false);
    start_bgm(&bgm);

    // 触屏输入垫（直接触摸界面元素，不画虚拟按键，`docs/modernization.md` §5）
    let mut touch = TouchPad::new();

    // 首帧强制渲染（font/audio 已注入）
    #[cfg(feature = "hd-8bit")]
    if game.shell().settings().quality == ti_platform::settings::QualityMode::Hd {
        game.render_hd();
    } else {
        game.render();
    }
    #[cfg(not(feature = "hd-8bit"))]
    game.render();
    game.shell_mut().last_rendered = Some(game.menu_selected());

    // P0-7：帧计时（与 native 壳同机制，驱动多帧精灵动画推进）
    let mut prev_now_ms: u64 = 0;

    loop {
        // 视口先跟随窗口尺寸（letterbox / pillarbox，P4-2）
        game.shell_mut().refit(screen_width(), screen_height());

        // P0-7：算本帧耗时并喂给 GameApp（驱动多帧精灵动画帧推进）
        let now_ms = (macroquad::time::get_time().max(0.0) * 1000.0) as u64;
        let delta = now_ms.saturating_sub(prev_now_ms);
        prev_now_ms = now_ms;
        let frame_delta_ms = u32::try_from(delta).unwrap_or(16).clamp(1, 250);
        game.set_frame_delta_ms(frame_delta_ms);

        // 触屏输入 → GameApp.poll_touch → 场景跳转
        let (mx, my) = mouse_position();
        let down = is_mouse_button_pressed(MouseButton::Left);
        let sample = touch.feed_screen(&game.shell().vp, mx, my, down);
        let changed = game.poll_touch(sample);

        if game.should_quit() {
            break;
        }
        play_last_sfx(&mut game, se.as_ref());

        // 场景切换 / 选中变化时整帧重渲染；否则 S4.3 静态帧优化跳帧
        let current_selected = game.menu_selected();
        let scene_key = SceneKind::from_legacy(game.scene()).discriminant();
        if changed
            || game.shell().last_rendered != Some(current_selected)
            || game.shell().last_rendered_scene != Some(scene_key)
        {
            // S7.5/S7.6：hd-8bit 开启且画质=HD 时走 RGBA8 HD 渲染，否则 RGB565 原路径
            #[cfg(feature = "hd-8bit")]
            if game.shell().settings().quality == ti_platform::settings::QualityMode::Hd {
                game.render_hd();
            } else {
                game.render();
            }
            #[cfg(not(feature = "hd-8bit"))]
            game.render();
            game.shell_mut().last_rendered = Some(current_selected);
            game.shell_mut().last_rendered_scene = Some(scene_key);
        }
        // 呈现
        #[cfg(feature = "hd-8bit")]
        if game.shell().settings().quality == ti_platform::settings::QualityMode::Hd {
            game.present_hd();
        } else {
            game.shell_mut().present();
        }
        #[cfg(not(feature = "hd-8bit"))]
        game.shell_mut().present();
        next_frame().await;
    }
}

/// 起循环 BGM（Android 首次交互后出声，无桌面自动播放限制差异）。
fn start_bgm(bgm: &Option<Sound>) {
    if let Some(sound) = bgm {
        play_sound(sound, PlaySoundParams { looped: true, volume: BGM_VOLUME });
    }
}

/// 播放一次音效（`GameApp.last_sfx` 非空时，交互触发）+ 清空避免连播。
fn play_last_sfx(game: &mut GameApp, se: Option<&Sound>) {
    if game.last_sfx().is_some() {
        if let Some(sound) = se {
            play_sound(sound, PlaySoundParams { looped: false, volume: SE_VOLUME });
        }
    }
    game.clear_last_sfx();
}
