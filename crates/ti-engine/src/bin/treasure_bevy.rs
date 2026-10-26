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

use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::camera::{Camera2d, ClearColor, OrthographicProjection, Projection, RenderTarget};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::MouseButton;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::sprite::Sprite;
use bevy::text::Font;
use bevy::window::{Window, WindowResolution};
use ti_app::game_app::{GameApp, GameInput, KeyState, Scene};
// S18.8a：`parse_font_bytes` 仅 native 分支用（wasm 分支无 std::fs 资源读）→ 按目标编译。
#[cfg(not(target_arch = "wasm32"))]
use ti_app::menu_shell::parse_font_bytes;
use ti_app::scenes::SceneKind;
use ti_app::stats::FpsMeter;
use ti_app::touch::TouchPad;
use ti_platform::gfx::UploadMode;
use ti_platform::settings::ScaleMode;

/// 窗口物理像素（逻辑竖屏半采样，与 macroquad native 壳同规格；
/// 飞轮 R1 ③-18：经 `LOGICAL_W/H` 派生，不裸写 540/960）。
const WIN_W: u32 = LOGICAL_W / 2;
const WIN_H: u32 = LOGICAL_H / 2;

/// S5.2 同源固定种子（= `tests/scripts/full_run.json seed`；飞轮 R1 ⑥-11：壳内 2 处合一命名
/// 常量，doccheck2 `check_render_single_source` 对账 JSON ↔ 本常量，改 seed 忘改另一处 = 出图/自检静默不同源）。
const SELFTEST_SEED: u32 = 42;

/// 逻辑画布（`ti_render::framebuffer` 单一事实源；飞轮 R1 ⑥-05：引擎侧改派生）。
const LOGICAL_W: u32 = ti_render::framebuffer::LOGICAL_W_I32 as u32;
const LOGICAL_H: u32 = ti_render::framebuffer::LOGICAL_H_I32 as u32;

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

/// S18.7：离屏截图目标（`--shot` 模式；验证 Bevy 原生渲染真实出画）。
#[derive(Resource, Clone)]
struct ShotTarget(Handle<Image>);

/// S18.7：截图读回结果（RGBA8 像素）。
static SHOT_READBACK: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

/// S18.7：截图请求标记（只发一次）。
#[derive(Resource, Default)]
struct ShotDone(bool);

/// S18.7：截图读回 observer（`Screenshot::image` → 像素）。
#[allow(clippy::needless_pass_by_value)]
fn store_shot(captured: On<bevy::render::view::screenshot::ScreenshotCaptured>) {
    let raw: Vec<u8> =
        captured.image.data.as_ref().map_or_else(Vec::new, |d| d.as_slice().to_vec());
    if let Ok(mut g) = SHOT_READBACK.lock() {
        *g = Some(raw);
    }
}

/// S18.7：请求一次离屏截图（帧数 ≥3 后）。
#[allow(clippy::needless_pass_by_value)]
fn request_shot(
    mut shot: ResMut<ShotDone>,
    target: Res<ShotTarget>,
    frames: Res<bevy::diagnostic::FrameCount>,
    mut commands: Commands,
) {
    // S18.8b：贴图资产是**异步 IO**（资产服务器读盘）→ 截图帧阈值从 12 提到 40，
    // 否则截到的是「贴图未加载」的半成品帧。
    if shot.0 || frames.0 < 40 {
        return;
    }
    shot.0 = true;
    commands.spawn(bevy::render::view::screenshot::Screenshot::image(target.0.clone()));
}

/// S18.7：读回就绪 → 退出（`--shot` 模式）。
///
/// S18.8b：截图有**竞态** —— 目标纹理尚未渲染出内容就被截下时，读回是全透明的空帧，
/// 而旧逻辑只在「非空」时退出 → 循环跑完仍写出一张空 PNG（证据静默失真）。
/// 现改为：空帧 = **允许重截**（`ShotDone` 复位 → 下一帧重新请求），非空才退出。
fn finish_shot(
    mut exits: MessageWriter<AppExit>,
    mut shot: ResMut<ShotDone>,
    device: Option<Res<bevy::render::renderer::RenderDevice>>,
) {
    if let Some(device) = device {
        let _ = device.wgpu_device().poll(bevy::render::render_resource::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
    let Some(raw) = SHOT_READBACK.lock().ok().and_then(|g| g.clone()) else { return };
    if raw.chunks(4).any(|c| c[0] > 8 || c[1] > 8 || c[2] > 8) {
        exits.write(AppExit::Success);
    } else {
        shot.0 = false;
    }
}

/// S18.7：**Bevy 原生 index 场景**实体（`scene_render::IndexScene`）。
///
/// Bevy = 渲染引擎：index 场景直接 spawn Sprite/Text2d 实体，**不产生帧缓冲、
/// 不上传整帧纹理**；其余场景在 S18.7 迁完前仍走 CPU 纹理（过渡期）。
#[derive(Resource, Default)]
struct NativeIndex {
    /// 上帧列表**内容哈希**（`None` = 尚未建立）。
    ///
    /// S18.7a：列表每帧重录，实体只在**内容变化**时重建
    /// → 选择态变化立即生效，未变帧零实体开销。
    content_hash: Option<u64>,
}

/// 帧纹理全屏精灵标记（S18.5 相机 zoom/pan 接线：`camera_system` 经此实体驱动
/// 纹理位移 / 正交 scale；与相机实体区分 = `Without<Camera2d>`）。
#[derive(Component)]
struct FrameSprite;

/// letterbox 基础 scale（窗口物理 → 1080 逻辑；与 S18.4 `spawn_stage` 同式）。
#[allow(clippy::cast_precision_loss)]
fn letterbox() -> f32 {
    letterbox_size(WIN_W as f32, WIN_H as f32)
}

/// 任意窗口物理尺寸的 letterbox scale（`min(w/1080, h/1920)`；2026-10-26 窗口缩放跟随）。
#[allow(clippy::cast_precision_loss)]
fn letterbox_size(window_w: f32, window_h: f32) -> f32 {
    let w = window_w / LOGICAL_W as f32;
    let h = window_h / LOGICAL_H as f32;
    if w < h { w } else { h }
}

/// Startup：相机（正交 scale=0.5 letterbox 满铺）+ 全屏底色精灵。
///
/// S18.8a：`S18.7a` 起 7 场景全走 `DrawList` → Bevy 实体（不再有「CPU 帧缓冲 → 整帧纹理上传」路径），
/// 故占位纹理 + `FrameTex` 资源随之退役（S18.8a 门禁 wasm clippy 覆盖到本壳，死代码清零）。
fn spawn_stage(mut commands: Commands) {
    let scale = letterbox();
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
    // 飞轮 R1 ①-13：底色走 `Theme::DEFAULT.parch_dark`（§5.1 权威口径：运行时 theme ≡
    // DEFAULT；CPU 侧同位 = 帧缓冲清屏色），双壳底色同源不漂移。
    commands.spawn((
        Sprite::from_color(
            ti_engine::canvas::rgb565_to_color(ti_render::theme::Theme::DEFAULT.parch_dark),
            Vec2::from([LOGICAL_W as f32, LOGICAL_H as f32]),
        ),
        // S18.7：CPU 纹理层置于最底（z=-13；原生实体 z∈[-12,-1] 全在其上）。
        Transform::from_xyz(0.0, 0.0, -13.0),
        FrameSprite,
    ));
}

/// 下一缩放模式（循环，`ScaleMode::ALL` 顺序；与 macroquad 壳同函数口径）。
fn next_scale_mode(cur: ScaleMode) -> ScaleMode {
    let all = ScaleMode::ALL;
    let i = all.iter().position(|&m| m == cur).unwrap_or(0);
    all[(i + 1) % all.len()]
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)] // 系统参数天然 8 个（+2026-10-26 窗口缩放跟随加 `Query<&Window>`）
/// 输入系统：键盘沿 + F1–F4 + 指针（触屏优先/鼠标兜底）→ `GameApp` 逻辑。
fn input_system(
    store: ResMut<GameStore>,
    mut mouse_pos: ResMut<MousePos>,
    mut acc: ResMut<AccumulatedMouseMotion>,
    kb: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut state: Local<InputState>,
    window: Query<&Window>,
) {
    // S18.5：滚轮 zoom + 左键拖拽 pan（仅主地图；逻辑 px = 物理 px / 当前 `vp.scale()`）。
    mouse_pos.x += acc.delta.x;
    mouse_pos.y += acc.delta.y;
    let drag = (mouse.pressed(MouseButton::Left), acc.delta);
    acc.delta = bevy::prelude::Vec2::ZERO;
    // 2026-10-26 窗口缩放跟随：F1 重算 `vp` 用**当前**窗口**物理**尺寸（壳坐标系 =
    // 物理 px：鼠标/触摸 delta 与 `WindowResolution` 均物理；`win.size()` 是逻辑 px，勿用）。
    let window_size = match window.single() {
        Ok(win) => Vec2::new(win.physical_width() as f32, win.physical_height() as f32),
        Err(_) => Vec2::new(WIN_W as f32, WIN_H as f32),
    };

    let Ok(mut g) = store.0.lock() else {
        return;
    };
    // S18.5 相机（world_canvas 镜头）：主地图场景才允许 zoom/pan（其它场景相机冻结）。
    let camera_live = g.scene() == Scene::MainMap;
    if camera_live {
        // 滚轮 0.20 event 接线挂 S18.8（winit MotionEvent 面）；键盘 `=`/`-` 同构 zoom（view 中心锚点）。
        if kb.just_pressed(KeyCode::Equal) {
            g.camera_zoom_at(
                [LOGICAL_W as f32 / 2.0, LOGICAL_H as f32 / 2.0],
                1.25,
                [LOGICAL_W as f32, LOGICAL_H as f32],
            );
        }
        if kb.just_pressed(KeyCode::Minus) {
            g.camera_zoom_at(
                [LOGICAL_W as f32 / 2.0, LOGICAL_H as f32 / 2.0],
                0.8,
                [LOGICAL_W as f32, LOGICAL_H as f32],
            );
        }
        if drag.0 {
            // 窗口缩放跟随：物理 px → 逻辑 px 除**当前** `vp.scale()`（窗口尺寸变化时
            // `window_resize_system` 已重建 `vp`；固定 540×960 壳 ≡ 旧 `letterbox()` 0.5）。
            let vp_scale = g.shell().vp.scale();
            g.camera_pan(drag.1.x / vp_scale, drag.1.y / vp_scale);
        }
    }
    // 键盘「按住」快照（KeyCode → 语义键，与 macroquad 壳同表）。
    let raw = KeyState {
        up: kb.pressed(KeyCode::ArrowUp),
        down: kb.pressed(KeyCode::ArrowDown),
        left: kb.pressed(KeyCode::ArrowLeft),
        right: kb.pressed(KeyCode::ArrowRight),
        confirm: kb.pressed(KeyCode::Enter) || kb.pressed(KeyCode::Space),
        back: kb.pressed(KeyCode::Backspace) || kb.pressed(KeyCode::Escape),
        landmark: kb.pressed(KeyCode::KeyE),
        spin: kb.pressed(KeyCode::Space),
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
        spin: key_edge(raw.spin, state.prev_keys.spin),
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
/// 窗口缩放跟随系统（用户 2026-10-26 报告：桌面版不随窗口缩放）：`resizable` 窗口
/// 尺寸变化时按当前物理尺寸重建 shell `vp`（屏幕 px → 1080 逻辑反变换），鼠标/触摸
/// 命中判定自动跟上；相机正交 scale 由 `camera_system` 读 `vp.scale()` 驱动
/// （默认 Smooth = `min(w/1080, h/1920)` letterbox，内容恒完整可见 + 居中）。
fn window_resize_system(
    store: ResMut<GameStore>,
    mut last_wh: Local<(f32, f32)>,
    window: Query<&Window>,
) {
    let Ok(win) = window.single() else {
        return;
    };
    // 壳的屏幕坐标约定 = 窗口**物理**像素（鼠标/winit delta、`WindowResolution` 均物理）；
    // `win.size()` 是逻辑像素（高 DPI 下 ≠ 物理），必须用物理访问器。
    let wh = (win.physical_width() as f32, win.physical_height() as f32);
    if *last_wh == wh {
        return;
    }
    *last_wh = wh;
    let Ok(mut g) = store.0.lock() else {
        return;
    };
    let mode = g.shell().scale_mode();
    g.shell_mut().set_scale_mode(mode, wh.0, wh.1);
}

#[allow(clippy::needless_pass_by_value)]
/// S18.5 相机系统：`WorldCam`（zoom/pan，`game_app::camera_*`）→ Bevy 正交 scale +
/// 帧纹理精灵位移（`world_canvas` 镜头）。默认 cam=(540,960)/zoom=1 时 = S18.4 IDENTITY 零差。
///
/// 2026-10-26（用户报告「桌面版不随窗口缩放」）：正交 scale 改读 **shell `vp.scale()`**
/// （`window_resize_system` 随窗口尺寸变化重建；默认 Smooth = `min(w/1080, h/1920)`
/// letterbox，内容恒完整可见 + 居中），与鼠标/触摸输入同一反变换源，画/摸不错位。
fn camera_system(
    store: Res<GameStore>,
    mut cameras: Query<&mut Projection, With<Camera2d>>,
    mut sprite: Query<&mut Transform, (With<FrameSprite>, Without<Camera2d>)>,
) {
    let Ok(g) = store.0.lock() else {
        return;
    };
    let cam = *g.world_cam();
    let scale = g.shell().vp.scale() * cam.zoom;
    if let Ok(mut proj) = cameras.single_mut() {
        if let Projection::Orthographic(o) = &mut *proj {
            o.scale = scale;
        }
    }
    if let Ok(mut st) = sprite.single_mut() {
        st.translation.x = cam.cam[0] - LOGICAL_W as f32 / 2.0;
        st.translation.y = LOGICAL_H as f32 / 2.0 - cam.cam[1];
    }
}

#[allow(clippy::needless_pass_by_value)]
/// 渲染系统：GameApp 帧推进 + 静止跳渲染 + Framebuf → RGBA8 纹理。
fn render_system(store: ResMut<GameStore>, time: Res<Time>, mut exits: MessageWriter<AppExit>) {
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
    // S18.7a: live path = **draw list** (recorded) -> Bevy entities (`native_index_system` executes).
    // No framebuffer / no full-frame texture upload (all 7 scenes go through Bevy).
    g.render();
    g.shell_mut().last_rendered = Some(g.menu_selected());
    g.shell_mut().last_rendered_scene = Some(scene_key);
    g.clear_last_sfx();
}

/// S18.7：Bevy 原生 index 场景维护（**独占系统**：需 `&mut World` 建/销实体）。
///
/// - 首次进入 index：按 `scenes.json` 的 `index.primitives` spawn Sprite/Text2d 实体；
/// - 选择态变化：换卡片底色 / 文字色；
/// - 离开 index：despawn 原生实体（其余场景走 CPU 纹理过渡路径）。
fn native_index_system(world: &mut World) {
    let Some(store) = world.get_resource::<GameStore>() else {
        return;
    };
    let Ok(g) = store.0.lock() else {
        return;
    };
    let list = g.draw_list().clone();
    drop(g);

    // S18.7a: 画制列表 -> Bevy 实体（**覆盖全 7 场景**：
    // 场景代码不变，绘制目标可插拒 = CPU 帧缓冲 / draw list）。
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&list.ops, &mut hasher);
    let content_hash = std::hash::Hasher::finish(&hasher);

    let mut native = world.remove_resource::<NativeIndex>().unwrap_or_default();
    if native.content_hash == Some(content_hash) {
        world.insert_resource(native);
        return;
    }
    // 先按标记清理上帧实体（否则切场景留鬼影）。
    let stale: Vec<Entity> = world
        .query_filtered::<Entity, With<ti_engine::scene_render::DrawListEntity>>()
        .iter(world)
        .collect();
    for e in stale {
        world.despawn(e);
    }
    let font = world
        .get_resource::<AssetServer>()
        .map(|server| server.load::<Font>("assets/fonts/lxgw-wenkai-reg.ttf"));
    ti_engine::scene_render::spawn_draw_list(world, font, &list);
    native.content_hash = Some(content_hash);
    world.insert_resource(native);
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
    // 飞轮 R1 ⑦-11：legacy `.Map` 场景注入循环已删（`scene_cache` 零读方死字段；
    // legacy `.Map` 解析归 `legacy/` 参照，契约 1.3）。
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
    ("spin", 1),
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
    match action {
        "confirm" => GameInput::Confirm,
        "next" => GameInput::Next,
        "move_down" => GameInput::MoveDown,
        "back" => GameInput::Back,
        "spin" => GameInput::Spin,
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
/// 仓库根（资产根 / 证据输出路径）。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn main() {
    let selftest = std::env::args().any(|a| a == "--selftest");
    let shot = std::env::args().any(|a| a == "--shot");
    // S18.7b：`--scene=<id>`（index/select_role/select_map/main_map/vs/message/saveload）——
    // Bevy 壳经 `ti_app::forward_nav::navigate` 导航到目标场景，其 DrawList 由场景代码
    // 渲染进同一 `DrawList`（静态原语 + 运行时内容），再实体化为 Bevy 精灵。
    let scene_id = std::env::args()
        .skip(1)
        .find_map(|a| a.strip_prefix("--scene=").map(str::to_owned))
        .unwrap_or_else(|| "index".to_owned());
    // S19.7 D5：出图带状态（`--spin=N`）—— 摇 N 次骰后再截图 ⇒ 落格提示条 / 航程条可见
    // （静态出图不含运行时状态，提示条此前无出图证据）。
    let spin_n = std::env::args()
        .skip(1)
        .find_map(|a| a.strip_prefix("--spin=").and_then(|v| v.parse::<u32>().ok()))
        .unwrap_or(0);
    let mut game = GameApp::with_upload_mode(WIN_W as f32, WIN_H as f32, UploadMode::Partial);
    load_resources(&mut game);
    if scene_id != "index" {
        ti_app::forward_nav::navigate(&mut game, &scene_id);
    }
    if spin_n > 0 {
        // 固定种子（同 S18.4 selftest 口径 [`SELFTEST_SEED`]）⇒ 出图可复现（否则每次落点不同，
        // 可能落在无事件的普通格 ⇒ 提示条时有时无，验收证据不稳定）。
        game.set_rng_seed(SELFTEST_SEED);
        for _ in 0..spin_n {
            game.handle(ti_app::GameInput::Spin);
        }
        // S19.7 D5：提示条按帧衰减，而离屏截图的实际捕获帧受贴图异步加载影响（≫120 帧）
        // ⇒ 出图里提示条已过期。这里顶住衰减，保证出图含提示条（验收证据）。
        game.hold_board_toast();
    }
    if !selftest {
        // S18.7a: live path = draw list (Bevy entities); the scene code is unchanged.
        game.set_record_draw(true);
    }
    // 双渲染期无 bevy 音频：曲目数 0（S18.8 切 bevy_audio 后恢复）。
    game.shell_mut().set_audio(0, 0);
    if selftest {
        // S5.2 同源固定种子（full_run.json seed = [`SELFTEST_SEED`]），e2e 可复现。
        game.set_rng_seed(SELFTEST_SEED);
    }

    let window = Window {
        resolution: WindowResolution::new(WIN_W, WIN_H),
        title: window_title(),
        // 2026-10-26（用户报告「桌面版不随窗口缩放」）：窗口可拖拽/最大化，
        // 内容 letterbox 跟随（`window_resize_system` 重建 `vp` + `camera_system` 读 `vp.scale()`）。
        resizable: true,
        ..Default::default()
    };
    // S18.7：**官方 `DefaultPlugins`**（不再手搓插件表）。
    //
    // 教训（EXP-101 待写）：手写插件表漏 `SpritePlugin`/`SpriteRenderPlugin`/
    // `CameraPlugin`/`CorePipelinePlugin`/`init_asset::<Shader>()` 等 → 实体在但**从未渲染**
    // （S18.1 spike 的矩形、S18.4 的 CPU 纹理都是黑的，验收只查 exit 0 没看画面）。
    // 正确用法 = 引擎官方插件组 + 按需 `.set(...)` 覆写。
    let mut app = App::new();
    let _ = app
        .add_plugins(
            bevy::DefaultPlugins
                .set(bevy::window::WindowPlugin {
                    // S18.7：`--shot` 离屏模式不建窗口（照 S18.3 example 口径）。
                    primary_window: if shot { None } else { Some(window) },
                    ..Default::default()
                })
                // 资产根 = 仓库根（字体 `assets/fonts/...`）。
                .set(bevy::asset::AssetPlugin {
                    file_path: repo_root().to_string_lossy().to_string(),
                    ..Default::default()
                })
                .set(bevy::render::RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..Default::default()
                })
                .set(bevy::log::LogPlugin { level: bevy::log::Level::WARN, ..Default::default() }),
        )
        .init_resource::<ClearColor>()
        .init_resource::<MousePos>()
        .init_resource::<NativeIndex>()
        .insert_resource(GameStore(std::sync::Mutex::new(game)));
    if selftest {
        app.insert_resource(SelfTest { step: 0, repeat: 0, failures: 0, done: false })
            .add_systems(Update, (selftest_system, render_system, stats_system).chain());
    } else {
        // S18.5：相机系统 = `WorldCam` → Bevy 正交 scale / 精灵位移（渲染前接线）。
        // 2026-10-26：`window_resize_system` 最先跑（窗口尺寸变 → 重建 `vp`），
        // 本帧 input/camera 即读到新 `vp`（画/摸同源不错位）。
        app.add_systems(
            Update,
            (window_resize_system, input_system, camera_system, render_system, stats_system)
                .chain(),
        );
        app.add_systems(Update, native_index_system.after(render_system));
    }
    app.add_systems(Startup, spawn_stage);
    if shot {
        run_shot(&mut app, &scene_id);
        return;
    }
    app.run();
}

/// S18.7：离屏截图模式（`--shot`）：渲染到离屏目标 → GPU 读回 → 写 `evidence/bevy_native_<scene>.png`。
///
/// S18.8b：贴图资产是**异步读盘**（Bevy 资产服务器）→ 每帧 sleep 4ms 让 IO 落地；
/// 空帧自动重截（`finish_shot` 复位 `ShotDone`）。
fn run_shot(app: &mut App, scene_id: &str) {
    // S18.7：离屏渲染目标（`--shot` 验证 Bevy 原生渲染真实出画）。
    let mut target_image =
        Image::new_target_texture(LOGICAL_W, LOGICAL_H, TextureFormat::Rgba8UnormSrgb, None);
    target_image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let target_handle = app.world_mut().resource_mut::<Assets<Image>>().add(target_image);
    app.insert_resource(ShotTarget(target_handle.clone()));
    app.world_mut().spawn((
        Camera2d,
        RenderTarget::Image(target_handle.into()),
        Transform::IDENTITY,
    ));
    app.init_resource::<ShotDone>()
        .add_observer(store_shot)
        .add_systems(Update, (request_shot, finish_shot).chain());
    // 手动 update 循环 + 设备轮询（照 S18.3 `bevy_text_align` example 口径：
    // 截图异步完成，需 poll 设备才能拿到结果）。
    //
    // ⚠️ `app.finish()` + `app.cleanup()` **不可省**：`finish()` 才会把渲染 app 的
    // extract 调度启动起来（`RenderAppChannels` 由此就位）；缺它 → extract 阶段
    // panic「resource does not exist: RenderAppChannels」。
    app.finish();
    app.cleanup();
    for _ in 0..60 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(4));
        if let Some(device) = app.world().get_resource::<bevy::render::renderer::RenderDevice>() {
            let _ = device.wgpu_device().poll(bevy::render::render_resource::PollType::Wait {
                submission_index: None,
                timeout: None,
            });
        }
        if SHOT_READBACK
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .is_some_and(|raw| raw.chunks(4).any(|c| c[0] > 8 || c[1] > 8 || c[2] > 8))
        {
            break;
        }
    }
    let empty = SHOT_READBACK
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .is_none_or(|raw| !raw.chunks(4).any(|c| c[0] > 8 || c[1] > 8 || c[2] > 8));
    if empty {
        eprintln!(
            "[shot] WARNING: readback still empty (target texture has no content) - evidence unusable"
        );
    }
    write_shot_png(&format!("evidence/bevy_native_{scene_id}.png"));
}

/// S18.7：把 GPU 读回像素写成 PNG（`--shot` 输出）。
fn write_shot_png(path: &str) {
    let Some(raw) = SHOT_READBACK.lock().ok().and_then(|g| g.clone()) else {
        eprintln!("[shot] 无读回像素（渲染目标未就绪？）");
        return;
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let out = root.join(path);
    if let Some(parent) = out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let nonblack = raw.chunks(4).filter(|c| c[0] > 8 || c[1] > 8 || c[2] > 8).count();
    let alpha_nonzero = raw.chunks(4).filter(|c| c[3] > 8).count();
    // 采样：左上角 1 点 + **真实屏幕中心**（旧版取的是第 960 行第 0 列
    // = 左边缘，容易被误读成「整屏被木框色铺满」；EXP-104）。
    let first: Vec<String> = raw.chunks(4).take(1).map(|c| format!("{c:?}")).collect();
    let center_idx = ((LOGICAL_H / 2) * LOGICAL_W + LOGICAL_W / 2) as usize;
    let midpx: Vec<String> =
        raw.chunks(4).skip(center_idx).take(1).map(|c| format!("{c:?}")).collect();
    eprintln!("[shot] 非黑像素 = {nonblack} / alpha>8 = {alpha_nonzero} / {}", raw.len() / 4);
    eprintln!("[shot] 左上角 = {first:?} 屏幕中心 = {midpx:?}");
    let png = ti_render::png::encode_png_rgba(&raw, LOGICAL_W, LOGICAL_H);
    match std::fs::write(&out, &png) {
        Ok(()) => eprintln!("[shot] 已写 {}", out.display()),
        Err(e) => eprintln!("[shot] 写文件失败: {e}"),
    }
}

/// 窗口标题（飞轮 R1 ⑤-04 i18n 面量归零）：主源 = `strings.json app.window_title`；
/// 缺表/缺键 = crash-safe 回退（同 S21.14 图标代码矢量回退口径）。
fn window_title() -> String {
    ti_core::data::pairs()
        .iter()
        .find(|pair| pair.0 == "app.window_title")
        .map_or_else(|| "金银岛 TreasureIsl（Bevy）".to_owned(), |pair| pair.1.to_string())
}
