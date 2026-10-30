# 22 · 开发管理平台（TI Dev Console）定案

> 用户 2026-10-30 拍板：平台**与游戏运行时解耦**，纯管理 + 辅助开发；严格按阶段开发，
> 每个任务完成后起**两个对抗子代理**审计完成度，loop 推进。
> 单一事实源：本文（定位/边界/架构/数据源/协议/安全）；WBS = [`plan-s32-platform.md`](../plan-s32-platform.md)；
> 进度 = `plan/ledger.yaml`；审计协议 = 本文 §7。

## 1. 定位

**是什么**：单机优先的 Web 开发管理台。把仓库里已经存在的事实（台账、文档、证据、资产、门禁输出、
审计报告）变成一屏可读、可查、可执行的界面，并把「双审计 + 收口」固化成一条可追溯的流水线。

**不是什么**（防止范围蔓延）：

- 不是游戏创作平台 / 关卡编辑器 / 运行时调试器（另立项，不在 S32）；
- 不是游戏运行时：不加载 wasm、不碰渲染/玩法/数值；
- 不是 SaaS 多租户（P3 才评估）；
- 不自建数据库（v1 文件系统即数据库，见 §3.1 边界）。

## 2. 用户与场景

| 角色 | 场景 |
|---|---|
| 主线程（开发会话） | 开轮看「下一任务 / 阻塞 / 门禁 / 审计」；收口时看三处同步结果 |
| 审计子代理 | 读平台生成的审计任务书（prompt）与证据路径，输出 findings（只读） |
| 人工 | 浏览器看阶段进度、证据截图、资产与地图包、门禁历史 |

## 3. 设计原则

1. **文件系统即数据库（边界写死）**：
   - 平台**唯一的写点 = `evidence/console/`**（门禁日志、审计报告、截图、执行记录）；
   - 其余仓库文件一律**只读索引**，平台不写台账、不写 `docs/`、不写 `assets/`；
   - 例外（受控执行）：白名单脚本按**各自已登记的行为**写自己的产物（`task_close.py` 写台账 +
     `release.md` 计数、`exp_log.py` 写 `docs/experience/`、`build_wasm.ps1` 写 `web/`），
     平台只负责**触发**并把输出落 `evidence/console/`，不代写。
2. **只读默认**：写操作（跑门禁 / 收口 / 构建 / 审计归档）必须显式触发，走 §4.2 命令白名单。
3. **单一事实源**：业务结论只链接（`EXP-xxx` / `F-xxx` / 业务文档），平台不二次表述、不缓存结论。
4. **零游戏依赖（依赖图可断言）**：
   - 平台依赖面 = `crates/ti-devmeta`（台账解析 + 静态 HTTP 服务，纯 std + `serde_json`）+ `serde_json`；
   - `ti-console` **不得**依赖 `ti-core` / `ti-app` / `ti-engine` / `ti-render` / `ti-platform` / `bevy` / `macroquad`；
   - **为什么需要 `ti-devmeta`**：`ti-tools` 经 `ti-platform(features=["ogg"])` 间接拖入整条游戏栈
     （含 macroquad + opus 编码器），直接依赖 `ti-tools` 会让「零游戏依赖」在依赖图上失效；
     故把 `ledger` 解析与静态 HTTP（现 `ti-tools/src/{ledger.rs,bin/serve.rs}`）上提为轻量 crate，
     `ti-tools` 改为**薄 re-export**（行为不变，`cargo run -p ti-tools --bin ledger` 照旧）；
   - 机器断言：`cargo tree -p ti-console` 输出不得含上述游戏 crate（S32.1 DoD）。
5. **本地优先**：默认 `127.0.0.1`；远程访问必须显式 `--host` + token（§10）。

## 4. 架构

```
Browser（静态 UI，无 npm 构建）
   │ fetch / 轮询（只读 GET 默认；写操作 POST + token）
ti-console（Rust bin）
   ├─ repo adapter：ledger / docs / evidence / assets / maps 只读索引（唯一读点）
   ├─ runner：白名单命令执行（gate / task_close / exp_log / build_wasm / serve）
   └─ audit orchestrator：生成审计任务书 + 归档报告 + 收敛判定（不自己起 LLM 子代理，见 §7.3）
```

- **前端**：`crates/ti-console/web/` 静态文件（原生 JS + CSS），无 node/npm 工具链。
- **后端**：Rust，复用 `ti-devmeta::ledger` 解析台账；其余数据源按需只读读取。
- **状态**：`evidence/console/` 下的日志与审计报告；不引入数据库。

### 4.1 HTTP 选型（定案）

- **v1 = std `TcpListener` + 阻塞式工作线程**（复用 `ti-tools/src/bin/serve.rs` 已成熟做法，上提到 `ti-devmeta::http`）。
- **不引 `axum` / `tokio`**：平台是本地单用户只读台 + 少量受控执行，异步栈带来的依赖体积与
  编译时间不成比例；`serve.rs` 已证明 std 足以支撑静态文件 + JSON 路由 + 轮询。
- 何时重新评估：P1 需要 SSE 长连接流式日志，或 P3 需要多用户并发 → 在 `docs/process/01_architecture.md` §4
  登记新依赖（含「为什么不用现有方案」）后再引入。
- 硬约束：单进程**单实例锁**（`evidence/console/.lock`），同一时刻只允许一个执行任务（§10.4）。

### 4.2 命令白名单（唯一一份，WBS 引用此处，不另列）

| 动作 | 入口 | 参数（只接受枚举/id，不接受自由字符串） | 写面 |
|---|---|---|---|
| 跑门禁 | `scripts/gate.ps1` | `tier ∈ {fast, touch, full}` → `-Fast` / `-Touch` / 无开关 | 平台只写日志 |
| 只读收口体检 | `scripts/task_close.py` | `--task <id> --check` | 无 |
| 收口 | `scripts/task_close.py` | `--task <id> --cmd <白名单 id>...`（id 从 `console_commands.json` 精确映射） | 脚本自身写台账/计数 |
| EXP 查重 | `scripts/exp_log.py` | `--volume --id --check` | 无 |
| 构建 wasm | `scripts/build_wasm.ps1` | 无参数（或 `-Check`） | 脚本自身写 `web/` |
| 本地预览 | `cargo run -p ti-tools --bin serve` | `--dir web --port <1024..65535>` | 无 |
| 地图包校验 | `scripts/map_pack.py` | `verify/check --id <已装包 id>` | 无 |
| 双审计 | §7.3 任务书生成 | `--task <id> --round <n>` | 只写 `evidence/console/` |

- **禁止**：把 HTTP 请求体里的任意字符串透传给 `--cmd`（`task_close.py --cmd` 是自由 shell 面 ⇒ RCE，B-08）；
  `--cmd` 的取值只能是 `console_commands.json` 里登记的**精确命令串**，未登记 id 直接 400。
- **禁止**：`sh -c` / `cmd /c` 拼接；一律 `Command::new(程序).args([...])` 逐参数传递。

### 4.3 Linux / Windows 命令映射

- 平台**不调用 `playbook.ps1`**（它只是 Windows 便捷入口，Linux 无 `powershell` 别名问题）；
- 门禁统一 `pwsh -NoProfile -File scripts/gate.ps1`（Linux 实测环境已有 `pwsh`，见 `docs/migration_linux.md`）；
- Python 解释器按平台选 `python`（Windows）/ `python3`（其他），逻辑同 `gate.ps1` 的 `$py`。

## 5. 数据源（只读）

| 源 | 用途 |
|---|---|
| `plan/ledger.yaml` | 任务 / 阶段 / 依赖 / 阻塞 / 进度 / rounds |
| `plan/packages.yaml` | 模块包划分（并行轮视图） |
| `docs/`（含 `docs/experience/`） | 索引 / 全文检索 / `EXP-xxx` 反查 / 单一事实源链接 |
| `evidence/`（含 `evidence/console/`） | 验收证据 / 截图 / 门禁历史 / 审计报告 |
| `assets/gate_data.json` | 门禁用固定数据（只读展示） |
| `assets/ui_manifest.json` | UI 资产清单（缩略图 + 哈希 + 尺寸） |
| `assets/registry.json` | 字体/音频资产登记（S30.0 全类型登记） |
| `assets/maps/<id>/` + `assets/maps/.default` | 已装地图包 + 当前默认包指针 |
| `dist/maps/*.map-pack` | 可分发件（存在才展示；`dist/` 被 `.gitignore` 忽略，缺失不报错） |

## 6. 页面与能力矩阵

### 6.1 页面（P0 建、P1/P2 扩展）

| 页面 | 内容 | 阶段 |
|---|---|---|
| 概览 | 当前阶段 / 进度条 / 下一任务 / 阻塞清单 / 最近门禁 / 最近审计 | P0（S32.3） |
| 任务 | 台账全表（筛选 status/阶段/rounds），任务详情含 dod + evidence 链接 | P0 |
| 证据 | `evidence/` 浏览 + 图片预览 + 缺失告警 | P0 |
| 审计 | 审计报告列表 + 收敛轮次 + findings 未闭环数 | P0（只读）/ P1（触发） |
| 门禁 | 分档执行 + 轮询日志 + 历史 | P1（S32.4） |
| 资产 | UI 清单 / 地图包 / 校验状态 | P2（S32.7） |
| 文档 | 全文检索 + ID 反查 | P2（S32.8） |
| 构建 | build_wasm / serve / 变更断言 | P2（S32.9） |

### 6.2 能力矩阵

| 阶段 | 能力 | 任务 |
|---|---|---|
| P0 只读骨架 | 平台骨架（`ti-devmeta` + `ti-console`）/ 只读 API / 看板 UI v1 | S32.0~S32.3 |
| P1 执行与审计 | 一键门禁（分档）/ 双审计任务书 + 收敛判定 / 收口 | S32.4~S32.6 |
| P2 辅助开发 | 资产浏览 / 文档检索 / 构建与预览控制 | S32.7~S32.9 |
| P3 部署（可选） | 部署 / 鉴权 / 多工作区 | S32.10（**不计入 loop 终止条件**） |

## 7. 双审计协议（核心纪律）

### 7.1 触发时机

- 每个任务**在置 `verified` 之前**必须跑一轮（硬规则 2 的补充，不替代主线程实跑）；
- 阶段收口跑**阶段级**审计（对整阶段的产物与判据做一次 A/B）。

### 7.2 两个子代理

| 角色 | 视角 | 输出 |
|---|---|---|
| A 完成度审计 | DoD 逐条 → 证据 / 代码 / 实跑输出 | 每条 PASS/FAIL + 缺口 + 复现命令 |
| B 一致性与风险 | 架构 / 规范 / 文档 / 门禁 / 回归 / 安全 / 依赖 | 严重度 + 位置（文件:行） + 建议 |

- 并行、**互不可见**、**只读**（不改文件、不改台账、不跑构建）。
- 必须引用**文件路径 + 行号**或**命令实跑输出**，禁止「看起来没问题」。
- 结论**不等于验收**：主线程必须独立复核（EXP-038 / `docs/loop.md` §6.2 第 8 条）。
- 报告命名区分角色：`evidence/console/audit-<task>-r<N>-a.md`（A）/ `-b.md`（B）。

### 7.3 载体（平台不自己起 LLM 子代理）

平台**没有**能力自行派生 LLM 子代理（那是会话 harness 的能力）。落地方式：

1. 平台生成**审计任务书**：`evidence/console/audit-<task>-r<N>-<role>.prompt.md`（从模板 + 台账 dod + 证据路径填充）；
2. 主线程用 harness 按任务书起两个子代理（A/B 并行、互不可见）；
3. 子代理报告由主线程写回 `evidence/console/audit-<task>-r<N>-<role>.md`（或平台归档粘贴内容）；
4. 平台做**机械判定**：解析报告里的 `总判定：PASS/FAIL` 与 findings 计数 → 收敛轮次表 + 未闭环清单。

> 平台只做「任务书生成 + 归档 + 判定」，**不做**审计本身；这样平台不依赖任何模型 SDK。

### 7.4 收敛判定（引用飞轮口径，不另立）

- 修复 → 复审计；**连续 2 轮 0 新发现** = 通过（与 [`06_flywheel.md`](06_flywheel.md) 的停机判据同源：
  「0 新发现」= 0 新 finding，不含已知项的重复陈述）；
- 3 轮未过 → 按 `docs/loop.md` §4 拆子任务，原任务 `blocked` + `blocked_by` 写原因；
- **与飞轮的分工**：飞轮 = 多角色在全库范围找问题（开轮/停机见 06）；本协议 = **每任务 2 只读审计员**的
  验收前置。两者判据措辞一致，**不新增收敛定义**；平台页面同时展示飞轮轮次与本协议轮次。

### 7.5 主线程独立复核

- 复核动作 = 主线程自己跑 DoD 命令 + 抽查子代理引用的行号/文件确实存在；
- 复核记录写进 `evidence/S32.x.log`（「复核：<命令> → <结果>」），不只写「已复核」。

## 8. 北极星判据（含测量口径）

| 判据 | 阈值 | 测量方式（可复现） |
|---|---|---|
| 首屏可见关键信息 | < 2 s | `curl -s -o /dev/null -w '%{time_total}' http://127.0.0.1:8787/api/overview`（warm，连做 5 次取中位数）+ 浏览器 `performance.now()` 打印 DCL 到 `/api/overview` 渲染完成；结果落 `evidence/console/dashboard-perf.log` |
| 门禁可见可查 | — | 分档执行 + 日志轮询 + `evidence/console/gate-<ts>.log` 历史列表 |
| 每任务双审计可追溯 | — | 每轮 A/B 报告落盘 + 收敛轮次表 + 未闭环 findings 计数 |
| 平台自身质量 | 0 违规 | 既有门禁自动覆盖（见 §11.2）+ `cargo test -p ti-console` 实跑 |
| **0 游戏产物变更** | 0 差异 | `git diff --exit-code -- crates/ti-core crates/ti-app crates/ti-engine crates/ti-render crates/ti-platform assets/ android/ tests/` + `web/` 全量 sha256 清单前后对比（`web/` 被 `.gitignore` 忽略，`git status` 不能作为证据） |

## 9. 不做清单

- 不做游戏创建、关卡编辑器、运行时调试器（除非另立项）。
- 不做账号体系、多租户、计费（P3 再评估）。
- 不做任意 shell：只跑 §4.2 白名单动作 + 枚举/id 参数。
- 不引入数据库：v1 用仓库文件（§3.1 写面边界）。
- 不在平台里复述业务结论：只链接单一事实源。
- 不在平台内嵌 LLM SDK / 模型调用（§7.3）。

## 10. 安全与边界

### 10.1 网络面

- 默认 `127.0.0.1:8787`；`--host` 非回环地址时**必须**同时给 `--token`，否则拒绝启动。
- 请求方法白名单：只读 `GET`；写操作 `POST` + `X-TI-Token` 头（token 由 `--token` 或
  `evidence/console/token` 提供，启动时若不存在则生成并 `0600`）。
- 请求体上限 64 KiB；只接受 `Content-Type: application/json`。

### 10.2 路径面

- 所有文件访问经 `repo` 适配层：`canonicalize` 后必须仍以**仓库根**为前缀（防 `..` / 符号链接逃逸）；
- 只暴露显式白名单目录（`docs/`、`evidence/`、`plan/`、`assets/`、`dist/maps/`）；
- `assets/` 只返回登记过的文件（`ui_manifest.json` / `registry.json` / `maps/` 内 manifest 引用），
  不做任意目录列举。

### 10.3 执行面

- 只执行 §4.2 表内动作；参数只接受枚举/id（§4.2 明列禁止项）；
- 子进程 `env` 白名单透传（不继承全部环境变量），工作目录固定仓库根；
- 每次执行记录：动作 id / 参数 / 起止时间 / 退出码 / 输出落盘路径 → `evidence/console/exec-<ts>.log`。

### 10.4 并发与回滚

- **单实例锁**：`evidence/console/.lock`（创建即占用，退出删除）；占用中再触发执行 → 409 + 当前任务 id；
- **不做自动回滚**：受控执行可能改台账/`web/`，回滚 = 人工 `git checkout` / 重跑构建；
  平台只保证「执行前记录 `git rev-parse HEAD` + 执行后 `git status --short` 快照」进日志，供人工判断；
- **证据保留**：`evidence/console/` 下日志按类型保留最近 50 份（超出按时间删最旧，删除动作记 `retention.log`）；
  审计报告**不自动清理**（验收追溯用）。

## 11. 视觉与质量口径

### 11.1 视觉改动（硬规则 9 适用）

平台 UI 同样是「表现层视觉改动」⇒ **先出静态原型截图 `evidence/console/ui-<stage>.png`，用户确认留痕后才落码**；
确认留痕口径同 `doccheck2::check_visual_design_signoff`（证据里含「视觉验收」+ 确认来源）。

### 11.2 平台自身质量（**不新增门禁顶层项**）

- 理由：`crates/ti-console` 与 `crates/ti-devmeta` 注册为 workspace 成员后，
  **自动被既有门禁覆盖** —— `cargo fmt --check` / `cargo clippy --workspace` / `cargo test --workspace`
  / 占位扫描（`crates/**/*.rs`）/ `unwrap` 扫描 / `check_code_standards.py`（遍历 `crates/`）。
  再加一个「平台」顶层项 = 重复劳动 + 改变门禁项数口径（项数唯一事实源 = `docs/parity.md`），无收益。
- 补充断言：S32 各任务的 DoD 各自要求 `cargo test -p ti-console`（或 `-p ti-devmeta`）实跑 + 依赖图断言 +
  `0 游戏产物变更`（§8 测量方式）。

## 12. 工具与登记纪律

- 新增 `scripts/*` 工具：按 `docs/process/14_tooling_catalog.md` 3 处登记 + 工具契约（`check_tooling_contract`）；
- 新增 cargo bin（`ti-console` / `ledger` 等）：登记 `14_tooling_catalog.md` §3.2 长期工具行 +
  `docs/process/04_tooling.md` 速查表 + `docs/README.md` 索引；
- 平台代码受 `docs/process/16_code_standards.md` 约束（新代码零豁免）；CS-1 资产路径读点例外只在
  `crates/ti-console/src/repo/`（平台唯一读点，类比 `crates/ti-tools/`），扩豁免时须同步 16 号文档。
