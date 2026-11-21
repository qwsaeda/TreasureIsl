# 飞轮 R24 · 定向轮（R23 回归面 + 面板 e2e 质量 + 扫描面外残余）（2026-11-21）

方法：开轮 gate 0 failure；FIND subagent 14 条（高3/中7/低4，落盘 /tmp/opencode/audit_r24_find.md）
+ 主线程裁决后全修。其中 3 条高价值是 R23 自己引入的回归。

## FIND 14 条分靶点
- 靶点1 面板 e2e 质量 4（FIND-006 弱断言 `>`/FIND-007 poll 只覆盖 2/5/FIND-012 注释 gid + roll_before 摆设/FIND-013 满级无测试）
- 靶点2 R23 回归 5（FIND-001 损耗%丢 /100、5%显示500% / FIND-002 硬编码 851 漏键化 + 整文件 CJK 宽豁免 / FIND-003 新断言可静默空转 / FIND-004 h=820 非8整除 + 常量零门禁 / FIND-014 前缀豁免孤儿盲区）
- 靶点3 浏览器 e2e 1（FIND-008 面板键无状态断言 + L4 降级吞 L3 pageerror）
- 靶点4 全域漂移 2（FIND-005 23号 610/566 并存 / FIND-010 release 8 vs AGENTS 9 + FIND-011 evidence 354）
- 靶点5 check_event_keys_present 鲁棒性（FIND-003 解析脆弱 + FIND-014 反向对账缺失）

## 修复（一次编译）
- FIND-001 损耗%：`/100` 恢复（万分比→百分比）；FIND-002 851 键化 `ship.stat_fmt` + check_hardcode 整文件豁免收窄为「仅诊断宏行」（负例护栏实跑：插玩家可见中文⇒精准 FAIL+还原 0 hits）。
- FIND-003/014 check_event_keys_present：定位 `const EVENT_KINDS:`（非首个提及）、捕获 0 = error、take 3200、反向对账（strings event.* 非固定 key 须被 EVENT_KINDS 消费）。
- FIND-004 h=820→816 + doccheck2 新增 `ui_panel_landscape_8pt_check` 扫 chrome.rs 四常量各分量 `%8==0`（负例护栏：h=820⇒精准 FAIL+还原 0 error）。
- FIND-005 23号删尾部 566 旧从句 + 实测 611（commit 前）；FIND-006 event e2e `>`→精确 +200/+3。
- FIND-007 补 merchant/build/ship 三条 `*_via_poll_keyboard`（5/5 面板真键盘覆盖；build Enter 成功后自动关面板故 Back/Enter 分开验证）。
- FIND-008 浏览器 e2e 逐键 pageerror 切片 + L4 降级门控（非 L4 层全过才 PASS，面板键异常不被吞）；web_playtest 逐键记账。
- FIND-009 manifest 5 条 evidence→真实 run.log（实跑生成）+ desc 断言数对齐；FIND-010/011 计数对齐（release 9 / evidence 355）。
- FIND-012 roll_before 摆设→真 `assert_eq!(roll_before, roll_after)` + 注释 gid 对齐；FIND-013 补 `ship_panel_auto_close_at_max_level`（连升到 max=4，满级 Enter 自动关）。
- 负例护栏两条实跑通过（CJK 插桩 / 8pt h 回退）。

## 停机判定（06_flywheel §4）
找问题 = 14 CONFIRMED 全处置（含 3 条 R23 自引入回归）/ 审计 = 0 误报（全实测坐实+未发现项单列）/ 修复 = 0 blocked。→ 收敛停机。

## 验证
cargo test 1026/0（+4 新测试）+ clippy 0 + audit_code 0 + CS 0 + check_hardcode 0 hits（收窄豁免后）
+ validate_data OK + studio_tests 3 类 OK + doccheck 182/38/199/356 0 + doccheck2 53 项 0
+ gate 36 项 0 failure。台账 362 任务/361 verified/1 pending。
