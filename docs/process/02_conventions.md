# 工程约定（Conventions）

与 [`style.md`](../style.md) 的关系：`style.md` 是 Rust 语法层规范（宏、trait 写法、clippy），
本文件是**架构与流程层**约定。冲突时 `style.md` 管语法、本文件管「该不该这么写」。
两者都被 `scripts/gate.ps1` 强制。

## 1. 数值与文本：禁止硬编码

| 禁止 | 要求 | 反例 → 正例 |
|---|---|---|
| `.rs` 里写 `0x03E0`、`176`、`210` 等布局/配色常量 | 进 `assets/data/scenes.json` 或命名常量 | `fill_rect(20, y-10, 136, 26, 0x03E0)` → `fill_rect(*r, *t, *w, *h, *color)`（来自 scene 表） |
| `.rs` 里写中文字符串 | 进 `assets/data/strings.json`，用 key | `draw_text_row("选择角色", ...)` → `draw_text_row(&S.get("scene.select_role.title"), ...)` |
| 玩法数值（金/伤害/价格/BOSS HP）写进 `.rs` | 进 `assets/data/balance.json` | `oracle_boss_hp(tier)` 硬编码表 → `balance.boss[tier]` |
| 中文注释混进 API 文档 | 文档注释可用中文，但**不含数值断言** | — |

理由：数值硬编码 = 每次调平衡都要改代码 + 重编译 + 评审 diff 噪声；LLM 容易「顺手改数值」
（AGENTS.md 禁区第 1 条）。data/ 化后，数值改动是纯 JSON diff，可评审、可回滚、LLM 不该碰。

## 2. 门禁红线（`scripts/gate.ps1` 判定）

| 模式 | 范围 | 判据 |
|---|---|---|
| `TODO` / `FIXME` / `unimplemented!` / `todo!` / `panic!` | `crates/**/src/**`（tests 除外） | 0 命中 |
| `.unwrap()` / `.expect()` | `crates/**/src/**`（tests 除外） | 0 命中 |
| `--release` 下 `unsafe` | 全 workspace | deny |
| clippy `-D warnings` | 全 workspace all-targets | 0 warning |
| fmt | 全 workspace | 0 diff |

新错误一律用 `thiserror` 定义的具名变体，**不得**用字符串 panic。参考 `ti-formats/src/error.rs`。

## 3. 命名

- crate：`ti-<layer>`；模块 `snake_case`；类型 `PascalCase`。
- 数据字段：`snake_case`；JSON key 同名（serde `rename_all = "snake_case"`）。
- 场景 id：小写短横线（`select_role`），与 `scenes.json` 一致。
- 私有辅助：`_` 前缀；禁止无意义缩写（不用 `h`/`d` 当参数名，用 `hp`/`damage`）。

## 4. 错误处理

1. 玩法层（`ti-core`）：内部 `Result`，对外暴露**不含 IO 细节**的业务错误；
2. 格式层（`ti-formats`）：每个格式一个 `XxxError`（thiserror），变体对应具体失败原因（不合并成 `Invalid`）；
3. 表现层（`ti-app`）：渲染/输入不返回错误，失败即记录日志 + 跳过该帧；
4. **数据文件加载失败 = 硬错误**（不进 fallback 默认值）。

## 5. 测试

| 类型 | 位置 | 判据 |
|---|---|---|
| 单元测试 | `crates/<c>/tests/<name>.rs` | 一个文件一个关注点；名字不含编号（不许 `test_1`） |
| 全流程回归 | `crates/ti-app/tests/` + `tests/scripts/full_run.json` | 固定 seed、逐步断言场景流转 |
| 渲染对齐 | `crates/ti-app/tests/render_parity*.rs` | 确定性 / 非空白 / 场景区分 / 真字形 |
| 数据表完整性 | `crates/ti-core/tests/` | 每个 `assets/data/*.json` 能解析、字段齐全、数值在合法域 |

测试里允许 `.unwrap()` / `.expect("...")`，但 `.expect` 必须带**解释性消息**（不许裸 `.unwrap()`）。

## 6. 门禁友好的提交粒度

一个 commit = 一个任务 = gate 可过。禁止「先提交半成品再补测试」——
`rounds` 达 3 次仍未 verified 的任务，按 `loop.md` §4 拆子任务。

## 7. 日志与调试

- 禁裸 `println!`（除 headless/bin 验收输出）；用 `stats` feature 的 FPS/跳帧统计（见 `stats.rs`）。
- 调试打印走 `#[cfg(feature = "stats")]` 或 `tracing`（若引入）。

## 8. 文档

- 结论单一事实源，跨文档引用 ID 不复述；
- 业务规则写 [`docs/business/`](../business/)，工程规范写本目录，格式事实写 `legacy/docs/facts/`；
- **不要**在 `.rs` 注释里把「推测」当事实陈述——推测写进 `docs/business/` 对应模块并标注验证方法；
- 新文档登记 [`docs/README.md`](../README.md)。

## 9. 模块边界与批次节奏

> 本节是**工程约定**（怎么写代码 + 怎么推进）；流程层（谁做什么、几件套）见
> [`00_index.md`](00_index.md) §标准工作流。

### 9.1 按模块功能开发（先定边界，再落码）

1. **一个文件 = 一个职责**：文件首部 doc 必须写清「负责什么 / 不负责什么」
   （`docs/style.md` §5.12.1）。文件名 = 职责名（`board.rs` / `saveload.rs` / …）。
2. **先出模块边界与函数清单再落码**：动手前先列出「哪些函数归哪个模块」的清单
   （可用一次性解析脚本产出），据此机械搬移 —— 避免「先切文件再想边界」造成的返工
   （典型症状：同一函数同时出现在两个模块 ⇒ E0592 重复定义）。
3. **跨模块调用显式化**：跨模块要调的 `fn` 改 `pub(crate)`；其它模块通过
   `use super::X` 显式导入，不靠 `use super::*`（通配导入会被 clippy pedantic 拦）。
4. **私有字段对子模块可见**：拆目录模块（`foo.rs` → `foo/`）时父模块私有项对其后代可见
   ⇒ **不需要**把字段改成 `pub`；只有跨 crate 才要 `pub`。
5. 目录模块化后**公共 API 路径不变**（`crate::foo::Bar`），外部调用方零改动。

### 9.2 批次节奏（编译成本纪律）

1. **批量编辑 → 一次编译**：同一任务的编辑全部落完再编译；「改一处编一次」的探针节奏
   在大改动里会成倍放大编译时间（一次 2181 行文件的模块拆分实测编译 10 次）。
2. **探针编译仅限结构性不确定处**：如按行号切分文件（段边界未知）时，先用一次性脚本
   验证边界（解析 + 打印），再落文件 —— 而不是「改一点、编一次、看报错再改」。
3. **报错一次性收口**：拿到一批编译错误后，先归类（同名重复 / 缺 import / 可见性），
   再批量修，不要逐条改逐条编。
4. **自动收敛优先**：未用 import 交给 `cargo fix` 批量处理；断言/路径类硬错误手工批量修。
