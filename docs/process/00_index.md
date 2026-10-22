# 标准化开发流程（Process）

本目录是**不依赖任何逆向产物**的工程规范。新人（或新会话）只读本目录 +
[`docs/business/`](../business/00_index.md) 即可参与开发，无需读 `legacy/docs/facts/`、
`legacy/docs/spec/`、`ref/pseudocode/`。

> **逆向产物的定位变更（2026-10 决策）**
> 逆向产物（`legacy/docs/facts/`、`legacy/docs/spec/`、`ref/pseudocode/*.c`、`crates/ti-core/src/oracle/`）
> **降级为「参照物」**，不再是开发依据、不是验收基准、不进代码注释作为权威来源。
> 业务规则的唯一事实源是 [`docs/business/`](../business/00_index.md) + `assets/data/*.json`。
> 详见 [`docs/plan-s8-refactor.md`](../plan-s8-refactor.md) §1。

## 分工表：谁做什么

按「可验证性 × 重复度」分派，**不要让 LLM 做它不擅长的事**。

| 性质 | 工作 | 承担者 | 判据 |
|---|---|---|---|
| **一次性、可评审** | 业务设计、数值手感、平衡 | 人 + LLM 起草 | 评审签字，写进 `docs/business/` |
| **重复、机械、跨文件** | 脚手架生成、批量改名、格式转换 | **脚本**（`scripts/*.py`、`cargo run -p ti-tools`） | 幂等 + 有单测 |
| **样板代码** | 场景 / 控件 / 模块骨架 | **模板**（[`03_templates.md`](03_templates.md)）+ 脚本套用 | 套用后无需手改即编译 |
| **有成熟库** | 压缩、序列化、UI 布局、随机 | **现成库**（[`01_architecture.md`](01_architecture.md) §4 清单） | 不自造轮子 |
| **一次性、有明确验收** | 单场景逻辑、单系统接线 | **LLM** | 单测绿 + 人工复核 |
| **机器可判定** | fmt / clippy / test / 覆盖率 | **脚本**（`scripts/gate.ps1`） | 全绿 |

**LLM 的禁区**（必须由人确认或脚本判定）：

1. 不得凭记忆断言数值/常量 —— 数值只能来自 `assets/data/*.json` 或 `docs/business/`；
2. 不得「顺手重构」任务范围外的文件；
3. 不得把推测写成事实（`推测` 必须显式标注且给出验证方法）；
4. 不得删除门禁项或写 `TODO`/`unimplemented!`/`unwrap`（tests 除外）。

## 标准工作流

```
1. 认领   读 plan/ledger.yaml，取第一个 deps 全 verified 且 status != verified 的任务
          （`cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` 只校验台账合法性，不选任务）
          ⚠ 任务 ≥ 2 个且文件域互不相交时，改走并行轮（05_module_parallel.md §1），
            不要串行一个做完再下一个
2. 读规格 docs/business/<模块>.md（业务意图，07_ui_flow.md 管表现层）
          + docs/process/02_conventions.md（工程约定）
3. 分类   查 §分工表：这个任务的主体属于「脚本 / 模板 / 库 / LLM」哪一类？
4. 实施   模板 + 脚本优先；能用库不写循环；数值进 data/，不写进 .rs
          （并行轮 = 照任务书白名单开发，越界即违规；见 05_module_parallel.md §4）
         ① **先定模块边界与函数清单**（每个文件 = 一个职责；函数清单先写出来再落码，
            不边想边改 ⇒ 一次改到位；见 02_conventions.md §9）；
         ② **批量编辑 → 一次编译**：同一任务的编辑全部落完再编译，中途不反复
            「改一处编一次」（探针编译仅限结构性不确定的切分边界）；
         ③ 编译报错**一次性收口**（同类错误批量修），不逐条改逐条编。
5. 验收   该任务 dod 命令实跑 → tee evidence/<id>.log（须含任务自身判据）
          （并行轮包内只报「包内绿」，verified 由集成轮给，05_module_parallel.md §5）
6. 门禁   powershell -ExecutionPolicy Bypass -File scripts/gate.ps1
7. 记账   全绿才置 verified + evidence 路径；否则 rounds+=1
8. 复核   子代理报告完成后，主线程独立复核（loop.md §6.2 第 8 条）
```

## 任务粒度

一个任务 = **一次能独立验收的改动**。判据：

- 能用一条命令验证（`cargo test -p X --test Y` / `python scripts/z.py --check`）；
- 触及文件数 ≤ 8；
- 不同时改「接口 + 实现 + 数据」三层（拆成 `.a` `.b` 子任务）。

反例：`实现全部玩法模块`（太大）→ 拆 `S8.4 场景 trait + 场景注册表` / …
正例：`S8.3.1 给 GameApp 引入 trait Scene，Index/SelectRole 两场景实现之`（小、可验收）。

## 本目录清单

| 文件 | 内容 |
|---|---|
| [`01_architecture.md`](01_architecture.md) | 分层、依赖方向、trait 约定、**现成库清单**（不自造轮子） |
| [`02_conventions.md`](02_conventions.md) | 数据驱动、命名、错误、测试、日志约定 |
| [`03_templates.md`](03_templates.md) | 可复制的模版：场景 / 控件 / 数据表 / 模块 / 测试 |
| [`04_tooling.md`](04_tooling.md) | 脚本与 codegen 工具链、幂等要求、新增工具的登记方式 |
| [`05_module_parallel.md`](05_module_parallel.md) | **模块并行开发 loop**（契约冻结 → 多包并行 → 集成联调 → 记账；包白名单互斥 + 任务书模板 + 纪律 5 条） |
| [`06_flywheel.md`](06_flywheel.md) | **对抗流水线飞轮**（找问题 / 审计 / 修复 3 角色 + 开轮必读 6 条 + 飞轮回写 + 停机条件；gate 第 17 项） |
| [`07_render_pipeline.md`](07_render_pipeline.md) | **legacy 像素管线契约**（路径表 / z-order 文件顺序 / 越界求交裁剪 / PCX 透明位 index 0）；S16 起**降级为逆向参照**——运行时架构见 08 |
| [`08_render_arch.md`](08_render_arch.md) | **运行时渲染架构**（分层 / 绘制原语 / z-order=调用顺序 / 5 条坐标与像素来源契约 / grep 门禁 / 测试重锚矩阵；改渲染代码前必读） |
| [`09_render_framework.md`](09_render_framework.md) | **`ti-render` crate 边界契约**（职责/禁止项 / 模块结构 / Theme schema / 从 ti-app 的迁移映射 / 数据分离原则；改 `crates/ti-render/` 代码前必读） |
| [`10_ui_redesign.md`](10_ui_redesign.md) | **UI 重设计规格（S17 单一事实源）**：设计令牌 v2 / 8pt 版面 / 图标 14 项 / 动效 4 参数（航海夜·精修）；改 UI 视觉量前必读 |
| [`11_bevy_migration.md`](11_bevy_migration.md) | **Bevy 0.20 迁移裁决单一事实源**（差距定稿 / 定案推翻签核 / 架构形状 / 新 crate `ti-engine` / WBS S18.0–S18.8 / 风险） |
| [`12_bevy_usage.md`](12_bevy_usage.md) | **Bevy 0.20 用法规范**（官方 `DefaultPlugins` 强制 / 1080 坐标换算 / z 层负值升序 / GPU Text2d 文本 / 离屏截图验收口径 / 已知坑位清单）；写 Bevy 代码前必读 |
| [`13_ui_asset_pipeline.md`](13_ui_asset_pipeline.md) | **UI 资产管线**（资产清单 `assets/ui_manifest.json` = 单一事实源；`python scripts/ui_assets.py add/build/verify` 生图+抠透明+入库；`powershell scripts/ui_preview.ps1 <scene>` 出图验收） |

## 变更面声明（每次开发必填）

**每个任务开工前必须在任务书 / 台账 dod / 交付说明里声明「改哪一块」**；**跨模块必须写原因**。

### 四块（改动面坐标系）

| 块 | 位置 | 判据 | 谁定 |
|---|---|---|---|
| **业务规则** | `docs/business/*.md`（唯一事实源） | 玩法语义/数值/节奏 | **人评审签字**（LLM 只起草） |
| **数据** | `assets/data/*.json` | 数值与布局表 | 业务规则定案后落表 |
| **规则实现** | `crates/ti-core/src/rules/*` | 纯函数规则（不碰表现层） | 业务规则 + 数据 |
| **表现** | `crates/ti-app/src/scenes/*`、`crates/ti-render/`、`crates/ti-engine/` | 渲染 / 交互 / 音效 | 规则层定案后接线 |
| **资产** | `assets/ui/*` + `assets/ui_manifest.json` | 贴图/图标/字体 | 清单驱动（`scripts/ui_assets.py`） |

### 声明格式（任务书 / 交付说明统一用）

```
改动面：业务规则=progression.md §落格节奏 | 数据=balance.board | 规则=rules::board | 表现=main_map.rs | 资产=无
跨模块原因：<为什么必须同时动这几块；为什么不能拆成独立可验收子任务>
```

### 纪律

1. **不声明 = 违规**：评审/复核阶段发现「改动面未声明」⇒ 任务退回（对齐 §分工表「LLM 禁区 2：不得顺手重构任务范围外的文件」）；
2. **跨模块必须给原因**：「接口 + 实现 + 数据」三层同改时，必须说明为何不能拆成 `.a`/`.b` 子任务；能拆就拆（`§任务粒度`：一个任务 = 一次能独立验收的改动，触及文件数 ≤ 8，不同时改三层）；
3. **业务规则先定案**：改动面含「业务规则」时，**先落 `docs/business/` 并评审**，再动代码/数据（对齐 §分工表「业务设计 = 人 + LLM 起草 → 评审签字」）；
4. **资产先过清单**：改动面含「资产」时，先改 `assets/ui_manifest.json` 再生成，禁止手改 `assets/ui/` 里的成品图（`13_ui_asset_pipeline.md` §4）。

## 文档治理（与 [`loop.md`](../loop.md) 一致）

1. **单一事实源**：一个结论只写一处，其他文档引用 ID；
2. **长行禁令**：表格单元格不得塞整段散文（>300 字符）；
3. **不复制粘贴**：`parity.md` / `README.md` 里的清单改为链接到本目录；
4. 新增文档必须登记 [`docs/README.md`](../README.md)（门禁第 9 项自动校验）。