# 17 · 布局引擎定案（S29：B1 全 bevy_ui 响应式 —— UI + 命中 + 重排全进 Bevy）

> **本文件 = 「版面/命中/响应式写在哪、怎么换引擎」的单一事实源**。
> 触发：用户 2026-10-27「有引擎却不用，布局还是写死的」→ 反复强调**真响应式** → 最终拍板
> **B1 = Bevy 引擎自带响应式（`bevy_ui` 自动布局 + 窗口 resize + `bevy_picking`），不自造求解器**。
>
> **定案推翻签核（重要）**：本 S29 曾一度定案「引擎无关布局求解器」（`ti-render::layout` + 布局树 +
> 重排 DSL），**该方向已作废**（`crates/ti-render/src/layout.rs` 及其 lib.rs 接线为遗留物，
> 由 S29.0-void 任务删除）。推翻原因：用户明确要「引擎自带」而非自研求解器。
>
> **3 条既有约定随 B1 松绑（签核留档，后续同步 `11_bevy_migration.md` / `12_bevy_usage.md` /
> `08_render_arch.md` / `09_render_framework.md`）**：
> ① 逻辑层 bevy-free → **仅 UI 交互层**（命中/拾取/hover）进 Bevy，游戏逻辑核心（战斗/经济/存档/状态机）仍 bevy-free；
> ② 双渲染像素对拍（`bevy-cpu-parity`）→ **改口径**：游戏世界渲染仍 CPU↔Bevy 对拍，UI 层 Bevy-only 无 CPU 对拍；
> ③ 场景 = 数据（`scenes.json` 单一事实源）→ **UI chrome 转 Bevy 代码**（`bevy_ui` 组件），游戏内容数据（board/terrain/balance）仍数据驱动。
>
> 关联文档：`11_bevy_migration.md`（引擎裁决）/ `12_bevy_usage.md`（Bevy 用法与不用清单，S29 起改）/
> `08_render_arch.md`（双渲染一致性，UI 层松绑）/ `business/08_layout_design.md`（**保留为游戏内容 1080 坐标表**；UI chrome 坐标改由 bevy_ui 自动布局）。

## 1. B1 架构（UI + 命中 + 重排全进 Bevy）

| 层 | 现状 | B1 后 |
|---|---|---|
| UI / chrome（菜单·面板·按钮·HUD·提示签） | `scenes.json` 手写绝对坐标 + `ti-render` 绘制 | **`bevy_ui`** `Node` + `Layout`（flexbox）自动布局，声明式建 UI 树 |
| 命中 / 交互（点击·选中·hover） | `touch.rs` 手写几何函数 + 命中优先级表 | **`bevy_picking`** 2D 拾取（游戏精灵）+ `bevy_ui` `Interaction`/Button（UI 控件） |
| 响应式 / 重排 | 1080 锁死 + letterbox 拉边 | `bevy_ui` **自动重排** + `WindowResize` 事件实时生效（真响应式） |
| 游戏世界（地图·环岛棋盘·精灵·战斗） | 1080 正交相机 + 数据驱动 | **不变**（1080 正交 + board.json 等数据驱动；仍换引擎友好） |
| 文本 / 图片 | 1080 字号 + 固定 sprite | **等比缩放**（bevy_ui 文本随视口；游戏世界 sprite 走相机 `o.scale`） |

**要点**：「响应式」主要作用在 **UI 层**（bevy_ui 自动布局 + resize）；**游戏世界**保持 1080 数据驱动
（它是游戏内容不是 UI，用数据驱动坐标最合适，且保留换引擎余地）。命中：UI 控件走 bevy_ui 内建交互，
游戏内容（格子/节点牌/船/中央据点岛）走 `bevy_picking` 对 1080 正交相机的 2D 拾取。

## 2. 3 条约定松绑的具体后果（已签核）

1. **逻辑层 bevy-free（部分松绑）**：`ti-app` 的场景流/战斗/经济/存档/状态机**仍 bevy-free**（可 headless、
   可换引擎）。松绑的只是「UI 交互层」：命中/拾取/hover 逻辑进 Bevy 侧（`bevy_picking` + `bevy_ui::Interaction`），
   经**事件**（`Picked` / `UiAction`）回传给逻辑核心。逻辑核心仍只消费事件，不依赖 Bevy。
   ⇒ 换引擎成本：**上升**（UI 交互层要随引擎重写），游戏逻辑核心不受影响。
2. **双渲染像素对拍（改口径）**：`bevy-cpu-parity` 门禁**只保游戏世界**（1080 正交精灵/地形/棋盘）
   CPU↔Bevy 像素对拍；**UI 层（bevy_ui）Bevy-only**，不做 CPU 对拍（CPU 帧缓冲渲染不了 bevy_ui）。
   ⇒ 门禁项 `bevy-cpu-parity` 加「UI 层排除」口径；CPU 参考后端继续充当游戏世界对拍锚。
3. **场景 = 数据（部分松绑）**：`scenes.json` 继续承载**游戏内容数据**（棋盘格/节点/精灵引用），
   但 **UI chrome 的版面坐标改由 `bevy_ui` 组件代码定义**（不再在 `scenes.json` 手抄 x/y/w/h）。
   ⇒ 「改 UI 版面不改码」对 UI 层不再成立（UI 版面 = Bevy 代码）；游戏内容数据仍「改数据不改码」。
   ⇒ `doccheck2::check_scene_narrative`（N8 bbox 手抄对账）等断言**改口径或退役**（S29 门禁升级任务）。

## 3. 「换引擎」成本（B1 后的诚实评估）

| 资产 | 换引擎（Bevy → 任意 2D 渲染器）是否存活 |
|---|---|
| 游戏内容数据（board/terrain/balance/strings）+ 游戏世界 1080 渲染 | ✅ 存活 |
| 游戏逻辑核心（ti-app 战斗/经济/存档/状态机，bevy-free） | ✅ 存活 |
| **UI 层（bevy_ui 组件树 + 自动布局 + resize 重排）** | ❌ 重写（绑 Bevy） |
| **命中/拾取层（bevy_picking + bevy_ui Interaction）** | ❌ 重写（绑 Bevy） |

结论：**B1 把「UI + 命中 + 重排」的收益锁进 Bevy，换取「换引擎更贵」**。这是用户 2026-10-27 拍板的
**权衡**（最大化引擎便利 > 换引擎余地）。若未来真要换引擎，UI/命中层重写是既定成本。

### 3.1 实测回填（S29.1~S29.5，B1 落地后的真实成本面）

> 单一事实源 = 本节（S29.5 收口回填）。**迁移面比原计划小，但「换引擎更贵」的结论被实测放大**。

| 资产 | 实测状态（S29.5 收口） | 换引擎是否存活 |
|---|---|---|
| 游戏内容数据（board/terrain/balance/strings） | 不变 | ✅ 存活 |
| 游戏世界 1080 渲染（`DrawList` → Bevy 实体） | 不变（S29.3 未触碰） | ✅ 存活 |
| 游戏逻辑核心（ti-app 战斗/经济/存档/状态机） | 不变（bevy-free） | ✅ 存活 |
| **UI chrome（5 纯 UI 场景）** | `bevy_ui` 组件树（`Node`/`Text`/`ImageNode` + taffy 自动布局） | ❌ 重写 |
| **命中/拾取** | `bevy_picking`（UI 层命中 + 1080 层**手动几何**命中） | ❌ 重写 |
| **UI 命中的几何** | 已迁 chrome 的命中**只在 UI 层**（`PickTarget` 挂 UI 节点）；1080 层另走纯几何 `hit_1080_pick` | ❌ 重写（两层各一套） |

**实测发现（换引擎时必须重新解决的具体坑）**：

1. **Bevy 特性分层坑**：`bevy_ui` 只含布局核心，渲染管线在独立特性 `bevy_ui_render` ⇒ 只开前者
   = 布局跑但**零像素**（S29.1~S29.2 白跑）。换引擎时要问「UI 库是否自带渲染管线」。
2. **多相机静默失效**：`Camera2d` 超过 1 台后 `Query::single()`/`single_mut()` **多实体失败且静默** ⇒
   正交 scale 不再更新 ⇒ 窗口缩放跟随失效（S29.1~S29.4 潜伏，S29.5 由 `resizetest` 判据抓出）。
   换引擎时「相机/画布数量会变」的场合必须显式标记目标。
3. **两层叠压顺序 = 契约**：UI 层叠在 1080 世界层之上 ⇒ 同屏位 UI 节点会**遮住** 1080 动态内容
   ⇒ chrome 与 world 层叠的场景（vs 血条 / main_map 棋盘 + HUD 数值）**不能只迁 chrome**
   （`19_bevy_ui.md` 9.8-13）。这条把 B1 的可迁移面从「9 场景」收敛为「**5 纯 UI 场景**」。
4. **拾取后端覆盖面**：`bevy_picking` 的 mesh 射线只拾取 `Mesh2d`/`Mesh3d`/`SimplifiedMesh` ⇒
   `Sprite` 标记**从未被拾取**（S29.3 P0 根因）⇒ 1080 层命中改走**手动几何**（纯函数、可单测）。
   换引擎时「2D 精灵能否被拾取」必须实测，不能假定。

**结论（更新）**：B1 的锁定面比原估**更小**（5 场景 chrome + 命中两层），但**换引擎成本比原估更高**
（多了「两层叠压契约」「手动几何命中」「多相机标记」三项须重新实现）。

## 4. WBS（台账 S29 系列，B1 版；替代原「引擎无关求解器」WBS）

| 任务 | 内容 | 依赖 |
|---|---|---|
| S29.0 | 本 B1 定案文档 + 3 条约定松绑签核 + 台账 WBS 更新（撤销原求解器 S29.1~S29.5，改 B1 任务） | 本批 |
| S29.0-void | **删除遗留求解器**：`crates/ti-render/src/layout.rs` + `lib.rs` 接线（原方向作废物）；`cargo build/test` 全 workspace 绿 | S29.0 |
| S29.1 | `bevy_ui` UI 层基建：启用 bevy UI 插件 + UI 相机（与 1080 游戏正交相机分层）+ 一个场景（index）UI 用 bevy_ui 建 + 编译绿 | S29.0 |
| S29.2 | `bevy_picking` 命中层：游戏世界 2D 拾取（对 1080 正交相机）+ UI 控件 `bevy_ui::Interaction`/Button；`touch.rs` 手写几何函数逐个改走 picking/事件；事件回传逻辑核心 | S29.1 |
| S29.3 | 全场景 UI 迁移 bevy_ui（9 场景 + main_map v5 + 后续 10 副本直接 bevy_ui 建）：版面坐标从 `scenes.json` 手写 → bevy_ui 自动布局；**resize 实时重排生效** | S29.2 |
| S29.4 | 门禁改口径：`bevy-cpu-parity` 加「UI 层排除」（只保游戏世界对拍）+ `check_scene_narrative` N8/8pt 等改口径或退役 + `12_bevy_usage.md` §9「bevy_ui 不用」改「bevy_ui 已启用」+ 同步 `11_bevy_migration.md` / `08_render_arch.md` / `09_render_framework.md` | S29.3 |
| S29.5 | 收口：全场景 UI 走 bevy_ui + 命中走 picking + resize 真重排实测 + 游戏世界像素对拍 ≤0.5% + gate 全绿 + 换引擎成本章节回填（§3） | S29.4 |

**与 S27/S28 的关系**：S27.4（v5 环形棋盘）游戏世界部分**仍 1080 数据驱动**（board.json 环格），
其 UI chrome/命中改走 S29.1~S29.2 的 bevy_ui + picking；S28.6 副本场景 UI 直接 bevy_ui 建（S29.3）。
**依赖调整**：S27.4 不再前置「S29.1 求解器」，改前置「S29.2 命中层」（若命中要 picking）。

**与 S31 地图包的关系（S27.4 之后）**：地图 = **参数化**档（③）——环几何/焦点/CTA 等参数
**来自已装包内 `layout.json`**（`assets/maps/<id>/`；规范
[`21_map_package_spec.md`](21_map_package_spec.md)），运行期经 `ti_core::data` 可注入源切换
（`activate_pack(id)`），`scenes.json` 为内嵌默认包。
⇒ **换图只换包不改 Rust**；本文「游戏世界保持 1080 正交 + 数据驱动」的口径不变（数据源从
`scenes.json` 扩为「包优先、默认包兜底」）。

### 4.1 S29.1 模块拆分（执行拆分：模块 / 职责 / 步骤）

**模块与职责（一个文件 = 一个职责，02_conventions §9）**：

| 模块 | 文件 | 职责 |
|---|---|---|
| 依赖开关 | 根 `Cargo.toml` `[workspace.dependencies]`（S27.7 起 ti-engine 并回根 workspace，bevy 特性登记在根） | bevy 特性加 `"bevy_ui"`（编译进 `Node`/`Text`/`IsDefaultUiCamera`/`ComputedNode`；`Button`/`Interaction` 在 0.20 已废弃不挂，交互归 S29.2 `bevy_picking`，见 12 §9.8） |
| UI 基建（零游戏语义） | `crates/ti-engine/src/ui.rs`（新增） | `spawn_ui_camera`（`(Camera2d, IsDefaultUiCamera, Camera{order:1, clear_color:None})` 叠游戏相机之上）+ `build_proof_panel`（通用浮动 bevy_ui 面板：`Node` flex + `BackgroundColor` + `Text` + 按钮外观卡〔**不**挂 0.20 已废弃的 `Button` 标记〕，不含场景名/文案语义；`ChildOf` 层级） |
| 壳接线（游戏层） | `crates/ti-engine/src/bin/treasure_bevy.rs` | `spawn_stage` 后调用 `ui::spawn_ui_camera` + `ui::build_proof_panel`（live 路径；`--selftest` 不加，保像素对拍口径不变） |
| 用法/坑位 | `docs/process/12_bevy_usage.md` §9 | 记「双相机 UI/世界分层」：UI 相机 order=1 + `clear_color=None`（不清屏，游戏世界透出）+ UI 树走屏幕像素、游戏世界走 1080 正交 `o.scale` |

**步骤（批量编辑一次编译，禁探针节奏，硬规则 8）**：
1. `Cargo.toml` 加 `bevy_ui` 特性 → 2. 建 `ui.rs`（相机 + 面板 helper）→ 3. `lib.rs` `pub mod ui;` → 4. `treasure_bevy.rs` 接线 →
5. **一次** `cargo build -p ti-engine` 收口 → 6. `--shot index` 离屏跑 + `ui_probe_system` 打 `[ui] bevy_ui probe: …computed_layout=N`（**证据口径 = `ComputedNode` 计数 > 0**：taffy 自动布局真跑了；离屏像素无法可靠合成 UI 相机，见 12 §9.8）→
 7. `12_bevy_usage.md` §9.8 记坑位 → 8. 证据 + 台账 S29.1 verified + doccheck/doccheck2 + commit/push。

**S29.1 判据（收窄为「基建 + 最小验证」，index 整 chrome 迁移归 S29.3）**：
`cargo build -p ti-engine` 绿 + `ui_probe_system` 打 `computed_layout>0`（bevy_ui 树已建 + taffy 自动布局真跑，
离屏像素不合成 UI 相机故不作判据）+
`ui.rs` 零游戏语义（doccheck2 框架边界）+ 证据 `evidence/S29.1.log`。resize 真重排实测 + index 全 chrome
迁移 = S29.3 判据（不在本任务）。

## 5. 文本 / 图片等比缩放（B1 承接）

- **UI 层**：`bevy_ui` 文本（`Text`）与图片（`UiImage`）随 UI 相机/视口缩放；bevy_ui 自动布局处理重排。
- **游戏世界**：sprite 尺寸随相机 `o.scale`（EXP-116/117 已固化 `o.scale` 语义）；1080 为**游戏内容**设计锚。
- 缩放律沿用 §等比（`s` 驱动字号/图片）；bevy_ui 场景自动，游戏世界走 `o.scale`。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| bevy_ui 与 1080 游戏正交相机**双相机分层**坐标/缩放混淆 | UI 相机与游戏正交相机严格分层（§1）；`12_bevy_usage.md` 增补「双相机 UI/世界分层」坑位 |
| `bevy-cpu-parity` 改口径漏改 → 门禁红 | S29.4 同批改口径 + gate 内对账；UI 层排除写进门禁断言 |
| 命中从「共享层几何」→「Bevy picking」，headless 单测失去命中覆盖 | 逻辑核心仍消费事件（可 headless 测事件→逻辑）；拾取几何本身 Bevy 侧（接受）；关键命中加 e2e 断言 |
| 3 条约定松绑后旧文档（11/12/08/09）与 B1 矛盾 → doccheck 漂移 | S29.4 同步 4 篇文档；doccheck/doccheck2 兜底 |
| 换引擎成本上升（用户权衡后接受） | §3 留档；若日后换引擎，UI/命中重写为既定成本 |

## 7. 改动面声明（硬规则 7）

| 块 | S29.0 | 落码任务 |
|---|---|---|
| 业务规则 | 无 | — |
| 数据 | UI chrome 坐标从 `scenes.json` 移出（游戏内容数据保留） | S29.3 |
| 规则 | 无（命中事件→逻辑核心的事件契约 = 表现层契约） | S29.2 |
| 表现 | bevy_ui UI 层 + bevy_picking 命中 + resize 重排（全进 Bevy） | S29.1~S29.4 |
| 资产 | 无（资产源不变，bevy_ui 走同一资产服务器） | — |
| 文档 | 本文（B1 定案）+ 撤销原求解器方向 + 11/12/08/09 四篇同步 + 台账 WBS | S29.0 / S29.4 |
