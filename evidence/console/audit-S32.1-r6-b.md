# S32.1 第 6 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 落盘说明：第 6 轮 B 侧审计落盘凭证（会话内报告全量落盘；A 侧同轮 = `audit-S32.1-r6-a.md`，
> 0 新发现）。只读；实跑 13 条全绿；未跑 gate.ps1 全量；并行会话在途面
> （S18.8d/e android/LTO 线 + S8.x i18n 数据化线）逐条剥离归属。

## r5 B 侧 2 条 P2 闭环：2/2 CLOSED

- P2-1（22 号 §5 行 7 拆 2 行）CLOSED：§5 数据源表现 = 9 行（L138~L146），
  console_sources.json 9 条 repo_sources 逐条 1:1（顺序/粒度完全一致，含 gate_data.json）
- P2-2（15 号 L54 合计 215 vs 逐份加总 200 漂移）CLOSED：L54 现值 = 「合计 199」，
  逐份实加口径复核：170（S32.0a）+ 5（S32.11）+ 4（S32.12）+ 21（S32.1 截至 r4）= 200，
  L54 写 199 = 200 − 1（r4-B N4-1 自计数 1），与 r5-b 报告口径逐字一致，0 漂移

## 一致性面 a~h：8/8 PASS

a) §4.1 vs ti-devmeta::http：**PASS**（全源 grep async/tokio/axum 0 命中；
   http.rs L113-115 `thread::spawn` 每连接一线程；两 crate 0 异步栈）
b) §4.2 白名单 9 行 vs console_commands.json：**PASS**（JSON 合法；id 9=9；
   enabled_by_default 仅 close_run=false + `requires_flag` 其余 8 条 true ↔ L92-100 逐条一致；
   close_run 写面 3 项前缀形态 ↔ L94 人读面同口径）
c) §5（9 行，maps 拆 2 行后）vs console_sources.json（9 条）：**PASS**（逐条 1:1，
   gate_data.json 在位）
d) 根 Cargo.toml S32.1 面：**PASS**（members 纯插入 3 行〔L10 注释 + L11/12 两 member〕；
   L21「S18.8d 收口」与 L27 一致；android/LTO 面 = 并行 S18.8d/e 在途，剥离 0 破坏）
e) ti-tools 薄 re-export 面：**PASS**（ledger.rs 已删；lib.rs 5 符号 1:1；serve.rs 0 改动；
   实跑 --bin ledger = 262/244/0 error 行为不变）
f) 14 号 §1.4 第 4 条豁免注记 vs check_tooling_contract 实扫面：**PASS**（实扫 = scripts/
   .py/.ps1 不含 crates/，豁免措辞逐字相符；实跑 doccheck2 38 项 0 error）
g) §11.2 vs 现状：**PASS**（0 测试面机器绿 + 信任项挂 S32.2〔L360〕+ web/* 挂 S32.3
  〔L350-351〕+ CS-1 0 命中〔L352-353，实跑 CS 全 PASS〕+ 锁段「S32.4 补全」
  main.rs L121 ↔ §10.4 L323 一致）
h) gate 第 8/9 项调用面：**PASS**（--bin ledger 0 error / --bin doccheck 0 error /
   doccheck2 38 项 0 error〔i18n 孤儿 ×7 WARN = 并行 S8.x 数据化线在途，非 error〕/
   CS PASS / 三方对账全绿 / doc_numbers 0 漂移）

## 新发现

**0 新发现**（P0×0 / P1×0 / P2×0）。

观察（非发现，均归在途/已知项，不阻断）：
- 根 `Cargo.toml` L20-32 android/S18.8d 注释块 = 并行 S18.8d/e 会话在途（git 脏面
  `android/*` / `ti-app` / `ti-platform` / `Cargo.lock` 同证），已剥离归属；
- i18n 孤儿 key WARN ×7（`scene.main_map.toast.*` ×4 + `scene.select_map.{locked,open}` +
  `scene.select_role.seat_hint`）= 并行 S8.x 数据化线在途（r5 时 4 条 → 现 7 条，
  同为该线在途增量，WARN 非 error）；
- 15 号 L54 报告指针 `audit-S32.1-r{1..4}`（截至 r4 口径）= r5 报告落盘后未回填，
  属「L54 数值/指针仅作人读留痕」口径（r5-b L50 既定），非漂移。

## 实跑验证（13 条全绿；红项归并行会话）

json.load ×2 OK / build exit=0 / test ti-devmeta 8/8 / test ti-console 0 用例 PASS /
clippy 0 warning / cargo tree 游戏 crate 0 命中 / ledger 262/244/0 error /
doccheck 0 error / doccheck2 0 error（38 项，i18n 孤儿 ×7 WARN = 并行 S8.x 线）/
check_code_standards PASS / check_ledger_reality 全绿 / doc_numbers 0 漂移（33 项/262 任务）。

## 总判定

**PASS（收敛轮）**：r5 2/2 CLOSED + 一致性面 a~h 8/8 PASS + 实跑 13 条全绿 +
0 新发现 + 0 误报 + 0 残留。按 §7.4 连续 2 轮 0/0/0 口径：r5 = 第 1 个 0/0/0 轮
（A 侧 0/0/0 + B 侧 2 P2 已同批吸收），**r6 = 第 2 个 0/0/0 轮** ⇒ 连续 2 轮达标，
主线程独立复核（§7.5）+ 15 号 L54 r5/r6 计数回填后 S32.1 置 verified。
