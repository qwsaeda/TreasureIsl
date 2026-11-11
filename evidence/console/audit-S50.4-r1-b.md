# S50.4 双审计 r1 B 侧（一致性/风险）

总判定: PASS
新发现: 12
误报: 0
残留: 0

## findings（B-01~B-12，P1×1 + P2×11）
- B-01 P1 项数口径分裂：WBS/32号/AGENTS/ledger S50.3「第 44 项」但 gate_data.json 实测 rules-files = 第 43 项（43 条末位）⇒ 全库 44→43
- B-02/B-10/B-11 P2 时点标注：35/36 项并存须时点语境（S50.0 立项 35 / S50.4 收口 36 / §7 既有 43 立项时点）
- B-03 P2 WBS 行 72 waiver 引用勘误（process_ratchet.json → studio_tests_waiver.json）
- B-04 P2 「id 映射」→「command 映射」（14/04/README 三处 + 源码 _command_mapped_in_console 全文匹配口径）
- B-05 P2 waiver rules_battle 重复 2 次 ⇒ 去重 26→25
- B-06 P2 check_rules_files glob 粗验限定 frontmatter 段（正文裸路径列表项误判面消除）
- B-07 P2 fn check_* 「第 N 项」doc 注释编号时点快照（43/44 争议两处对齐）
- B-08 P2 parity.md 代码块补 25b 明细行 + 「机器口径 35」残留
- B-12 P2 WBS 任务表 S50.2 行「或 doccheck2 第 43 项」措辞回填实际落地门（gate 25b）

5 重点核查全 PASS（3 类断言自洽 / glob 粗验不误判 / visual-signoff 改精不漏 9 条数据面任务 / doc_numbers 不误伤「3 项」/ 改动面游戏 6 crate 0 diff）。
