# 审计任务书 S32.4 · r5 · B 侧（一致性复核轮）

> 你是一个**只读**对抗审计子代理（B：一致性与风险）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r4 收敛验证轮**（确认 r4 修复未引入新违约 + 一致性全保持）。

## 审计面（B 侧：架构 / 规范 / 文档 / 门禁 / 回归 / 安全 / 依赖）

重点检查（严重度 + 位置文件:行 + 建议）：

1. **r4 修复无回归**：`apply_minimal_env`（env_clear + 白名单）应用于 3 处 spawn 后——`cargo fmt --all -- --check` OK；`cargo clippy --workspace --all-targets -- -D warnings` 0 error；`cargo test -p ti-console` 全绿；gate 实跑仍 PASS（`gate-1793583779.log` 17/0/16，env_clear 后子进程面工作）。
2. **最小环境集**：`MINIMAL_ENV_KEYS` 13 键（§10.3 清单）；`apply_minimal_env` `env_clear()` + 白名单逐键透传 + 缺键跳过；`spawn_gate`/`run_test`/`build_artifacts` 3 处均调用。
3. **锁语义**：`check_lock_available` 四分支（无锁 Ok / 活跃 409 / 陈旧接管 / 损坏>5s 接管 + <5s 409）；`create_new` 原子占锁 + 409 不删日志 + spawn 失败释放锁 + reserving→真实 pid + 收割终态保留。
4. **一致性**：`gate.rs` `TIERS/SIDE_EFFECTS/tier_args` ↔ `console_commands.json` `enum_param.values/writes/arg_map/runner_fallback` 逐字段一致；`command`/`interp` ↔ `spawn_gate` 实际解释器。
5. **文档 + gitignore + 报告**：`.gitignore` 含 `/evidence/console/gate.lock`；`22_dev_platform.md` §6.1/§10.3/§10.4 + `14_tooling_catalog.md` 登记与实现一致；r1~r4 审计报告 6 份落盘且含四行机器字段。
6. **硬规则**：`ti-console/src` 0 命中 `TODO/FIXME/unimplemented!/todo!/panic!/.unwrap(/.expect(`（容错面除外）；无一次性脚本散落；无仓库外写入。
7. **回归**：`routes.rs` `/api/health`、`/api/overview`、`/api/tests` 仍维持。

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
- 必须引用**文件路径 + 行号**或**命令实跑输出**；可用只读命令：cat/grep/ls/git check-ignore/cargo clippy -p ti-console/cargo test -p ti-console（只读）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
