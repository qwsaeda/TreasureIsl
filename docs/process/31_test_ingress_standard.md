# 31 · 测试接入标准（游戏测试平台管理固化）

> **本文件 = 测试接入标准的唯一事实源**。触发：用户 2026-11-06
> 「游戏测试也应该平台管理起来，固化一个标准——新增加脚本只能按标准写测试代码接入到平台中就行了」。
> 用户拍板（2026-11-06）：① 范围 = 定案 + 落地；② 标准深度 = **完整标准**（schema 契约 + 登记流程 +
> 平台接入口径 + 门禁断言）；③ 与 S44 关系 = **S44 后独立**。
>
> 关联（单一事实源引用，不复述）：`23_game_testing.md`（测试标杆：分级/分类/类型/行业参照 + 平台测试页）、
> `22_dev_platform.md`（平台 §10.3 执行面白名单）、`14_tooling_catalog.md`（工具登记）。
> 进度 = `plan/ledger.yaml`；WBS = [`plan-s45-test-ingress.md`](../plan-s45-test-ingress.md)。

## 1. 定位

**是什么**：把「新增测试按标准写 + 登记 `assets/data/test_manifest.json` → 平台 `/api/tests` 自动发现 +
`?run=<id>` 白名单执行」**固化**成标准。平台测试页已 data-driven（读机读清单），登记即接入、
**零平台 Rust 改动**；本文件 = 该接入口径的 schema 契约 + 登记流程 + 门禁断言。

**不是**（防止范围蔓延，镜像 23 号 §6「不做什么」）：

- 不是重放断言（运行测试走 test_manifest command 本体）；
- 不做任意 shell（command 程序白名单 + 逐参数非 shell，22 号 §10.3）；
- 不内嵌测试框架（测试本体 = 既有 cargo test / 脚本，平台只编排）。

## 2. schema 契约（`test_manifest.json` `tests[]`）

| 字段 | 必填 | 契约 |
|---|---|---|
| `id` | ✓ | 唯一（无重复）；`^[a-z0-9_]+$`（平台 `?run=<id>` 枚举 id） |
| `level` | ✓ | `L1`（单元）`L2`（集成/模块）`L3`（系统/流程）`L4`（验收/端到端）——23 号 §2 分级 |
| `category` | ✓ | `functional` `regression` `visual` `audio` `performance` `compatibility`——23 号 §3 分类 |
| `type` | ✓ | `smoke` `ci` `playtest` `exploratory` `benchmark`——23 号 §4 类型 |
| `name` | ✓ | 人类可读名（平台测试页展示） |
| `desc` | ✓ | 测试用例说明（做什么/测什么/达到什么效果，≥20 字符；平台测试页展开渲染，S49.7 补强（2026-11-10 用户「光有测试用例不行，要有说明」）——「光有测试用例不行，要有说明」） |
| `command` | ✓ | 可运行命令串（§3 command 契约）；`null`/缺 = 不可自动运行（人工验收 / gate 覆盖 / 证据基，平台禁 run） |
| `evidence` | 与 `gate` 至少一者 | `evidence/` 下日志/截图路径（须存在，`validate_data` 对账） |
| `gate` | 与 `evidence` 至少一者 | 门禁项号 `1..=33`（证据 = 对应 gate 运行日志） |
| `group` | ✓（S49.7 起） | **业务域**：`core`（核心规则/玩法）`asset`（资产与内容管线）`platform`（平台与工具）`device`（端侧与兼容）`perf`（性能与体积）——平台测试页主分组维度（23 号 §6.3 侧栏 + §7.2） |
| `timeout_s` | ○ | 作业超时秒数（缺省不限；超时 ⇒ 作业 `exit_code=-1` + note `timeout`，23 号 §6.2 收割） |
| `prove` | ✓（可自动运行者） | 本次运行**证明落点**（`evidence/tests/<id>/` 前缀）；与静态基线 `evidence` 并存不冲突（23 号 §6.3「证明」行） |

- **`evidence` XOR `gate` 语义**：`evidence` 指向独立证据日志（如 `evidence/S13.3.log`）；
  `gate` = 该测试由门禁项覆盖（如 `web_playtest` gate=23）；两者至少一者（都设 = 双证据，允许）。

## 3. command 契约（可自动运行测试）

| 规则 | 契约 |
|---|---|
| 程序白名单 | 首词 ∈ `cargo` `python3` `pwsh` `powershell`（镜像平台 `run_test` 白名单，22 号 §10.3；`gate`/`smoke_ps1` 用 `powershell`，Linux 映射 `pwsh`） |
| 非 shell 拼接 | command 串**逐参数拆分**（空格，无引号嵌套——平台 `Command::new(程序).args([...])`，禁 `sh -c`/`cmd /c`） |
| 命令真实性 | `cargo test -p <pkg> --test <name>` → `crates/<pkg>/tests/<name>.rs` 存在；`cargo run -p <pkg> --bin <bin>` → `crates/<pkg>/src/bin/<bin>.rs` 存在（防清单指向已删/改名测试，`validate_data` 已有） |
| 不可自动运行 | `command` = `null`/缺/非命令描述（如「用户视觉验收（设计稿先行，硬规则 9）」；`visual_signoff` 例；`web_playtest`/`bevy_cpu_parity`/`wasm_browser_e2e` = gate 覆盖）→ 平台禁 run（无 command 字段面；23 号 §6.1「门禁仍由 gate.ps1 触发，平台只读其结果」） |

## 4. 登记流程（新增测试接入平台）

1. **写测试**：按 23 号分级/分类/类型写测试（`crates/<pkg>/tests/<name>.rs` 或 `scripts/` 脚本），
   判据/证据按既有测试范式。
2. **登记**：`test_manifest.json` `tests[]` 加一行（`id`/`level`/`category`/`type`/`name`/`command`/
   `evidence` 或 `gate`），字段按 §2 schema。
3. **门禁**：`validate_data::check_test_manifest` 全过（§6 断言；登记违规非 0）。
4. **平台自动接入**：`/api/tests` 读清单 → 新测试自动出现在平台测试页分组树（**零平台 Rust 改动**）。
5. **运行**：`/api/tests?run=<id>` → 平台白名单 id 枚举 → `Command::new` 逐参数 spawn →
   日志落 `evidence/console/logs/test-<id>-<ts>.log`（gitignored）。
6. **证据**：测试实跑输出 / 截图落 `evidence/`；`evidence` 字段指向之（与清单对账）。

## 5. 平台接入口径（data-driven，零平台代码）

- 平台测试页 = **读 `test_manifest.json`**（`repo::data::tests`，白名单数据源路径）；登记即接入，
  **不改 `ti-console` Rust**。
- 平台执行 = **白名单 id 枚举**（`?run=<id>`，`<id>` 必须登记于 `tests[].id`；未登记 400）+ 程序白名单
  （§3 首词）+ 逐参数非 shell（22 号 §10.3）。
- **零平台代码断言**（S45.1 判据）：新增测试登记后，`git diff crates/ti-console/` = 空（平台未改）。

## 6. 门禁断言（`validate_data::check_test_manifest` 强化）

现有（23 号 §7 已落地）：`id` 唯一 / `level`∈L1~L4 / `category` / `type` / `evidence` 与 `gate` 至少一者 /
evidence 路径存在 / `gate`∈1..=33 / command 真实性（--test/--bin 路径存在）。

**S45.1 强化**：
- **command 程序白名单**：`command` 首词 ∈ §3 白名单（登记 `sh`/`bash`/`cmd`/裸脚本 = 违规，平台跑不了）；
- **command 无 shell 拼接面**：command 含 `;`/`&&`/`|`/`>`/反引号/`$(` = 违规（shell 拼接面，禁 22 号 §10.3）；
- **id 格式**：`^[a-z0-9_]+$`（`?run` 枚举面）；
- **name 必填**：缺 `name` = 违规（平台测试页展示面）。
- **desc 必填**：缺 `desc`（做什么/测什么/达到什么效果，≥20 字符）= 违规（S49.7 补强；`validate_data::check_test_manifest` 断言；平台测试页展开渲染说明）

**S49.7 强化**（平台测试页分组分类 + 逐个运行 + 出证明）：
- **group 必填 + 值域**：`group` ∈ `core|asset|platform|device|perf`（缺 = 违规；平台测试页主分组维度，23 号 §6.3/§7.2）；
- **prove 路径形状**：可自动运行者 `prove` 须以 `evidence/tests/` 开头（本次运行证明落点约定，23 号 §6.2 证明行）；
- **timeout_s 合理性**：`timeout_s` 若给出须为正整数（≤ 7200）；
- **零平台代码断言续**：新增测试登记后 `git diff crates/ti-console/` = 空（平台未改；仅 `test_manifest.json` + 测试文件变化）。

## 7. 关联与登记

- 单一事实源引用（不复述）：`23_game_testing.md`（分级/分类/类型/行业参照/平台测试页）、
  `22_dev_platform.md`（§10.3 执行面）、`14_tooling_catalog.md`（测试脚本登记）。
- 登记：`docs/README.md` + `AGENTS.md` 路由表 + `docs/process/00_index.md` 目录清单。
- 23 号 §7 补「测试接入标准」段（指向本文件 + §2/§3 契约摘要，不复述）。
