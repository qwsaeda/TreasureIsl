# 审计任务书 S32.4 · r1 · B 侧（一致性与风险审计）

> 你是一个**只读**对抗审计子代理（B：一致性与风险）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S32.4 门禁执行（`POST /api/gate`）。

## 审计面（B 侧：架构 / 规范 / 文档 / 门禁 / 回归 / 安全 / 依赖）

重点检查（严重度 + 位置文件:行 + 建议）：

1. **架构/边界**：`runner/` 是否与 22 号 §3 边界一致（平台 0 游戏依赖；runner 白名单命令执行、禁自由字符串透传）；`routes.rs` 是否维持既有只读数据源面不变。
2. **安全**：`POST /api/gate` 鉴权（X-TI-Token 403）、body 上限（22 号 §10.3 64 KiB）、Content-Type 白名单、单实例锁（`gate.lock`）并发、日志/锁文件名防穿越、`?log=` 参数白名单形状。
3. **门禁**：新代码是否引入 `cargo fmt` / `cargo clippy -D warnings` 违约（gate 第 1/3 项）；`console_commands.json` `gate` 条目与 `runner::gate` 的 `TIERS`/`SIDE_EFFECTS`/`arg_map` 是否一致（22 号 §4.2 两份必须一致）；`cargo test -p ti-console` 是否绿。
4. **文档**：新端点 `GET/POST /api/gate` 是否在 22 号 §5 API 表 / 14 号工具台账登记；`docs/process/22_dev_platform.md` 是否与实现一致。
5. **回归**：既有 `/api/*` 只读端点是否仍工作（`/api/health`、`/api/overview`、`/api/tests` 等）。
6. **硬规则**：无占位（TODO/unimplemented!/unwrap 在 src 库代码外——注意 `runner/gate.rs` 的 `unwrap_or`/`ok()?` 是容错面非 panic）；无一次性脚本散落；无仓库外写入。

## 报告格式契约（机器可解析，必须遵守）

报告正文**第一个 ``` 代码块**内必须含以下四行（缺任一行 ⇒ 判「不可解析」）：

```
总判定: PASS|FAIL
新发现: <整数>
误报: <整数>
残留: <整数>
```

- `新发现` = 本报告新发现的 issue 数（含 P2 口径精化建议）；`误报` = 主线程复核为误报数（你写 0）；`残留` = 前轮残留未闭环数（r1 写 0）。
- 之后用 findings 表：`| 严重度 | 位置 | 问题 | 建议 |`（P0=阻断 / P1=应修 / P2=建议）。
