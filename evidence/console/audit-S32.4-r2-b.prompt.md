# 审计任务书 S32.4 · r2 · B 侧（一致性复核）

> 你是一个**只读**对抗审计子代理（B：一致性与风险）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r1 收敛验证轮**（复核 r1-b 的 7 项（2 P1 + 5 P2）是否已修复 + 新代码不再引入违约）。

## r1-b 需复核的 7 项（r1 报告 `evidence/console/audit-S32.4-r1-b.md`）

1. **P1：pwsh 硬编码无回退** → 修复应 = `runner/gate.rs` `spawn_gate` 候选 `pwsh`→`powershell`；`console_commands.json` `gate.runner_fallback=["powershell"]`。
2. **P1：main.rs `.lock` 无陈旧接管** → 修复应 = `write_lock` JSON（pid+started_ts+action）+ pid 存活 + 30 分钟接管（`LOCK_STALE_SECS=1800`）。
3. **P2：body 上限 64KiB vs 256KiB** → 修复应 = `handle_gate` `body.len()>65536` → 413。
4. **P2：Content-Type `contains`** → 修复应 = `split(';').next().trim()=="application/json"`。
5. **P2：gate.lock 未文档化** → 修复应 = 22 号 §10.4 补「门禁执行锁 gate.lock 两锁分立」段。
6. **P2：GET/POST /api/gate 未登记** → 修复应 = 22 号 §6.1 门禁行 + 14 号 ti-console 行点名 `/api/gate`。
7. **P2：auth_ok 弱鉴权 + main.rs 警告不符** → 修复应 = main.rs 警告文案改「写面要求 X-TI-Token 在场（env 未设=非空即可）」。

## 复核要点

- 逐条看 r1 的 7 项是否已修复（引文件:行 或 实跑输出）；clippy/fmt/test 是否仍绿；无新引入违约。
- 报告正文**第一个 ``` 代码块**内必须含四行：`总判定: PASS|FAIL` / `新发现: <int>` / `误报: 0` / `残留: <int>`。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`。

关键证据（只读）：`crates/ti-console/src/runner/gate.rs`、`crates/ti-console/src/routes.rs`、`crates/ti-console/src/main.rs`、`crates/ti-console/console_commands.json`、`docs/process/22_dev_platform.md`、`docs/process/14_tooling_catalog.md`、`evidence/console/audit-S32.4-r1-b.md`。
