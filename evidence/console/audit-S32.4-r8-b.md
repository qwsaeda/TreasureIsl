# S32.4 双审计 r8 · B 侧（一致性确认轮）报告

> task: S32.4 · round: r8 · side: B · new: 0
> 注：本报告由 r8 子代理输出归档（原始输出见会话记录）。
> 本轮 = r8 收敛确认轮（B 侧，只读）。r7 为首次全 0 轮（`r7a: 新发现 0 残留 0`、`r7b: 新发现 0 残留 0`），本轮复核对 7 个审计面逐一实跑/代码定位复核：无回归、最小环境集、锁语义、一致性、文档+gitignore+报告、硬规则、路由回归全部满足，无新违约，故 `残留 = 0`、`总判定 = PASS`。

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

## findings

| 严重度 | 位置 | 问题 | 建议 |
|---|---|---|---|
| — | 无 | 无新发现（七面复核全满足） | 保持现状，无需动作 |

## 面 1 · 无回归

| 判据 | 验证证据 | 判定 |
|---|---|---|
| `cargo fmt --all -- --check` OK | 实跑 **exit=0**（0 差异） | ✅ |
| `cargo clippy -p ti-console --all-targets -- -D warnings` 0 error | 实跑 **exit=0**，无 error（唯一 warning = ti-engine 既有 manifest 双 bin，非 ti-console、非 error） | ✅ |
| `cargo test -p ti-console` 全绿 | 实跑 **exit=0**：4+4+4+4+8+4+4 = **32 passed / 0 failed**（+0 doctest） | ✅ |
| `run_gate` 签名不变 | `crates/ti-console/src/runner/gate.rs:177` `pub fn run_gate(repo_root: &Path, tier: &str) -> Result<Value, (u16, String)>` | ✅ |
| 400/409/500 三分支齐备 | `crates/ti-console/src/routes.rs:310-314` `match code { 400 => "400 Bad Request", 409 => "409 Conflict", _ => "500 Internal Server Error" }` | ✅ |

## 面 2 · 最小环境集

- `crates/ti-console/src/runner/mod.rs:16-30` `MINIMAL_ENV_KEYS` = **13 键**（PATH/HOME/USERPROFILE/CARGO_HOME/RUSTUP_HOME/SYSTEMROOT/TEMP/TMP/APPDATA/LOCALAPPDATA/XDG_CACHE_HOME/TI_CHROMIUM_PATH/TI_CONSOLE_TOKEN），与 `docs/process/22_dev_platform.md` §10.3（:356-358）逐键一致。
- `runner/mod.rs:34-41` `apply_minimal_env` = `env_clear()` + 白名单逐键透传 + 缺键跳过。
- 所有平台 spawn 均调用：gate（`gate.rs:115`）/ run_test（`data.rs:279`）/ build_artifacts run（`data.rs:319`）/ list（`data.rs:335`）/ apk_size（`data.rs:405`）/ refset（`data.rs:428`）= 6 个 spawn 点全覆盖。

## 面 3 · 锁语义

- `check_lock_available` 四分支（`gate.rs:138-168`）：无锁 Ok（:139-141）/ 活跃 409（:142-146）/ 陈旧接管 remove（:147-151）/ 损坏 >5s 接管（:154-156）+ <5s 409（:158-164）。
- `create_new` 原子占锁（`gate.rs:197`，`AlreadyExists` → 409，:200-206）；日志文件**建于锁成功之后**（:224），409 败者不触碰日志（r2-a 修复保持）。
- spawn 失败释放锁 + 删日志（`gate.rs:236-241`）。
- reserving 占位（pid=0，`gate.rs:210-218`）→ 真实 pid 更新（`gate.rs:246-257`）。
- 收割线程 `child.wait()` 后置 `running=false` + `exit_code` + `ended_ts` 保留终态（`gate.rs:262-272`）。

## 面 4 · 一致性

- `gate.rs:20` `TIERS=["fast","touch","full"]` ↔ `console_commands.json` `gate.enum_param.values` 一致。
- `gate.rs:24-25` `SIDE_EFFECTS` ↔ `gate.writes` 一致（web_playtest.log/web_playtest.png/font_render.txt）。
- `gate.rs:38-45` `tier_args`（fast→`[-Fast]`/touch→`[-Touch]`/full→`[]`）↔ `gate.enum_param.arg_map` 一致。
- `spawn_gate`（`gate.rs:108` 循环 `["pwsh","powershell"]`）↔ `runner=pwsh` + `runner_fallback=["powershell"]`；返回 `command`/`interp` 元数据如实（`gate.rs:278-279`）。
- JSON id 唯一：**count=14，`dup_ids=[]`**（无重复）。

## 面 5 · 文档 + gitignore + 报告

- `.gitignore:66` `/evidence/console/gate.lock`；`git check-ignore -v` → `.gitignore:66` 命中。
- `22_dev_platform.md` §6.1（:171 页面表）/§10.3（:353-363）/§10.4（:365-376 锁语义含损坏接管 `CORRUPT_LOCK_MIN_AGE_SECS=5`）与实现一致。
- `14_tooling_catalog.md:262` `ti-console` 登记（`GET/POST /api/gate` 鉴权/分档/锁/后台 spawn + 副作用只记录）与实现一致。
- **r1~r7 共 14 份报告均在盘**：`audit-S32.4-r{1..7}-{a,b}.md`，大小 1567~5207 字节，每份首代码块四行机器字段齐全（`first_block_fields=4` 全 14 份）。r7 为首次全 0 轮（r6 仍有 2 项新发现），本轮为连续第 2 轮全 0。

## 面 6 · 硬规则

- `ti-console/src` 0 命中 `TODO|FIXME|unimplemented!|todo!|panic!|.unwrap(|.expect(`（grep 全量 0）。
- 无一次性脚本散落：`scripts/tmp/` 仅 `README.md`；仓库根无散落 `.py`/`.ps1`。
- 无仓库外写入：本子代理全程只读；`evidence/console/gate.lock` 当前**不存在**（无残留锁/无正在运行的 gate）。

## 面 7 · 路由回归

`routes.rs` 路由匹配块六端点均维持：`/api/health`（:147）/`/api/overview`（:164）/`/api/tests`（:174）/`/api/gate`（:184）/`/api/build`（:188）/`/api/apk-size`（:192）。

---

**结论**：S32.4 双审计连续第 2 轮全 0（r7 → r8），七面一致性全保持、无新违约、r1~r7 报告 14 份落盘断言通过 ⇒ 本侧判定 **PASS**。
