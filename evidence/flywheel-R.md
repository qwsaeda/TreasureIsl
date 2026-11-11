# 全域飞轮 R（2026-11-11，4 靶点合并 1 轮）

## 开轮
- flywheel_readme.ps1（S51.0 机制使用面第一次真用上：7 条必读 + docs/rules 5 红线按包域注入 + gate 36 项 + doccheck2 断言 + 7 包白名单）
- 4 靶点 = 用户拍板全选（全维度漂移 / 代码面 / 文档面 / 门禁元审计）合并 1 轮
- 3 角色并发：找问题（只读，100 条/轮）→ 审计（逐条裁决）→ 修复（真漂移落地）→ 主线程复核 + 需人裁决定夺

## 找问题 = 76 条（8 P0 + 35 P1 + 33 P2，按严重度取，非凑数）

## 审计裁决 = 真漂移 47 / 误报 19 / 需人裁决 15（81 行含拆分归并，独立修复点 ≈55）
（全文 evidence/flywheel-audit-verdicts.md 落 /tmp/opencode/audit_76_verdicts.md 摘要）

## 修复（真漂移 47 条落地，纯文档对齐 / 零视觉变化 / 不改游戏行为）
- P0-①b 删 9 孤儿 scene.vs.* i18n key（逐 key grep 0 代码消费后删；误删 scene.vs.hint 已还原）
- P0-②/③ AGENTS.md S28.0 业务漂移：`5 席位=选角≥1玩家+4AI`→`2..5 人类热座+可配 AI 席（balance.selection.min_players=2/max_ai_seats=4）`；`可测规则表 13 条`→`35 条（以 territory.md §10 为准）`
- P0-④a main_map.rs 幻象令牌 isle_volcano/isle_treasure/isle_boss → isle_ember/isle_gold/isle_black（theme.json 实有 key，零视觉变化，残留 0）
- P0-④b + P1-17/27 theme.json 加 5 新令牌 ink/ink_dark/isle_flag_dark/vs_duel_edge_1/2/3（ti-render Theme+DEFAULT+by_name+shell fixture 同步）；main_map 4×0x7BEF + vs_anim_v4 3 裸色 → theme.by_name().unwrap_or(回退同值)
- P0-⑥ host.rs 2 处硬编码 CJK toast → i18n_fmt("scene.main_map.toast.territory_fmt/dungeon_fail")（复用已有 key，check_hardcode.py game_app/ 豁免注释收窄）
- P0-⑦ 删恒 PASS check_scene_band_count（doccheck2.rs RUST_CHECKS 删 1 fn+1 行 + gate_data.json 删 1 行，43→42 项；select_role 退役后该断言永远 0 命中 = 形同虚设）
- P1-26 balance.json 加 combat.anim.hp_yellow + ti-core CombatAnimCfg.hp_yellow 字段 + vs_anim_v4 健康条黄色段走数据（原 0xE8C0 字面量）
- P1-28~33/P2-22/23/24/26 26_dev_stages_overview.md：280/270/9/1→334/333/0/1 blocked + 「33 项」×5→36 项 + 待办 16→0 + 下一节点 S34.0→「台账全收口，飞轮找问题/用户拍板新方向」 + S34 进行中→全 verified
- P2-1/2/4 AGENTS.md 计数 327→334/333 + gate 35→36 + 41→42 audit-codified（doccheck2 删 P0-⑦ 后）
- P2-5/6 territory.md §3 表 2 处空格对齐 balance.json
- P2-11 parity.md:62 P0-5 表行空列修复
- P2-34/35 select_map.locked.h 复核 = scenes.json 已有 "h":12，审计发现过期，无需改
- 死代码 3 函数（ai_turn_v2/victory_seat/player_defeated）= 需人裁决，未删（rules 层 pub + 内联测试引用，删 0 收益，棘轮已豁免）

## 需人裁决 15 条（主线程定夺）
- isle.*.guard/.benefit 20 key：**改判不删**（territory.md §3 明确「守军/每圈产出」列 = 业务规格预置 i18n 文案，副本场景（S33.6 后 10 专属场景）将消费；当前 0 引用 = 副本未实现，非孤儿）
- 越界 import `use ti_core::rules::*`（ti-app 表现层）：**误报不改**（表现层→规则层 = 正确分层方向；红线 1「越界」指反向 rules→渲染）
- 测试盲区补测（P1-1~11/22/23 各 rules 函数 0 集成测试 + P1-3/P24 force 测试层）：高成本，留后续专项
- scene.index.medallion 6 key / glmapm04..20.Map 17 key：动态 key 拼接（format!("glmapm{N:02}.Map")）= 静态扫描盲区，确认代码消费，保留
- board m21..m40 vs selection.maps 20 条 / 72 孤儿 WARN 批 / U2/U4 定案 / 70%缩放 / 8 豁免前缀：设计定案，留后续

## 自验（修复后独立实跑）
- cargo build --workspace 0 error（1 warning = ti-engine manifest，既有）
- cargo test --workspace 全 target ok 0 failed
- doccheck2: 0 error across 42 audit-codified（删 P0-⑦ 后 43→42）
- doccheck: 177 docs 0 error
- 三方对账: 333 verified 全绿 + 0 孤儿 evidence
- validate_data: OK（strings 226 键）
- check_hardcode/check_code_standards/audit_code/studio_tests: 全 0
- AGENTS.md 计数（334/333 + gate 36 + 42 audit-codified）与台账/gate_data.json 实测一致

## 停机判定
0 新发现（修复 agent 全绿）∧ 0 误报（动态 key/分层 import 类已识别不改）∧ 0 残留（真漂移 47 全修，需人裁决 15 全部定夺有结论）
⇒ 飞轮 R 收口。残留 = 测试盲区补测（P1/P2 共 ~14 条，高成本）+ 设计定案 7 条 = 留后续专项，不阻塞。
