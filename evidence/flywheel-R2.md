# 全域飞轮 R2（2026-11-11，S51 机制使用面第一次真用 = 3 subagent 规范结构 + A/B 双审计对抗）

## 开轮
- flywheel_readme.ps1（7 条必读 + docs/rules 5 红线按包域注入 + gate 36 项 + doccheck2 断言 + 7 包白名单）
- **本轮与 R1 的本质区别**：3 角色 subagent 的 prompt 头部 = flywheel_readme 输出全文（规范结构下发），
  不再手搓任务描述式 prompt；审计从单线程降级改 **A/B 双 agent 对抗**（A 完成度 / B 一致性 独立并行）；
  修复 agent prompt 带「五步全走 + 红线基准面」字段。

## 找问题 = 42 条（0 P0 > 14 P1 > 28 P2；全在 R1 修复 + 3 新断言后的门禁盲区面）

## 审计 A/B 双 agent 对抗
- A 侧（完成度）= 19 真漂移 / 14 误报 / 9 需人裁决；补找 B-1~B-3（数据字段 0 消费 / c1 契约缺 modules / 测试名 MISSING）
- B 侧（一致性）= 独报 9 条（B-01~B-09）；抓到 A 看不到的 3 条真漂移：
  B-01/B-02（doccheck2 项数 42→44 计数漂移，AGENTS:16 + 26号:141）/ B-04（lighthouse benefit 三方不一致）/ B-08（world_canvas 锚点待验）
- 主线程亲自复核 top 8 P0/P1 独立 grep 复验（EXP-038/058：子代理声称不算数）

## 修复（真漂移 5 条落地，零游戏行为变化）
1. AGENTS.md:16 + 26号:141 计数 42→44 项（B-01/B-02）
2. c1 契约补 7 modules 公共面 + 越界 import 合法化注（P1-15/B-2：表现层→规则层 = 正确分层，红线 1「越界」指反向 rules→渲染）
3. lighthouse benefit 三方对齐（B-04：doc 删「（少 1 格结算）」说明后缀，对齐 balance/strings「航行加速」）
4. progression.md 加 m21..m40 扩展包定位注（需人裁决 ④）
5. territory.md U2/U4 标「已定案」（需人裁决 ⑨，balance.manpower loss_pct=1500/weapon_loss_pct=2000 已落地）

## 需人裁决 9 条全定夺（主线程）
① isle.*.guard/.benefit 20 key = 保留（业务规格预置文案）② 测试盲区 ~14 类 = S52.1 留后续
③ 死代码 3 函数 = S52.3 留后续（不删）④ m21..m40 = 扩展包定位 ⑤ medallion 6 key = 动态 key 保留
⑥ glmapm17 key = 动态 key 保留 ⑦ 70% 棋子缩放 = 设计常量不改 ⑧ force.rs 内联 7 测试 = 保留 ⑨ U2/U4 = 已定案

## 留后续登记（15 号台账 32/33/34 行 + S52.1/2/3 pending 台账）
- S52.1 rules 集成测试盲区 ~14 类补测专项
- S52.2 i18n 假孤儿 9 条清零（i18n_fmt 扫描面扩）
- S52.3 死代码 3 函数处置

## 自验
- cargo build --workspace 0 error + doccheck2 44 项 0 error + doccheck 0 error
- 三方对账 334 verified 全绿 + 0 孤儿 evidence + ledger 338 任务/334 verified/3 pending
- 负例护栏（R1 已验 3 断言 × 各 1 命中）

## 停机判定
42 条 = 真漂移 5 全修 ∧ 需人裁决 9 全定夺 ∧ 误报/已修/值一致 28 不改 ∧ 高成本 3 项登记 S52.x
⇒ 0 新发现（R1 已修项全 PASS，无 R1 漏修残留）∧ 0 误报（动态 key / 设计常量已识别）∧ 0 残留（S52.x 挂账有任务号）
R2 收口。A/B 双审计首轮 = 各 1 轮（连续 2 轮 0/0/0 收敛口径：R2 为开轮，R3 收敛轮）。
