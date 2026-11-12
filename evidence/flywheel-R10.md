# 飞轮 R10（2026-11-12，平台面首次审计轮）

## 开轮前置
- 新建 docs/rules/ti-console-rules.md（平台红线第 6 个，flywheel_readme rules 5→6）
- R10 域 = 平台面（ti-console 7 模块 8937 行 + ti-devmeta，R3~R9 未覆盖）
- S55 断言实战拦截：新红线文档引入 177→178 docs 漂移 → 26号 同步

## 三轮对抗（找问题 + A 完成度 + B 一致性，独立并行）
**找问题 agent 4 P2**：
- F3-1 ui_theme 22号§4.2 人读表漏行（whitelist_audit 机器断言盲区）
- F5-1 22号「门禁 33 项」×3 → 36
- F5-2 gate 项号值域 1..=33 → 1..=36
- F5-3 30号「第 35 项 visual-signoff」→ 第 37 项

**A 侧（完成度，盲扫抓找问题漏扫面）3 条**：
- F-A1 23号「33 项/1..=33」×7（找问题只扫了 22号，漏 23号同族声称面）
- F-A2 23号:10 静态快照「82 测试文件/327 evidence」→ 实测 106/507
- F-A3 29号:76「第 35 项」→ 第 37 项（30号改 37 后 29号同检查漏改）

**B 侧（一致性，独报值漂移面）3 条**：
- B-1 23号:10 evidence 507→534（git-tracked 口径，随盘增不随文改）
- B-2 14号「33 项」×3 → 36（门禁结构演进人工行未回写）
- B-3 32号 S50 时点快照 4 处加「时点」注（41 项/35 项非现行，防误读）

## 修复落地（主线程 7 条全修，纯文档 + validate_data.py 值域 + 新红线 1 文件）
- 22号 §4.2 + 30号 §3.1 补 ui_theme 写点行 + 计数 8→9
- 22号/23号/14号「33 项」全改 36（含勘误注）
- 31号/validate_data.py/23号 gate 值域 1..=33→1..=36
- 30号/29号「第 35 项」→「第 37 项 visual-signoff」
- 23号:10 静态快照 106 文件/534 evidence（git-tracked 口径注明）
- 32号 4 处时点快照加注

## 判据
- doccheck2 47 项 0 error + doccheck 0 + validate_data 0 违规（值域 1..=36）
- 三方对账 339 + close_tests 12/12（ui_theme 补行后 22号 §4.2 表 24 id ↔
  HUMAN_VIEW_IDS ↔ console_commands.json 三方逐 id 相等）
- 0 游戏 crate diff

## 平台面 0 轮判定（CCGS §7.4）
- R10 = 7 findings（非 0 轮）→ 须 R11 0 轮复验确认「R10 修后 0 + R11 0」连续 2 轮 0 才平台面停机
