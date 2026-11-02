# S32.4 双审计 r5 · B 侧（一致性复核轮）报告

> task: S32.4 · round: r5 · side: B · new: 2
> 注：本报告由 r5 子代理输出归档（原始输出见会话记录；findings 描述与 r6 任务书 `audit-S32.4-r6-a/b.prompt.md` 一致）。

```
总判定: PASS
新发现: 2
误报: 0
残留: 0
```

> 审计面 = r4 修复无回归 + 最小环境集 + 锁语义 + 一致性 + 文档/gitignore/报告 + 硬规则 + 回归。r4 四项 findings（F1~F4）全部闭环（`残留 = 0`），r4 修复未引入新违约；2 项新发现均 P2（非阻断），故总判定 PASS。

## 1. r4 修复无回归（全部实测通过）

| 判据 | 实跑结果 |
|---|---|
| `cargo fmt --all -- --check` | exit **0**（0 差异） |
| `cargo clippy -p ti-console --all-targets -- -D warnings` | exit **0**（0 error；仅 1 条 `ti-engine` manifest 级警告，非 ti-console、非 `-D warnings` 提升面，不计违约） |
| `cargo test -p ti-console` | **32/32 passed，0 failed**（config 4 / data 4 / gate 8 / guard 4 / snapshot 4 / doctest 0；`gate_tests` 8 项全过） |
| gate 实跑（env_clear 后子进程面） | `evidence/console/logs/gate-1793583779.log`：**17 PASS / 0 FAIL / 16 SKIP**（33 项），末行 `=== gate: 0 failure(s) ===`，锁 `exit_code:0` |

## 2. 最小环境集（全部满足）

- `runner/mod.rs:16-30` `MINIMAL_ENV_KEYS` = **13 键**，与 `22_dev_platform.md` §10.3（`:356-360`）逐键一致。
- `runner/mod.rs:34-41` `apply_minimal_env` = `env_clear()` + 白名单逐键透传 + 键不在父环境跳过。
- 3 处均调用（4 个 call site 覆盖 3 函数）：`runner/gate.rs:115`（`spawn_gate`）、`repo/data.rs:279`（`run_test`）、`repo/data.rs:319`（`build_artifacts ?run=`）+ `:335`（`build_artifacts list`）。

## 3. 锁语义（全部满足）

- `runner/gate.rs:138-168` `check_lock_available` 四分支齐备：无锁 `Ok` / 活跃 `409` / 陈旧接管 `Ok`+删锁 / 损坏超 `CORRUPT_LOCK_MIN_AGE_SECS=5` 秒接管而 <5 秒 `409` 不接管。
- `create_new` 原子占锁（`:197`）；`AlreadyExists` → 409 **不删日志**（`:200-206`）；spawn 失败释放锁+删日志（`:237-241`）；占位锁 `reserving:true` → 真实 pid（`:209-257`）；收割线程终态保留（`:262-272`）。
- 损坏锁分支有单测：`tests/gate_tests.rs:107-127`（2 个 `#[test]`，均过）。

## 4. 一致性（全部满足）

- `gate.rs` `TIERS`/`SIDE_EFFECTS`/`tier_args` ↔ `console_commands.json` `enum_param.values`/`writes`/`arg_map` 逐项一致；`runner:"pwsh"`+`runner_fallback:["powershell"]` ↔ `spawn_gate` 候选一致。
- 响应 `command`/`interp`（`:278-279`）用 `spawn_gate` 实际返回的 `interp`（`:124`），非硬编码 pwsh。

## 5. 文档 + gitignore + 报告（1 项新发现，见 F1）

- `.gitignore:66` 含 `/evidence/console/gate.lock`；`git check-ignore evidence/console/gate.lock` → exit 0（已忽略）。✓
- `22_dev_platform.md` §6.1（`:178`）/§10.3（`:356-360`）/§10.4（`:370`）+ `14_tooling_catalog.md`（`:262`）登记与实现一致。✓
- r1~r3 报告 **6 份落盘**且首个代码块四行机器字段齐备。**但 r4 报告缺失 → F1**。

## 6. 硬规则（全部满足）

- `ti-console/src` **0 命中** `TODO/FIXME/unimplemented!/todo!/panic!/.unwrap(/.expect(`（grep exit 1；容错面除外）。
- `scripts/tmp/` 仅 `README.md`；仓库根无散落 `.py`/`.ps1`；无仓库外写入。

## 7. 回归（全部满足）

- `routes.rs` `/api/health`（`:147`）、`/api/overview`（`:164`）、`/api/tests`（`:174`）、`/api/gate`（`:184`）、`/api/build`（`:188`）均维持。

## findings 表（r5-b 2 项 P2）

| 严重度 | 位置 | 问题 | 建议 |
|---|---|---|---|
| P2 | `evidence/console/audit-S32.4-r4-a.md` / `audit-S32.4-r4-b.md`（均缺失） | **r4 双审计报告未落盘**：仅 `r4-a/b.prompt.md` 存在，无 `.md` 报告；r5-a prompt 引用 `audit-S32.4-r4-b.md` 形成悬空引用 | 补落盘 `audit-S32.4-r4-a/b.md`（含首代码块四行机器字段），并核对 r4 四项 findings 已随 r5 闭环 |
| P2 | `crates/ti-console/console_commands.json:73` 与 `:181`（同 id） | **机器真源重复 id `build_check`**：14 个 id / 13 唯一——`build_check` 既映射 `pwsh scripts/build_wasm.ps1 -Check`（`:73-82`）又映射 `python3 scripts/build_artifacts.py check`（`:181-190`），语义不同、共用一 id | 去重：改名其一（如 `build_wasm_check` / `build_artifacts_check`）；或 S32.6 对账 id 集合改容忍重复 |

> 修复状态（r6 复核）：2/2 已闭环。`残留 = 0`。
