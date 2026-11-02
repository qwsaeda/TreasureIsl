# 审计任务书 S32.4 · r8 · B 侧（一致性确认轮）

> 你是一个**只读**对抗审计子代理（B：一致性与风险）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行。**本轮 = r7 收敛确认轮**（确认连续第 2 轮全 0：r7 为首次全 0 轮，r8 须再全 0 才置 verified；一致性全保持 + 无新违约 + r1~r7 报告 14 份落盘断言）。

## 审计面（B 侧：架构 / 规范 / 文档 / 门禁 / 回归 / 安全 / 依赖）

重点检查（严重度 + 位置文件:行 + 建议）：

1. **无回归**：`cargo fmt --all -- --check` OK；`cargo clippy --workspace --all-targets -- -D warnings` 0 error；`cargo test -p ti-console` 全绿；`run_gate` 仍 `Result<Value,(u16,String)>` 且 400/409/500 三分支齐备。
2. **最小环境集**：`MINIMAL_ENV_KEYS` 13 键（§10.3 清单）；`apply_minimal_env` env_clear + 白名单 + 缺键跳过；所有平台 spawn（gate/run_test/build_artifacts/apk_size）均调用。
3. **锁语义**：`check_lock_available` 四分支（无锁 Ok / 活跃 409 / 陈旧接管 / 损坏>5s 接管 + <5s 409）；`create_new` 原子占锁 + 409 不删日志 + spawn 失败释放锁 + reserving→真实 pid + 收割终态保留。
4. **一致性**：`gate.rs` `TIERS/SIDE_EFFECTS/tier_args` ↔ `console_commands.json` `enum_param.values/writes/arg_map/runner_fallback` 逐字段一致；`command`/`interp` ↔ `spawn_gate` 实际解释器；JSON id 14 唯一（无重复）。
5. **文档 + gitignore + 报告**：`.gitignore` 含 `/evidence/console/gate.lock`；`22_dev_platform.md` §6.1/§10.3/§10.4 + `14_tooling_catalog.md` 登记与实现一致；**r1~r7 审计报告 14 份落盘**（`audit-S32.4-r{1..7}-{a,b}.md`）且含四行机器字段。
6. **硬规则**：`ti-console/src` 0 命中 `TODO/FIXME/unimplemented!/todo!/panic!/.unwrap(/.expect(`（容错面除外）；无一次性脚本散落；无仓库外写入。
7. **回归**：`routes.rs` `/api/health`、`/api/overview`、`/api/tests`、`/api/gate`、`/api/build`、`/api/apk-size` 仍维持。

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
- 必须引用**文件路径 + 行号**或**命令实跑输出**，禁止「看起来没问题」。可用只读命令：cat/grep/ls/awk/git check-ignore/cargo clippy -p ti-console/cargo test -p ti-console（只读）。

把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。
