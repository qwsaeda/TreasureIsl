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