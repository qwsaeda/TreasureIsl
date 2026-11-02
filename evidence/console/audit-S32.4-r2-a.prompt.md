# 审计任务书 S32.4 · r2 · A 侧（完成度复核）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r1 收敛验证轮**（复核 r1-a 的 4 项 P2 是否已修复 + DoD 仍满足）。

## r1-a 需复核的 4 项 P2（r1 报告 `evidence/console/audit-S32.4-r1-a.md`）

1. **双启动竞态**（`runner/gate.rs` spawn 后写锁）→ 修复应 = `create_new` 原子占锁 + 占位锁（reserving）再 spawn。
2. **副作用路径仅瞬时响应** → 修复应 = `side_effects` 写进 gate.lock（`get /api/gate` `last.side_effects` 非空）。
3. **`?log=` 未拒 `/`** → 修复应 = 校验加 `!name.contains('/') && !name.contains('\\')`。
4. **台账标题 `.lock` vs `gate.lock`** → 修复应 = `plan/ledger.yaml` S32.4 title 写 `gate.lock`（与实现一致）。

## 复核要点

- 逐条看 r1 的 4 项 P2 是否已修复（引文件:行 或 实跑输出）；DoD 核心（fast PASS / 403/400/409 / 历史 / side_effects / gate 全绿）是否仍满足。
- 报告正文**第一个 ``` 代码块**内必须含四行：`总判定: PASS|FAIL` / `新发现: <int>` / `误报: 0` / `残留: <int>`（残留 = r1 未闭环数，你判 0 或如实）。
- 之后 findings 表 `| 严重度 | 位置 | 问题 | 建议 |`。

关键证据（只读）：`evidence/console/logs/gate-1793580084.log`（fast PASS 17/0/16）、`crates/ti-console/src/runner/gate.rs`、`crates/ti-console/src/routes.rs`、`crates/ti-console/src/main.rs`、`plan/ledger.yaml`、`evidence/console/audit-S32.4-r1-a.md`。
