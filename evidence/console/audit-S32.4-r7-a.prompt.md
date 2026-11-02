# 审计任务书 S32.4 · r7 · A 侧（收敛验证轮）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r6 收敛验证轮**（复核 r6 的 2 项 findings 是否已修复 + DoD 仍满足）。

## r6 需复核的 2 项 findings（均 P2，见 `evidence/console/audit-S32.4-r6-b.md`）

**r6-b F1（r5 报告未落盘）**：修复应 = `evidence/console/audit-S32.4-r5-{a,b}.md` 持久化。验证：`ls evidence/console/audit-S32.4-r{1,2,3,4,5}-{a,b}.md` 10 文件存在；每份第一个 ``` 代码块含四行。

**r6-b F2（apk_size 两处 python3 未套最小环境集）**：修复应 = `repo/data.rs` `apk_size` 的 `apk_audit.py --apk` / `--refset` 两处 `Command::new` 补 `crate::runner::apply_minimal_env(&mut cmd)`。验证：grep "apply_minimal_env" data.rs ≥5 处（run_test/build_artifacts×2/apk_size×2）；`/api/apk-size` 实跑 200 且 `verdict=PASS`（env_clear 后 apk_audit.py 仍工作）。

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
- `新发现` = 本报告新发现数；`误报` = 0；`残留` = r6 未闭环数（你判 0 或如实）。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`。
- 必须引用**文件路径 + 行号**或**命令实跑输出**；可用只读命令：cat/grep/ls/curl/cargo clippy -p ti-console/cargo test -p ti-console（只读）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
