# 审计任务书 S32.4 · r5 · A 侧（收敛验证轮）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r4 收敛验证轮**（复核 r4 的 4 项 findings 是否已修复 + DoD 仍满足）。

## r4 需复核的 4 项 findings（均 P2，见 `evidence/console/audit-S32.4-r4-b.md`）

**r4-b F1（r3-b 报告未落盘）**：修复应 = `evidence/console/audit-S32.4-r3-b.md` 持久化。验证：`ls evidence/console/audit-S32.4-r{1,2,3}-{a,b}.md` 6 文件存在；每份第一个 ``` 代码块含四行（总判定/新发现/误报/残留）。

**r4-b F2（损坏锁分支无单测）**：修复应 = `gate_tests.rs` 补损坏锁 2 测（`check_lock_available` 超 5 秒接管 Ok + <5 秒 409）。验证：`grep -n "corrupt_lock" crates/ti-console/tests/gate_tests.rs` 2 个 `#[test]`；`cargo test -p ti-console --test gate_tests` 8 过。

**r4-b F3（§10.4 未登记损坏锁语义）**：修复应 = `docs/process/22_dev_platform.md` §10.4 gate.lock 段补「损坏锁接管（r3-b P2）：非合法 JSON + 超 5 秒接管 / <5 秒 409」。验证：grep "损坏锁接管" 22_dev_platform.md。

**r4-b F4（§10.3 最小环境集未实现）**：修复应 = `runner/mod.rs` `MINIMAL_ENV_KEYS` + `apply_minimal_env`（env_clear + 白名单）应用于 `spawn_gate`（gate.rs）+ `run_test`/`build_artifacts`（data.rs）。验证：grep "apply_minimal_env" gate.rs + data.rs（≥3 处）；读 mod.rs `MINIMAL_ENV_KEYS` 含 §10.3 13 键。

## DoD 仍满足抽查

- `evidence/console/logs/gate-1793583779.log` = 17 PASS / 0 FAIL / 16 SKIP，末行 `=== gate: 0 failure(s) ===`（r4 修复后重跑 fast 仍 PASS，env_clear 后子进程面工作）。
- `cargo clippy --workspace --all-targets -- -D warnings` 0 error；`cargo fmt --all -- --check` OK；`cargo test -p ti-console` 全绿。

## 报告格式契约

报告正文**第一个 ``` 代码块**内必须含四行（缺任一行 ⇒ 判不可解析）：
```
总判定: PASS|FAIL
新发现: <整数>
误报: 0
残留: <整数>
```
- `新发现` = 本报告新发现数；`误报` = 0；`残留` = r4 未闭环数（你判 0 或如实）。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`。
- 必须引用**文件路径 + 行号**或**命令实跑输出**；可用只读命令：cat/grep/ls/cargo clippy -p ti-console/cargo test -p ti-console（只读）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
