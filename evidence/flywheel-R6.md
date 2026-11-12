# 飞轮 R6（2026-11-12，R5 收口后全库终扫轮，发现 1 P2 + B 侧 10 条幻影测试名/标签漂移）

## R6 找问题 agent（全域终扫）：1 finding
- **F-1（P2）**：`docs/business/territory.md` §10 可测规则表 11 行引用的「单测名」= 幻影名（全库 grep 0 命中），v3 7 行 + 4 行按「规则语义命名」未对齐实际测试函数名

## R6 A 侧（完成度）：0 findings（F-1 修复收口复核通过）
- 11 幻影名全改实际测试名后逐名 grep 实存（territory_rules.rs/selection_seats.rs/app_seats_e2e.rs/events.rs/board.rs 等全命中）
- §10 全表 35 行 0 其它漏网（38 个非测试标识符 0 缺失）
- 数值声称点 0 漂移

## R6 B 侧（一致性，独报 A 看不到的值漂移面 = 10 条真漂移）
| 条 | 类别 | 位置 | 裁决 |
|---|---|---|---|
| B-01 | P1 | combat.md:113 可测表表头指向 rules_battle.rs + 9 行中 4 条张冠李戴 | 真漂移（测试实际分布在 force.rs 内联 + territory_v2_rules.rs）|
| B-02~B-06 | P2 ×5 | combat.md 5 幻影名（unarmed_crew_reduced_power/battle_deterministic_no_rng/defender_power_wins/battle_both_sides_lose_crew_and_arms/battle_same_seed_same_result） | 全真漂移（force.rs 实存名 = power_weights_armed_over_barehand/deterministic_same_input_same_result/defender_wins_on_tie_or_higher/both_sides_attrition/combat_same_seed_same_result）|
| B-07 | P2 | economy.md:155 4 幻影名（crew_bought_from_slave_merchant 等） | 真漂移（merchant.rs 实存 = buy_crew_capped_at_boat_capacity/buy_crew_at_capacity_rejected/buy_arms_debits_and_adds_capped_at_crew/buy_arms_one_per_person_cap）|
| B-08 | P2 | persistence.md:53 save_migrate_v2_to_v3_defaults 缺 _boat_level 后缀 | 真漂移（tests/save_migration.rs:282 实存全名）|
| B-09 | P2 | territory.md:268 `ai_siege` 误写测试列 | 真漂移（ai_siege = i18n key 前缀非测试名；实存 = ai_sieges_player_cell_with_visible_feedback）|
| B-10 | P2 | AGENTS.md:8 + 26号:14/109/155/163 + release:239「1 blocked（S7.7 可选）」标签过期 | 真漂移（实测 S7.7 = verified，S49.1 = blocked 用户拍板作废）|

## 修复落地（主线程 10 条全修 + F-1 已修）
- combat.md 表头 + 7 行改 force.rs/territory_v2_rules.rs 实际测试名 + 归属标注
- economy.md 4 名 → merchant.rs 实际内联测试名
- persistence.md 补 _boat_level 后缀
- territory.md:268 ai_siege 澄清（= i18n key，实存测试名补上）
- S7.7/S49.1 标签 5 处文档改 S49.1（S7.7 已 verified）
- cross-doc-counts 模板同步 release.md 新串（防 doccheck2 报漂移）
- 判据：doccheck 0 + doccheck2 44 项 0 error + 三方对账 337 + S7.7 blocked 残留 0 + 9 幻影名残留 0

## 停机判定（CCGS 双审计 §7.4 连续 2 轮 0/0/0）
- R5 = 第 1 个 0 轮（0 findings）；R6 = 找问题 1 + B 侧 10 findings（非 0 轮）
- **连续 2 轮 0/0/0 未达成** → 须 R7 再开 1 轮 0/0/0 才正式停机
- B 侧 10 条根因（文档 §可测表引用幻影测试名 / 标签过期）建议 S53 固化 doccheck2 新断言（扫描 .md `` `snake_case` `` vs crates/ 实存测试函数，缺即 WARN），本轮先人工修 + 留痕
