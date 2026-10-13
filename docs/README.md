# 文档索引（Docs Index）

`AGENTS.md` 只做路由表与硬规则；本文件是**全库索引**。新增文档必须在本文件登记（硬规则 6）。

## 入口

| 你要做的事 | 读 |
|---|---|
| **新开工 / 参与开发（先读这个）** | [`process/00_index.md`](process/00_index.md)（分工表/工作流）+ [`business/00_index.md`](business/00_index.md)（业务唯一事实源） |
| 任何开工动作之前 | [`loop.md`](loop.md) + `plan/ledger.yaml` + 本文件 |
| 判断「做完了没有」/ 验收 | [`parity.md`](parity.md)（P0~P4 五级判据 + gate 18 项 + doccheck 独立第 9 项，数值断言与台账计数漂移检查已并入 doccheck） |
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

## 全库清单

### 规范与机制

| 文件 | 作用 |
|---|---|
| [`parity.md`](parity.md) | P0~P4 对齐标准 + `gate.ps1` 18 项（doccheck 独立第 9 项含跨文档数值漂移 + release 台账计数同步，`cargo run -p ti-tools --bin doccheck`） |
| [`release.md`](release.md) | 发布说明（项目概况/平台产物/体积预算/可玩收口/已知限制/如何运行/验收） |
| [`style.md`](style.md) | Rust 工程规范（workspace/lints/常量/宏/错误/trait/测试/命名/依赖/clippy/MCP） |
| [`git.md`](git.md) | 推送规范（远端 origin/分支模型/推送规则/commit 格式/推送检查清单） |

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

### 业务设计（`docs/business/`，唯一事实源，不依赖逆向产物）

| 文件 | 内容 |
|---|---|
| [`business/00_index.md`](business/00_index.md) | **业务总入口**：与逆向产物的边界 + 设计意图/数值分层 + 编写约定 |
| [`business/loop_core.md`](business/loop_core.md) | 核心循环（探索→贸易→建造→卡牌→战斗）、目标/失败、回合节奏 |
| [`business/economy.md`](business/economy.md) | 贸易（买卖结算/无动态行情）/ 建造（单调升级链）/ 银行（溢出） |
| [`business/combat.md`](business/combat.md) | 战斗：伤害模型（自主设计非逆向）、回合流程、胜负判定 |
| [`business/progression.md`](business/progression.md) | 地图网格/等距投影/移动规则/据点/卡牌事件（可复现抽牌） |
| [`business/persistence.md`](business/persistence.md) | 存档 v2 JSON schema + 迁移 + 可复现 RNG + 设置 |

> **定位变更（2026-10）**：逆向产物（`docs/facts/`、`docs/spec/`、`ref/pseudocode/`、`oracle/`）
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
| [`plan-s0-s2.md`](plan-s0-s2.md) | S0 地基 / S1 资源逆向 / S2 三端骨架 |
| [`plan-s3.md`](plan-s3.md) | S3 玩法模块（每模块 8 子步） |
| [`plan-s4.md`](plan-s4.md) | S4 现代化与发布 |
| [`plan-s5.md`](plan-s5.md) | S5 可玩收口（交互壳 + 全流程 + P4-1 + 业务对齐） |
| [`plan-s7.md`](plan-s7.md) | S7 HD 资产增强管线（分类 → 批量超分/矢量化 → 重绘 → 多分辨率打包） |
| [`plan-s8-refactor.md`](plan-s8-refactor.md) | **S8 标准化改造**（定位变更 + 固化/模版化/脚本化/库复用 + 分波次任务） |
| [`plan-s10-refactor.md`](plan-s10-refactor.md) | **S10 现代化收口**（P0 错乱修复 + 占位清零 + GPU 化渲染 + 对抗流水线 loop 至停机） |
| [`pipeline.md`](pipeline.md) | TEST-AUDIT-FIX 多代理协作循环（测试→审计→复核→修复→再测） |
| [plan-s11-forward.md](plan-s11-forward.md) | **S11 正向开发收口**（逆向归档 legacy/ + 业务闭环 + 模块化并行 loop；§7 交接快照） |
| [plan-s12-parallel.md](plan-s12-parallel.md) | **S12 模块化并行 WBS**（一轮 4 步执行：S12.0 契约冻结 → P1–P5 并行 → S12.6 集成；W2 起 P6 表现层） |

### 工具与产出物索引

| 文件 | 作用 |
|---|---|
| `crates/ti-tools/src/bin/specskel.rs` | spec 骨架半自动生成（S0.7；命令见 `AGENTS.md` 命令节） |
| `evidence/S0.7.log` | specskel 验收日志（单测 6/6 + 三模块实跑） |

### 逆向事实库（结论，带证据）

| 文件 | 内容 |
|---|---|
| [`facts/index.md`](facts/index.md) | 收录规则、分卷表、**已知缺口**、逆向工作流 |
| [`facts/exe.md`](facts/exe.md) | 工具环境实测 + PE 头/节表/导入/版本（F-101~F-108） |
| [`facts/pak.md`](facts/pak.md) | 容器 `LucencyPackage` 头/索引/key/重打包（F-201~F-210, F-305, F-322） |
| [`facts/map.md`](facts/map.md) | `.Map` 两族布局、控件字段语义、事件 id 回填与分派（F-301/302/306/307/308/311/315/316/317/318） |
| [`facts/font.md`](facts/font.md) | `GoldLand.Fnt` / `font.dat` 布局与位序（F-303） |
| [`facts/save.md`](facts/save.md) | `settings.dat` / `gl*.sav` / `gl.rec`（F-304） |
| [`facts/tools.md`](facts/tools.md) | IDA / REA MCP 可用性 / RNG 来源（F-313/314/319/320/321） |

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

### 玩法规格（S3 每模块一份）

| 文件 | 对应任务 |
|---|---|
| [`spec/main_loop.md`](spec/main_loop.md) | S3.1 主循环/场景管理 |
| [`spec/render.md`](spec/render.md) | S3.2 资源/渲染核心 |
| [`spec/menu.md`](spec/menu.md) | S3.3 index/intro 主菜单 |
| [`spec/selection.md`](spec/selection.md) | S3.4 selectrole/selectmap |
| [`spec/main_map.md`](spec/main_map.md) | S3.5 主地图 |
| [`spec/trade.md`](spec/trade.md) | S3.6 贸易 |
| [`spec/build.md`](spec/build.md) | S3.7 建造/城堡 |
| [`spec/battle.md`](spec/battle.md) | S3.8 战斗 |
| [`spec/cards.md`](spec/cards.md) | S3.9 卡牌/事件 |
| [`spec/save_load.md`](spec/save_load.md) | S3.10 存读档/设置 |
| [`spec/messages.md`](spec/messages.md) | S3.11 消息/帮助/信息 |
| [`spec/rng.md`](spec/rng.md) | S3.12 RNG 与全局规则 |
| [`gamebook.md`](gamebook.md) | **可玩实现手册**（把已验证 rules 接进交互壳的全流程接线规格，S5 起） |

### 设计文档（`docs/design/`，**已弃用/归档**，S8）

> ⚠️ 本目录为逆向规格参照物，不再是开发依据或验收基准。
> 业务规则唯一事实源 = [`business/`](business/00_index.md) + `assets/data/*.json`。
> 见 [`plan-s8-refactor.md`](plan-s8-refactor.md) §1。

| 文件 | 内容 |
|---|---|
| [`design/00_index.md`](design/00_index.md) | **弃用/归档索引**：定位说明 + 文件清单 + 使用指引 |
| [`game_design.md`](game_design.md) | **游戏设计总纲**（概述/架构/全流程/设计文档索引/实现路由） |
| [`design/scene_flow.md`](design/scene_flow.md) | 场景状态机、主菜单、角色/地图选择、事件码体系 |
| [`design/map_system.md`](design/map_system.md) | 地图数据、地形网格几何、移动规则、据点/地标、事件分派 |
| [`design/economy.md`](design/economy.md) | 贸易系统 + 建造系统（建筑/银行/金上限） |
| [`design/combat.md`](design/combat.md) | 战斗系统（BOSS 档/伤害公式/回合制）+ 卡牌事件系统 |
| [`design/persistence.md`](design/persistence.md) | 存读档（文件布局/魔数/迁移）+ RNG（LCG 算法（oracle 历史参照）/PCG（当前实现）/播种/调用点） |
| [`design/ui_controls.md`](design/ui_controls.md) | 消息界面 + 控件表 + 资源容器 + 渲染管线 + 控制方案 |
| [`design/constants.md`](design/constants.md) | 全局常量表（分辨率/金上限/BOSS HP/RNG 常数/价格表等） |
| [`design/main_loop.md`](design/main_loop.md) | 主循环与游戏状态机（场景枚举/状态转换/跨场景状态/实现路由） |
| [`design/input_mapping.md`](design/input_mapping.md) | 输入映射层（键盘+触摸双路径/手势/场景特定操作） |

### 实施文档（`docs/impl/`，实现级完整规格）

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
3. **长行禁令**：表格单元格不得塞整段散文（>300 字符）；长结论拆成「短结论 + 指向 `docs/facts/<分卷>.md`」。
4. **`doccheck` 门禁**：第 9 项自动校验 ID 唯一性、跨文档引用有效性、数字断言一致性、evidence 内容特征。见 [`parity.md`](parity.md)。
