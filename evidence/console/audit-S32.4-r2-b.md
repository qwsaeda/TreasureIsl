# S32.4 双审计 r2 · B 侧（一致性复核）报告

> task: S32.4 · round: r2 · side: B · new: 2

```
总判定: PASS
新发现: 2
误报: 0
残留: 0
```

## r1 的 7 项修复逐条验证（全部已修复）

| # | 严重度 | r1 问题 | 修复证据 | 判定 |
|---|---|---|---|---|
| 1 | P1 | pwsh 硬编码无回退 | `gate.rs:88` `for interp in ["pwsh","powershell"]` 顺序回退；`console_commands.json:35-37` `runner_fallback:["powershell"]` | ✅ 已修复 |
| 2 | P1 | main.rs `.lock` 无陈旧接管 | `main.rs:88-92` `write_lock` 写 JSON `{pid,started_ts,action}`；`:33-38` `process_alive`（Linux `/proc`）；`:25` `LOCK_STALE_SECS=1800`；`:66-77` pid 不存在且超 30 分钟 ⇒ 接管 | ✅ 已修复 |
| 3 | P2 | body 上限 64KiB vs 256KiB | `routes.rs:282` `body.len()>65_536` → `413 Payload Too Large` | ✅ 已修复 |
| 4 | P2 | Content-Type `contains` 过宽 | `routes.rs:266-269` `split(';').next().trim().to_ascii_lowercase()=="application/json"`（精确 + charset） | ✅ 已修复 |
| 5 | P2 | gate.lock 未文档化 | `22_dev_platform.md:370` §10.4「门禁执行锁（S32.4 起，两锁分立）」段 | ✅ 已修复 |
| 6 | P2 | GET/POST /api/gate 未登记 | `22_dev_platform.md:178` §6.1 门禁行 + `14_tooling_catalog.md:262` ti-console 行 | ✅ 已修复 |
| 7 | P2 | auth_ok 弱鉴权 + 警告不符 | `main.rs:136-141` 警告文案「写操作面（POST）要求 X-TI-Token 在场（env 未设=非空即可；env 设了=精确匹配）」与 `routes.rs:324-330` `auth_ok` 语义精确一致 | ✅ 已修复 |

## 新代码违约复核（全部绿）

- `cargo fmt --all -- --check` → exit 0（无 diff）
- `cargo clippy -p ti-console --all-targets -- -D warnings` → exit 0（0 error）
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0（0 error）
- `cargo test -p ti-console --test gate_tests` → **6 passed**
- `cargo test -p ti-console` 全量 → lib 4 + main 4 + config 4 + data 4 + gate 6 + guard 4 + snapshot 4 + doc 0 = 全绿
- 硬规则：`ti-console/src` 0 命中 `TODO/FIXME/unimplemented!/todo!/panic!/.unwrap(/.expect(`（`unwrap_or/map_err/ok()?` 容错面符合 gate.rs 口径）
- 回归端点在位：`routes.rs:147` `/api/health`、`:164` `/api/overview`、`:174` `/api/tests`
- 一致性：`gate.rs` `TIERS/SIDE_EFFECTS/tier_args` ↔ `console_commands.json` `enum_param.values/writes/arg_map` 逐字段一致；`scripts/tmp/` 仅 README.md（无散落一次性脚本）；锁文件 `.lock`/`gate.lock` 状态合法

## findings 表（r2-b 2 项 P2）

| 严重度 | 位置 | 问题 | 建议 |
|---|---|---|---|
| P2 | `crates/ti-console/src/runner/gate.rs:225` | `run_gate` 响应 `command` 字段**硬编码 `"pwsh -File scripts/gate.ps1 ..."`**，未反映 `spawn_gate` 实际回退解释器（pwsh 缺失走 powershell 时元数据失真） | 由 `spawn_gate` 返回实际解释器名（或响应加 `interp` 字段）写入 `command` |
| P2 | `evidence/console/audit-S32.4-r1-b.md`（及 r1-a.md）**未落盘**——r2 任务书引用的 r1 报告文件不存在，仅 `.prompt.md` 任务书在盘；收敛可追溯性缺一份已保存审计报告（7 项内容在任务书内可验证，但留痕不全） | r1 报告未落盘 | 主线程持久化 r1-a/r1-b 报告到 `evidence/console/`，与任务书引用路径一致 |

## 判定依据小结

- r1 的 7 项：**全部已修复**，`残留 = 0`。
- 新代码违约：fmt/clippy/test 全绿、硬规则 0 命中、回归端点维持、`console_commands.json` 一致性成立。
- 新发现 2 项（均 P2，非阻断）：`command` 硬编码 pwsh（元数据失真）+ r1 报告未落盘（可追溯性）。
- `误报 = 0`。

**结论：S32.4 r1 收敛验证通过（PASS），7/7 闭环，新增 2 项 P2 建议，无新代码违约。**
