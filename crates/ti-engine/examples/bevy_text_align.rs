//! S18.3 三档字号对齐验收：GPU（TTF 图集）vs CPU（`ab_glyph` 位图光栅）逐档对照。
//!
//! 独立进程（winit event loop 主线程约束）：
//! ```sh
//! cargo run --example bevy_text_align
//! ```
//!
//! 判定：advance 对齐——同一 TTF（`assets/fonts/lxgw-wenkai-reg.ttf`）下，
//! CPU `TtfFont::advance` 求和 与 GPU `TextLayoutInfo` 排版宽度 的比例 ∈ [0.80, 1.30]
//! （parley/cosmic-text 与 `ab_glyph` shaping 规则不同，per-char advance 必然不完全相同；
//! 比例一致 = shaping 配置未漂移）。GPU 非黑像素 >0 确认文字已渲染。

use std::path::PathBuf;
use std::sync::Mutex;

use ab_glyph::{Font as AbGlyphFont, PxScale, ScaleFont};
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::camera::RenderTarget;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{PollType, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::text::{Font, TextLayoutInfo};
use bevy::window::ExitCondition;
use ti_engine::canvas::{BevyCanvas, color_to_rgb565};
use ti_render::canvas::Canvas;
use ti_render::font::TtfFont;
use ti_render::framebuffer::{FB_PIXELS, Framebuf, LOGICAL_H_I32, LOGICAL_W_I32};
use ti_render::theme::Theme;

thread_local! {
    static READBACK: Mutex<Option<Vec<u8>>> = const { Mutex::new(None) };
}

/// advance 比例阈值（parley vs `ab_glyph` shaping 差异容忍区间）。
const ADV_LO: f32 = 0.80;
const ADV_HI: f32 = 1.30;

/// 三档档位（与 `BevyCanvas` 三档口径同名）。
#[derive(Clone, Copy)]
enum Tier {
    /// 正文档（44px）。
    Body,
    /// 标题档（96px）。
    Title,
    /// 提示档（28px）。
    Hint,
}

/// 三档测试文本（含 CJK + ASCII + 数字，覆盖三档字号）。
const SAMPLES: [(&str, Tier); 3] = [
    ("金银岛 TreasureIsl 2026", Tier::Body),
    ("启航出发 · 大航海时代", Tier::Title),
    ("触摸屏幕开始航行", Tier::Hint),
];

/// 文字行 y 坐标（逻辑；与 `BevyCanvas` 的三档字号对应）。
const ROWS: [(i32, f32); 3] = [(200, 44.0), (400, 96.0), (600, 28.0)];

/// CPU 路：`ab_glyph` 光栅化三档文字到 `Framebuf`（与 `MenuShell::draw_ttf_row` 同算法）。
fn cpu_render(ttf: (&TtfFont, &TtfFont, &TtfFont), out: &mut Framebuf) {
    out.clear(0x0000);
    for ((text, tier), (y, _)) in SAMPLES.iter().zip(ROWS.iter()) {
        let font = match tier {
            Tier::Title => ttf.1,
            Tier::Hint => ttf.2,
            Tier::Body => ttf.0,
        };
        draw_text_row(out, font, text, 100, *y, 0xFFFF);
    }
}

/// 逐字光栅化（advance 推进 + `Raster::blit`，baseline 对齐同 `draw_ttf_row`）。
fn draw_text_row(fb: &mut Framebuf, font: &TtfFont, text: &str, x: i32, y: i32, color: u16) {
    let scaled = font.font().as_scaled(PxScale::from(font.size_px()));
    #[allow(clippy::cast_precision_loss)]
    let mut cursor = x as f32;
    for ch in text.chars() {
        let raster = font.rasterize(ch);
        if raster.ink_count() > 0 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let y_offset = match scaled.outline_glyph(scaled.scaled_glyph(ch)) {
                Some(outlined) => -outlined.px_bounds().min.y.ceil() as i32,
                None => 0,
            };
            #[allow(clippy::cast_possible_truncation)]
            raster.blit(fb, cursor.round() as i32, y + y_offset, color);
        }
        cursor += font.advance(ch);
    }
}

/// GPU 路：落 ECS（`BevyCanvas` + TTF 句柄），并记录 `TextLayoutInfo` 排版宽度。
#[allow(clippy::needless_pass_by_value)]
fn spawn_gpu_scene(
    mut once: ResMut<SceneOnce>,
    theme: Res<ThemeRes>,
    font: Res<FontRes>,
    mut commands: Commands,
) {
    if once.0 {
        return;
    }
    once.0 = true;
    let mut canvas = BevyCanvas::new((theme.0).clone());
    canvas.set_font(font.0.clone());
    canvas.clear_frame(0x0000);
    for ((text, tier), (y, _)) in SAMPLES.iter().zip(ROWS.iter()) {
        match tier {
            Tier::Title => canvas.draw_title_text(text, 100, *y, 0xFFFF),
            Tier::Hint => canvas.draw_hint_text(text, 100, *y, 0xFFFF),
            Tier::Body => canvas.draw_text(text, 100, *y, 0xFFFF),
        }
    }
    commands.queue(move |world: &mut World| {
        canvas.flush(world);
    });
}

/// 读回 GPU 排版结果（advance 对齐判定用）。
///
/// 等字体资产加载完且 `TextLayoutInfo.size` 全部 > 0（已布局）后才采集一次。
#[allow(clippy::needless_pass_by_value)]
fn collect_layouts(
    mut layouts: ResMut<LayoutsRes>,
    fonts: Res<Assets<Font>>,
    font_res: Res<FontRes>,
    q: Query<(Entity, &Text2d, &TextLayoutInfo)>,
) {
    if layouts.0.is_some() {
        return;
    }
    if !fonts.contains(&font_res.0) {
        return;
    }
    let mut v: Vec<(String, f32, f32)> = Vec::new();
    for (_, t, info) in q.iter() {
        v.push((t.0.clone(), info.size.x, info.size.y));
    }
    if !v.is_empty() && v.iter().all(|(_, w, _)| *w > 0.0) {
        layouts.0 = Some(v);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request_shot(
    once: Res<SceneOnce>,
    layouts: Res<LayoutsRes>,
    mut shot: ResMut<ShotDone>,
    target: Res<ImgTargetRes>,
    frames: Res<bevy::diagnostic::FrameCount>,
    mut commands: Commands,
) {
    if !once.0 || layouts.0.is_none() || shot.0 || frames.0 < 3 {
        return;
    }
    shot.0 = true;
    commands.spawn(Screenshot::image(target.0.clone()));
}

/// 读回请求已发出标记（只发一次）。
#[derive(Resource, Default)]
struct ShotDone(bool);

/// 离屏渲染目标句柄资源（`Screenshot::image` 读回共用）。
#[derive(Resource, Clone)]
struct ImgTargetRes(Handle<Image>);

#[allow(clippy::needless_pass_by_value)]
fn store_readback(captured: On<ScreenshotCaptured>) {
    let raw: Vec<u8> =
        captured.image.data.as_ref().map_or_else(Vec::new, |d| d.as_slice().to_vec());
    READBACK.with(|cell| {
        if let Ok(mut g) = cell.lock() {
            *g = Some(raw);
        }
    });
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[derive(Resource, Default)]
struct SceneOnce(bool);

#[derive(Resource, Clone)]
struct ThemeRes(Theme);

#[derive(Resource, Clone)]
struct FontRes(Handle<Font>);

#[derive(Resource, Default)]
struct LayoutsRes(Option<Vec<(String, f32, f32)>>);

/// GPU RGBA8 → RGB565 转换（与 `bevy_vs_cpu_pixel.rs` 同口径）。
fn gpu_to_rgb565(gpu_raw: &[u8]) -> Vec<u16> {
    let mut out = vec![0u16; FB_PIXELS];
    let gw = LOGICAL_W_I32 as usize;
    let gh = LOGICAL_H_I32 as usize;
    for y in 0..gh {
        for x in 0..gw {
            let o = (y * gw + x) * 4;
            if o + 2 < gpu_raw.len() {
                let c = Color::srgb(
                    f32::from(gpu_raw[o]) / 255.0,
                    f32::from(gpu_raw[o + 1]) / 255.0,
                    f32::from(gpu_raw[o + 2]) / 255.0,
                );
                out[y * gw + x] = color_to_rgb565(c);
            }
        }
    }
    out
}

/// advance 对齐判定（打印逐档比例）。
fn check_advance(layouts: &[(String, f32, f32)], ttf: (&TtfFont, &TtfFont, &TtfFont)) -> bool {
    println!("S18.3 三档字号对齐（CPU ab_glyph advance vs GPU parley 排版宽度）：");
    let mut ok_all = true;
    for (text, gpu_w, gpu_h) in layouts {
        let tier = SAMPLES.iter().find(|(t, _)| t == text).map_or(Tier::Body, |(_, tier)| *tier);
        let font = match tier {
            Tier::Title => ttf.1,
            Tier::Hint => ttf.2,
            Tier::Body => ttf.0,
        };
        let cpu_w: f32 = text.chars().map(|ch| font.advance(ch)).sum();
        let ratio = if cpu_w > 0.0 { *gpu_w / cpu_w } else { 0.0 };
        let ok = *gpu_w > 0.0 && (ADV_LO..=ADV_HI).contains(&ratio);
        ok_all &= ok;
        println!(
            "  「{text}」 cpu_adv={cpu_w:.1} gpu_w={gpu_w:.1} 比例={ratio:.3} {}（cpu 行高={:.1} gpu h={gpu_h:.1}）",
            if ok { "OK" } else { "超差" },
            font.line_height()
        );
    }
    ok_all
}

fn main() {
    let theme = Theme::DEFAULT;
    let font_path = repo_root().join("assets/fonts/lxgw-wenkai-reg.ttf");
    let bytes = std::fs::read(&font_path).unwrap_or_else(|e| {
        eprintln!("S18.3 失败：读不到字体 {}：{e}", font_path.display());
        std::process::exit(2);
    });

    #[allow(clippy::cast_precision_loss)]
    let ttf_title = TtfFont::from_bytes(&bytes, theme.title_font_size as f32);
    #[allow(clippy::cast_precision_loss)]
    let ttf_body = TtfFont::from_bytes(&bytes, theme.font_size as f32);
    #[allow(clippy::cast_precision_loss)]
    let ttf_hint = TtfFont::from_bytes(&bytes, theme.hint_font_size as f32);
    let (Some(ttf_title), Some(ttf_body), Some(ttf_hint)) = (ttf_title, ttf_body, ttf_hint) else {
        eprintln!("S18.3 失败：TTF 解析失败");
        std::process::exit(2);
    };

    let mut cpu_fb = Framebuf::new();
    cpu_render((&ttf_body, &ttf_title, &ttf_hint), &mut cpu_fb);

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
            .set(bevy::asset::AssetPlugin {
                file_path: repo_root().to_string_lossy().to_string(),
                ..Default::default()
            })
            .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..Default::default()
            })
            .disable::<bevy::winit::WinitPlugin>(),
    )
    .init_resource::<ClearColor>()
    .insert_resource(ThemeRes(theme))
    .init_resource::<SceneOnce>()
    .init_resource::<LayoutsRes>()
    .init_resource::<ShotDone>()
    .add_observer(store_readback)
    .add_systems(Update, (spawn_gpu_scene, collect_layouts, request_shot).chain());

    app.finish();
    app.cleanup();

    let target_handle: Handle<Image> =
        app.world_mut().resource_mut::<Assets<Image>>().add(target_image);
    let render_target = RenderTarget::Image(target_handle.clone().into());
    app.insert_resource(ImgTargetRes(target_handle));
    app.world_mut().spawn((Camera2d, render_target, Transform::IDENTITY));

    let font_handle: Handle<Font> =
        app.world().resource::<AssetServer>().load("assets/fonts/lxgw-wenkai-reg.ttf");
    app.insert_resource(FontRes(font_handle));

    for _ in 0..30 {
        app.update();
        if let Some(device) = app.world().get_resource::<RenderDevice>() {
            let _ =
                device.wgpu_device().poll(PollType::Wait { submission_index: None, timeout: None });
        }
        if READBACK.with(|c| c.lock().is_ok_and(|g| g.is_some())) {
            break;
        }
    }

    let gpu_raw = READBACK.with(|c| c.lock().ok().and_then(|g| g.clone()));
    let Some(gpu_raw) = gpu_raw else {
        eprintln!("S18.3 失败：GPU 读回未建立");
        std::process::exit(2);
    };

    let gpu565 = gpu_to_rgb565(&gpu_raw);
    let gpu_nonblack = gpu565.iter().filter(|p| **p != 0).count();
    println!("S18.3 GPU 非黑像素 {gpu_nonblack}（>0 = 文字已渲染）");

    let layouts = app.world().resource::<LayoutsRes>().0.clone();
    let Some(layouts) = layouts else {
        eprintln!("S18.3 失败：未取到 TextLayoutInfo");
        std::process::exit(2);
    };

    let adv_ok = check_advance(&layouts, (&ttf_body, &ttf_title, &ttf_hint));
    if !adv_ok {
        eprintln!("S18.3 失败：advance 比例超出 [{ADV_LO}, {ADV_HI}]");
        std::process::exit(1);
    }
    if gpu_nonblack == 0 {
        eprintln!("S18.3 失败：GPU 文字未渲染（非黑像素 0）");
        std::process::exit(1);
    }
    println!("S18.3 通过（advance 比例 ∈ [{ADV_LO},{ADV_HI}]；GPU 文字已渲染）");
}
