//! 触屏命中层：Bevy 拾取（`bevy_picking`）→ 归一 [`PickEvent`]（S29.2，B1 定案 `17_layout_engine.md`）。
//!
//! 设计（零游戏语义，`doccheck2` 框架边界守）：
//! - [`PickTarget`] 组件挂在 chrome/世界实体上（`scene_render` 从 `scenes.json` 原语 `pick_key`
//!   数据驱动挂，opaque 拷贝；`key` 语义由 ti-app `route_hit` 解释，本层不懂）。
//! - 指针交互走 `bevy_picking` 指针事件：**观察者只记录被交互实体**（`e.entity`）到 [`PickPending`]，
//!   再由 [`pick_flush`] 系统解析 [`PickTarget`] 并发 [`PickEvent`]（观察者参数最小化，绕开 0.20
//!   观察者系统参数限制）。
//!   - `PointerClick`（press+release 同一实体 = tap）→ `PickEvent{ key, index, swipe: 0 }`
//!   - `PointerDragEnd`（拖拽 = swipe）→ 按 `distance` 主轴发 `PickEvent{ key, index, swipe: dir }`
//! - 空白 swipe（`vs`/`saveload`/`message`/`select_*` 下滑=返回/翻页常发生在无实体处）由壳侧
//!   原始指针兜底：本层暴露 [`PickActivity`] 门控（本帧是否已发过事件），壳只在其为 false 时
//!   补发空白 `PickEvent{ key: 0 }`，避免双发。
//!
//! 命中/几何全在 Bevy 侧（B1 §6 风险表：拾取几何本身 Bevy 侧，接受）；逻辑核心仍消费
//! [`PickEvent`]（事件→逻辑 headless 可测，ti-app `route_hit`）。

use bevy::ecs::message::{Message, MessageWriter};
use bevy::picking::events::{PointerClick, PointerDragEnd};
use bevy::prelude::*;

/// 目标键：无目标 / 空白（壳兜底空白 swipe 用；`route_hit` 按场景解释 0 = 无元素命中）。
pub const PICK_KEY_NONE: u32 = 0;

/// 拾取标记实体标记（`scene_render::spawn_pick_layer` 挂；壳在 chrome 重建时按此标记清理旧标记）。
#[derive(Component)]
pub struct PickLayerEntity;

/// Opaque 目标标识（渲染层只拷贝，语义归 ti-app）。
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PickTarget {
    /// 目标族键（数据 `pick_key`；0 = 未标注）。
    pub key: u32,
    /// 目标项下标（原语 `item`；-1 = 无）。
    pub index: i32,
}

/// swipe 方向编码（[`PickEvent::swipe`]；0 = 非 swipe = tap）。
pub const SWIPE_NONE: u8 = 0;
/// 向上。
pub const SWIPE_UP: u8 = 1;
/// 向下。
pub const SWIPE_DOWN: u8 = 2;
/// 向左。
pub const SWIPE_LEFT: u8 = 3;
/// 向右。
pub const SWIPE_RIGHT: u8 = 4;

/// 一次完成的指针交互（tap / swipe）。
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickEvent {
    /// 目标族键（[`PICK_KEY_NONE`] = 无元素命中 / 空白）。
    pub key: u32,
    /// 目标项下标（-1 = 无）。
    pub index: i32,
    /// [`SWIPE_NONE`]=点按；否则 = 滑动主轴方向。
    pub swipe: u8,
}

/// 本帧待解析的指针交互（观察者写、[`pick_flush`] 读）。
#[derive(Resource, Default, Debug)]
pub struct PickPending {
    /// 本帧点按（press+release 同一实体）的实体。
    clicks: Vec<Entity>,
    /// 本帧拖拽结束（被拖实体 + 位移，屏幕像素）。
    drags: Vec<(Entity, Vec2)>,
}

impl PickPending {
    /// 记录一次点按实体。
    pub fn push_click(&mut self, entity: Entity) {
        self.clicks.push(entity);
    }

    /// 记录一次拖拽结束（被拖实体 + 位移）。
    pub fn push_drag(&mut self, entity: Entity, distance: Vec2) {
        self.drags.push((entity, distance));
    }
}

/// 本帧拾取活动门控：任一 [`pick_flush`] 发过 [`PickEvent`] 置 true（[`clear_pick_activity`] 复位）。
///
/// 壳侧空白 swipe 兜底据此避免与实体事件双发。
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickActivity(bool);

impl PickActivity {
    /// 标记本帧已发过拾取事件。
    pub fn mark(&mut self) {
        self.0 = true;
    }

    /// 本帧是否已发过。
    #[must_use]
    pub fn active(self) -> bool {
        self.0
    }
}

/// 帧末复位门控。
pub fn clear_pick_activity(mut act: ResMut<PickActivity>) {
    act.0 = false;
}

/// 按 `distance`（屏幕像素，start→end）主轴定 swipe 方向；位移 < 阈值 = 点按（返回 [`SWIPE_NONE`]）。
#[must_use]
pub fn swipe_dir_from(distance: Vec2, threshold: f32) -> u8 {
    if distance.length() < threshold {
        return SWIPE_NONE;
    }
    if distance.abs().x >= distance.abs().y {
        if distance.x > 0.0 {
            SWIPE_RIGHT
        } else {
            SWIPE_LEFT
        }
    } else if distance.y > 0.0 {
        // 屏幕像素 y 向下为正 = 游戏 1080 坐标 y 向下 = 下滑。
        SWIPE_DOWN
    } else {
        SWIPE_UP
    }
}

/// 注册拾取层：门控 + pending 资源 + [`PickEvent`]（message）+ 观察者 + flush 系统。
pub fn register_pick(app: &mut App) {
    app.init_resource::<PickActivity>()
        .init_resource::<PickPending>()
        .add_message::<PickEvent>()
        .add_observer(pick_on_click)
        .add_observer(pick_on_drag_end)
        .add_systems(
            PostUpdate,
            (pick_flush, clear_pick_activity).chain(),
        );
}

/// 观察者：点按（press+release 同一可拾取实体）→ 记实体进 [`PickPending`]。
/// `On<E>` 传值 = Bevy 0.20 观察者系统惯用法（非 `&On`），抑制「未消费」误报。
#[allow(clippy::needless_pass_by_value)]
fn pick_on_click(ev: On<PointerClick>, mut pending: ResMut<PickPending>) {
    pending.push_click(ev.entity);
}

/// 观察者：拖拽结束（被拖实体）→ 记实体 + 位移进 [`PickPending`]。
#[allow(clippy::needless_pass_by_value)]
fn pick_on_drag_end(ev: On<PointerDragEnd>, mut pending: ResMut<PickPending>) {
    pending.push_drag(ev.entity, ev.distance);
}

/// 帧内解析：把 [`PickPending`] 的交互实体解出 [`PickTarget`] 并发 [`PickEvent`]（排空 pending）。
fn pick_flush(
    mut pending: ResMut<PickPending>,
    targets: Query<&PickTarget>,
    mut act: ResMut<PickActivity>,
    mut events: MessageWriter<PickEvent>,
) {
    for &entity in &pending.clicks {
        let Ok(t) = targets.get(entity) else {
            continue;
        };
        act.mark();
        events.write(PickEvent { key: t.key, index: t.index, swipe: SWIPE_NONE });
    }
    for &(entity, distance) in &pending.drags {
        let dir = swipe_dir_from(distance, 20.0);
        if dir == SWIPE_NONE {
            continue;
        }
        let t = targets.get(entity).copied().unwrap_or_default();
        act.mark();
        events.write(PickEvent { key: t.key, index: t.index, swipe: dir });
    }
    *pending = PickPending::default();
}
