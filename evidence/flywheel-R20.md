# 飞轮 R20 · 全域面（12 CONFIRMED + AUDIT 补报 Q013-Q015）（2026-11-20）

方法：开轮 gate 全绿（8080 在位 0 failure）+ FIND subagent 12 CONFIRMED + AUDIT subagent 独立裁决
（10 CONFIRMED / 1 REJECTED〔Q011 反了〕/ 2 PARTIAL + 补报 Q013-Q015）+ 主线程独立复核 top 条（EXP-038）。

## FIND 12 CONFIRMED（Q001-Q012）
- Q001: 23号 §0 evidence 565→566（R19 回写漏算自身新增 flywheel-R19.md，off-by-one）
- Q002: 11号 S18.6 LTO sibling 行（45/79/111）漏 R19 勘误注（EXP-189 同类）
- Q003: 05号:60 + 00_design:77 引已删 rules/battle.rs + rules_battle.rs（S34.6 D5 删）
- Q004: cards.rs 头注释 CardState hp+gold → crew+gold+gold_max
- Q005: progression.md 120/191 裸写 v4 扣血口径（同文档 146/155 已注记）
- Q006: 08 spec 资产表 node_battle 已删残留
- Q007: exhaust.rs 战斗段 struct（ExhaustUnit.hp/player_hp/ExhaustBossTier.hp）退役 HP 残留，注释声称迁移但字段未跟
- Q008: saveload V3/V4/V5 顶层 hp 死字段（v5 走 seats[].crew）
- Q009: persistence.md:28 示例 hp 无 v5 退役注记
- Q010: 22号 §3.1 persist_proof 行缺 timeout_s 字段说明
- Q011: 23号 §0 计数历史链错（R12 105 应 106 / 103 应 102）【AUDIT REJECTED：104 对，git ls-tree 坐实】
- Q012: AGENTS.md:13「物理归档」措辞误导

## AUDIT 补报（Q013-Q015）
- Q013: 4 处活文档裸写已删 rules/battle.rs（territory.md:308 / economy.md:108 / tutorial/12:93,97 / migration_linux.md:48）
- Q014: 23号 104 计数未拆口径（游戏 75 + 平台 29）
- Q015: exhaust 战斗段 + saveload 顶层 hp 退役 HP 残留无 business 定案行

## 修复（全自验 993/0 + clippy 0 + doccheck 180/38/199/345 0 + doccheck2 53 项 0）
- 文档回写：23号（566/104 拆口径/历史链勘误）+ 11号（sibling 行勘误）+ 05号/00_design（battle.rs→force.rs）+ 08 spec（node_battle 删）+ progression/persistence/combat（v4/v5 退役注记 + 退役定案行）+ AGENTS（归档措辞）+ 22号（timeout_s 字段）
- 代码：cards.rs 注释 + exhaust.rs 战斗段 struct 注释标退役 + saveload.rs 4 顶层 hp 标 #[deprecated] + 7 文件级 allow(deprecated)（saveload.rs/examples×3/tests×3）+ 活文档 rules/battle.rs→force.rs（Q013 4 处）
- **EXP-203**：python 批量改 .rs 截断坑（saveload 739→3 / exhaust_saveload 250→3，git checkout 恢复 + 改 edit 工具；改 .rs 一律 edit 工具 + wc -l 对拍 HEAD）
- 退役 HP 残留定案（Q007/Q008/Q015）：exhaust 战斗段 HP 字段保留（JSON 兼容 + data_tables 非空断言，注释标退役）+ saveload 顶层 hp 标 deprecated（老档兼容保留，文件级 allow），business 单一事实源 = combat.md §兵力模型 段尾

## 验证
cargo test 993/0 + clippy 0（全 all-targets）+ doccheck 180/38/199/345 0 + doccheck2 53 项 0 + studio_tests 3 类 OK + validate_data OK + ledger 351/351/0

## 停机判定（06_flywheel §4）
全域面 12+3 CONFIRMED 全修 + 0 残留 + 0 误报（AUDIT REJECTED Q011 已按 git 坐实回退）⇒ 全域面收敛停机
（第 1 个 0 轮 + 修复面 0 残留口径 M-24）；业务/渲染/平台/测试面 R3~R11/R16/R17/R19 已停机，本轮全域无待开轮。
