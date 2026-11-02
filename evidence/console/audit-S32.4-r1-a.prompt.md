# 审计任务书 S32.4 · r1 · A 侧（完成度审计）

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行（`POST /api/gate`）。

## 被审 DoD（plan/ledger.yaml S32.4）

> 改动面=工具（受控执行）+ 证据；代码=crates/ti-console/src/runner/gate.rs；判据=实跑 fast 档 PASS（日志含每项 PASS/SKIP）+ 失败负例退出码非 0 且页面可见 + 非法 tier 400 + 锁占用时 409 + 无 token 写操作 403 + 历史可查 + 页面上明示「fast/touch 为开发档，不构成 gate 全绿证据」（22 号 §4.2）+ 门禁副作用路径写进执行日志 + gate 全绿 + 双审计通过 + evidence/S32.4.log

## 证据路径（全部只读）

- 主验收证据：`evidence/S32.4.log`
- 代码：`crates/ti-console/src/runner/gate.rs`、`crates/ti-console/src/runner/mod.rs`、`crates/ti-console/src/routes.rs`、`crates/ti-console/src/lib.rs`、`crates/ti-console/src/main.rs`
- 前端：`crates/ti-console/web/index.html`
- 单测：`crates/ti-console/tests/gate_tests.rs`
- 门禁执行日志：`evidence/console/logs/gate-*.log`（含 fast 档 `gate-1793577400.log`、full 档 `gate-1793578297.log`）
- 白名单：`crates/ti-console/console_commands.json`（`gate` 条目）
- 规格：`docs/process/22_dev_platform.md` §4.2/§7、`docs/plan-s32-platform.md` L31

## 审计面（A 侧：DoD 逐条 → 证据/代码/实跑输出）

逐条对照 DoD，每条给 **PASS/FAIL + 缺口 + 复现命令**。必须引用**文件路径 + 行号**或**命令实跑输出**，禁止「看起来没问题」。结论 ≠ 验收（主线程独立复核）。

## 报告格式契约（机器可解析，必须遵守）

报告正文**第一个 ``` 代码块**内必须含以下四行（缺任一行 ⇒ 判「不可解析」）：

```
总判定: PASS|FAIL
新发现: <整数>
误报: <整数>
残留: <整数>
```

- `新发现` = 本报告新发现的 issue 数（含 P2 口径精化建议）；`误报` = 主线程复核为误报数（你写 0）；`残留` = 前轮残留未闭环数（r1 写 0）。
- 值域：`总判定` 只允许 PASS/FAIL。
- 之后用 findings 表：`| 严重度 | 位置 | 问题 | 建议 |`（P0=阻断 / P1=应修 / P2=建议）。
