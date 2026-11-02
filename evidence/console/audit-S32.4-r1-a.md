# S32.4 双审计 r1 · A 侧（完成度审计）报告

> task: S32.4 · round: r1 · side: A · new: 4
> 注：本报告由 r1 子代理输出归档（原始输出见会话记录；findings 描述与 r2 任务书 `audit-S32.4-r2-a.prompt.md` 一致）。

```
总判定: PASS
新发现: 4
误报: 0
残留: 0
```

> 审计面 = S32.4 DoD（`plan/ledger.yaml`）逐条对照。DoD 核心判据全部满足（fast PASS / 403 / 400 / 409 / 历史 / side_effects / full 全绿），但发现 4 项 P2 口径精化建议（不阻断 DoD，故总判定 PASS）。

## findings 表（r1-a 4 项 P2）

| 严重度 | 位置 | 问题 | 建议 |
|---|---|---|---|
| P2 | `crates/ti-console/src/runner/gate.rs`（原实现 spawn 后写锁） | 双启动竞态：先 spawn 后写锁，并发两 POST 存在竞态窗口 | `create_new` 原子占锁 + 占位锁（reserving）再 spawn，spawn 后更新真实 pid |
| P2 | `runner/gate.rs` 启动响应 | 副作用路径仅瞬时响应（`side_effects` 只在 POST 响应，未持久化） | `side_effects` 写进 gate.lock（`GET /api/gate` `last.side_effects` 非空） |
| P2 | `runner/gate.rs` `gate_status` `?log=` 分支 | `?log=` 未拒 `/`（仅拒 `..`），路径穿越面不全 | 校验加 `!name.contains('/') && !name.contains('\\')` |
| P2 | `plan/ledger.yaml` S32.4 title | 台账标题写 `evidence/console/.lock`（平台进程锁），与门禁执行锁 `gate.lock` 混淆 | title 改 `gate.lock` 并明示「与平台进程锁 .lock 分立」 |

> 修复状态（r2 复核）：4/4 已闭环。`残留 = 0`。
