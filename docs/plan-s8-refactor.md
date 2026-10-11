# S8 改造计划（Refactor Plan）

**目标**：把项目从「逆向驱动的功能性复刻」改造为**标准游戏工程**。
手段按「固化 / 模版化 / 脚本化 / 复用现成库」分派，交给 LLM、脚本与 loop 各司其职。

## 1. 定位变更（前置决策，2026-10）

| 对象 | 旧定位 | 新定位 |
|---|---|---|
| `docs/facts/`、`ref/pseudocode/` | 开发依据 + 验收基准 | **参照物**（可查原版形态，不驱动开发） |
| `docs/spec/*.md`（逆向规格） | 业务规则来源 | 参照物（历史归档） |
| `crates/ti-core/src/oracle/` | 差分测试基准（自证） | **冻结**，不再新增；新系统不写 oracle |
| `crates/ti-core/src/rules/` | 逆向转写实现 | **重写**为按 `docs/business/` 实现 |
| `crates/ti-app/game_app.rs` | 21 个重复方法 | 重构为 Scene trait + 场景注册表 |
| 数值 / 文本 | 硬编码在 `.rs` | `assets/data/*.json` |
| 控件 | `fill_rect` 手绘 | 控件层驱动（`ref/controls.json` → data → widgets） |

**关键澄清**：资源格式层（pak/PCX/字体/`.Map`/存档）是**实测事实**，保留不改；
要重写的是**玩法语义**（那 67 处「推测」，2026-10-10 实数含 oracle+rules+tests）。

## 2. 改造清单（按优先级）

| # | 改造项 | 手段 | 落点 | 判据 |
|---|---|---|---|---|
| 1 | 业务流程文档独立 | **固化** | `docs/business/*` | 业务规则不引用 oracle/facts，数值指向 balance.json |
| 2 | 工程流程文档 | **固化** | `docs/process/*` | 分工表 + 模板 + 工具链齐备 |
| 3 | 数值/文本数据化 | **固化** | `assets/data/*.json` | `check_hardcode.py` 0 命中（src 无中文/坐标/配色） |
| 4 | 场景重构（21→注册表） | **模版化** | `ti-app/src/scenes/` + `Scene` trait | 每个场景 1 个 struct，键盘/触摸共用一份逻辑 |
| 5 | 控件层（2041 条接入） | **脚本化** | `gen_layout.py` + `ti-app/src/ui/widgets/` | 控件表驱动渲染，替代手绘 |
| 6 | 玩法重写（去「推测」） | **固化 + LLM** | `ti-core/src/rules/` | 每模块含「设计意图 + 数值注入 + 可复现」单测；oracle 冻结 |
| 7 | 渲染基线改造 | **固化** | `render_parity` | 参考图用**真实资源解码**产出，不自指 |
| 8 | 工具链固化 | **脚本化** | `scripts/*.py` + `ti-tools` | 幂等 + `--check`；新增门禁第 10 项 |
| 9 | 库复用 | **复用库** | `Cargo.toml` | `rand_pcg`/`fnv` 替手写 LCG/FNV；禁造轮子 |

## 3. 分波次与台账任务

> 每个任务 = 一次可独立验收的改动（见 `process/00_index.md` §任务粒度）。
> 建议按 S8.0→S8.7 顺序推进，deps 串联。

| 任务 | 标题 | 手段 | deps | 判据（dod 摘要） |
|---|---|---|---|---|
| **S8.0** | 流程 + 业务文档落地 | 固化 | — | `docs/process/*` + `docs/business/*` 齐备，doccheck 全绿 |
| **S8.1** | 数值/文本抽 `assets/data/*.json` + 加载层 | 固化 | S8.0 | balance/strings/scenes JSON 落地 + 加载器 + 单测；`.rs` 数值改从 JSON 读 |
| **S8.2** | 工具链：`gen_layout` + `validate_data` + `check_hardcode` + 门禁第 10 项 | 脚本化 | S8.1 | 三个脚本幂等 + `--check`；`check_hardcode` 进 gate 且能抓出现存硬编码 |
| **S8.3** | 玩法重写：13 模块按 `docs/business/` 重写（分 5 子任务 `.a`–`.e`） | 固化+LLM | S8.1 | 每模块：设计意图注释 + 数值注入 + 可复现单测；`docs/spec` 只作参照 |
| **S8.4** | 场景重构：Scene trait + 注册表 + 7 场景 | 模版化 | S8.3 | `game_app.rs` 21 个重复方法 → 每场景 1 struct；键盘/触摸共用 `on_event` |
| **S8.5** | 控件层：2041 条控件 → widget 渲染 | 脚本化+模版化 | S8.2, S8.4 | 控件表驱动；`fill_rect` 手绘清零；控件单测（墨点/命中） |
| **S8.6** | 渲染基线改造（去自指） | 固化 | S8.4 | 参考图由真实资源解码独立产出；渲染断言对真实基线 |
| **S8.7** | 库复用收尾 | 复用库 | S8.3 | `rand_pcg` 替手写 LCG；`fnv` 替手写；`docs/process/01_architecture.md` §4 清单落地 |

> 波次内可并行：S8.3 各模块（文件域独立）；S8.5 与 S8.6 可在 S8.4 后并行。

## 4. 不做的事（明确边界）

1. **不再写新 oracle**：`crates/ti-core/src/oracle/` 冻结，仅供历史 diff 参考；
2. **不再以「推测」进代码注释**：推测要么验证、要么进 `docs/business/` 并给验证方法；
3. **不推翻资源格式层**：pak/PCX/字体/`.Map` 是实测事实，继续用；
4. **不追求 1:1 复刻原版**：目标是**好玩的同类游戏**，业务规则自主设计、可调；
5. **不删现有能跑的渲染/输入**：改造是重构不是重写，渐进替换。

## 5. 与旧阶段的关系

- S0–S7 已 `verified` 的**资产层**（格式解析、三端构建、资源管线、门禁）**全部保留**；
- S3 的 `rules/*` 因含「推测」进入 S8.3 重写队列；
- `parity.md` 的 P0/P4 判据继续有效（格式/渲染对齐是真事实），
  但**业务对齐不再以 oracle diff 为准**，改以 `docs/business/` 规则 + 单测为准；
- `loop.md` 循环机制不变，只是「下一个任务」来源从 `plan-s4/s5` 增补 S8 队列。

## 7. 执行排期（子任务派发，2026-10 补）

> 代码已写完、待验收（子会话跑测试 + 补 `evidence/` + 置 ledger verified）；
> 文件域不相交的子任务可**并行**，按依赖拓扑分波。
> 每子任务独占文件域，其他子任务**不得改**其文件域；改共享文件只许最小化注册行。

### 7.1 任务状态基线

| 子任务 | ledger ID | 代码状态 | 待办 |
|---|---|---|---|
| 战斗 | S8.3.a | 已完 | 补 evidence + 置 verified |
| 存读档 | S8.3.d | 已完 | 3 个测试文件对齐新 JSON v2 API（主线程已改，子会话跑通） |
| 菜单选择 | S8.3.e | 已完 | 删/改残留 diff 测试（引用 rules 新类型 vs oracle 旧类型） |
| 场景骨架 | S8.4 | 已完 | 修 `game_app.rs` E0592/E0616/E0624（私有化 + `dyn SceneHost` 回调） |
| 控件层 | S8.5 | 已完 | 修 `widgets.rs`（`Framebuf` 无 `fill_rect` → `MenuShell` + 类型转换） |
| RNG/FNV | S8.7 | 已完 | 删 `Cargo.toml` 未用的 `rand`；`build_assets.rs` 的 `fnv1a64_hex` 已修 |
| 渲染基线 | S8.6 | 未完 | 修 `render_parity.rs` E0596 + 参考图改由 `ti-formats` 独立产出 |

### 7.2 文件域隔离（并行波）

**波 1（4 子会话并行，文件域互不相交，deps 全为 S8.1 verified）：**

| 子任务 | 独占文件域（其他子任务禁改） |
|---|---|
| A 战斗 | `crates/ti-core/src/rules/battle.rs`、`crates/ti-core/tests/rules_battle.rs`、`evidence/S8.3.a.log` |
| B 存读档 | `crates/ti-core/src/rules/saveload{,_json}.rs`、`crates/ti-core/tests/{rules_saveload,rules_saveload_json,save_migration}.rs`、`evidence/S8.3.d.log` |
| C 菜单选择 | `crates/ti-core/src/rules/{menu,selection,main_loop,messages,render}.rs`、`crates/ti-core/tests/diff_{menu,selection,main_loop,render}.rs`、`evidence/S8.3.e.log` |
| F RNG/FNV | `crates/ti-core/src/rules/rng.rs`、`crates/ti-tools/src/hash.rs`、`crates/*/Cargo.toml`、`evidence/S8.7.log` |

**波 2（deps S8.3 父 verified，波 1 完成后）：**

| 子任务 | 独占文件域 | 已知残留错误 |
|---|---|---|
| D 场景骨架 | `crates/ti-app/src/game_app.rs`、`crates/ti-app/src/scenes/*`、`crates/ti-app/tests/*`、`evidence/S8.4.log` | `E0592`（`scene` 字段/方法名冲突）、`E0616`（私有字段跨模块访问）、`E0624`（`handle_X`/`render_X` 私有，`scenes/*.rs` 直接调 → 改经 `&mut dyn SceneHost` 回调） |

**波 3（deps S8.4，可 2 并行）：**

| 子任务 | 独占文件域 | 已知残留错误 |
|---|---|---|
| E 控件层 | `crates/ti-app/src/widgets.rs`、`crates/ti-app/tests/widgets.rs`、`evidence/S8.5.log` | `widgets.rs`：`Framebuf` 无 `fill_rect` → 改 `MenuShell`；`color_of` 的 `rec.kind % palette.len()` u32/usize 转换；`hit()` 需 `.collect().rev()` |
| G 渲染基线 | `crates/ti-app/tests/render_parity.rs`、`evidence/S8.6.log` | `E0596`（`a` 未 `mut`）；参考图需由 `ti-formats` 资源解码独立产出（去自指） |

**收尾（主线程）：** 各子任务文件域全绿 → 按 `loop.md` §6.2 第 8 条复核（5 类产出物 + `cargo test` + ledger + gate）→ `S8.3` 父任务全子任务 verified → `gate.ps1` 13 项全绿。

### 7.3 各子任务 DoD 验收命令

| 子任务 | DoD 命令 | 通过判据 |
|---|---|---|
| A 战斗 | `cargo test -p ti-core --test rules_battle` | `test result: ok`，0 failed |
| B 存读档 | `cargo test -p ti-core --test rules_saveload --test rules_saveload_json --test save_migration` | 全 `ok` |
| C 菜单选择 | `cargo test -p ti-core --test diff_menu --test diff_selection --test diff_main_loop --test diff_render` | 全 `ok`（残留 diff 已删/改） |
| D 场景骨架 | `cargo test -p ti-app` | 全 `ok`，且 `game_app.rs` 无 `handle_X`/`touch_X`/`render_X` 三连 |
| E 控件层 | `cargo test -p ti-app --test widgets` | 全 `ok` |
| F RNG/FNV | `cargo build --workspace && cargo clippy --workspace --all-targets -- -D warnings` | 无未用 `rand` 警告，无手写 LCG/FNV 残留 |
| G 渲染基线 | `cargo test -p ti-app --test render_parity` | 全 `ok`，基线来自 `ti-formats` 资源解码 |
| 收尾 | `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` | 13 项全绿，`tee evidence/<id>.log` |

### 7.4 备注（主线程已顺手改、留待子会话验收）

- `rules_saveload.rs` / `rules_saveload_json.rs` / `save_migration.rs`：`GameRng::next()` 改 `next_u32()`、`a/b` 加 `mut`；`SaveManager` 旧 API 调用改新 JSON v2 API（B 子任务跑通即可）。
- `build_assets.rs`：`fnv1a64(&bytes)` 改 `fnv1a64_hex(&bytes)`（manifest 值为 `String`）（F 子任务确认）。
- 不强制回退；各子会话在波 1 验收时一并跑通。

## 8. 风险与对策

| 风险 | 对策 |
|---|---|
| 重写玩法回归（改动面大） | 保留旧 rules 作参考实现，逐模块重写+单测，测试先行 |
| 数值搬迁遗漏 | S8.2 `check_hardcode` 机器抓残留，不靠人眼 |
| 控件层与现有渲染不兼容 | S8.5 单列任务，特征开关 `controls-v2`，可回退 |
| 循环推进变慢（重构任务大） | 任务粒度切小（process §任务粒度），rounds≥3 强制拆子任务 |