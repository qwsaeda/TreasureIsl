//! S85.3 通用组件渲染器：把 schema [`Component`] 列表落成 `bevy_ui` 节点树的通用通道。
//!
//! 从 `crates/ti-engine/src/ui.rs` 拆出（结构审计：文件规模 >900 行 / 函数 >140 行），
//! 零游戏语义边界不变：只认 [`Component`]（type/role/binding/layout/action/state）、
//! [`AssetRoles`]（role→资产路径）与 [`ComponentHost`]（语义字符串→业务数据/命中 key），
//! 绝不出现 Character / MapSlot / GameApp 等业务类型；业务取数全在壳侧 resolver。
//!
//! 公共入口 [`build_component_chrome`] 由 `ui.rs` 的同名薄封装再导出，保持
//! `ti_engine::ui::build_component_chrome` 路径不变（壳侧业务逻辑零改动）。

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::{FontSize, FontSource, TextColor, TextFont};
use bevy::ui::prelude::{ImageNode, NodeImageMode};
use ti_core::data::ui_schema::{AssetRoles, Component, Layout};

use crate::ui::{ChromeRow, build_chrome_row, centered_left_pct, palette, tier_font_px};

/// binding 解析结果（零游戏语义：只有「文本」/「列表行」两类）。
#[derive(Debug, Clone)]
pub enum BindingValue {
    /// 单段文本（`text`/`button` 组件的 binding → i18n key 解析结果）。
    Text(String),
    /// 业务列表行（`list`/`panel` 组件的 binding → 壳侧展开）。
    Rows(Vec<ChromeRow>),
    /// 无值（缺 binding / 解析失败 → 组件降级为不显示内容）。
    Empty,
}

/// 通用组件宿主：壳侧实现，把**语义字符串**（binding/action）解析为渲染数据 / 命中 key。
///
/// 这是渲染器与业务之间的**窄接口**：渲染器只传字符串、只收 [`BindingValue`]，
/// 不知道字符串背后是角色、存档槽还是地图卡（S85.4 扩展为完整 binding 注册表）。
pub trait ComponentHost {
    /// binding 名 → 解析值（i18n 文本或业务行）。
    fn resolve(&self, binding: &str) -> BindingValue;
    /// action 名 → opaque 命中 key（具体映射由壳侧注册表决定；`None` = 不可点）。
    fn action_key(&self, action: &str) -> Option<u32>;
}

/// 组件可见状态判定（`state` 是允许出现的状态集合）。
///
/// 当前数据只在 `list` 上声明 `["normal","selected"]`（两种状态都显示，选中高亮在行内），
/// 故恒可见；单侧状态（仅 `selected` / 仅 `normal`）按选中下标做粗过滤。
/// 逐项 `item` 级精确过滤随 S85.4 的 binding-index 一同落地。
fn component_visible(component: &Component, selected: i64) -> bool {
    if component.state.is_empty() {
        return true;
    }
    let has_normal = component.state.iter().any(|s| s == "normal");
    let has_selected = component.state.iter().any(|s| s == "selected");
    match (has_normal, has_selected) {
        (true, true) | (false, false) => true,
        (true, false) => selected < 0,
        (false, true) => selected >= 0,
    }
}

/// 布局交叉轴对齐（意图 → taffy `AlignItems`）。
fn component_align(align: &str) -> AlignItems {
    match align {
        "start" | "flex_start" => AlignItems::FlexStart,
        "end" | "flex_end" => AlignItems::FlexEnd,
        "center" => AlignItems::Center,
        _ => AlignItems::Stretch,
    }
}

/// 锚点参考位置（视口百分比 0..100）。
fn anchor_xy(anchor: &str) -> (f32, f32) {
    match anchor {
        "top_center" => (50.0, 0.0),
        "top_right" => (100.0, 0.0),
        "center_left" => (0.0, 50.0),
        "center" => (50.0, 50.0),
        "center_right" => (100.0, 50.0),
        "bottom_left" => (0.0, 100.0),
        "bottom_center" => (50.0, 100.0),
        "bottom_right" => (100.0, 100.0),
        _ => (0.0, 0.0), // top_left / 缺省
    }
}

/// 锚点对应的「节点尺寸扣除比例」（让节点**锚点**而非左上角落到参考点）。
fn anchor_shift(anchor: &str) -> (f32, f32) {
    let shift_x = match anchor {
        "top_center" | "center" | "bottom_center" => 0.5,
        "top_right" | "center_right" | "bottom_right" => 1.0,
        _ => 0.0,
    };
    let shift_y = match anchor {
        "center_left" | "center" | "center_right" => 0.5,
        "bottom_left" | "bottom_center" | "bottom_right" => 1.0,
        _ => 0.0,
    };
    (shift_x, shift_y)
}

/// 布局意图 → taffy `Node`（**只用百分比 / Auto，无绝对像素**）。
///
/// `fill`/`center`/`side_*`/`bottom_bar` 为语义布局；`anchored` 用锚点 + `offset_pct`
/// + `size_pct`（皆百分比）做精细定位——两者都交给 taffy，resize 时自动重排。
///
/// 屏心居中派生复用 `crate::ui::centered_left_pct`（`(100.0 - w) / 2.0`），
/// 与 `canvas.rs` 世界层 `as f32 / 2.0` 半值派生成对（doccheck2 `check_engine_no_bare_half`）。
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn component_layout_node(layout: &Layout, kind: &str) -> Node {
    let mut node = Node {
        display: Display::Flex,
        align_items: component_align(&layout.align),
        ..Node::default()
    };
    if layout.align.is_empty() {
        node.align_items = AlignItems::Center;
    }
    let width = layout.width_pct.unwrap_or(34.0);
    match layout.intent.as_str() {
        "fill" => {
            node.width = Val::Percent(100.0);
            node.height = Val::Percent(100.0);
        }
        "center" => {
            // panel 缺省高度 = 宽度（消息面板 55%×55%）；image 缺省 Auto = 原图尺寸居中。
            let w = layout.size_pct.map_or(width, |s| s.w);
            let h =
                layout.size_pct.map_or_else(|| if kind == "panel" { width } else { 0.0 }, |s| s.h);
            node.position_type = PositionType::Absolute;
            node.left = Val::Percent(centered_left_pct(w));
            node.top = Val::Percent(centered_left_pct(h));
            node.width = if w > 0.0 { Val::Percent(w) } else { Val::Auto };
            node.height = if h > 0.0 { Val::Percent(h) } else { Val::Auto };
            node.justify_content = JustifyContent::Center;
            if kind == "panel" {
                node.padding = UiRect::px(
                    palette::MSG_PANEL_PAD,
                    palette::MSG_PANEL_PAD,
                    palette::MSG_PANEL_PAD,
                    palette::MSG_PANEL_PAD,
                );
                node.row_gap = Val::Px(palette::MSG_PANEL_GAP);
                node.border_radius = BorderRadius::px(
                    palette::MSG_PANEL_RADIUS,
                    palette::MSG_PANEL_RADIUS,
                    palette::MSG_PANEL_RADIUS,
                    palette::MSG_PANEL_RADIUS,
                );
            }
        }
        "side_left" | "side_right" => {
            node.position_type = PositionType::Absolute;
            node.top = Val::Percent(4.0);
            node.width = Val::Percent(width);
            node.height = Val::Auto;
            node.flex_direction = FlexDirection::Column;
            node.padding = UiRect::px(
                palette::PANEL_PAD,
                palette::PANEL_PAD,
                palette::PANEL_PAD,
                palette::PANEL_PAD,
            );
            node.row_gap = Val::Px(palette::PANEL_ROW_GAP);
            node.border_radius = BorderRadius::px(
                palette::PANEL_RADIUS,
                palette::PANEL_RADIUS,
                palette::PANEL_RADIUS,
                palette::PANEL_RADIUS,
            );
            if layout.intent == "side_left" || layout.side == "left" {
                node.left = Val::Percent(3.0);
            } else {
                node.right = Val::Percent(3.0);
            }
        }
        "bottom_bar" => {
            node.position_type = PositionType::Absolute;
            node.left = Val::Percent(0.0);
            node.top = Val::Percent(palette::MSG_HINT_TOP_PCT);
            node.width = Val::Percent(100.0);
            node.height = Val::Auto;
            node.justify_content = JustifyContent::Center;
        }
        "flex_col" | "flex_row" => {
            let col = layout.intent == "flex_col";
            node.flex_direction = if col { FlexDirection::Column } else { FlexDirection::Row };
            let gap = Val::Px(layout.gap.unwrap_or(8) as f32);
            if col {
                node.row_gap = gap;
            } else {
                node.column_gap = gap;
            }
        }
        "anchored" => {
            let (ax, ay) = anchor_xy(&layout.anchor);
            let (sx, sy) = anchor_shift(&layout.anchor);
            let (ox, oy) = layout.offset_pct.map_or((0.0, 0.0), |o| (o.x, o.y));
            let w = layout.size_pct.map_or(0.0, |s| s.w);
            let h = layout.size_pct.map_or(0.0, |s| s.h);
            node.position_type = PositionType::Absolute;
            node.left = Val::Percent(ax + ox - w * sx);
            node.top = Val::Percent(ay + oy - h * sy);
            node.width = if w > 0.0 { Val::Percent(w) } else { Val::Auto };
            node.height = if h > 0.0 { Val::Percent(h) } else { Val::Auto };
        }
        _ => {
            node.width = Val::Auto;
            node.height = Val::Auto;
        }
    }
    node
}

/// 单组件 `type` 分派：把内容组件挂到已 spawn 的 `node` 实体上。
///
/// 从 `build_component_chrome` 抽出（函数规模阈值）；原内联分支里的 `continue`
/// 在 helper 内改为 `return`，跳过末尾的 action → `PickTarget` 挂载，行为完全一致。
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn spawn_component_content(
    commands: &mut Commands,
    server: &AssetServer,
    font: &Handle<Font>,
    component: &Component,
    roles: &AssetRoles,
    host: &dyn ComponentHost,
    node: Entity,
    selected: i64,
) {
    let theme = ti_render::theme::Theme::DEFAULT;
    match component.kind.as_str() {
        "image" | "grad" | "glow" => {
            let path = roles.path(&component.role).map(str::to_owned).or_else(|| {
                if component.icon.is_empty() {
                    None
                } else {
                    Some(format!("assets/ui/icon_{}.png", component.icon))
                }
            });
            if let Some(path) = path {
                commands.entity(node).insert((
                    ImageNode {
                        image: server.load(path),
                        color: Color::WHITE,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
        }
        "rect" => {
            let color = theme
                .by_name(&component.role)
                .map_or(palette::PANEL_BG, crate::canvas::rgb565_to_color);
            commands.entity(node).insert((BackgroundColor(color), Pickable::IGNORE));
        }
        "text" => {
            let text = match host.resolve(&component.binding) {
                BindingValue::Text(t) => t,
                _ => component.text.clone(),
            };
            if text.is_empty() {
                return;
            }
            let color = match component.tier.as_str() {
                "title" => palette::GOLD_TEXT,
                "hint" => palette::MUTED_GOLD,
                _ => palette::CREAM_TEXT,
            };
            commands.entity(node).insert((
                Text(text),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(tier_font_px(&component.tier)),
                    ..default()
                },
                TextColor(color),
                Pickable::IGNORE,
            ));
        }
        "list" | "panel" => spawn_list_content(commands, font, component, host, node),
        "button" => {
            let text = match host.resolve(&component.binding) {
                BindingValue::Text(t) => t,
                _ => component.text.clone(),
            };
            commands
                .entity(node)
                .insert((BackgroundColor(palette::BRASS_SELECT), Pickable::IGNORE));
            commands.spawn((
                Text(text),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(palette::BUTTON_FONT),
                    ..default()
                },
                TextColor(palette::DARK_BROWN_TEXT),
                Pickable::IGNORE,
                ChildOf(node),
            ));
        }
        "badge" | "icon" => {
            let path = roles.path(&component.role).map(str::to_owned).or_else(|| {
                if component.icon.is_empty() {
                    None
                } else {
                    Some(format!("assets/ui/icon_{}.png", component.icon))
                }
            });
            if let Some(path) = path {
                let tint = theme
                    .by_name(&component.role)
                    .map_or(palette::GOLD_TEXT, crate::canvas::rgb565_to_color);
                commands.entity(node).insert((
                    ImageNode {
                        image: server.load(path),
                        color: tint,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
        }
        _ => {}
    }
    // action → opaque PickTarget（覆盖 IGNORE：可点击项必须可拾取）。
    // list/panel 的 key 挂在逐行实体上（上面已处理），此处只处理非列表组件。
    if component.kind != "list" && component.kind != "panel" {
        if let Some(key) = host.action_key(&component.action) {
            commands.entity(node).insert((
                crate::pick::PickTarget { key, index: i32::try_from(selected).unwrap_or(0) },
                Pickable::default(),
            ));
        }
    }
}

/// `list`/`panel` 组件内容（从 `spawn_component_content` 抽出，函数规模纪律）。
///
/// S87.6：`row_style=image` = 整图铺满、面板透明（木牌直接挂地图上无暗底）；
/// 其余沿用缩略图行（`build_chrome_row`）。逐行 `PickTarget` 由 `pick_index` 派生。
#[allow(clippy::too_many_arguments)]
fn spawn_list_content(
    commands: &mut Commands,
    font: &Handle<Font>,
    component: &Component,
    host: &dyn ComponentHost,
    node: Entity,
) {
    let BindingValue::Rows(rows) = host.resolve(&component.binding) else {
        return;
    };
    let image_rows = component.row_style == "image";
    let bg = if image_rows {
        palette::TRANSPARENT
    } else if component.kind == "panel" {
        palette::PANEL_WOOD_BG
    } else {
        palette::PANEL_BG
    };
    commands.entity(node).insert((BackgroundColor(bg), Pickable::IGNORE));
    let key = host.action_key(&component.action);
    for row in &rows {
        let row_entity = if image_rows {
            crate::ui::build_image_row(commands, row, node)
        } else {
            build_chrome_row(commands, font, row, node)
        };
        if let (Some(key), Some(index)) = (key, row.pick_index) {
            commands.entity(row_entity).insert(crate::pick::PickTarget { key, index });
        }
    }
}

/// 建**通用组件 schema chrome**：把 [`Component`] 列表按 `z` 升序落成 `bevy_ui` 节点树。
///
/// - 全屏根 `Pickable::IGNORE`（沿用 `build_data_chrome`，不挡下层 1080 拾取）；
/// - `type` 分派见 `spawn_component_content`；
/// - `layout` 意图 → taffy `Node`（全百分比 / Auto，无绝对像素）；
/// - `state` 可见性过滤；`action` → opaque `PickTarget`（key 由宿主注册表派生）。
///
/// 返回根实体（调用方 despawn 即整棵子树消失）。零游戏语义。
#[must_use]
pub fn build_component_chrome(
    commands: &mut Commands,
    server: &AssetServer,
    font: &Handle<Font>,
    components: &[Component],
    roles: &AssetRoles,
    host: &dyn ComponentHost,
    selected: i64,
) -> Option<Entity> {
    if components.is_empty() {
        return None;
    }
    // 全屏根（`%` 布局据此相对视口；根节点铺满屏幕 ⇒ 必须 IGNORE，否则吃掉所有指针事件）。
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                position_type: PositionType::Relative,
                ..Node::default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let mut sorted: Vec<&Component> = components.iter().collect();
    sorted.sort_by_key(|c| c.z);
    for component in sorted {
        if !component_visible(component, selected) {
            continue;
        }
        // 壳侧已滤掉「CPU 1080 层已画」的装饰背景（见 treasure_bevy::build_schema_chrome），
        // 此处只渲染本组件自身。
        let node = commands
            .spawn((component_layout_node(&component.layout, &component.kind), ChildOf(root)))
            .id();
        spawn_component_content(commands, server, font, component, roles, host, node, selected);
    }
    Some(root)
}
