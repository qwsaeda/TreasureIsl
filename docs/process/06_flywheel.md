# 对抗流水线飞轮（Flywheel）

**定位**：把「子代理对抗找问题 → 审计 → 修复 → 飞轮回写（固化/EXP/lint/脚本）」
固化成可复用 loop，是 [`05_module_parallel.md`](05_module_parallel.md) 并行轮之外的**质量飞轮**。
并行轮管「模块边界各写各的」，飞轮管「写完后找问题-审计-修复-回写」；
两者共用同一份开轮必读清单（本文 §开轮必读，为唯一事实源）。

## 1. 流水线 3 角色（每轮并发）

| 角色 | 输入 | 输出 | 禁区 |
|---|---|---|---|
| **找问题 agent** | 开轮必读清单 + 本包/全域代码 | 「发现清单」：每条含 `文件:行` + 类别（漂移/越界/硬编码/未实现/风格）+ 证据 | 不改任何文件 |
| **审计 agent** | 发现清单 | 「审计结论」：逐条裁决 真漂移 / 误报 / 需人裁决，附理由 | 不改任何文件 |
| **修复 agent** | 审计结论（仅真漂移 + 已裁决定论的需人裁决项） | 改码 + 补测试 + 飞轮回写（§3）+ 自验 | 不得改审计结论 |

修复 agent 改完必须自验：`cargo test -p <改动 crate>` + `cargo clippy -p <改动 crate> --all-targets -- -D warnings`
全绿才进「飞轮回写」，全绿前不许回写。

## 2. 开轮必读清单（飞轮增强核心，EXP-068）

每个子代理（找问题 / 审计 / 修复，含并行轮包会话）开轮 prompt **必须**附以下 6 条，
并在回报里**逐条回执**（读了哪份、命中几条现行规范、是否读到与本包域相关的 EXP）：

1. `docs/process/00_index.md`（分工）+ `docs/business/00_index.md`（业务唯一事实源）
2. `docs/process/01_architecture.md` + `docs/process/02_conventions.md`（工程规范）
3. `docs/contracts/00_index.md` + 本次涉及的 c1/c2/c3（跨包面）
4. `docs/experience/index.md`（EXP 总表，**重点读与本次改动域相关的 EXP**，
   如 EXP-061 行内全路径、EXP-064 环境事实前置、EXP-067 跨层几何正逆成对测试）
5. `scripts/gate.ps1` 当前项清单 + `doccheck2` 当前断言清单（知道门禁查什么，
   避免产出必然被门禁拦的写法）
6. **在途并行包白名单**（本轮 P 包清单 + 各自文件域，避免误改他包文件）

**回执判据**：缺任一清单条目的下发 = 任务书不合格；子代理回报缺「规范回执」段
（逐条列出读了哪些 + 命中几条 + 相关 EXP）= 回报不合格，打回。

## 3. 飞轮回写（每轮必做）

| 发现类型 | 回写目标 | 判据 |
|---|---|---|
| 可机械判定的漂移 | `crates/ti-tools/src/bin/doccheck2.rs` 新断言 | 下一条能自动 grep/断言复现 |
| 需人裁决 / 设计分歧 | `docs/experience/` 新 EXP + `evidence/<task>.log` | 争议未定论，记录裁决理由 |
| 重复踩坑（≥2 次同模式） | EXP +（可选）lint/`scripts/check_*.py` | 第 3 次前必须固化（EXP-048） |
| 进度 | `plan/ledger.yaml` verified（**仅集成轮给**）+ `evidence/<task>.log` | 实跑输出 tee，非 gate 总输出 |

回写必须进门禁视野：新 doccheck2 断言需同步更新 `doccheck2 .` 的 checks 计数；
新 EXP 需登记 `docs/experience/index.md`（EXP-063：文档引用 ID 前先登记）。

## 4. 停机条件

某轮同时满足：
- 找问题 = **0 新发现**（发现清单空）；
- 审计 = **0 误报**（无「误报」结论项）；
- 修复 = **0 残留**（审计结论全部闭环，无 blocked/需人裁决遗留）。

三条齐备 → 飞轮收敛，停机；任一条未满足 → 开下一轮。
停机结论需主线程复核（EXP-038/EXP-058：子代理声称不算数），复核证据入 `evidence/flywheel.log`。

## 5. 与 05_module_parallel 的关系

| | 并行轮（05） | 飞轮（本文） |
|---|---|---|
| 管什么 | 模块边界各写各的（契约→并行→集成 4 步） | 写完后找问题-审计-修复-回写 |
| 开轮必读 | §4.5 引用本文 §2 | 唯一事实源在本文 §2 |
| 停机 | 终止条件见 `docs/loop.md` §3 | §4 三条收敛判据 |
| 台账 | 集成轮给 verified | 飞轮不写 verified（除非被并入集成轮） |

## 6. 开轮必读脚本化（EXP-070）

EXP-068 规定开轮必读 6 条，但 LLM 每次需逐个 Read 工具调用 6 份文档 + 读 gate + doccheck2 + packages.yaml，
N 个子代理各跑一遍，工具调用爆炸 + 容易漏读（本轮 P6 子代理没读 06_flywheel 的在途白名单）。

**脚本化替代**：

```
powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package <本包>
# 例如：
powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package P6
```

输出包含：
1. 环境事实（工作目录 / shell / crate 前缀 / 逆向产物位置）
2. 必读文档 6 条（逐条路径 + 一句话要点）
3. 在途并行包白名单（读 `plan/packages.yaml` 机器生成，含 P1–P6 + D 的 files/tests）
4. 本域相关 EXP（按包域关键词过滤 `docs/experience/index.md`）
5. gate 项清单（从 `scripts/gate.ps1` 正则提取）
6. doccheck2 断言清单（从 `doccheck2.rs` 正则提取 `fn check_*` 函数名）

脚本读的是各文件的**实时内容**，规范/EXP/门禁变更后下次跑自动反映，无需改 prompt。

**LLM 子代理开轮流程（替代手动读文档）**：
1. 跑 `powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package <本包>`；
2. 确认输出完整（6 条必读 + 白名单 + 本域 EXP + gate 项 + doccheck2 断言均非空）；
3. 回执「已读脚本输出，命中 N 条规范」，不再逐个 Read 工具调用；
4. 若输出中某条 EXP 与本次改动强相关 → 再读该 EXP 正文（分卷文件）；
5. gate 加 `flywheel-readme -Check` 项（第 17 项）验证脚本输出完整性，缺项 FAIL。

## 7. 与 05_module_parallel 的关系

| | 并行轮（05） | 飞轮（本文） |
|---|---|---|
| 管什么 | 模块边界各写各的（契约→并行→集成 4 步） | 写完后找问题-审计-修复-回写 |
| 开轮必读 | §4.5 引用本文 §2 | 唯一事实源在本文 §2 + §6 脚本化 |
| 停机 | 终止条件见 `docs/loop.md` §3 | §4 三条收敛判据 |
| 台账 | 集成轮给 verified | 飞轮不写 verified（除非被并入集成轮） |

## 8. 单轮流程

```
① 开轮（主线程）
   跑 scripts/flywheel_readme.ps1 -Package <包> 拿必读清单 → 并发下发 3 角色 prompt
② 找问题（只读）
   发现清单（文件:行 + 类别 + 证据），禁止改文件
③ 审计
   逐条裁决 → 审计结论
④ 修复 + 自验
   改码/测试 + cargo test/clippy 全绿 → 飞轮回写（§3 表）
⑤ 主线程复核 + 判定停机（§4）
   收敛 → 停机 + evidence/flywheel.log；未收敛 → 下一轮
```
