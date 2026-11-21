# 飞轮 R26 · 定向轮（R25 新改动回归面 + 扫描面外根因 + 证据面）（2026-11-21）

方法：开轮 gate 0 failure；FIND subagent 15 条（高3/中高1/中7/低4 + 4 阴性，落盘 /tmp/opencode/audit_r26_find.md）
+ 主线程裁决后全修。规律验证：R25 新引入的门禁断言/脚本改动自身 = 本轮高价值 finding 源。

## FIND 15 条裁决
- F-01【真·R25 自引入】manifest keyboard desc「30」≠ 源码 33（R25 F-09 改完即被同提交 F-14 加 3 断言破坏）→ 33 + validate_data 加「desc 声称断言数 == 源码 assert 宏数」对账（负例改 99 精准 FAIL）。
- F-02/F-03【真·扫描面外根因】停机口径 M-24 散落 13 处，15号 row21/26号:116 仍写「连续 2 轮」为现行；`.opencode/skills/`/`evidence/` 全在扫描面外 → 并入 check_flywheel_readback（不增项数）守 SKILL.md+15号+26号+rules 现行行不得含旧口径（负例插旧口径精准 FAIL）；15号/26号文本回写。
- F-04【裁决=非真缺陷】「eprintln!( 后 ≤4 行塞玩家可见字面量」构造本身是非法代码（字面量语法上就是 eprintln 参数），R26 真括号栈判定按语法豁免正确；合法边界 7 例全过。
- F-05【真】嵌套 format!/Some/vec!/&/花括号宏诊断串假阳性 → 括号栈判定 + NESTED_IN_DIAGNOSTIC 向外找外层诊断宏（7 边界 fixture 全过 + 全量 0 hits + 玩家可见插桩 FAIL）。
- F-06【真】负例未编码 → 7 边界 fixture 写进 `check_hardcode.py --selftest` + gate check_hardcode 段并入 selftest（一键复现，不增 gate 项数）。
- F-07【真】event_static_keys 白名单内容不验真 → 验每个 fixed key 在 strings 实存且不在 EVENT_KINDS 消费集（负例加孤儿 key 精准 FAIL）。
- F-08【真】段边界注释误报/同行闭合无界 → 剥注释 + 从 `= [` 括号配平（负例注释插 MaritimeEvent:: 不误报）。
- F-09【真】claims 缩锚后轮次列表失守 → check_cross_doc_counts 加三文档轮次列表签名一致性断言（不恢复长锚/不增项数，负例改一处精准 FAIL）。
- F-10【真】unwrap_or(Null) 吞根因 → 区分读失败/解析失败分别报错。
- F-11【真】8pt 形状不符双报 → 形状错误计入 found 避免总数断言双报。
- F-12【真】keyboard run.log 是 R24 旧产物（7 tests）→ R26 重跑刷新（7 passed）。
- F-13【真】evidence/flywheel-R25.md 356→357；F-14【真】23号文末 612+→相对描述；F-15【低·记档】MACRO_NAMES 集合无 selftest 枚举（本次已把关键 7 fixture 编码，宏集合扩 format_args）。
- 4 阴性核实：22号/06号口径一致✓ / 357 task refs 守门在✓ / 8pt 不误报✓ / hardcode 全域 0 hits✓。

## 停机判定（06_flywheel §4 / M-24 第 1 个 0 轮）
找问题 = 14 CONFIRMED + 1 裁决非真缺陷 / 审计 = 0 误报（4 阴性核实）/ 修复 = 0 blocked。→ 收敛停机。

## 验证
cargo test 1026/0 + clippy 0 + audit_code 0 + CS 0 + check_hardcode selftest OK + 严格 0 hits
+ validate_data OK（desc 断言数对账）+ studio_tests OK + doccheck 182/38/200/357 0
+ doccheck2 53 项 0 + gate 36 项 0 failure。
