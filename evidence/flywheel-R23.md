# 飞轮 R23 · 定向轮（R22 残留规范项 + 新盲区）（2026-11-21）

方法：开轮 gate 0 failure；FIND subagent 23 条（高6/中12/低5，落盘 /tmp/opencode/audit_r23_find.md）
+ 主线程裁决后全修。

## FIND 23 条分靶点
- 靶点1 i18n 扫描盲区 6（FIND-001~006）：动态拼 key 不可见 + 无正向校验 / 5 面板 7 处硬编码中文恶化 1→7 / 脆弱豁免
- 靶点2 8pt 坐标 6（FIND-007~012）：5 面板横屏 y=130/190/230 非 8 倍数 / 8pt 门禁只扫 scenes.json
- 靶点3 浏览器 e2e 4（FIND-013~016）：wasm/browser 从未按 T/B/M/V/N / 面板 e2e 未登记 manifest（违反 S45）
- 靶点4 R22 残留恶化 4（FIND-017~020）：from_unlocked「0 消费方」事实漂移 / 硬编码 1→7 / 8pt 未修 / 竖屏 Vs 分叉
- 靶点5 全域 3（FIND-021~023）：README S39.0 in_progress 过期 / 23号测试文件数 108 过期 / evidence 计数 566→610 过期

## 修复（一次编译）
- FIND-005 i18n 硬编码：7 处格式串键化（strings +5 keys，sgetf 位置参数），sign_num 助手处理 `{:+}`。
- FIND-001 动态 key 正向校验：doccheck2 新增 check_event_keys_present（从 EVENT_KINDS 常量派生 7 key，
  先正向后豁免），`.event.` 加 exempt_prefixes 消 WARN；负例护栏实跑（删 blacksmith key ⇒ 精准 FAIL + 还原 0 error）。
- FIND-007~012 8pt：5 面板横屏坐标收进 chrome::PANEL_LANDSCAPE{_TALL,_MERCHANT,_SHIP} 具名常量，y=128/192/232 全 8 整除。
- FIND-015 面板 e2e 登记 test_manifest（+5 条，31→36），validate_data OK。
- FIND-013/014 浏览器 e2e：wasm_browser_e2e.py + web_playtest.cjs 补 T/B/M/V/N 面板键尽力覆盖（0 pageerror）。
- FIND-017 事实更正：R22 证据 + S61.1 dod「from_unlocked 0 消费方」→「BOSS 钩 host.rs:1090 唯一生产消费方，
  全解锁下恒等」，保留非功能缺口裁决。
- FIND-020 现状登记：竖屏 Vs 原型保留对角旧布局（用户仅看横屏）。
- FIND-021/022/023 计数回写：README S39.0 verified / 23号测试文件 113 顶层（114 含嵌套）+ evidence 610。

## 停机判定（06_flywheel §4）
找问题 = 23 CONFIRMED 全处置 / 审计 = 0 误报（全实测坐实）/ 修复 = 0 blocked。→ 收敛停机。

## 验证
cargo test 1022/0 + clippy 0 + audit_code 0 + CS 0 + validate_data OK（36 manifest）
+ doccheck 182/38/199/355 0 + doccheck2 53 项 0 + gate 36 项 0 failure。
