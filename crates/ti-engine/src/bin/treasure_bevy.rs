//! S18.4 Bevy 呈现壳：`GameApp` 逻辑 + CPU `Framebuf` → Bevy 纹理呈现（双渲染期过渡形态）。
//!
//! 架构（`docs/process/11_bevy_migration.md` §4）：
//! - **逻辑**：完整 `GameApp`（场景流转/战斗/经济/存档；`render_to_framebuf` 纯 CPU，0 漂移）；
//! - **呈现**：Bevy 2D（`Camera2d` + `Projection::Orthographic(scale)`；`scale` = **世界单位/像素**
//!   = 1/(letterbox 视口比 × zoom)，见 `bevy_ortho_scale` 口径卡）letterbox 1080×1920
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

mod resizeprobe;
mod resizetest;

use resizeprobe::attach_probe;
use resizetest::attach_resizetest;

use std::path::{Path, PathBuf};

use bevy::asset::RenderAssetUsages;
use bevy::camera::{Camera2d, ClearColor, OrthographicProjection, Projection, RenderTarget};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::sprite::Sprite;
use bevy::text::Font;
use bevy::window::{Window, WindowResolution};
use ti_app::game_app::{GameApp, GameInput, HitData, KeyState, Scene};
// S18.8a：`parse_font_bytes` 仅 native 分支用（wasm 分支无 std::fs 资源读）→ 按目标编译。
#[cfg(not(target_arch = "wasm32"))]
use ti_app::menu_shell::parse_font_bytes;
use ti_app::scenes::{SceneKind, SelectionHost, SessionHost};
use ti_core::domain::messages::MsgKind;
use ti_core::rules::messages::page_text;
use ti_app::stats::FpsMeter;
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
///
/// S29.2（B1）：`touch`（`TouchPad` 手势 pad）退役 —— 命中/手势全走 Bevy 拾取层
/// （`pick_event_system` 消费 `PickEvent` → `poll_hit`）。
#[derive(Default)]
struct InputState {
    /// 上一帧「按住」按键快照（沿 = 当前 && !上一帧）。
    prev_keys: KeyState,
    /// 上一帧 F1–F4 状态（功能键沿触发）。
    prev_function_keys: [bool; 4],
}

/// 指针位置（**窗口逻辑像素**；2026-10-26 EXP-117：由窗口绝对光标位置 `Window::cursor_position()`
/// 覆盖，delta 累加仅作光标移出窗口时的回落）。
#[derive(Resource, Default)]
struct MousePos {
    /// 窗口逻辑像素 x（bevy 0.20 = 物理 ÷ `scale_factor`）。
    x: f32,
    /// 窗口逻辑像素 y。
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

/// S27.5：shot 模式资产预载状态（句柄清单 + 全部 Loaded 就绪旗）。
///
/// 根因（EXP-127 后续实测）：冷启动时截图帧阈值（40 帧）内资产服务器可能只加载了
/// **部分**贴图 ⇒ 截到「半成品帧」（部分资产上屏、部分缺席，既非纯色也非完整，
/// `is_blank_frame` 纯色判据拦不住）。治本 = `run_shot` 建装时先把 `assets/ui/` +
/// `assets/fonts/` 全量 `load_untyped`，本系统逐帧轮询 `LoadState`，全部 `Loaded`
/// （或个别 `Failed`，不死等，内容判据仍兜底）后 `ready = true`，`request_shot`
/// 才允许请求截图。
#[derive(Resource)]
struct ShotPreload {
    handles: Vec<bevy::asset::UntypedHandle>,
    ready: bool,
}

/// S27.5：逐帧轮询预载状态（`request_shot` 前置，见 [`ShotPreload`]）。
#[allow(clippy::needless_pass_by_value)]
fn shot_preload_poll(
    server: Res<bevy::asset::AssetServer>,
    mut preload: ResMut<ShotPreload>,
    mut tick: Local<usize>,
) {
    if preload.ready {
        return;
    }
    *tick += 1;
    let mut all_settled = true;
    let mut failed = 0usize;
    let mut hist: std::collections::BTreeMap<&'static str, usize> = std::collections::BTreeMap::new();
    for h in &preload.handles {
        let name = match server.load_state(h) {
            bevy::asset::LoadState::Loaded => "Loaded",
            bevy::asset::LoadState::Failed(_) => {
                failed += 1;
                "Failed"
            }
            bevy::asset::LoadState::Loading => "Loading",
            bevy::asset::LoadState::NotLoaded => "NotLoaded",
        };
        *hist.entry(name).or_insert(0) += 1;
        if name != "Loaded" && name != "Failed" {
            all_settled = false;
        }
    }
    if *tick % 100 == 0 {
        eprintln!("[shot] preload tick={} hist={hist:?}", *tick);
    }
    if all_settled {
        if failed > 0 {
            eprintln!("[shot] WARN: {failed} 个预载资产 Failed（不死等，内容判据仍兜底）");
        }
        eprintln!("[shot] preload ready at tick={}", *tick);
        preload.ready = true;
    }
}

/// S18.7：请求一次离屏截图（帧数 ≥3 后）。
#[allow(clippy::needless_pass_by_value)]
fn request_shot(
    mut shot: ResMut<ShotDone>,
    target: Res<ShotTarget>,
    frames: Res<bevy::diagnostic::FrameCount>,
    preload: Res<ShotPreload>,
    mut commands: Commands,
) {
    // S18.8b：贴图资产是**异步 IO**（资产服务器读盘）→ 截图帧阈值从 12 提到 40，
    // 否则截到的是「贴图未加载」的半成品帧。
    if shot.0 || frames.0 < 40 {
        return;
    }
    // S27.5：预载未就绪 = 不截（冷启动部分资产已上屏、部分缺席 = 「半成品帧」，
    // 纯色判据拦不住；治本 = 等 `assets/ui/` + `assets/fonts/` 全部 Loaded）。
    if !preload.ready {
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
///
/// S27.5（用户报「没有筛子」排查中实测）：**非空 ≠ 有内容** —— 冷启动（首次 build /
/// 冷 OS 缓存）时 wgpu 管线与贴图未就绪，截到的是**相机 clear 色全幅帧**（如
/// `(43,44,47)`），按旧判据「任一像素 >8」即算非空 → 静默写出全灰空图（`main_map`
/// 连续两次；热缓存后同命令自愈）。判据升级为「非空 **且非纯色帧**」：单一 RGB 颜色
/// 占比 ≥99% 视为未就绪，重截直到真内容（600 帧兜底防死循环）。
#[allow(clippy::needless_pass_by_value)]
fn finish_shot(
    mut exits: MessageWriter<AppExit>,
    mut shot: ResMut<ShotDone>,
    frames: Res<bevy::diagnostic::FrameCount>,
    device: Option<Res<bevy::render::renderer::RenderDevice>>,
) {
    // 兜底（S27.5）：600 帧（~10s）仍无内容帧 = 资产/渲染异常，强制退出并告警
    // （证据可能失真——但死循环更不可接受；正常场景数帧内即就绪）。
    if frames.0 > 600 {
        eprintln!("[shot] WARN: 600 帧内未截到内容帧（贴图/管线未就绪？），强制退出");
        exits.write(AppExit::Success);
        return;
    }
    if let Some(device) = device {
        let _ = device.wgpu_device().poll(bevy::render::render_resource::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
    let Some(raw) = SHOT_READBACK.lock().ok().and_then(|g| g.clone()) else { return };
    let has_content = raw.chunks(4).any(|c| c[0] > 8 || c[1] > 8 || c[2] > 8)
        && !is_blank_frame(&raw);
    if has_content {
        exits.write(AppExit::Success);
    } else {
        shot.0 = false;
    }
}

/// S27.5：纯色帧判定（单一 RGB 颜色占比 ≥99% = 相机 clear 色未就绪帧）。
fn is_blank_frame(raw: &[u8]) -> bool {
    let mut counts: std::collections::HashMap<[u8; 3], u32> = std::collections::HashMap::new();
    let mut total: u32 = 0;
    for c in raw.chunks_exact(4) {
        *counts.entry([c[0], c[1], c[2]]).or_insert(0) += 1;
        total += 1;
    }
    counts.values().copied().max().is_some_and(|max| max * 100 >= total * 99)
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

/// **游戏世界相机**标记（S29.5）。
///
/// live 路径有 3 台 `Camera2d`：游戏正交（`spawn_stage`）/ `bevy_ui` UI 相机（`IsDefaultUiCamera`）
/// / 离屏截图相机（`--shot` 的 `RenderTarget::Image`，其正交 scale 须按目标尺寸 contain 固定）。
/// 三者用「相机类型」区分不可靠（`single_mut()` 多实体时**静默失败** ⇒ S29.1 起
/// `camera_system` 不再更新世界相机正交 scale ⇒ 窗口缩放跟随失效）⇒ 用显式标记锁定世界相机。
#[derive(Component)]
struct WorldCam;

/// letterbox 基础 scale（shell 视口比 = **像素/逻辑单位**；与 S18.4 `spawn_stage` 同式）。
#[allow(clippy::cast_precision_loss)]
fn letterbox() -> f32 {
    letterbox_size(WIN_W as f32, WIN_H as f32)
}

/// 任意窗口物理尺寸的 letterbox 视口比（`min(w/1080, h/1920)` = 像素/逻辑单位；
/// 2026-10-26 窗口缩放跟随）。
///
/// ⚠️ 口径：这是 **shell 视口比**（`Viewport::fit` 同值），**不是** Bevy 正交 `scale`；
/// Bevy 侧取其倒数（见 [`bevy_ortho_scale`]）。
#[allow(clippy::cast_precision_loss)]
fn letterbox_size(window_w: f32, window_h: f32) -> f32 {
    let w = window_w / LOGICAL_W as f32;
    let h = window_h / LOGICAL_H as f32;
    if w < h { w } else { h }
}

/// Bevy 2D 正交 `scale`（**世界单位/像素**）= letterbox 视口比与 `WorldCam` zoom 的倒数积。
///
/// 口径依据（bevy 0.20 实测，勿改）：
/// - `OrthographicProjection.scale` = 世界单位/像素；可见世界区域 = `scale × 视口 px`
///   （`OrthographicProjection.area` 为 pub 字段，运行时可直接读出实际值）；
/// - shell 视口比 `vp.scale()` = 像素/逻辑单位 ⇒ 二者互为倒数；
/// - `WorldCam.zoom` 大 = 放大（可见世界收缩）⇒ 与 `scale` 反比。
///
/// ⇒ `o.scale = 1 / (px_per_unit × zoom)`。
///
/// 反例留档（EXP-115）：2026-10-26 曾误取 `o.scale = vp.scale()`（540×960 窗口下
/// `area=270×480` = 画布中央 1/4、4× 放大；拖大窗口时内容反而变小 ⇒「画面不跟着缩放」）。
fn bevy_ortho_scale(px_per_unit: f32, zoom: f32) -> f32 {
    1.0 / (px_per_unit * zoom)
}

/// Startup：相机（正交 scale = letterbox 视口比的**倒数** = 世界单位/像素）+ 全屏底色精灵。
///
/// S18.8a：`S18.7a` 起 7 场景全走 `DrawList` → Bevy 实体（不再有「CPU 帧缓冲 → 整帧纹理上传」路径），
/// 故占位纹理 + `FrameTex` 资源随之退役（S18.8a 门禁 wasm clippy 覆盖到本壳，死代码清零）。
fn spawn_stage(mut commands: Commands) {
    // 540×960 窗口 ⇒ 视口比 0.5 px/逻辑单位 ⇒ Bevy 正交 scale = 1/0.5 = 2.0（世界单位/像素）。
    let scale = bevy_ortho_scale(letterbox(), 1.0);
    commands.spawn((
        Camera2d,
        WorldCam,
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

/// S29.1（B1）：`bevy_ui` UI 相机 + 最小证明面板（叠在 1080 游戏世界之上；证明 UI 相机分层
/// + `bevy_ui` 渲染 + resize 自动重排）。仅 live/`--shot` 路径注册，`--selftest` 不加（保像素对拍口径）。
fn spawn_ui_chrome(mut commands: Commands, shot: Option<Res<ShotTarget>>) {
    let cam = ti_engine::ui::spawn_ui_camera(&mut commands);
    // `--shot` 离屏模式：UI 相机绑到同一离屏目标（`RenderTarget::Image`）⇒ bevy_ui 叠层进截图。
    if let Some(st) = shot {
        commands.entity(cam).insert(RenderTarget::Image(st.0.clone().into()));
    }
    // S29.3：chrome 树由 `chrome_ui_system` 按场景建（此处只建 UI 相机）。
}

/// S29.1：`bevy_ui` 布局探针（headless 证据：自动布局是否真跑了、树是否建了）。
///
/// `--shot` 离屏模式跑 5 帧后打一行日志：`Node`/`Text` 实体数 + 绝对定位面板的 `GlobalTransform`。
/// 证明 `bevy_ui` 管线（`Node`/`Text` 布局系统）在本壳生效，非仅编译通过。
#[derive(Resource, Default)]
struct UiProbe {
    frames: u32,
    logged: bool,
}

fn ui_probe_system(
    mut probe: ResMut<UiProbe>,
    nodes: Query<(&Node, Option<&GlobalTransform>)>,
    texts: Query<&Text>,
    computed: Query<&bevy::ui::ComputedNode>,
    cameras: Query<(&bevy::camera::Camera, &bevy::camera::Projection)>,
) {
    probe.frames += 1;
    if probe.logged || probe.frames < 5 {
        return;
    }
    probe.logged = true;
    let mut abs = 0usize;
    for (node, gt) in &nodes {
        if node.position_type == PositionType::Absolute {
            abs += 1;
            if let Some(transform) = gt {
                eprintln!(
                    "[ui] bevy_ui abs panel @ ({:.0},{:.0}) world",
                    transform.translation().x,
                    transform.translation().y
                );
            }
        }
    }
    let computed_count = computed.iter().count();
    eprintln!(
        "[ui] bevy_ui probe: node={} text={} abs={abs} computed_layout={computed_count} frame={}",
        nodes.iter().count(),
        texts.iter().count(),
        probe.frames
    );
    // S29.3：布局**尺寸**证据（`computed_layout` 只证明「算了布局」，不证明「尺寸非 0」）。
    // UI 在离屏目标不显示的最常见根因 = 布局尺寸为 0（`Val::Percent` 对着 0 视口解析）。
    let mut max_w = 0.0f32;
    let mut max_h = 0.0f32;
    for cn in &computed {
        max_w = max_w.max(cn.size.x);
        max_h = max_h.max(cn.size.y);
    }
    eprintln!("[ui] bevy_ui layout size: max_node={max_w:.0}x{max_h:.0} px");
    for (cam, proj) in &cameras {
        let vp = match proj {
            bevy::camera::Projection::Orthographic(o) => {
                format!("ortho scale={:.3}", o.scale)
            }
            _ => "other".to_owned(),
        };
        let target = cam
            .computed
            .target_info
            .as_ref()
            .map_or_else(|| "none".to_owned(), |t| format!("{:?}", t.physical_size));
        eprintln!(
            "[ui] camera order={} active={} viewport={:?} target={target} {vp}",
            cam.order, cam.is_active, cam.viewport,
        );
    }
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
    mut state: Local<InputState>,
    window: Query<&Window>,
) {
    // S18.5：滚轮 zoom + 左键拖拽 pan（仅主地图；逻辑 px / 当前 `vp.scale()` = 逻辑画布单位）。
    // 2026-10-26（用户报「首页每个按键点击都没反应」，EXP-117）：指针位置改取窗口**绝对**
    // 光标位置 `Window::cursor_position()`（bevy 0.20 = **逻辑**像素，winit 物理 ÷ scale_factor）。
    // 旧实现只累加 `AccumulatedMouseMotion` 的 delta ⇒ 位置 = 「光标相对**启动时**位置的位移」
    // 而非绝对位置 ⇒ 光标不在窗口左上角启动时全部命中偏移（首页 6 徽章一个都点不动）。
    // delta 累加仅作光标移出窗口时的回落（此时 `cursor_position()` = None）。
    mouse_pos.x += acc.delta.x;
    mouse_pos.y += acc.delta.y;
    if let Some(c) = window
        .single()
        .ok()
        .and_then(bevy::window::Window::cursor_position)
    {
        mouse_pos.x = c.x;
        mouse_pos.y = c.y;
    }
    let drag = (mouse.pressed(MouseButton::Left), acc.delta);
    acc.delta = bevy::prelude::Vec2::ZERO;
    // 2026-10-26 窗口缩放跟随：F1 重算 `vp` 用**当前**窗口**逻辑**尺寸。
    // 坐标空间口径（bevy 0.20 实测）：相机投影 = 逻辑视口（`camera.rs: logical_viewport_size`），
    // 鼠标/触摸 = 逻辑像素（`bevy_winit: 物理坐标 ÷ scale_factor`）⇒ `vp` 必须用逻辑尺寸，
    // 任何 DPI 下输入/渲染同源；`physical_*` 只在 scale_factor≠1 时与逻辑不等，勿用。
    let window_size = match window.single() {
        Ok(win) => Vec2::new(win.width(), win.height()),
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
}

/// S29.2：场景 → `scenes.json` 键（读该场景 `primitives` 的 `pick_key` 挂拾取标记；
/// `Intro`/`Exit` 无场景数据 = 空串 = 不挂拾取层）。
fn scene_json_key(scene: Scene) -> &'static str {
    match scene {
        Scene::Index => "index",
        Scene::SelectRole => "select_role",
        Scene::SelectMap => "select_map",
        Scene::MainMap => "main_map",
        Scene::Vs => "vs",
        Scene::Message => "message",
        Scene::Saveload => "saveload",
        Scene::Intro | Scene::Exit => "",
    }
}

/// S29.3（B1）：per-scene `bevy_ui` chrome 载体（UI chrome 迁 `bevy_ui` 后的生命周期）。
///
/// 换场景 / 换选中项 → 拆旧树 + 重建（bevy 0.20 `despawn` 递归 despawn `ChildOf` 子树）。
#[derive(Resource, Default)]
struct ChromeUi {
    /// 已建 chrome 根实体（`None` = 本场景无 `bevy_ui` chrome）。
    root: Option<Entity>,
    /// 已建 chrome 的（场景判别, 选中下标）——变了才重建（避免每帧重建）。
    built: Option<(i32, usize)>,
}

/// S29.3（B1）：按当前场景建 `bevy_ui` chrome（**自动布局**：行位置/高度由 taffy 算，
/// 不手写像素行；面板宽 `%` ⇒ resize 真重排）。
///
/// 意图数据在壳侧组装（`ti-core` 域 + `strings.json` + 资产路径），
/// `ti-engine::ui` 侧保持零游戏语义（只认「行 = 图 + 两行字 + 选中态」）。
/// S29.3：按当前场景建 `bevy_ui` chrome（**自动布局**：行位置/高度由 taffy 算，
/// 不手写像素行；面板宽 `%` ⇒ resize 真重排）。意图数据在壳侧组装
/// （`ti-core` 域 + `strings`/i18n + 资产路径），`ti-engine::ui` 侧保持零游戏语义。
fn build_scene_chrome(
    commands: &mut Commands,
    server: &AssetServer,
    font: &Handle<Font>,
    scene: SceneKind,
    g: &GameApp,
) -> Option<Entity> {
    match scene {
        SceneKind::SelectRole => build_select_role_chrome(commands, server, font, g),
        SceneKind::Saveload => Some(build_saveload_chrome(commands, font, g)),
        SceneKind::SelectMap => Some(build_select_map_chrome(commands, font, g)),
        SceneKind::Index => build_index_chrome(commands, server, font, g),
        SceneKind::Message => Some(build_message_chrome(commands, font, g)),
        // vs / main_map / Intro / Exit = **游戏世界场景 / 无场景数据**（chrome 与 world
        // 层叠：vs HP 填充 / main_map 棋盘 + 实体 + toast + HUD 数值 在 1080 层；bevy_ui
        // UI 相机叠在 1080 之上 ⇒ 把 chrome 迁 bevy_ui 会**遮住** 1080 动态内容）。
        // B1「游戏世界保持 1080」⇒ 这两场景 chrome 仍 1080 画。
        // S27.5（用户 2026-10-28 报「红圈 = S29·bevy_ui 探针面板碍事」）：S29.1 证明面板
        // 退役 —— 未迁场景 = 该场景 0 个 bevy_ui 节点（`None`），不再挂占位面板。
        // resize 验证口径改走 `--resizetest`（`ComputedNode` 第 3 不变量，S29.5）+
        // `--resizeprobe` 活体探针，正常玩法画面 0 干扰。
        _ => None,
    }
}

/// 首页 6 徽章径向环（**参数化**：环心 + 半径 + 60° 步进算 6 格中心 `%`，不逐格手写像素）。
/// 每格 = `medallion_ring.png` 底 + 白蒙版图标（`gold`/选中 `text_on_wood` 跨层 tint）+ 文案，
/// 挂 `PickTarget` key=1 index=0..5（**UI 相机层**命中；1080 层该场景徽章原语已标 `ui_layer` 跳过）。
#[allow(clippy::too_many_lines)]
fn build_index_chrome(
    commands: &mut Commands,
    server: &AssetServer,
    font: &Handle<Font>,
    g: &GameApp,
) -> Option<Entity> {
    const ICONS: [&str; 6] = ["compass", "chest", "anchor", "gear", "book", "door"];
    // 徽章底图 = 读 index 场景数据（首个 `pick_key=1` 原语 `params.asset`，单一事实源，壳侧不硬编码资产路径）
    let medallion_asset = ti_core::data::scenes()
        .ok()
        .and_then(|scenes| scenes.get("index"))
        .and_then(|sc| {
            sc.primitives
                .iter()
                .find(|p| p.params.get("pick_key").and_then(serde_json::Value::as_u64) == Some(1))
        })
        .and_then(|p| p.params.get("asset").and_then(serde_json::Value::as_str))
        .map(str::to_owned)?;
    let root = commands
        .spawn((
            bevy::ui::Node {
                width: bevy::ui::Val::Percent(100.0),
                height: bevy::ui::Val::Percent(100.0),
                display: bevy::ui::Display::Flex,
                position_type: bevy::ui::PositionType::Relative,
                ..Default::default()
            },
            bevy::picking::Pickable::IGNORE,
        ))
        .id();
    let selected = g.menu_selected();
    let theme = ti_render::theme::Theme::DEFAULT;
    let gold = ti_engine::canvas::rgb565_to_color(theme.gold);
    let on_wood = ti_engine::canvas::rgb565_to_color(theme.text_on_wood);
    // 环参数（1080 单源：画布尺寸走 `LOGICAL_*` 常量，不裸写 1080/1920/540）：
    // 环心 (画布半宽, 970)、半径 400、6 格 60° 步进（0 = 顶）。
    let cw = ti_render::framebuffer::LOGICAL_W_I32 as f32;
    let ch = ti_render::framebuffer::LOGICAL_H_I32 as f32;
    let cx = cw / 2.0 / cw * 100.0; // 环心 x = 屏心（50%）
    let cy = 970.0 / ch * 100.0; // 环心 y
    let rx = 400.0 / cw * 100.0; // 环半径 x（`%` 宽）
    let ry = 400.0 / ch * 100.0; // 环半径 y（`%` 高）
    let cell_w = 144.0 / cw * 100.0;
    let cell_h = 144.0 / ch * 100.0;
    let mut cell_entities = Vec::with_capacity(6);
    for (i, icon_name) in (0..6usize).zip(ICONS.iter()) {
        let deg = -90.0f32 + i as f32 * 60.0;
        let angle = deg * std::f32::consts::PI / 180.0;
        let x_pct = cx + rx * angle.cos();
        let y_pct = cy + ry * angle.sin();
        let is_sel = i == selected;
        let tint = if is_sel { on_wood } else { gold };
        // 格节点（绝对定位，中心 = 环心 + 半径向量）
        let cell = commands
            .spawn((
                bevy::ui::Node {
                    width: bevy::ui::Val::Percent(cell_w),
                    height: bevy::ui::Val::Percent(cell_h),
                    position_type: bevy::ui::PositionType::Absolute,
                    left: bevy::ui::Val::Percent(x_pct - cell_w / 2.0),
                    top: bevy::ui::Val::Percent(y_pct - cell_h / 2.0),
                    ..Default::default()
                },
                ChildOf(root),
            ))
            .id();
        // 徽章底图（铺满格）
        commands.spawn((
            bevy::ui::Node {
                width: bevy::ui::Val::Percent(100.0),
                height: bevy::ui::Val::Percent(100.0),
                position_type: bevy::ui::PositionType::Absolute,
                ..Default::default()
            },
            bevy::ui::prelude::ImageNode {
                image: server.load(medallion_asset.clone()),
                color: bevy::color::Color::WHITE,
                image_mode: bevy::ui::prelude::NodeImageMode::Stretch,
                ..Default::default()
            },
            bevy::picking::Pickable::IGNORE,
            ChildOf(cell),
        ));
        // 图标（格中心 50%×50%，白蒙版 × 跨层 tint）
        commands.spawn((
            bevy::ui::Node {
                width: bevy::ui::Val::Percent(50.0),
                height: bevy::ui::Val::Percent(50.0),
                position_type: bevy::ui::PositionType::Absolute,
                left: bevy::ui::Val::Percent(25.0),
                top: bevy::ui::Val::Percent(25.0),
                ..Default::default()
            },
            bevy::ui::prelude::ImageNode {
                image: server.load(format!("assets/ui/icon_{icon_name}.png")),
                color: tint,
                image_mode: bevy::ui::prelude::NodeImageMode::Stretch,
                ..Default::default()
            },
            bevy::picking::Pickable::IGNORE,
            ChildOf(cell),
        ));
        // 文案（格下方居中）
        let label_key = format!("scene.index.medallion.{i}");
        let label = ti_app::i18n::sget(&label_key);
        commands.spawn((
            bevy::ui::Node {
                width: bevy::ui::Val::Percent(cell_w),
                height: bevy::ui::Val::Auto,
                position_type: bevy::ui::PositionType::Absolute,
                left: bevy::ui::Val::Percent(x_pct - cell_w / 2.0),
                top: bevy::ui::Val::Percent(y_pct + cell_h / 2.0 + 0.4),
                ..Default::default()
            },
            bevy::prelude::Text(label.to_string()),
            bevy::text::TextFont {
                font: bevy::text::FontSource::Handle(font.clone()),
                font_size: bevy::text::FontSize::Px(22.0),
                ..Default::default()
            },
            bevy::prelude::TextColor(if is_sel { on_wood } else { gold }),
            bevy::picking::Pickable::IGNORE,
            ChildOf(root),
        ));
        commands.entity(cell).insert(ti_engine::pick::PickTarget { key: 1, index: i32::try_from(i).unwrap_or(-1) });
        cell_entities.push(cell);
    }
    let _ = &cell_entities; // 格实体挂 PickTarget 即命中目标（子节点已 IGNORE）
    Some(root)
}

/// 消息/帮助场景：面板 + 动态标题（`msg.<kind>`）+ 正文（`bevy_ui` 自动折行）+ 页点 + 页码。
/// 翻页/换页类型时经 `chrome_ui_system` 重建。
fn build_message_chrome(commands: &mut Commands, font: &Handle<Font>, app: &GameApp) -> Entity {
    let root = commands
        .spawn((
            bevy::ui::Node {
                width: bevy::ui::Val::Percent(100.0),
                height: bevy::ui::Val::Percent(100.0),
                display: bevy::ui::Display::Flex,
                position_type: bevy::ui::PositionType::Relative,
                ..Default::default()
            },
            bevy::picking::Pickable::IGNORE,
        ))
        .id();
    let kind = app.msg_kind().unwrap_or(MsgKind::Help);
    let page = app.msg_page();
    let total = app.msg_total();
    let title_key = format!("msg.{}", kind.label());
    let title = ti_app::i18n::sget(&title_key);
    let body_key = page_text(kind, page);
    let body = ti_app::i18n::sget(&body_key);
    let page_str = ti_app::i18n::sgetf("scene.message.page", &[&(page + 1), &total]);
    let cw = ti_render::framebuffer::LOGICAL_W_I32 as f32;
    let ch = ti_render::framebuffer::LOGICAL_H_I32 as f32;
    // 面板（1080 rect 400,704,608,336 → `%`；暖棕底 + flex column）
    let panel = commands
        .spawn((
            bevy::ui::Node {
                width: bevy::ui::Val::Percent(608.0 / cw * 100.0),
                height: bevy::ui::Val::Percent(336.0 / ch * 100.0),
                position_type: bevy::ui::PositionType::Absolute,
                left: bevy::ui::Val::Percent(400.0 / cw * 100.0),
                top: bevy::ui::Val::Percent(704.0 / ch * 100.0),
                display: bevy::ui::Display::Flex,
                flex_direction: bevy::ui::FlexDirection::Column,
                align_items: bevy::ui::AlignItems::Center,
                justify_content: bevy::ui::JustifyContent::Center,
                padding: bevy::ui::UiRect::px(24.0, 24.0, 24.0, 24.0),
                row_gap: bevy::ui::Val::Px(14.0),
                border_radius: bevy::ui::BorderRadius::px(12.0, 12.0, 12.0, 12.0),
                ..Default::default()
            },
            bevy::ui::BackgroundColor(bevy::color::Color::srgba(0.24, 0.16, 0.09, 0.92)),
            bevy::picking::Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();
    // 标题 / 正文（自动折行）/ 页码（面板内 flex column 子）
    msg_text_child(commands, panel, font, title.to_string(), None, 30.0, bevy::color::Color::srgb(0.97, 0.88, 0.70));
    msg_text_child(commands, panel, font, body.to_string(), Some(92.0), 20.0, bevy::color::Color::srgb(0.92, 0.86, 0.72));
    // 页点（`total` 个方点；`i < page` 实心，否则暗底）
    let dots_row = commands
        .spawn((
            bevy::ui::Node {
                display: bevy::ui::Display::Flex,
                flex_direction: bevy::ui::FlexDirection::Row,
                justify_content: bevy::ui::JustifyContent::Center,
                column_gap: bevy::ui::Val::Px(8.0),
                ..Default::default()
            },
            bevy::picking::Pickable::IGNORE,
            ChildOf(panel),
        ))
        .id();
    for i in 0..total {
        let filled = i < page;
        let color = if filled {
            bevy::color::Color::srgb(0.97, 0.88, 0.70)
        } else {
            bevy::color::Color::srgb(0.15, 0.10, 0.06)
        };
        commands.spawn((
            bevy::ui::Node {
                width: bevy::ui::Val::Px(10.0),
                height: bevy::ui::Val::Px(10.0),
                border_radius: bevy::ui::BorderRadius::px(2.0, 2.0, 2.0, 2.0),
                ..Default::default()
            },
            bevy::ui::BackgroundColor(color),
            bevy::picking::Pickable::IGNORE,
            ChildOf(dots_row),
        ));
    }
    msg_text_child(commands, panel, font, page_str, None, 18.0, bevy::color::Color::srgb(0.70, 0.62, 0.48));
    // 底部提示行（全宽绝对定位）
    let hint = ti_app::i18n::sget("scene.message.nav_hint");
    msg_text_child_abs(commands, root, font, hint.to_string(), 86.0, 18.0, bevy::color::Color::srgb(0.70, 0.62, 0.48));
    root
}

/// 面板内 flex 子：居中文本（`width_pct` = 占父宽百分比，`None` = auto）。
fn msg_text_child(
    commands: &mut Commands,
    parent: Entity,
    font: &Handle<Font>,
    content: String,
    width_pct: Option<f32>,
    size: f32,
    color: bevy::color::Color,
) {
    let mut node = bevy::ui::Node {
        display: bevy::ui::Display::Flex,
        justify_content: bevy::ui::JustifyContent::Center,
        align_items: bevy::ui::AlignItems::Center,
        ..Default::default()
    };
    if let Some(w) = width_pct {
        node.width = bevy::ui::Val::Percent(w);
        node.height = bevy::ui::Val::Auto;
    }
    commands.spawn((
        node,
        bevy::prelude::Text(content),
        bevy::text::TextFont {
            font: bevy::text::FontSource::Handle(font.clone()),
            font_size: bevy::text::FontSize::Px(size),
            ..Default::default()
        },
        bevy::prelude::TextColor(color),
        bevy::picking::Pickable::IGNORE,
        ChildOf(parent),
    ));
}

/// 全宽绝对定位文本（底部提示行；`top_pct` = 顶边 `%`）。
fn msg_text_child_abs(
    commands: &mut Commands,
    parent: Entity,
    font: &Handle<Font>,
    content: String,
    top_pct: f32,
    size: f32,
    color: bevy::color::Color,
) {
    commands.spawn((
        bevy::ui::Node {
            width: bevy::ui::Val::Percent(100.0),
            height: bevy::ui::Val::Percent(3.0),
            position_type: bevy::ui::PositionType::Absolute,
            left: bevy::ui::Val::Percent(0.0),
            top: bevy::ui::Val::Percent(top_pct),
            display: bevy::ui::Display::Flex,
            justify_content: bevy::ui::JustifyContent::Center,
            align_items: bevy::ui::AlignItems::Center,
            ..Default::default()
        },
        bevy::prelude::Text(content),
        bevy::text::TextFont {
            font: bevy::text::FontSource::Handle(font.clone()),
            font_size: bevy::text::FontSize::Px(size),
            ..Default::default()
        },
        bevy::prelude::TextColor(color),
        bevy::picking::Pickable::IGNORE,
        ChildOf(parent),
    ));
}

/// 选角 5 行列表（缩略图 + 两级文案 + 选中态）；行挂 `PickTarget` key=2。
fn build_select_role_chrome(
    commands: &mut Commands,
    server: &AssetServer,
    font: &Handle<Font>,
    g: &GameApp,
) -> Option<Entity> {
    let strings = ti_core::data::strings().ok()?;
    let selected = g.character().map_or(0usize, |c| c.index() as usize);
    let rows: Vec<ti_engine::ui::ChromeRow> = ti_core::domain::selection::Character::all()
        .iter()
        .enumerate()
        .map(|(idx, ch)| {
            let label = ch.label();
            let name_key = format!("role.name.{label}");
            let desc_key = format!("role.desc.{label}");
            ti_engine::ui::ChromeRow {
                image: Some(server.load(format!("assets/ui/role_{label}.png"))),
                line1: strings.get(&name_key).to_owned(),
                line2: strings.get(&desc_key).to_owned(),
                selected: idx == selected,
            }
        })
        .collect();
    let chrome = ti_engine::ui::build_list_chrome(commands, font, "", &rows);
    attach_row_pick(commands, &chrome, 2);
    Some(chrome.root)
}

/// 存读档 5 槽列表（i18n 槽文案 + 有档/空档 + 光标选中态）；行挂 `PickTarget` key=3。
fn build_saveload_chrome(
    commands: &mut Commands,
    font: &Handle<Font>,
    g: &GameApp,
) -> Entity {
    let slots: Vec<_> = g.save_slots().to_vec();
    let cursor = g.saveload_cursor();
    let rows: Vec<ti_engine::ui::ChromeRow> = slots
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let has = s.version > 0;
            let state = ti_app::i18n::sget(if has { "slot.has" } else { "slot.empty" });
            let idx = i + 1;
            let line1 = ti_app::i18n::sgetf("slot.label_fmt", &[&idx, &state]);
            let line2 = if has {
                ti_app::i18n::sgetf(
                    "scene.saveload.slot_summary_fmt",
                    &[&s.character, &s.gold, &s.rng_steps],
                )
            } else {
                ti_app::i18n::sget("scene.saveload.empty_line").to_string()
            };
            ti_engine::ui::ChromeRow { image: None, line1, line2, selected: i == cursor }
        })
        .collect();
    let chrome = ti_engine::ui::build_list_chrome_pos(
        commands,
        font,
        ti_app::i18n::sget("scene.saveload.title"),
        &rows,
        ti_engine::ui::PanelSide::Left,
    );
    attach_row_pick(commands, &chrome, 3);
    chrome.root
}

/// 选图 3 卡片列表（卡片名 i18n `glmapm0N.Map` + 锁/开放态 + 选中高亮）；行挂 `PickTarget` key=2。
fn build_select_map_chrome(
    commands: &mut Commands,
    font: &Handle<Font>,
    g: &GameApp,
) -> Entity {
    let current = g.map_slot().map_or(0usize, |m| m.zero_index() as usize);
    let rows: Vec<ti_engine::ui::ChromeRow> =
        ti_core::domain::selection::MapSlot::all()
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let name_key = format!("glmap{}.Map", m.id());
                let line1 = ti_app::i18n::sget(&name_key).to_string();
                let locked = !g.map_unlocked(*m);
                let line2 =
                    ti_app::i18n::sget(if locked { "scene.select_map.locked" } else { "scene.select_map.open" }).to_string();
                ti_engine::ui::ChromeRow {
                    image: None,
                    line1,
                    line2,
                    selected: i == current,
                }
            })
            .collect();
    let chrome = ti_engine::ui::build_list_chrome_pos(
        commands,
        font,
        ti_app::i18n::sget("scene.select_map.title"),
        &rows,
        ti_engine::ui::PanelSide::Right,
    );
    attach_row_pick(commands, &chrome, 2);
    chrome.root
}

/// 给列表 chrome 的逐行挂 `PickTarget`（**UI 相机层**命中：`bevy_ui` 拾取后端免 marker，
/// 事件落到挂 `PickTarget` 的行节点；1080 层该场景原语已标 `ui_layer` 跳过，不双发）。
fn attach_row_pick(commands: &mut Commands, chrome: &ti_engine::ui::ListChrome, key: u32) {
    for (idx, row_entity) in chrome.rows.iter().enumerate() {
        commands.entity(*row_entity).insert(ti_engine::pick::PickTarget {
            key,
            index: i32::try_from(idx).unwrap_or(-1),
        });
    }
}


/// S29.3：per-scene `bevy_ui` chrome 系统（换场景 / 换选中 → 拆旧建新）。
#[allow(clippy::needless_pass_by_value)]
fn chrome_ui_system(
    mut commands: Commands,
    mut chrome: ResMut<ChromeUi>,
    server: Res<AssetServer>,
    store: ResMut<GameStore>,
) {
    // 系统参数按 Bevy 惯例传值（`Res`/`ResMut` 本身即引用语义容器）。
    let Ok(g) = store.0.lock() else {
        return;
    };
    let scene = SceneKind::from_legacy(g.scene());
    // 重建键 = 场景判别 + 场景相关选中态（选角 = 角色序 / 存读档 = 光标；变化才拆旧建新）。
    let sel_key = match scene {
        SceneKind::SelectRole => g.character().map_or(0usize, |c| c.index() as usize),
        SceneKind::Saveload => g.saveload_cursor(),
        SceneKind::SelectMap => g.map_slot().map_or(0usize, |m| m.zero_index() as usize),
        // 消息：翻页 / 换类型 ⇒ 重建（`page*16 + kind 序`）
        SceneKind::Message => {
            let kind = g.msg_kind().unwrap_or(MsgKind::Help);
            let kind_idx = MsgKind::all().iter().position(|k| *k == kind).unwrap_or(0);
            g.msg_page() as usize * 16 + kind_idx
        }
        _ => 0,
    };
    let key = (scene.discriminant(), sel_key);
    if chrome.built == Some(key) {
        return;
    }
    if let Some(old) = chrome.root.take() {
        commands.entity(old).despawn();
    }
    // S29.3：CJK 字体（不给 ⇒ 中文豆腐块 □□；`complex_script_segmentation` 特性已开）
    let font = ti_engine::canvas::BevyCanvas::load_project_font(&server, "assets/");
    chrome.root = build_scene_chrome(&mut commands, &server, &font, scene, &g);
    chrome.built = Some(key);
}

/// S29.2（B1）：Bevy 拾取命中 → 逻辑（`poll_hit`）。消费 [`PickEvent`]（tap/swipe）→
/// `GameApp::poll_hit(HitData)`。命中/几何全在 Bevy 拾取层（`pick.rs`），本系统只做事件→逻辑。
#[allow(clippy::needless_pass_by_value)]
fn pick_event_system(store: ResMut<GameStore>, mut picker: MessageReader<ti_engine::pick::PickEvent>) {
    let Ok(mut g) = store.0.lock() else {
        return;
    };
    let log_hits = std::env::var("TI_LOG_HITS").is_ok();
    for ev in picker.read() {
        let hit = HitData { key: ev.key, index: ev.index, swipe: ev.swipe };
        let _ = g.poll_hit(Some(hit));
        if log_hits {
            eprintln!("[hit] key={} index={} swipe={} scene={:?}", ev.key, ev.index, ev.swipe, g.scene());
        }
    }
}

/// S29.3 P0（用户报「菜单按键无响应」根因）：**1080 层手动 tap 命中**。
///
/// `bevy_picking` 的 mesh 后端只拾取 `Mesh2d`/`Mesh3d`/`SimplifiedMesh`（`ray_cast:117`），
/// 1080 拾取标记是 `Sprite`（无 `Mesh2d`）⇒ **从未被拾取** ⇒ 菜单/选图/存档 1080 层点不动。
/// 本系统补 1080 几何命中：左键 tap → 窗口逻辑 px 转 1080 letterbox 坐标 → 落在哪个
/// `pick_key>0`（且未迁 `bevy_ui`）原语矩形 → 发 [`PickEvent`]（`pick_event_system` → `poll_hit`）。
///
/// 与 `bevy_ui` 命中层**不双发**：一个物理点击要么落在已迁 `bevy_ui` 的格（`bevy_picking` 拾取，
/// 1080 层该原语 `ui_layer` 已跳过）要么落在未迁的 1080 矩形（本系统命中，`bevy_ui` 层无该格）。
#[allow(clippy::needless_pass_by_value)]
fn manual_1080_hit_system(
    store: Res<GameStore>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_pos: Res<MousePos>,
    window: Query<&Window>,
    mut picker: MessageWriter<ti_engine::pick::PickEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(g) = store.0.lock() else {
        return;
    };
    let scene_key = scene_json_key(g.scene());
    let dbg = std::env::var("TI_LOG_HITS").is_ok();
    drop(g);
    if scene_key.is_empty() {
        return;
    }
    // 窗口逻辑 px → 1080 letterbox 坐标（画布恒完整可见，居中）。
    let Ok(win) = window.single() else {
        return;
    };
    // 该场景 `pick_key>0` 且未迁 bevy_ui 的原语矩形（1080 空间；`z` 越大越靠上）。
    let mut rects: Vec<ti_engine::pick::PickRect> = Vec::new();
    if let Ok(scenes) = ti_core::data::scenes() {
        if let Some(sc) = scenes.get(scene_key) {
            for p in &sc.primitives {
                let key = p
                    .params
                    .get("pick_key")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|v| u32::try_from(v).ok())
                    .unwrap_or(0);
                if key == 0 {
                    continue;
                }
                if p
                    .params
                    .get("ui_layer")
                    .and_then(serde_json::Value::as_str)
                    == Some("bevy_ui")
                {
                    continue;
                }
                let index = p
                    .params
                    .get("pick_index")
                    .and_then(serde_json::Value::as_i64)
                    .and_then(|v| i32::try_from(v).ok())
                    .unwrap_or_else(|| i32::try_from(p.item).unwrap_or(-1));
                rects.push(ti_engine::pick::PickRect {
                    key,
                    index,
                    z: p.z,
                    x: p.rect.x as f32,
                    y: p.rect.y as f32,
                    w: p.rect.w as f32,
                    h: p.rect.h as f32,
                });
            }
        }
    }
    let canvas_w = ti_render::framebuffer::LOGICAL_W_I32 as f32;
    let canvas_h = ti_render::framebuffer::LOGICAL_H_I32 as f32;
    let hit = ti_engine::pick::hit_1080_pick(
        win.width(),
        win.height(),
        mouse_pos.x,
        mouse_pos.y,
        canvas_w,
        canvas_h,
        &rects,
    );
    if let Some((key, index)) = hit {
        picker.write(ti_engine::pick::PickEvent { key, index, swipe: ti_engine::pick::SWIPE_NONE });
        if dbg {
            eprintln!("[hit1080] key={key} index={index} @win({:.0},{:.0})", mouse_pos.x, mouse_pos.y);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
/// 窗口缩放跟随系统（用户 2026-10-26 报告：桌面版不随窗口缩放）：`resizable` 窗口
/// 尺寸变化时按当前逻辑尺寸重建 shell `vp`（屏幕 px → 1080 逻辑反变换），鼠标/触摸
/// 命中判定自动跟上；相机正交 scale 由 `camera_system` 按 `bevy_ortho_scale` 推导
/// （取 `vp.scale()` 的**倒数** ⇒ 画布恒完整可见 + 内容随窗口等比缩放）。
fn window_resize_system(
    store: ResMut<GameStore>,
    mut last_wh: Local<(f32, f32)>,
    window: Query<&Window>,
) {
    let Ok(win) = window.single() else {
        return;
    };
    // 坐标空间口径（bevy 0.20 实测，勿改回 physical）：
    // - 相机 2D 投影 = **逻辑**视口（`bevy_render/camera.rs`：`logical_viewport_size` → `update`）；
    // - 鼠标/触摸输入 = **逻辑**像素（`bevy_winit`：物理坐标 ÷ scale_factor）；
    // ⇒ shell `vp` 必须用**逻辑**尺寸重建，任意 DPI 下输入与渲染同源。
    let wh = (win.width(), win.height());
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
/// 2026-10-26（用户报告「桌面版不随窗口缩放」）：正交 scale 改由 **shell 视口比** 推导，
/// 但**取倒数**（Bevy `o.scale` = 世界单位/像素，shell `vp.scale()` = 像素/逻辑单位），
/// 见 [`bevy_ortho_scale`] 口径卡。效果：画布恒完整可见（letterbox），
/// 且**内容随窗口尺寸等比缩放**（窗口变大 → 可见世界不变 → 1 逻辑单位占更多物理像素）。
fn camera_system(
    store: Res<GameStore>,
    // S29.5：只驱动**游戏世界相机**（`WorldCam` 标记）—— UI 相机（`IsDefaultUiCamera`）与
    // 离屏截图相机（`--shot` 的 `RenderTarget::Image`，scale 须按目标尺寸 contain 固定）都不动。
    // 旧口径 `Query<&mut Projection, With<Camera2d>>` + `single_mut()`：S29.1 起有 2 台相机 ⇒
    // 多实体 ⇒ `single_mut()` 失败 ⇒ **静默不更新**正交 scale ⇒ 窗口缩放跟随失效。
    mut cameras: Query<&mut Projection, With<WorldCam>>,
    mut sprite: Query<&mut Transform, (With<FrameSprite>, Without<Camera2d>)>,
) {
    let Ok(g) = store.0.lock() else {
        return;
    };
    let cam = *g.world_cam();
    let scale = bevy_ortho_scale(g.shell().vp.scale(), cam.zoom);
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
    let scene_key = scene_json_key(g.scene());
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
    // 先按标记清理上帧实体（否则切场景留鬼影）：绘制列表 + 拾取标记一并清。
    let stale: Vec<Entity> = world
        .query_filtered::<Entity, Or<(With<ti_engine::scene_render::DrawListEntity>, With<ti_engine::pick::PickLayerEntity>)>>()
        .iter(world)
        .collect();
    for e in stale {
        world.despawn(e);
    }
    let font = world
        .get_resource::<AssetServer>()
        .map(|server| server.load::<Font>("assets/fonts/lxgw-wenkai-reg.ttf"));
    ti_engine::scene_render::spawn_draw_list(world, font, &list);
    // S29.2：拾取层（该场景 `primitives` 带 `pick_key` 者挂透明可拾取标记 + `PickTarget`）。
    let pick_prims: Vec<ti_core::data::Primitive> = ti_core::data::scenes()
        .ok()
        .and_then(|m| m.get(scene_key))
        .map(|s| s.primitives.clone())
        .unwrap_or_default();
    let _n_pick = ti_engine::scene_render::spawn_pick_layer(world, &pick_prims);
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

/// S31.2 地图包运行期注入（`docs/process/21_map_package_spec.md` §7 选 B）：
/// `TI_MAP=<id>` 环境变量（缺 = `.default` 指针包；再缺 = 纯内嵌默认包，0 行为变化）。
/// 须在**首次表访问前**调用（`table_source::install` 迟注入 = 忽略 + 告警）。
/// 无包可定位（目录缺 / 无 manifest）= 静默走内嵌默认包（wasm 部署侧常态，不打日志）。
fn apply_map_pack(root: &Path) {
    use ti_core::data::map_pack;
    use ti_core::data::table_source;

    let requested: Option<String> = std::env::var("TI_MAP").ok();
    let explicit = requested.is_some();
    let Some(map_id) = requested.or_else(|| map_pack::default_pack_id(root)) else {
        return;
    };
    let tables = map_pack::read_pack_tables(root, &map_id);
    let Some(tables) = tables else {
        eprintln!("[map-pack] TI_MAP={map_id} 缺包/不合规 ⇒ 内嵌默认包（§6 回退）");
        return;
    };
    // 逐段预检（缺段 / 非对象 = 该段回退内嵌，不整包失败）。
    let available = map_pack::available_pack_ids(root);
    let available_len = available.len();
    let effective = table_source::PackTables {
        source_id: Some(tables.id.clone()),
        board: tables
            .board_json
            .filter(|raw| map_pack::precheck_table(raw, map_pack::table_key::BOARD)),
        scenes: tables
            .scenes_json
            .filter(|raw| map_pack::precheck_table(raw, map_pack::table_key::SCENES)),
        strings: tables
            .strings_json
            .filter(|raw| map_pack::precheck_table(raw, map_pack::table_key::STRINGS)),
        available_ids: Some(available.clone()),
    };
    if table_source::install(effective) {
        let requested_mode = if explicit { "显式" } else { "默认包指针" };
        eprintln!(
            "[map-pack] 已注入 {map_id}（{requested_mode}；board/scenes/strings 段注入 + 缺段内嵌回退；已装包清单 {available_len} 项）"
        );
    }
}

/// 启动参数解析（`main` 前置：0 校验逻辑外的参数面，`main` 保短）。
#[derive(Debug, Clone)]
struct Args {
    /// 互斥模式标志（`--resizetest` / `--resizeprobe` / `--shot` / `--selftest`）。
    mode_flags: Vec<String>,
    /// 目标场景（`--scene=<id>`，缺 = `index`）。
    scene_id: String,
    /// 多视口出图（`--viewport=WxH`，S29.5；缺 = 壳默认视口）。
    viewport: Option<(u32, u32)>,
    /// 摇骰次数（`--spin=N`，S19.7 D5；缺 = 0）。
    spin_n: u32,
}

impl Args {
    /// 解析 `std::env::args()`（未知 `--k=v` 忽略，口径 = 既有 `arg_value` 行为）。
    fn parse() -> Self {
        Self {
            mode_flags: [
                "--selftest",
                "--shot",
                "--resizetest",
                "--resizeprobe",
            ]
            .iter()
            .filter(|m| std::env::args().any(|a| a == **m))
            .map(|m| (*m).to_owned())
            .collect(),
            scene_id: arg_value("--scene=").unwrap_or_else(|| "index".to_owned()),
            viewport: arg_value("--viewport=").and_then(|v| parse_viewport(&v)),
            spin_n: arg_value("--spin=").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0),
        }
    }
}

fn main() {
    let args = Args::parse();
    let selftest = args.mode_flags.contains(&"--selftest".to_owned());
    let shot = args.mode_flags.contains(&"--shot".to_owned());
    let resizetest = args.mode_flags.contains(&"--resizetest".to_owned());
    let resizeprobe = args.mode_flags.contains(&"--resizeprobe".to_owned());
    if resizetest && (shot || selftest) {
        eprintln!("[resizetest] 与 --shot/--selftest 互斥（需要真窗口）");
        std::process::exit(2);
    }
    if resizeprobe && (shot || selftest) {
        eprintln!("[resizeprobe] 与 --shot/--selftest 互斥（需要真窗口）");
        std::process::exit(2);
    }
    let scene_id = args.scene_id;
    let viewport = args.viewport;
    // S31.2：地图包注入（须先于 `GameApp` 构造 / 任何表访问；缺包 = 内嵌默认，0 行为变化）。
    apply_map_pack(&repo_root().join("assets").join("maps"));
    let spin_n = args.spin_n;
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
        // S29.3（B1）：UI chrome 迁 bevy_ui（场景层跳过已迁 chrome 原语 → bevy_ui 层画，
        // 避免 1080 + bevy_ui 双画）。**selftest 不开** —— CPU↔Bevy 对拍口径只保游戏世界
        // （UI 层改口径归 S29.4）。
        game.set_chrome_in_bevy_ui(true);
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
        // S29.2（B1）：Bevy 拾取层（`pick.rs`：`PickTarget` 指针事件 → `PickEvent`）+
        // 命中→逻辑（`pick_event_system` 消费 `PickEvent` → `poll_hit`）。
        ti_engine::pick::register_pick(&mut app);
        // S29.3 P0：1080 层手动 tap 命中（`Sprite` 标记不被 bevy_picking 拾取；几何命中兜底）。
        // 须在 `input_system`（更新 `MousePos`）之后、`pick_event_system`（排空 `PickEvent`）之前。
        app.add_systems(Update, manual_1080_hit_system.after(input_system));
        app.add_systems(Update, pick_event_system.after(manual_1080_hit_system));
        // S29.3（B1）：per-scene `bevy_ui` chrome（自动布局；换场景/换选中重建）。
        app.init_resource::<ChromeUi>();
        app.add_systems(Update, chrome_ui_system.after(render_system));
        if resizetest {
            // 机器判定：程序化翻倍窗口 → ①画布恒完整可见（`o.area`≈1080×1920）
            // ②内容随窗口等比放大（px/unit 翻倍）；见 `resizetest_system`。
            attach_resizetest(&mut app);
        }
        if resizeprobe {
            // 活体探针：画面左上角实时 `win/sf/vp/mode/zoom/area`（见 `resizeprobe.rs`）。
            attach_probe(&mut app);
        }
    }
    app.add_systems(Startup, spawn_stage);
    if !selftest {
        // S29.1（B1）：bevy_ui UI 相机 + 最小证明面板（live/shot 路径；selftest 不加，保像素对拍口径）。
        // Startup 顺序 = 添加序（spawn_stage 先建游戏相机，spawn_ui_chrome 后建 UI 相机叠层）。
        app.add_systems(Startup, spawn_ui_chrome);
    }
    if shot {
        run_shot(&mut app, &scene_id, viewport);
        return;
    }
    app.run();
}

/// 取命令行里 `--<prefix><value>` 形式的参数值（S29.5：参数解析收敛）。
fn arg_value(prefix: &str) -> Option<String> {
    std::env::args()
        .skip(1)
        .find_map(|a| a.strip_prefix(prefix).map(str::to_owned))
}

/// 解析 `--viewport=WxH`（S29.5 多视口出图；非法 = None = 默认 1080×1920）。
fn parse_viewport(spec: &str) -> Option<(u32, u32)> {
    let (w_raw, h_raw) = spec.split_once('x')?;
    let w = w_raw.trim().parse::<u32>().ok()?;
    let h = h_raw.trim().parse::<u32>().ok()?;
    if w == 0 || h == 0 {
        return None;
    }
    Some((w, h))
}

/// S18.7：离屏截图模式（`--shot`）：渲染到离屏目标 → GPU 读回 → 写 `evidence/bevy_native_<scene>.png`。
///
/// S18.8b：贴图资产是**异步读盘**（Bevy 资产服务器）→ 每帧 sleep 4ms 让 IO 落地；
/// 空帧自动重截（`finish_shot` 复位 `ShotDone`）。
/// S27.5：启动时全量预载 `ui/`+`fonts/`（治冷启动半成品帧），就绪后才允许截图。
fn run_shot(app: &mut App, scene_id: &str, viewport: Option<(u32, u32)>) {
    // S18.7：离屏渲染目标（`--shot` 验证 Bevy 原生渲染真实出画）。
    // S29.5：`viewport` = 多视口出图（`--viewport=WxH`）⇒ 目标按该尺寸建 ⇒
    // bevy_ui chrome 随视口**真重排**（taffy 重算 `%`/flex）；1080 世界按 contain 缩放
    // （正交 `scale` = max(1080/W, 1920/H) ⇒ 画布恒完整可见，不裁切）。
    let (target_w, target_h) = viewport.unwrap_or((LOGICAL_W, LOGICAL_H));
    let mut target_image =
        Image::new_target_texture(target_w, target_h, TextureFormat::Rgba8UnormSrgb, None);
    target_image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let target_handle = app.world_mut().resource_mut::<Assets<Image>>().add(target_image);
    app.insert_resource(ShotTarget(target_handle.clone()));
    let contain_scale = (LOGICAL_W as f32 / target_w as f32).max(LOGICAL_H as f32 / target_h as f32);
    app.world_mut().spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: contain_scale,
            near: 0.0,
            far: 1000.0,
            ..OrthographicProjection::default_2d()
        }),
        RenderTarget::Image(target_handle.into()),
        Transform::IDENTITY,
    ));
    // S27.5：资产预载（治冷启动「半成品帧」，EXP-127 后续实测）：`assets/ui/` +
    // `assets/fonts/` 全量 `load_untyped`，`shot_preload_poll` 全部 Loaded 才放行
    // 截图请求（`is_blank_frame` 纯色判据 + 600 帧兜底保留为纵深防御）。
    let mut handles: Vec<bevy::asset::UntypedHandle> = Vec::new();
    let server = app.world().resource::<bevy::asset::AssetServer>().clone();
    let repo = repo_root();
    let mut dirs = vec![repo.join("assets/ui"), repo.join("assets/fonts")];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                dirs.push(p);
            } else {
                let rel =
                    p.strip_prefix(&repo).unwrap_or(&p).to_string_lossy().replace('\\', "/");
                match p.extension().and_then(|e| e.to_str()) {
                    Some("ttf" | "otf") => {
                        handles.push(server.load::<Font>(rel).into());
                    }
                    Some("png") => {
                        handles.push(server.load::<Image>(rel).into());
                    }
                    _ => {}
                }
            }
        }
    }
    eprintln!("[shot] 预载 {} 个资产（ui/ + fonts/）", handles.len());
    app.insert_resource(ShotPreload { handles, ready: false });
    app.init_resource::<ShotDone>()
        .add_observer(store_shot)
        .add_systems(Update, (shot_preload_poll, request_shot, finish_shot).chain());
    // S29.1：bevy_ui 布局探针（headless 证据）
    app.init_resource::<UiProbe>().add_systems(Update, ui_probe_system);
    // 手动 update 循环 + 设备轮询（照 S18.3 `bevy_text_align` example 口径：
    // 截图异步完成，需 poll 设备才能拿到结果）。
    //
    // ⚠️ `app.finish()` + `app.cleanup()` **不可省**：`finish()` 才会把渲染 app 的
    // extract 调度启动起来（`RenderAppChannels` 由此就位）；缺它 → extract 阶段
    // panic「resource does not exist: RenderAppChannels」。
    app.finish();
    app.cleanup();
    // S27.5：循环上限 1800（冷启动 VM 上逐文件杀毒扫描实测 ~60 帧只加载 31/66）。
    // 正常路径 = 预载就绪 → 截图 → 读回非空 break（通常数十帧）；上限只兜底异常。
    for _ in 0..1800 {
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
    let suffix = viewport.map_or_else(String::new, |(w, h)| format!("_{w}x{h}"));
    write_shot_png(&format!("evidence/bevy_native_{scene_id}{suffix}.png"), target_w, target_h);
}

/// S18.7：把 GPU 读回像素写成 PNG（`--shot` 输出）。
fn write_shot_png(path: &str, width: u32, height: u32) {
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
    let center_idx = ((height / 2) * width + width / 2) as usize;
    let midpx: Vec<String> =
        raw.chunks(4).skip(center_idx).take(1).map(|c| format!("{c:?}")).collect();
    eprintln!("[shot] 非黑像素 = {nonblack} / alpha>8 = {alpha_nonzero} / {}", raw.len() / 4);
    eprintln!("[shot] 左上角 = {first:?} 屏幕中心 = {midpx:?}");
    let png = ti_render::png::encode_png_rgba(&raw, width, height);
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
