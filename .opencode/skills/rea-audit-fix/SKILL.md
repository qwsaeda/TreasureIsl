---
name: rea-audit-fix
description: Use when the user asks to run the twin-agent adversarial audit (双 agent 对抗审计 / 找 100 条 / FIND-AUDIT), verify doc↔code drift, or codify repeated audit findings into doccheck2. Front-loaded triggers: "对抗审计", "双 agent", "找 100 条", "FIND/AUDIT", "审计哪些能固化", "重复性检查".
---

# 双 agent 对抗审计（FIND → AUDIT → FIX）

固定循环，已在 金银岛 仓库跑通 5 轮。步骤如下，不要跳步。

## 1. FIND（找问题，100 条）

派 `general` 子代理，prompt 固定要点：
- 角色：FIND 子代理，第 N 轮对抗审计。
- 目标：仓库根 C:\金银岛，找出 100 条一致性/漂移问题，编号 Q001..Q100。
- 每条给：位置（文件:行号）、现象（实际值 vs 期望值，或代码 API vs 文档描述矛盾）、建议修复动作。
- 必扫区域（每轮更新，但骨架不变）：
  1. `crates/ti-app/src/game_app.rs` + `scenes/*.rs`：oracle 调用残留、`Unit{hp,atk}` 硬编码、Build/Trade 内联结算 vs 委托 `TradeLedger`/`BuildManager`、boss tier 随机 vs `map.boss_tier`。
  2. `assets/data/strings.json` 88+ keys vs `i18n::sget`/`sgetf` 实际引用（孤儿 key 检查）。
  3. gate 项数：`scripts/gate.ps1` 当前几项？`AGENTS.md`/`docs/parity.md`/`docs/loop.md`/`docs/release.md` 里的数字是否互相一致。
  4. LCG hex `0x343FD` 全库残留（`docs/design/constants.md`、`docs/impl/10_rng.md`、`docs/impl/06_cards_events.md`、`docs/facts/tools.md` F-320）；误算 `0x3443D` 是否还有。
  5. 据点 `>40` 阈值一致性（`docs/business/progression.md`、`docs/design/constants.md` §6.6、`assets/data/balance.json`）。
  6. `docs/impl/12_audio.md` 的 `AudioChannel`/`enum ChannelState{Idle,Playing,Paused,Stopped}` vs `crates/ti-platform/src/audio.rs` 真实 `struct ChannelState{active,volume,position,looping}`。
  7. `docs/business/*.md` 引用的数值（cards.event_chance、buildings base_cost、goods prices、boss_tiers hp/atk）是否逐一对得上 `assets/data/balance.json`。
  8. `docs/impl/*.md` 各模块错误枚举（E0xxx）是否与代码真实 error.rs 一致。
  9. `docs/plan-s8-refactor.md` 各批次描述与 `game_app.rs`/`scenes/*.rs` 实际重构状态。
  10. `AGENTS.md` 路由表/硬规则/命令节 与仓库实际文件是否存在性核对。
  11. `docs/experience/*.md` EXP-0xx 编号唯一性、index.md 是否全部登记。
  12. `plan/ledger.yaml` 任务计数 vs `docs/release.md`「N 任务 M verified」是否又漂移。
  13. `check_hardcode.py`/`validate_data.py`/`gen_layout.py` 内部断言数值（2041/20 场景/字符串 key 数）是否与 `strings.json`、`controls.json` 实时值一致。
- 取证：用 Read/Grep/Glob/Bash（PowerShell 5.1）实际取证，**不许凭记忆编造行号**。
- 输出：纯 markdown `## Q001` ~ `## Q100`，每条 3 行内。最后给统计（CONFIRMED-candidate 数量 + 最高价值 5 条 Qxxx 编号+一句话）。
- **必须把完整清单 Write 到 `C:\Users\Administrator\AppData\Local\Temp\opencode\audit_roundN_find.md`**（子代理输出不自动持久化，AUDIT 读不到就无法工作）。最终回复只返回统计摘要，不要把 100 条全文贴回。

## 2. AUDIT（逐条 verify）

派第二个 `general` 子代理，prompt 固定要点：
- 角色：AUDIT 子代理，读取上一步 Write 的 `audit_roundN_find.md`。
- 对 **每一条 Q** 判定：CONFIRMED / REJECTED / PARTIAL / NO-LINE。
- 重点独立复核（不要盲信 FIND）：
  - 战斗数值类（Unit hp/atk vs balance boss_tiers，推演是否可胜）
  - gate 项数漂移（grep 全库 `12 项`/`13 项`，与 `gate.ps1` 实际 `Report`/`Skip` 调用数比对）
  - business↔balance 数值（Read `balance.json` 逐字段比对）
  - `impl/12_audio.md` vs `audio.rs` ChannelState（读真实 struct 字段）
- 每条一行：`Qxxx [判定] 一句话证据（文件:行号 实际值）`。
- 最后给统计：CONFIRMED/REJECTED/PARTIAL/NO-LINE 各多少 + CONFIRMED 里最高价值 8 条编号+一句话。
- **必须 Write 到 `audit_roundN_audit.md`**。最终回复只返回统计 + 8 条最高价值 CONFIRMED。

## 3. 主线程独立复核（EXP-038）

不要盲信 AUDIT。对最高价值 8 条，主线程亲自 Read/Grep 取证一遍再动手。

## 4. FIX

- 按 CONFIRMED 清单修复。docs 为主；代码改动不得引入 `unwrap/expect/TODO/FIXME`（门禁红线）。
- 若某项是「同类高频复现」，**顺手补进 `crates/ti-tools/src/bin/doccheck2.rs`** 作为新断言（见 doc-check skill）。
- 每改一处用 Edit 工具做精确替换；**禁止用 PowerShell `Set-Content`/`Out-File` 改 CJK 源文件**（EXP-051）。

## 5. GATE

- `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1`（当前 13 项）。
- `cargo run -q -p ti-tools --bin doccheck2 .` 必须 0 error。
- gate 全绿 + doccheck2 0 error 才算本轮收口。

## 6. 循环判定

- 若 FIND+AUDIT 均 0 条 CONFIRMED → 循环结束。
- 否则进入第 N+1 轮（FIND 区域清单随本轮已修项更新，避免重复找同一条）。

## 固化的已知高频项（doccheck2 已覆盖，不用再人肉找）

| 检查 | 位置 |
|---|---|
| `impl/12_audio.md` ChannelState API 一致性 | `doccheck2` 第 1 项 |
| shell 层 `oracle_*` 函数调用残留 | `doccheck2` 第 2 项（与 `check_oracle_boundary.py` 双保险） |
| LCG hex 误算 `0x3443D`（权威处） | `doccheck2` 第 3 项 |
| 据点阈值反向断言 `<= 40` | `doccheck2` 第 4 项 |
| i18n 孤儿 key（失败级：代码用了但 JSON 没有；WARN 级：JSON 有但代码未引用） | `doccheck2` 第 5 项 |
| `release.md` 门禁项数漂移 | `doccheck2` 第 6 项 |
| 2041/UI 场景 20/`0x343FD`/`20011591` 数值断言 + release 台账计数 | `doccheck`（gate 第 9 项） |
