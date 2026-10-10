# 标准化开发流程（Process）

本目录是**不依赖任何逆向产物**的工程规范。新人（或新会话）只读本目录 +
[`docs/business/`](../business/00_index.md) 即可参与开发，无需读 `docs/facts/`、
`docs/spec/`、`ref/pseudocode/`。

> **逆向产物的定位变更（2026-10 决策）**
> 逆向产物（`docs/facts/`、`docs/spec/`、`ref/pseudocode/*.c`、`crates/ti-core/src/oracle/`）
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
| **有成熟库** | 压缩、序列化、UI 布局、随机 | **现成库**（[`01_architecture.md`](01_architecture.md) §5 清单） | 不自造轮子 |
| **一次性、有明确验收** | 单场景逻辑、单系统接线 | **LLM** | 单测绿 + 人工复核 |
| **机器可判定** | fmt / clippy / test / 覆盖率 | **脚本**（`scripts/gate.ps1`） | 全绿 |

**LLM 的禁区**（必须由人确认或脚本判定）：

1. 不得凭记忆断言数值/常量 —— 数值只能来自 `assets/data/*.json` 或 `docs/business/`；
2. 不得「顺手重构」任务范围外的文件；
3. 不得把推测写成事实（`推测` 必须显式标注且给出验证方法）；
4. 不得删除门禁项或写 `TODO`/`unimplemented!`/`unwrap`（tests 除外）。

## 标准工作流

```
1. 认领   手动读 plan/ledger.yaml，取第一个 deps 全 verified 且 status != verified 的任务
           （`cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` 只校验台账合法性，不选任务）
2. 读规格 docs/business/<模块>.md（业务意图）
          + docs/process/02_conventions.md（工程约定）
3. 分类   查 §分工表：这个任务的主体属于「脚本 / 模板 / 库 / LLM」哪一类？
4. 实施   模板 + 脚本优先；能用库不写循环；数值进 data/，不写进 .rs
5. 验收   该任务 dod 命令实跑 → tee evidence/<id>.log（须含任务自身判据）
6. 门禁   powershell -ExecutionPolicy Bypass -File scripts/gate.ps1
7. 记账   全绿才置 verified + evidence 路径；否则 rounds+=1
8. 复核   子代理报告完成后，主线程独立复核（loop.md §6.2 第 8 条）
```

## 任务粒度

一个任务 = **一次能独立验收的改动**。判据：

- 能用一条命令验证（`cargo test -p X --test Y` / `python scripts/z.py --check`）；
- 触及文件数 ≤ 8；
- 不同时改「接口 + 实现 + 数据」三层（拆成 `.a` `.b` 子任务）。

反例：`实现全部玩法模块`（太大）→ 拆 `S8.3.1 场景 trait` / `S8.3.2 场景注册表` / …
正例：`S8.3.1 给 GameApp 引入 trait Scene，Index/SelectRole 两场景实现之`（小、可验收）。

## 本目录清单

| 文件 | 内容 |
|---|---|
| [`01_architecture.md`](01_architecture.md) | 分层、依赖方向、trait 约定、**现成库清单**（不自造轮子） |
| [`02_conventions.md`](02_conventions.md) | 数据驱动、命名、错误、测试、日志约定 |
| [`03_templates.md`](03_templates.md) | 可复制的模版：场景 / 控件 / 数据表 / 模块 / 测试 |
| [`04_tooling.md`](04_tooling.md) | 脚本与 codegen 工具链、幂等要求、新增工具的登记方式 |

## 文档治理（与 [`loop.md`](../loop.md) 一致）

1. **单一事实源**：一个结论只写一处，其他文档引用 ID；
2. **长行禁令**：表格单元格不得塞整段散文（>300 字符）；
3. **不复制粘贴**：`parity.md` / `README.md` 里的清单改为链接到本目录；
4. 新增文档必须登记 [`docs/README.md`](../README.md)（门禁第 9 项自动校验）。