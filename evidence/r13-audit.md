# R13 双 agent 对抗审计收口（2026-11-16，FIND 100 条 + AUDIT 判定 + 主线程 8 条复核）

## AUDIT 判定统计
- CONFIRMED 23（去重 8 组独立缺陷）/ REJECTED 55 / PARTIAL 22 / NO-LINE 0
- FIND 自报 24 CONFIRMED 偏差：10 条行号定位失准（16/24/25 号「33/34 项」声称面实测 0 命中）⇒ REJECTED；
  时点快照/历史计划/EXP 叙事卷 = 豁免面 ⇒ REJECTED

## 主线程复核 8 条全修
| 条 | 缺陷 | 修复 |
|---|---|---|
| Q003 | 26 号 L14/L155/L163「流程强化 8 条」vs AGENTS.md「7 条」（枚举 S54.1/S55.1/S56.0/S57.0/1/2a/2b = 7） | 26 号 3 处 → 7 条 |
| Q005 | 26 号 L141「doccheck2 47 项」漏 S57.1 +1 = 48 项 | → 48 项 + S57.1 注 |
| Q079 | ti-console web/index.html 3 处硬编码「门禁 33 项」（R10 勘误漏平台前端） | 3 处 → 36 项 |
| Q038 | 27 号 8 处「gate 33 项」无时点标注 | → 36 项 |
| Q002 | plan-s56 复选框 S56.0/S57.0/1/2a 未随台账 verified 回写 | 4 行 → [x] verified |
| Q088 | process.md L548 乱码「（off门禁 34 项）」 | 时点标注修正（现行 36 项注） |
| Q098 | 26 号 L181 门禁 36 项详列实列 31 项缺 dod-commands/studio-tests/android 真机 | 补全 3 项 + 去重口径注 |
| Q017 | check_ledger_reality.md「100%」vs「60% 阶梯」混读歧义 | 消歧注（实测值 vs 下限常量） |
| Q050 | 08 号「gate 第 33/34 项」 | 实测 0 残留 = REJECTED（AUDIT 误报） |

## 固化面
- 高频复现模式（数值声称点同组多文档口径漂移：7/8 条 + 47/48 项 + 33/36 项 3 组同模式）：
  ①「流程强化 N 条」= 枚举面（S54.1/S55.1/S56.0/S57.0~S57.2b = 7 条，S53 属飞轮 R6 断言固化不计数）——
  doccheck2 cross-doc-counts 未覆盖「N 条」枚举面（只覆盖 N 任务/N verified），EXP-192 登记；
  ②「doccheck2 47/48 项」= gate_data.json 实测位表面（doccheck2::check_doccheck_counts_match 已覆盖 26 号工具矩阵行，
  本轮 L141 即该断言面 ⇒ 已固化载体）；
  ③「门禁 33/36 项」= doc_numbers.py 声称回写面（S46.2 已覆盖 8 文档 + S57.2b 扩 2 文件；ti-console 前端 3 处
  = 平台域硬编码面，**未进 doc_numbers 声称点文件面** ⇒ R13 真缺口，EXP-192 登记 + 后续 doc_numbers 扩
  `crates/ti-console/web/index.html` 声称面）。
- 复选框回写面（plan-s56 4 行 [ ] 未随 verified 回写）= 台账 WBS 文档面，无机器断言（doccheck2 process-ledger
  守 15 号，plan-sNN 复选框面 = 人工面），EXP-192 登记。

## gate 全量：36 项 0 failure（2 环境 SKIP）
