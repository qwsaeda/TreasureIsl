# AI 玩家自动化对抗 E2E · 方案对比（2026-11-20）

## 现状（已有面）
- **AI 决策**：`territory_v2::ai_policy_v2`（兵力模型，数据驱动 `balance.territory.ai`：
  `siege_mode`/`upgrade_mode` 公式形态 + `attack_threshold`/`upgrade_priority`/`toll_aversion` 系数，
  EXP-196 固化「调平衡不改代码」）。纯函数，输入 = `Balance` + `TerritoryState` + 席位 `gold/crew/arms`，
  **零渲染依赖 ⇒ headless 可驱动**。
- **对局生命周期判定**（rules 层全有）：`bankrupt(seat)` / `victory_seat(seats)` /
  `player_defeated` / `players_defeated`（territory_v2.rs:179-250）+ `is_cleared`/`on_boss_defeated`
  （progression.rs:95-146）+ `assault`（攻占一次性结算，耗兵力 + 重定驻军 1/3 守恒）。
- **单回合 e2e**（已有，但不是"一局对抗"）：`app_ai_turn_e2e`（AI 摇骰/攻占可见 + toast）/
  `app_siege_e2e` / `app_seats_e2e` / `app_territory_turns`。
- **单人 agent 试玩**（不是 AI 对手）：`full_run.json`（16 步人路径）/ `random_session_demo`。

## 缺的（用户要的面）
**完整对局自动驱动**：N 个 AI 席（2..5，`balance.selection.max_ai_seats=4`）从头跑整局
（选角→摇骰→落格分叉→领土战争→BOSS→通关/破产），每个 AI 回合自动调 `ai_policy_v2` 决策，
**无人工输入**，跑到 `victory_seat`/全破产/`is_cleared` 终局，断言对局性质。

## 三个技术方案

### 方案 A · 规则层 headless 对局驱动器（推荐先做）
**载体**：新 `crates/ti-core/tests/adversarial_e2e.rs`（或 `examples/adversarial_session.rs`）
+ 一个纯规则层驱动器 `rules/territory_v2::run_adversarial_game`（或测试内 loop）。
- 驱动器 = `loop { 每席依次：摇骰(GameRng 可播种)→沿环移动→落格结算(ai_policy_v2 决策→assault/进副本/升级/过路费)→资源变动→bankrupt 判定 }` 直到 `victory_seat` 或全破产 或回合上限。
- **断言面**：① 确定性（同 seed 同参数 ⇒ 同终局，复现）② 兵力/金守恒（攻占 1/3 驻军守恒 + 双方必损）③ 终局可达（`victory_seat` 非 None 或全破产，不卡死）④ 回合上限兜底（max_rounds 不无限拉锯）⑤ 多 AI 不串席（SeatIdx 隔离）。
- **优点**：纯规则层，headless 快（ms 级跑满局 × 多 seed），直接断言对局性质，复用现成 `ai_policy_v2`+`force::resolve`+生命周期函数，零渲染。**这就是"AI 自动对抗真实 E2E 玩"的最小可信形态。**
- **代价**：~200 行测试 + 可选 1 个规则层驱动器函数；不动 app 壳/渲染。
- **风险**：摇骰→移动→落格的"回合编排"逻辑现散在 app 壳（board.rs），规则层驱动器需把编排收敛成纯函数（可测面），或测试直接驱动 app `GameApp`（稍重但更真）。

### 方案 B · app 壳级 e2e（GameApp 驱动，最"真"）
**载体**：`crates/ti-app/tests/adversarial_game_e2e.rs`，用 `GameApp`（如 `app_ai_turn_e2e` 现有模式）
喂 N 个 AI 席，逐回合调 `GameInput`（摇骰/确认），AI 决策经 `board.rs:516 ai_policy_v2` 自动落格。
- **优点**：走**真实 app 编排**（场景态/落格分叉/AI toast 全链路），最接近"真玩一局"；复用 `app_ai_turn_e2e` 接线。
- **代价**：比 A 重（GameApp 构造 + 场景推进 + 多席热座编排），跑一局稍慢（但仍 headless ms~s 级）。
- **风险**：多 AI 席在 app 壳的热座轮转 + 破产剔除 + 终局判定编排较复杂（现 `app_seats_e2e` 只验单动作）；需把"整局循环"在测试里 loop 到终局。

### 方案 C · 引入学习型 AI（MCTS/自我对弈）——不推荐
**载体**：训练 AI 对手替代规则式 `ai_policy_v2`。
- **判定**：本项目 AI = **业务规则驱动**（territory.md §1.2 攻占/升级公式 = 设计意图 + 数据可调），
  引 MCTS/强化学习 = 违背「调平衡不改代码」纪律（EXP-196）+ 引入不可复现（学习态）+ 远超游戏规模
  （31 格环 + 兵力模型 = 规则式决策已够）。**除非用户要"会学习的对手"，否则不做。**

## 推荐路径
1. **先 A**（规则层 headless 驱动器 + 5 类断言，1 个新测试文件，复用现成 rules 函数）= "AI 自动对抗真实 E2E" 的最小可信落地 + 机器判据。
2. A 验证对局性质后，**按需升 B**（app 壳级真链路，若用户要"真玩一局"的视觉/场景面）。
3. C 不做（除非要学习型对手）。

## 待你拍板
- 选 A / B / A→B / 其它？
- AI 席数（2 最小 / 4 满配 `max_ai_seats` / 2+玩家混排）？
- 断言面要不要含「胜率分布」（多 seed 统计 AI 胜率，偏性能/统计）还是只要「对局性质」（确定性/守恒/可达/不卡死，偏正确性）？

## 方案 A 落地实测（2026-11-20，`crates/ti-core/tests/adversarial_e2e.rs`）

A 驱动器已实现（纯规则层，复用 `BoardSession::step` + `ai_policy_v2` + `TerritoryApply` + `assault`
+ `bankrupt`/`victory_seat_active`，零新规则代码）。5 条机器判据全过（test 998/0）：
**确定性**（同 seed 同席数 ⇒ 同终局快照逐位复现）/ **金有界**（≤ `gold_max`）/ **不无限拉锯**
（`rounds` ≤ 兜底上限）/ **多席不串**（快照长度 = 席数）/ **回合上限生效**。

### 探针坐实的真业务缺口（待拍板，不擅自改 balance）
**对局不自然收敛到唯一胜者**：400 回合多数 seed 仍 2 席存活（实测 seed=2026：3 席全活，
金 = [32679, 6488, 72078]，`winner=None`）。根因 = ① `apply_collect` 每圈产出金无衰减（金单调
累积，400 回合打满不了 `gold_max=999999` 上限破产）② 破产判据单用金耗尽（兵力经 `assault`
1/3 驻军损耗恒 >0 不淘汰）。**A 测试只验驱动器对局性质（不卡死/确定性/守恒）；「对局可收敛」
= 平衡调参缺口，3 个候选处置（用户拍板后落）：**
- **A1** BOSS 终结（`boss_min_laps` 满圈后 BOSS 现身 = 终局判据，对局有自然终点）
- **A2** 金衰减（每圈产出递减 / 维护费，让金可耗尽 ⇒ 破产可触发）
- **A3** 破产扩判据（金=0 **或** 兵力=0 出局，兵力 1/3 损耗累积终归零）
