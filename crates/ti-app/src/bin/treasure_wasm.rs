//! S2.5 浏览器入口：macroquad wasm 壳（`<canvas>` + miniquad JS 宿主）。
//!
//! S5.1 起：交互核心下沉到共享 [`ti_app::game_app::GameApp`]（13 玩法模块接线，
//! `docs/gamebook.md`），本 bin 只负责：
//! 1. `#[macroquad::main]` 异步主循环（wasm32 目标）；
//! 2. 资源走 `macroquad::file::load_file`（HTTP fetch）→ 喂给 `GameApp.shell`；
//! 3. 音效走 `music.lpk` → `prepare_music` → `macroquad::audio` 输出链。

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use macroquad::prelude::*;
use ti_app::audio_prep::{MusicBundle, prepare_music, wav_pcm16};
use ti_app::game_app::{GameApp, KeyState};
use ti_app::menu_shell::parse_font_bytes;
use ti_app::scenes::SceneKind;
use ti_app::touch::TouchPad;
use ti_platform::settings::ScaleMode;
use ti_platform::storage::{Backend, MemoryStore};

#[cfg(feature = "stats")]
use macroquad::time::get_time;
#[cfg(feature = "stats")]
use ti_app::stats::FpsMeter;

/// BGM 音量（`macroquad::audio::PlaySoundParams::volume`，0.0..=1.0）。
const BGM_VOLUME: f32 = 0.5;
/// 单次音效（SE）音量。
const SE_VOLUME: f32 = 1.0;

/// 已载入的真实资源（启动时一次性构建，wasm 单线程安全）。
struct Shell {
    /// S6.4：上一帧耗时（毫秒），驱动精灵动画帧推进。
    last_frame_ms: u32,
    /// S6.4：macroquad 全局时钟上一帧读数（毫秒）。
    prev_now_ms: u64,
    game: GameApp,
    touch: TouchPad,
    bgm: Option<Sound>,
    se: Option<Sound>,
}

impl Shell {
    /// 异步构建壳：加载真实字体与音效（HTTP fetch），起循环 BGM。
    async fn new() -> Self {
        let mut game = GameApp::new(screen_width(), screen_height());

        // 真实字体（GoldLand.Fnt + font.dat）
        if let (Ok(font_bytes), Ok(charset_bytes)) = (
            macroquad::file::load_file("assets/GoldLand.Fnt").await,
            macroquad::file::load_file("assets/font.dat").await,
        ) {
            game.shell_mut().set_font(parse_font_bytes(&font_bytes, &charset_bytes));
        }

        // 真实音效（music.lpk → 解包 → 混音 → OGG 转码 → macroquad 音频后端）
        let (bundle, loaded) = Self::load_audio_bank().await;
        let total_count = loaded.len();
        let mod_count = bundle.as_ref().map_or(0, |b| b.mod_count);
        game.shell_mut().set_audio(total_count, mod_count);

        // 精灵图集（ui.pak → SpriteAtlas）+ 场景 .Map 文件（blit 用）
        if let Ok(ui_pak) = macroquad::file::load_file("assets/ui.pak").await {
            game.set_atlas(&ui_pak);
        }
        // 地形（glmap_m01.pak 瓦片 + glmapm01.Map 布局），主地图 blit 真实地形
        if let (Ok(pak), Ok(map_bytes)) = (
            macroquad::file::load_file("assets/glmap_m01.pak").await,
            macroquad::file::load_file("assets/glmapm01.Map").await,
        ) {
            game.set_terrain(&pak, &map_bytes, "glmapm01.Map");
        }
        // P2-8/N7：HD PNG 档（`assets_hd_pack/1x/ui/`）。注入清单来自
        // `assets_hd_pack/hd_f1_manifest.json`（EXP-048/060：注入数 = 目录全量，
        // 不再硬编码 f1 数组）。缺失 manifest 时回退空列表（PCX 原路径，不 panic）。
        #[cfg(feature = "hd-8bit")]
        {
            use ti_app::hd_manifest::{
                HdF1Manifest, collect_hd_pngs_sync, manifest_expected_count,
            };
            let manifest: HdF1Manifest =
                match macroquad::file::load_file("assets_hd_pack/hd_f1_manifest.json").await {
                    Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
                    Err(_) => HdF1Manifest::default(),
                };
            if !manifest.static_f1.is_empty() || !manifest.multiframe_f1.is_empty() {
                // collect_hd_pngs_sync 是同步 fetch；wasm 侧在 async 上下文里逐条
                // await macroquad::file::load_file 后喂入同步闭包（先批量预取再 collect）。
                let static_names: Vec<String> = manifest
                    .static_f1
                    .iter()
                    .map(|f1| format!("assets_hd_pack/1x/ui/{f1}_hd.png"))
                    .collect();
                let multi_names: Vec<String> = manifest
                    .multiframe_f1
                    .iter()
                    .flat_map(|f1| {
                        (0..5).map(move |fr| format!("assets_hd_pack/1x/ui/{f1}_hd_f{fr}.png"))
                    })
                    .collect();
                let all_names: Vec<String> = static_names.into_iter().chain(multi_names).collect();
                let mut fetch_map: std::collections::HashMap<String, Vec<u8>> =
                    std::collections::HashMap::new();
                for name in &all_names {
                    if let Ok(bytes) = macroquad::file::load_file(name.as_str()).await {
                        fetch_map.insert(name.clone(), bytes);
                    }
                }
                let mapping = collect_hd_pngs_sync(&manifest, &mut |name| fetch_map.remove(name));
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
            if let Ok(bytes) = macroquad::file::load_file(&format!("assets/{name}")).await {
                game.add_scene(name, &bytes);
            }
        }

        // S1.7b：BGM 优先取 OGG 曲目（MOD 转码），SE 取 WAV 最短
        let bgm = pick_bgm(&loaded, bundle.as_ref());
        let se = pick_sound(&loaded, bundle.as_ref(), false);

        let shell =
            Shell { game, touch: TouchPad::new(), bgm, se, last_frame_ms: 0, prev_now_ms: 0 };
        shell.start_bgm();
        shell
    }

    /// 起循环 BGM（浏览器自动播放策略下可能要等首次交互才出声）。
    fn start_bgm(&self) {
        if let Some(sound) = self.bgm.as_ref() {
            play_sound(sound, PlaySoundParams { looped: true, volume: BGM_VOLUME });
        }
    }

    /// S6.4：用 macroquad 全局时钟算本帧耗时（毫秒），驱动精灵动画帧推进。
    fn tick_frame_time(&mut self) {
        let now_ms = (macroquad::time::get_time().max(0.0) * 1000.0) as u64;
        let delta = now_ms.saturating_sub(self.prev_now_ms);
        self.prev_now_ms = now_ms;
        // 首帧或时钟回退时取默认步长（~60fps），避免动画静止
        self.last_frame_ms = u32::try_from(delta).unwrap_or(16).clamp(1, 250);
    }

    /// 播放一次音效（`GameApp.last_sfx` 非空时）+ 记录选择（内存存储）。
    fn play_last_sfx(&mut self) {
        let sfx = self.game.last_sfx();
        let event_id = ti_app::MENU[self.game.menu_selected()].event_id;
        let mut store = MemoryStore::new();
        let _ = store.store("last_menu", &event_id.to_le_bytes());
        if sfx.is_some() {
            if let Some(sound) = self.se.as_ref() {
                play_sound(sound, PlaySoundParams { looped: false, volume: SE_VOLUME });
            }
        }
        self.game.clear_last_sfx();
    }

    /// 载入 `music.lpk` → 解包 → 混音 → OGG 转码 → 交给 macroquad 音频后端。
    ///
    /// 任何一步失败都只让曲目数归 0（状态行可见），不在启动路径上失败。
    /// OGG 曲目（S1.7b：INHL.MOD → opus-pure OGG/Opus）作为 BGM 候选一并装入。
    async fn load_audio_bank() -> (Option<MusicBundle>, Vec<(usize, Sound)>) {
        let Ok(bytes) = macroquad::file::load_file("assets/music.lpk").await else {
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
        // S1.7b：MOD 条目转 OGG/Opus 后直接加载（macroquad 的 quad-snd 支持 OGG）
        let base = loaded.len();
        for ogg_track in &bundle.ogg_tracks {
            if let Ok(sound) = load_sound_from_bytes(&ogg_track.ogg).await {
                loaded.push((base, sound));
            }
        }
        (Some(bundle), loaded)
    }
}

/// 选 BGM：优先取 OGG 曲目（索引 ≥ tracks.len()，S1.7b MOD 转码产物），
/// 其次取 WAV 混音曲目中最长的。
fn pick_bgm(loaded: &[(usize, Sound)], bundle: Option<&MusicBundle>) -> Option<Sound> {
    let bundle = bundle?;
    let ogg_start = bundle.tracks.len();
    // OGG 曲目（BGM）优先
    if let Some((_, sound)) = loaded.iter().find(|(i, _)| *i >= ogg_start) {
        return Some(sound.clone());
    }
    // 回退：WAV 最长
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

/// 窗口配置：强制原版逻辑分辨率 176×220（F-101）。
fn window_conf() -> Conf {
    Conf {
        window_title: "金银岛 TreasureIsl".to_owned(),
        window_width: 176,
        window_height: 220,
        ..Default::default()
    }
}

/// 主循环（S5.1：`GameApp` 驱动全场景 + S4.3 脏行带呈现 + 可选 stats FPS 打印）。
#[macroquad::main(window_conf)]
async fn main() {
    let mut shell = Shell::new().await;
    // 首帧强制渲染（font/audio 已注入）
    shell.game.render();
    shell.game.shell_mut().last_rendered = Some(shell.game.shell().selected);
    #[cfg(feature = "stats")]
    let mut fps = FpsMeter::new();

    loop {
        // 视口先跟随窗口尺寸（letterbox / pillarbox，P4-2）：
        // 触摸命中判定用逻辑坐标，必须先有本帧视口才能正确反变换
        shell.game.shell_mut().refit(screen_width(), screen_height());

        // S5.1：交互核心驱动（各场景输入 → GameApp.poll → 场景跳转）
        let k = KeyState {
            up: is_key_pressed(KeyCode::Up),
            down: is_key_pressed(KeyCode::Down),
            left: is_key_pressed(KeyCode::Left),
            right: is_key_pressed(KeyCode::Right),
            confirm: is_key_pressed(KeyCode::Enter),
            back: is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Escape),
            landmark: is_key_pressed(KeyCode::Space),
            buy: is_key_pressed(KeyCode::T),
            sell: is_key_pressed(KeyCode::Y),
            build: is_key_pressed(KeyCode::B),
        };
        // S6.4：算本帧耗时并喂给 GameApp（驱动多帧精灵动画推进）
        shell.tick_frame_time();
        shell.game.set_frame_delta_ms(shell.last_frame_ms);
        let mut changed = shell.game.poll(&k);
        // F1 循环切换缩放模式（等比/整数倍/拉伸/裁剪），立即按窗口尺寸重建视口
        if is_key_pressed(KeyCode::F1) {
            let all = ScaleMode::ALL;
            let cur = all.iter().position(|&m| m == shell.game.shell().scale_mode()).unwrap_or(0);
            let next = all[(cur + 1) % all.len()];
            shell.game.shell_mut().set_scale_mode(next, screen_width(), screen_height());
            changed = true;
        }
        // F2 循环切换上采样滤波（像素锐利/平滑）
        if is_key_pressed(KeyCode::F2) {
            shell.game.shell_mut().cycle_filter_mode();
            changed = true;
        }
        // F3 循环切换边框装饰样式（纯黑/深蓝/同背景）
        if is_key_pressed(KeyCode::F3) {
            shell.game.shell_mut().cycle_border_style();
            changed = true;
        }
        // S7.6 F4 循环切换画质模式（原版/HD）
        if is_key_pressed(KeyCode::F4) {
            shell.game.shell_mut().cycle_quality();
            changed = true;
        }
        // 触屏/鼠标：直接触摸界面元素，不画虚拟按键（`docs/modernization.md` §5）。
        // macroquad 把触摸合成为鼠标事件，故与桌面共用同一路径。
        {
            let (mx, my) = mouse_position();
            let down = is_mouse_button_pressed(MouseButton::Left);
            let sample = shell.touch.feed_screen(&shell.game.shell().vp, mx, my, down);
            changed |= shell.game.poll_touch(sample);
        }
        if shell.game.should_quit() {
            break;
        }
        // 交互触发音效（一次性）
        shell.play_last_sfx();

        // S5.1：场景切换 / 选中变化时整帧重渲染；否则 S4.3 静态帧优化跳帧
        let current_selected = shell.game.shell().selected;
        let scene_key = SceneKind::from_legacy(shell.game.scene()).discriminant();
        if changed
            || shell.game.shell().last_rendered != Some(current_selected)
            || shell.game.shell().last_rendered_scene != Some(scene_key)
        {
            // S7.5/S7.6：hd-8bit 开启且画质=HD 时走 RGBA8 HD 渲染，否则 RGB565 原路径
            #[cfg(feature = "hd-8bit")]
            if shell.game.shell().settings().quality == ti_platform::settings::QualityMode::Hd {
                shell.game.render_hd();
            } else {
                shell.game.render();
            }
            #[cfg(not(feature = "hd-8bit"))]
            shell.game.render();
            shell.game.shell_mut().last_rendered = Some(current_selected);
            shell.game.shell_mut().last_rendered_scene = Some(scene_key);
        }
        #[cfg(feature = "hd-8bit")]
        if shell.game.shell().settings().quality == ti_platform::settings::QualityMode::Hd {
            shell.game.present_hd();
        } else {
            shell.game.shell_mut().present();
        }
        #[cfg(not(feature = "hd-8bit"))]
        shell.game.shell_mut().present();
        #[cfg(feature = "stats")]
        {
            let total_ms_f = get_time().max(0.0) * 1000.0;
            let now_ms = u128::from(total_ms_f.max(0.0).min(u32::MAX as f64) as u32);
            let report =
                fps.tick(now_ms, shell.game.shell().uploads(), shell.game.shell().skipped());
            if let Some((avg, uploads, skipped)) = report {
                println!("[stats] avg {avg:.1} fps | uploads={uploads} skipped={skipped}");
            }
        }
        next_frame().await;
    }
}
