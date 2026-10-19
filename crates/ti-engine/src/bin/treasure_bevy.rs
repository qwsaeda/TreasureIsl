//! S18.4 Bevy 呈现壳：`GameApp` 逻辑 + CPU `Framebuf` → Bevy 纹理呈现（双渲染期过渡形态）。
//!
//! 架构（`docs/process/11_bevy_migration.md` §4）：
//! - **逻辑**：完整 `GameApp`（场景流转/战斗/经济/存档；`render_to_framebuf` 纯 CPU，0 漂移）；
//! - **呈现**：Bevy 2D（`Camera2d` + `Projection::Orthographic(scale=0.5)` letterbox 1080×1920
//!   + 逐帧 RGBA8 `Image` + `Sprite`，带宽口径 = S10.4 macroquad `FullRebuild`；
//!     S18.7 v3 场景落 Bevy 原生绘制后消除逐帧上传）；
//! - **输入**：Bevy 0.20 输入 API（`ButtonInput<KeyCode>` / `ButtonInput<MouseButton>` /
//!   `AccumulatedMouseMotion` / `Touches`）→ `KeyState` 沿 + `TouchPad::feed_screen`；
//! - **音频**：双渲染期关（bevy audio 与 macroquad `links = "alsa"` 冲突，§6；S18.8 切 `bevy_audio`）；
//! - **stats**：`Time` fps（静止跳帧指标退役，S18.4 重锚 = `stats::FpsMeter::bevy_tick`）。
//!
//! 三端口径：
//! - `native-shell`：`std::fs` 读仓库资源（与 macroquad native 壳同源）；
//! - `wasm-shell`：build-only（双渲染期资源链 = S18.8 bevy 资产管线；本壳 wasm 构建可跑
//!   无资源降级形态：全豆腐块 + 0 曲目 + 默认场景）；
//! - `android-shell`：本机无 NDK = SKIP（Mac 出包链路随 S18.8 更新 `android/README.md`）。
//!
#![allow(clippy::cast_precision_loss)]
//! 用法：
//! ```sh
//! cargo run -p ti-app --features bevy-shell,native-shell --bin treasure-bevy-native
//! ```

use bevy::a11y::AccessibilityPlugin;
use bevy::asset::{AssetPlugin, Assets, RenderAssetUsages};
use bevy::camera::{Camera2d, ClearColor, OrthographicProjection, Projection};
use bevy::image::{Image, ImagePlugin};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::MouseButton;
use bevy::input::touch::Touches;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::Sprite;
use bevy::text::TextPlugin;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy::winit::WinitPlugin;
use ti_app::game_app::{GameApp, GameInput, KeyState, Scene};
use ti_app::menu_shell::parse_font_bytes;
use ti_app::scenes::SceneKind;
use ti_app::stats::FpsMeter;
use ti_app::touch::TouchPad;
use ti_platform::gfx::UploadMode;
use ti_platform::settings::ScaleMode;

/// 窗口物理像素（540×960 = 1080×1920 逻辑竖屏半采样，与 macroquad native 壳同规格）。
const WIN_W: u32 = 540;
const WIN_H: u32 = 960;

/// 逻辑画布（`ti_render::framebuffer` 单一事实源）。
const LOGICAL_W: u32 = 1080;
const LOGICAL_H: u32 = 1920;

/// `GameApp` 存 Resource（系统按 `.chain()` 顺序独占 `Mutex` 借用；0.20 调度可能跑在
/// worker 线程 → `RefCell`（非 `Sync`）不可，用 `std::sync::Mutex`）。
#[derive(Resource)]
struct GameStore(std::sync::Mutex<GameApp>);

/// 输入沿状态（`Local`：与 macroquad 壳 `poll_input` 同表，`docs/gamebook.md` §3）。
#[derive(Default)]
struct InputState {
    /// 上一帧「按住」按键快照（沿 = 当前 && !上一帧）。
    prev_keys: KeyState,
    /// 上一帧 F1–F4 状态（功能键沿触发）。
    prev_function_keys: [bool; 4],
    /// 触屏指针 pad（屏幕像素 → 逻辑坐标手势归一）。
    touch: TouchPad,
}

/// 鼠标位置积分（`AccumulatedMouseMotion` 每帧 delta 自积，初值 = 窗口左上 (0,0)）。
#[derive(Resource, Default)]
struct MousePos {
    /// 窗口物理像素 x。
    x: f32,
    /// 窗口物理像素 y。
    y: f32,
}

/// 当前帧 RGBA8 纹理句柄（`Sprite.image` 显示目标；旧句柄 refcount 归零自动回收）。
#[derive(Resource, Default)]
struct FrameTex(Option<Handle<Image>>);

/// Startup：相机（正交 scale=0.5 letterbox 满铺）+ 占位纹理 + 全屏精灵。
#[allow(clippy::cast_possible_truncation)]
fn spawn_stage(mut commands: Commands, mut assets: ResMut<Assets<Image>>) {
    let scale = (WIN_W as f32 / LOGICAL_W as f32).min(WIN_H as f32 / LOGICAL_H as f32);
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale,
            near: 0.0,
            far: 1000.0,
            ..OrthographicProjection::default_2d()
        }),
        Transform::IDENTITY,
    ));
    let placeholder = Image::new(
        Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        TextureDimension::D2,
        vec![0u8; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    let handle = assets.add(placeholder);
    commands.insert_resource(FrameTex(Some(handle)));
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.1, 0.07, 0.05),
            Vec2::from([LOGICAL_W as f32, LOGICAL_H as f32]),
        ),
        Transform::IDENTITY,
    ));
}

/// RGB565 → RGBA8（5/6/5 位展 8 位，与 macroquad 呈现同量化口径）。
#[allow(clippy::cast_possible_truncation)]
fn rgb565_to_rgba8(fb: &[u16]) -> Vec<u8> {
    let mut out = vec![0u8; fb.len() * 4];
    for (i, c) in fb.iter().enumerate() {
        let r5 = (c >> 11) & 0x1F;
        let g6 = (c >> 5) & 0x3F;
        let b5 = c & 0x1F;
        out[i * 4] = ((u32::from(r5) * 255) / 31) as u8;
        out[i * 4 + 1] = ((u32::from(g6) * 255) / 63) as u8;
        out[i * 4 + 2] = ((u32::from(b5) * 255) / 31) as u8;
        out[i * 4 + 3] = 0xFF;
    }
    out
}

/// 下一缩放模式（循环，`ScaleMode::ALL` 顺序；与 macroquad 壳同函数口径）。
fn next_scale_mode(cur: ScaleMode) -> ScaleMode {
    let all = ScaleMode::ALL;
    let i = all.iter().position(|&m| m == cur).unwrap_or(0);
    all[(i + 1) % all.len()]
}

#[allow(clippy::needless_pass_by_value)]
/// 输入系统：键盘沿 + F1–F4 + 指针（触屏优先/鼠标兜底）→ `GameApp` 逻辑。
fn input_system(
    store: ResMut<GameStore>,
    mut mouse_pos: ResMut<MousePos>,
    mut acc: ResMut<AccumulatedMouseMotion>,
    kb: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut state: Local<InputState>,
) {
    mouse_pos.x += acc.delta.x;
    mouse_pos.y += acc.delta.y;
    acc.delta = bevy::prelude::Vec2::ZERO;
    let window_size = Vec2::new(WIN_W as f32, WIN_H as f32);

    let Ok(mut g) = store.0.lock() else {
        return;
    };
    // 键盘「按住」快照（KeyCode → 语义键，与 macroquad 壳同表）。
    let raw = KeyState {
        up: kb.pressed(KeyCode::ArrowUp),
        down: kb.pressed(KeyCode::ArrowDown),
        left: kb.pressed(KeyCode::ArrowLeft),
        right: kb.pressed(KeyCode::ArrowRight),
        confirm: kb.pressed(KeyCode::Enter) || kb.pressed(KeyCode::Space),
        back: kb.pressed(KeyCode::Backspace) || kb.pressed(KeyCode::Escape),
        landmark: kb.pressed(KeyCode::Space),
        buy: kb.pressed(KeyCode::KeyT),
        sell: kb.pressed(KeyCode::KeyY),
        build: kb.pressed(KeyCode::KeyB),
    };
    let key_edge = |current: bool, previous: bool| current && !previous;
    let k = KeyState {
        up: key_edge(raw.up, state.prev_keys.up),
        down: key_edge(raw.down, state.prev_keys.down),
        left: key_edge(raw.left, state.prev_keys.left),
        right: key_edge(raw.right, state.prev_keys.right),
        confirm: key_edge(raw.confirm, state.prev_keys.confirm),
        back: key_edge(raw.back, state.prev_keys.back),
        landmark: key_edge(raw.landmark, state.prev_keys.landmark),
        buy: key_edge(raw.buy, state.prev_keys.buy),
        sell: key_edge(raw.sell, state.prev_keys.sell),
        build: key_edge(raw.build, state.prev_keys.build),
    };
    g.poll(&k);

    // F1–F4 功能键沿（缩放/过滤/边框/画质循环，与 macroquad 壳同行为）。
    let f1 = kb.just_pressed(KeyCode::F1);
    let f2 = kb.just_pressed(KeyCode::F2);
    let f3 = kb.just_pressed(KeyCode::F3);
    let f4 = kb.just_pressed(KeyCode::F4);
    if key_edge(f1, state.prev_function_keys[0]) {
        let next = next_scale_mode(g.shell_mut().scale_mode());
        g.shell_mut().set_scale_mode(next, window_size.x, window_size.y);
    }
    if key_edge(f2, state.prev_function_keys[1]) {
        g.shell_mut().cycle_filter_mode();
    }
    if key_edge(f3, state.prev_function_keys[2]) {
        g.shell_mut().cycle_border_style();
    }
    if key_edge(f4, state.prev_function_keys[3]) {
        let _ = g.shell_mut().cycle_quality();
    }
    state.prev_function_keys = [f1, f2, f3, f4];
    state.prev_keys = raw;

    // 指针：触屏优先（bevy touch 特性，S18.1 结论），否则鼠标（winit 在触屏设备
    // 合成指针事件，触摸映射结论同源）。
    let mut pointer_down =
        mouse.just_pressed(MouseButton::Left) || mouse.pressed(MouseButton::Left);
    let mut pointer_pos = Vec2::new(mouse_pos.x, mouse_pos.y);
    if let Some(t) = touches.iter().next() {
        pointer_pos = t.position();
        pointer_down = true;
    }
    let sample = state.touch.feed_screen(&g.shell().vp, pointer_pos.x, pointer_pos.y, pointer_down);
    g.poll_touch(sample);
}

#[allow(clippy::needless_pass_by_value)]
/// 渲染系统：GameApp 帧推进 + 静止跳渲染 + Framebuf → RGBA8 纹理。
fn render_system(
    store: ResMut<GameStore>,
    mut assets: ResMut<Assets<Image>>,
    mut frame_tex: ResMut<FrameTex>,
    mut sprites: Query<&mut Sprite>,
    time: Res<Time>,
    mut exits: MessageWriter<AppExit>,
) {
    let delta_ms = u32::try_from(time.delta().as_millis()).unwrap_or(16).clamp(1, 250);
    let Ok(mut g) = store.0.lock() else {
        return;
    };
    g.set_frame_delta_ms(delta_ms);
    if g.should_quit() {
        exits.write(AppExit::Success);
        return;
    }
    let scene_key = SceneKind::from_legacy(g.scene()).discriminant();
    if g.transition_active()
        || g.shell().last_rendered != Some(g.menu_selected())
        || g.shell().last_rendered_scene != Some(scene_key)
    {
        g.render();
        g.shell_mut().last_rendered = Some(g.menu_selected());
        g.shell_mut().last_rendered_scene = Some(scene_key);
        // 双渲染期无 bevy 音频：Sfx 队列清空防积压（S18.8 切 bevy_audio）。
        g.clear_last_sfx();
        let fb = g.render_to_framebuf().to_vec();
        let img = Image::new(
            Extent3d { width: LOGICAL_W, height: LOGICAL_H, depth_or_array_layers: 1 },
            TextureDimension::D2,
            rgb565_to_rgba8(&fb),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        let handle = assets.add(img);
        frame_tex.0 = Some(handle.clone());
        if let Ok(mut sprite) = sprites.single_mut() {
            sprite.image = handle;
        }
    }
}

/// stats（feature `stats`）：fps 打印（S18.4 重锚：静止跳帧指标退役，GPU 口径 = Time delta）。
#[allow(clippy::needless_pass_by_value)]
fn stats_system(mut meter: Local<FpsMeter>, time: Res<Time>) {
    let now_ms = time.elapsed().as_millis();
    if let Some(fps) = meter.bevy_tick(now_ms) {
        println!("[stats] avg {fps:.1} fps（bevy 口径：跳帧指标退役）");
    }
}

/// 资源加载（非 wasm 构建）：`std::fs` 读仓库资源（与 macroquad native 壳同源路径）。
#[cfg(not(target_arch = "wasm32"))]
fn load_resources(game: &mut GameApp) {
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .map_or_else(|| PathBuf::from("."), PathBuf::from)
    }
    fn read_local(name: &str) -> Option<Vec<u8>> {
        std::fs::read(repo_root().join("legacy").join("data").join(name)).ok()
    }
    if let (Some(font), Some(charset)) = (read_local("GoldLand.Fnt"), read_local("font.dat")) {
        game.shell_mut().set_font(parse_font_bytes(&font, &charset));
    }
    if let Some(ui_pak) = read_local("ui.pak") {
        game.set_atlas(&ui_pak);
    }
    if let (Some(pak), Some(map)) = (read_local("glmap_m01.pak"), read_local("glmapm01.Map")) {
        game.set_terrain(&pak, &map, "glmapm01.Map");
    }
    {
        let hd_dir = repo_root().join("assets_hd_pack").join("1x");
        if hd_dir.is_dir() {
            game.load_hd_pack_from_dir(&hd_dir);
        }
    }
    for name in [
        "index.Map",
        "intro.Map",
        "selectrole.Map",
        "selectmap.Map",
        "main.Map",
        "vs.Map",
        "msgs.Map",
        "saveload.Map",
    ] {
        if let Some(bytes) = read_local(name) {
            game.add_scene(name, &bytes);
        }
    }
}

/// wasm（双渲染期）：build-only 降级形态——资源链 = S18.8 bevy 资产管线，
/// 本壳 wasm 构建以无资源形态运行（豆腐块 + 默认场景，壳不 panic）。
#[cfg(target_arch = "wasm32")]
fn load_resources(_game: &mut GameApp) {
    eprintln!(
        "[bevy-wasm] 双渲染期降级形态：资源链待 S18.8 bevy 资产管线（当前无字体/场景，可玩度降级不 panic）"
    );
}

/// S18.4 e2e 自检：内嵌 `tests/scripts/full_run.json` 16 步主循环（同源同序，seed 42），
/// 在 Bevy 进程中驱动 `GameApp` 逻辑链并断言场景跳转（0 失败 = PASS）。
///
/// 用法：`treasure-bevy-native --selftest`（自动退出；`AppExit` 码 = 失败数 >0 时 Failure）。
#[derive(Resource)]
struct SelfTest {
    /// 已消费步数。
    step: usize,
    /// 当前步内重复计数（`attack` ×200）。
    repeat: u32,
    /// 失败断言数。
    failures: u32,
    /// 步骤是否耗尽（系统据此发 `AppExit`）。
    done: bool,
}

/// 16 步主循环（action, 重复次数）——与 `tests/scripts/full_run.json` 逐条同源。
const SELFTEST_STEPS: [(&str, u32); 16] = [
    ("confirm", 1),
    ("next", 1),
    ("confirm", 1),
    ("next", 1),
    ("confirm", 1),
    ("walk", 1),
    ("trade_buy", 1),
    ("trade_sell", 1),
    ("build", 1),
    ("landmark", 1),
    ("attack", 200),
    ("back", 1),
    ("move_down", 1),
    ("confirm", 1),
    ("write_save", 1),
    ("load_save", 1),
];

/// 每步预期场景（`full_run.json` `expect` 列逐条同源）。
const SELFTEST_EXPECT: [&str; 16] = [
    "SelectRole",
    "SelectRole",
    "SelectMap",
    "SelectMap",
    "MainMap",
    "MainMap",
    "MainMap",
    "MainMap",
    "MainMap",
    "Vs",
    "Vs",
    "Index",
    "Index",
    "Saveload",
    "Saveload",
    "MainMap",
];

/// 步 action → `GameInput`（与 headless bin `to_input` 同映射，参数取 `full_run.json` 值）。
#[allow(clippy::match_same_arms)]
fn selftest_input(action: &str) -> GameInput {
    use ti_core::domain::economy::Building;
    use ti_core::domain::map::Dir;
    match action {
        "confirm" => GameInput::Confirm,
        "next" => GameInput::Next,
        "move_down" => GameInput::MoveDown,
        "back" => GameInput::Back,
        "walk" => GameInput::Walk(Dir::South),
        "trade_buy" => GameInput::TradeBuy(1),
        "trade_sell" => GameInput::TradeSell(1),
        "build" => GameInput::Build(Building::Castle),
        "landmark" => GameInput::Landmark(6),
        "attack" => GameInput::Attack,
        "write_save" => GameInput::WriteSave(0),
        "load_save" => GameInput::LoadSave(0),
        _ => GameInput::Confirm,
    }
}

/// 场景 → 名字（与 headless bin `scene_name` 同表）。
fn selftest_scene_name(scene: Scene) -> &'static str {
    use ti_app::game_app::Scene as S;
    match scene {
        S::Intro => "Intro",
        S::Index => "Index",
        S::SelectRole => "SelectRole",
        S::SelectMap => "SelectMap",
        S::MainMap => "MainMap",
        S::Vs => "Vs",
        S::Message => "Message",
        S::Saveload => "Saveload",
        S::Exit => "Exit",
    }
}

#[allow(clippy::needless_pass_by_value)]
/// 自检系统：每帧推进 1 步（ttack 200 帧），断言场景 = 预期。
fn selftest_system(
    store: ResMut<GameStore>,
    mut test: ResMut<SelfTest>,
    mut exits: MessageWriter<AppExit>,
) {
    let Ok(mut g) = store.0.lock() else {
        return;
    };
    let (action, repeat) = SELFTEST_STEPS[test.step];
    g.handle(selftest_input(action));
    test.repeat = test.repeat.saturating_add(1);
    if test.repeat < repeat {
        return;
    }
    test.repeat = 0;
    let expect = SELFTEST_EXPECT[test.step];
    let got = selftest_scene_name(g.scene());
    if got != expect {
        test.failures = test.failures.saturating_add(1);
        eprintln!(
            "[selftest] step {step_index} ({action}) 失败：预期 {expect}，实际 {got}",
            step_index = test.step
        );
    }
    test.step = test.step.saturating_add(1);
    if test.step >= SELFTEST_STEPS.len() {
        test.done = true;
        if test.failures == 0 {
            println!(
                "[selftest] PASS（{}/{} 步，0 失败；S5.2 全循环在 Bevy 壳内 e2e 通过）",
                test.step,
                SELFTEST_STEPS.len()
            );
            exits.write(AppExit::Success);
        } else {
            eprintln!("[selftest] FAIL（{} 失败/{} 步）", test.failures, SELFTEST_STEPS.len());
            exits.write(AppExit::error());
        }
    }
}

/// 主：构造壳 + Bevy App（S18.1 实测最小 plugin 链，关 audio）+ 三系统。
fn main() {
    let selftest = std::env::args().any(|a| a == "--selftest");
    let mut game = GameApp::with_upload_mode(WIN_W as f32, WIN_H as f32, UploadMode::Partial);
    load_resources(&mut game);
    // 双渲染期无 bevy 音频：曲目数 0（S18.8 切 bevy_audio 后恢复）。
    game.shell_mut().set_audio(0, 0);
    if selftest {
        // S5.2 同源固定种子（full_run.json seed=42），e2e 可复现。
        game.set_rng_seed(42);
    }

    let window = Window {
        resolution: WindowResolution::new(WIN_W, WIN_H),
        title: "金银岛 TreasureIsl（Bevy）".to_owned(),
        resizable: false,
        ..Default::default()
    };
    let mut app = App::new();
    let _ = app
        .add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(AssetPlugin::default())
        .add_plugins(ImagePlugin::default())
        .add_plugins(MeshPlugin)
        .add_plugins(AccessibilityPlugin)
        .add_plugins(bevy::input::InputPlugin)
        .add_plugins(RenderPlugin::default())
        .add_plugins(WinitPlugin::default())
        .add_plugins(WindowPlugin { primary_window: Some(window), ..Default::default() })
        .add_plugins(TextPlugin)
        .init_resource::<ClearColor>()
        .init_resource::<MousePos>()
        .insert_resource(GameStore(std::sync::Mutex::new(game)));
    if selftest {
        app.insert_resource(SelfTest { step: 0, repeat: 0, failures: 0, done: false })
            .add_systems(Update, (selftest_system, render_system, stats_system).chain());
    } else {
        app.add_systems(Update, (input_system, render_system, stats_system).chain());
    }
    app.add_systems(Startup, spawn_stage);
    app.run();
}
