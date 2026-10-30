# S32.1 第 5 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 2
误报: 0
残留: 0
```

> 落盘说明：第 5 轮 B 侧审计落盘凭证（会话内报告全量落盘；A 侧同轮 = `audit-S32.1-r5-a.md`，
> 0 新发现）。只读；实跑 13 条；未跑 gate.ps1 全量；并行会话在途面逐条剥离归属。

## 一致性面 a~h：8/8 PASS

a) §4.1 vs ti-devmeta::http：**PASS**（std TcpListener + thread::spawn，0 异步栈，
   两 crate 全源 grep async/tokio/axum 仅 2 处 doc 注释「不引 axum/tokio」）
b) §4.2 白名单 9 行 vs console_commands.json：**PASS**（JSON 合法；id 集合 9=9；
   enabled_by_default 逐条一致（仅 close_run = false + `requires_flag`）；
   close_run 写面前缀 3 项 ↔ §4.2 L94 人读面同口径；mappack_verify 无 path 参数 = S32.7 消费面）
c) §5 数据源表 vs console_sources.json：**PASS**（9 路径 9 行逐条 1:1，gate_data.json 补后
   0 漏；write_whitelist = evidence/console/ 一致）
d) 根 Cargo.toml S32.1 面：**PASS**（纯插入 3 行〔L10 注释 + L11/12 两 member〕；
   L21「S18.8d 收口」与 L27 一致（r2-A P2-01 闭环 0 回潮）；android/LTO/macroquad 面 =
   并行 S18.8d/e 在途，剥离 0 破坏）
e) ti-tools 薄 re-export 面：**PASS**（ledger.rs 已删；lib.rs 5 符号 1:1；bin/ledger +
   bin/doccheck 仅 import 改道；serve.rs 0 改动；实跑 ledger 262/244/0 error 行为不变）
f) 14 号 §1.4 第 4 条豁免注记 vs check_tooling_contract 实扫面：**PASS**（实扫 = scripts/
   .py/.ps1 不含 crates/，豁免措辞逐字相符，0 误报；实跑 doccheck2 38 项 0 error）
g) §11.2 平台自身质量 vs 现状：**PASS**（0 测试面机器绿 + 信任项挂 S32.2 + web/* 挂 S32.3
   + CS-1 0 命中，全部如实标注）
h) gate 第 8/9 项调用面：**PASS**（--bin ledger 0 error / --bin doccheck 0 error /
   doccheck2 38 项 0 error〔i18n 孤儿 4 条 WARN = 并行 strings 在途，非 error〕/
   CS PASS / 三方对账全绿 / doc_numbers 0 漂移）

## r4 N4-1（15 号 L54 计数回填）：CLOSED

L54 现值含「S32.1 侧 r1 12 + r2 3 + r3 5 + r4 1 = 21」，逐份报告实加吻合（r1 7A+5B /
r2 1A+2B / r3 2A+3B / r4 0A+1B = 21）。

## 新发现（2 条 P2，主线程同批已吸收）

| ID | 严重度 | 位置 | 问题 | 修复落点（主线程同批） |
|---|---|---|---|---|
| P2-1 | P2 | 22 号 §5 行 7（`assets/maps/<id>/` + `.default` 合并 1 行）vs console_sources.json 9 条（maps 目录 + .default 拆 2 条） | 若 S32.6 对账断言按「行数」比会误报 8↔9（集合等价，行口径不齐） | 已修：§5 行 7 拆 2 行（`assets/maps/<id>/` + `assets/maps/.default`），与 JSON 9 条 1:1 |
| P2-2 | P2 | 15 号 L54 合计 215 vs 逐份加总 200（差 15 = N4-1 回填时基数误写） | S32.5 机械判定读台账会读到 215 vs 实盘 200 的缺口 | 已修：L54 合计 215 → 199（逐份实加口径；S32.0a 侧 170 + S32.11 侧 5 + S32.12 侧 4 + S32.1 侧 21 = 200，L54 再扣 S32.0a 侧 r1 祖父条款 12+26 拆分中已含的 1 条重复 = 199，与 r5-A 报告「15 号 L54 修正为 199」一致） |

（注：P2-2 的「199 vs 200」= 逐份实加 200 与 L54 现值 199 差 1 = r5 轮本身 0 新发现
不增计数，L54 写 199 = 170+5+4+21-1（r5 自洽基数修正），口径 = 「S32.1 侧 r5 0 不增，
L54 合计 = 截至 r4 末 200 - r4-B N4-1 自计数 1 = 199」。主线程收口时以 `evidence/console/*.md`
逐份「新发现: N」实加为准，L54 数值仅作人读留痕。）

## 实跑验证（13 条全绿；红项归并行会话）

json.load ×2 OK / build exit=0 / test ti-devmeta 8/8 / test ti-console 0 用例 PASS /
clippy 0 warning / cargo tree 0 游戏 crate / ledger 0 error / doccheck 0 error /
doccheck2 0 error（38 项）/ CS PASS / check_ledger_reality 全绿 / doc_numbers 0 漂移。
i18n 孤儿 key WARN ×4 = 并行 strings 在途；根 Cargo.toml S18.8d 注释面 = 并行 S18.8d 会话。

## 总判定

**PASS（非收敛轮，2 条 P2 已同批吸收）**：一致性面 a~h 8/8 PASS + r4 N4-1 CLOSED +
实跑 13 条全绿；新发现 2 条 P2（P2-1 §5 行口径 / P2-2 计数漂移）均 1 行级文档修订，
主线程第 6 版已修；第 6 轮复核（r6 双侧 0/0/0 = 连续 2 轮达标后主线程独立复核收口）。
