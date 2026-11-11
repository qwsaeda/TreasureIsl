# S50 WBS — 借鉴 CCGS 工作室制度：3 项过程约束补强 + 1 项元测试面

> **S50.0 立项（2026-11-11 用户拍板「两者都做」）**：源自 CCGS 对照分析（`docs/process/32_ccgs_borrow.md` 为设计稿，本文件为 WBS 单一事实源）。
> 原则：**只补过程约束（CCGS 强而我方弱的面），不动收口约束（我方已强）**；4 项明确不借鉴（49 agent 层级 / 75 slash 命令 / commit hook / 逐 agent 内嵌 5 步协议）。

## 1. 任务表（按依赖顺序）

| id | 标题 | 依赖 | 优先级 |
|---|---|---|---|
| **S50.0** | 立项定案（本文件 + 设计稿 32_ccgs_borrow.md + 登记 README/AGENTS + 台账条目） | S49.9 | P0 |
| **S50.1** | 状态文件即记忆（`docs/loop.md` §长会话纪律：压缩前/后协议 + `evidence/<task>.state.md` 三要素 + 主线程续接） | S50.0 | **P1（价值最高）** |
| **S50.2** | 元测试面（`scripts/studio_tests.py`：doccheck2 触发面 + test_manifest↔console_commands 映射 + 工具台账三处登记核验；并入 gate 第 25b 项〔实落 = gate 25b，非 doccheck2 项〕） | S50.0 | P1 |
| **S50.3** | 路径作用域红线（`docs/rules/*.md` 路径 glob 红线文件 N 个 + doccheck2 第 43 项断言「rules 文件存在 + 路径 glob 合法 + 内容非空 + 与 16_code_standards CS 矩阵交叉引用」） | S50.0 | P2 |
| **S50.4** | 收口：3 项全落地后 gate 全绿 + doccheck/doccheck2 0 error + 台账三方对账 + 双审计 2 轮收敛 | S50.1,S50.2,S50.3 | P0 |

## 2. S50.0 立项定案（本任务）

**改动面 = 纯文档 + 台账 + 登记，零代码改动。**

- 新增 `docs/plan-s50-studio-inferface.md`（本文件，WBS 单一事实源）
- 新增 `docs/process/32_ccgs_borrow.md`（设计稿：CCGS 调研摘录 + 机制级对照表 + 3 借鉴点详述 + 4 不借鉴理由；单一事实源 = 本文件 WBS，32 号 = 决策依据）
- 登记 2 处：`docs/README.md` 索引 + `AGENTS.md` 路由表（S50 行）
- 台账：`plan/ledger.yaml` 加 S50.0~S50.4 条目
- **判据**：
  - `docs/plan-s50-studio-interface.md` 要素齐全（§1 任务表 + §2 S50.0 判据 + §3~§5 S50.1~S50.3 判据 + §6 不借鉴清单）
  - `docs/process/32_ccgs_borrow.md` 存在且含 CCGS 调研摘录 + 对照表 + 3 借鉴点 + 4 不借鉴
  - 登记 2 处（README 索引 + AGENTS 路由表 S50 行）
  - 台账 S50.0~S50.4 5 条 pending 条目（status: pending，dod 含判据命令载体）
  - doccheck 0 error + doccheck2 0 error
  - `git diff crates/` = 空（纯文档立项，零 Rust 改动）
  - gate 全绿（35 项 0 failure）
  - evidence/S50.0.log 实跑判据输出
- **dod 命令载体**：
  ```sh
  test -f docs/plan-s50-studio-inferface.md && test -f docs/process/32_ccgs_borrow.md
  grep -c "S50.0\|S50.1\|S50.2\|S50.3\|S50.4" plan/ledger.yaml   # ≥5 行
  grep -n "S50" docs/README.md AGENTS.md                           # 2 处登记
  python3 scripts/check_ledger_reality.py                             # 三方对账
  python3 -c "import subprocess; s=subprocess.run(['cargo','run','-q','-p','ti-tools','--bin','doccheck'], capture_output=True, text=True); print(s.stdout); exit(0 if 'error' not in s.stdout.lower() or '0 errors' in s.stdout.lower() else 1)"
  ```

## 3. S50.1 状态文件即记忆（P1）

**改动面 = 文档（`docs/loop.md` 加长会话纪律节）+ 台账 + 证据格式约定，零代码改动。**

- `docs/loop.md` 新增 §「长会话压缩纪律（CCGS active.md 模式借鉴）」：
  1. **压缩前**（主线程自检）：把 3 要素写进 `evidence/<task>.state.md`：
     - ① 已读清单（本次会话读过哪些规范文档 + 命中几条现行规范）
     - ② 当前在途包白名单（`plan/packages.yaml` 相关 P 包清单 + 各自文件域）
     - ③ 下一步动作（具体到命令：如「跑 `python3 scripts/task_close.py --task S50.1` 收口」）
  2. **压缩后**（主线程第一件事）：读 `evidence/<task>.state.md` 续接，禁止凭对话摘要续接
  3. **触发条件**：会话已读规范文档 >20 份 或 单次任务跨 2 个以上 P 包 或 主线程自觉「快要压缩」（harness 压缩前提示）
- 登记：`docs/README.md` loop.md 行补「长会话压缩纪律」标签
- **判据**：
  - `docs/loop.md` 含新节 + 3 要素字段名固定（①②③）
  - `docs/README.md` loop.md 行更新
  - 台账 S50.1 verified（evidence/S50.1.log 含 loop.md 新节 grep 证据 + 用户确认留痕）
  - 后续真实长会话压缩时**实测一次**（下次开长会话触发压缩时，主线程压缩前写 state.md + 压缩后读 state.md 续接，证据入下次任务 evidence，**不阻塞 S50.1 收口**）
- **dod 命令载体**：
  ```sh
  grep -n "state.md\|长会话压缩纪律" docs/loop.md | head -5
  ```

## 4. S50.2 元测试面（P1）

**改动面 = 工具（`scripts/studio_tests.py` 新正式工具）+ 门禁（gate 第 25b 项〔实落 = gate 25b，非 doccheck2 项〕）+ 台账 + 工具契约 3 处登记。**

- 新增 `scripts/studio_tests.py`（纯 std 风格，python 3；工具契约 §14 号 4 条全满足：落地 scripts/ + 14 号表格行 + 04 速查 + README 索引 + `--check` 只读 + 非 0 退出）
  三类断言（对应 CCGS `studio-tests/` 元测试思想）：
  1. **doccheck2 触发面**：每个 `fn check_*` 在 `crates/ti-tools/src/bin/doccheck2.rs` 被注册表引用（grep 注册表 `check_<name>` 字符串命中）+ 在 `assets/gate_data.json` `doccheck2_checks` 清单有条目（双向对账，缺失即 FAIL）
  2. **test_manifest ↔ console_commands 映射**：`assets/data/test_manifest.json` 每个登记的测试 command 首词 ∈ 程序白名单（cargo/python3/pwsh/powershell）+ 该 command 在 `crates/ti-console/console_commands.json` 有对应 command 映射（无映射 = 该测试无法经平台白名单执行 = 平台面未覆盖 = 漂移 FAIL；**存量未映射 = 进 `scripts/studio_tests_waiver.json` 棘轮基线**，新增必映射）
  3. **工具台账三处登记核验**：`scripts/*.py` / `scripts/*.ps1` 每个文件在 ①`14_tooling_catalog.md` 表格行 + ②`04_tooling.md` 速查表 + ③`docs/README.md` 索引 三处均登记（grep 文件名命中），缺失任一即 FAIL（存量豁免 `scripts/tmp/` 前缀 + `scripts/studio_tests_waiver.json` 棘轮基线〔S50.2 自持，非 `process_ratchet.json`〕）
- 门禁接入：`scripts/gate.ps1` 加第 25b 项（`& $py scripts/studio_tests.py --check`；Tier 'skip' 收口档实跑，开发档 SKIP）；`assets/gate_data.json` 同步 + `doc_numbers --fix` 全库项数回写
- **判据**：
  - `python3 scripts/studio_tests.py --check` 实跑输出落 evidence/S50.2.log（含 3 类断言通过/失败计数 + 棘轮基线豁免清单）
  - gate 第 25b 项入 `gate.ps1` + `assets/gate_data.json` + `docs/parity.md` 项数回写（doc_numbers --fix）
  - 工具契约 3 处登记（14 号 §3.2 studio_tests.py 行 + 04 号速查表 + README 索引）
  - 负例护栏：临时把某个 test_manifest 条目的 command 首词改成 `sh` ⇒ studio_tests.py 非 0 退出；恢复后 0 退出
  - 零 Rust 改动（crates/ 0 diff；工具在 scripts/ 允许改）
  - gate 全绿 + doccheck/doccheck2 0 error
  - evidence/S50.2.log 实跑判据输出
- **dod 命令载体**：
  ```sh
  python3 scripts/studio_tests.py --check
  grep -n "studio_tests" scripts/gate.ps1 docs/parity.md assets/gate_data.json   # 3 处
  grep -n "studio_tests" docs/process/14_tooling_catalog.md docs/process/04_tooling.md docs/README.md   # 3 处登记
  ```

## 5. S50.3 路径作用域红线（P2）

**改动面 = 文档（`docs/rules/*.md` N 个新文件）+ 门禁（doccheck2 第 43 项断言）+ 台账。**

- 新增 `docs/rules/` 目录，N 个红线文件（CCGS `rules/*.md` 的 `paths:` frontmatter + 路径下编码红线正文模式；**N = 按 `plan/packages.yaml` 白名单域划分，首版 = 5 个**）：
  1. `docs/rules/ti-core-rules.md`（paths: `crates/ti-core/src/**`；红线 = 零 UI 直引 / 零渲染调用 / 数据驱动 key / 业务规则先定案 `docs/business/` 再动码）
  2. `docs/rules/ti-engine-rules.md`（paths: `crates/ti-engine/src/**`；红线 = 渲染层不认场景语义 / 游戏世界保持 1080 正交 + 数据驱动 / 通用原语闭集 / bevy_ui 与 1080 两层不混坐标系）
  3. `docs/rules/ti-app-scene-rules.md`（paths: `crates/ti-app/src/scenes/**`；红线 = chrome 归 bevy_ui（S29 B1）/ vs/main_map 游戏世界留 1080 / 场景窄接缝 `SceneHost`）
  4. `docs/rules/scripts-rules.md`（paths: `scripts/**`；红线 = 工具契约 4 条（§14 号）/ 禁止写仓库外 / 一次性脚本落 tmp/ 用完即删 / `--check` 只读 + 非 0 退出）
  5. `docs/rules/assets-data-rules.md`（paths: `assets/**` + `assets/data/**`；红线 = 资产登记单源 `ui_manifest.json`/`registry.json` / 不手写像素（§18 号）/ 地图包 I1~I9 不变量（§21 号））
- 每个红线文件 frontmatter 写 `paths:` glob 列表 + 正文 4~6 条红线 + 1~2 条「正确/错误示例」
- 门禁接入：`doccheck2` 加第 43 项断言 `check_rules_files`：`docs/rules/*.md` 全部存在 + 每个文件 frontmatter `paths:` 非空 + glob 语法合法（用 python 正则粗验）+ 正文非空 + 5 个文件齐全（缺文件即 FAIL）
- **判据**：
  - `docs/rules/` 5 个文件齐全 + frontmatter `paths:` 各非空
  - doccheck2 第 43 项入 `crates/ti-tools/src/bin/doccheck2.rs` 注册表 + `assets/gate_data.json` 同步
  - 负例护栏：临时删 `docs/rules/ti-core-rules.md` ⇒ doccheck2 第 43 项非 0 退出；恢复后 0 退出
  - 子代理开轮时（下次 flywheel_readme.ps1 跑时）**实测一次**：开轮必读清单加 `docs/rules/` 目录（按包域过滤 5 个文件取相关 1~2 个注入 prompt），证据入 S50.3 evidence
  - 零 Rust 改动（除 doccheck2.rs 加断言，crates/ 0 diff 断言面 = 游戏产品 6 crate）
  - gate 全绿 + doccheck/doccheck2 0 error
  - evidence/S50.3.log 实跑判据输出
- **dod 命令载体**：
  ```sh
  ls docs/rules/ | wc -l   # ≥5
  grep -l "paths:" docs/rules/*.md   # 5 个全命中
  grep -n "check_rules_files" crates/ti-tools/src/bin/doccheck2.rs assets/gate_data.json
  ```

## 6. S50.4 收口（P0）

- 依赖 S50.1 + S50.2 + S50.3 全 verified
- 判据：
  - gate 全绿（35 项 + 第 25b 项新加 = 36 项 0 failure；项数以 `docs/parity.md` 回写为准）
  - doccheck 0 error + doccheck2 0 error（含新增第 43 项 rules-files 断言）
  - 双审计 2 轮收敛（A 完成度 + B 一致性；连续 2 轮 0/0/0 = `state=pass`，报告入 `evidence/console/audit-S50.4-r<N>-{a,b}.md`）
  - 台账三方对账（`check_ledger_reality.py` 全绿）
  - evidence/S50.4.log 含 3 项落地判据复跑输出 + 双审计收敛表
- **不阻塞**：4 项不借鉴（49 agent 层级 / 75 slash 命令 / commit hook / 逐 agent 内嵌 5 步协议）**不立任务**，已在 `docs/process/32_ccgs_borrow.md` 决策依据段留痕。

## 7. 与既有体系的边界

| 面 | 本 S50 动什么 | 不动什么 |
|---|---|---|
| 过程约束 | 加 3 项（状态文件 / 元测试 / 路径红线） | 不动 gate 既有 35 项、不动 doccheck/doccheck2 既有 43 项、不动双审计协议、不动飞轮停机判据 |
| 收口约束 | 不动 | 台账三方对账 / evidence 实跑 / 双审计收敛 / 停机判据均保持 |
| 模型 pin / 成本控制 | 不动（CCGS 的 Opus/Sonnet/Haiku pin + lean/solo 档不借鉴，我方 -Fast/-Touch/full 三档 gate 已覆盖成本分层） | — |
| Agent 角色 | 不动（22 号双审计 A/B + 飞轮 3 角色已足够，不引入 49 agent 层级） | — |

## 8. 执行顺序

```
S50.0（本文件 + 32_ccgs_borrow.md 设计稿 + 登记 + 台账 5 条）
   ↓
S50.1（状态文件即记忆，文档改动）    S50.2（元测试面，工具 + 门禁）    S50.3（路径红线，文档 + 门禁）
   ↓                                  ↓                                  ↓
   └──────────────────── 三路并行（互不依赖，可分 3 个会话/子代理并发） ────────────────────┘
   ↓
S50.4（收口：gate 全绿 + 双审计 2 轮收敛 + 台账三方对账）
```
