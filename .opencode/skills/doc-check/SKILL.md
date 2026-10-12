---
name: doc-check
description: Use when the user asks to batch-fix doc↔code/data drift, update stale numbers in docs, or check whether a repeated finding is already covered by doccheck/doccheck2. Triggers: "批量修文档漂移", "数值对不上", "gate 项数", "台账计数", "EXP 重号", "business 数值", "哪个 doccheck 已经抓".
---

# 文档↔代码/数据一致性批量修复

## 先查是否已被固化脚本覆盖（不要人肉重复）

| 现象 | 先跑 | 说明 |
|---|---|---|
| LCG hex `0x3443D` 误算 | `cargo run -q -p ti-tools --bin doccheck2 .` | 第 3 项，扫 `impl/10_rng.md`+`impl/06_cards_events.md` 权威记录处 |
| 据点阈值反向断言 `<= 40` | 同上 | 第 4 项，扫 `docs/` 含「据点」的行 |
| i18n 代码用了但 JSON 没有的 key（裸 key 可见异常） | 同上 | 第 5 项失败级；JSON 有但代码未引用 = WARN 级（旧命名空间豁免） |
| `release.md` 门禁项数 vs `gate.ps1` 实测漂移 | 同上 | 第 6 项 |
| `impl/12_audio.md` ChannelState 与真实 `audio.rs` 字段矛盾 | 同上 | 第 1 项 |
| shell 层 `oracle_*` 函数调用 | 同上 | 第 2 项 |
| 2041/UI 场景 20/`0x343FD`/`20011591` 跨文档数值断言 | `cargo run -q -p ti-tools --bin doccheck` | gate 第 9 项 |
| `release.md`「N 任务 M verified」台账计数漂移 | 同上（`check_ledger_counts`） | 同上 |
| F-NNN/EXP-NNN 全库重号 | 同上 | 同上 |
| 文档引用的 F/EXP ID 不存在 | 同上 | 同上 |

**已被脚本抓到的问题，直接跑脚本看 ERROR 清单修复即可，不要再派子代理现场 grep。**

## 未被脚本覆盖的高频同类项（仍需人肉，可考虑补进 doccheck2）

### business/*.md ↔ assets/data/balance.json 数值漂移
Read `assets/data/balance.json`，逐字段比对 `docs/business/{economy,progression,combat,persistence}.md` 引用的数值：
- `trade.goods` buy 价格（20/76/95/114/133）
- `build.buildings` base_cost（1000/500/200/800）
- `cards.event_chance`（0.35）
- `combat.boss_tiers` 逐档 hp/atk
- `save.slot_count`（5）
文档里若用「设计示例/旧命名空间」（spice/silk/gem、palisade/harbor）但没标「示例非真实值」，补警示注释而非改示例本身。

### impl/*.md 引用的 API 与真实 .rs 不一致
例：`impl/05_combat.md` §7 曾引用不存在的 `BattleSession::new(seed,player,tier)`（真实是 `from_balance`/`new_with_cfg`）；`06_cards_events.md:335` 同错。
修法：Read 真实 `.rs` 方法签名，把文档里过时的 API 名改掉；若是「设计占位」代码块，加 `> 设计占位，勿照抄` 标注。

### gate 项数全库漂移
`gate.ps1` 顶层 `Report`/`Skip` 调用数 = 实际项数（当前 13 项）。
`grep` 全库 `12 项`/`13 项`，对照修正所有写死的文档（`AGENTS.md`、`loop.md`、`parity.md`、`plan-s*.md`、`gamebook.md`、`git.md`、`README.md`、`style.md`、`impl/14_asset_enhancement.md`、`experience/code.md`、`process/04_tooling.md`）。
`parity.md` 是唯一权威口径，其他文档以它为准。

### release.md 台账计数
跑 `cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` 拿实测 `N tasks, M verified`，更新 `docs/release.md` 对应行。`doccheck` 第 9 项 `check_ledger_counts` 会校验，改完必须与实测一致。

### experience/index.md EXP 编号唯一性
`grep "EXP-"` 全库，确认无重号；`index.md` 条目表每条 EXP 必须对应唯一编号 + 所属卷（formats/reverse/tools/build/process/code）。
重号时：保留较早的那条原编号，给较晚的换下一个空号，全文替换引用。

## 修完后

- `cargo run -q -p ti-tools --bin doccheck2 .` 0 error
- `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` 全绿
- 若本次修的是「同类高频复现」，把检查补进 `doccheck2.rs`（新加一个 `check_xxx` 函数 + 在 `main` 里 `all_errors.extend(...)`），让下一轮不用再人肉找。
