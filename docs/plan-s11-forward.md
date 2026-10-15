# S11 正向开发收口计划（Forward-Development Closure）

**目标**：把逆向产物物理归档到 `legacy/`，业务文档补齐闭环（零逆向引用），
并建立「契约先行 + 文件域不相交并行 + 集成联调」的模块化开发 loop，
之后才启动 S12 模块并行开发。S11 是**文档/目录/门禁/流程**层的收口，
不写玩法代码。

**前置决策（2026-10 用户）**：
- 逆向产物归档目录名 = `legacy/`（git 内保留历史）；
- `ti-core/src/oracle/` 归档为独立 crate `legacy-oracle`（`legacy/oracle-rust/`，
  workspace member，经 dev-dependency 参与测试编译），桥接层 `ti-core/src/oracle/`
  保留路径透明；
- 业务补强全做：去引用 + 新增 `business/07_ui_flow.md` + 重写 gamebook；
- **开发模式 = 模块并行**（契约冻结 → 文件域不相交并行 → 集成轮），提炼成
  可复用的 loop + 子代理任务书模板（§6，见 `process/05_module_parallel.md`）。

> S10.5 对抗流水线与 S11 并行推进（文件域：S11 动文档/目录/门禁/流程，
> S10.5 动审计漂移修复）。

## 1. 现状诊断（为什么现在做）

| 断点 | 位置 | 影响 |
|---|---|---|
| 业务文档引用逆向产物 | `business/progression.md`（F-307）、`00_index.md`（7 处）、旧 `gamebook.md`（逆向语境） | 「唯一事实源」名不副实 |
| `oracle/` 在玩法层 | `ti-core/src/oracle/`（11 文件）+ `tests/reference_diff_*.rs` | 物理位置暗示参照实现还是玩法层一部分 |
| 表现层无业务规格 | `business/` 缺 UI/交互/场景流 | 并行开发表现层无依据 |
| **开发模式串行** | 现状 loop 是单线程「gate 全绿才推进」，子代理无法并行开工 | 与真实程序员「模块划分 + 接口先行 + 并行 + 集成联调」相反 |
| 逆向产物散落 | 根目录 exe/i64、`data/`、`ref/`（1827 伪代码）、`docs/{facts,spec,design}/` | 新会话 LLM 仍会路由进逆向语境 |

**闭环判据（阶段 C 机器化断言）**：
1. `docs/business/**/*.md` grep `oracle|facts|spec/|ref/|推测` 命中 = 0（"legacy 出处标注"除外）；
2. `crates/ti-app/**` grep `oracle` 命中 = 0（S8.8 已保证，复验）；
3. `ti-core/src/` 的 oracle 桥接层仅测试编译（默认构建 0 行参照代码参与 ti-core lib 编译路径之外的产物）；
4. 根目录无 `TreasureIsl.exe/.i64`、`data/`、`ref/`；`legacy/` 存在；
5. `gate.ps1` 全绿（含新第 15 项 `check_forward_closed.py`，历史时点）。

## 2. 模块边界（并行开发的解耦面）

```
玩法层 ti-core（纯逻辑，零 IO）
  rules/combat      rules/economy        rules/progression(map+cards)
  rules/persistence(save+settings)       rules/rng
表现层 ti-app（场景/控件/渲染/输入）
  scenes/（S9.2 窄接口）  widgets/  gfx/  input/
平台层 ti-platform（窗口/帧/音频/存储）   格式层 ti-formats（pak/pcx/字体/地图/存档）
```

**包依赖铁律（并行开发的解耦前提）**：
- 包内零跨包 import（P1 的 `rules/battle.rs` 只许 import P2 已冻结契约 + P4 数据表）；
- 跨包只经 `balance.json` key + 公共 struct 签名（`rules::rng::GameRng`、`domain::economy::Unit`）；
- 公共 struct 冻结后改动须全票同步（集成轮约定，§6.3）。

| 并行包 | 域 | 依赖 |
|---|---|---|
| P1 玩法-战斗+经济 | `rules/combat`、`rules/economy`、`rules/cards`（结算器） | P2 公共契约 |
| P2 玩法-地图+卡牌抽取 | `rules/progression`、`rules/cards_draw`、`rules/rng` | 无（契约制定者） |
| P3 玩法-存档 | `rules/persistence`（v2 JSON+迁移+RNG 续流） | P1/P2 状态结构稳定 |
| P4 数据+工具 | `assets/data/*.json`、`ti-tools` 校验脚本 | 无（先行） |
| P5 表现层 | `ti-app/scenes`、`widgets`、`gfx`（S9.2+S9.5+S10.4） | P1–P3 会话 API |

## 3. 阶段 A — 逆向产物归档（主线程，1 轮）

### A.1 目录搬迁（git mv，保留历史）

```
legacy/
  TreasureIsl.exe, TreasureIsl.exe.i64   ← 根目录
  data/                                     ← 原版资源（ti-formats 测试 fixture，保留）
  ref/                                      ← 1827 伪代码 + functions*.csv + imports.json
  oracle-rust/                              ← 独立 crate legacy-oracle（Cargo.toml+src/）
  docs/{facts,spec,design,gamebook.md}
  scripts/{ida_*,s0_4_*,classify*,validate_data,gen_layout,rng_sites,...}  ← 17 个逆向脚本
```

**不归档**（正向开发仍用）：`docs/formats/`、`docs/experience/`、`docs/process/`、
`docs/business/`（B 阶段补强对象）、`crates/ti-formats/`、`assets*/`。

### A.2 oracle 隔离（✅ 已落地）

- `legacy/oracle-rust/`：独立 crate `legacy-oracle`（workspace member），
  代码为 `pub use ti_core::` 绝对路径版，**默认不编译**（仅经 ti-core
  dev-dependency 参与测试）；
- `crates/ti-core/src/oracle/`：桥接层，`pub use crate::` 路径版（与 legacy 同源镜像），
  保持 `rules/` 与集成测试路径不变；
- `tests/diff_*.rs` → `reference_diff_*.rs`（对照 legacy 快照）；
- `check_oracle_boundary.py` 第 12 项判据：shell 层 0 `oracle_*` 函数调用（不变）+
  桥接层与 legacy 同源校验（新增，防双份漂移）。

### A.3 引用更新

| 位置 | 改动 |
|---|---|
| `scripts/gate.ps1` 第 10b 项 | `ref/controls.json` → `legacy/ref/controls.json` |
| `scripts/check_ledger_reality.py` | oracle 符号期望改指桥接层/legacy（oracle 移出 ti-core 默认路径后需改判据） |
| `scripts/check_oracle_boundary.py` | 白名单 `crates/ti-core/src/oracle/` 保留（桥接层）+ 新增同源校验 |
| `ti-tools doccheck/doccheck2` | `ref/`、`docs/facts`、`docs/spec`、`docs/design` 路径断言改 `legacy/`；新增「business 零逆向引用」断言 |
| `AGENTS.md` 路由表 + `docs/README.md` | 逆向行改 `legacy/` 前缀 |

### A.4 DoD

- 归档物全部 `git mv` 完成，`git status` 无 untracked 大文件；
- `cargo build --workspace` + `cargo test --workspace` 全绿；
- 默认构建 0 行 oracle 参照代码参与 ti-core 产物（legacy-oracle 仅测试编译）；
- gate 全绿；doccheck 0 error。

## 4. 阶段 B — 业务文档闭环（主线程，1–2 轮）

### B.1 去逆向引用（存量）

| 文件 | 改动 |
|---|---|
| `business/progression.md` | F-307 等距投影公式内联（标注「实测值，出处 legacy/docs/facts/map.md F-307」） |
| `business/00_index.md` | 逆向段缩为一句「legacy/ 存档，仅出处标注」 |
| `business/combat.md/economy.md/persistence.md/loop_core.md` | 复核 0 逆向引用（当前基本干净，验证即可） |

### B.2 新增 `business/07_ui_flow.md`（表现层并行开发唯一依据）

1. **场景流**：场景枚举 + 转换表（菜单→选人→选图→主地图→据点交互→战斗→存读档→回菜单），触发输入与前置条件；
2. **控件规格**：44 种 f5 控件业务归类（按钮/列表/输入/滑动）+ 交互约定；
3. **输入映射**：键盘/触摸双路径 + 手势，场景级操作表；
4. **消息分页**：MsgKind 分类 + 分页规则；
5. **UI 约束**：176×220 逻辑分辨率、等距投影朝向、场景 key 命名（HD 防碰撞）；
6. **可测试规则表**：每条配单测名（沿用 business 编写约定）。

### B.3 重写 `docs/gamebook.md`

- 旧版在 `legacy/docs/gamebook.md`（已归档）；
- 新版定位「**业务→接线手册**」：只引 `business/` + `impl/` + 代码路径，
  删全部 `oracle::`、`spec/` 引用；与 P3-3 16 步脚本对齐。

### B.4 台账登记

S11.1（A）/S11.2（B）进 `plan/ledger.yaml`；`docs/README.md` 登记 `07_ui_flow.md`。

### B.5 DoD

- 闭环判据 1–5 机器断言通过；
- 人工复核：business/ 6+1 文件通读无逆向语境残留。

## 5. 阶段 C — 闭环验证 + 流程固化（主线程，0.5 轮）

1. `scripts/check_forward_closed.py` 固化为 gate 第 15 项（5 条 grep 断言，历史时点）；
2. `gate.ps1` 全绿 → tee `evidence/S11.1.log`；
3. 台账 S11.1/S11.2 verified；
4. **流程固化**：`docs/process/05_module_parallel.md`（§6 loop）+ 子代理任务书
   模板（`docs/process/03_templates.md` 增「模块包任务书」模版）+ `docs/README.md` 登记；
5. 宣布：此后开工路由只看 `business/` + `impl/` + `process/`，`legacy/` 仅出处标注。

## 6. 阶段 D 的 loop：模块化并行开发（S12 起执行）

> 用户观点（2026-10）：真实程序员开发 = 划分模块边界、各自开发、最后集成联调；
> 整个过程提炼成 loop，子代理/会话**模板化**遵守项目规范。
>
> **本节内容已迁移为唯一事实源**：[`process/05_module_parallel.md`](process/05_module_parallel.md)
> （一轮 4 步 + 包划分 + 并行纪律 5 条）；子代理任务书模板见
> [`process/03_templates.md`](process/03_templates.md) §7。
> 本文只保留执行单元决策：**并发会话 + 任务分支为主，子代理只做审计/调研**
> （本仓 Task 工具无写权限子代理；写代码的单位 = 一个会话 + 一支 `task/*` 分支）。

## 7. 交接与接手记录

### 7.1 第一版快照（2026-10-12 会话 A 收尾，未 commit）

> S11-A 物理归档完成，gate 16 项时点 10 PASS + 2 SKIP + 3 FAIL（fmt / clippy / gen_layout），**未 commit**。

**已完成（S11-A 物理归档）**：
- `legacy/` 树就位（exe/i64/data/ref/docs/oracle-rust）；`legacy/oracle-rust/` = 独立 crate `legacy-oracle`（workspace member，`pub use` 桥接层，默认构建 0 行参照代码参与）。
- `ti-core/src/oracle/` → 桥接层（`pub use crate::` 镜像 + `#![allow(missing_docs)]`）；`tests/diff_*.rs` → `reference_diff_*.rs`；`oracle-reference` feature + `legacy-oracle` dev-dep 就位。
- 路径修复全量：`.rs` 的 `join("data")` → `join("legacy").join("data")`；`check_hardcode.py` 严格 CJK 字面量 + EXEMPT 表 → 0 hits；`gate.ps1` 第 10a/10b 项指向 `legacy/scripts/`；`gen_layout`/`validate_data` 改 legacy 路径。
- S10 系列 合并冲突修复：`game_app.rs:draw_terrain` GPU 借用拆分、`e2e_playthrough.rs` API 对齐、`sprite_atlas.rs` 补回 `gpu_blit_scene`、CPU blit 路径解除 feature 门控。
- doccheck 全绿（88 docs / 37 facts / 0 error）；EXP-058 登记补入 `experience/process.md`；ledger 增 S11.1/S11.2（pending）；`release.md` 台账计数 91→93。

### 7.2 接手记录（2026-10-12 会话 B，本文 §7.1 的「新会话第一动作」执行结果）

**已清 gate 4 FAIL**（fmt ✓ / clippy ✓ / gen_layout ✓ / doccheck S10 系列 任务引用 ✓）：
- `cargo fmt --all` + clippy 逐条：`ti-core/data/*` 补 doc + `# Errors`；`gpu_renderer.rs` 模块级 `const IS_WASM` + `allow(cast_precision_loss)`；`scenes/mod.rs` `scene_transition!` 宏（20 条转换 + `Default` 一并生成）；`game_app.rs` `poll` 拆 `poll_main_map`（守 100 行门禁）+ 去 `map_or/usize→u32` 误用；`e2e_playthrough.rs` 冗余闭包改方法引用。
- `legacy/scripts/gen_layout.py` 源路径 `ref/` → `legacy/ref/`（产物 `source` 字段同步，`--check` 幂等）。

**已落地（§7.1 未完成清单 1–5）**：
1. B1 去逆向引用：`business/progression.md` F-307 公式内联（带 legacy 出处标注）、`business/00_index.md` 7 处缩为 1 句「legacy/ 存档」；
2. B2 `docs/business/07_ui_flow.md`（场景流/控件/键盘触摸输入/消息分页/UI 约束/可测试规则表 12 条）；
3. B3 `docs/gamebook.md` 重写（业务→接线 9 节，0 oracle/spec 引用，P3-3 16 步表 + 改动检查清单）；
4. B4 `scripts/check_forward_closed.py` 4 条断言进 gate **第 15 项（历史时点）**（C1 业务零逆向引用 / C2 表现层 0 oracle / C3 根目录清空 + legacy 在位 / C4 桥接层与归档同源；判据 3「默认构建 0 参照代码」= rules→oracle 调用摘除，**未收口，留给 S12 逐模块摘除，本项不假绿**）；
5. D `docs/process/05_module_parallel.md`（一轮 4 步 + P1–P6 包划分 + 并行纪律 5 条 + 台账口径）+ `03_templates.md` §7 任务书模板 + `docs/contracts/{00_index,c1,c2,c3}.md`（draft）+ `plan/packages.yaml` + `scripts/check_module_contracts.py` 进 gate **第 16 项（历史时点）**（白名单互斥 / 契约锚点 / 跨包可见面 / draft 残留 WARN）。

**剩余（会话 C 起点）**：
6. S11-B 验收：gate 16 项全绿（S11 时点，现 20 项）→ tee `evidence/S11.1.log` + `evidence/S11.2.log` → ledger S11.1/S11.2 置 verified（deps 建议改 `[S10.4]`，S10.5 并行不前置）；
7. S12.0 契约冻结轮：`docs/contracts/*` draft→frozen + `plan/packages.yaml` 全票 frozen + 基线标签 `baseline-s12` + 逐包任务书下发；
8. commit：S10 系列 成果 + S11-A + S11-B 一并 commit（legacy 归档 + oracle 桥 + 业务闭环 + gate 15/16 项历史时点），S12 各包另行 commit。

## 8. 开放点（已预设默认）

| # | 决策 | 选择 |
|---|---|---|
| 1 | oracle 隔离形态 | 独立 crate `legacy-oracle`（默认不编译）+ 桥接层路径透明（✅ A.2 已落地） |
| 2 | `legacy/data/` | 保留（ti-formats fixture，0.5MB） |
| 3 | 旧 `gamebook.md` | 归档 legacy，新版 docs/ 根重写（B.3） |
| 4 | 逆向脚本 | 进 `legacy/scripts/` |
| 5 | S12 开工前置 | S11 收口 + S10.5 对抗流水线停机 + 契约冻结轮完成 |
