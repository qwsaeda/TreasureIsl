# 文档索引（Docs Index）

`AGENTS.md` 只做路由表与硬规则；本文件是**全库索引**。新增文档必须在本文件登记（硬规则 6）。

## 入口

| 你要做的事 | 读 |
|---|---|
| **理解游戏怎么设计/架构/模块化（第一入口）** | [`00_design.md`](00_design.md)（正向设计总览：核心循环 + 数据分层 + 5 crate 架构 + 9 场景 + 7 模块包 + 窄接口；不读 legacy/） |
| **新开工 / 参与开发（先读这个）** | [`process/00_index.md`](process/00_index.md)（分工表/工作流）+ [`business/00_index.md`](business/00_index.md)（业务唯一事实源） |
| 任何开工动作之前 | [`loop.md`](loop.md) + `plan/ledger.yaml` + 本文件 |
| 判断「做完了没有」/ 验收 | [`parity.md`](parity.md)（P0~P4 五级判据 + gate 22 项 + doccheck 独立第 9 项，数值断言与台账计数漂移检查已并入 doccheck） |
| 写 Rust（布局/常量/错误/宏/trait/风格/测试位置） | [`process/01_architecture.md`](process/01_architecture.md) + [`process/02_conventions.md`](process/02_conventions.md)（新）或 [`style.md`](style.md)（语法层） |
| 套用现成模版 / 写新场景·控件·模块 | [`process/03_templates.md`](process/03_templates.md)（含 §7 模块包任务书模版） |
| 用脚本/codegen 批量处理 | [`process/04_tooling.md`](process/04_tooling.md) |
| **模块并行开发（契约→并行→集成）** | [`process/05_module_parallel.md`](process/05_module_parallel.md)（一轮 4 步 + 包划分 + 纪律 + §4.5 开轮必读）+ [`contracts/`](contracts/00_index.md) + `plan/packages.yaml` |
| **对抗流水线飞轮（找问题→审计→修复→回写）** | [`process/06_flywheel.md`](process/06_flywheel.md)（3 角色 + 开轮必读 6 条 + 飞轮回写 + 停机条件，EXP-068） |
| 查业务规则（战斗/经济/地图/存档） | [`business/`](business/00_index.md) 各系统文档（含 [`07_ui_flow.md`](business/07_ui_flow.md) 场景流/控件/输入/消息） |
| S8 标准化改造 | [`plan-s8-refactor.md`](plan-s8-refactor.md) |
| 逆向 exe 或 `data/` 格式（**仅参照物**） | [`facts/index.md`](facts/index.md) |
| 找当前阶段任务与验收命令 | [`plan-s0-s2.md`](plan-s0-s2.md)、[`plan-s3.md`](plan-s3.md)、[`plan-s4.md`](plan-s4.md)、[`plan-s5.md`](plan-s5.md)、[`plan-s7.md`](plan-s7.md)、[`plan-s8-refactor.md`](plan-s8-refactor.md) |
| 避免重复踩坑 | [`experience/index.md`](experience/index.md) |
| 引入新依赖 / 改 workspace 结构 | [`process/01_architecture.md`](process/01_architecture.md) §4（现成库清单）+ [`style.md`](style.md) §5.9 |
| 推送远端 / 分支与 commit 规范 | [`git.md`](git.md) |
| **常用命令一键入口（native 运行 / 门禁 / 测试 / 推送 / 元素验证 / 台账校验）** | `scripts/playbook.ps1`（`-List` 列全部；16 项命令固化为单一脚本，含 push 前置 gate 全绿检查） |

## 全库清单

### 规范与机制

| 文件 | 作用 |
|---|---|
| [`00_design.md`](00_design.md) | **正向设计总入口**：核心循环 + 数据驱动分层 + crate 架构 + 9 场景流 + 模块包 + 场景宿主窄接口（不读 legacy/） |
| [`parity.md`](parity.md) | P0~P4 对齐标准 + **门禁项数与计数约定的唯一事实源**（doccheck 独立第 9 项含跨文档数值漂移 + release 台账计数同步） |
| [`gamebook.md`](gamebook.md) | **可玩实现手册**：把已验证 `rules` 接进交互壳的全流程接线规格（S5 起） |
| [`release.md`](release.md) | 发布说明（项目概况/平台产物/体积预算/可玩收口/已知限制/如何运行/验收）⚠️ **GBK 编码，勿用 `Set-Content` 编辑** |
| [`style.md`](style.md) | Rust 工程规范（workspace/lints/常量/宏/错误/trait/测试/命名/依赖/clippy/MCP；**§5.6.6 标准库 trait 选用矩阵 + 重构操作规程**） |
| [`git.md`](git.md) | 推送规范（远端 origin/分支模型/推送规则/commit 格式/推送检查清单） |
| [`check_ledger_reality.md`](check_ledger_reality.md) | 防虚报：台账 `verified` ↔ `evidence` ↔ 代码符号三方对账（EXP-058；gate 第 14 项） |
| [`plan-archive.md`](plan-archive.md) | **历史阶段计划归档索引**（S0~S8 逐阶段摘要 + 原文件位置；查背景用，非开发依据） |

### 标准化流程（`docs/process/`，不依赖逆向产物）

| 文件 | 内容 |
|---|---|
| [`process/00_index.md`](process/00_index.md) | **流程总入口**：LLM/脚本/人工分工表 + 标准工作流 + 任务粒度 |
| [`process/01_architecture.md`](process/01_architecture.md) | 分层与依赖方向 + trait 约定 + **现成库清单（不自造轮子）** + 数据驱动 |
| [`process/02_conventions.md`](process/02_conventions.md) | 数值/文本数据化 + 门禁红线 + 命名 + 错误 + 测试 + 提交粒度 |
| [`process/03_templates.md`](process/03_templates.md) | **模版库**：场景/控件/数据表/玩法模块/测试/脚本 六个可复制模版 |
| [`process/04_tooling.md`](process/04_tooling.md) | 工具链现状 + 待建脚本（codegen/校验/硬编码扫描）+ 幂等要求 |
| [process/05_module_parallel.md](process/05_module_parallel.md) | **模块并行开发 loop**（一轮 4 步 + 包划分 + 并行纪律 5 条 + §4.5 开轮必读/规范回执；机读白名单 plan/packages.yaml） |
| [process/06_flywheel.md](process/06_flywheel.md) | **对抗流水线飞轮**（找问题/审计/修复 3 角色 + 开轮必读 6 条 + 飞轮回写 + 停机条件；EXP-068 单一事实源） |
| [process/07_render_pipeline.md](process/07_render_pipeline.md) | **legacy 像素管线契约**（路径表/z-order 文件顺序/越界求交裁剪/PCX 透明位 index 0；S16 起降级为逆向参照，gate 第 19/20 项锚点） |
| [process/08_render_arch.md](process/08_render_arch.md) | **运行时渲染架构**（分层/绘制原语/z-order=调用顺序/5 条坐标与像素来源契约/grep 门禁/测试重锚矩阵；改渲染代码必读） |
| [process/09_render_framework.md](process/09_render_framework.md) | **`ti-render` crate 边界契约**（职责与禁止项/模块结构/Theme schema/迁移映射/数据分离原则；改 `crates/ti-render/` 代码前必读） |
| [process/10_ui_redesign.md](process/10_ui_redesign.md) | **界面全量重设计·航海夜精修规格（S17 单一事实源）**：设计令牌 v2（色板 13 色/8pt 栅格/字号阶/动效 4 参数/图标 14 项）+ 7 场景版面增量 + 对比度 AA 表 + 验收锚点（改 UI 视觉量前必读） |
| [ui-redesign-v3/README.md](ui-redesign-v3/README.md) | **UI 重设计 v3 · 真游戏化版面**（对标拆解 01 + 场景叙事 DSL 规格 02 + 7 场景重设计提案 03 + index 落地 04 + vs 落地 05 + main_map 全量落地 06；gate 第 22 项 `check_scene_narrative` 锚点；v2 的"7 场景同模板"前提由 v3 取代） |
| [ui-redesign-v3/07_v3_5_pass.md](ui-redesign-v3/07_v3_5_pass.md) | **v3.5 真游戏化重设计**（地形数据 + 暖色板 + 画布贴合 + 实体图标化 + HUD/命令盘黄铜化；S18.9~S18.13 执行链；画面丑的根因=设计数据而非渲染/分离） |
| [process/11_bevy_migration.md](process/11_bevy_migration.md) | **渲染引擎迁移裁决（S18 · Bevy 0.20）**：差距定稿 + 定案推翻签核（macroquad 定案作废 / 旧 800KB 预算作废）+ 新 wasm 预算（≤8MB）+ 架构形状（ti-engine 新 crate + Canvas 后端替换 + 双渲染期）+ WBS S18.0–S18.8 + 风险（改引擎层代码前必读） |
| [process/12_bevy_usage.md](process/12_bevy_usage.md) | **Bevy 0.20 用法规范 / 最佳实践 / 坑位清单**（官方 `DefaultPlugins` 强制 + 1080 坐标换算 + z 层负值升序绘制 + GPU Text2d + 离屏截图验收口径 + `links=alsa` 隔离 + 已知坑位；S18.7 起 Bevy = 唯一渲染引擎） |

### 业务设计（`docs/business/`，唯一事实源，不依赖逆向产物）

| 文件 | 内容 |
|---|---|
| [`business/00_index.md`](business/00_index.md) | **业务总入口**：与逆向产物的边界 + 设计意图/数值分层 + 编写约定 |
| [`business/loop_core.md`](business/loop_core.md) | 核心循环（探索→贸易→建造→卡牌→战斗）、目标/失败、回合节奏 |
| [`business/economy.md`](business/economy.md) | 贸易（买卖结算/无动态行情）/ 建造（单调升级链）/ 银行（溢出） |
| [`business/combat.md`](business/combat.md) | 战斗：伤害模型（自主设计非逆向）、回合流程、胜负判定 |
| [`business/progression.md`](business/progression.md) | 地图网格/等距投影/移动规则/据点/卡牌事件（可复现抽牌）/跨图推进（地图解锁链 + 通关判定） |
| [`business/persistence.md`](business/persistence.md) | 存档 v2 JSON schema + 迁移 + 可复现 RNG + 设置 |
| [`business/07_ui_flow.md`](business/07_ui_flow.md) | **UI 流程**：场景流 / 控件 / 键盘触摸输入 / 消息分页（表现层并行开发依据） |
| [`business/08_layout_design.md`](business/08_layout_design.md) | **界面版面（1080）**：7 场景 1080×1920 版面 + 配色 + 字体 + 可测规则；**版面/坐标/配色单一事实源**（= `scenes.json` 同值） |

> **定位变更（2026-10）**：逆向产物（`legacy/docs/facts/`、`legacy/docs/spec/`、`ref/pseudocode/`、`oracle/`）
> 降级为**参照物**，不再作为业务规则来源与验收基准。详见 [`plan-s8-refactor.md`](plan-s8-refactor.md) §1。

### 跨包契约（`docs/contracts/`，模块并行冻结面）

| 文件 | 内容 |
|---|---|
| [`contracts/00_index.md`](contracts/00_index.md) | 契约清单 + 状态（draft/frozen）+ 全票规则 |
| [`contracts/c1_core_public_types.md`](contracts/c1_core_public_types.md) | ti-core 公共类型（`domain/*` + `rules::rng` + `data` 访问器）+ 校验锚点 |
| [`contracts/c2_data_keys.md`](contracts/c2_data_keys.md) | `assets/data/*.json` 的 key 契约（四张表 + 加载纪律） |
| [`contracts/c3_app_host_api.md`](contracts/c3_app_host_api.md) | 表现层会话 API（`SceneHost` 冻结面 / `MenuShell` / `GpuRenderer`） |

### 计划

| 文件 | 覆盖 |
|---|---|
| [`plan-archive.md`](plan-archive.md) | **已完成阶段（S0~S11）计划的单一索引**：逐阶段一句话概括 + 原文件跳转（原文件保留在 `docs/` 原位） |
| [`plan-s12-parallel.md`](plan-s12-parallel.md) | **S12 模块化并行 WBS**（一轮 4 步执行：S12.0 契约冻结 → P1–P5 并行 → S12.6 集成；W2 起 P6 表现层） |
| [`pipeline.md`](pipeline.md) | TEST-AUDIT-FIX 多代理协作循环（**已被 [`process/06_flywheel.md`](process/06_flywheel.md) 取代**，仅存历史） |
| [`plan-s16-loop.md`](plan-s16-loop.md) | **S16 试玩闭环 + 发布 + 设计 sheet**（WIP 草稿，S16 尚未登记台账，勿当依据） |

### 工具与产出物索引

| 文件 | 作用 |
|---|---|
| `scripts/playbook.ps1` | **常用命令一键入口**（16 项命令固化：native 运行 / 无头回归 / wasm / 冒烟 / 门禁 / 测试 / fmt / clippy / 元素验证 / 逐精灵渲染 / 推送前置检查 / 台账 / doccheck / 硬编码扫描；`-List` 列全部；UTF-8 BOM 防 PowerShell 编码错位） |
| `crates/ti-tools/src/bin/specskel.rs` | spec 骨架半自动生成（S0.7；命令见 `AGENTS.md` 命令节） |
| `evidence/S0.7.log` | specskel 验收日志（单测 6/6 + 三模块实跑） |

### 归档参照物（`legacy/docs/`，**S11 已物理归档，不是开发依据也不是验收基准**）

> ⚠️ 以下三组文档随 S11 逆向产物归档一起移到 `legacy/docs/`，仅供查原版形态。
> **业务规则唯一事实源 = [`business/`](business/00_index.md) + `assets/data/*.json`。**
> 定位变更缘由见 [`plan-s8-refactor.md`](plan-s8-refactor.md) §1。

**逆向事实库**（结论 + 证据，`F-NNN` 事实 ID 的定义处）

| 文件 | 内容 |
|---|---|
| [`legacy/docs/facts/index.md`](../legacy/docs/facts/index.md) | 收录规则、分卷表、**已知缺口**、逆向工作流 |
| [`legacy/docs/facts/exe.md`](../legacy/docs/facts/exe.md) | 工具环境实测 + PE 头/节表/导入/版本（F-101~F-108） |
| [`legacy/docs/facts/pak.md`](../legacy/docs/facts/pak.md) | 容器 `LucencyPackage` 头/索引/key/重打包（F-201~F-210, F-305, F-322） |
| [`legacy/docs/facts/map.md`](../legacy/docs/facts/map.md) | `.Map` 两族布局、控件字段语义、事件 id 回填与分派（F-301/302/306/307/308/311/315/316/317/318） |
| [`legacy/docs/facts/font.md`](../legacy/docs/facts/font.md) | `GoldLand.Fnt` / `font.dat` 布局与位序（F-303） |
| [`legacy/docs/facts/save.md`](../legacy/docs/facts/save.md) | `settings.dat` / `gl*.sav` / `gl.rec`（F-304） |
| [`legacy/docs/facts/tools.md`](../legacy/docs/facts/tools.md) | IDA / REA MCP 可用性 / RNG 来源（F-313/314/319/320/321） |

**玩法规格**（S3 每模块一份逆向规格：`main_loop` / `render` / `menu` / `selection` / `main_map` / `trade` / `build` / `battle` / `cards` / `save_load` / `messages` / `rng`）→ [`legacy/docs/spec/`](../legacy/docs/spec/)

**设计文档**（已弃用，S8）：`scene_flow` / `map_system` / `economy` / `combat` / `persistence` / `ui_controls` / `constants` / `main_loop` / `input_mapping` / 总纲 `game_design` → [`legacy/docs/design/00_index.md`](../legacy/docs/design/00_index.md)

> ⚠️ **数值口径已变**：归档文档里的常量/公式是历史参照，**当前权威数值在 `assets/data/balance.json`**，业务解释在 [`business/`](business/00_index.md)。

### 入门教程（`docs/tutorial/`，游戏开发学习材料）

| 文件 | 内容 |
|---|---|
| [`tutorial/README.md`](tutorial/README.md) | **教程总索引**：学习路径 + 进度追踪表 + 与项目文档的关系 |
| [`tutorial/01_game_loop.md`](tutorial/01_game_loop.md) | 游戏循环（update + render 分离、固定时间步长） |
| [`tutorial/02_state_machine.md`](tutorial/02_state_machine.md) | 状态与状态机（FSM、堆栈式状态、场景路由） |
| [`tutorial/03_coordinates.md`](tutorial/03_coordinates.md) | 坐标系与变换（世界/屏幕/本地、等距投影、视口） |
| [`tutorial/04_time_step.md`](tutorial/04_time_step.md) | 帧率无关更新（固定时间步长 + 累积器 + 插值） |
| [`tutorial/05_collision.md`](tutorial/05_collision.md) | 碰撞检测（AABB/圆形/网格、空间划分优化） |
| [`tutorial/06_input_system.md`](tutorial/06_input_system.md) | 输入系统（事件驱动 vs 轮询、输入映射、触摸手势） |
| [`tutorial/07_rendering_basics.md`](tutorial/07_rendering_basics.md) | 渲染基础（帧缓冲/图元/Z-order/裁剪/脏行带） |
| [`tutorial/08_ecs_vs_component.md`](tutorial/08_ecs_vs_component.md) | ECS vs 组件化（为什么本项目选组件化） |
| [`tutorial/09_business_render_separation.md`](tutorial/09_business_render_separation.md) | 业务/渲染分离（架构核心原则） |
| [`tutorial/10_project_architecture.md`](tutorial/10_project_architecture.md) | 项目架构对照（分层 + trait 约定 + 场景模型） |

### 经验库（过程与教训）

| 文件 | 内容 |
|---|---|
| [`experience/index.md`](experience/index.md) | 维护规则、status 标记约定、**全量条目索引表** |
| [`experience/formats.md`](experience/formats.md) | 容器/PCX/`.Map`/字体/音频/INI 的坑（15 条） |
| [`experience/reverse.md`](experience/reverse.md) | 逆向方法论（4 条） |
| [`experience/tools.md`](experience/tools.md) | IDA / Python / REA / clippy 的坑（10 条） |
| [`experience/build.md`](experience/build.md) | 构建与三端产物（9 条） |
| [`experience/process.md`](experience/process.md) | 流程与协作（8 条） |
| [`experience/code.md`](experience/code.md) | 代码级教训（4 条） |

### 格式规格

| 文件 | 内容 |
|---|---|
| [`formats/lucencypackage.md`](formats/lucencypackage.md) | `*.pak` / `*.lpk` 逐字段偏移表 |

### 实施文档（`docs/impl/`，**S3 时期实现级规格，历史参照**）

| 文件 | 内容 |
|---|---|
| [`impl/00_index.md`](impl/00_index.md) | **实施文档总索引**（架构总览/模块依赖/核心类型/实现优先级） |
| [`impl/01_architecture.md`](impl/01_architecture.md) | 系统架构、模块依赖、核心类型定义、帧循环 |
| [`impl/02_scene_flow.md`](impl/02_scene_flow.md) | 场景状态机、事件码分派、输入映射、全局单例 |
| [`impl/03_map_system.md`](impl/03_map_system.md) | 地图数据结构、移动算法、等距投影、摄像机 |
| [`impl/04_economy.md`](impl/04_economy.md) | 贸易系统、建造系统、银行系统、全部数值表 |
| [`impl/05_combat.md`](impl/05_combat.md) | 战斗系统（BOSS 档/伤害公式/回合制/胜负判定） |
| [`impl/06_cards_events.md`](impl/06_cards_events.md) | 卡牌/事件系统（效果码/事件 ID 映射/生成算法） |
| [`impl/07_ui_controls.md`](impl/07_ui_controls.md) | 消息界面、控件表、资源容器、渲染管线 |
| [`impl/08_input_mapping.md`](impl/08_input_mapping.md) | 输入系统（键盘/触摸/手势/场景行为表） |
| [`impl/09_persistence.md`](impl/09_persistence.md) | 持久化（文件布局/魔数/v1/v2 迁移/校验） |
| [`impl/10_rng.md`](impl/10_rng.md) | RNG 系统（LCG 算法（oracle 历史参照）/PCG（当前实现）/播种点/51 处调用点/确定性） |
| [`impl/11_modernization.md`](impl/11_modernization.md) | 现代化方案（双层渲染/缩放/DPI/虚拟手柄/PWA/Android） |
| [`impl/12_audio.md`](impl/12_audio.md) | 音频系统（BGM/音效/音量/静音/浏览器策略） |
 | [`impl/13_appendix.md`](impl/13_appendix.md) | 附录（全常量表/事件码/场景码/错误类型/资源清单） |
 | [`impl/14_asset_enhancement.md`](impl/14_asset_enhancement.md) | HD 资产增强管线（S7 分类清单/工具链选型/流水线步骤/体积预算/验收判据） |
 | [`impl/15_gameplay_full.md`](impl/15_gameplay_full.md) | **完整玩法规格**（全系统实现手册：场景状态机/事件码/核心循环/角色地图/主地图/贸易/建造/战斗/卡牌/消息/存读档/RNG/输入/音效/数值平衡） |
 | [`impl/16_modern_platform.md`](impl/16_modern_platform.md) | **现代多端实现指南**（架构分层/双层渲染/响应式布局/触屏/字体/音频/持久化/PWA/Android/资源分包/性能预算/错误处理） |

### 现代化

| 文件 | 内容 |
|---|---|
 | [`modernization.md`](modernization.md) | **多端现代化改造方案**（双层渲染/响应式布局/虚拟手柄/PWA/Android） |
 | [`impl/14_asset_enhancement.md`](impl/14_asset_enhancement.md) | **HD 资产增强管线**（S7 资源分类/超分/矢量化/多分辨率打包，路由见本表入口） |

## 文档治理约定

1. **单一事实源**：同一结论只写在一处，其他文档**引用 ID**（`F-xxx` / `EXP-xxx` / `P0-x`）而不是复述。
2. **反例**：曾出现 `parity.md` 的 P1-3 与 P3-1 复述同一判据、`style.md` 复述 REA 工具清单 —— 均已改为引用。
3. **长行禁令**：表格单元格不得塞整段散文（>300 字符）；长结论拆成「短结论 + 指向 `legacy/docs/facts/<分卷>.md`」。
4. **`doccheck` 门禁**：第 9 项自动校验 ID 唯一性、跨文档引用有效性、数字断言一致性、evidence 内容特征。见 [`parity.md`](parity.md)。
