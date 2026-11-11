---
name: flywheel-kickoff
description: Use when the user asks to start / run a flywheel round (对抗流水线飞轮：找问题→审计→修复→回写→停机), or "开一轮飞轮", "飞轮开轮", "跑飞轮", "找问题-审计-修复-回写", "3 角色并发". Orchestrates one full flywheel round per docs/process/06_flywheel.md §单轮流程. 强制规范结构下发：subagent prompt 头部 = flywheel_readme 输出（7 条必读 + 本域红线注入）+ 红线基准面 + 五步回执字段 + A/B 双审计对抗。
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

## 流程（对应 06_flywheel §7 单轮 5 步 + S51 规范结构下发）

### ① 开轮（主线程）
```powershell
powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package <本包>
# 全域轮省略 -Package；-Check 只校验输出完整性（gate 第 17 项）
```
- 确认输出 **7 项**齐全：环境事实 / 必读 7 条（第 7 条 = 开轮协作五步，S51.0）/ 在途白名单 / **本域红线（docs/rules 按包域 glob 注入 1~3 个）** / 本域 EXP / gate 项清单 / doccheck2 断言清单。
- **强制下发**：把该输出**全文**作为 3 角色 subagent prompt 的**固定头部**（不手工重述——手搓 prompt 丢规范 = 机制两张皮，EXP-186/飞轮 R 教训）。

### ② 找问题（只读 agent，CCGS 五步规范结构）

**subagent prompt 固定结构**（主线程照此下发，不是任务描述式）：

```
【规范头部】（flywheel_readme.ps1 -Package <域> 输出全文，逐字粘贴）
  - 7 条必读 + 本域红线（docs/rules/<x>-rules.md）+ 在途白名单 + 本域 EXP + gate/doccheck2 清单
【角色契约】
  - 只读，禁改任何文件
  - 审计基准面 = 本域红线文件（docs/rules/<x>-rules.md）+ 业务唯一事实源（docs/business/）
    + 数据面（assets/data/*.json）——逐条发现须对照红线判定「是否越界/硬编码/漂移」
【五步回执字段】（回报末尾必含，写码类全走；只读角色报「0 个决策点」）
  - 提问：开轮前列 1~3 个澄清问题（边界/数值/口径未写明的）；无 = 「0 个待澄清」
  - 方案：每个待决策点给 2~4 种处置建议 + 利弊（不直接挑一个就动手）；纯机械 = 「0 个决策点直接落地」
  - 拍板留痕：所选方案 + 主线程/用户拍板记录（消息 ID / 台账 dod）
  - 草稿批准状态：只读角色 = 「豁免（不改文件）」
【产出】发现清单（每条 文件:行 + 类别 + 实跑证据 + 落点建议）
```

- **禁改任何文件**。怎么组织找问题 + 对抗审计纪律 = 见 `rea-audit-fix` skill。

### ③ 审计（**A/B 双 agent 对抗**，CCGS 双审计协议 S50）

**不再用单线程审计**（飞轮 R 教训：单线程审计 = 降级）。改 A/B 两个独立 subagent：

**A 侧（完成度审计）**：
```
【规范头部】（同②，flywheel_readme 输出全文）
【角色契约】只读；输入 = ②发现清单
【任务】逐条裁决 真漂移/误报/需人裁决 + 补找 ②漏报的盲区（按红线面 + 业务规格 + 数据面三方对账）
【五步回执】提问/方案/拍板留痕/草稿批准状态（同上）
```

**B 侧（一致性审计）**：
```
【规范头部】（同②，flywheel_readme 输出全文）
【角色契约】只读；输入 = ②发现清单（与 A 独立，不读 A 结论）
【任务】逐条交叉验 ③文档↔④数据↔①代码 三方一致性 + 审计面自身漂移（doccheck2 断言口径过期/恒 PASS/误报）
【五步回执】（同上）
```

- **收敛判据**（CCGS 双审计 §7.4）：A/B 各出 findings → 主线程合并 → 修复 → **再开 A/B r2** → 连续 **2 轮 0/0/0**（0 新发现 / 0 误报 / 0 残留）才停机。
- 主线程对最高价值 8 条 P0 **亲自 Read/Grep 复核**（EXP-038/058：子代理声称不算数），复核结论写 evidence。

### ④ 修复 + 自验（写码 agent，CCGS 五步全走）
- 仅修「真漂移 + 已裁决定论的需人裁决项」。改码 + 补测试。
- **prompt 固定结构**：
```
【规范头部】（flywheel_readme 输出全文 + 本域红线 = 改前必读面）
【角色契约】可写；改动面 = 白名单内 + 审计裁定的真漂移清单
【五步全走】提问→方案（2~4 种修法 + 利弊）→拍板留痕→草稿（diff 先过目）→批准后才落地
【红线基准】改前对照 docs/rules/<域>-rules.md 逐条，改动面外的红线不碰
```
- **自验全绿才允许回写**：`cargo test -p <改动 crate>` + `cargo clippy -p <改动 crate> --all-targets -- -D warnings`。
- 飞轮回写（**按 06_flywheel §3 表对号入座**）：
  - 可机械判定的漂移 → `crates/ti-tools/src/bin/doccheck2.rs` 新断言（**补断言步骤唯一源 = `doc-check` skill §修完后**）
  - 需人裁决/设计分歧 → `docs/experience/` 新 EXP + `evidence/<task>.log`（**登记步骤 = `exp-record` skill**）
  - 重复踩坑 ≥2 次 → EXP +（可选）lint / `scripts/check_*.py`
- 每处文档/代码改动**用 Edit 工具精确替换，禁 PowerShell `Set-Content`/`Out-File` 改 CJK 文件**（EXP-051）。

### ⑤ 主线程复核 + 停机判定（06_flywheel §4）
某轮**同时**满足才停机：找问题 = 0 新发现 **且** 审计 A/B = 0 误报 **且** 修复 = 0 残留（无 blocked/需人裁决遗留）+ **A/B 连续 2 轮 0/0/0 收敛**。
- 四条齐 → 停机，停机结论 + 复核证据写 `evidence/flywheel.log`。
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
| 开轮必读 | 跑 `flywheel_readme.ps1` 下发（全文作 prompt 头部） | —（脚本，EXP-070） |
| 找问题/审计 | 编排 A/B 双 agent 对抗 + 主线程复核 | `rea-audit-fix`（对抗审计 + doccheck2 固化） |
| 修复后 gate | 收口跑 gate | `gate-green`（FAIL 归因） |
| 回写 EXP | 触发回写 | `exp-record`（选卷/取号/登记） |
| 回写 doccheck2 | 触发补断言 | `doc-check`（补 `check_xxx` + `all_errors.extend`） |

## 硬约束

- 3 角色里**找问题/审计 只读**，只有修复 agent 能改文件，且不得改审计结论。
- **规范结构下发强制**：3 角色 subagent prompt 头部 = `flywheel_readme.ps1` 输出全文（不手搓任务描述式 prompt，防机制两张皮）。
- **五步回执强制**：写码类 agent 回报必含「提问/决策点+拍板留痕/草稿批准状态」3 字段；只读角色报「0 个决策点」+「草稿批准状态=豁免」。缺字段 = 主线程拒收回报。
- **审计 = A/B 双 agent 对抗**（不降级单线程）；连续 2 轮 0/0/0 才停机（CCGS 双审计 §7.4 轮次完整性口径）。
- 自验（cargo test/clippy）全绿前**不许**回写（EXP-048/飞轮 §3 判据）。
- 停机结论**必须主线程复核**（子代理声称「收敛」不算数，EXP-038/058），复核证据入 `evidence/flywheel.log`。
- 飞轮本身**不写 ledger verified**（除非并入集成轮，06_flywheel §5）。
