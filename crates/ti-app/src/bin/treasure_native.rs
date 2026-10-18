//! 桌面（native）入口：`std::fs` 读本地资源 → 共享 `ti_app::menu_shell` → macroquad 窗口。
//!
//! 与 `treasure_wasm` 壳**共用同一份表现层**（[`MenuShell`]），差异只在资源 IO：
//! - wasm：`macroquad::file::load_file`（HTTP fetch）；
//! - native：`std::fs::read`（读仓库根 `legacy/data/` 下 4 个原始文件，与 wasm 同源资源）。
//!
//! 用法：`cargo run -p ti-app --features native-shell --bin treasure-native`
//! 资源路径 = 仓库根 `legacy/data/{GoldLand.Fnt,font.dat,settings.dat,music.lpk}`。

use std::path::{Path, PathBuf};

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use macroquad::prelude::*;
use ti_app::audio_prep::{MusicBundle, prepare_music, wav_pcm16};
use ti_app::game_app::{GameApp, KeyState};
use ti_app::menu_shell::parse_font_bytes;
use ti_app::scenes::SceneKind;
use ti_app::touch::TouchPad;
use ti_platform::gfx::UploadMode;
use ti_platform::settings::ScaleMode;

#[cfg(feature = "stats")]
use macroquad::time::get_time;
#[cfg(feature = "stats")]
use ti_app::stats::FpsMeter;

/// BGM 音量（0.0..=1.0）。
const BGM_VOLUME: f32 = 0.5;
/// 单次音效（SE）音量。
const SE_VOLUME: f32 = 1.0;

/// 已载入的真实资源（启动时一次性构建）。
struct Shell {
    game: GameApp,
    /// S6.4：上一帧耗时（毫秒），驱动精灵动画帧推进。
    last_frame_ms: u32,
    /// S6.4：macroquad 全局时钟上一帧读数（毫秒）。
    prev_now_ms: u64,
    /// 上一帧按键快照，用于把 held 键转成单次按键沿，避免菜单连续跳变。
    prev_keys: KeyState,
    /// 上一帧 F1/F2/F3/F4 按下状态，用于功能键边沿触发。
    prev_function_keys: [bool; 4],
    touch: TouchPad,
    bgm: Option<Sound>,
    se: Option<Sound>,
}

/// 本帧相对上一帧的按键沿（只有“刚按下”才返回 true）。
fn key_edge(current: bool, previous: bool) -> bool {
    current && !previous
}

/// 取下一个缩放模式（循环，`ScaleMode::ALL` 顺序）。
fn next_scale_mode(cur: ScaleMode) -> ScaleMode {
    let all = ScaleMode::ALL;
    let i = all.iter().position(|&m| m == cur).unwrap_or(0);
    all[(i + 1) % all.len()]
}

/// 仓库根（`ti-app/src/bin` → 上溯 3 级）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// 读本地资源文件；失败返回 `None`（壳以全豆腐块 / 0 曲目运行，不静默空白）。
fn read_local(name: &str) -> Option<Vec<u8>> {
    let path = repo_root().join("legacy").join("data").join(name);
    std::fs::read(path).ok()
}
/// 把 `music.lpk` 解包 → 混音 → OGG 转码 → 逐条交给 macroquad 音频后端。
///
/// 任何一步失败都只让曲目数归 0（状态行可见），不在启动路径上失败。
/// S1.7b：MOD 条目转 OGG/Opus 后直接加载，BGM 优先取 OGG 曲目。
async fn load_audio_bank() -> (Option<MusicBundle>, Vec<(usize, Sound)>) {
    let Some(bytes) = read_local("music.lpk") else {
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

/// 选 BGM：优先取 OGG 曲目（索引 ≥ tracks.len()，S1.7b MOD 转码产物），
/// 其次取 WAV 混音曲目中最长的。
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

impl Shell {
    /// 异步构建壳：本地读字体 + 音效，起循环 BGM。
    ///
    /// native 桌面端用 [`UploadMode::FullRebuild`]（EXP-045：native macroquad 复用纹理的
    /// `update_*` 不重进本帧批次，部分上传不反映到屏幕；改每脏帧整帧 `from_rgba8` 重建，
    /// 静止帧也 `draw_quad` 复用上次纹理避免黑底）。
    async fn new() -> Self {
        let mut game =
            GameApp::with_upload_mode(screen_width(), screen_height(), UploadMode::FullRebuild);

        // 真实字体（GoldLand.Fnt + font.dat）
        if let (Some(font_bytes), Some(charset_bytes)) =
            (read_local("GoldLand.Fnt"), read_local("font.dat"))
        {
            game.shell_mut().set_font(parse_font_bytes(&font_bytes, &charset_bytes));
        }

        // S15.6 QA107：TTF 改**内置自动加载**（`GameApp::with_upload_mode` 构造壳时
        // `MenuShell::builtin_ttf_set`，QA128 进程级缓存）——本 bin 不再显式
        // `find_ttf_font` + `set_ttf_font`（旧 S5 路径已删；字体缺失 → 壳构造时
        // `None` 三档 + `eprintln!` 诊断，回退 12px 位图，§3 不静默空白）。
        // 精灵图集 + 场景 .Map（本地 data/ 读，blit 真实贴图）
        if let Some(ui_pak) = read_local("ui.pak") {
            game.set_atlas(&ui_pak);
        }
        // 地形（glmap_m01.pak 瓦片 + glmapm01.Map 布局），主地图 blit 真实地形
        if let (Some(pak), Some(map_bytes)) =
            (read_local("glmap_m01.pak"), read_local("glmapm01.Map"))
        {
            game.set_terrain(&pak, &map_bytes, "glmapm01.Map");
        }
        // S7.5/S8.9：HD PNG 档（`assets_hd_pack/1x/`）——存在则注入真彩贴图；
        // 缺失则跳过，走 RGB565 PCX 回退（不 panic）。
        #[cfg(feature = "hd-8bit")]
        {
            let hd_dir = repo_root().join("assets_hd_pack").join("1x");
            if hd_dir.is_dir() {
                game.load_hd_pack_from_dir(&hd_dir);
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
            if let Some(bytes) = read_local(name) {
                game.add_scene(name, &bytes);
            }
        }

        // 真实音效（music.lpk → 解包 → 混音 → OGG 转码 → macroquad 音频后端）
        let (bundle, loaded) = load_audio_bank().await;
        let total_count = loaded.len();
        let mod_count = bundle.as_ref().map_or(0, |b| b.mod_count);
        game.shell_mut().set_audio(total_count, mod_count);

        // S1.7b：BGM 优先取 OGG 曲目（MOD 转码），SE 取 WAV 最短
        let bgm = pick_bgm(&loaded, bundle.as_ref());
        let se = pick_sound(&loaded, bundle.as_ref(), false);

        let shell = Shell {
            game,
            touch: TouchPad::new(),
            bgm,
            se,
            last_frame_ms: 0,
            prev_now_ms: 0,
            prev_keys: KeyState::default(),
            prev_function_keys: [false; 4],
        };
        shell.start_bgm();
        shell
    }

    /// 起循环 BGM（桌面首次交互即出声，无浏览器自动播放限制）。
    fn start_bgm(&self) {
        if let Some(sound) = self.bgm.as_ref() {
            play_sound(sound, PlaySoundParams { looped: true, volume: BGM_VOLUME });
        }
    }

    /// 播放一次音效（`GameApp.last_sfx` 非空时，交互触发）。
    fn play_last_sfx(&mut self) {
        if self.game.last_sfx().is_some() {
            if let Some(sound) = self.se.as_ref() {
                play_sound(sound, PlaySoundParams { looped: false, volume: SE_VOLUME });
            }
        }
        // 触发后清空，避免同一音效连播（下一帧交互重新触发）
        self.game.clear_last_sfx();
    }

    // 键鼠映射见 `docs/gamebook.md` §3（壳层输入归一）：鼠标左键**不**映射到 `confirm`，
    // 触屏/鼠标走 `GameApp::poll_touch` 的元素命中判定（点菜单行/地图格），
    // 若同时并入 `confirm` 会重复触发。

    /// S6.4：用 macroquad 全局时钟算本帧耗时（毫秒），驱动精灵动画帧推进。
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn tick_frame_time(&mut self) {
        let now_ms = (macroquad::time::get_time().max(0.0) * 1000.0) as u64;
        let delta = now_ms.saturating_sub(self.prev_now_ms);
        self.prev_now_ms = now_ms;
        // 首帧或时钟回退时取默认步长（~60fps），避免动画静止
        self.last_frame_ms = u32::try_from(delta).unwrap_or(16).clamp(1, 250);
    }

    fn poll_input(&mut self) -> bool {
        let raw = KeyState {
            up: is_key_pressed(KeyCode::Up),
            down: is_key_pressed(KeyCode::Down),
            left: is_key_pressed(KeyCode::Left),
            right: is_key_pressed(KeyCode::Right),
            confirm: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space),
            back: is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Escape),
            landmark: is_key_pressed(KeyCode::Space),
            buy: is_key_pressed(KeyCode::T),
            sell: is_key_pressed(KeyCode::Y),
            build: is_key_pressed(KeyCode::B),
        };
        // 游戏按键只取“刚按下”沿：按住不重复触发菜单/选择/战斗，
        // 避免 is_key_pressed 每帧轮询造成快速连跳。
        let k = KeyState {
            up: key_edge(raw.up, self.prev_keys.up),
            down: key_edge(raw.down, self.prev_keys.down),
            left: key_edge(raw.left, self.prev_keys.left),
            right: key_edge(raw.right, self.prev_keys.right),
            confirm: key_edge(raw.confirm, self.prev_keys.confirm),
            back: key_edge(raw.back, self.prev_keys.back),
            landmark: key_edge(raw.landmark, self.prev_keys.landmark),
            buy: key_edge(raw.buy, self.prev_keys.buy),
            sell: key_edge(raw.sell, self.prev_keys.sell),
            build: key_edge(raw.build, self.prev_keys.build),
        };
        let mut changed = self.game.poll(&k);

        let f1 = is_key_pressed(KeyCode::F1);
        let f2 = is_key_pressed(KeyCode::F2);
        let f3 = is_key_pressed(KeyCode::F3);
        let f4 = is_key_pressed(KeyCode::F4);
        if key_edge(f1, self.prev_function_keys[0]) {
            let next = next_scale_mode(self.game.shell_mut().scale_mode());
            self.game.shell_mut().set_scale_mode(next, screen_width(), screen_height());
            changed = true;
        }
        if key_edge(f2, self.prev_function_keys[1]) {
            self.game.shell_mut().cycle_filter_mode();
            changed = true;
        }
        if key_edge(f3, self.prev_function_keys[2]) {
            self.game.shell_mut().cycle_border_style();
            changed = true;
        }
        // S7.6：F4 循环切换画质（原版/HD）
        if key_edge(f4, self.prev_function_keys[3]) {
            let _ = self.game.shell_mut().cycle_quality();
            changed = true;
        }
        self.prev_function_keys = [f1, f2, f3, f4];
        self.prev_keys = raw;

        changed |= self.poll_touch();
        changed
    }

    /// 触屏/鼠标：屏幕指针采样 → 逻辑坐标手势 → `GameApp::poll_touch`
    /// （`docs/modernization.md` §5，直接触摸界面元素，不画虚拟按键）。
    ///
    /// macroquad 在触屏设备上把触摸合成为鼠标事件，故鼠标路径与触屏路径同源。
    fn poll_touch(&mut self) -> bool {
        let (mx, my) = mouse_position();
        let down = is_mouse_button_pressed(MouseButton::Left);
        let sample = self.touch.feed_screen(&self.game.shell().vp, mx, my, down);
        self.game.poll_touch(sample)
    }
}

/// 窗口配置：S15.6 QA126 现代窗口（540×960 竖屏逻辑 1080×1920 的半采样，F1 可循环缩放模式）。
fn window_conf() -> Conf {
    Conf {
        window_title: "金银岛 TreasureIsl".to_owned(),
        window_width: 540,
        window_height: 960,
        ..Default::default()
    }
}

/// 主循环（S4.3 脏行带呈现 + FullRebuild 上传 + 可选 stats FPS）。
///
/// S5.1：`GameApp` 驱动全场景（菜单→选人→选图→主地图→贸易/建造→战斗→存读档），
/// 首帧强制 `render()` 一次，之后每帧 `poll_input` + `render` + `present`。
#[macroquad::main(window_conf)]
async fn main() {
    let mut shell = Shell::new().await;
    // 首帧强制渲染（font/audio 已注入，画好菜单再进主循环）
    shell.game.render();
    shell.game.shell_mut().last_rendered = Some(shell.game.menu_selected());
    #[cfg(feature = "stats")]
    let mut fps = FpsMeter::new();

    loop {
        // 视口先跟随窗口尺寸（letterbox / pillarbox，P4-2）：
        // 触摸命中判定用逻辑坐标，必须先有本帧视口才能正确反变换
        shell.game.shell_mut().refit(screen_width(), screen_height());

        // S6.4：算本帧耗时并喂给 GameApp（驱动多帧精灵动画推进）
        shell.tick_frame_time();
        shell.game.set_frame_delta_ms(shell.last_frame_ms);

        // S5.1：交互核心驱动（各场景输入 → GameApp.handle → 场景跳转）
        let changed = shell.poll_input();
        if shell.game.should_quit() {
            break;
        }
        // 交互触发音效（一次性）
        shell.play_last_sfx();

        // S5.1：场景切换 / 选中变化时整帧重渲染；否则 S4.3 静态帧优化跳帧
        let current_selected = shell.game.menu_selected();
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
        // 呈现（FullRebuild：静止帧整帧 skip，脏帧整帧 from_rgba8 重建纹理）
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
