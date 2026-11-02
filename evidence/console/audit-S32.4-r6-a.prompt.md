# 审计任务书 S32.4 · r6 · A 侧（收敛验证轮）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r5 收敛验证轮**（复核 r5 的 2 项 findings 是否已修复 + DoD 仍满足）。

## r5 需复核的 2 项 findings（均 P2，见 `evidence/console/audit-S32.4-r5-b.md`）

**r5-b F1（r4 报告未落盘）**：修复应 = `evidence/console/audit-S32.4-r4-{a,b}.md` 持久化。验证：`ls evidence/console/audit-S32.4-r{1,2,3,4}-{a,b}.md` 8 文件存在；每份第一个 ``` 代码块含四行（总判定/新发现/误报/残留）。

**r5-b F2（console_commands.json 重复 id `build_check`）**：修复应 = 第二处（`python3 scripts/build_artifacts.py check`）改名 `build_artifacts_check`（console_commands.json + 22 号 §4.2 表同步）。验证：python3 解析 console_commands.json，id 集合 14 唯一（Counter 无重复）；grep "build_artifacts_check" 22_dev_platform.md。

## DoD 仍满足抽查

- `evidence/console/logs/gate-1793583779.log` = 17 PASS / 0 FAIL / 16 SKIP，末行 `=== gate: 0 failure(s) ===`。
- `cargo clippy --workspace --all-targets -- -D warnings` 0 error；`cargo fmt --all -- --check` OK；`cargo test -p ti-console` 全绿。

## 报告格式契约

报告正文**第一个 ``` 代码块**内必须含四行（缺任一行 ⇒ 判不可解析）：
```
总判定: PASS|FAIL
新发现: <整数>
误报: 0
残留: <整数>
```
- `新发现` = 本报告新发现数；`误报` = 0；`残留` = r5 未闭环数（你判 0 或如实）。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`。
- 必须引用**文件路径 + 行号**或**命令实跑输出**；可用只读命令：cat/grep/ls/python3 -c/cargo clippy -p ti-console/cargo test -p ti-console（只读）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
