# 19 · bevy_ui 专题（B1：UI chrome 进 Bevy · 自动布局 + resize 重排）

> **本文 = `bevy_ui` 用法与 0.20 实测坑位的单一事实源**（从 `12_bevy_usage.md` §9.8 拆出独立成篇，
> 因 S29.1~S29.3 持续增补已超总览承载）。
> 通用约定（插件组 / 坐标契约 / z 层 / 离屏截图 / 性能预算）仍看 [`12_bevy_usage.md`](12_bevy_usage.md)；
> 命中/拾取看 [`20_bevy_picking.md`](20_bevy_picking.md)；B1 架构裁决看 [`17_layout_engine.md`](17_layout_engine.md)。

## 1. 定位（B1 定案）

- **UI chrome**（面板 / 列表 / HUD / 按钮 / 文案）= **Bevy `bevy_ui`**（taffy 自动布局，resize 自动重排）。
- **游戏世界**（背景 / 地形 / 实体 / 地图）= **1080 正交 + 数据驱动**（`scenes.json` → 1080 精灵），不迁 bevy_ui。
- 总纲「**不手写像素**：声明意图/数据，引擎算坐标」见 [`18_asset_content_pipeline.md`](18_asset_content_pipeline.md) §1。

## 2. feature 与插件

- bevy `2d` 特性**不含** UI；须 `Cargo.toml` bevy `features += ["bevy_ui"]`
  （= `bevy_internal/bevy_ui`，核心：布局 / `Node` / `Text` / `UiPlugin`）。
- ⚠️ **`bevy_ui` ≠ `bevy_ui_render`**：**渲染管线是独立特性**
  （`bevy_internal/bevy_ui_render = ["dep:bevy_ui_render", "bevy_sprite_render", "bevy_ui", ...]`），
  且 `DefaultPlugins` 只在 `#[cfg(feature = "bevy_ui_render")]` 下挂 `bevy_ui_render::UiRenderPlugin`。
  **只开 `bevy_ui` 的后果 = 布局跑（`ComputedNode` 有值）但一个像素都不画** —— 极易误判成「相机/合成问题」。
  ⇒ **两个都要开**：`features += ["bevy_ui", "bevy_ui_render"]`。
  （本项目 S29.1~S29.2 只开了 `bevy_ui` ⇒ **UI 层从未渲染过**；S29.3 实测补上后才出现，
  见 §9 坑位 9.8-10/9.8-11。）
- 禁止手搓插件表（见 `12_bevy_usage.md` §2）。

## 3. UI 相机分层（双相机）

```rust
commands.spawn((
    Camera2d,
    IsDefaultUiCamera,                                  // 0.20 必带（否则 bevy_ui 认不到默认 UI 相机）
    RenderLayers::layer(1),                            // S29.3：只画 UI（见 §3.1）
    Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },  // order 高于游戏相机(0) + 不清屏
    Transform::IDENTITY,
));
```
- `order` 高于游戏世界相机（0）+ `clear_color = None` ⇒ 叠在世界上面、不覆盖世界。
- `--shot` 离屏模式：给 UI 相机插 `RenderTarget::Image(同一离屏目标)` ⇒ UI 叠层进截图。
- UI 树走**屏幕像素**（bevy_ui 自动布局）；游戏世界走 1080 正交 `o.scale`。**两套坐标系勿混**。

### 3.1 `RenderLayers` 隔离（必做，否则 UI 相机把世界重画一遍）

UI 相机是**普通 `Camera2d`** ⇒ 它的**世界 pass** 也会画 2D 实体（1080 精灵）：
- 活体窗口：UI 相机按 `scale=1.0`（非游戏相机的 letterbox scale）把世界**再画一遍**并盖在上面 ⇒ 画面错位；
- `--shot` 离屏目标：UI 相机的 pass 会**清掉目标** ⇒ 截图只剩 UI、1080 世界全没。

⇒ **UI 相机必须 `RenderLayers::layer(1)`**（世界实体留在默认层 0）：
`bevy_ui` 的 **UI pass 不受 `RenderLayers` 约束**（`bevy_ui_render` 无 `RenderLayers` 引用），
故 UI 节点**不必**改层，仍在默认层 0 也能被 UI pass 画出；`RenderLayers` 只用于**挡掉 UI 相机的世界 pass**。
（`RenderLayers` 路径 = `bevy::camera::visibility::RenderLayers`，不在 prelude 里，需显式 import。）

## 4. 层级与 Node（0.20 全 inline）

- **没有 `Entity::spawn`/`set_parent`**（0.20 移除）：层级一律 **`ChildOf(parent)`** 组件（prelude 内）。
  `EntityCommands::with_children` 可用，但 `bevy::ecs::entity::EntityCommands` **import 不解析**（E0432）→ 别 import。
- `Node { position_type, left/right/top/bottom/width/height: Val, align_items, justify_content,
  flex_direction, padding: UiRect, border_radius: BorderRadius, row_gap, column_gap, ..default() }`
  - `Val::Px / Val::Percent / Val::Auto`；`UiRect::px(l,r,t,b)`；`BorderRadius::px(tl,tr,br,bl)`。
  - 背景色 = **独立组件 `BackgroundColor`**（**不是** `Node` 字段）。
  - 绝对定位 = `position_type: PositionType::Absolute` + `left/right/top/bottom`（配 `%` ⇒ resize 贴边不漂）。

## 5. 文本

- `Text(String)` + `TextFont { font_size: FontSize::Px(28.0), ..default() }`（`FontSize` 是**枚举** `Px/Vw/Vh/VMin/VMax/Rem`，**不是** `Option<f32>`）。
- 颜色 = `TextColor(Color)`。
- **`GlobalTransform.translation` 是方法**（0.20）：`transform.translation().x`，不是字段。
- ⚠️ **字体必须显式给**：`TextFont.font` 是 **`FontSource` 枚举**（0.20：`FontSource::Handle(Handle<Font>)` /
  `Family(SmolStr)` / `Families` / …），**不是** `Handle<Font>`；默认值 = `Handle::default()`（无字形）
  ⇒ **中文全渲染成豆腐块 □□**（英文照样出，迷惑性极强）。
  正确写法：`TextFont { font: FontSource::Handle(project_font_handle.clone()), font_size: FontSize::Px(20.0), ..default() }`，
  字体句柄用 `ti_engine::canvas::BevyCanvas::load_project_font(&server, "assets/")`（项目 CJK TTF）。
  （`complex_script_segmentation` 特性已开 ⇒ CJK 断词/塑形正常，见 `12_bevy_usage.md` §9.6。）

## 6. UI 图片 = `ImageNode`（0.20 新 API，本轮实测敲定）

> 旧版 Bevy 的 UI 图片组件是 `Image`（`Image::default().with(handle)`）；**0.20 已不适用**：
> `bevy::sprite::Image` 现在是**资源类型**（`bevy_image::Image`，被 `Handle<Image>` 引用），
> UI 里的图片组件叫 **`ImageNode`**（`bevy::ui::ImageNode`，`bevy_ui::widget::image`）。

```rust
use bevy::ui::ImageNode;                       // prelude 亦导出
commands.spawn((
    Node { width: Val::Px(72.0), height: Val::Px(96.0), ..default() },
    ImageNode::new(asset_server.load("assets/ui/role_panda.png")),  // 相对 assets/ 根
    ChildOf(row_node),
));
```
- `ImageNode` 字段：`image: Handle<Image>`、`color`（tint）、`image_mode: NodeImageMode`、`visual_box`、`rect`、`flip_x/y`。
- `#[require(Node, ImageNodeSize)]` ⇒ spawn `ImageNode` 时 `Node` 自动补齐（但**显式写 `Node` 的宽高**才控尺寸）。
- `NodeImageMode`：`Auto`（默认，按源图尺寸 + 布局约束）/ `Stretch`（拉满节点、忽略原比例）/ `Sliced`（九宫格）/ `Repeat`。
  缩略图用 **`Stretch`**（节点定 72×96，图按节点铺）。
- 资源路径相对 `assets/` 根（`AssetServer::load("assets/ui/xxx.png")`），与 1080 精灵路同源（见 `12` §9.5）。

## 7. 自动布局（taffy）意图化范式

- **不写行坐标**：竖排列表 = 面板 `flex_direction: FlexDirection::Column` + `row_gap: Val::Px(..)`；
  行内 = `FlexDirection::Row` + `column_gap`。行位置/高度由 taffy 算，**不手写 `list.y`/`row_h`**。
- **面板定位用 `%`**（`right: Val::Percent(3.0)` + `width: Val::Percent(34.0)`）⇒ resize 自动重排、贴边不漂。
- 选中态 = 行 `BackgroundColor` 换色（黄铜高亮 / 透明），**不另画边框原语**。
- 行内缩略图 = `ImageNode`（§6）；文案 = `Text` + `TextFont` 两级（标题 body 字号 / 副标题 hint 字号）。
- 布局**意图数据与渲染分离**：行内容（图片 handle / 标题 / 副标题 / 选中态）由调用方（壳）组装传入，
  `ti-engine::ui` 侧保持**零游戏语义**（只认「行 = 图 + 两行字 + 选中态」）。

### 7.1 chrome 迁移的三个联动（S29.3 实测口径）

把某段 chrome 从 1080 层迁到 bevy_ui，须**三处同改**（漏一处 = 双画 / 双事件 / 截图缺件）：

| 面 | 做法 |
|---|---|
| **数据** | 原语 `params` 加 `ui_layer: "bevy_ui"` 标记（**不改坐标**）⇒ 已迁 chrome 清单在数据里，可机器判 |
| **1080 渲染** | 场景层渲染时跳过 `ui_layer == "bevy_ui"` 的原语（否则 1080 与 bevy_ui **双画**）；非原语的行底/选中边等代码绘制件也要同条件跳过 |
| **拾取** | 已迁 chrome 的命中**搬到 UI 层**（bevy_ui 节点直接挂 `PickTarget`）；1080 层**不再挂**该原语的拾取标记（`spawn_pick_layer` 跳过 `ui_layer == "bevy_ui"`）⇒ 同一元素只发一次命中事件（双发 = 「点一下进两步」） |

- **坐标系差异**：bevy_ui 在**屏幕像素**空间，1080 拾取标记在 **1080 世界**空间 ⇒ 两者**对不齐**；
  故已迁 chrome 必须整段（含命中）搬到 UI 层，不能「画在 UI、命中留在 1080」。
- CPU 后端（Android macroquad 壳）无 bevy_ui ⇒ 迁移开关须**按后端可关**（本项目 = `GameApp::set_chrome_in_bevy_ui`，默认关）。

### 7.2 参数化径向环（index 6 徽章，S29.3d）

列表/卡片 chrome 用 flex 竖排（`build_list_chrome`/`build_list_chrome_pos`）。**环形菜单**（index 罗盘环 6 徽章）
用**参数化**定位：环心 + 半径 + 60° 步进算 6 格中心 `%`（不逐格手写像素，B1 ③ 参数化档）：

```
angle_i = -90° + i·60°        // 0 = 顶
x% = cx + rx·cos(angle_i)      // cx/rx = 环心/半径 x（`%` 宽）
y% = cy + ry·sin(angle_i)      // cy/ry = 环心/半径 y（`%` 高，屏幕 y 向下）
```

- 环心/半径/格尺寸**走 `LOGICAL_*` 常量**换算（不裸写 1080/1920/540，`bevy-camera-scale-convention`）；
  窗口 resize 时各格按 `%` 跟随视口重排（9:16 默认尺寸下与 1080 环位对齐）。
- 每格 = `ImageNode`（徽章底 + 白蒙版图标，图标 `color` 跨层 tint `gold`/选中 `text_on_wood`）+ 文案；
  **格节点**挂 `PickTarget`（`key=1`），格内**子节点全 `Pickable::IGNORE`**（否则 `PointerClick` 落到子节点
  解析不到 `PickTarget`，见 `20_bevy_picking.md` 20-10）。
- 徽章底图路径 = **读场景数据**（`pick_key=1` 原语 `params.asset`），壳侧不硬编码资产路径（CS-1）。

## 8. 证据口径（headless）

- `--shot` 离屏无法靠像素可靠验证 UI 叠加（UI 相机与 shot 相机同 `RenderTarget::Image` 的合成时序不可靠）
  ⇒ **证据口径 = 跑 N 帧后 `Query<&bevy::ui::ComputedNode>` 计数 > 0**（= taffy 自动布局真跑了、给全部节点算出布局）。
- ti-engine `--shot` 内置 `ui_probe_system` 打：`[ui] bevy_ui probe: node=N text=N abs=N computed_layout=N frame=N`。
- 「自动布局真跑」的判据 = `computed_layout` ≈ `node` 数（每个已布局节点一个 `ComputedNode`）。

## 9. 0.20 坑位清单（实测）

| # | 坑 | 口径 |
|---|---|---|
| 9.8-1 | `2d` 特性不含 UI | bevy `features += ["bevy_ui"]` |
| 9.8-2 | UI 相机须 `IsDefaultUiCamera` + `order: 1` + `clear_color: None` | 缺 `order`/不清屏 ⇒ 覆盖游戏世界 |
| 9.8-3 | `Button`/`Interaction` 已废弃（0.20 标 `#[doc(hidden)]`，post-0.20 删） | 交互走 [`20_bevy_picking.md`](20_bevy_picking.md)；面板只做视觉 |
| 9.8-4 | 无 `Entity::spawn`/`set_parent` | 层级用 `ChildOf(parent)` |
| 9.8-5 | `Node` 字段全 inline；背景色是独立组件 `BackgroundColor`（默认透明 `Color::NONE`） | 勿找 `Node::margin` 子 struct |
| 9.8-6 | `FontSize` 是枚举不是 `Option<f32>` | `FontSize::Px(28.0)` |
| 9.8-7 | `GlobalTransform.translation` 是方法 | `transform.translation().x` |
| 9.8-8 | 离屏像素验证 UI 不可靠 | 证据 = `ComputedNode` 计数探针（**不够**，见 9.8-12） |
| 9.8-9 | **UI 图片组件是 `ImageNode`（不是 `Image`/`Sprite`）** | `ImageNode::new(handle)`；`NodeImageMode::Stretch` 铺满节点 |
| 9.8-10 | **`bevy_ui` ≠ `bevy_ui_render`：只开前者 = 布局跑但零像素**（S29.3 实测） | 两个都开；`DefaultPlugins` 仅在 `bevy_ui_render` 下挂 `UiRenderPlugin` |
| 9.8-11 | **UI 相机必须 `RenderLayers::layer(1)` 隔离**（否则重渲世界/清掉离屏目标） | `bevy::camera::visibility::RenderLayers`（非 prelude）；UI pass 不受该约束 |
| 9.8-12 | **`ComputedNode` 计数 > 0 不等于「画出来了」** | 证据须加**布局尺寸**（`ComputedNode.size`）+ 像素采样（截图取样点）；S29.3 探针已扩：`[ui] bevy_ui layout size: max_node=WxH px` |
| 9.8-13 | **`TextFont.font` 是 `FontSource` 枚举；不给字体 = 中文豆腐块 □□** | `FontSource::Handle(font.clone())` + `load_project_font` |

## 10. 相关文档

- [`12_bevy_usage.md`](12_bevy_usage.md)：通用约定（插件组 / 坐标 / z 层 / 离屏截图 / 预算）
- [`20_bevy_picking.md`](20_bevy_picking.md)：命中/拾取/事件契约
- [`17_layout_engine.md`](17_layout_engine.md)：B1 架构 + WBS
- [`18_asset_content_pipeline.md`](18_asset_content_pipeline.md)：「不手写像素」总纲 + 资产登记
