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
//!
//! **S31.2d（2026-10-29）命中修正**：`bevy_ui` 0.20 自带 `UiPickingPlugin`（`"2d"` 特性链启用），
//! `PointerClick` 直接拾取 `Node` 实体（无需 `Mesh2d` marker）。`attach_row_pick` 把 `PickTarget`
//! 挂在行 `Node` 实体上即可（`pick_flush` 按被点实体查 `PickTarget`）。

use bevy::camera::ClearColorConfig;
use bevy::camera::visibility::RenderLayers;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::{FontSize, FontSource, TextColor, TextFont};
use bevy::ui::IsDefaultUiCamera;
use bevy::ui::prelude::{ImageNode, NodeImageMode};

/// S38 审计 F-01/F-02（用户「引擎用了但很多硬编码」）：bevy_ui chrome + VS 原型的**视觉量
/// 集中为具名常量**——消除 `Color::srgb()/srgba()` + 尺寸魔法数字字面量，CS-1 单源。
///
/// **值不变 = 零视觉变化**（等价替换）。若需走 `ti_render::Theme`（RGB565）映射
/// `rgb565_to_color(theme.xxx)`——注意 Theme 值与现值有偏差，会改变视觉（A 纪律设计稿），
/// 故本审计先做**常量单源**（消除硬编码），Theme 完整迁移列为后续（需用户确认设计稿）。
///
/// 尺寸用 `f32` 常量（引用处 `UiRect::px(PANEL_PAD, …)`），避免 const-fn 限制。
pub mod palette {
    use bevy::color::Color;
    use ti_render::theme::Theme;

    // ── 颜色（S38 Theme 迁移：常量 = theme_color(Theme::DEFAULT.<token>, alpha)，编译期派生）──
    /// `RGB565` → `Color`（含 alpha），const。S38 Theme 迁移：palette 颜色常量编译期从
    /// `Theme::DEFAULT` 派生（theme.json/`Theme::DEFAULT` = 单源；alpha 因 `RGB565` 无 alpha 通道单独存）。
    /// 数学与 [`crate::canvas::rgb565_to_color`] 一致（r/g/b = 5/6/5 bit → 0..1）。
    #[allow(clippy::cast_precision_loss)]
    const fn theme_color(token: u16, alpha: f32) -> Color {
        const MAX5: u32 = 31;
        const MAX6: u32 = 63;
        let v = token as u32;
        let r = (((v >> 11) & MAX5) * 255 / MAX5) as f32 / 255.0;
        let g = (((v >> 5) & MAX6) * 255 / MAX6) as f32 / 255.0;
        let b = ((v & MAX5) * 255 / MAX5) as f32 / 255.0;
        Color::srgba(r, g, b, alpha)
    }
    /// 面板底（棕黑半透明，ui.rs 列表/面板）。
    pub const PANEL_BG: Color = theme_color(Theme::DEFAULT.ui_panel_bg, 0.82);
    /// 金文本（标题/选中行，ui.rs）。
    pub const GOLD_TEXT: Color = theme_color(Theme::DEFAULT.ui_gold_text, 1.0);
    /// 黄铜高亮（选中行/按钮，ui.rs）。
    pub const BRASS_SELECT: Color = theme_color(Theme::DEFAULT.ui_brass_select, 1.0);
    /// 透明。
    pub const TRANSPARENT: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
    /// 米白文本（副标题，ui.rs）。
    pub const CREAM_TEXT: Color = theme_color(Theme::DEFAULT.ui_cream_text, 1.0);
    /// 灰金副文本（ui.rs + `treasure_bevy`）。
    pub const MUTED_GOLD: Color = theme_color(Theme::DEFAULT.ui_muted_gold, 1.0);
    /// 深棕文本（ui.rs）。
    pub const DARK_BROWN_TEXT: Color = theme_color(Theme::DEFAULT.ui_dark_brown_text, 1.0);
    /// 面板深棕半透明（`treasure_bevy` message/其它面板）。
    pub const PANEL_WOOD_BG: Color = theme_color(Theme::DEFAULT.ui_panel_wood_bg, 0.92);
    /// 羊皮亮文本（`treasure_bevy`）。
    pub const PARCH_BRIGHT: Color = theme_color(Theme::DEFAULT.ui_parch_bright, 1.0);
    /// 羊皮米（`treasure_bevy`）。
    pub const PARCH_CREAM: Color = theme_color(Theme::DEFAULT.ui_parch_cream, 1.0);
    /// 深棕文本（蓝分量 0.06，`treasure_bevy`）。
    pub const DARK_BROWN_06: Color = theme_color(Theme::DEFAULT.ui_dark_brown_06, 1.0);
    // VS 原型（vs_duel.rs）
    /// VS HP 槽底（黑半透明）。
    pub const DUEL_SLOT_BG: Color = theme_color(Theme::DEFAULT.ui_duel_slot_bg, 0.9);
    /// VS 按钮底（棕半透明）。
    pub const DUEL_BTN_BG: Color = theme_color(Theme::DEFAULT.ui_duel_btn_bg, 0.95);
    /// VS 道具行底。
    pub const DUEL_ROW_BG: Color = theme_color(Theme::DEFAULT.ui_duel_row_bg, 0.9);
    /// VS 回合徽章（黄铜圆盘）。
    pub const DUEL_ROUND_BADGE: Color = theme_color(Theme::DEFAULT.ui_duel_round_badge, 1.0);
    /// VS 弹窗白闪（alpha 由调用方乘 k）。
    pub const DUEL_POPUP_FLASH: Color = theme_color(Theme::DEFAULT.ui_duel_popup_flash, 1.0);
    /// 调试红棕背景（`bevy_app` 占位）。
    pub const DEBUG_RED_BROWN: Color = theme_color(Theme::DEFAULT.ui_debug_red_brown, 1.0);

    // ── 尺寸（F-02，值 = 原字面量，f32 常量）──
    /// 面板内边距。
    pub const PANEL_PAD: f32 = 16.0;
    /// 面板圆角。
    pub const PANEL_RADIUS: f32 = 12.0;
    /// 面板行距。
    pub const PANEL_ROW_GAP: f32 = 12.0;
    /// 面板行距（小）。
    pub const PANEL_ROW_GAP_SMALL: f32 = 10.0;
    /// 标题字号。
    pub const TITLE_FONT: f32 = 26.0;
    /// 正文字号。
    pub const BODY_FONT: f32 = 20.0;
    /// 提示字号。
    pub const HINT_FONT: f32 = 15.0;
    /// 列表标题字号。
    pub const LIST_TITLE_FONT: f32 = 28.0;
    /// 按钮字号。
    pub const BUTTON_FONT: f32 = 24.0;
    /// 行内列距。
    pub const ROW_COLUMN_GAP: f32 = 12.0;
    /// 行内边距。
    pub const ROW_PAD: f32 = 8.0;
    /// 行圆角。
    pub const ROW_RADIUS: f32 = 8.0;
    /// 缩略图宽。
    pub const THUMB_W: f32 = 72.0;
    /// 缩略图高。
    pub const THUMB_H: f32 = 96.0;
    /// 按钮宽。
    pub const BUTTON_W: f32 = 240.0;
    /// 按钮高。
    pub const BUTTON_H: f32 = 56.0;
    /// 按钮圆角。
    pub const BUTTON_RADIUS: f32 = 10.0;
    // ── F-02b 延伸（treasure_bevy.rs 消息面板/索引环标签 + vs_duel.rs 精灵，值 = 原字面量）──
    /// 索引环标签内边（横/纵）。
    pub const RING_LABEL_PAD_X: f32 = 10.0;
    /// 索引环标签内边纵。
    pub const RING_LABEL_PAD_Y: f32 = 4.0;
    /// 索引环标签字号。
    pub const RING_LABEL_FONT: f32 = 24.0;
    /// 消息面板内边。
    pub const MSG_PANEL_PAD: f32 = 24.0;
    /// 消息面板行距。
    pub const MSG_PANEL_GAP: f32 = 14.0;
    /// 消息面板圆角。
    pub const MSG_PANEL_RADIUS: f32 = 12.0;
    /// 消息标题字号。
    pub const MSG_TITLE_FONT: f32 = 30.0;
    /// 消息正文字号。
    pub const MSG_BODY_FONT: f32 = 20.0;
    /// 消息页码字号。
    pub const MSG_PAGE_FONT: f32 = 18.0;
    /// 消息提示字号。
    pub const MSG_HINT_FONT: f32 = 18.0;
    /// 消息正文宽（%）。
    pub const MSG_BODY_WIDTH_PCT: f32 = 92.0;
    /// 消息提示顶边（%）。
    pub const MSG_HINT_TOP_PCT: f32 = 86.0;
    /// 消息页点列距。
    pub const MSG_DOTS_GAP: f32 = 8.0;
    /// 消息页点边长。
    pub const MSG_DOT_SIZE: f32 = 10.0;
    /// 消息页点圆角。
    pub const MSG_DOT_RADIUS: f32 = 2.0;
    /// VS 回合徽章边长。
    pub const DUEL_BADGE_SIZE: f32 = 64.0;
    /// VS 卡牌精灵宽。
    pub const DUEL_CARD_W: f32 = 120.0;
    /// VS 卡牌精灵高。
    pub const DUEL_CARD_H: f32 = 60.0;
    // VS 动效（F-04：集中 vs_duel 时序/偏移/位图烘焙色，值 = 原字面量零视觉变化）。
    /// VS 冲刺距离（px）。
    pub const DUEL_STRIKE_REACH: f32 = 260.0;
    /// VS 冲刺时长（ms）。
    pub const DUEL_STRIKE_MS: u32 = 260;
    /// VS 命中定格时长（ms）。
    pub const DUEL_HIT_STOP_MS: u32 = 90;
    /// VS 伤害数字上飘时长（ms）。
    pub const DUEL_POPUP_MS: u32 = 700;
    /// VS 伤害数字上飘距离（px）。
    pub const DUEL_POPUP_RISE_PX: f32 = 120.0;
    /// VS 伤害数字锚点横向偏移（px，向命中方偏移）。
    pub const DUEL_HIT_ANCHOR_DX: f32 = 60.0;
    /// VS 伤害数字锚点纵向比例（`fighter_h` 的百分比）。
    pub const DUEL_HIT_ANCHOR_Y_PCT: f32 = 0.55;
    /// VS 屏幕震动幅度（px）。
    pub const DUEL_SHAKE_AMPLITUDE: f32 = 14.0;
    /// VS 屏幕震动时长（ms）。
    pub const DUEL_SHAKE_REMAIN_MS: u32 = 160;
    /// 帧 delta 回退（`Time::delta()` 读失败/极小帧，ms；`vs_duel` 动画 + 游戏帧共用）。
    pub const FRAME_DELTA_FALLBACK_MS: u32 = 16;
    /// VS 动画帧 delta 上限（ms，防动画跳帧；游戏帧上限在 `render_system` 另定 250）。
    pub const DUEL_ANIM_DELTA_CLAMP_MS: u32 = 60;
    /// 伤害数字位图描边 RGB（RGBA8 字节域，`bake_damage_texture`）。
    pub const DUEL_DMG_OUTLINE_RGB: (u8, u8, u8) = (0x20, 0x12, 0x08);
    /// 伤害数字位图芯 RGB（RGBA8 字节域，`bake_damage_texture`）。
    pub const DUEL_DMG_CORE_RGB: (u8, u8, u8) = (0xFF, 0xE8, 0xC0);
}

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
            Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
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

/// 列表 chrome 产物（根实体 + 逐行 `Node` 实体；调用方给行实体挂 `PickTarget` 即可命中）。
///
/// S31.2d（2026-10-29）：`bevy_ui` 自带 `UiPickingPlugin` 直接拾取 `Node` 实体，
/// 无需 `Mesh2d` marker。`rows` = 行 `Node` 实体，挂 `PickTarget` 后鼠标点击生效。
#[derive(Debug, Clone)]
pub struct ListChrome {
    /// 全屏根实体（换场景时 despawn 整棵子树）。
    pub root: Entity,
    /// 逐行实体（顺序 = 行序；挂 `PickTarget` 即该行可点）。
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
        padding: UiRect::px(
            palette::PANEL_PAD,
            palette::PANEL_PAD,
            palette::PANEL_PAD,
            palette::PANEL_PAD,
        ),
        border_radius: BorderRadius::px(
            palette::PANEL_RADIUS,
            palette::PANEL_RADIUS,
            palette::PANEL_RADIUS,
            palette::PANEL_RADIUS,
        ),
        row_gap: Val::Px(palette::PANEL_ROW_GAP),
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
            BackgroundColor(palette::PANEL_BG),
            // 面板容器本身不拾取（行节点才是命中目标）；否则面板区域会挡住下层 1080 拾取标记。
            Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();

    // 面板标题（空串 = 不建标题节点；标题已在别处画的场景用）
    if !title.is_empty() {
        commands.spawn((
            Node { display: Display::Flex, align_items: AlignItems::Center, ..Node::default() },
            Text(title.to_string()),
            TextFont {
                font: FontSource::Handle(font.clone()),
                font_size: FontSize::Px(palette::TITLE_FONT),
                ..default()
            },
            TextColor(palette::GOLD_TEXT),
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

/// 列表 chrome 的**单行**（flex row = 缩略图 + 文案列；选中行换黄铜底色）。
/// 返回行 `Node` 实体（调用方挂 `PickTarget` 即可命中，`UiPickingPlugin` 直接拾取 `Node`）。
fn build_chrome_row(
    commands: &mut Commands,
    font: &Handle<Font>,
    row: &ChromeRow,
    panel: Entity,
) -> Entity {
    let row_bg = if row.selected {
        palette::BRASS_SELECT // 选中 = 黄铜高亮
    } else {
        palette::TRANSPARENT // 非选中 = 透明
    };
    let row_node = commands
        .spawn((
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                column_gap: Val::Px(palette::ROW_COLUMN_GAP),
                padding: UiRect::px(
                    palette::ROW_PAD,
                    palette::ROW_PAD,
                    palette::ROW_PAD,
                    palette::ROW_PAD,
                ),
                border_radius: BorderRadius::px(
                    palette::ROW_RADIUS,
                    palette::ROW_RADIUS,
                    palette::ROW_RADIUS,
                    palette::ROW_RADIUS,
                ),
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
                width: Val::Px(palette::THUMB_W),
                height: Val::Px(palette::THUMB_H),
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
        TextFont {
            font: FontSource::Handle(font.clone()),
            font_size: FontSize::Px(palette::BODY_FONT),
            ..default()
        },
        TextColor(palette::CREAM_TEXT),
        Pickable::IGNORE,
        ChildOf(text_col),
    ));
    commands.spawn((
        Text(row.line2.clone()),
        TextFont {
            font: FontSource::Handle(font.clone()),
            font_size: FontSize::Px(palette::HINT_FONT),
            ..default()
        },
        TextColor(palette::MUTED_GOLD),
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
                padding: UiRect::px(
                    palette::PANEL_PAD,
                    palette::PANEL_PAD,
                    palette::PANEL_PAD,
                    palette::PANEL_PAD,
                ),
                border_radius: BorderRadius::px(
                    palette::PANEL_RADIUS,
                    palette::PANEL_RADIUS,
                    palette::PANEL_RADIUS,
                    palette::PANEL_RADIUS,
                ),
                row_gap: Val::Px(palette::PANEL_ROW_GAP_SMALL),
                ..Node::default()
            },
            BackgroundColor(palette::PANEL_BG),
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
        TextFont {
            font: FontSource::Handle(font.clone()),
            font_size: FontSize::Px(palette::LIST_TITLE_FONT),
            ..default()
        },
        TextColor(palette::GOLD_TEXT),
        Pickable::IGNORE,
        ChildOf(panel),
    ));

    // 按钮外观卡（视觉占位；点击接线归 S29.2 bevy_picking）
    let button = commands
        .spawn((
            Node {
                width: Val::Px(palette::BUTTON_W),
                height: Val::Px(palette::BUTTON_H),
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::px(
                    palette::BUTTON_RADIUS,
                    palette::BUTTON_RADIUS,
                    palette::BUTTON_RADIUS,
                    palette::BUTTON_RADIUS,
                ),
                ..Node::default()
            },
            BackgroundColor(palette::BRASS_SELECT),
            Pickable::IGNORE,
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text(button_label.to_string()),
        TextFont {
            font: FontSource::Handle(font.clone()),
            font_size: FontSize::Px(palette::BUTTON_FONT),
            ..default()
        },
        TextColor(palette::DARK_BROWN_TEXT),
        Pickable::IGNORE,
        ChildOf(button),
    ));

    root
}
