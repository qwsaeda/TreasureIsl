# 12 Bevy 使用规范（0.20 · 本项目口径）

> **S18.7b：像素基线机制已退役（不再存在）**。
> 依旧的 `assets/render_baseline/` / `forward_baseline/` / `hd_baseline/` 三套 golden PNG、
> `render_snapshot` / `render_snapshot_forward` / `visual_regression` 三个 diff 测试、`visual_baseline.ps1`
> 与其门禁项、`baseline-mirror-identity` 校验均**已全部删除**。
> 现行机制：**结构不变式断言**（`tests/render_invariants.rs`，7 场景 × 非黑覆盖率/内容签名互异/暖色板在位/TTF 在位）
> 与 **CPU↔Bevy 内存对拍**（`bevy-cpu-parity` 门禁项：GPU 真实离屏读回 vs CPU 期望逐像素对照）。
> 本文中仍有的「基线」词句为**历史记述**，不再是现行机制。

**定位**：Bevy = **唯一渲染引擎**（不是纹理显示器）。本文件是 `ti-engine` 里 Bevy 用法的
单一事实源：**怎么搭、怎么画、怎么排层、怎么验证、哪些坑已实测**。

> 迁移裁决见 `11_bevy_migration.md`（差距 / 预算 / 架构形状 / WBS）。
> 本文件是**用法层**：写 Bevy 代码前必读（与 `08_render_arch.md` 渲染契约、
> `09_render_framework.md` 框架边界互锁）。

---

## 1. 版本与特性（pin）

| 项 | 值 | 理由 |
|---|---|---|
| 引擎 | `bevy = "0.20"`（精确 pin，不用 `0.20.*`） | 0.21+ 的 2D/文本 API 破坏性变更多 |
| 依赖方式 | `default-features = false` + `features = ["2d", "bevy_text", "touch", "keyboard", "mouse"]` | 2D + CJK 文本 + 输入；**不要加 audio**（`links=alsa` 冲突，见 §9.1） |
| 渲染后端 | `bevy_render`（wgpu） | GPU 路径；WebGL2 由 wasm 构建时降级 |
| 载体 | `crates/ti-engine`（**独立 workspace**） | `links=alsa` 冲突隔离（§9.1） |

**特性口径实测**（`bevy/Cargo.toml` 0.20）：

- `2d` = `default_app` + `default_platform` + `2d_bevy_render` + `scene` + `picking`
- `default_app` = `async_executor` + `bevy_asset` + `bevy_log` + `bevy_state` + `reflect_auto_register`
  → **不含 audio** ✓（`bevy_audio` 是独立特性，**勿开**）
- `default_platform` = `std` + `bevy_gilrs` + `bevy_winit` + `bevy_clipboard` + `default_font`
  + `multi_threaded` + `webgl2` + `x11` + `wayland` + `custom_cursor` + `sysinfo_plugin`

---

## 2. 插件组（**必须用官方 `DefaultPlugins`，禁止手搓插件表**）

```rust
let mut app = App::new();
app.add_plugins(
    bevy::DefaultPlugins
        // 资产根 = 仓库根（字体 `assets/fonts/...` 直接可加载）
        .set(bevy::asset::AssetPlugin {
            file_path: repo_root().to_string_lossy().to_string(),
            ..Default::default()
        })
        // 同步管线编译：首帧即可出画（异步编译会让早期帧空白/黑）
        .set(bevy::render::RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..Default::default()
        })
        .set(bevy::log::LogPlugin { level: bevy::log::Level::WARN, ..Default::default() })
        .set(bevy::window::WindowPlugin {
            primary_window: Some(window),   // 离屏验证模式用 None
            ..Default::default()
        }),
);
```

**为什么禁手搓插件表**（EXP-102 实测）：手写 `MinimalPlugins` + 手动 `add_plugins` 列表
漏掉 `SpritePlugin` / `SpriteRenderPlugin` / `CameraPlugin` / `CorePipelinePlugin` /
`init_asset::<Shader>()` / `PipelinedRenderingPlugin` → **实体存在但一个像素都没画出来**
（S18.1 spike 的矩形、S18.4 的 CPU 纹理层都是黑的，验收只查 `exit 0` 没看画面 → 缺陷潜伏
了整个 S18.0–S18.4）。**新增 Bevy 渲染能力时一律改 `DefaultPlugins` 的 `.set(...)` 覆写，
不要自己列插件。**

---

## 3. 坐标契约（1080×1920 逻辑 ↔ Bevy 世界）

- 逻辑坐标：`(0,0)` 左上、`y` 向下（`08_render_arch.md` 契约）。
- Bevy 2D 世界：`y` 向上，相机原点居中。
- 换算（**唯一口径**，`scene_render.rs::to_world`）：

  ```rust
  world = (logical_x - 540.0, 960.0 - logical_y)
  ```

- 相机 = `Camera2d` + `Projection::Orthographic(OrthographicProjection { scale, ..default_2d() })`；
  **⚠️ `scale` 语义 = 世界单位/像素**（不是像素/逻辑单位！）——bevy 0.20 实测
  （`OrthographicProjection.update`：`area = scale × 视口 px`；`area` 为 pub 字段，运行时可直读）：

  | 量 | 口径 | 540×960 窗口时的值 |
  |---|---|---|
  | `o.scale`（Bevy 正交） | **世界单位/像素** | `2.0`（= 1/0.5） |
  | `vp.scale()`（shell 视口比 / `Viewport::fit`） | 像素/逻辑单位 | `0.5` |

  ⇒ 二者互为**倒数**：`o.scale = 1 / (vp.scale() × zoom)`（`zoom` 大 = 放大 ⇒ 可见世界收缩）。
  取反的后果（EXP-115）：`area = 270×480` = 画布中央 1/4（4× 放大），窗口变大时内容反而变小。
  判据请读**渲染真值** `o.area`（≈1080×1920 = 画布完整可见），不要只看 `o.scale` 数值。
  窗口尺寸变化时 `window_resize_system` 重建 `vp`，`camera_system` 据此改写 `o.scale`。

---

## 4. z 层（**最易错，务必按此**）

Bevy 0.20 实测（EXP-103）：

| 事实 | 结论 |
|---|---|
| 相机朝 **-Z** 看，默认正交 `near = 0` | **可见实体必须 `z < 0`**（`z > 0` 在相机背后，不渲染） |
| 精灵排序键 = `FloatOrd(transform.translation.z)`，**z 升序绘制** | **z 越大越靠上**（与 CPU 侧「调用序即层序」同义） |

→ 层序映射：**背景最负 → 前景最接近 0**。

| 层 | z | 元素 |
|---|---|---|
| 底 | `-12` | 夜空渐变条带 |
| | `-11` | 星点 |
| | `-10` | 月亮光晕（烘焙纹理） |
| | `-9` | 海平线 |
| | `-8` | 木框外沿 |
| | `-7` | 金色内线 |
| | `-6` / `-5` | 指南针环 / 中心点 |
| | `-4` / `-3` | 标题 / 副标题（Text2d） |
| | `-2` | 菜单卡底 |
| | `-1` | 菜单图标 / 菜单文字 |

**反例（今天真实踩到）**：把背景放 `-1`、前景放 `-12` → 背景盖住全部（画面只剩底色）；
把 CPU 纹理层放 `z=0` → 它在原生实体之上（同样盖住全部）。CPU 纹理层必须 `< -12`。

**层序单一来源**：`ti-app/src/z_layers.rs`（`Z_TERRAIN..Z_OVERLAY` 值域），本表是其在
Bevy 侧的落位；改层序先改 `z_layers.rs` + 数据，不在渲染代码里写字面量。

---

## 5. 元素落地方式（Bevy 原生，不用 CPU 帧缓冲）

| 元素 | 落地 | 备注 |
|---|---|---|
| 实心矩形（底色/框/卡片） | `Sprite::from_color(color, Vec2::new(w,h))` + `Transform` | 渐变 = N 条 Sprite（32 条足够） |
| 文本 | `Text2d::new(String)` + `TextColor` + `TextFont { font_size, font }` | **GPU TTF 图集**（CJK 走 `assets/fonts/lxgw-wenkai-reg.ttf`） |
| 像素级形状（图标/月晕） | **启动时烘焙一次** RGBA 纹理 → `Sprite { image, custom_size }` | 一次性 CPU 几何 → GPU 采样；不每帧烘焙 |
| 图标几何 | `ti_render::icons::icon`（CPU 几何语言）烘焙 48×48 | 复用既有图标语言，不重画 |
| 场景描述 | `scenes.json` 的 `primitives` 表（`kind` + `rect` + `param` + `color` 令牌名） | 数据驱动（`11_bevy_migration.md` §S18.6） |

**禁止**：整帧 CPU 渲染 → 上传纹理（= 把 Bevy 当显示器，付了体积代价却没换来引擎能力）。

---

## 6. 文本规范

- 一律 GPU `Text2d`；**不**用 CPU 位图字体（`glyph_raster` 路径只留 headless 断言用）。
- 字体句柄：`AssetServer::load::<Font>("assets/fonts/lxgw-wenkai-reg.ttf")`（资产根 = 仓库根）。
- 字号 = `FontSize::Px(theme.<tier>_font_size)`（令牌，不写死数值）。
- 文本度量（布局用）以 `ti-render::TtfFont` 为准（与 `MenuShell` 同源），Bevy 侧
  `TextLayoutInfo` 仅用于 GPU 对照（`S18.3` 比例 ∈ [0.80,1.30]）。

---

## 7. 离屏截图 / 视觉验证（S18.7 起唯一口径）

```rust
// ① 离屏目标（1080×1920，RENDER_WORLD 供相机渲染）
let mut target = Image::new_target_texture(1080, 1920, TextureFormat::Rgba8UnormSrgb, None);
target.asset_usage = RenderAssetUsages::RENDER_WORLD;
let handle = app.world_mut().resource_mut::<Assets<Image>>().add(target);
app.world_mut().spawn((Camera2d, RenderTarget::Image(handle.into()), Transform::IDENTITY));

// ② 手动 update 循环（不 app.run()）：finish() 必做，否则渲染 app 的 extract 调度没起来
app.finish();
app.cleanup();
for _ in 0..60 {
    app.update();
    if let Some(dev) = app.world().get_resource::<RenderDevice>() {
        let _ = dev.wgpu_device().poll(PollType::Wait { submission_index: None, timeout: None });
    }
    if readback_has_content() { break; }   // 判据 = 非黑像素 > 0
}
```

**必守三条**（今天全部踩到）：

1. `app.finish()` + `app.cleanup()` **不可省** —— 缺它 → extract 阶段 panic
   `resource does not exist: RenderAppChannels`。
2. 截图**异步**：observer 回调才拿得到数据；必须 `poll` 设备。
3. **判据 = 非黑像素 > 0**（不是"图片存在"）：全黑 = 没渲染出来，不是"画面是黑的"。

**第 4 条（EXP-104：采样点标签骗人）**：

4. **判据顺序 = ① 先看图（`read` PNG 肉眼）→ ② 再看计数型判据（非黑像素比）→ ③ 最后才看采样点颜色**。
   - 采样点**必须标注真实坐标**（如「左上角 (0,0)」「中心 (540,960)」）；旧实现把
     `raw.len()/4/2`（= 第 960 行**第 0 列** = 左边缘）打印成「中点」→ 读者会把
     「木框色」误读成「整屏铺满木框色」，进而**误改 3 处本来正确的渲染代码**。
   - **禁止**由单个采样点推断整屏结论；四角/中心的颜色**本来就该不同**
     （边框 ≠ 天空）。
   - 改渲染前**必须先读实现**（绘制列表 → 原语执行器 → 宿主桥接 → trait 默认实现
     四层），连改多处再验证 = 返工。

---

## 8. 输入规范

| 输入 | Bevy 侧 | 游戏侧 |
|---|---|---|
| 键盘 | `ButtonInput<KeyCode>` | `GameApp::poll(&KeyState)`（键位映射在壳层，与 macroquad 壳同表） |
| 鼠标移动 | `AccumulatedMouseMotion` | 相机 pan（`WorldCam::pan`） |
| 鼠标按键 | `ButtonInput<MouseButton>` | 菜单选择 / 拖拽 |
| 触摸 | `Touches`（`touch.position()`） | `TouchPad::feed_screen`（S18.1 结论：winit 触摸走指针事件） |
| 滚轮 | `MotionEvent`（`bevy::winit::event`） | 相机 zoom（**当前未接**，见 §9.4） |

**一帧顺序**：`input_system` → `camera_system` → `render_system` → `native_index_system`
（独占系统，需 `&mut World` 建/销实体）。

---

## 9. 已知坑位清单（EXP-102 / EXP-103 实测）

### 9.1 `links = "alsa"` 冲突
`bevy_dev_tools` → `bevy_audio` → `rodio` → `cpal` → `alsa-sys` 与 macroquad →
`quad-alsa-sys` 同 link 名。cargo 的 links 冲突检查 **target-independent 且 optional 依赖
也进 resolution 图** → **ti-app 任何形态都不能持 bevy 依赖** → 双 alsa 链只能靠
**独立 workspace** 隔离（`ti-engine` 自带 `[workspace]` 表）。

### 9.2 手搓插件表 → 实体在、画不出（见 §2）
漏 `SpritePlugin`/`SpriteRenderPlugin`/`CameraPlugin`/`CorePipelinePlugin`/`Shader` 资产初始化。

### 9.3 z 符号与排序（见 §4）
`z > 0` 在相机背后；z 升序绘制 = 越大越靠上；背景必须最负。

### 9.4 已知未接
- **滚轮 zoom**：`MotionEvent` 未接（键盘 `=`/`-` 临时替代）→ S18.8。
- **其余 6 场景**仍走 CPU 纹理路径（index 已 Bevy 原生）→ S18.7 逐场景迁移。
- **CPU 帧缓冲执行器**（`ti-render/src/framebuffer.rs` + `Canvas` trait + 像素基线）待删 →
  S18.8（macroquad 退役同轮删净）。

### 9.5 资源命名/路径
- 资产根 = 仓库根（`AssetPlugin.file_path`），字体路径恒为 `assets/fonts/...`。
- wasm 构建时 `file_path` 不可用（浏览器无文件系统）→ wasm 侧走 `assets/` 相对 URL
  （S18.8 资源管线同轮处理）。

### 9.6 CJK 文本每帧刷 ICU4X 错（EXP-119，S26.1 实测）
bevy 0.20 文本走 parley → icu_segmenter：**未开 `complex_script_segmentation` feature 时
回退 `new_southeast_asian()` 字典（仅 My/Km/Lo/Th），CJK 词元分词逐词未命中** ⇒ debug 态
每帧打 `ICU4X data error: No segmentation model for language: ja`（`ja` = CJK 字典键，zh 亦
走此键；每场景 ~24 行）。**CJK 文本应用必开**：bevy `features += ["complex_script_segmentation"]`
（→ parley/complex-scripts → 完整 ICU 字典含 cjdict）。判据 = 跑任一实机命令 grep
`No segmentation model` = 0 行。

### 9.7 `src/bin/*.rs` 子模块互引 = 隐式第 3 bin（EXP-120）
cargo auto-roots 把 `src/bin/` 每个 `.rs` 都探测为独立 bin；当某 bin 文件同时被另一 bin
以 `mod` 引用（如 `treasure_bevy` 的 `mod resizetest;`）时，`--all-targets` 编译到它即爆
E0433（too many `super`）+ E0601（缺 main）。**bin 文件互引子模块的布局必须
`[package] autobins = false`**（ti-engine 已配；gate 只编译声明 bin 时该坑潜伏）。

### 9.8 bevy_ui 启用（S29.1，B1 定案）—— 0.20 实测坑位
B1（`docs/process/17_layout_engine.md`）把 UI+命中+重排全进 Bevy。启用 `bevy_ui` 在 0.20 实测：

- **feature 必开**：bevy `2d` 特性**不含** UI；`Cargo.toml` bevy `features += ["bevy_ui"]`
  （=`bevy_internal/bevy_ui`，连带 `bevy_ui` core + `bevy_ui_render` 渲染管线，都在
  `DefaultPlugins` 内自动注册）。缺 feature → 无 `Node`/`Text`/`UiPlugin`，代码不过。
- **UI 相机 = `IsDefaultUiCamera`**：`commands.spawn((Camera2d, IsDefaultUiCamera,
  Camera { order: 1, clear_color: ClearColorConfig::None, ..default() }, Transform::IDENTITY))`。
  `order` 高于游戏世界相机（0）+ `clear None` 才叠在世界上面不覆盖世界。
- **`Button`/`Interaction` 已废弃**（0.20 标 `#[doc(hidden)]`，post-0.20 删）：交互一律走
  **`bevy_picking`**（S29.2）；证明面板只 spawn「按钮外观」（`Node`+`Text`+`BackgroundColor`），
  **不**挂 `Button` 标记。
- **没有 `Entity::spawn`/`set_parent`**（0.20 移除）：层级一律 **`ChildOf(parent)` 组件**
  （`bevy::hierarchy`，在 prelude）；`EntityCommands` 的 `with_children` 可用但
  `bevy::ecs::entity::EntityCommands` import 不解析（E0432）→ 别 import，用 `commands.spawn` 即可。
- **`Node` 字段全 inline**（0.20 重构，无 `margin/padding` 子 struct 名变化）：`Node {
  position_type, left/right/top/bottom/width/height: Val, align_items, justify_content,
  padding: UiRect, border_radius: BorderRadius, row_gap, ..default() }`；`Val::Px/Percent/Auto`；
  `UiRect::px(l,r,t,b)`；`BorderRadius::px(tl,tr,br,bl)`。背景色 = **独立组件 `BackgroundColor`**
  （不是 `Node` 字段）。
- **`Text`/`TextFont`**：`TextFont { font_size: FontSize::Px(28.0), ..default() }`（`FontSize`
  是枚举 `Px/Vw/Vh/VMin/VMax/Rem`，**不是** `Option<f32>`）；`Text::from_label` 简写可用。
- **`GlobalTransform.translation` 是方法**（0.20）：`transform.translation().x`，不是字段。
- **headless 布局证据 = `ComputedNode` 探针**：bevy_ui 布局由 taffy 算出
  `bevy::ui::ComputedNode`（每已布局节点一个）。`--shot` 离屏无法靠像素验证 UI 叠加
  （UI 相机与 shot 相机同 `RenderTarget::Image` 的合成时序不可靠）→ **证据口径 = 跑 N 帧后
  `Query<&ComputedNode>` 计数 > 0**（= 自动布局真跑了）。ti-engine `--shot` 内置
  `ui_probe_system` 打 `[ui] bevy_ui probe: node=N text=N abs=N computed_layout=N`。

---

## 10. 性能与体积预算

| 项 | 口径 | 现状 |
|---|---|---|
| wasm 体积 | 主体 ≤8MB（strip + LTO + wasm-opt，S18.6 落地） | S18.4 未 strip 39MB → S18.6 收 |
| 实体数 | 首界面 ~130（116 Sprite + 8 Text2d + 2 Camera + 图标纹理 6） | ✅ 远低于预算 |
| 每帧上传 | **0 字节**（Bevy 原生 = 无整帧纹理上传） | ✅ index 已达成 |
| 纹理 | 图标 48×48×6 烘焙一次 + 渐变条带 | ✅ |
| draw call | Sprite 批处理（2D 批） | 渐变条带 32 个 Sprite = 32 批内 drawcall ✓ |

---

## 11. 代码组织（本项目口径）

```
crates/ti-engine/
  src/canvas.rs        = BevyCanvas（op 记录器；headless 契约测试用）
  src/scene_render.rs   = **Bevy 原生场景渲染器**（primitives → 实体；本文件规范的落地实现）
  src/bin/treasure_bevy.rs = Bevy 壳（native/wasm 同源；`--shot` 离屏验证 / `--selftest` e2e）
crates/ti-render/       = 渲染**资源与度量**（theme 令牌 / TTF 度量 / 图标几何 / PNG 编解码）
                          ✗ 帧缓冲执行器 + Canvas trait 随 S18.8 删
```

**职责边界**：`ti-render` 只留「与后端无关的资源与度量」；`ti-engine` 独占渲染实现；
`ti-app` 只产出场景描述（`scenes.json` 原语表）+ 游戏状态。

---

## 12. 验收口径（Bevy 侧）

| 验收 | 手段 | 判据 |
|---|---|---|
| 编译 | `cargo build -p ti-engine`（两 workspace） | exit 0 |
| 契约 | `crates/ti-engine/tests/bevy_native_render.rs` | 实体数 / 颜色 / 图标纹理 = 3 passed |
| 出画 | `treasure-bevy-native --shot` | **非黑像素 > 0**（今天口径：2072562/2073600） |
| e2e | `treasure-bevy-native --selftest` | 16/16 PASS |
| 门禁 | `scripts/gate.ps1` | 0 failure |

⛔ **只查 `exit 0` 不看画面 = 没验收**（S18.1–S18.4 的教训：黑屏也算过）。

## 13. 通用原语闭集（S18.7a：数据驱动的真正边界）

渲染层**只认 6 种通用原语**；界面 = 这些原语的**数据组合**（`scenes.json` 的
`primitives` 数组）。**场景语义（"菜单""木框"）只存在于数据，渲染层不认识。**

| kind | 语义 | Bevy 落地 | CPU 落地 |
|---|---|---|---|
| `rect` | 实心矩形 | `Sprite::from_color` | `fill_rect` |
| `text` | 文本（`tier` = title/body/hint） | `Text2d` + GPU TTF | `draw_*_text` |
| `icon` | 图标（`params.icon` = 图标名） | 烘焙纹理 → `Sprite` | `icons::icon` |
| `grad` | 垂直渐变（`params.from_token`/`to_token`/`steps`） | N 条 `Sprite` | N 条 `fill_rect` |
| `glow` | 径向光晕 | 烘焙径向纹理 → `Sprite` | 逐行扫描线 |
| `disc` | 实心圆盘 | 烘焙圆形纹理 → `Sprite` | 逐行扫描线 |

**原语字段**（`ti-core::data::Primitive`）：

| 字段 | 语义 |
|---|---|
| `kind` | 六种之一（闭集） |
| `rect` | `x`/`y`/`w`/`h`（`glow`/`disc` = 圆心 + 半径） |
| `params` | 原语专属参数（数值/字符串皆可） |
| `color` | **theme 令牌名**（渲染层零裸色值） |
| `text` / `text_key` | 文本内容 / i18n key |
| `tier` | 文本档位（title/body/hint） |
| `z` | 层序（**负值**；越大越靠上 = 后画者在上） |
| `when` / `item` | 选中态过滤（`selected`/`normal` + 菜单项序号）→ **选中态也数据化** |

### 13.1 如何证明数据分离（可机器判定，非口号）

| 判据 | 落地 |
|---|---|
| ① 改数据能改画面，渲染代码 0 改动 | 对照实验（改 `scenes.json` → 画面变 → `git diff` 渲染层 = 空） |
| ② 加视觉元素 = 只加 JSON | 新增效果 = 追加 `primitives` 项 |
| ③ 渲染层无场景语义 | **gate 第 24 项 `check_data_driven`**：grep 渲染层禁出现场景名/场景专属 kind；kind 分支 ⊆ 六原语 |
| ④ 令牌拼错会静默失真 | 同门禁：数据侧引用的令牌必须存在于 theme（防"令牌不存在 → 渲染器回退"） |
| ⑤ 两后端同源 | 同一份 `primitives` → CPU 执行器（`ti-render::generic`）与 Bevy 执行器（`ti_engine::scene_render`）出同一画面 |

**判据 ③ 的实现**：`scripts/check_data_driven.py`（gate 已挂）。

### 13.2 关键纪律（踩坑沉淀）

- **令牌必须存在**：拼错 → 渲染器回退默认色 → 画面静默失真（实测 `sky_horizon` 不存在 → 海面变米白）。
  → 门禁 ④ 守住。
- **`params` 值类型**：曾用 `BTreeMap<String, i64>`，图标名（字符串）解析失败 → **整条原语被丢弃**
  （serde 失败 ≠ 字段缺失）→ 改 `serde_json::Value` 通用。
- **8pt 对齐脚本别吃细线**：`round(v/8)*8` 会把 1–4px 装饰线清零（`round(0.5)=0`）→
  位置 8pt、装饰尺寸保留原值（门禁豁免 `primitives` 子树）。
- **选中态也是数据**：`when` + `item` → 不用在渲染代码里写"如果第 0 项被选中就画金色卡"。
