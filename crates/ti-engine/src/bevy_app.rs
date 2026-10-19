//! S18.1 spike：最小 Bevy App（验证三端可跑 + 2D 渲染链路在位）。
//!
//! 窗口 540×960（= 1080×1920 竖屏逻辑的半采样，与 macroquad native 壳同规格）；
//! 内容 = 1 色块（`Sprite::from_color`）+ 1 段 `Text2d`（默认字体；CJK TTF 在 S18.3）。
//!
//! 版本口径：Bevy 0.20（pin，见 `docs/process/11_bevy_migration.md` §1）。
//! API 形态以 0.20 registry 源码实测（编译迭代 3 轮定稿）：
//! - `WindowPlugin { primary_window: Option<Window> }`（0.20 无 `primary_window_builder`）
//! - `Window.resolution = WindowResolution::new(w, h)`（物理像素，无 logical builder）
//! - `Sprite::from_color(color, size)`（色块一步构造；0.20 无 `Sprite::new`）
//! - 2D 文字 = `bevy_sprite::Text2d` + `bevy_text::{TextFont, FontSize}`（0.20 拆分，
//!   字号 = `FontSize::Px(f32)` 枚举，非裸 f32）
//! - 退出 = `MessageWriter<AppExit>` 消息制（0.20 `AppExit` 是 Message，无 `trigger()`；
//!   `App::run` 内部监 `AppExit` 消息自动停循环）
//! - 音频关（双渲染期 `links = "alsa"` 冲突，见 `docs/process/11_bevy_migration.md` §6；S18.8 恢复）

use std::time::Duration;

use bevy::a11y::AccessibilityPlugin;
use bevy::asset::AssetPlugin;
use bevy::camera::ClearColor;
use bevy::image::ImagePlugin;
use bevy::input::InputPlugin;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::text::{FontSize, TextFont, TextPlugin};
use bevy::window::WindowResolution;
use bevy::winit::WinitPlugin;

/// S18.1 spike App：窗口 + 1 色块 + 1 段文字（`Startup` 系统 spawn）。
#[derive(Debug)]
pub struct SpikeApp {
    app: App,
}

impl Default for SpikeApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SpikeApp {
    /// 构造 spike App（窗口 540×960 物理像素；native 自动选 GPU 后端，wasm = WebGL2）。
    pub fn new() -> Self {
        let window = WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(540, 960),
                title: "金银岛 Bevy S18.1 spike".to_owned(),
                ..Default::default()
            }),
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
            .add_plugins(InputPlugin)
            .add_plugins(RenderPlugin::default())
            .add_plugins(WinitPlugin::default())
            .add_plugins(window)
            .add_plugins(TextPlugin)
            .add_systems(Startup, spawn_index)
            .add_systems(Update, exit_after_window);
        // 0.20 拆分：`ClearColor` 资源由应用侧 init（`ViewPlugin` 私有，DefaultPlugins 里才挂）。
        app.init_resource::<ClearColor>();
        Self { app }
    }

    /// 跑主循环（native 阻塞；wasm 由宿主驱动）。
    pub fn run(&mut self) {
        self.app.run();
    }
}

/// Startup：`index` 场景占位（1 色块 + 1 段文字；真实场景渲染在 S18.2 `BevyCanvas` 落地）。
fn spawn_index(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.23, 0.10, 0.09), Vec2::new(400.0, 200.0)),
        Transform::from_translation(Vec3::new(0.0, 160.0, 0.0)),
    ));
    commands.spawn((
        Text2d::new("Bevy S18.1 spike index placeholder"),
        TextFont { font_size: FontSize::Px(32.0), ..Default::default() },
        Transform::from_translation(Vec3::new(0.0, -40.0, 0.0)),
    ));
}

/// Update：累计跑满 5s（≈300 帧@60fps）发 `AppExit` 自动退出
/// （spike 自检：窗口存活 5s + 无 panic 即 2D 渲染链路在位）。
/// 0.20 API：`Time::elapsed()` = 累计 Duration（每帧 delta 是 `Time::delta()`）。
#[allow(clippy::needless_pass_by_value)]
fn exit_after_window(time: Res<Time>, mut exits: MessageWriter<AppExit>) {
    if time.elapsed() > Duration::from_secs(5) {
        exits.write(AppExit::Success);
    }
}
