---
name: flywheel-kickoff
description: Use when the user asks to start / run a flywheel round (对抗流水线飞轮：找问题→审计→修复→回写→停机), or "开一轮飞轮", "飞轮开轮", "跑飞轮", "找问题-审计-修复-回写", "3 角色并发". Orchestrates one full flywheel round per docs/process/06_flywheel.md §单轮流程.
---

# 飞轮开轮（flywheel-kickoff）

编排**一整轮**对抗流水线飞轮：开轮下发 → 找问题 → 审计 → 修复+自验 → 飞轮回写 → 主线程复核+停机判定。
流程与 3 角色/回写/停机判据的**唯一事实源**是 [`docs/process/06_flywheel.md`](../../../docs/process/06_flywheel.md)（§1 3 角色 / §2 开轮必读 / §3 回写 / §4 停机 / §7 单轮流程）；本 skill 只做**编排**，细节交各专项 skill，不重述。

## 触发场景

- 「开一轮飞轮」「跑飞轮」「飞轮开轮」「找问题-审计-修复-回写」「3 角色并发」
- 某包/全域开发完成，需要一轮质量闭环（区别于并行开发轮，见 `05_module_parallel`）

## 前置（不满足先补齐，别硬开）

1. 选定**本轮包域**（`plan/packages.yaml` 的一个 P 包 / D / 或「全域」）。
2. 若 `scripts/gate.ps1` 当前有 FAIL：先按 `gate-green` skill 归因（是不是并行半成品），**半成品不修就直接开飞轮会把别人 WIP 当 bug 抓**。
3. 在途并行包白名单已知（避免修复 agent 误改他包文件）——`flywheel_readme.ps1` 会自动从 `plan/packages.yaml` 生成。

## 流程（对应 06_flywheel §7 单轮 5 步）

### ① 开轮（主线程）
```powershell
powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package <本包>
# 全域轮省略 -Package；-Check 只校验输出完整性（gate 第 17 项）
```
- 确认输出 6 项齐全：环境事实 / 必读 6 条 / 在途白名单 / 本域 EXP / gate 项清单 / doccheck2 断言清单（EXP-070 脚本化，替代逐个 Read）。
- 把该输出 + 本包域 + 白名单，作为 **3 角色 prompt 的共同上下文**下发（并发）。

### ② 找问题（只读 agent）
- 输入：开轮必读 + 本包/全域代码。输出「发现清单」：每条 `文件:行` + 类别（漂移/越界/硬编码/未实现/风格）+ 证据。
- **禁改任何文件**。**怎么组织找问题 + 对抗审计 = 见 `rea-audit-fix` skill**（本步复用它的找问题/审计纪律）。

### ③ 审计（agent）
- 输入：发现清单。逐条裁决 **真漂移 / 误报 / 需人裁决**，附理由。**禁改文件**。
- 主线程对最高价值 8 条亲自 Read/Grep 复核一遍再进修复（EXP-038/058：子代理声称不算数）。

### ④ 修复 + 自验（agent）
- 仅修「真漂移 + 已裁决定论的需人裁决项」。改码 + 补测试。
- **自验全绿才允许回写**：`cargo test -p <改动 crate>` + `cargo clippy -p <改动 crate> --all-targets -- -D warnings`。
- 飞轮回写（**按 06_flywheel §3 表对号入座**）：
  - 可机械判定的漂移 → `crates/ti-tools/src/bin/doccheck2.rs` 新断言（**补断言步骤唯一源 = `doc-check` skill §修完后**）
  - 需人裁决/设计分歧 → `docs/experience/` 新 EXP + `evidence/<task>.log`（**登记步骤 = `exp-record` skill**）
  - 重复踩坑 ≥2 次 → EXP +（可选）lint / `scripts/check_*.py`
- 每处文档/代码改动**用 Edit 工具精确替换，禁 PowerShell `Set-Content`/`Out-File` 改 CJK 文件**（EXP-051）。

### ⑤ 主线程复核 + 停机判定（06_flywheel §4）
某轮**同时**满足才停机：找问题 = 0 新发现 **且** 审计 = 0 误报 **且** 修复 = 0 残留（无 blocked/需人裁决遗留）。
- 三条齐 → 停机，停机结论 + 复核证据写 `evidence/flywheel.log`。
- 任一未满足 → 开下一轮（回 ①）。

## 收口校验

```powershell
powershell -ExecutionPolicy Bypass -File scripts/gate.ps1                    # 项数以 docs/parity.md §代码质量门禁 计数约定为准
cargo run -q -p ti-tools --bin doccheck2 .                                  # 必须 0 error（新增断言同步 checks 计数）
powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Check  # 第 17 项，输出完整性
```

## 与其它 skill 的分工（避免重复）

| 步 | 本 skill | 交出的专项 skill |
|---|---|---|
| 开轮必读 | 跑 `flywheel_readme.ps1` 下发 | —（脚本，EXP-070） |
| 找问题/审计 | 编排 + 主线程复核 | `rea-audit-fix`（对抗审计 + doccheck2 固化） |
| 修复后 gate | 收口跑 gate | `gate-green`（FAIL 归因） |
| 回写 EXP | 触发回写 | `exp-record`（选卷/取号/登记） |
| 回写 doccheck2 | 触发补断言 | `doc-check`（补 `check_xxx` + `all_errors.extend`） |

## 硬约束

- 3 角色里**找问题/审计 只读**，只有修复 agent 能改文件，且不得改审计结论。
- 自验（cargo test/clippy）全绿前**不许**回写（EXP-048/飞轮 §3 判据）。
- 停机结论**必须主线程复核**（子代理声称「收敛」不算数，EXP-038/058），复核证据入 `evidence/flywheel.log`。
- 飞轮本身**不写 ledger verified**（除非并入集成轮，06_flywheel §5）。
