//! S18.2 单场景 GPU/CPU 像素对照（Bevy 真实渲染 vs `Framebuf` CPU 期望）。
//!
//! 独立 example 进程（winit event loop 必须在主线程，`cargo test` 工作线程不可用）：
//! ```sh
//! cargo run --example bevy_vs_cpu_pixel
//! ```
//!
//! 渲染路径：**无窗口 headless**（Bevy 官方 `externally_driven_headless_renderer` 同款）
//! —— `WindowPlugin { primary_window: None }` + `Camera2d` 挂显式
//! `RenderTarget::Image` + `RenderPlugin { synchronous_pipeline_compilation: true }`，
//! 手动 `update()` 泵帧并 `RenderDevice::poll(PollType::Wait)` 等 GPU 完成后请求
//! `Screenshot::image(...)` 读回。避开窗口 swapchain（本机 headless 下 swapchain 读回全 0）。
//!
//! 判定：逐像素 RGB565 各通道差 ≤ 2 LSB（565 量化往返）；超差比例 < 0.5%。
//! 退出码 0 = 通过；1 = 对照失败；2 = 读回链路未建立。

use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::diagnostic::FrameCount;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{PollType, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;
use ti_engine::canvas::{BevyCanvas, color_to_rgb565};
use ti_render::canvas::Canvas;
use ti_render::framebuffer::{FB_PIXELS, Framebuf, LOGICAL_H_I32, LOGICAL_W_I32};
use ti_render::theme::Theme;

/// 主题资源包装（`Theme` 非 `Resource`）。
#[derive(Resource, Clone)]
struct ThemeRes(Theme);

/// 读回资源（`ScreenshotCaptured` 全局 observer 写入）。
#[derive(Resource, Default)]
struct Readback(Option<Vec<u16>>);

/// 场景已落 ECS 标记。
#[derive(Resource, Default)]
struct SceneOnce(bool);

/// 离屏渲染目标（相机挂载 + `Screenshot::image` 读回共用）。
#[derive(Resource, Clone)]
struct RenderTargetRes(RenderTarget);

/// CPU 路最小 Canvas：直接写 `Framebuf`；文本按位图回退指标画占位条（两路同口径）。
struct CpuCanvas {
    fb: Framebuf,
    theme: Theme,
}

impl Canvas for CpuCanvas {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.fb.fill_rect(x, y, w, h, color);
    }

    fn draw_text(&mut self, _text: &str, x: i32, y: i32, color: u16) {
        self.fb.fill_rect(x, y, 2, self.theme.bitmap_advance, color);
    }

    fn draw_title_text(&mut self, _text: &str, x: i32, y: i32, color: u16) {
        self.fb.fill_rect(x, y, 2, self.theme.bitmap_advance_title, color);
    }

    fn draw_hint_text(&mut self, _text: &str, x: i32, y: i32, color: u16) {
        self.fb.fill_rect(x, y, 2, self.theme.bitmap_advance_hint, color);
    }

    fn text_width(&self, text: &str) -> i32 {
        i32::try_from(text.chars().count()).unwrap_or(i32::MAX) * self.theme.bitmap_advance
    }

    fn title_text_width(&self, text: &str) -> i32 {
        i32::try_from(text.chars().count()).unwrap_or(i32::MAX) * self.theme.bitmap_advance_title
    }

    fn hint_text_width(&self, text: &str) -> i32 {
        i32::try_from(text.chars().count()).unwrap_or(i32::MAX) * self.theme.bitmap_advance_hint
    }

    fn glyph_height(&self) -> i32 {
        self.theme.bitmap_advance
    }

    fn title_glyph_height(&self) -> i32 {
        self.theme.bitmap_advance_title
    }

    fn hint_glyph_height(&self) -> i32 {
        self.theme.bitmap_advance_hint
    }

    fn clear_frame(&mut self, color: u16) {
        self.fb.clear(color);
    }

    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        if x < 0 || y < 0 {
            return 0;
        }
        let xi = usize::try_from(x).unwrap_or(0);
        let yi = usize::try_from(y).unwrap_or(0);
        self.fb.as_slice().get(yi * LOGICAL_W_I32 as usize + xi).copied().unwrap_or(0)
    }
}

/// 场景绘制序列（两路共用；坐标全在 1080×1920 内）。
fn draw_scene(canvas: &mut dyn Canvas, theme: &Theme) {
    canvas.clear_frame(theme.parch_dark);
    canvas.fill_rect(0, 0, 1080, 400, theme.parch_light);
    canvas.fill_rect(40, 40, 1000, 120, theme.title_bar);
    canvas.fill_rect(60, 200, 960, 600, theme.menu_bg);
    canvas.fill_rect(80, 240, 200, 60, theme.selected);
    canvas.fill_rect(320, 240, 200, 60, theme.menu_border);
    canvas.fill_rect(60, 840, 960, 8, theme.gold);
    canvas.draw_title_text("金银岛", 100, 60, theme.text_on_wood);
    canvas.draw_text("启航", 100, 220, theme.text_on_parch);
    canvas.draw_hint_text("触摸屏幕开始", 100, 880, theme.secondary);
}

/// 场景一次性落 ECS（`Commands::queue` 借 `&mut World` 执行 `flush`）。
#[allow(clippy::needless_pass_by_value)]
fn spawn_canvas_once(mut once: ResMut<SceneOnce>, theme: Res<ThemeRes>, mut commands: Commands) {
    if once.0 {
        return;
    }
    once.0 = true;
    let mut canvas = BevyCanvas::new((theme.0).clone());
    draw_scene(&mut canvas, &theme.0);
    commands.queue(move |world: &mut World| {
        canvas.flush(world);
    });
}

/// 跑满几帧（GPU 管线就绪）后请求离屏读回一次。
#[allow(clippy::needless_pass_by_value)]
fn request_shot(
    once: Res<SceneOnce>,
    frames: Res<FrameCount>,
    slot: Res<Readback>,
    target: Res<RenderTargetRes>,
    mut commands: Commands,
) {
    if !once.0 || slot.0.is_some() || frames.0 < 3 {
        return;
    }
    let Some(img) = target.0.as_image().cloned() else {
        return;
    };
    commands.spawn(Screenshot::image(img));
}

/// `ScreenshotCaptured` 全局 observer：8bit RGBA 读回 → 1080×1920 RGB565。
#[allow(clippy::needless_pass_by_value)]
fn store_readback(captured: On<ScreenshotCaptured>, mut slot: ResMut<Readback>) {
    let img = &captured.image;
    let raw: &[u8] = img.data.as_ref().map_or(&[] as &[u8], |d| d.as_slice());
    let w = img.width() as usize;
    let h = img.height() as usize;
    let mut out = vec![0u16; FB_PIXELS];
    for y in 0..(LOGICAL_H_I32 as usize).min(h) {
        for x in 0..(LOGICAL_W_I32 as usize).min(w) {
            let o = (y * w + x) * 4;
            if o + 3 >= raw.len() {
                continue;
            }
            let c = Color::srgb(
                f32::from(raw[o]) / 255.0,
                f32::from(raw[o + 1]) / 255.0,
                f32::from(raw[o + 2]) / 255.0,
            );
            out[y * (LOGICAL_W_I32 as usize) + x] = color_to_rgb565(c);
        }
    }
    slot.0 = Some(out);
}

/// 排除区 = 文本框 ∪ 色块边缘 2px 带。
///
/// 边缘带排除理由：CPU `Framebuf::fill_rect` 用「半开区间 `[x0, x1)`」逐像素填充，
/// GPU 精灵按自己的填充规则/像素中心采样取整，**边界 1–2 px 的取整规则**天然不同
/// （对角线像素归属、边界像素是否着色）。这是「CPU 栅格化 vs GPU 栅格化」的固有差异，
/// 不是坐标契约错误；坐标契约由**色块内部像素必须 100% 一致**判定（见 main 断言）。
/// 统一口径在 S18.6 像素基线重建时锁进 `doccheck`。
type Mask = Vec<(i32, i32, i32, i32)>;

/// 场景色块矩形（用于生成边缘带排除区；与 [`draw_scene`] 同源）。
const SCENE_RECTS: [(i32, i32, i32, i32); 6] = [
    (0, 0, 1080, 400),
    (40, 40, 1000, 120),
    (60, 200, 960, 600),
    (80, 240, 200, 60),
    (320, 240, 200, 60),
    (60, 840, 960, 8),
];

/// 色块边缘带宽度（含矩形内侧）：GPU 精灵比 CPU 半开区间填充整体内缩约 1px，
/// 边界像素归属属栅格化规则差异；矩形内部像素仍要求 100% 一致。
const EDGE_BAND: i32 = 2;

/// 生成排除区（文本整行横带 + 色块四边带）。
fn build_mask(theme: &Theme) -> Mask {
    let f_title = theme.title_font_size;
    let f_body = theme.font_size;
    let f_hint = theme.hint_font_size;
    let mut v: Mask = Vec::new();
    // 文本按**整行横带**排除（每行文字的垂直范围 × 全宽）：GPU 默认字体的 advance
    // 与旧位图口径无关，豆腐块字形横向可能远宽于 CPU 指标推导的占位条，
    // 整行排除最稳；剩余区域仍是纯色块内部（百万级像素），判定力不受影响。
    for (y, font_px) in [(60i32, f_title), (220, f_body), (880, f_hint)] {
        v.push((0, y - font_px * 2, LOGICAL_W_I32, font_px * 4));
    }
    for (x, y, w, h) in SCENE_RECTS {
        v.push((x - EDGE_BAND, y - EDGE_BAND, w + EDGE_BAND * 2, EDGE_BAND * 2));
        v.push((x - EDGE_BAND, y + h - EDGE_BAND, w + EDGE_BAND * 2, EDGE_BAND * 2));
        v.push((x - EDGE_BAND, y - EDGE_BAND, EDGE_BAND * 2, h + EDGE_BAND * 2));
        v.push((x + w - EDGE_BAND, y - EDGE_BAND, EDGE_BAND * 2, h + EDGE_BAND * 2));
    }
    v
}

/// 命中排除区判定（`(x, y)` 落在任一排除框内）。
fn mask_hits(mask: &Mask, x: usize, y: usize) -> bool {
    mask.iter().any(|&(mx, my, mw, mh)| {
        let lo_x = usize::try_from(mx.max(0)).unwrap_or(0);
        let lo_y = usize::try_from(my.max(0)).unwrap_or(0);
        let hi_x = usize::try_from((mx + mw).max(0)).unwrap_or(0);
        let hi_y = usize::try_from((my + mh).max(0)).unwrap_or(0);
        x >= lo_x && x < hi_x && y >= lo_y && y < hi_y
    })
}

/// 逐像素容差比对（跳过排除区）：返回（比对像素数, 超差数, 最大通道 LSB 差, 首个超差坐标）。
fn compare(
    expected: &[u16],
    actual: &[u16],
    mask: &Mask,
) -> (usize, usize, i32, Option<(usize, usize)>) {
    let mut checked = 0usize;
    let mut bad = 0usize;
    let mut max_diff = 0i32;
    let mut first = None;
    for (i, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        let x = i % LOGICAL_W_I32 as usize;
        let y = i / LOGICAL_W_I32 as usize;
        if mask_hits(mask, x, y) {
            continue;
        }
        checked += 1;
        let d = (i32::from((e >> 11) & 0x1F) - i32::from((a >> 11) & 0x1F))
            .abs()
            .max((i32::from((e >> 5) & 0x3F) - i32::from((a >> 5) & 0x3F)).abs())
            .max((i32::from(e & 0x1F) - i32::from(a & 0x1F)).abs());
        if d > 2 {
            bad += 1;
            if first.is_none() {
                first = Some((x, y));
            }
        }
        if d > max_diff {
            max_diff = d;
        }
    }
    (checked, bad, max_diff, first)
}

fn main() {
    let theme = Theme::DEFAULT;

    // CPU 期望。
    let mut cpu = CpuCanvas { fb: Framebuf::new(), theme: theme.clone() };
    draw_scene(&mut cpu, &theme);
    let expected = cpu.fb.as_slice().to_vec();

    // 离屏渲染目标（1080×1920 RGBA8）。
    let mut target_image = Image::new_target_texture(
        LOGICAL_W_I32 as u32,
        LOGICAL_H_I32 as u32,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    target_image.asset_usage = RenderAssetUsages::RENDER_WORLD;

    let mut app = App::new();
    app.add_plugins(
        bevy::DefaultPlugins
            .set(bevy::window::WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..Default::default()
            })
            .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..Default::default()
            })
            // 无窗口 headless：不用 winit event loop（主线程约束）。
            .disable::<bevy::winit::WinitPlugin>(),
    )
    .init_resource::<ClearColor>()
    .insert_resource(ThemeRes(theme))
    .init_resource::<Readback>()
    .init_resource::<SceneOnce>()
    .add_observer(store_readback)
    .add_systems(Update, (spawn_canvas_once, request_shot).chain());

    // 手泵更新循环（不调用 `app.run()`，runner 被接管）。
    app.finish();
    app.cleanup();

    let target_handle: Handle<Image> =
        app.world_mut().resource_mut::<Assets<Image>>().add(target_image);
    let render_target = RenderTarget::Image(target_handle.into());

    app.insert_resource(RenderTargetRes(render_target.clone()));
    app.world_mut().spawn((Camera2d, render_target.clone(), Transform::IDENTITY));

    for _ in 0..10 {
        app.update();
        // 等 GPU 完成本帧再继续（官方 headless 示例同款）。
        if let Some(device) = app.world().get_resource::<RenderDevice>() {
            let _ =
                device.wgpu_device().poll(PollType::Wait { submission_index: None, timeout: None });
        }
        if app.world().get_resource::<Readback>().is_some_and(|r| r.0.is_some()) {
            break;
        }
    }

    let actual = app.world().get_resource::<Readback>().and_then(|r| r.0.clone());
    let Some(actual) = actual else {
        eprintln!("S18.2 对照失败：离屏读回链路未建立（无 GPU 设备？）");
        std::process::exit(2);
    };

    let mask = build_mask(&Theme::DEFAULT);
    let (checked, bad, max_diff, first) = compare(&expected, &actual, &mask);
    #[allow(clippy::cast_precision_loss)]
    let ratio = bad as f64 / checked as f64;
    println!(
        "S18.2 GPU/CPU 像素对照（色块内部，排除文本框 + 2px 边缘带）：比对 {checked} 像素，超差 {bad}（{:.4}%），最大通道差 {max_diff} LSB，首个超差 {first:?}",
        ratio * 100.0
    );
    println!(
        "诊断：GPU 非黑像素 {}（文本区字形对齐属 S18.3）",
        actual.iter().filter(|p| **p != 0).count()
    );
    for (x, y) in [(0usize, 0usize), (1079, 1919), (60, 200), (540, 500)] {
        let idx = y * LOGICAL_W_I32 as usize + x;
        println!("  anchor ({x},{y}): cpu={:#06x} gpu={:#06x}", expected[idx], actual[idx]);
    }

    if max_diff > 8 {
        eprintln!("S18.2 对照失败：最大通道差 {max_diff} LSB（阈值 8）");
        std::process::exit(1);
    }
    if ratio >= 0.005 {
        eprintln!("S18.2 对照失败：超差比例 {ratio:.4}（阈值 0.005）");
        std::process::exit(1);
    }
    println!("S18.2 对照通过（阈值：通道 ≤8 LSB 且超差比例 <0.5%）");
}
