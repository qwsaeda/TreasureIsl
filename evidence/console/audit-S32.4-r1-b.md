# S32.4 双审计 r1 · B 侧（一致性与风险审计）报告

> task: S32.4 · round: r1 · side: B · new: 7
> 注：本报告由 r1 子代理输出归档（原始输出见会话记录；findings 描述与 r2 任务书 `audit-S32.4-r2-b.prompt.md` 一致）。

```
总判定: FAIL
新发现: 7
误报: 0
残留: 0
```

> 审计面 = 架构 / 规范 / 文档 / 门禁 / 回归 / 安全 / 依赖。发现 2 P1（应修）+ 5 P2（建议），故总判定 FAIL。

## findings 表（r1-b 7 项）

| 严重度 | 位置 | 问题 | 建议 |
|---|---|---|---|
| P1 | `runner/gate.rs` spawn（原硬编码 `pwsh`） | pwsh 硬编码无回退；Linux 无 `pwsh` 时门禁无法启动 | 候选解释器 `pwsh`→`powershell` 顺序回退 + `console_commands.json` `gate.runner_fallback` |
| P1 | `main.rs` `write_lock` | 平台进程锁 `.lock` 无陈旧接管（崩溃/ kill -9 后残留阻启） | 锁 JSON（pid+started_ts+action）+ pid 存活 + 30 分钟接管（`LOCK_STALE_SECS=1800`） |
| P2 | `routes.rs` `handle_gate` | body 上限 256KiB（`Content-Length` 读面）vs 22 号 §10.1 要求 64KiB | `body.len() > 65_536` → 413 |
| P2 | `routes.rs` `handle_gate` ct_ok | Content-Type `contains("application/json")` 过宽（`text/application/json-foo` 误收） | `split(';').next().trim()=="application/json"` 精确匹配 |
| P2 | `docs/process/22_dev_platform.md` §10.4 | `gate.lock`（门禁执行锁）未文档化，只有平台进程锁 `.lock` 口径 | 补「门禁执行锁（S32.4 起，两锁分立）：`evidence/console/gate.lock`」段 |
| P2 | `22_dev_platform.md` §6.1 + `14_tooling_catalog.md` | `GET/POST /api/gate` 未在 API 表 / 工具台账登记 | §6.1 门禁行 + 14 号 ti-console 行点名 `/api/gate` |
| P2 | `main.rs` 非回环警告 | `auth_ok` 弱鉴权（env 未设=非空即可）与警告文案「只读面可用…403」不符 | 警告文案改「写操作面（POST）要求 X-TI-Token 在场（env 未设=非空即可；env 设了=精确匹配）」 |

> 修复状态（r2 复核）：7/7 已闭环。`残留 = 0`。
