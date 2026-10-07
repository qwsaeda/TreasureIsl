# 循环协议（Loop Protocol）

进度推进的唯一机制。任何会话开工前先读本文 + `plan/ledger.yaml`，结束时台账必须是合法状态。

## 1. 台账 `plan/ledger.yaml`

```yaml
version: 1
tasks:
  - id: S0.1              # 对应 docs/plan-s0-s2.md 中的任务 ID
    title: 建 cargo workspace + 门禁
    status: pending       # pending | in_progress | blocked | verified
    deps: []              # 依赖任务 ID，全部 verified 才可取用
    dod: "scripts/gate.sh 全绿"        # 验收判据（抄自 WBS）
    evidence: null        # verified 后填 evidence/<id>.log 相对路径
    blocked_by: null      # blocked 时填原因
    rounds: 0             # 已失败验收轮数，>=3 必须拆子任务
```

规则：
- `status` 只能按 `pending → in_progress → verified` 单向流转；回退只能到 `in_progress`，并写 `blocked_by`。
- `verified` 必须有 `evidence`，否则台账校验（gate 第 8 项）失败。
- `rounds >= 3` 的任务必须拆成子任务（新增 `Sx.y.a` 形式 ID）后才可继续。

## 2. 循环体（每轮迭代）

```
1. 读 ledger → 选第一个 deps 全部 verified 且 status != verified 的任务
2. 实现（按 WBS 产出物落地，禁止留占位实现）
3. 跑该任务的验收命令 → 输出存 evidence/<id>.log
4. 跑 scripts/gate.sh（全局门禁，8 项）
5. gate 全绿 → ledger 置 verified + evidence；否则 rounds+=1，按规则拆解/标记 blocked
6. 输出进度报告：verified/total、阻塞项、下一轮目标
7. 回到 1
```

## 3. 终止条件（全部满足才算项目完成）

1. ledger 中全部任务 `verified`；
2. `docs/parity.md` 的 P0~P4 五级标准全部 pass（各自校验命令产出报告存 `evidence/`）;
3. 三端产物生成：桌面 exe、wasm 站点、APK；
4. `scripts/gate.sh` 8 项全绿，占位扫描 0 命中。

## 4. 卡死与回退

| 情况 | 处置 |
|---|---|
| 验收连续失败 3 轮 | 拆子任务，原任务转 `blocked`，`blocked_by` 写具体缺口 |
| 发现计划与实测冲突 | 以实测为准 → 更新 `docs/reverse-facts.md`，再改 WBS/台账，**不许静默绕过** |
| 踩到新坑（工具/格式/平台） | 立即写入 `docs/experience.md`（格式见该文件），并在任务 evidence 中引用 |
| 需要暂停（外部依赖缺失） | 任务标 `blocked`，继续推进无依赖任务，不空转 |

## 5. 会话交接

- 会话结束前：ledger 合法、`evidence/` 无缺失、经验库已回写。
- 会话开始时：先 `git status`（如已 init）+ 读 ledger + 读 `docs/experience.md` 最近条目，避免重复踩坑。
