# 文档索引（Docs Index）

`AGENTS.md` 只做路由表与硬规则；本文件是**全库索引**。新增文档必须在本文件登记（硬规则 6）。

## 入口

| 你要做的事 | 读 |
|---|---|
| 任何开工动作之前 | [`loop.md`](loop.md) + `plan/ledger.yaml` + 本文件 |
| 判断「做完了没有」/ 验收 | [`parity.md`](parity.md)（P0~P4 五级判据 + gate 9 项） |
| 写 Rust（布局/常量/错误/宏/trait/风格/测试位置） | [`style.md`](style.md) |
| 逆向 exe 或 `data/` 格式 | [`facts/index.md`](facts/index.md) |
| 找当前阶段任务与验收命令 | [`plan-s0-s2.md`](plan-s0-s2.md)、[`plan-s3.md`](plan-s3.md)、[`plan-s4.md`](plan-s4.md) |
| 避免重复踩坑 | [`experience/index.md`](experience/index.md) |
| 引入新依赖 / 改 workspace 结构 | [`style.md`](style.md) §5.1、§5.9 |

## 全库清单

### 规范与机制

| 文件 | 作用 |
|---|---|
| [`loop.md`](loop.md) | 循环协议、台账 schema、并行会话文件域规则 |
| [`parity.md`](parity.md) | P0~P4 对齐标准 + `gate.ps1` 9 项 |
| [`style.md`](style.md) | Rust 工程规范（workspace/lints/常量/宏/错误/trait/测试/命名/依赖/clippy/MCP） |

### 计划

| 文件 | 覆盖 |
|---|---|
| [`plan-s0-s2.md`](plan-s0-s2.md) | S0 地基 / S1 资源逆向 / S2 三端骨架 |
| [`plan-s3.md`](plan-s3.md) | S3 玩法模块（每模块 8 子步） |
| [`plan-s4.md`](plan-s4.md) | S4 现代化与发布 |
| [`plan-s5.md`](plan-s5.md) | S5 可玩收口（交互壳 + 全流程 + P4-1 + 业务对齐） |

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
| [`facts/pak.md`](facts/pak.md) | 容器 `LucencyPackage` 头/索引/key/重打包（F-201~F-210, F-305） |
| [`facts/map.md`](facts/map.md) | `.Map` 两族布局、控件字段语义、事件 id（F-301~F-318） |
| [`facts/font.md`](facts/font.md) | `GoldLand.Fnt` / `font.dat` 布局与位序（F-303） |
| [`facts/save.md`](facts/save.md) | `settings.dat` / `gl*.sav` / `gl.rec`（F-304） |
| [`facts/tools.md`](facts/tools.md) | IDA / REA MCP 可用性 / RNG 来源（F-313/314/319/320） |

### 经验库（过程与教训）

| 文件 | 内容 |
|---|---|
| [`experience/index.md`](experience/index.md) | 维护规则、status 标记约定、**全量条目索引表** |
| [`experience/formats.md`](experience/formats.md) | 容器/PCX/`.Map`/字体/音频/INI 的坑（14 条） |
| [`experience/reverse.md`](experience/reverse.md) | 逆向方法论（4 条） |
| [`experience/tools.md`](experience/tools.md) | IDA / Python / REA / clippy 的坑（8 条） |
| [`experience/build.md`](experience/build.md) | 构建与三端产物（4 条） |
| [`experience/process.md`](experience/process.md) | 流程与协作（8 条） |
| [`experience/code.md`](experience/code.md) | 代码级教训（2 条） |

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

### 设计文档（`docs/design/`，实现级规格）

| 文件 | 内容 |
|---|---|
| [`game_design.md`](game_design.md) | **游戏设计总纲**（概述/架构/全流程/设计文档索引/实现路由） |
| [`design/scene_flow.md`](design/scene_flow.md) | 场景状态机、主菜单、角色/地图选择、事件码体系 |
| [`design/map_system.md`](design/map_system.md) | 地图数据、地形网格几何、移动规则、据点/地标、事件分派 |
| [`design/economy.md`](design/economy.md) | 贸易系统 + 建造系统（建筑/银行/金上限） |
| [`design/combat.md`](design/combat.md) | 战斗系统（BOSS 档/伤害公式/回合制）+ 卡牌事件系统 |
| [`design/persistence.md`](design/persistence.md) | 存读档（文件布局/魔数/迁移）+ RNG（LCG 算法/播种/调用点） |
| [`design/ui_controls.md`](design/ui_controls.md) | 消息界面 + 控件表 + 资源容器 + 渲染管线 + 控制方案 |
| [`design/constants.md`](design/constants.md) | 全局常量表（分辨率/金上限/BOSS HP/RNG 常数/价格表等） |

### 现代化

| 文件 | 内容 |
|---|---|
| [`modernization.md`](modernization.md) | **多端现代化改造方案**（双层渲染/响应式布局/虚拟手柄/PWA/Android） |

## 文档治理约定

1. **单一事实源**：同一结论只写在一处，其他文档**引用 ID**（`F-xxx` / `EXP-xxx` / `P0-x`）而不是复述。
2. **反例**：曾出现 `parity.md` 的 P1-3 与 P3-1 复述同一判据、`style.md` 复述 REA 工具清单 —— 均已改为引用。
3. **长行禁令**：表格单元格不得塞整段散文（>300 字符）；长结论拆成「短结论 + 指向 `docs/facts/<分卷>.md`」。
4. **`doccheck` 门禁**：第 9 项自动校验 ID 唯一性、跨文档引用有效性、数字断言一致性、evidence 内容特征。见 [`parity.md`](parity.md)。