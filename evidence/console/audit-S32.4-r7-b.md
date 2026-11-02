# S32.4 双审计 r7 · B 侧（一致性复核轮）报告

> task: S32.4 · round: r7 · side: B · new: 0
> 注：本报告由 r7 子代理输出归档（原始输出见会话记录）。
> 审计面 = r6 修复无回归 + 最小环境集 + 锁语义 + 一致性 + 文档/gitignore/报告 + 硬规则 + 回归（只读，0 改文件/改台账/跑构建；仅允许只读命令）。

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

r6 的 2 项 P2 findings 均闭环（`残留 = 0`）：① r5 报告未落盘 → `evidence/console/audit-S32.4-r5-{a,b}.md` 现均在盘且四行机器字段齐全；② `data.rs:402/:426` apk_size 两处 `Command::new` 未套最小环境集 → 现两处均已补 `apply_minimal_env`（`data.rs:405`、`:428`），`/api/apk-size` 实跑验证无回归。七面逐面复核无新违约，故总判定 PASS。

## r6 修复无回归（面 1，全部满足）

| 判据 | 验证证据（命令实跑 / 文件:行） | 判定 |
|---|---|---|
| `cargo fmt --all -- --check` OK | 实跑 exit=0（0 差异） | ✅ |
| `cargo clippy -p ti-console --all-targets -- -D warnings` 0 error | 实跑 exit=0，`Finished dev profile` 无 error（唯一 warning 为 ti-engine 既有 manifest 双 bin 警告，非 ti-console、非 error） | ✅ |
| `cargo test -p ti-console` 全绿 | 实跑 exit=0：4+4+4+4+8+4+4 = **32 passed / 0 failed** | ✅ |
| `/api/apk-size` 实跑 200（verdict=PASS 或 built:false） | `curl http://127.0.0.1:8787/api/apk-size` → **HTTP=200**；`verdict=PASS`，`built=true`，`so_ratio=0.3836`，`refset` 非 null（`{audio,data,fonts,legacy,ui}` 5 键）——两处 spawn（`--apk`/`--refset`）均在最小环境集下成功 | ✅ |
| `data.rs` 无 `Command::new` 遗漏 | grep `Command::new` data.rs：run_test(:276)/build_artifacts run(:316)/list(:331)/apk_size(:402)/:429（refset）——全部套 `apply_minimal_env` 或属既有面；无裸 spawn 遗漏 | ✅ |

## 最小环境集（面 2，全部满足）

- `runner/mod.rs:16-30` `MINIMAL_ENV_KEYS` = **13 键**（PATH/HOME/USERPROFILE/CARGO_HOME/RUSTUP_HOME/SYSTEMROOT/TEMP/TMP/APPDATA/LOCALAPPDATA/XDG_CACHE_HOME/TI_CHROMIUM_PATH/TI_CONSOLE_TOKEN），与 `22_dev_platform.md` §10.3 逐键一致。
- `runner/mod.rs:34-41` `apply_minimal_env` = `env_clear()` + 白名单逐键透传 + 缺键跳过。
- 平台 spawn 全覆盖：gate（`gate.rs:115`）/run_test（`data.rs:279`）/build_artifacts（`:319`、`:335`）/apk_size（`:405`、`:428`）= 6 个 spawn 点全套。
- `repo/import.rs:158`（map_pack.py）与 `repo/snapshot.rs:186`（git status）同模式未套最小环境集——**分属 S32.14/S32.2 面，不计 S32.4 findings**，待各自任务复核（注明，不阻断）。

## 锁语义（面 3，全部满足）

- `check_lock_available`（`gate.rs:138-168`）四分支：无锁→Ok / 活跃→409 / 陈旧→接管 / 损坏→>5s 接管 + <5s 409；`CORRUPT_LOCK_MIN_AGE_SECS=5`（:67）。
- `create_new` 原子占锁（:197）；`AlreadyExists`→409 不删日志（:200-206）；spawn 失败释放锁+删日志（:228-242）；占位锁 reserving→真实 pid（:210-218→:246-257）；收割线程终态（:262-272）。

## 一致性（面 4，全部满足）

- `gate.rs` `TIERS=["fast","touch","full"]`/`SIDE_EFFECTS`/`tier_args` ↔ `console_commands.json` `enum_param.values`/`writes`/`arg_map` 逐字段一致。
- `runner:"pwsh"`+`runner_fallback:["powershell"]` ↔ `spawn_gate` 候选 `["pwsh","powershell"]` 一致。
- 响应 `command`/`interp`（:278-279）用 `spawn_gate` 实际返回的 `interp`（:108/:124），非硬编码 pwsh。

## 文档 + gitignore + 报告（面 5，全部满足）

- `.gitignore:66` 含 `/evidence/console/gate.lock`；`git check-ignore evidence/console/gate.lock` → exit=0（已忽略）。
- `22_dev_platform.md` §6.1（:178）/§10.3（:356-360）/§10.4（:370）+ `14_tooling_catalog.md`（:262）登记与实现一致。
- r1~r6 审计报告 **12 份落盘**（`audit-S32.4-r{1..6}-{a,b}.md`）且各自第一个代码块四行机器字段齐全。

## 硬规则（面 6，全部满足）

- `ti-console/src` **0 命中** `TODO/FIXME/unimplemented!/todo!/panic!/.unwrap(/.expect(`（grep exit 1；容错面 ok()/unwrap_or/map_err 除外）。
- `scripts/tmp/` 仅 README.md；仓库根无散落一次性 `.py`/`.ps1`；无仓库外写入。

## 回归（面 7，全部满足）

- `routes.rs` `/api/health`（:147）、`/api/overview`（:164）、`/api/tests`（:174）、`/api/gate`（:127+`:184`）、`/api/build`（:188）、`/api/apk-size` 均维持。

## 结论

- r6 的 2 项 P2 findings 均闭环（`残留 = 0`）；七面逐面复核无新违约。
- 本报告**无新发现**、无误报、无残留。总判定 **PASS**（首次全 0 轮）。
