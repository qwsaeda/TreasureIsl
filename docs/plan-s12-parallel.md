# S12 — 模块化并行开发计划（WBS 草案，S12.0 起逐包展开）

**定位**：S12 的 WBS。按 `docs/process/05_module_parallel.md` 一轮 4 步
（契约冻结 → 多包并行 → 集成联调 → 记账）执行；包划分与白名单以
`plan/packages.yaml`（机读）+ `docs/process/05_module_parallel.md` §3（人读）为准。
本文只定**轮次结构与各包目标**，不重复白名单（单一事实源 = packages.yaml）。

## 1. 轮次结构

```
W1（本轮）
  S12.0  契约冻结轮（主线程）：c1/c2/c3 置 frozen + packages.yaml 7 包锁定
         + 逐包任务书（03_templates §7）+ baseline-s12 标签
  S12.1  P1 战斗域        （并发会话/分支 task/S12.1，deps D）
  S12.2  P2 经济域        （并发会话/分支 task/S12.2，deps D）
  S12.3  P3 地图卡牌域    （并发会话/分支 task/S12.3，deps D）
  S12.4  P4 流程菜单域    （并发会话/分支 task/S12.4，deps D）
  S12.5  P5 存档+RNG 域  （并发会话/分支 task/S12.5，deps D+P1–P4 状态结构稳定）
         ⚠ P5 依赖 P1–P4 的 domain 结构稳定：若 P1–P4 在集成前改了公共 struct，
           P5 打回（05_module_parallel.md §7），不强行开
  S12.6  集成联调轮（主线程）：按依赖序合并 task/* 分支 → 全 workspace build + test
          → P3-3 16 步 → gate 20 项全绿 → 逐包回写 evidence + 台账 verified

W2（下一轮，P5 绿后开）
  S12.7  P6 表现层包（deps P1–P5 + c3 frozen）：scenes/widgets/touch/gfx
  S12.8  集成联调轮（P6 合入 + 全流程回归）
```

## 2. 各包目标（任务书正文按 `03_templates.md` §7 模板逐包生成）

| 包 | 本轮目标 | 包内验收（不依赖他人产物） |
|---|---|---|
| D 数据 | balance.json 段按域补齐 key（economy/combat/trade/build/cards/messages/save/selection/ui）；validate_data.py 域校验 | `cargo test -p ti-core --test data_tables` + `python scripts/validate_data.py` |
| P1 战斗 | `rules/battle.rs` 会话 + 结算按 [`business/combat.md`](../business/combat.md) 规则表全覆盖 | `cargo test -p ti-core --test rules_battle` + 越界 grep = 0 |
| P2 经济 | `rules/{trade,build}.rs` 买卖/银行/升级按 [`business/economy.md`](../business/economy.md) | `cargo test -p ti-core --test rules_trade --test rules_build` + 越界 grep = 0 |
| P3 地图卡牌 | `rules/{main_map,cards,cards_draw}.rs` 移动/据点分派/抽牌按 [`business/progression.md`](../business/progression.md) | `cargo test -p ti-core --test reference_diff_cards` + 越界 grep = 0 |
| P4 流程菜单 | `rules/{menu,selection,messages,main_loop,render}.rs` 场景流/消息分页按 [`business/07_ui_flow.md`](../business/07_ui_flow.md) + [`business/loop_core.md`](../business/loop_core.md) | `cargo test -p ti-core --test rules_messages` + 越界 grep = 0 |
| P5 存档+RNG | `rules/{saveload,saveload_json,rng}.rs` 存档 v2 + 可复现随机按 [`business/persistence.md`](../business/persistence.md) | `cargo test -p ti-core --test rules_saveload --test rules_saveload_json --test save_migration` + 越界 grep = 0 |
| P6 表现层（W2） | `ti-app/scenes/*` + `widgets` + `touch` + `gfx` 按 [`business/07_ui_flow.md`](../business/07_ui_flow.md) 可测规则表 12 条 | `cargo test -p ti-app --test game_app_flow --test e2e_playthrough` + 越界 grep = 0 |

## 3. 台账登记

S12.0–S12.6 在 `plan/ledger.yaml` 各占一行（`deps` 指向 packages.yaml 的 `depends_on`，
机器判定见 gate 第 16 项 M1（历史时点））。S12.7/S12.8 在 W2 开轮前再登记。
并行轮期间 ledger 计数会漂移（多包并发回写 verified）——由集成轮统一对账，
不是漂移 bug（EXP-038 只约束「单任务 verified 行不得被他人改写」）。

## 4. 停机条件

W1 停机 = S12.6 集成轮 gate 20 项全绿 + P3-3 16 步 0 失败 + P1–P5 全部 verified；
P6（W2）开轮判据 = S12.6 绿 + c3 契约 frozen。
