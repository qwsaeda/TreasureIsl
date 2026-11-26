# S87 计划 · 世界层可视化编辑（vs / main_map 纳入声明式 schema）

> 用户 2026-11-25 拍板「两个都做（1080 世界层 schema 渲染器）」：把 vs / main_map
> 也纳入可视化编辑——用户复核发现「5 个纯 UI 场景可随意调 UI，但 vs/main_map 不行」，
> 要求补上。延续 S85（声明式组件 schema）+ S86（平台可视化编辑器）的方向。
>
> **硬规则 9（A 纪律）**：本文件 = 设计稿，必须用户过目确认后才落码。S87 全任务 pending。

## 0. 现状诊断

- **S85** 只迁 5 纯 UI 场景（index/select_role/select_map/message/saveload）到 `components_v1.json`
  + bevy_ui 通用渲染器；**vs/main_map 按 B1 签核留 1080**——bevy_ui UI 相机 order=1 叠在 1080 世界之上
  会遮住棋盘/HP/立绘等动态内容（S29.3e 踩坑）。
- **vs/main_map 元素来源**：`scenes.json` 7/16 条游戏世界 prim + `extra`/`ring` 参数 + Rust 过程式绘制
  （`render_hud_v3`/`render_v6_elem`/`render_siege`/`vs_duel`）。不可在平台可视化编辑。
- **S86 编辑器** scene 白名单 = 5 纯 UI 场景 + main_map 预览；不支持世界层元素拖拽。

## 1. 目标（用户诉求）

vs / main_map 的**静态版面元素**也走声明式 schema（数据驱动），可在平台可视化编辑、拖拽改位置、
真渲染预览、导出即生效；**过程式动画/时序**保留 Rust 代码（避免把每帧动画写进数据）。

### 1.1 关键取舍：CPU 世界层渲染（非 bevy_ui）

不把 vs/main_map 迁到 bevy_ui（会遮挡），而是做「**schema → 1080 CPU Canvas**」渲染通道：
schema 描述元素（type/role/世界坐标意图/z/action/binding），CPU 渲染器调用 `ti_render::canvas::Canvas`
的 `fill_rect`/`blit_image`/`draw_text` 原语画到 1080 Framebuf，与世界层同坐标系、无遮挡。
`bevy-cpu-parity` 口径不变（UI Node=0，仅游戏世界对拍）。

**用户 2026-11-25 补充边界**：vs **只数据化静态部分**——出场立绘/HP 槽/按钮/道具栏/回合标识等
可按 schema 声明与可视化编辑；**战斗动画时序（攻击/受击/道具帧、抖动/上飘/命中停顿）保留 Rust 过程式**
（vs_duel/vs_anim_proto），不写进数据。避免把每帧动画塞进 schema 造成数据膨胀与时序失真。

## 2. 影响面（explore 回填 · 第 3 路 渲染器实现/门禁迁移）

> 来源：渲染器实现路径 explore 子代理实测（2026-11-25）。

**架构定案**：世界层 schema 渲染器落 `ti-app/src/scenes/world_schema.rs`（**非 ti-render**——
doccheck2 `render-framework-boundary` 禁 ti-render 依赖 ti-core，放 ti-render 必 FAIL）。
依赖链 `ti-core(world schema) → ti-app(renderer) → ti_render::canvas::Canvas`，零新增依赖、无环。
复用 `HostCanvas`/`with_canvas`（chrome.rs:1181-1251）把 `dyn SceneHost` 桥到 Canvas——
**不可绕过 `RenderHost::fill_rect`**（否则 mock 测试记录 0 调用、编译绿但行为不等价，EXP 教训）。

**可复用 vs 重写**：`BindingValue`/`ComponentHost`/`UiAction` 闭集/`component_visible`/`role→路径`
原样复用；taffy `Node` 布局与 `spawn_component_content` 必须重写为 `placement→Rect(1080)` + Canvas 分派；
颜色走 `Theme::by_name`（禁字面量，engine-no-color-literal）。建议抽「后端无关中间层」
`ResolvedDrawCmd{kind,role,rect,color_token,tier,text,action_key}`，bevy 端映射 ECS、CPU 端映射 Canvas，
语义不重复。

**影子期并存**：新增 `render_world_components` 与 `render_primitives_filtered` **互斥非叠加**，
默认路径不变，`TI_WORLD_SCHEMA=1` 才走新渲染器 ⇒ gate 零空窗；S87.3/4 分别切 main_map/vs 后
旧路径留一轮回退再删。世界层**不挂** `chrome_in_bevy_ui` 守卫（vs/main_map 当前就没有）。

**门禁迁移清单**：`render_intent`（main_map/vs 版面锚 → world schema placement）、`landscape_drawlist`
（vs/main_map 段注释 + 静态框）、`e2e_playability`（text 原语 → world_components）、
`main_map_pick_e2e`（pick_key=4 → schema action="spin" 同批重锚，保「玩不了」守门）、
doccheck2 `scene-layout-fidelity`（建议 scenes.json 几何键保留为权威、world schema 用 world_key 引用 ⇒ 锚点零改）。
`bevy-cpu-parity` **口径不改**（其 example 是固定合成场景，不锚 live vs/main_map；世界 schema 输出 Canvas op
非 bevy_ui Node，天然不入 UI Node 计数）。`render-invariants` 7 场景非黑保留。

**17 号补 §9 签核**：世界层 schema = 1080 CPU 世界层、不入 UI Node 口径、不触发两层遮挡回潮
（世界元素禁声明 bevy_ui placement）、动态数值/动画保留 Rust 过程式画在 schema 层之后。

## 2b. 影响面（explore 回填 · 第 1 路 世界层元素清单）

> 来源：世界层元素清单 explore 子代理实测（2026-11-25）。

**默认生产路径只有两条**：`main_map.rs::render_v6_elem`（2183-2360，读 `main_map_v6_layout_landscape.json`）
+ `vs_anim_proto_landscape.rs::render_v2_elem_landscape`（23-131）；其余 v5/v6_proto/vs_proto/vs_anim_v4
均为环境变量 dormant 原型，不纳入 S87。

**可数据化边界（用户 11-25 确认「vs 静态部分可做：出场/按钮等」）**：
- **main_map 全量 schema 化**：满屏海图底、4 边木框、中央岛 4 拆件、31 地块贴图、船/小旗/徽标、
  舵轮+罗盘、HUD 双铜牌、面板几何、toast/outcome/handoff 盖板（盖板强约束不能局部化）。
- **vs 静态部分**：主题背景框 (44,44,1832,992)、4 层羊皮边框、顶双 HP 状态框 (64,40,880,110)/(976,40,880,110)、
  VS 徽标底 (890,470,140,140)、战报框 (240,720,1440,300)、横幅 (710,80,500,300)、角色基准落点（按 phase 分组）。
- **保留 Rust 过程式**：缆绳 Bresenham、噪点/波纹/暗角、多边形/圆、`clip_text` 逐字截断、
  `tri()` bob、阶段时间轴/兵力插值、受击红闪、道具 FX、缺图回退 `asset_present` 守卫。
- **binding 运行值**：HUD 金币/持有量、骰数、席位徽标/名牌、toast、终局文案、HP/兵力数值、战报行。

**守门不变**：`--shot`/`render_intent`（main_map ≥980‰ / vs ≥850‰ 非黑）、`landscape_drawlist`
（vs bg/双 HP 框、main_map v6 满屏 image）、`bevy-cpu-parity`（合成场景，不锚 live）四项口径保持——
schema 渲染器最终仍走 `RenderHost::draw_target()`→`Canvas`→`DrawList`，产出逐 op 等价序列。

## 2c. 影响面（explore 回填 · 第 2 路 数据契约/命中/绑定）

> 来源：数据契约 explore 子代理实测（2026-11-25）。

**独立文件定案**：新建 `assets/data/ui_schema/world_components.json` + `.schema.json`，不动 `components_v1`
（其 5 场景闭集 + 禁绝对坐标硬约束与世界层不兼容）。type 闭集取 `Rect/Text/Image`（CPU DrawOp 三型），
`grad/glow/panel` 降级为组合或删。字段复用 `id/role/binding/action/state/tier/z/overrides`；新字段：
- `geometry:{source:"main_map_v6_layout_landscape.json",ref:"tiles[3]"}`（v6 JSON 保留为几何单源，world schema 只引用不吸收，遵「不手写像素」）；
- `world_rect:{x,y,w,h}`（仅 world 允许，受 source/ref 约束，禁散落裸像素）；
- `pick:{action,index}` → key 由 `UiAction` 派生（spin=4 不变）；
- `binding:{kind:"host",path:"EconomyHost.gold",fmt:"..."}` → `&dyn SceneHost` 白名单 path + `BindingValue` 扩 Number/Pair/Formatted；
- `visible_when:{path,op,value}`（运行时可见性）；`anim_ref`（代码时间轴名）+ `visible_in_phase[]`。

**命中**：world schema 渲染产出「DrawOp + 合成 Primitive(pick_key)」双输出，喂现有
`manual_1080_hit_system`/`spawn_pick_layer` 纯几何链（`hit_1080_pick`），无需改 UiAction→PickTarget 语义；
pick z 与绘制 z 同源。

**⚠️ 真漂移须处理**：`UiAction` 的 `attack/do_item/defend`=2/3/4 与 vs 实际 on_hit 语义
（2=快进/3=确认/4=重打）不符。建议扩 `vs_skip/vs_confirm/vs_reset`（key 11/12/13）三变体，
button 声明新 action，不改现有 vs 交互；须同步 `components_v1.schema.json` + `ui_schema_check.py` + doccheck2 三处。

**注入**：`InjectedSchema` 加 `world_components` 字段 + `world_components()` loader；`apply_preview_injection`
增一行读 `world_components.json`（S86 预览零写盘闭环）。`table_source`/`map_pack` 三段表不动
（几何差异继续走 layout.json upsert + `tile_<mid>_<i>.png` 命名约定）。

## 3. WBS（已在台账 S87.0~S87.6 全 pending）

| 任务 | 内容 | 判据 |
|---|---|---|
| S87.0 | 立项：调研 + 设计稿 + 用户过目 + 登记 | 用户确认留痕 + doccheck 0 error |
| S87.1 | 世界层 schema 规范（world_components + JSON Schema + 校验器） | 非法字段负例 FAIL |
| S87.2 | 世界层 schema 渲染器（schema→1080 CPU Canvas） | main_map/vs 非黑 + 与旧画面一致 |
| S87.3 | main_map 元素数据化 + 命中 action 接入 | 元素可改数据 + pick 可用 |
| S87.4 | vs 元素数据化 + 动画边界保留代码 | 静态版面可改 + 动效不变 |
| S87.5 | 编辑器扩到 7 场景（世界层可视化 + 预览） | 7 场景可拖拽 + FAIL 禁摆 |
| S87.6 | 门禁/测试/17号文档迁移 + 视觉确认 + 收口推送 | gate 36 项 0 failure + push |

## 4. 判据

- 用户视觉确认（A 纪律）：vs/main_map 世界层元素可视化后画面与改前一致（或按用户新意）
- gate 36 项 0 failure + doccheck/doccheck2 0 error + 台账三方对账
- 7 场景均可在平台可视化编辑（5 纯 UI + 2 游戏世界），FAIL 禁摆，真渲染预览 diff 0

## 5. 风险 / 约束

- **B1 签核变更**：17 号需补「世界层 schema 渲染不属 UI 层、不入 parity UI Node 口径」签核，其余 B1 约定不变。
- **动态数值绑定**：HUD 金币/兵力/HP 是运行时值，schema `binding` 需扩"运行时取值路径"（SceneHost 访问器）。
- **动画边界**：vs 战斗动画是时序驱动，保留 Rust 过程式；只数据化静态版面元素。
- **地图包/主题注入**：world schema 文件是否纳入 `table_source`/`map_pack` 注入链（调研回填后定）。
