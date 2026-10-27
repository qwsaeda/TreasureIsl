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
use bevy::camera::visibility::RenderLayers;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::text::{FontSize, FontSource, TextColor, TextFont};
use bevy::picking::Pickable;
use bevy::ui::prelude::{ImageNode, NodeImageMode};
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
            // S29.3：UI 相机**只画 UI**（`RenderLayers` 隔离到 1 层）——
            // 否则它是普通 `Camera2d`，会把 1080 世界按 1:1 再渲一遍盖在正确缩放的世界上
            // （活体窗口 = 画面错位；`--shot` = 目标被清掉只剩 UI）。
            // bevy_ui 的 UI pass 不受 `RenderLayers` 约束 ⇒ UI 节点仍在默认层 0 即可被 UI pass 画出。
            RenderLayers::layer(1),
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            Transform::IDENTITY,
        ))
        .id()
}

/// 列表 chrome 的**意图数据**（一行 = 缩略图 + 标题 + 副标题 + 选中态；零游戏语义，
/// 具体文案/资产由调用方（壳）从 `ti_core` 域 + `strings` + 资产路径组装传入，S29.3）。
#[derive(Debug, Clone)]
pub struct ChromeRow {
    /// 行缩略图（已预载的 bevy 图片 handle；`None` = 该行无缩略图）。
    ///
    /// **0.20 注意**：UI 图片组件是 [`bevy::ui::ImageNode`]（`ImageNode::new(handle)`），
    /// **不是** 旧版 Bevy 的 `Image` 组件、也不是 `Sprite`（`Sprite` 是 1080 世界层用的）。
    pub image: Option<Handle<Image>>,
    /// 行标题（tier = body 字号）。
    pub line1: String,
    /// 行副标题/简介（tier = hint 字号）。
    pub line2: String,
    /// 是否选中行（高亮底色）。
    pub selected: bool,
}

/// 列表 chrome 产物（根实体 + 逐行实体；调用方可给行挂命中标记 `PickTarget`）。
#[derive(Debug, Clone)]
pub struct ListChrome {
    /// 全屏根实体（换场景时 despawn 整棵子树）。
    pub root: Entity,
    /// 逐行实体（顺序 = 行序；挂 `PickTarget` 即该行可拾取）。
    pub rows: Vec<Entity>,
}

/// 建 **自动布局列表 chrome**（S29.3 B1：行位置/高度由 `taffy` flex 算，不手写像素行；
/// 面板宽/边距用 `%` ⇒ resize 自动重排）。零游戏语义：`title`/各行由调用方传入。
///
/// 布局意图：一侧绝对定位竖排面板（`right`/`width` 用 `%` ⇒ resize 贴边不漂移），
/// 面板 = `flex column` + 行间距；每行 = `flex row`（缩略图 + 文案列），选中行换高亮底色。
///
/// **字体**：`font` = 项目 CJK 字体句柄（`load_project_font`）。**不给会出豆腐块** ——
/// `TextFont::font` 默认 `Handle::default()`（无字形）⇒ 中文全变 □□（S29.3 实测）。
#[must_use]
pub fn build_list_chrome(
    commands: &mut Commands,
    font: &Handle<Font>,
    title: &str,
    rows: &[ChromeRow],
) -> ListChrome {
    build_list_chrome_pos(commands, font, title, rows, PanelSide::Right)
}

/// 面板贴边方位（`%` 定位 ⇒ resize 自动重排，不手写像素）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSide {
    /// 右侧贴边（`right`/`width` = %）。
    Right,
    /// 左侧贴边（`left`/`width` = %）。
    Left,
}

/// 建自动布局列表 chrome（面板方位/宽可配；行内容同 [`build_list_chrome`]）。
#[must_use]
pub fn build_list_chrome_pos(
    commands: &mut Commands,
    font: &Handle<Font>,
    title: &str,
    rows: &[ChromeRow],
    side: PanelSide,
) -> ListChrome {
    // 全屏根（bevy_ui 树须有铺满屏幕的根；`%` 布局据此相对屏幕）
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                position_type: PositionType::Relative,
                ..Node::default()
            },
            // S29.3：UI 根节点**铺满屏幕** ⇒ 默认可拾取会**吃掉所有指针事件**
            // （`should_block_lower` = true）⇒ 1080 层的拾取标记（如 index 6 徽章）收不到点击
            // ⇒「菜单按键无响应」。`Pickable::IGNORE` = 拾取完全忽略本实体（不挡下层、不发事件）。
            Pickable::IGNORE,
        ))
        .id();

    // 侧边面板（`%` 定位 + 宽 ⇒ resize 自动重排；竖排 flex + 行间距）
    let mut node = Node {
        display: Display::Flex,
        position_type: PositionType::Absolute,
        top: Val::Percent(4.0),
        width: Val::Percent(34.0),
        height: Val::Auto,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        padding: UiRect::px(16.0, 16.0, 16.0, 16.0),
        border_radius: BorderRadius::px(12.0, 12.0, 12.0, 12.0),
        row_gap: Val::Px(12.0),
        ..Node::default()
    };
    match side {
        PanelSide::Right => {
            node.right = Val::Percent(3.0);
        }
        PanelSide::Left => {
            node.left = Val::Percent(3.0);
        }
    }
    let panel = commands
        .spawn((
            node,
            BackgroundColor(Color::srgba(0.10, 0.07, 0.03, 0.82)),
            // 面板容器本身不拾取（行节点才是命中目标）；否则面板区域会挡住下层 1080 拾取标记。
            Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();

    // 面板标题（空串 = 不建标题节点；标题已在别处画的场景用）
    if !title.is_empty() {
        commands.spawn((
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            Text(title.to_string()),
            TextFont { font: FontSource::Handle(font.clone()), font_size: FontSize::Px(26.0), ..default() },
            TextColor(Color::srgb(0.95, 0.85, 0.60)),
            Pickable::IGNORE,
            ChildOf(panel),
        ));
    }

    // 逐行：flex row（缩略图 + 文案列）；taffy 算行高/位置（不写像素行）
    let mut row_entities = Vec::with_capacity(rows.len());
    for row in rows {
        row_entities.push(build_chrome_row(commands, font, row, panel));
    }

    ListChrome { root, rows: row_entities }
}

/// 列表 chrome 的**单行**（flex row = 缩略图 + 文案列；选中行换黄铜底色）。返回行实体。
fn build_chrome_row(
    commands: &mut Commands,
    font: &Handle<Font>,
    row: &ChromeRow,
    panel: Entity,
) -> Entity {
    let row_bg = if row.selected {
        Color::srgb(0.72, 0.45, 0.16) // 选中 = 黄铜高亮
    } else {
        Color::srgba(0.0, 0.0, 0.0, 0.0) // 非选中 = 透明
    };
    let row_node = commands
        .spawn((
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                padding: UiRect::px(8.0, 8.0, 8.0, 8.0),
                border_radius: BorderRadius::px(8.0, 8.0, 8.0, 8.0),
                ..Node::default()
            },
            BackgroundColor(row_bg),
            ChildOf(panel),
        ))
        .id();

    // 缩略图（bevy_ui 内用 `ImageNode`（0.20 API；非旧 `Image`/`Sprite`）+ 定尺寸 `Node`；
    // `NodeImageMode::Stretch` = 铺满节点、忽略原图比例）
    // S29.3：子节点挂 `Pickable::IGNORE` —— 否则它是行内**最上层可悬停**实体，`PointerClick`
    // 会落在子节点（无 `PickTarget`）而非行节点 ⇒ 命中解析不到。IGNORE 后行节点 = 该区唯一
    // 可拾取实体，事件正确落到挂 `PickTarget` 的行上。
    if let Some(img) = &row.image {
        commands.spawn((
            Node {
                width: Val::Px(72.0),
                height: Val::Px(96.0),
                ..Node::default()
            },
            ImageNode {
                image: img.clone(),
                image_mode: NodeImageMode::Stretch,
                ..Default::default()
            },
            Pickable::IGNORE,
            ChildOf(row_node),
        ));
    }

    // 文案列（标题 + 副标题；taffy 自动换行/高度）
    let text_col = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..Node::default()
            },
            Pickable::IGNORE,
            ChildOf(row_node),
        ))
        .id();
    commands.spawn((
        Text(row.line1.clone()),
        TextFont { font: FontSource::Handle(font.clone()), font_size: FontSize::Px(20.0), ..default() },
        TextColor(Color::srgb(0.95, 0.90, 0.78)),
        Pickable::IGNORE,
        ChildOf(text_col),
    ));
    commands.spawn((
        Text(row.line2.clone()),
        TextFont { font: FontSource::Handle(font.clone()), font_size: FontSize::Px(15.0), ..default() },
        TextColor(Color::srgb(0.70, 0.62, 0.48)),
        Pickable::IGNORE,
        ChildOf(text_col),
    ));
    row_node
}

/// 通用浮动 `bevy_ui` 面板（全屏根 flex + 右上角绝对定位卡，`%` 定位 ⇒ resize 时贴角不漂移）。
///
/// 返回根 `Node` 实体（S29.2 起用于挂命中/交互；S29.3 起按场景建 chrome）。
/// 零游戏语义：`title`/`button_label` 为调用方传入的通用文本。
#[must_use]
pub fn build_proof_panel(
    commands: &mut Commands,
    font: &Handle<Font>,
    title: &str,
    button_label: &str,
) -> Entity {
    // 全屏根（bevy_ui 树须有铺满屏幕的根，`%` 布局据此相对屏幕）
    //
    // S29.3 P0（用户报「菜单按键无响应」）：全屏根默认 `Pickable`（`should_block_lower`=true）
    // 会**吃掉所有指针事件** ⇒ 1080 层徽章/卡片等拾取标记收不到点击（本面板是未迁场景的占位，
    // 铺满屏幕挡在它们之上）。证明面板全部节点挂 `Pickable::IGNORE` = 拾取完全忽略（不挡下层、不发事件）。
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Relative,
                ..Node::default()
            },
            Pickable::IGNORE,
        ))
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
            Pickable::IGNORE,
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
        TextFont { font: FontSource::Handle(font.clone()), font_size: FontSize::Px(28.0), ..default() },
        TextColor(Color::srgb(0.95, 0.85, 0.60)),
        Pickable::IGNORE,
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
            Pickable::IGNORE,
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text(button_label.to_string()),
        TextFont { font: FontSource::Handle(font.clone()), font_size: FontSize::Px(24.0), ..default() },
        TextColor(Color::srgb(0.15, 0.10, 0.05)),
        Pickable::IGNORE,
        ChildOf(button),
    ));

    root
}
