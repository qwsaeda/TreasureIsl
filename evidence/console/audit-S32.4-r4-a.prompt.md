# 审计任务书 S32.4 · r4 · A 侧（收敛验证轮）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r3 收敛验证轮**（复核 r3 的 2 项 findings 是否已修复 + DoD 仍满足）。

## r3 需复核的 2 项 findings

**r3-b P1（`gate.lock` 未入 `.gitignore`）**：`.gitignore` S32 段漏 `gate.lock`，运行时锁产物会误入 git。
- 修复应 = `.gitignore` 加 `/evidence/console/gate.lock`。
- 验证：`git check-ignore evidence/console/gate.lock` 应输出该路径（被忽略）。

**r3-b P2（锁损坏/空 → 永久 409 无接管）**：进程崩溃于 `create_new`→`writeln` 微秒窗口 → 锁文件空/半截 → `read_lock` None → 任何 POST 恒 409，需手动删锁。
- 修复应 = 锁预检区分「损坏」：锁存在但非合法 JSON 且文件超 5 秒 → 接管；< 5 秒（可能正写入）→ 409。
- 验证：读 `check_lock_available`（锁预检函数），确认 `None` 分支含 `lock_older_than(path, CORRUPT_LOCK_MIN_AGE_SECS)` 判断；`run_gate` 首调 `check_lock_available(repo_root)?`。

## DoD 仍满足抽查

- `evidence/console/logs/gate-1793582497.log` = 17 PASS / 0 FAIL / 16 SKIP，末行 `=== gate: 0 failure(s) ===`（r3 修复后重跑 fast 仍 PASS）。
- `cargo clippy --workspace --all-targets -- -D warnings` 0 error；`cargo fmt --all -- --check` OK；`cargo test -p ti-console --test gate_tests` 6 过。
- 运行时验证（只读）：损坏锁 <5s → POST 409；损坏锁 >5s → POST 接管启动（`started=true`）。

## 报告格式契约

报告正文**第一个 ``` 代码块**内必须含四行（缺任一行 ⇒ 判不可解析）：
```
总判定: PASS|FAIL
新发现: <整数>
误报: 0
残留: <整数>
```
- `新发现` = 本报告新发现数；`误报` = 0；`残留` = r3 未闭环数（你判 0 或如实）。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`。
- 必须引用**文件路径 + 行号**或**命令实跑输出**；可用只读命令：cat/grep/ls/git check-ignore/cargo clippy -p ti-console/cargo test -p ti-console（只读）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
