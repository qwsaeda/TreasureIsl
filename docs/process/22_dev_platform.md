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
- 不自建数据库（v1 文件系统即数据库，见 §3.1 边界）；
- 不做「重新构建游戏产物」的前端（见 §9：只提供只读体检与预览）。

## 2. 用户与场景

| 角色 | 场景 |
|---|---|
| 主线程（开发会话） | 开轮看「下一任务 / 阻塞 / 门禁 / 审计」；收口时看三处同步结果 |
| 审计子代理 | 读平台生成的审计任务书（prompt）与证据路径，输出 findings（只读） |
| 人工 | 浏览器看阶段进度、证据截图、资产与地图包、门禁历史 |

## 3. 设计原则

1. **文件系统即数据库（写面边界写死）**
   - **平台自身**的唯一写点 = `evidence/console/`（审计报告 + 执行日志；运行期日志落
     `evidence/console/logs/`，该子目录在 `.gitignore` 内 ⇒ 清理不脏仓库）；
   - 其余仓库文件一律**只读索引**：平台自身不写台账、不写 `docs/`、不写 `assets/`；
   - **受控脚本**按各自已登记的行为写自己的产物，写点见**已知写点表**（下表；新增白名单动作须同步本表，
     表外写点若被发现 ⇒ 补表后再上线该动作）：

     | 动作 | 脚本写点（git 跟踪，除注明外） | 备注 |
     |---|---|---|
     | 跑门禁 `scripts/gate.ps1` | `evidence/web_playtest.log`、`evidence/web_playtest.png`、`evidence/font_render.txt` | **实测**：第 23 项 browser-playtest **无分档守卫**（三档都会执行）⇒ 跑一次门禁必改前两个跟踪文件；`font_render.txt` 由 `cargo test --workspace` 的 `ti-formats` font_parity 测试写（内容确定 ⇒ 无 diff，但写点客观存在） |
     | 只读收口体检 `scripts/task_close.py --check` | 无 | 只读 |
     | 收口 `scripts/task_close.py` | `evidence/<task>.log`、`plan/ledger.yaml`（任务行）、`docs/release.md`（计数） | 平台**默认只开放 `--check`**；写入口受 §4.2 白名单 id 约束 |
     | EXP 查重 `scripts/exp_log.py --check` | 无 | **当前不开放写入入口**（将来开放须同步本表 + §4.2） |
     | 构建体检 `scripts/build_wasm.ps1 -Check` | 无 | 只读（不构建/不拷贝/不写 evidence）；**平台不执行完整构建** ⇒ 不会改写 `web/` 与 `evidence/S18.8a-wasm-size.log` |
     | 地图包校验 `scripts/map_pack.py verify/check` | 无（只读） | 同脚本的写子命令**平台一律不开放**：`install/uninstall` 写 `assets/maps/<id>/` + `assets/maps/.default`（**条件写**：install 缺时 / uninstall 仅卸默认包时）+ `assets/maps/index.json` + `assets/data/board.json`；`add` 写 `assets/maps/<id>/`；`sync` 写 `assets/data/{board,scenes,strings}.json` + `assets/data/ui_schema/scene_manifest_v3.json`（写点清单以 21 号 §8 为准） |

2. **只读默认**：写操作（跑门禁 / 收口体检 / EXP 查重 / 审计归档）必须显式触发，走 §4.2 命令白名单。
3. **单一事实源**：业务结论只链接（`EXP-xxx` / `F-xxx` / 业务文档），平台不二次表述、不缓存结论。
4. **零游戏依赖（依赖图可断言）**：
   - `crates/ti-devmeta` = **纯 std**（台账解析 + 静态 HTTP 服务，从 `crates/ti-tools/` 上提，单实现）；
   - `crates/ti-console` = `ti-devmeta` + `serde_json`（仅 JSON 编解码）；
   - `ti-console` **不得**依赖 `ti-core` / `ti-app` / `ti-engine` / `ti-render` / `ti-platform` / `bevy` / `macroquad`；
   - **为什么必须抽 `ti-devmeta`**：`ti-tools` 经 `ti-platform(features=["ogg"])` 间接拖入整条游戏栈
     （含 macroquad + opus 编码器），直接依赖 `ti-tools` 会让「零游戏依赖」在依赖图上失效；
     `ti-tools` 改为**薄 re-export**（行为不变，`cargo run -p ti-tools --bin ledger` 照旧输出不变）；
   - 机器断言：`cargo tree -p ti-console` 输出不得含上述游戏 crate（S32.1 DoD）。
5. **本地优先**：默认 `127.0.0.1:8787`；远程访问必须显式 `--host` + token（§10.1）。

## 4. 架构

```
Browser（静态 UI，无 npm 构建）
   │ fetch / 轮询（只读 GET 默认；写操作 POST + X-TI-Token）
ti-console（Rust bin；依赖 = ti-devmeta + serde_json）
   ├─ repo adapter：ledger / docs / evidence / assets 只读索引（唯一读点）
   ├─ runner：白名单命令执行（gate / task_close --check / exp_log --check / build_wasm -Check）
   └─ audit orchestrator：生成审计任务书 + 归档报告 + 收敛判定（不自己起 LLM 子代理，见 §7.3）
```

- **前端**：`crates/ti-console/web/` 静态文件（原生 JS + CSS），无 node/npm 工具链。
- **后端**：Rust，复用 `ti-devmeta::ledger` 解析台账；其余数据源按需只读读取。
- **状态**：`evidence/console/` 下的日志与审计报告；不引入数据库。

### 4.1 HTTP 选型（定案）

- **v1 = std `TcpListener` + 每连接一线程的阻塞模型**（原型：`crates/ti-tools/src/bin/serve.rs`）。
- **为什么够用**：`serve.rs` 已实测证明「std `TcpListener` accept 循环 + 静态文件响应」可行；
  平台新增的只是 **JSON 路由 / POST / 请求体上限 / 轮询** 四个面 —— 在同一 std 线程模型上补齐，
  本地单用户场景不需要异步栈。
- **不引 `axum` / `tokio`**：异步栈带来的依赖体积与编译时间与收益不成比例；且 §3.4 要求零游戏依赖面最小化。
- 何时重新评估：P1 需要 SSE 长连接流式日志，或 P3 需要多用户并发 ⇒ 在 `docs/process/01_architecture.md` §4
  登记新依赖（含「为什么不用现有方案」）后再引入。
- 硬约束：单实例锁（§10.4），同一时刻只允许**平台自己发起**的一个执行任务。

### 4.2 命令白名单（人读视图；机器真源 = `crates/ti-console/console_commands.json`）

> **两份必须一致**：本节是**人读视图**，`console_commands.json` 是**机器真源**；
> 两者动作 id 集合必须相等 ⇒ S32.6 DoD 加**对账断言**（集合一致；不一致即 FAIL）。

| id | 入口 | 参数（只接受枚举/id，不接受自由字符串） | 写面 | 默认启用 |
|---|---|---|---|---|
| `gate` | `pwsh`/`powershell` + `-File scripts/gate.ps1` | `tier ∈ {fast, touch, full}` → `-Fast` / `-Touch` / 无开关 | 门禁既有副作用：`evidence/web_playtest.{log,png}`、`evidence/font_render.txt`（§3.1） | 是 |
| `close_check` | `python3` + `scripts/task_close.py` | `--task <台账 id> --check` | 无 | 是 |
| `close_run` | `python3` + `scripts/task_close.py` | `--task <台账 id> --cmd <白名单 id>...`；id 从 `console_commands.json` 精确映射为**固定命令串** | 脚本自身写台账 + 计数 + 证据日志（机器真源写面 = `[plan/ledger.yaml, docs/release.md, evidence/前缀]`，`evidence/` 为前缀形态：运行时按台账 id 展开，S32.6 对账按前缀匹配） | **否**（需 `--enable-write` 显式开启） |
| `exp_check` | `python3` + `scripts/exp_log.py` | `--volume <枚举> --id <EXP-id> --check` | 无 | 是 |
| `build_check` | `pwsh`/`powershell` + `-File scripts/build_wasm.ps1 -Check` | 无 | 无 | 是 |
| `serve_web` | 平台自身静态服务（`ti-devmeta::http`） | `--dir web`（目录枚举固定） | 无 | 是 |
| `mappack_verify` | `python3` + `scripts/map_pack.py verify <path>` | `path` = 平台按**自己的已装包扫描**（`assets/maps/*/manifest.json` 存在且目录名 ≠ `_template`）内部拼出的 `assets/maps/<id>` | 无 | 是 |
| `mappack_check` | `python3` + `scripts/map_pack.py check` | 无 | 无 | 是 |
| `audit_book` | 平台自身 | `--task <台账 id> --round <n> --role <a\|b>` | 只写 `evidence/console/` | 是 |

- **默认启用列的机器真源** = `console_commands.json` 的 `enabled_by_default` 字段；
  **会改仓库跟踪文件（台账 / `docs/` / `assets/` / `evidence/` 根）的写动作默认关闭**（当前仅 `close_run` 一条，
  需 `--enable-write` 显式开启）；平台自有写点（`evidence/console/`）的 `audit_book` 默认启用。
  S32.6 对账断言同时校验「id 集合」与「默认启用列」。
- **`map_pack.py` 输出是人类可读文本**（无 `--json`）⇒ 平台只能解析文本行：
  已知脆弱点 = 输出格式变更会让平台解析漂移 ⇒ S32.7 DoD 要求断言「平台展示的违规条数与
  `map_pack.py verify` 退出码 + 输出行数逐条一致」，格式变了即 FAIL（不得静默显示 0 违规）。

- **门禁档位口径**：`-Fast` / `-Touch` 是**开发档**（多项 SKIP），**不构成 gate 全绿证据**；
  「gate 全绿」= 无参数全量跑完（项数以 `docs/parity.md` 为准，当前 = 33 项；门禁结构变化时本节不逐项跟进）。
- **禁止 1**：把 HTTP 请求体里的任意字符串透传给 `--cmd`（`task_close.py --cmd` 是自由 shell 面 ⇒ RCE）；
  `--cmd` 取值只能是 `console_commands.json` 里登记的 **id**，未登记 id 直接 400。
- **禁止 2**：`sh -c` / `cmd /c` 字符串拼接；一律 `Command::new(程序).args([...])` 逐参数传递。
- **禁止 3**：平台执行**完整构建**（会重写 `web/` 与 git 跟踪的 `evidence/S18.8a-wasm-size.log`）
  ⇒ 只提供 `-Check` 体检；产物预览走平台自身静态服务。

### 4.3 Linux / Windows 命令映射

- 平台**不调用 `playbook.ps1`**（它只是 Windows 便捷入口，Linux 无 `powershell` 命令名）；
- PowerShell 解释器：候选顺序 `pwsh` → `powershell`（Windows 仅有 PowerShell 5.1 时回退）；
  `pwsh` 优先与 `docs/migration_linux.md` §4 一致（Windows 另加 `powershell` 回退）；
- Python 解释器：候选顺序 `python3` → `python`，与 `gate.ps1` 的 `$py` 逻辑一致；
- 平台自身 HTTP 服务不依赖任何外部命令（纯 std）。
- **已知缺陷（实测，影响平台展示门禁结果的可信度）**：`scripts/gate.ps1` 第 17 项内部**硬编码**
  `powershell`（该行号随 gate.ps1 在途改动漂移；S32.11 开工时须重实测行号回填，勿按文档时点值定位）⇒ 在 Linux（无 `powershell`）上该命令不存在，但
  `$LASTEXITCODE` 仍为 0 ⇒ **该项判 PASS（假阳性）**。平台展示门禁结果时必须把该项标注为
  「不可信（Linux 假 PASS）」；**修复已落码（台账 `S32.11`，`evidence/S32.11.log`，verified 待双审计收敛）**：
  gate.ps1 第 17 项宿主解释器探测（`pwsh` 优先 → `powershell` 回退，Windows 行为不变）
  + 无宿主时记名 SKIP 不假 PASS（宿主在位 = 真 PASS；无宿主 = 记名 SKIP，原「不可信（Linux 假 PASS）」标注在落码后失效）；
  `S32.12` 因 DoD 含「gate 全绿」判据而 `deps=[S32.11]`（**S32.11 verified 后解锁**，取件规则保证不提前开工；
  无宿主机器的 SKIP 口径在 S32.12 判据实跑时复核第 17 项非 SKIP）。

## 5. 数据源（只读）

| 源 | 用途 |
|---|---|
| `plan/ledger.yaml` | 任务 / 阶段 / 依赖 / 阻塞 / 进度 / rounds（单行 flow-mapping，`ti-devmeta::ledger` 解析） |
| `docs/`（含 `docs/experience/`） | 索引 / 全文检索 / `EXP-xxx` 反查 / 单一事实源链接 |
| `evidence/`（含 `evidence/console/`） | 验收证据 / 截图 / 门禁历史 / 审计报告 |
| `assets/gate_data.json` | 门禁用固定数据（只读展示） |
| `assets/ui_manifest.json` | UI 资产清单（缩略图 + 哈希 + 尺寸） |
| `assets/registry.json` | 字体/音频资产登记（S30.0 全类型登记） |
| `assets/maps/` | 已装地图包目录项前缀（已装包集合口径 = `map_pack.py` 的 `installed_packs()`：**有 `manifest.json` 且目录名 ≠ `_template`**；平台按此**自扫描**，不解析 `list` 的人类可读输出；`index.json` 在 `assets/maps/` 前缀内放行 = 包索引只读面） |
| `assets/maps/.default` | 当前默认包指针（`map_pack.py` 的 `.default` 指针文件；缺失 = 无默认包，平台不报错） |
| `assets/ui/`（`ui_manifest` 登记的 png 实体） | UI 图片资产实体（目录项前缀；登记文件 = `assets/ui_manifest.json` 的 `assets[].file` 清单，S32.2 混合白名单面，§10.2「assets/ 只返回登记过的文件」落实） |
| `dist/maps/*.map-pack` | 可分发件（`<id>.map-pack/` 目录 + `<id>.map-pack.zip`，1 对 1；存在才展示；`dist/` 被 `.gitignore` 忽略，缺失不报错；S32.2 白名单 = `dist/maps/` 目录项前缀，**不登记** index.json 类文件项） |

> **数据源路径的单一事实源** = `crates/ti-console/console_sources.json`（仓库根相对路径清单）；
> Rust 侧只持有该清单路径 + 运行时拼接 ⇒ **0 个 `assets/xxx.json` 字面量** ⇒ 不触发 CS-1（§11.2）。

## 6. 页面与能力矩阵

> **web/ 指代澄清（防混淆，S32.3 双审计 r1 B 侧 4-F2）**：平台壳静态面 =
> `crates/ti-console/web/`（`index.html` 单文件交付，JS/CSS 全内联，0 外部静态资源依赖）；
> 根 `web/` = S18.8a 游戏构建产物（`treasure-bevy-wasm` + HTML 胶水，`.gitignore` `/web/`），
> 是另一物——平台「不执行完整构建 ⇒ 不重写根 `web/`」（§3 禁止 3）≠ 平台壳 `crates/ti-console/web/`。

### 6.1 页面

| 页面 | 内容 | 阶段 |
|---|---|---|
| 概览 | 当前阶段 / 进度条 / 下一任务 / 阻塞清单 / 最近门禁 / 最近审计 | P0（S32.3） |
| 任务 | 台账全表（按 status / 阶段 / rounds 筛选），任务详情含 dod + evidence 链接 | P0 |
| 证据 | `evidence/` 浏览 + 缺失告警 + 双审计报告清单（`audit-*.md` 文件 + §7.3 四行机器字段：task/round/侧/新发现/误报/残留，`GET /api/audit`） | P0 |
| 门禁 | 分档执行 + 轮询日志 + 历史 | P1（S32.4） |
| 资产 | UI 图片预览（`ui_manifest` 全量登记图（登记数随资产批次增长，S32.3 快照 = 75 张） + **标签分组 = role + 时间倒序** `generated_at` 降序 + **lightbox 点击放大** + 元数据含 AI prompt 原文；图片直读 = `assets/ui/` 白名单前缀，`GET /api/assets`）/ 校验状态 | P0（S32.3 看板 v1）+ P2（S32.7 校验状态） |
| 设计图 | AI 生成设计稿浏览（`evidence/ui_concept/`，**非运行截图**；`GET /api/evidence?sub=ui_concept` 子目录枚举 + 白名单直读 + 组别标签〔主地图/首页/岛型·地图/对战/界面/资产表/其他〕+ lightbox 放大） | P0（S32.3） |
| 地图 | 地图包管理面（`assets/maps/<id>/` 已装包清单 + 描述符摘要〔schema/id/version/data_version/中央岛/岛型数/31 格不变量/环几何/资产数〕+ `.default` 指针 + 制作指引 `scripts/map_pack.py` 命令面 + 21 号标准链接；`GET /api/map`，**只读浏览**，制作写操作走 `map_pack.py` 工具链、平台 0 写盘） | P0（S32.3） |
| 文档 | 全文检索 + ID 反查 | P2（S32.8） |
| 构建 | `build_wasm -Check` 体检结果 + 产物清单 + `web/` 预览 | P2（S32.9） |
| 设置 | token 粘贴（存 `sessionStorage`）+ 写操作开关 + 平台信息 | P0（S32.3，与 `GET /api/session` 配套） |

### 6.2 能力矩阵

| 阶段 | 能力 | 任务 |
|---|---|---|
| P0 只读骨架 | 平台骨架（`ti-devmeta` + `ti-console`）/ 只读 API / 看板 UI v1 | S32.0a~S32.3 |
| P1 执行与审计 | 一键门禁（分档）/ 双审计任务书 + 收敛判定 / 收口 | S32.4~S32.6 |
| P2 辅助开发 | 资产浏览 / 文档检索 / 构建体检与预览 | S32.7~S32.9 |
| P3 部署（可选，**不入台账**） | 部署 / 鉴权 / 多工作区 | 需要时另立项（见 WBS §P3） |

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
- 报告命名区分角色：`evidence/console/audit-<台账 id>-r<N>-a.md`（A）/ `-b.md`（B）；
  `<台账 id>` = **被审任务的当前台账 id**（拆分前遗留的历史轮次保留原 id 文件名，见 `evidence/console/` 现有 r1~r3）。

### 7.3 载体（平台不自己起 LLM 子代理）

平台**没有**能力自行派生 LLM 子代理（那是会话 harness 的能力）。落地方式：

1. 平台生成**审计任务书**：`evidence/console/audit-<task>-r<N>-<role>.prompt.md`（模板 + 台账 dod + 证据路径）；
2. 主线程用 harness 按任务书起两个子代理（A/B 并行、互不可见）；
3. 子代理报告**由主线程写回** `evidence/console/audit-<task>-r<N>-<role>.md`
   （平台也提供「粘贴归档」入口：主线程把报告正文粘给平台，平台落盘同一文件名）；
4. 平台做**机械判定**：解析报告抬头里的**机器可解析字段** → 收敛轮次表 + 未闭环清单。

   **报告格式契约（机器可解析）** —— 报告抬头必须含以下**四行**，平台按此解析（缺行 ⇒ 判「不可解析」而非 PASS）：

   ```
   总判定: PASS|FAIL
   新发现: <整数>
   误报: <整数>
   残留: <整数>
   ```

   **位置口径（机器可解析）**：四行必须落在**报告正文的第一个围栏代码块（``` 块）内**
   （标题 / 引用块说明之后、findings 表格之前）；解析器 = 读文件 → 取第一个 ``` 代码块 → 逐行前缀匹配
   `总判定:` / `新发现:` / `误报:` / `残留:`，缺任一行 ⇒ 判「不可解析」。
   **值域口径**：`总判定` 只允许 `PASS` / `FAIL` 两个值；其余值（含占位轮写的 `中断（无报告）`）
   ⇒ 该报告判「不可解析」⇒ **该轮不计入收敛轮次**（占位轮不得被机械判定误读成 0/0/0 有效轮；
   S32.5 收敛单测须含「占位轮值域异常不计轮」断言）。
   **计数口径**：「0 新发现」= 该报告**新发现行 = 0**；P2 级**口径精化建议**同样计入新发现
   （子代理报什么就算什么，主线程不得把「P2 不阻断」读成「不算新发现」）。
   **轮次完整性口径（S32.5 机械判定断言面）**：断言「**N 轮 × A/B 角色齐全**」
   （轮次序号连续无缺、每轮 a/b 两份文件在盘；占位文件计轮次完整性但不计四行值域轮）；
   **不断言绝对份数**（份数随轮次增长漂移，写死将永久缺一报红——第 15 轮 NEW-01 实例）。
   第 7 轮实例：r7-b 报「3 条 P2 建议 + 0 P0/P1」⇒ 该侧新发现 = 3，该轮未收敛；
   建议吸收后由**下一轮（第 8 轮）**复核，不复读同一轮。
   **祖父条款**：r1~r3 六份历史报告（契约引入前）按**人工口径**读，平台不得因缺行判其「不可解析」而误报；
   第 4 轮起新写的报告一律四行齐全（r4 实测已含四行代码块）。

> 平台只做「任务书生成 + 归档 + 判定」，**不做**审计本身 ⇒ 平台不依赖任何模型 SDK。

### 7.4 收敛判定（引用飞轮口径，不另立）

- 采用 [`06_flywheel.md`](06_flywheel.md) §停机判据的**同一条 + 轮次要求**：
  **0 新发现 + 0 误报 + 0 残留**，且**连续 2 轮**全部满足 = 通过；
- **未过处理**：`docs/loop.md` §4「验收连续失败 3 轮 ⇒ 拆子任务」——
  拆成 `Sx.y.a` / `Sx.y.b` 形式子任务，**原行删除或 `rounds` 归零**
  （台账校验器 `crates/ti-tools/src/ledger.rs` 对 `rounds >= 3` 直接报 error ⇒ 不归零会让门禁第 8 项红；
  `docs/loop.md` §4 已同步为该口径）。
- **`rounds` 起算口径**：拆分出的子任务 `rounds` 从 **1** 起算（拆分动作本身计 1 轮），
  不继承被拆任务的失败轮数（继承会让新任务一上线就撞 `rounds >= 3` 红线）。
- **两个计数器严格分离（不写回、不同步）**：
  - 「审计轮」= 双审计跑了几轮，记录在 `evidence/console/` 报告文件名与 15 号台账第 21 行（**不进台账**）；
    连续 2 轮达标 = 收敛。
  - 「台账 `rounds`」= **验收失败轮数**（`docs/loop.md` §1；`crates/ti-tools/src/ledger.rs` 对 `rounds >= 3` 报 error），
    只在「主线程判据实跑未过 / gate 未全绿」时 +1（审计 findings ≠ 验收失败）。
  - **禁止把审计轮数写回台账 `rounds`**（写回 ⇒ `rounds>=3` 红线与「拆分后子任务 rounds 从 1 起算」形成无限再拆死锁，第 6 轮审计 P1 实证）。
  - **拆分触发 = 主线程独立判定**（`docs/loop.md` §4）：台账 `rounds >= 3`，**或** 审计连续 3 轮未收敛（主线程判断，
    把拆分记录写进 15 号第 21 行的复发证据，不改 `rounds`）；拆分出的子任务 `rounds` 从 1 起算。
- **与飞轮的分工**：飞轮 = 全库范围找问题（开轮/停机见 06）；本协议 = **每任务 2 只读审计员**的验收前置。
  两者**用同一停机定义**，差别只在范围（单任务 vs 全库）与轮次（2 轮 vs 飞轮自定），**不新增收敛定义**；
- 平台页面同时展示飞轮轮次与本协议轮次。

### 7.5 主线程独立复核

- 复核动作 = 主线程自己跑 DoD 命令 + 抽查子代理引用的行号/文件确实存在；
- 复核记录写进 `evidence/S32.x.log`（「复核：<命令> → <结果>」），不只写「已复核」。

## 8. 北极星判据（含测量口径）

| 判据 | 阈值 | 测量方式（可复现） |
|---|---|---|
| 首屏可见关键信息 | < 2 s | **区间 = navigationStart → 关键信息渲染完成**（含导航/下载/HTML 解析，**不以 DOMContentLoaded 为起点** ⇒ 不能靠「先渲染骨架后取数」绕过）：`performance.getEntriesByType('navigation')[0]` 的 `startTime` 到「`/api/overview` 数据上屏」的 `performance.now()`；另测 `curl -s -o /dev/null -w '%{time_total}' http://127.0.0.1:8787/api/overview`（warm，连做 5 次取中位数）作为接口侧对照；两者都落 `evidence/console/dashboard-perf.log` |
| 门禁可见可查 | — | 分档执行 + 日志轮询 + `evidence/console/logs/gate-<ts>.log` 历史列表 |
| 每任务双审计可追溯 | — | 每轮 A/B 报告落盘 + 收敛轮次表 + 未闭环 findings 计数（§7.4 三条判据） |
| 平台自身质量 | 0 违规 | 既有门禁自动覆盖（见 §11.2）+ `cargo test -p ti-console` / `-p ti-devmeta` 实跑 |
| 快照纪律有载体 | 0 空判 | 「0 游戏产物变更」的内容级指纹快照由 **`crates/ti-console` 的快照模块实现**（S32.2 落地；登记见 `docs/process/15_process_ledger.md` 第 23 行，状态 = **待固化（挂 S32.2，S32.2 完成前不得当载体断言）**）⇒ 纪律不悬空 |
| **0 游戏产物变更** | 0 差异 | **增量口径 + 内容级快照（会话相对）**：动作前后各取一次冻结路径的**内容指纹**并逐文件比对，差集为空。指纹 = 每个文件 `sha256`（忽略 `.gitignore` 命中的文件）+ 未跟踪文件清单；**不能用 `git status --porcelain`**（它只给路径 + 状态码，不含内容 ⇒ 已脏文件再改内容时快照不变 = 口径失效，见第 4 轮 P1）。冻结路径 = `crates/ti-core crates/ti-app crates/ti-engine crates/ti-render crates/ti-platform crates/ti-formats assets/ android/ tests/`（游戏产物面；工具面 `crates/ti-tools crates/ti-devmeta crates/ti-console` **不在冻结范围**，S32.1 改 `ti-tools` 由 `cargo run -p ti-tools --bin ledger` 0 error 断言行为不变）。**根 `Cargo.toml` / `Cargo.lock` 不在冻结路径**（S32.1 必须新增 workspace member ⇒ 会改二者；口径 = 「`Cargo.toml` 的 `members` 段仅允许新增 `crates/ti-devmeta` / `crates/ti-console` 两行，其余逐字节一致；`Cargo.lock` 仅允许新增对应 package 条目」）。**门禁既有副作用路径** `evidence/web_playtest.{log,png}`、`evidence/font_render.txt` 只记录不判定（§3.1），但必须写进执行日志并提示。`web/` 产物清单（文件名 + 体积）前后一致（平台不执行完整构建 ⇒ 天然不变） |

> `web/` 与 `dist/` 被 `.gitignore` 忽略 ⇒ **`git status` 不能作为「0 游戏产物变更」的证据**（EXP-058 反虚报口径）。

## 9. 不做清单

- 不做游戏创建、关卡编辑器、运行时调试器（除非另立项）。
- 不做账号体系、多租户、计费（P3 再评估）。
- 不做任意 shell：只跑 §4.2 白名单动作 + 枚举/id 参数。
- 不引入数据库：v1 用仓库文件（§3.1 写面边界）。
- 不在平台里复述业务结论：只链接单一事实源。
- 不内嵌 LLM SDK / 模型调用（§7.3）。
- **不做完整构建 / 不重写游戏产物**（只提供 `-Check` 体检 + `web/` 预览）。
- 不重述地图包标准与资产管线标准：涉及地图包/资产的事实一律引用 [`21_map_package_spec.md`](21_map_package_spec.md)
  与 [`18_asset_content_pipeline.md`](18_asset_content_pipeline.md)，本文只写「平台怎么调它」。
- 不结构化解析 `plan/packages.yaml`（嵌套 YAML 需新依赖，收益低）⇒ 平台只给链接，真源仍是该文件。

## 10. 安全与边界

### 10.1 网络面与 token

- 默认 `127.0.0.1:8787`；`--host` 非回环地址时**必须**同时给 `--token`（或环境变量 `TI_CONSOLE_TOKEN`），否则**拒绝启动**
  （S32.1 DoD 判据：实测非回环 + 无 token ⇒ 进程拒绝启动）；
- **token 存放**：默认落**仓库外**平台目录 —— Linux `$XDG_CONFIG_HOME/ti-console/token`（回退 `~/.config/ti-console/token`），
  Windows `%APPDATA%\ti-console\token`；权限 `0600`（Windows 尽力）；`--token` 传入时不落盘；
  **禁止**写进 `evidence/console/`（git 跟踪目录）⇒ `.gitignore` 已加 `/evidence/console/token` 兜底；
- **取得途径**：`cargo run -p ti-console -- token` 把 token 打印到 stdout（用户自行复制）；
  启动日志打印 **token 文件路径**（不含内容）；
- **token 分发**：平台**不回传 token**；`GET /api/session` 只返回 `{write_enabled: bool}`。
  浏览器侧：用户在「设置」里粘贴 token → 存 `sessionStorage`（关闭即失）→ 写操作带 `X-TI-Token` 头；
- 请求方法白名单：只读 `GET`；写操作 `POST` + `X-TI-Token`（缺 token ⇒ 403）；请求体上限 64 KiB；
  只接受 `Content-Type: application/json`。

### 10.2 路径面

- 所有文件访问经 repo 适配层：比较对象 = **`canonicalize(repo_root)` 自身**（仓库根可能是符号链接，
  Windows 另有大小写/分隔符差异 ⇒ 不 canonicalize 会误判逃逸），被访问路径 canonicalize 后必须仍以该值为前缀；
- 只暴露 `console_sources.json` 白名单里的目录（`docs/`、`evidence/`、`plan/`、`assets/`、`dist/maps/`）；只读 GET 端点的查询参数（如 `/api/whitelist?path=`）= 白名单判定器入参，**不受** §4.2「参数只接受枚举/id」约束（该约束面 = 写操作命令白名单的 `--cmd` 参数，§10.3 执行面；只读面 0 写盘 0 命令映射）；
- `assets/` 只返回登记过的文件（清单 + `maps/` 内 manifest 引用），不做任意目录列举。
  （S32.2 起落实面 = `console_sources.json` **混合白名单**〔目录项前缀 + 文件项精确〕数据驱动，
  上列 5 目录名 = 语义面；逐文件放行以 JSON 清单为准，白名单判定器 = `repo::guard::check_relative`。）
  - **png 实体直读口径**（S32.3 资产页消费面）：`ui_manifest.json` 登记的图片实体 = `assets/ui/` 目录项前缀放行（白名单 10 条之一），浏览器以 `http://host/assets/ui/<name>.png` 直读（content-type = `image/png`，`ti-devmeta::http::mime_for` 面）；未登记文件（如 `assets/data/*.json`）404 拒。

### 10.3 执行面

- 只执行 §4.2 表内动作；参数只接受枚举/id；
- 子进程环境变量**最小集**透传：`PATH`、`HOME`、`USERPROFILE`、`CARGO_HOME`、`RUSTUP_HOME`、
  `SYSTEMROOT`、`TEMP`、`TMP`、`APPDATA`、`LOCALAPPDATA`、`XDG_CACHE_HOME`、
  `TI_CHROMIUM_PATH`、`TI_CONSOLE_TOKEN`；其余不透传
  （`gate.ps1` 依赖 PATH/HOME 定位 cargo；node / Playwright 依赖 SYSTEMROOT/TEMP/APPDATA/XDG_CACHE_HOME）；
  平台按 OS 取该表的交集（Windows 侧忽略 `XDG_CACHE_HOME` 等 POSIX 专有项，反之亦然）；
- 每次执行记录：动作 id / 参数 / 起止时间 / 退出码 / 输出落盘路径 / **执行前后 `git status` 快照（仅记录用，
  不是 §8「0 游戏产物变更」的判定口径——判定口径 = §8 的内容级 sha256 指纹差）**
  → `evidence/console/logs/exec-<ts>.log`（gitignored 子目录）。

### 10.4 并发、回滚与证据保留

- **单实例锁**：`evidence/console/.lock`（gitignored；含 pid + 起始时间 + 动作 id）。S32.1 骨架期 = **pid 最小面**（仅写 pid 互斥；起始时间 + 动作 id + 30 分钟陈旧接管判定 = S32.4 并发面补全，`main.rs` 锁段注释已标注「S32.4 补全」）。
  互斥范围 = **平台自己发起的执行**（不互斥人工/会话直跑 `scripts/gate.ps1`，也不跨 worktree）；
  陈旧锁判定 = pid 不存在 **且** 起始时间超 30 分钟 ⇒ 接管并记 `stale-lock`（完整规格 = S32.4 并发面：起始时间 + 动作 id 字段 + 30 分钟接管判定；S32.1 骨架期锁文件仅 pid，接管判定暂不实现）。
- **不做自动回滚**：受控执行可能改台账，回滚 = 人工 `git checkout` / 重跑脚本；
  平台只保证「执行前记录 `git rev-parse HEAD` + 执行后 `git status --short` 快照」进日志，供人工判断
  （同 §10.3：该快照**仅记录**，与 §8 的内容级指纹判定口径分离）。
- **证据保留**：运行期日志落 `evidence/console/logs/`（gitignored）⇒ 清理**不脏仓库**；
  保留最近 50 份，**只删未入库文件**（`git ls-files` 命中的一律不删）；删除动作记 `retention.log`；
  **审计报告永不自动清理**（验收追溯用）。

## 11. 视觉与质量口径

### 11.1 视觉改动（硬规则 9 适用）

平台 UI 属「表现层视觉改动」⇒ **先出静态原型截图 `evidence/console/ui-<stage>.png`，用户确认（消息留痕）后才落码**。

- **机器兜底现状**：`doccheck2::check_visual_design_signoff` 只对 dod 含 `scenes.json` / `theme.json` 的任务触发
  ⇒ 平台 UI 任务**不在其触发面内** ⇒ 本条为**人工留痕**（无自动断言），留痕格式：
  `evidence/S32.x.log` 内含「用户视觉验收通过」+ 原型截图路径。
- 登记位置：`docs/process/15_process_ledger.md` 第 **22** 行（半自动，人核最后一步；第 21 行 = 每任务双审计）。

### 11.2 平台自身质量（**不新增门禁顶层项**）

- 理由：`crates/ti-console` 与 `crates/ti-devmeta` 注册为 workspace 成员后，
  **Rust 侧自动被既有门禁覆盖** —— `cargo fmt --check` / `cargo clippy --workspace` / `cargo test --workspace`
  / 占位扫描（`crates/**/*.rs`）/ `unwrap` 扫描 / `check_code_standards.py`（遍历 `crates/`）。
  再加一个「平台」顶层项 = 重复劳动 + 改变门禁项数口径（项数唯一事实源 = `docs/parity.md`），无收益。
- **覆盖边界（如实标注）**：`crates/ti-console/web/*`（JS/CSS 静态资源，S32.3 起 = `index.html`
  单文件内联 JS/CSS，0 外部静态资源依赖）**不在**上述扫描面
  ⇒ 由 **S32.3 判据兜底**（浏览器 0 console error + 首屏 <2 s + 页面数据与 API 抽样一致）。
  **指代澄清（S32.3 双审计 B 侧 4-F2）**：本节 `web/*` = `crates/ti-console/web/`（平台壳）；**根 `web/` = S18.8a 游戏构建产物（wasm 交付件，gitignored，勿混）**。
- **CS-1 口径**：平台 Rust 侧持 `console_sources.json`（路径清单）⇒ **0 个 `assets/xxx.json` 字面量**
  ⇒ CS-1 天然 0 命中，**不需要扩豁免**（不动棘轮）。
  若将来必须硬编资产路径，须按 `docs/process/16_code_standards.md` 流程扩允许位
  （`crates/ti-console/src/repo/`，类比既有 `crates/ti-tools/` 允许位）+ 同步 16 号文档 + `scripts/check_code_standards.py`。
- **已知兜底缺口（如实标注）**：「22 号八要素齐全」+「5 处登记」两条判据**目前无机器断言**
  （`doccheck` / `doccheck2` 无相关检查项）⇒ 后续迭代删段/漏登不会被门禁发现；
  兜底落在 **S32.5**（审计任务书模板必须逐条覆盖 22 号八要素清单 ⇒ 漏段在生成任务书时暴露）。
- **机器面盲区（如实标注，不扩面）**：S32 文档面（本节 / WBS）对门禁项数的声称点（§4.2「当前 = 33 项」等）**不在** `doccheck2::check_gate_item_count` 声称面（gate.ps1 头注释/横幅 + parity/git/README）也不在 `doc_numbers.py` 声称点内⇒ 门禁结构再漂移时机器不报，当前由**双审计人工 grep** 兜住（复发记录 = 15 号第 21 行）；扩面归属 = 后续门禁结构演进时随 `check_gate_item_count` 声称面扩容，**非 S32 任务**。
- **信任项备注（S32.2 起 repo 适配层「0 写盘」已断言；`build_wasm.ps1 -Check`「不构建/不拷贝/不写 evidence」与 map_pack `install/uninstall` 条件写行为仍挂账 = S32.4/S32.9 执行面落地后改全断言）**：S32.2 落地 = `repo/` 适配层 24 条单测含 0 写盘面（repo/ 全模块 0 处写盘 API，唯一写点 = `evidence/console/.lock`，gitignored）⇒ 本节 repo 面改「已断言」；`-Check` 实跑 0 写盘断言 = S32.9 载体（执行面）。
- 补充断言：S32 各任务的 DoD 各自要求 `cargo test -p ti-console`（或 `-p ti-devmeta`）实跑 +
  依赖图断言 + `0 游戏产物变更`（§8 测量方式）。

## 12. 工具与登记纪律

- 新增 `scripts/*` 工具：按 `docs/process/14_tooling_catalog.md` 3 处登记 + 工具契约（`check_tooling_contract`）；
- 新增 **cargo bin**（`ti-console` / `ti-devmeta` / `ledger` 等）：登记落点 =
  `docs/process/14_tooling_catalog.md`（**工具唯一事实源**）新增「cargo bin / crate 工具」小节
  + `docs/process/04_tooling.md` §1 的「`crates/…` 各 bin」行 + `docs/README.md` 索引（三处）。
  **已知无机器兜底**：`doccheck2::check_tooling_registration` 只扫 `scripts/*.py|ps1`
  ⇒ cargo bin 登记靠人工核对（登记进 14 号后由 doccheck 的「文档提及的任务/工具」链路间接可见）；
- **cargo bin 进 14 号的连带同步项**（S32.1 一并落，否则「登记行」不属于 14 号任何一张既有表）：
  §1.1 工具分类（四类 → **五类**，「目录位置」列加 `crates/`）+ §1.2 第 1 步（落 `scripts/` → **或 `crates/*` 的 cargo bin**）
  + §1.4 工具契约第 1/2 条的适用范围 + §3 新增 **§3.5「cargo bin / crate 工具」**；
- 平台内**可执行动作的白名单与命令映射** = `crates/ti-console/console_commands.json`（机器真源）；
  22 号 §4.2 是人读视图，两者 id 集合须一致（S32.6 对账断言）；
- 平台代码受 `docs/process/16_code_standards.md` 约束（新代码零豁免）。
