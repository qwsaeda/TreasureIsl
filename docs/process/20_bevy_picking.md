# 20 · bevy_picking 专题（B1：命中/几何全进 Bevy）

> **本文 = `bevy_picking` 用法 + 0.20 API 差异 + 本项目命中事件契约的单一事实源**（从 `12_bevy_usage.md` 拆出独立成篇）。
> UI 层用法看 [`19_bevy_ui.md`](19_bevy_ui.md)；通用约定看 [`12_bevy_usage.md`](12_bevy_usage.md)；B1 裁决看 [`17_layout_engine.md`](17_layout_engine.md)。

## 1. 定位（S29.2）

- 「点哪 = 哪个元素」的判定**全进 Bevy**：手写坐标几何（`ti-app/src/touch.rs` 的 `layout` 模块）**已退役**。
- 命中 = **数据驱动**：`scenes.json` 原语挂 `pick_key`/`pick_index` → Bevy 拾取标记（透明精灵）→ 指针事件 → 归一事件 → 逻辑核心。
- 逻辑核心消费归一 **`HitData{key, index, swipe}`**（`ti_app::game_app`），headless 可测（不依赖活体窗口）。

## 2. 0.20 API 结构（与旧版差异大，勿凭记忆写）

- **没有 `PickingSection`/`PickingHandler`/`PickKind`**（0.20 重构掉）。现行：
  - `MAIN_WORLD` 网格**默认可拾取**；要排除 = 加 `Pickable::IGNORE`。
  - 要「默认不可拾取、显式 opt-in」= `MeshPickingSettings { require_markers: true, .. }` + 相机挂 `MeshPickingCamera` + 目标挂 `Pickable`。
  - 类型：`Pickable`（根组件）、`MeshPickingCamera`/`MeshPickingPlugin`/`MeshPickingSettings`（`mesh_picking` feature）、
    `DefaultPickingPlugins`/`InteractionPlugin`。
  - 悬停态：资源 `HoverMap(PointerId → EntityHashMap<HitData>)` + 实体侧 `Hovered(bool)`/`DirectlyHovered(bool)`。
- 指针事件在 `bevy::picking::events`（均 `EntityEvent` + `Message`）：
  `Pointer{id, target, position}`、`PointerPress{entity, pointer, button, hit, count}`、`PointerRelease{...}`、
  **`PointerClick`**（press+release 同一实体 = tap）、**`PointerDragEnd{entity, pointer, button, distance: Vec2}`**（= swipe）、
  另有 `PointerOver/Enter/Out/Leave/Move/Scroll`。

## 3. 0.20 用 `Message` 取代传统 `Event`（**易错**）

| 旧（≤0.19） | 新（0.20） |
|---|---|
| `#[derive(Event)]` | `#[derive(Message)]` |
| `app.add_event::<M>()` | `app.add_message::<M>()` |
| `EventWriter<M>::send(e)` | `MessageWriter<M>::write(m) -> MessageId` |
| `EventReader<M>::iter()` | `MessageReader<M>::read() -> MessageIterator`（+ `.update()` / `.clear()`） |

- **`add_event`/`EventWriter`/`Event` derive 在 0.20 已移除**（用了就编译不过）。
- 事件类型可同时是 `EntityEvent`（可作 observer 触发器）与 `Message`（可作全局消息载体）——本项目 `PickEvent` 即此形态。

## 4. 本项目命中事件契约

| 件 | 位置 | 职责 |
|---|---|---|
| `PickTarget{key, index}` | `ti-engine::pick` | 挂被拾取实体上的**不透明**目标标记（`key` 语义由 ti-app 解释，本层零游戏语义） |
| `PickEvent{key, index, swipe}` | `ti-engine::pick` | 归一命中事件（`Message`）：`swipe` 0=点按，1=上/2=下/3=左/4=右 |
| `PickPending` | `ti-engine::pick` | 帧内待解析（观察者写 → flush 读） |
| `PickActivity` | `ti-engine::pick` | 帧门控（任一事件发过即置位，帧末复位）——壳侧空白 swipe 兜底据此防双发 |
| `register_pick(app)` | `ti-engine::pick` | init 资源 + `add_message` + 注册观察者 + `PostUpdate (pick_flush, clear_pick_activity).chain()` |
| `swipe_dir_from(distance: Vec2, threshold: f32) -> u8` | `ti-engine::pick` | 位移主轴定方向（位移 < 阈值 = 点按） |
| `spawn_pick_layer(world, &[Primitive]) -> usize` | `ti-engine::scene_render` | 原语 `params["pick_key"]>0` → 透明可拾取标记 + `PickTarget`（`index` = `params["pick_index"]` 或原语 `item`） |
| `HitData{key, index, swipe}` | `ti-app::game_app` | 逻辑核心消费的归一命中（`poll_hit` → `route_hit` → 各场景 `on_hit`） |

## 5. 观察者参数最小化（0.20 `IntoObserver` 限制）

- 观察者签名里带 `Option<&PickTarget>` 之类**可选借用**参数在 0.20 会撞 observer 参数限制（编译不过）。
- **本项目做法**：观察者只带必需参数（`On<PointerClick>` / `On<PointerDragEnd>` + `ResMut<PickPending>`），
  **只记 `e.entity`**（`Entity` 拷贝）进 `PickPending`；由后续 `pick_flush` 系统用 `Query<&PickTarget>` 解析目标并发事件。
  ⇒ 观察者参数最小化 + 目标解析集中在一处（可测）。
- `On<E>` **传值**是 Bevy 0.20 观察者惯用法（registry 内无 `&On<` 实例）⇒ clippy `needless_pass_by_value` 需显式 `#[allow]`。

## 6. 数据化命中（`scenes.json`）

- 原语 `params` 增两键：`pick_key`（u32，`>0` = 可交互；0/缺失 = 不挂）+ `pick_index`（i32，显式下标，缺省 = 原语 `item`）。
- 行定义类原语（无 `item`）须显式给 `pick_index`（例：列表行底框 = 行下标）。
- 命中键约定（ti-app 解释，opaque 键）：`1` = index 罗盘环徽章；`2` = select_role/select_map 列表行；`3` = saveload 槽位；
  `4/5/6/7` = 主地图 HUD（摇骰/买/卖/建造）。**键语义不在 ti-engine 硬编码**（`key` 是不透明数）。
- 门禁：doccheck2 `touch-render-pair` 项守「`scenes.json` 有 `pick_key` + 场景消费 `on_hit` + `touch.rs` 几何退役 + 拾取层在位」。

## 7. 坑位清单（实测）

| # | 坑 | 口径 |
|---|---|---|
| 20-1 | 0.20 无 `PickingSection`/`PickingHandler` | 用 `Pickable` + `MeshPickingSettings` + `MeshPickingCamera` |
| 20-2 | 0.20 `Event` → `Message`（`add_event`/`EventWriter` 已删） | `add_message` + `MessageWriter::write` + `MessageReader::read` |
| 20-3 | 观察者可选借用参数撞 `IntoObserver` 限制 | 观察者只记 `e.entity`，解析下沉到 flush 系统 |
| 20-4 | `On<E>` 传值触发 clippy `needless_pass_by_value` | 显式 `#[allow]`（Bevy 惯用法） |
| 20-5 | 指针事件只在 `bevy::picking::events` | `PointerClick` = tap；`PointerDragEnd.distance` = swipe 位移 |
| 20-6 | 拾取标记 = 透明精灵会被误画 | `Color::srgba(0,0,0,0)` + `PickLayerEntity` 标记随 chrome 重建清理 |
| 20-7 | 空白处 swipe 无实体可派发 | `PickActivity` 帧门控：壳侧兜底 swipe 须看本帧是否已发过实体事件 |

## 8. 相关文档

- [`12_bevy_usage.md`](12_bevy_usage.md)：通用约定（插件组 / 坐标 / z 层 / 离屏截图）
- [`19_bevy_ui.md`](19_bevy_ui.md)：UI 层（自动布局 / resize / `ImageNode`）
- [`17_layout_engine.md`](17_layout_engine.md)：B1 架构 + WBS（S29.2 落点）
- [`18_asset_content_pipeline.md`](18_asset_content_pipeline.md)：「不手写像素」总纲
