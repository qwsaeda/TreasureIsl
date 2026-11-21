# 飞轮 R25 · 定向轮（R24 新改动回归面 + 结构性去漂移 + 扫描面外过期口径）（2026-11-21）

方法：开轮 gate 0 failure；FIND subagent 16 条（高5/中6/低5 + 2 阴性，落盘 /tmp/opencode/audit_r25_find.md）
+ 2 条阴性 V-01/V-02 核实；主线程裁决后全修。F-13 是我并行提交 EXP-204 引入的真 gate 红。

## FIND 16 条分靶点
- 靶1 R24 回归面 5（F-01 fixed 数组数据写死 Rust / F-02 take3200 截断无边界判定 / F-03 DIAGNOSTIC_MACRO 整行搜索双向脆弱 / F-04 game_app/ 宽豁免下 board.rs:494 玩家可见 CJK / F-05 8pt 形状不符静默 continue）
- 靶2 结构性漂移 3（F-06 claims 短语写死飞轮轮次列表每轮改 4 处 / F-07 23号 evidence 数硬编码无守门已 stale / F-08 历史括注膨胀）
- 靶3 证据面 2（F-09 manifest desc 断言数 ship/keyboard 过期 / F-10 prove 目录布局分叉；V-01 5 份 run.log 真输出）
- 靶4 扫描面外 3（F-11 flywheel-kickoff SKILL 3 处「2 轮」废止口径 / F-12 ti-console-rules:31 同源 / F-13 EXP-204 引入 doccheck 计数红）
- 靶5 新测试质量 3（F-14 两条 poll 测试弱断言 / F-15 重复播种 / F-16 互斥测试口径；V-02 满级/roll 断言可证伪）

## 修复（一次编译）
- F-13：26号:141 199→200（EXP-204 回写引入的真红，gate 回绿）。
- F-06：doccheck2 claims[0] 短语缩到稳定锚 `{tasks} 任务（{verified} verified`，飞轮轮次列表移出精确匹配 ⇒ 不再每轮改 4 处。
- F-07/F-08：23号:10 evidence 数改「以 git ls-files 实测为准」+ 历史时点链移文末「计数变更历史」段。
- F-01：gate_data.json 加 `event_static_keys` 机读源，check_event_keys_present 读数据不写死 Rust（缺键报错不空转）。
- F-02：EVENT_KINDS 段边界从 take(3200) 改为 `];` 闭合；解析 key 数须 == MaritimeEvent:: 声明数（负例：改 key 精准 2 error）。
- F-03/F-04：check_hardcode 的 CJK 宽豁免收窄为「字面量最近包裹调用是诊断宏」——CALL_OPEN 追踪 name!/(，跨行 eprintln! 识别 + 同行混合/纯玩家可见两类负例均精准 FAIL+还原 0 hits；board.rs:494 玩家可见 toast fallback 删硬编码直接 sget。
- F-05：ui_panel_landscape_8pt_check 形状不符报 error 不静默 continue + 解析数 == 声明数（多行常量变形被抓）。
- F-09：manifest desc ship 7→10 / keyboard 15→30；F-10 23号 §7.2 补 prove 同构口径。
- F-11/F-12：SKILL.md 3 处 + ti-console-rules:31「连续 2 轮 0/0/0」→「第 1 个 0 轮即通过（M-24/EXP-190），第 2 轮可选；历史快照不回改」。
- F-14：merchant poll Enter 补 arms+1/gold-50 精确断言、ship 补 level+1（去「面板仍开」弱断言）；F-15 删重复播种；F-16 文档注明 handle 互斥 vs poll 路由分工。
- 负例护栏 3 条实跑通过（event key 改坏 / CJK 同行混合 / CJK 跨行诊断识别）。

## 停机判定（06_flywheel §4 / M-24 第 1 个 0 轮）
找问题 = 16 CONFIRMED 全处置 / 审计 = 0 误报（2 阴性核实）/ 修复 = 0 blocked。→ 收敛停机。

## 验证
cargo test 1026/0 + clippy 0 + audit_code 0 + CS 0 + check_hardcode 0 hits + validate_data OK
+ studio_tests OK + doccheck 182/38/200/356 0 + doccheck2 53 项 0 + gate 36 项 0 failure。
