# S32.4 双审计 r3 · B 侧（一致性复核轮）报告

> task: S32.4 · round: r3 · side: B · new: 2
> 注：本报告由 r3 子代理输出归档（原始输出见会话记录；findings 描述与 r4 任务书 `audit-S32.4-r4-a/b.prompt.md` 一致）。

```
总判定: FAIL
新发现: 2
误报: 0
残留: 0
```

> 审计面 = r2 修复无回归 + 锁语义 + 一致性 + 文档/gitignore + 硬规则 + 回归。发现 1 P1 + 1 P2，故总判定 FAIL。

## r2 修复无回归复核（全部满足）

- `cargo fmt --all -- --check` → exit 0（0 差异）
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0（0 error；仅 1 条 `ti-engine` manifest 级警告，非 `-D warnings` 提升面、非 ti-console，不计违约）
- `cargo test -p ti-console` → **30/30 passed，0 failed**（unittests 4+4、config 4、data 4、gate 6、guard 4、snapshot 4、doctest 0）
- `run_gate` 返回 `Result<Value,(u16,String)>`；400（非法 tier）/409（活跃/竞态）/500（IO/spawn）三分支齐备

## 锁语义 + 一致性 + 硬规则 + 回归（全部满足）

- 日志创建移到锁后（`:149 create_new` → `:171 写占位锁` → `:176 File::create(&log_path)`）；409 分支不删日志；spawn 失败释放锁+删日志；占位锁 reserving + 真实 pid；收割线程终态。
- `TIERS=["fast","touch","full"]` ↔ `console_commands.json` `enum_param.values` 逐项一致；`tier_args` ↔ `arg_map`；`SIDE_EFFECTS` ↔ `writes`；`spawn_gate` 候选 `["pwsh","powershell"]` ↔ `runner:"pwsh"`+`runner_fallback:["powershell"]`。
- 响应 `command`/`interp` 用 `spawn_gate` 实际返回的 `interp`（非硬编码 pwsh，r2-b P2-1 已闭环）。
- `22_dev_platform.md` §6.1/§10.4 + `14_tooling_catalog.md` 登记与实现一致；r1/r2 报告 4 份落盘且四行机器字段齐备。
- `ti-console/src` 0 命中 `panic!/todo!/unimplemented!/TODO/FIXME/.unwrap(/.expect(`（容错面 ok()/unwrap_or/map_err 除外）；`scripts/tmp/` 仅 `README.md`；无仓库外写入。
- `routes.rs` `/api/health`/`/api/overview`/`/api/tests` 均维持。

## findings 表（r3-b 2 项）

| 严重度 | 位置 | 问题 | 建议 |
|---|---|---|---|
| P1 | `.gitignore:64-66` + `evidence/console/gate.lock` | **`gate.lock` 未入 `.gitignore`**：S32 段只忽略 `token`/`.lock`/`logs/`，漏 `gate.lock`；`git check-ignore evidence/console/gate.lock` → exit 1（未忽略）。运行时产物误入 git，且若误提交 `running=true` + 死 pid + 近期锁，检出后 `POST /api/gate` 首 30 分钟恒 409 | `.gitignore` 补 `/evidence/console/gate.lock` |
| P2 | `runner/gate.rs:122-159` | **锁文件损坏/空 → 永久 409，无接管**：`read_lock` 对非法/空 JSON 返回 `None` → 跳过陈旧判定 → `create_new` 失败 `AlreadyExists` → 409 且不清理；进程崩溃于写入窗口后恒 409 需手动删锁 | `AlreadyExists` 分支内 `read_lock` None（非法 JSON）→ 视同陈旧接管（限 1 次重试，防活锁误删）；或 `lock_is_stale` 对 None 走接管 |

> 修复状态（r4 复核）：2/2 已闭环。`残留 = 0`。
