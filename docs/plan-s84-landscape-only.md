# S84 计划 · 纯横屏重写（删除竖屏数据 / 资产 / 实现）

> 用户 2026-11-24 拍板「竖屏，全部删除」+「1-3 全做」（延续 2026-11-08「不要竖屏，都是横屏」决策）：
> ① 删 S76 七份场景文档里的竖屏截图/章节 ② 删 `scenes.json` 每场景的竖屏顶层 primitives/字段
> ③ 把项目坐标系从 1080×1920 改为纯横屏 1920×1080。
>
> **重要澄清（已与用户确认口径）**：2026-11-08 删的是「竖屏主题化任务」（配色主题），
> 1080×1920 是 `docs/process/08_render_arch.md` 的宪法契约、P0~P4 全锚定它。用户明确要「1-3 全做」
> = 连坐标系一起改。本计划是**架构级重写**，不是内容运营。
>
> 里程碑判据：gate 36 项 0 failure + doccheck/doccheck2 0 error + 台账三方对账 + 出图验证（横屏场景视觉正确）。

## 0. 现状（重写前基线 · S84.0 explore 实测回填）

> 来源：主线程 2 路 explore 子代理实测（2026-11-24），带文件路径+行号，非估计。

- **坐标系三源**（必须同改，否则 bevy-cpu-parity 静默错位）：
  - `ti_render::framebuffer.rs`：CANVAS_W/H=1080/1920、LOGICAL_W_I32/USIZE=1080、LOGICAL_H_I32=1920、FB_PIXELS=积；LANDSCAPE_W/H_*=1920/1080（S38 已存在）。
  - `ti_platform::gfx.rs:21-33`：CANVAS/FB/LW/LH 同值副本写死。
  - `ti_platform::settings.rs:73-94`：Orientation 枚举（默认 Portrait）+ logical_size()/from_size()。
- **横屏基础设施已全存在**（S38 方案 A）：`if landscape` 分支遍布 main_map.rs(37)/treasure_bevy.rs(22)/canvas.rs(9)/vs.rs(7)；`scenes.json` 双份数据 + `prims_for/extra_for`；横屏专属 `main_map_v6_layout_landscape.json`/`vs_anim_proto_landscape.rs`/`message_panel_params(landscape)`。
  ⇒ **改造本质 = 「扶正横屏 + 删竖屏默认」，不是从零造横屏**。
- **影响面实测数字**：生产源码方向分支 ~77 处；测试文件锚 1080/1920 = **47 个**（35 个用 `GameApp::new(1080,1920)` 夹具，多为静默语义错位风险）；docs 含 1080×1920 声称 = **41 个**。
- **gate 硬依赖**：第 13 项 doccheck2 六子检查（runtime-no-legacy-coords 字符串锁 `= 1080;`/scene-layout-fidelity 锚 1080 标量/layout-doc-table-cross-check 剥 landscape 段对账竖屏表/engine-no-bare-half 锁 540/960/landscape-no-portrait-hardcode 要求 letterbox_size 含 LANDSCAPE_*）；第 19 项 render-invariants；第 21.5 项 bevy-cpu-parity（CPU Framebuf 与 GPU 离屏同坐标系）；第 22/23 项浏览器 e2e。

**最小可验证闭环（S84.2）**：常量翻转（三源同改 1920/1080）+ `Orientation::default()`→Landscape + doccheck2 字符串锁同步——不动任何 `if landscape` 分支即可让 `--shot`/wasm/native 默认全横屏。真正的大工作量在第二阶段「删竖屏数据/分支/文档」。

## 0.1 数据/平台/工具影响面（S84.0 explore 实测回填）

> 来源：数据层 explore 子代理实测（2026-11-24），带路径+行号。

**关键区分（不可一刀切）**：
- **真正要删的竖屏数据** = 7 场景顶层 `primitives` 数组 + `portrait_zones` + index.ring（罗盘环）+ 顶层 extra 里的竖屏坐标值（如 `status.y=1840`、`ring.center_x=540`）。
- **不能删** = 顶层 `extra`/`extra_scalar` 的**键结构**（status/body/panel/world_canvas/ring/menu/grid.* 等横屏路径仍消费）。`index.menu.*` 必须留，`index.ring.*` 可删。

**6A 零 Rust 兼容技巧（推荐过渡）**：把每场景顶层 `primitives` 直接换成原 `landscape.primitives`（1920×1080）并删 `landscape` 段——因 `prims_for(true)` 回退条件 = `landscape_prims.is_empty()`（`ti-core/src/data/mod.rs:803`），横屏调用自然落到 `&primitives`；`extra_for` 同构。**全部横屏渲染路径立即正确，Rust 结构体可不动**。代价 = 仍留竖屏死代码，待 6B 逐步清。

**6B 彻底简化的必改点**：
- 数据结构：`ti-core/src/data/layout.rs` 删 RawLandscape/RawScene.landscape/portrait_zones/解析链；`mod.rs` 删 SceneLayout 对应字段 + `prims_for/extra_for`，消费方直接用 `.primitives/.extra`。
- 引擎/应用：去方向分支——`index_data_items` 竖屏分支 + `index_ring_rect`、`index_param_rect` 竖屏环分支、`message_panel_params` else、main_map.rs 37 处 PANEL else、vs.rs 分支、`treasure_bevy.rs` 22 处方向判别。
- 工具/平台改键目标：`ui_theme.py`/`map_pack.rs::merge_theme_scenes`/`ui_wizard.rs`/`ti-console themes.rs` 写 `index.landscape.primitives` → `index.primitives`；`validate_data::check_scenes_item_contract`/`check_data_driven`/`ui_assets` 原语扫描盲区（扁平化后反而更严，利好）。
- doccheck2：`check_scene_layout_fidelity` 竖屏锚（medallion_ring/chart_full/btn_spin/`y:1840`/`w:448`）改横屏；`check_layout_doc_table_cross_check` 剥 landscape 段逻辑改；`landscape-no-portrait-hardcode` 退役；N8 乘数 1080×1920→1920×1080。
- 退役 `scripts/gen_index_medallions.py`（服务竖屏 index.ring）。
- **main_map 横屏原语补 `pick_key=4`**（现横屏 6 条无 pick_key，否则摇骰不可点 + `main_map_pick_e2e` FAIL）。
- 测试：47 文件的 1080 锚点 + 35 个 `GameApp::new(1080,1920)` 夹具改 1920×1080；`bevy_native_render.rs` 读竖屏 42 条原语要改；`e2e_playability.rs` 的 `dynamic_entities_land_in_declared_portrait_zones` 随 portrait_zones 删除重写。

## 1. 三层删除范围

| 层 | 删什么 | 保留/改造 |
|---|---|---|
| **① 文档层** | 7 份场景文档的竖屏真渲染图（`bevy_native_<scene>.png`）、ASCII 竖屏布局图、竖屏坐标表、横竖差异里的竖屏侧 | 横屏图 + 装配图；全文只讲 1920×1080 |
| **② 数据层** | `scenes.json` 每场景顶层竖屏 primitives/extra/portrait_zones/focus_anchor 等 | `landscape` 段提升为主（去嵌套）；资产只留横屏引用集 |
| **③ 实现层** | `Orientation::Portrait` 分支、方向判别、CPU 路径竖屏原语表、竖屏出图/resizetest 竖屏分支 | 单一横屏坐标系 1920×1080；`Landscape` 成为唯一形态 |

## 2. WBS（阶段门，逐阶段验收）

| 任务 | 阶段 | 内容 | 判据 |
|---|---|---|---|
| **S84.0** | 立项 | 本文 WBS 定案 + 影响面清单（explore 回填）+ 登记（README/AGENTS/台账） | doccheck 0 error + 台账条目 |
| **S84.1** | 文档层 | 删 7 份场景文档竖屏图/章节，全文只讲横屏 | 7 文档 0 竖屏引用 + 图片引用全可达 + doccheck 0 error |
| **S84.2** | 常量翻转（最小闭环） | 三源同改 1920/1080（framebuffer/gfx/settings）+ `Orientation::default()`→Landscape + doccheck2 字符串锁；**保留全部 `if landscape` 分支** | cargo test 编译过 + `--shot` 默认 1920×1080 + gate 全绿 |
| **S84.3** | 数据层 | scenes.json landscape 提升为主 + 删竖屏原语段；`SceneLayout` 简化 | validate_data 0 违规 + 场景加载测试过 |
| **S84.4** | 渲染/UI 层 | ti-engine/ti-app 删方向分支 + 竖屏渲染路径；`chrome_in_bevy_ui` 口径不变 | 7 场景 `--shot` 横屏出图非黑 + bevy-cpu-parity |
| **S84.5** | 测试/门禁 | 45 个测试文件的 1080 锚点改 1920；gate/parity 判据改横屏 | cargo test 全绿 + gate 36 项 0 failure |
| **S84.6** | 收口 | 文档计数/声称点同步 + 三方对账 + 推送 | gate 全绿 + ledger 全 verified + push |

## 3. 执行顺序

S84.0 → S84.1（文档，零风险）→ S84.2（常量）→ S84.3（数据）→ S84.4（渲染）→ S84.5（测试门禁）→ S84.6（收口）。
每阶段门：实跑判据 → evidence/<id>.log → gate 全绿 → 台账 verified。

## 4. 关键约束与风险

1. **宪法契约变更**：`08_render_arch.md` §坐标系 1080×1920 是硬契约，改它须同步改该文档 + `docs/parity.md` P4 + `style.md`/`12_bevy_usage.md` 的 1080 换算口径。
2. **双渲染一致性**：`bevy-cpu-parity` 依赖 CPU↔Bevy 同坐标系对拍，CPU 后端 `ti-render` 的 framebuffer 也得改 1920×1080。
3. **资产引用集**：`scripts/apk_audit.py --refset` 按引擎引用集打包，竖屏资产删后引用集缩小，APK 体积应下降（验证）。
4. **平台工具**：`ui_verify.py` 设备矩阵含竖屏档（844x390 实为横屏小尺寸），需确认无竖屏档遗留。
5. **不可逆性**：删竖屏数据/实现后，P5 真机（横屏 900×1600 / 1600×900）判据要重核。

## 5. 参考

- `docs/plan-s49.md` §竖屏主题作废（2026-11-08 决策出处）
- `docs/process/08_render_arch.md` §坐标系宪法契约
- `docs/process/17_layout_engine.md` B1（UI bevy_ui 自动布局）
- `docs/plan-s38-landscape.md`（若存在）横屏适配方案
