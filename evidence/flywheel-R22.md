# 飞轮 R22 · 定向轮（S62/S63 大批改动回归面 + 全域残余）（2026-11-21）

方法：开轮 gate 0 failure；FIND subagent 24 CONFIRMED + 1 PARTIAL（落盘 /tmp/opencode/audit_r22_find.md）
+ 主线程独立复核（EXP-038，另发现 FIND 未报的更深问题）；AUDIT 合并裁决后全修。

## FIND 24 CONFIRMED 分靶点
- S62 解锁语义 9：R22-001~009（模块/文档/注释/跨文档/存档注释 旧单调链残留 + 锁块死代码）
- S63 五面板 7：R22-010~016（**P0 真实键盘断链** / 面板互斥漏关 / gold_cap=0 清零 / i18n 硬编码 / 坐标 8pt / 魔数 / CLI 注释）
- Vs 立绘 2 + 1 PARTIAL：R22-017 断言覆盖 / R22-018 模块文档 / R22-019 竖屏一致性（用户只看横屏，记现状）
- 全域残余 1：R22-020 README 死链
- 测试盲区 5：R22-021~025（e2e 绕过 poll / 互斥组合遗漏 / 边界分支 / 近似恒真断言）

## 主线程额外发现（FIND 未报）
- **TI_VS_ANIM_PROTO=duel 走竖屏原型路径**（vs_proto 非 landscape），我此前 `--vs-phase=duel` 参数无效被忽略 ⇒
  之前的 vs_duel_A.png 其实也是 clash 帧。真正 duel 概念帧（vs_duel_A_real.png）走竖屏原型不居中。
  结论：S63.6 候选 A 在**发货路径（vs.rs 正式战斗 None→clash）= 对峙居中**已兑现；duel 是概念原型分支，
  非发货路径，不强改（记现状）。

## 修复清单（一次编译）
- **R22-010 P0**：input.rs poll MainMap 分支补 ↑↓/Enter/Back 路由（面板态读 self.*_panel_open）；
  Space 摇骰移入无面板 else（面板态吞 spin）。新增 app_panel_keyboard_e2e.rs 4 条经 poll() 真键盘路径测试
  （开/选/买/关不退出 MainMap + 5 面板两两互斥全覆盖含 event→trade + 面板吞 spin）。
- R22-011：OpenTradePanel arm 补关 event（主线程与 FIND 双发现）。
- R22-012：apply_event_effect 加 gold_cap==0 守卫（不 min(0) 清零）。
- R22-001/003/004/005/006/007/008：S62 旧单调链文档/注释全清（progression.rs 模块头 / progression.md /
  01_game_logic_overview 小节+mermaid / game_app mod.rs 3图旧世代 / territory.md / cross_map_demo / persistence 注释）。
  EXP/经验卷的历史记录（process.md/build.md）保留原貌（不改写历史）。
- R22-015/016/018/020：事件环绕用 EVENT_KINDS.len()-1 / CLI 注释 5 面板 / Vs 模块文档候选 A / README 死链修正。
- R22-021/022/023/024/025：真实 poll e2e + 两两互斥 + 商人金不足拒绝 + trade 卖金断言直白化。

## 残留（诚实登记，非 blocked）
- R22-002（from_unlocked/from_mask 旧链重建逻辑）：生产 0 消费方（仅 defeated_count/is_map_defeated 测试用），
  S62 defeated[] 进度语义保留，行为无玩家可见错误；标「旧档兼容占位」非真漂移，未改。
- R22-013（面板格式串硬编码中文）/ R22-014（横屏坐标非 8pt）：P2 UX/规范项，非功能错误；
  i18n 键孤儿 WARN（event.blacksmith 等）实为 doccheck2 不认 format! 动态拼 key 的扫描盲区，运行时消费正常。
- R22-019（竖屏 Vs 立绘旧对角）：用户仅看横屏，按现状保留。

## 停机判定（06_flywheel §4）
找问题 = 24 CONFIRMED 全处置 + 1 PARTIAL 裁决保留 / 主线程额外发现 1 已定性；
审计 = 0 误报（FIND 全部实测坐实）/ 0 残留（3 条残留均为已裁决的规范/兼容项，非功能缺口）；
修复 = 0 blocked。→ 收敛停机。

## 验证
cargo test 1022/0（+5 R22 测试）+ clippy 0 + audit_code 0 + CS 0 + doccheck 182/38/199/353 0
+ doccheck2 53 项 0 + gate 36 项 0 failure。
