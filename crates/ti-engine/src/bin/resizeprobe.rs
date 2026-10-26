//! `--resizeprobe` 活体探针（2026-10-26 用户报「窗口能拖但画面不跟着缩放」，需零歧义验证）：
//!
//! - 画面左上角（1080×1920 画布坐标）`Text2d` 实时显示
//!   `win=逻辑宽x高 sf=DPI vp=正交scale mode=缩放档 zoom=镜头倍率`；
//! - 控制台每 30 帧打一行（有 console 的跑法可见）。
//!
//! 判读：拖窗口时 `win=` 与 `vp=` 必须联动变化（winit Resized → `Window` 组件 →
//! `window_resize_system` 重建 shell `vp` → `camera_system` 写相机正交 scale）；
//! 数字不动 = 链路卡住，数字动但画面观感不变 = 视觉口径问题，另查。
//! 探针文本/输出全 ASCII（`check_hardcode` 约束）。

use bevy::prelude::*;
use bevy::camera::{Camera2d, Projection};
use bevy::sprite::Text2d;
use bevy::text::{FontSize, TextColor, TextFont};

use super::{camera_system, GameStore, LOGICAL_H, LOGICAL_W};
use ti_platform::settings::ScaleMode;

/// `--resizeprobe` 开关（`main` 按 CLI 参数插入；未开启时系统零开销早退）。
#[derive(Resource, Default)]
pub struct ResizeProbe {
    pub enabled: bool,
}

/// 探针文本实体标记。
#[derive(Component)]
pub struct ProbeText;

/// 接线：插入探针资源 + spawn 文本 + 每帧刷新系统（`main` 只留一行调用，
/// 避免 `main()` 触 clippy `too_many_lines`）。
pub fn attach_probe(app: &mut App) {
    app.insert_resource(ResizeProbe { enabled: true })
        .add_systems(Startup, spawn_probe)
        .add_systems(Update, probe_system.after(camera_system));
}

/// Startup：画布左上角 spawn 探针文本（z=-0.5 恒在最前可见；相机朝 -Z 看，
/// `z>0` 在相机背后不渲染 —— `12_bevy_usage.md` 坑位）。
pub fn spawn_probe(mut commands: Commands) {
    commands.spawn((
        ProbeText,
        Text2d::new("resizeprobe: waiting...".to_string()),
        TextColor(Color::WHITE),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(
            -(LOGICAL_W as f32) / 2.0 + 28.0,
            LOGICAL_H as f32 / 2.0 - 40.0,
            -0.5,
        )),
    ));
}

/// 每帧刷新探针文本 + 每 30 帧打一行控制台。
///
/// 额外输出相机 `o.area`（Bevy 0.20 `OrthographicProjection.area` = pub 字段 = 实际可见世界
/// 区域）⇒ 直接读出「Bevy 到底显示了多少世界单位」，不靠推导。
#[allow(clippy::needless_pass_by_value)]
pub fn probe_system(
    probe: Res<ResizeProbe>,
    window: Query<&Window>,
    cams: Query<&Projection, With<Camera2d>>,
    mut texts: Query<&mut Text2d, With<ProbeText>>,
    store: Res<GameStore>,
    mut frame: Local<u32>,
) {
    if !probe.enabled {
        return;
    }
    let Ok(win) = window.single() else {
        return;
    };
    let Ok(g) = store.0.lock() else {
        return;
    };
    let mode = g.shell().scale_mode();
    let vp_scale = g.shell().vp.scale();
    let zoom = g.world_cam().zoom;
    drop(g);
    let area = cams.single().ok().and_then(|c| match c {
        Projection::Orthographic(o) => Some(o.area),
        _ => None,
    });
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (wu, hu) = (
        win.width().clamp(0.0, f32::MAX).round() as u32,
        win.height().clamp(0.0, f32::MAX).round() as u32,
    );
    let area_txt = area.map_or_else(|| "area=n/a".to_owned(), |a| {
        format!(
            "area={:.0}x{:.0}@({:.0},{:.0})",
            a.max.x - a.min.x,
            a.max.y - a.min.y,
            a.min.x,
            a.min.y
        )
    });
    let line = format!(
        "win={}x{} sf={} vp={:.3} mode={} zoom={:.2} {area_txt}",
        wu,
        hu,
        win.scale_factor(),
        vp_scale,
        mode_tag(mode),
        zoom,
    );
    *frame = frame.wrapping_add(1);
    if *frame % 30 == 0 {
        eprintln!("[resizeprobe] {line}");
    }
    if let Ok(mut t) = texts.single_mut() {
        t.0 = line;
    }
}

fn mode_tag(m: ScaleMode) -> &'static str {
    match m {
        ScaleMode::Smooth => "smooth",
        ScaleMode::Integer => "integer",
        ScaleMode::Stretch => "stretch",
        ScaleMode::ZoomCrop => "zoomcrop",
    }
}
