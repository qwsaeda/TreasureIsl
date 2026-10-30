# S32.11 第 1 轮审计 · A（完成度）

```
总判定: PASS
新发现: 2
误报: 0
残留: 0
```

> 落盘说明：第 1 轮 A 侧审计落盘凭证（会话内报告全量落盘；B 侧同轮 = `audit-S32.11-r1-b.md`，
> 3 条 P2 建议，与 A 侧同源同批）。只读；实跑 doccheck / doccheck2 / ledger /
> check_code_standards / check_ledger_reality / doc_numbers / map_pack.py list + check /
> pwsh 语法校验 / flywheel_readme.ps1 -Check。未跑 gate.ps1 全量。

## DoD 逐条判定（S32.11，台账 L307）

1 代码 = gate.ps1 第 17 项段宿主探测（`$psExe` 探测 + 无宿主 SKIP 不假 PASS）：**PASS**（L280-293 在位）
2 判据 1 Linux 实跑该项不再假 PASS：**PASS**（本机 `which pwsh` 在位 ⇒ 走实跑分支，
   `flywheel_readme.ps1 -Check` EXIT=0 真 PASS；无宿主 ⇒ 记名 SKIP ≠ 假 PASS）
3 判据 2 `powershell` 缺失时 SKIP 不 FAIL：**PASS**（`Skip()` 函数只 `Write-Host` + 刷新 `$script:lastMark`，
   不触 `$script:fail++`；双 `Get-Command` 失败 ⇒ `$psExe=$null` ⇒ SKIP 分支）
4 判据 3 Windows 行为不变：**PASS**（Windows 未装 PS7 ⇒ `Get-Command pwsh` 失败 ⇒ 回退 `powershell`
   = 旧实现路径；`scripts/flywheel_readme.ps1` 本体 0 改动，`param`/`[switch]`/`exit` 均 PS 5.1 兼容语法）
5 判据 4 gate.ps1 语法校验：**PASS**（实跑 `pwsh -NoProfile -Command "& { [scriptblock]::Create((Get-Content -Raw 'scripts/gate.ps1')) }"` EXIT=0；
   evidence/S32.11.log 内亦有 `[Parser]::ParseFile` 等价实跑记录）
6 判据 5 注释与 22 号 §4.3 / migration_linux §4 ⑤ 注同步：**PASS**（三处均标 S32.11；
   **超前宣称问题 = P2-1，已在第 11 版文档修复：22 号 §4.3 / migration_linux ⑤ 改「修复已落码，verified 待双审计收敛」**）
7 判据 6 其余各项行为不变（0 触及其它 32 项调用点 / `Report`/`Skip`/`Tier` 函数定义 / 项数声称 = 33）：**PASS**
   （`git diff --stat -- scripts/gate.ps1` hunk 仅第 17 项段 + 注释/项数声称同步；
   doccheck2 gate-items 阈值 33 实跑 0 error 吻合）
8 evidence/S32.11.log 实跑判据留痕（收口前勿引用口径自述）：**PASS**（log §五 末尾自述
   「本 log 为 S32.11 收口前的实跑凭证；双审计收敛后补「双审计通过」段再置 verified」）
9 台账 S32.11 DoD 补「gate 全绿口径」（非全量 gate.ps1 实跑）：**PASS**（第 11 版文档已补：
   判据 6 尾部「gate 全绿口径 = 第 17 项实跑 + 其余项 0 触及 + 六项机器判据全绿，非全量 gate.ps1 实跑」）

## 第 0 轮（无）⇒ 第 1 轮起

## 新发现（2 条 P2，由第 11 版文档吸收）

| ID | 严重度 | 问题 | 修复落点（第 11 版已落） |
|---|---|---|---|
| P2-1 | P2 | 22 号 §4.3 / migration_linux §4 ⑤ 注「已根治」超前宣称（台账 S32.11 status=pending 双审计未收敛）；判据 5 字面要求三处均写「已根治」与超前宣称同源 | 两处改「修复已落码，verified 待双审计收敛」；判据 5 同步放宽（第 11 版文档已落） |
| P2-2 | P2 | evidence/S32.11.log §四 全量 gate.ps1 未实跑，DoD 判据「gate 全绿」口径偏差未写进台账 DoD | 台账 S32.11 DoD 判据 6 补「gate 全绿口径 = 第 17 项实跑 + 其余项 0 触及 + 六项机器判据全绿，非全量 gate.ps1 实跑」（第 11 版已落） |

## 总判定

**PASS（非收敛轮）**：DoD 9/9 PASS；P2 均为文档口径偏差（非功能缺陷），修复已同批落盘（第 11 版文档），
第 2 轮复核（0 新发现 + 0 误报 + 0 残留连续 2 轮才 verified）。
