# 对抗审计报告 S32.4 · r2 · A 侧（完成度复核）

```
总判定: PASS
新发现: 1
误报: 0
残留: 0
```

## r1 的 4 项 P2 复核结论（全部已闭环）

**P2-1 双启动竞态 — 已修复 ✅**
`crates/ti-console/src/runner/gate.rs:149` `std::fs::OpenOptions::new().write(true).create_new(true).open(&lock_p)`（O_EXCL 原子占锁）；`:165-174` 占位锁（`pid=0` + `reserving=true` + `side_effects`）写于 **spawn 之前**；`:182` spawn；`:193-204` spawn 后更新真实 pid（`reserving=false`）；`:152-159` `AlreadyExists` → 409 竞态分支存在。`is_busy` 已删（全库 grep 0 命中），`lock_is_stale`（`:65-78`）取代。判据满足。

**P2-2 副作用路径仅瞬时响应 — 已修复 ✅**
`gate.rs:173`（占位锁）、`:200`（真实锁）均含 `"side_effects": SIDE_EFFECTS`；收割线程 `:214-218` 以 `lock_meta`（`lock.clone()`，含 side_effects）写终态 meta。实读 `evidence/console/gate.lock`：
`{"ended_ts":1793580324,"exit_code":0,"log":"evidence/console/logs/gate-1793580084.log","pid":82472,"reserving":false,"running":false,"side_effects":["evidence/web_playtest.log","evidence/web_playtest.png","evidence/font_render.txt"],"started_ts":1793580084,"tier":"fast"}` → `last.side_effects` 非空，判据满足。`SIDE_EFFECTS` 常量（`:24-25`）= 任务书给定三路径。

**P2-3 `?log=` 未拒 `/` — 已修复 ✅**
`gate.rs:259-267` tail 分支：`name.contains("..") || name.contains('/') || name.contains('\\')` → `return None`。`/` 与 `\` 均拒绝（`:263-264`）。单测 `tests/gate_tests.rs:92-95` 覆盖 `..` 负例。

**P2-4 台账标题 `.lock` vs `gate.lock` — 已修复 ✅**
`plan/ledger.yaml:312` S32.4 title 含 `单实例锁（evidence/console/gate.lock，与平台进程锁 .lock 分立）` — 明确写 `evidence/console/gate.lock`，且「与平台进程锁 .lock 分立」为**消歧**表述（明确两者分立），非混淆表述；`.lock`/`gate.lock` 歧义已消除。

## DoD 核心复核（抽查通过）

| DoD 项 | 证据 |
|---|---|
| 实跑 fast PASS | `evidence/console/logs/gate-1793580084.log`：`grep -cE "^  PASS/FAIL/SKIP"` = **17 / 0 / 16**，末行 `=== gate: 0 failure(s) ===` |
| 失败负例退出码非 0 可见 | `evidence/console/logs/gate-1793577717.log`：27/3/2，末行 `=== gate: 3 failure(s) ===`；`evidence/S32.4.log` 行 12 记录 `exit_code=1` + 前端 `renderGateStatus` 显示 |
| 非法 tier 400 / 锁 409 / 无 token 403 | `routes.rs:255`（403）、`:270`（Content-Type 400）、`run_gate` `:118`（400）、`:123`/`:156`（409）；`run_gate` 返回 `(u16, String)` |
| 门禁副作用写进执行日志 | 见 P2-2，gate.lock 持久化 `side_effects` |
| gate 全绿（full） | `evidence/console/logs/gate-1793578297.log`：30/0/2，末行 `=== gate: 0 failure(s) ===` |
| 页面明示 fast/touch 开发档 | `crates/ti-console/web/index.html:335` `⚠ fast/touch 为开发档，不构成 gate 全绿证据` |

## findings 表

| 严重度 | 位置 | 问题 | 建议 |
|---|---|---|---|
| P2 | `crates/ti-console/src/runner/gate.rs:143`（`File::create(&log_path)` 在 `:149` create_new **之前**）+ `:140`（`gate-{ts}.log` 仅秒级时间戳） | 新发现：日志文件先于锁创建，且日志名仅秒级唯一。两并发 POST 同秒时：B 的 `File::create` **截断** A 已建的日志（同名 gate-T.log），随后 B 的 `create_new` 失败 → `:153 remove_file` **删除** A 正在写入的日志；A 的 gate 子进程仍在跑（写入已 unlink 的 inode），但日志内容丢失、`GET /api/gate?log=` 与 history 均不可见（`read_to_string` 失败）。属 r1 P2-1 修复后残留的**日志侧**同类竞态（锁侧已闭环，日志侧未闭环）。 | ① 将日志文件创建移到 **create_new 成功之后**（占锁后再建日志）；② 或日志名加防碰撞因子（pid/纳秒/uuid），确保并发不同 POST 不同日志名。两者取其一即可，建议①为最小改动（建日志失败仍可释放锁，现有 `:176-178`/`:185-187` 清理路径已覆盖）。 |

## 判定依据小结

- r1 的 4 项 P2：**全部已修复**（代码行号 + 实读 gate.lock 内容佐证），`残留 = 0`。
- DoD 核心：fast PASS 17/0/16、失败负例 3 failures 可见、403/400/409 分支齐备、side_effects 持久化、full 档 30/0/2 全绿、页面开发档明示 —— 全部满足。
- 新发现 1 项（P2，非阻断）：日志侧同秒竞态（`create_before_lock` + 秒级日志名 + 409 时 `remove_file` 删他人日志）。不影响 gate 正确执行与锁语义，故不降级 `总判定`；建议下轮修复。
- `误报 = 0`（r1 四项均为真实缺陷，修复后闭环）。

**结论：S32.4 r1 收敛验证通过（PASS），4/4 闭环，新增 1 项 P2 建议（日志侧竞态），DoD 仍满足。**
