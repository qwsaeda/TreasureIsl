# 审计任务书 S32.4 · r8 · A 侧（收敛确认轮）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r7 收敛确认轮**（确认连续第 2 轮全 0：r7 为首次全 0 轮，r8 须再全 0 才置 verified；复核 r7 无未闭环 + 无新发现 + DoD 仍满足）。

## r7 全 0 确认 + 收敛断言（A 侧重点）

1. **r7 报告在盘且四行齐全**：`evidence/console/audit-S32.4-r7-{a,b}.md` 存在，第一个 ``` 代码块含四行（总判定/新发现/误报/残留）。验证：`ls evidence/console/audit-S32.4-r7-{a,b}.md` + awk 抽取四行。
2. **r1~r7 报告 14 份落盘断言**：`ls evidence/console/audit-S32.4-r{1..7}-{a,b}.md` = 14 文件；每份第一个 ``` 代码块含四行机器字段（总判定/新发现/误报/残留）。验证：`ls ... | wc -l` = 14 + 逐份 awk。
3. **r7 无未闭环**：r7-a 报告 `新发现: 0` / `误报: 0` / `残留: 0` 与 r7-b 相同（读两份 r7 报告确认数字 = 0/0/0）。
4. **DoD 仍满足抽查**：`evidence/console/logs/gate-1793583779.log` = 17 PASS / 0 FAIL / 16 SKIP，末行 `=== gate: 0 failure(s) ===`；`cargo clippy --workspace --all-targets -- -D warnings` 0 error；`cargo fmt --all -- --check` OK；`cargo test -p ti-console` 全绿。

## 报告格式契约

报告正文**第一个 ``` 代码块**内必须含四行（缺任一行 ⇒ 判不可解析）：
```
总判定: PASS|FAIL
新发现: <整数>
误报: 0
残留: <整数>
```
- `新发现` = 本报告新发现数（含 P2 建议）；`误报` = 0；`残留` = r7 未闭环数（你判 0 或如实）。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`（P0=阻断 / P1=应修 / P2=建议）。
- 必须引用**文件路径 + 行号**或**命令实跑输出**，禁止「看起来没问题」。可用只读命令：cat/grep/ls/awk/cargo clippy -p ti-console/cargo test -p ti-console（只读）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
