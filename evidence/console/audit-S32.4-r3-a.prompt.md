# 审计任务书 S32.4 · r3 · A 侧（收敛验证轮）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r2 收敛验证轮**（复核 r2 的 3 项 findings 是否已修复 + DoD 仍满足）。

## r2 需复核的 3 项 findings

**r2-a P2-1（日志侧同秒竞态）**：原 `runner/gate.rs` 日志文件在 `create_new` 锁**之前**创建 + 秒级日志名 → 并发同秒两 POST 时 409 败者 `remove_file` 误删持锁者日志。
- 修复应 = 日志创建移到 `create_new` 成功**之后**（持锁后建日志；409 败者不再触碰日志）。
- 验证：读 `run_gate`，确认 `File::create(&log_path)` 出现在 `OpenOptions::create_new(true)` 成功分支**之后**；`AlreadyExists` 分支**不再** `remove_file(&log_path)`（只返回 409）。

**r2-b P2-1（`command` 硬编码 pwsh）**：响应 `command` 字段硬编码 `"pwsh -File ..."`，未反映实际回退解释器。
- 修复应 = `spawn_gate` 返回实际解释器名 + 响应加 `interp` 字段 + `command` 用实际 interp。
- 验证：读 `spawn_gate` 返回 `Result<(Child, &'static str), String>`；`run_gate` 解构 `(child, interp)`；响应含 `"interp": interp` + `command` 用 `interp`。

**r2-b P2-2（r1 报告未落盘）**：`evidence/console/audit-S32.4-r1-{a,b}.md` 不存在（仅 `.prompt.md`）。
- 修复应 = 主线程持久化 r1-a/r1-b（及 r2-a/r2-b）报告到 `evidence/console/`。
- 验证：`ls evidence/console/audit-S32.4-r{1,2}-{a,b}.md` 4 文件存在；每个含第一个 ``` 代码块四行（总判定/新发现/误报/残留）。

## DoD 仍满足抽查

- `evidence/console/logs/gate-1793581541.log` = 17 PASS / 0 FAIL / 16 SKIP，末行 `=== gate: 0 failure(s) ===`（r2 修复后重跑 fast 仍 PASS）。
- `cargo clippy --workspace --all-targets -- -D warnings` 应 0 error；`cargo fmt --all -- --check` OK；`cargo test -p ti-console --test gate_tests` 6 过。

## 报告格式契约

报告正文**第一个 ``` 代码块**内必须含四行（缺任一行 ⇒ 判不可解析）：
```
总判定: PASS|FAIL
新发现: <整数>
误报: 0
残留: <整数>
```
- `新发现` = 本报告新发现数；`误报` = 0；`残留` = r2 未闭环数（你判 0 或如实）。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`。
- 必须引用**文件路径 + 行号**或**命令实跑输出**；可用只读命令：cat/grep/ls/cargo clippy -p ti-console/cargo test -p ti-console（只读，不改仓库）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
