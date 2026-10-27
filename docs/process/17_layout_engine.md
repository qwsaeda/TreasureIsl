# 17 · 布局引擎定案（S29：声明式布局树 + 引擎无关求解器 + 真响应式重排 + 命中派生）

> **本文件 = 「布局怎么算、写在哪、怎么响应视口、怎么换引擎」的单一事实源**。
> 触发：用户 2026-10-27「有引擎确不用的问题，布局还是写死的」→ 要求**综合分析**
> （最大化引擎便利与稳定 + 保留换引擎余地），落地范围 = **全场景一次性迁移**；
> 响应式档位 = **全场景真重排（b）**；文本/图片 = **等比缩放**（用户 2026-10-27 两次拍板）。
>
> 关联文档：`11_bevy_migration.md`（引擎裁决）/ `12_bevy_usage.md`（Bevy 用法与不用清单）/
> `08_render_arch.md`（双渲染一致性）/ `09_render_framework.md`（ti-render 边界）/
> `business/08_layout_design.md`（**降级为「参考视口 1080 坐标表镜像」**，S29.4 起由求解器断言生成）。

## 1. 现状盘点（引擎用在哪 / 没用在哪儿 / 为什么）

**已用**（保留，别动）：wgpu 渲染（Sprite/Text2d/Camera2d）、资产服务器（TTF+PNG 异步）、
输入事件、CJK 文本（`complex_script_segmentation`）、截图读回（`ScreenshotCaptured` Observer）。

**故意没用**（三条硬约束决定，`12_bevy_usage.md` §9）：
| 能力 | 不用原因（= 保留的稳定性） |
|---|---|
| `bevy_ui`（Node/Layout + Button/Interaction） | ① `ti-app` 逻辑层必须 bevy-free（headless 跑门禁/测试，`links=alsa` 依赖冲突）；② 双渲染期 CPU 帧缓冲 ↔ Bevy 像素对拍（`bevy-cpu-parity` 门禁）要求版面**同一份数据**，bevy_ui 组件树只在 Bevy 世界存在 ⇒ 版面真相分裂两套；③ bevy_ui 走独立 UiCamera/CSS 布局，与本项目「单一设计坐标 + 一个正交相机」冲突。**真响应式（b）不改此结论**：响应式由**引擎无关求解器**提供，比 bevy_ui 更可移植、可离线测试 |
| `bevy_picking` | 命中是**交互逻辑**，必须与渲染同坐标系且可离线单测（EXP-067/071 正逆成对）；picking 只在 Bevy 侧可用 ⇒ 只能做**可选增强**（hover），不能当真相源 |
| 后处理 / 3D / 物理 / 动画系统 | 项目不需要（全精灵+Text2d 2D 海盗经营） |
| `bevy_audio` | 双渲染期刻意关（alsa 冲突），S18.8 并回后开 |

**病根（用户「布局写死」的正确诊断）**：
`scenes.json` 每场景 = 几十上百个手写 `x/y/w/h` 绝对矩形（钉死 1080）；命中判定 =
`touch.rs` 里**按版面形状手写几何函数**（`menu_row_at` / `menu_medallion_at` /
`scene_list_bands` / `slot_bands` / `grid_cell_at`（等距逆投影）…）；N8 焦点 bbox 要
**手抄** `scenes.json ↔ manifest bbox_pct`；改一次版面 = 人肉对坐标 + 人肉推命中公式。
**「1080 锁」把版面钉死在单一分辨率，加场景/换引擎/改分辨率全放大成本**。
缺的不是「用没用 bevy_ui」，是缺一个「**布局求解层 + 视口参数化**」。

## 2. 目标拆解（用户三条要求）

1. **最大化引擎的便利与稳定**：自动布局（不手抄坐标）、**对任意宽高比真重排**、
   命中自动派生、版面即数据（改 JSON 不改码）、所有行为可离线单测。
2. **换引擎余地**：版面真相 / 命中 / 重排 / 逻辑必须**引擎无关**（可移植），
   换引擎时只换渲染适配器 + 重跑像素对拍，**场景数据/布局树/重排规则/命中/逻辑/资产全部存活**；
   响应式视口参数化后，换到的**任意**能给出 viewport 的渲染器都自动获得重排能力。
3. **真响应式（用户拍板 b 档）**：1080 不再是锁，只是**参考视口（REF）**；
   文本/图片随视口**等比缩放**；每个场景可写自己的**重排规则**（声明式，引擎无关）。

三条的交集 = **把「引擎能力」中可移植的部分（布局语义、重排语义、命中语义、缩放律）
做进 bevy-free 共享层；把不可移植的部分（GPU 渲染、资产服务器、文本引擎）留在可替换的
渲染适配器里**。`resolve(tree, viewport)` 是核心：视口是**参数**，不是常量。

## 3. 参考视口 vs 实际视口（拔掉 1080 锁的正确切法）

| 概念 | 定义 | 用途 | 是否锁死 |
|---|---|---|---|
| **REF 参考视口** | `1080×1920`（9:16 竖） | 设计原点；`08_layout_design.md` 坐标表锚；CPU 帧缓冲**固定**渲染尺寸；`bevy-cpu-parity` 像素对拍**锚点**；8pt 断言空间（s=1） | 是（但只是「锚」不是「锁」：其它视口照样重排，不破坏 REF） |
| **VP 实际视口** | 后端真实尺寸（Bevy 窗口 / web canvas / Android view） | `resolve` 的第二参数；真响应式全在这发生 | 否（任意宽高比） |

**关键澄清（防误解）**：CPU 帧缓冲后端**仍固定在 REF 尺寸**（帧缓冲本质 = 固定宽高 +
像素对拍需要单尺寸）；「真响应式」= ① 布局/命中/重排逻辑 `resolve(tree, VP)` 是**纯函数**
（headless 可对任意 VP 单测）② Bevy/web/Android 渲染器把**实际 VP** 喂给 `resolve` 并
在窗口 resize 时重解（真重排）。CPU 后端只是「在 REF 尺寸下渲染 + 充当对拍锚」，
不承担多分辨率。

## 4. 方案对比（综合评估矩阵）

| 维度 | A 求解器（固定 1080） | B bevy_ui | C 维持手写 | **D = 引擎无关求解器 + 真重排(b) + 等比缩放（采纳）** |
|---|---|---|---|---|
| 引擎便利 | ★★★★ | ★★★★★ | ★ | ★★★★★（任意宽高比重排 + 缩放律） |
| 稳定性 | ★★★★★ | ★★★ | ★★★★ | ★★★★★（纯函数 + 单测 + 缩放律可离线证伪） |
| 双渲染一致性 | ★★★★★ | ✗ | ★★★★ | ★★★★★（两后端同一 `resolve` 结果；对拍锚在 REF） |
| 换引擎成本 | ★★★★★ | ✗ 绑死 | ★★★★ | ★★★★★（重排/缩放/命中全引擎无关，换引擎=换适配器） |
| 逻辑层 bevy-free | ✓ | ✗ | ✓ | ✓ |
| 工作量 | 中 | 大 | 0 | **大**（求解器 + 重排 DSL + 全场景重排规则 + 宽高比矩阵门禁） |
| 风险 | 中 | 高 | 慢性 | 迁移面大 → 双轨逐场景切闸 + 矩阵门禁兜底 |

**定案 = D**：① `ti-render::layout` 引擎无关求解器，`resolve(tree, VP) → Resolved{rects, hits, s}`，
视口是参数；② 缩放律 `s`（等比，用户拍板）驱动**字号 + 图片资产**尺寸；③ 布局树用
weight/百分比/宽高比约束 + **每场景可选 `reflow` 规则**（声明式条件重排 = 真重排 b 档）；
④ `scenes.json` 升级 `layout` 段（旧 `rect`/绝对坐标段**保留 fallback**，双轨逐场景切闸）；
⑤ 命中 = 同一棵解析树逆查询（`touch.rs` 手写几何函数逐个退役）；
⑥ `bevy_picking` 仅 Bevy 壳做 hover 增强（**后置、可选、不改真相源**）；
⑦ `bevy_ui` 永久弃用（结论留档，`12_bevy_usage.md` §9 增补交叉引用）。

## 5. 缩放律（等比，用户拍板）

`s = clamp(scale_by(VP), s_min, s_max)`：
- **默认 `scale_by = fit`**：`s = min(VP.w/REF.w, VP.h/REF.h)`（均匀 fit，不溢出）；
  竖屏手机 = `height` 锚（`s = VP.h/REF.h`）可逐场景 override。
- `s` 驱动：`字号_px = s × 参考字号`；`图片 sprite_scale = s`（Bevy 侧，HD 高清源不糊；
  CPU 侧按 `s` 缩放烘焙帧缓冲）。
- **地板 `s_min`（默认 0.5，S29.4 定）** 防文字/图标过小；`s_max`（默认 3.0）防桌面放大溢出。
- 8pt 网格：在 **REF 空间（s=1）** 断言 8 整除；运行时随 `s` 等比，不做动态 8pt 对齐
  （动态对齐会抖动，违背稳定性目标；重排只动「容器/对齐」，字号/图标尺寸走 `s`）。

## 6. 布局树 Schema（`scenes.json` 每场景新增 `layout` 段，**归一化/权重空间**）

节点集（**闭集**，求解器未知节点 = 报错不 panic）：

| 节点 | 字段 | 语义 |
|---|---|---|
| `stack {dir,gap,pad,children}` | `dir`=row/col；gap/pad 用 REF 单位（乘 `s`） | flexbox 子集（不换行） |
| `weight`（子属性） | 同层子按 `weight` 分剩余空间 | 权重分栏（替代手算列宽；**真重排主要靠它**） |
| `aspect {ratio,children}` | 容器维持 w/h 比（如 ring 外框） | 宽高比约束 |
| `anchor {x,y,w,h,align,children}` | REF 单位绝对框 + 对齐（center/start/end/stretch） | 过渡期混排：老场景未迁部分仍走绝对 |
| `ring {n,radius,card_w,card_h,start_angle,children[]}` | 圆周均布 n 卡；radius/card 用 REF 单位（乘 `s`），随容器宽高比自适应 | **S27.4 环形棋盘**：31 张岛卡 = 1 行声明 |
| `grid {rows,cols,tw,th,origin}` | 方格网格（替代 `iso_cell_fit` 手算） | 旧 `grid.*` 键声明式化 |
| `reflow { when, overrides[] }` | `when` = 小闭集条件式（`aspect<1`、`VP.w<800`、`s<0.7`、`&&/||`）；`overrides` = 对子节点 set weight/隐藏/移位/换 dir | **真重排（b）规则**：每场景可写多条，声明式 + 可离线测试 |
| `item {asset/icon/text_key,…, hit={kind,id}}` | 叶子 = 原语 + **命中声明**；尺寸走 `s` | 命中盒 = 叶子解析 rect（`hit.id` 供事件路由） |

求解器（`ti-render/src/layout.rs`，纯函数、**引擎无关、零外部依赖**）：
```
resolve(tree: &LayoutTree, vp: Viewport) -> Option<Resolved>
Resolved {
  s: f32,                       // 缩放律结果（§5）
  rects: Vec<ResolvedNode>,     // 各节点在 VP 空间的解析 rect（含 z）
  hits: HitTree,               // 按 z 序命中盒森林；查询 = 逆序取首个包含点
}
```
**不变式（门禁守，全部 headless 可单测）**：
① 同一 `(tree, VP)` 在 CPU/Bevy 两后端 `resolve` 结果**逐 rect 恒等**（对拍判据，锚在 REF）；
② `resolve` **幂等 + 无副作用**（禁 RNG/时间戳；同输入同输出）；
③ 未知节点/未知 `when` token = 返回 `None`（crash-safe，与现行 `rect` 段回退同口径）；
④ 任一 VP 下解析 rect 全在该 VP 的 `[0,VP.w]×[0,VP.h]` 内（越界 = 该视口出空图并 log，不 panic）；
⑤ REF 视口（s=1）下非豁免键满足 8pt（承接现行 `ui_8pt_walk`）。

## 7. 命中判定迁移（EXP-067/071 正逆成对的延续）

命中盒 = `resolve(tree, 当前 VP).hits`；`touch.rs` 手写几何函数**逐场景退役**：

| 现状（手写几何） | S29 后 |
|---|---|
| `menu_row_at` / `scene_list_bands` / `slot_bands`（列表 y 段手推） | `stack dir=col` + `HitTree` 逆查询 |
| `menu_medallion_at`（罗盘环手算角位） | `ring` 节点解析（S22 首页第一个迁） |
| `iso_grid_cell_at`（等距逆投影） | `grid` 节点（v5 主地图弃等距，此函数退役） |
| main_map 节点牌/船点击（按 `track_node.size` 手算） | `ring` + `item.hit` |
| 摇骰/三键 CTA（rect 命中） | `item.hit` |

**响应式命中正确性**：命中盒随 `resolve(tree, 当前 VP)` 生成，窗口 resize 后重解即得
新盒——不需要手写「小屏命中区重算」。迁移纪律不变：**每切一个场景 = `layout` 段先建 +
双轨对拍（REF 下新旧解析 rect 全等）+ 命中切 `HitTree` + 删旧几何函数**，一次一场景。

## 8. 真重排（b 档）规则纪律

- 重排规则只写在 `reflow { when, overrides }` 里（**闭集条件 + 闭集 override 动作**），
  不写自由代码 ⇒ 引擎无关、可离线枚举验证。
- `when` 条件闭集：`aspect < / <= / > / >=`、`VP.w / VP.h` 阈值比较、`s < 阈值`，
  用 `&&`/`||` 组合；未知 token = `resolve` 返回 `None`（不变式③）。
- `overrides` 动作闭集：`set_weight`、`hide`、`set_dir`、`move_to`（reparent 到声明的锚点）。
- 每场景重排规则数**有界**（建议 ≤ 5 条/场景，超出需 doccheck 复核）——防止重排规则失控
  变回「手写坐标」。

## 9. 门禁与文档耦合（S29 批次内同改，缺一 FAIL）

**宽高比矩阵（有界、非无穷，S29.4 定）**：
`REF 1080×1920` / `4:5 1080×1350` / `3:4 1080×1440` / `1:1 1080×1080` /
`16:9 1920×1080` / `21:9 2520×1080` / `min 540×960`。每点跑 headless `resolve` 验不变式④⑤，
Bevy 侧多视口截图做视觉签收（人工 + 出图）。

| 门禁 | 现状 | S29 后 |
|---|---|---|
| N8（`check_scene_narrative`） | `focus_anchor.bbox` ↔ `manifest bbox_pct` **手抄** ±1px | 焦点 rect 由 `resolve(tree,REF)` 得出，`bbox_pct` 自动生成（`layout_spec_check.py` 改读解析） |
| 8pt（`ui_8pt_walk`） | 非豁免键 8 整除（1080 空间） | REF（s=1）空间断言；运行时随 `s` 等比（§5） |
| `check_layout_doc_table_cross_check` | `08_layout_design.md` 表 ↔ `scenes.json` 键 | `08_layout_design.md` **降级为「REF 视口坐标镜像」**，由求解器在 REF 下生成，doccheck 改断一致 |
| `validate_data.py` | 标量正整数 | 增 `layout` 段 schema（节点闭集 + `reflow` when/overrides 闭集 + 字段类型） |
| `bevy-cpu-parity` | GPU/CPU 像素对拍 | 对拍**锚在 REF**；增**解析恒等**断言（两后端 `resolve(tree,REF)` 逐 rect 相等，先于像素对拍） |
| 新增 `layout_matrix` | — | headless 对 §9 矩阵每点跑 `resolve` 验不变式④⑤（纯函数，秒级，进 `gate` 快档） |
| `doc_numbers.py` 台账计数 | 已修口径（EXP-123/124） | 不变 |

## 10. 迁移策略（全场景一次性迁移 = 用户拍板；10 副本场景后续直接按 layout 建）

全场景 = 9 存量场景（index/select_role/select_map/main_map/vs/build/trade/saveload/
message）+ main_map v5 版面（S27.4）；**后续 10 副本场景（S28.6）直接按 `layout` 段新建，
不走双轨**（新场景没有旧 `rect` 段可 fallback）。

双轨期规则（存量 9 场景）：
1. 每场景 `layout` 段与旧 `rect`/绝对坐标段**并存**；渲染与命中**默认走旧段**；
2. 场景迁移 = 写 `layout` 段（含该场景 `reflow` 规则）→ REF 下双轨对拍
   （`resolve(layout,REF) == 旧段解析 rect`，逐 rect ±0）→ 加 `"layout_active": true`
   切闸 → 删旧几何函数 → 跑 §9 宽高比矩阵 headless 验证；
3. 全场景切闸完成 = S29.5 判据（无 `layout_active` 缺省场景 + `touch.rs` 手写几何清零 +
   矩阵全绿）；旧 `rect` 段**留 1 个版本周期**后删（防回滚）。

## 11. 换引擎成本评估（目标 2 的直接回答，响应式加持后更强）

| 资产 | 换引擎（Bevy → 任意 2D 渲染器）是否存活 |
|---|---|
| `scenes.json` 布局树 + `reflow` 规则 / `strings.json` / `balance.json` / `board.json` / 资产 PNG/TTF | ✅ 存活（纯数据，且响应式规则随数据走） |
| `ti-render::layout` 求解器（含重排 DSL + 缩放律 + 命中派生） | ✅ 存活（零外部依赖；换引擎原样复用，**新渲染器只要给 viewport 就免费获得重排**） |
| `ti-app` 场景流 / 战斗 / 经济 / 存档 / 状态机 | ✅ 存活（本就 bevy-free，CPU 帧缓冲后端即「无引擎」证明） |
| `ti-engine`（Bevy 适配器：精灵 spawn / 资产服务器 / 文本 / 相机 / 窗口 / **resize→重解**） | ❌ 重写（= 唯一要换的层，当前 ~2 文件 + 1 工作区；新适配器加一行 resize 回调即可） |
| 像素对拍门禁 | 🔄 重锚（新后端在 REF 尺寸 vs CPU 帧缓冲；`layout_matrix` headless 验证不变） |

结论：**布局/重排/命中/缩放全进共享层后，「换引擎」= 重写 1 个渲染适配器 + 重跑 1 条
REF 对拍**，响应式能力**随数据与求解器自动迁移**——这是「最大化引擎便利」与「换引擎余地」
的最优平衡点。

## 12. WBS（台账 S29 系列）

| 任务 | 内容 | 依赖 |
|---|---|---|
| S29.0 | 本定案文档 + 台账登记（综合分析 / 定案 D / 参考视口 vs 实际视口 / 缩放律 / 布局树+重排 DSL Schema / 命中迁移 / 门禁宽高比矩阵 / 迁移纪律 / 换引擎成本） | 本批 |
| S29.1 | `ti-render::layout` 求解器：`resolve(tree,VP)`（stack/weight/aspect/anchor/ring/grid/reflow/item）+ 缩放律 + HitTree + 5 不变式单测（含重排 DSL 解析 + 矩阵代表点） | S29.0 |
| S29.2 | `scenes.json` `layout` 段 schema（含 `reflow` 闭集）+ `validate_data.py` 校验 + 双轨对拍/矩阵工具 `scripts/layout_diff.py`（REF 新旧解析 ±0 + 矩阵 headless 验证，`--check` 只读） | S29.1 |
| S29.3 | 双后端接线：CPU/Bevy 渲染器读 `resolve` 结果 + Bevy **resize→重解** + `touch.rs` 切 `HitTree`（全 9 场景逐场景切闸，每闸跑矩阵） | S29.2 |
| S29.4 | 门禁升级：N8/8pt/layout-doc-table 改求解器断言 + `layout_matrix` headless 项 + `08_layout_design.md` 降级为 REF 镜像 + `doc_numbers` 计数同步 + 定 `s_min/s_max` + 矩阵定稿 | S29.3 |
| S29.5 | 全场景切闸完成 + 旧几何函数清零 + REF 像素对拍 ≤0.5% + 矩阵全绿 + 换引擎成本章节回填实测 + `12_bevy_usage.md` §9 增补「bevy_ui 永久弃用」注 | S29.4 |

**与 S27/S28 的关系**：S27.4（v5 环形棋盘落码）**等 S29.1 完成后再开工**（ring 节点 +
随视口自适应是 S29.1 的第一个验收用例）；S28 领地规则层（S28.1）与布局无关，可并行；S28.6 副本
场景直接按 `layout` 段新建（§10）。

## 13. 风险

| 风险 | 缓解 |
|---|---|
| 全场景一次迁移 + 真重排规则面大（9 场景 + 门禁 + 双后端 + 矩阵） | 双轨逐场景切闸 + 每闸 REF ±0 + 矩阵 headless 兜底 + 回滚 = 关 `layout_active` |
| 重排规则失控变回「手写坐标」 | §8 纪律：reflow 只用闭集 when/overrides + 每场景 ≤5 条（doccheck 复核） |
| 缩放律 `s` 抖动 / 字号过小 | 默认 fit + `s_min` 地板（§5）；REF 空间 8pt 断言；矩阵签收 |
| 求解器 bug 静默改版面 | 5 不变式全单测 + `layout_diff.py` REF ±0 + 矩阵 headless + REF 像素对拍 |
| N8 手抄断言改自动后旧基线漂移 | S29.4 同批升级，gate 内对账，禁止只改一半 |
| bevy_picking 增强引入第二命中真相 | 定案：picking 只做 hover 视觉，点击/选中/业务一律 `HitTree`（§4 ⑥） |
| CPU 帧缓冲无法多分辨率 → 误以为「响应式没验证」 | §3 澄清：响应式 = headless `resolve` 纯函数验证 + Bevy 多视口截图签收；CPU 帧缓冲只做 REF 对拍锚 |

## 14. 改动面声明（硬规则 7）

| 块 | S29.0 | 落码任务 |
|---|---|---|
| 业务规则 | 无 | — |
| 数据 | 无（schema 先落文档） | S29.2（scenes.json layout 段 + reflow 闭集 + validate_data） |
| 规则 | 无 | S29.1（ti-render::layout，纯几何+重排 DSL，非玩法规则） |
| 表现 | 规格定义（求解器/重排/缩放律/命中/双轨纪律/矩阵） | S29.1~S29.4 |
| 资产 | 无（缩放走 sprite_scale，资产源不变） | — |
| 工具 | `scripts/layout_diff.py`（新增，工具契约 3 处登记） | S29.2 |
| 文档 | 本文 + `12_bevy_usage.md` §9 交叉引用 + `08_layout_design.md` 降级为 REF 镜像注 | S29.0/S29.5 |
