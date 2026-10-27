//! S29.1（B1）：`bevy_ui` UI 层基建（零游戏语义）—— UI 相机分层 + 通用浮动面板。
//!
//! 定位 = `canvas.rs` / `scene_render.rs` 的延伸（Bevy ↔ 渲染框架桥接，零游戏语义，
//! 见 `docs/process/11_bevy_migration.md` §4 + 定案 `docs/process/17_layout_engine.md` B1）：
//! - [`spawn_ui_camera`]：生成 UI 相机（`order=1` + 不清屏），把 `bevy_ui` 叠在 1080 游戏正交相机之上。
//! - [`build_proof_panel`]：通用浮动 `bevy_ui` 面板（全屏根 `Node` flex + 右上角绝对定位卡 +
//!   `Text` 标题 + 按钮外观卡），供 S29.1 最小验证（证明 UI 相机叠层 + `bevy_ui` 渲染 + resize 自动重排）。
//!
//! **双相机分层坑位**（已记 `docs/process/12_bevy_usage.md` §9）：
//! - UI 相机 `order = 1` + `clear_color = None`（不清屏，游戏世界透出）；游戏相机 order=0 负责清屏底色。
//! - UI 树走**屏幕像素**（`bevy_ui` 自动布局 + 窗口 resize 自动重排）；游戏世界走 1080 正交 `o.scale`。
//! - 本模块零游戏语义（不出现场景名/文案/角色）；具体文案由调用方（`treasure_bevy` 壳）传入。
//!
//! 注：Bevy 0.20 的 `Button`/`Interaction` 标记已弃用（0.20 起交互改走 `bevy_picking`，S29.2），
//! 故本面板的「按钮」只做**视觉**（Node + 背景 + 圆角 + 文本），不挂 `Button` 标记；点击接线归 S29.2。

use bevy::camera::ClearColorConfig;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::text::{FontSize, TextColor, TextFont};
use bevy::ui::IsDefaultUiCamera;

/// 生成 UI 相机（叠在游戏正交相机之上；`order=1` + `clear_color=None` 不清屏，游戏世界透出）。
///
/// 须在 Spawn 阶段、游戏相机（`Camera2d` order 0）已 spawn 之后调用。
/// 返回相机实体（调用方可按需插 [`RenderTarget`]，如离屏 `--shot` 模式绑到同一离屏目标）。
#[must_use]
pub fn spawn_ui_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Camera2d,
            IsDefaultUiCamera,
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            Transform::IDENTITY,
        ))
        .id()
}

/// 通用浮动 `bevy_ui` 面板（全屏根 flex + 右上角绝对定位卡，`%` 定位 ⇒ resize 时贴角不漂移）。
///
/// 返回根 `Node` 实体（S29.2 起用于挂命中/交互；S29.3 起按场景建 chrome）。
/// 零游戏语义：`title`/`button_label` 为调用方传入的通用文本。
#[must_use]
pub fn build_proof_panel(commands: &mut Commands, title: &str, button_label: &str) -> Entity {
    // 全屏根（bevy_ui 树须有铺满屏幕的根，`%` 布局据此相对屏幕）
    let root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Relative,
            ..Node::default()
        })
        .id();

    // 右上角绝对定位浮动卡（暖棕底 + 圆角 + 内边距 + 子间距；`ChildOf` 挂 root = 0.20 层级口径）
    let panel = commands
        .spawn((
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                top: Val::Percent(3.0),
                right: Val::Percent(3.0),
                width: Val::Auto,
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::px(16.0, 16.0, 16.0, 16.0),
                border_radius: BorderRadius::px(12.0, 12.0, 12.0, 12.0),
                row_gap: Val::Px(10.0),
                ..Node::default()
            },
            BackgroundColor(Color::srgba(0.10, 0.07, 0.03, 0.82)),
            ChildOf(root),
        ))
        .id();

    // 标题文本（奶油色）
    commands.spawn((
        Node {
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Node::default()
        },
        Text(title.to_string()),
        TextFont { font_size: FontSize::Px(28.0), ..default() },
        TextColor(Color::srgb(0.95, 0.85, 0.60)),
        ChildOf(panel),
    ));

    // 按钮外观卡（视觉占位；点击接线归 S29.2 bevy_picking）
    let button = commands
        .spawn((
            Node {
                width: Val::Px(240.0),
                height: Val::Px(56.0),
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::px(10.0, 10.0, 10.0, 10.0),
                ..Node::default()
            },
            BackgroundColor(Color::srgb(0.72, 0.45, 0.16)),
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text(button_label.to_string()),
        TextFont { font_size: FontSize::Px(24.0), ..default() },
        TextColor(Color::srgb(0.15, 0.10, 0.05)),
        ChildOf(button),
    ));

    root
}
