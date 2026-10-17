# 地图 / 据点 / 卡牌事件（Progression）

## 设计意图

地图是**探索的舞台**：等距网格 + 摄像机，玩家在格间移动，踩到据点/事件格触发
交互。地图推进玩家与 BOSS 相遇。地图与地形数据来自真实资源（`glmapm0*.Map`，
格式见 `docs/formats/`），但**玩法规则（移动/触发/卡牌）是本目录定义的设计**。

## 地图结构

| 项 | 设计 |
|---|---|
| 网格 | 每地图 `grid_w × grid_h`（来自资源，实测 20×20 / 20×20 / 15×15） |
| 投影 | 等距：`iso_x = 32*(row+col)`，`iso_y = 16*(row + grid_w - 1 - col)`（实测值，出处 `legacy/docs/facts/map.md` F-307；此处内联为业务公式，不依赖原版产物） |
| 摄像机 | 跟随玩家，按投影中心计算，整格对齐（不做平滑插值，保证确定性；实现落点 `ti_platform::gfx::FramePresenter`/`Viewport`，见 impl/03_map_system.md §5） |
| 地形类型 | 由资源 terrain 记录给出（水/陆/障碍），移动合法性查表 |

## 移动规则

```
方向键 / 滑动 → 目标格
  若 越界 或 不可通行 → 不移动（不消耗回合）
  否则 移动一格 → 触发「落格结算」（见下）
```

- **一格一步**，每步触发一次结算，保持回合制节奏；
- 不可通行判定来自地形表（不进代码，进 `data/maps.json`）。

## 据点与事件（落格结算）

```
移动到新格
  → 若该格是据点 → 打开据点菜单（贸易/建造/信息）
  → 否则        → 掷骰判定是否触发卡牌事件（概率见 balance.cards.event_chance）
  → 遇到 BOSS 格 → 进入战斗场景（combat.md）
```

## 卡牌事件系统

卡牌是**轻量 roguelite 式事件**：抽一张牌，立刻生效（多为一次性增益/损失）。

> 以下为**示例节选**（3 张），完整数值（5 张卡，deck 1..5）以 `assets/data/balance.json` 为准。

```jsonc
"cards": {
  "deck": [                       // 卡池（示例节选；id/effect schema 与 balance.json 一致）
    { "id": 1, "name_key": "cards.1", "weight": 3, "effect": { "kind": "gold",   "value": 20 } },
    { "id": 2, "name_key": "cards.2", "weight": 2, "effect": { "kind": "heal",   "value": 15 } },
    { "id": 3, "name_key": "cards.3", "weight": 3, "effect": { "kind": "damage", "value": 10 } }
  ],
  "event_chance": 0.35,           // 移动触发卡牌的概率
  "hand_size": 1                  // 一次抽几张（balance.json 当前值 = 1）
                                     // FIND-029 设计裁决：单卡 roguelite 事件（抽一张立即生效），非占位；
                                     // 调 >1 需同步 main_map.rs draw_one 的 n 参数（每步抽 n 张逐一结算）
}
```

> 上表为示例节选（3 张），完整 5 张卡（id 1..5）以 `assets/data/balance.json` `cards.deck` 为准。
> `effect` schema 为 `{ kind, value }`（非 `{ "gold": +200 }` 旧设计占位），见 `docs/impl/06_cards_events.md`。

### 抽牌（可复现）

用 `GameRng` 按权重抽卡（可播种 PCG，`rand_pcg`，S8.7 落地，走确定性序列）：
同 seed + 同移动序列 ⇒ 同一串卡牌结果。**禁用 `thread_rng`**。

### 效果结算

- `gold`/`hp` 增减后立即做区间钳制（`0 ≤ gold ≤ gold_max`、`0 ≤ hp ≤ hp_max`）；
- 效果码采用「加法为主、覆盖为辅」，便于扩展而不改结算器。

## 可测试规则

| 规则 | 单测 |
|---|---|
| 越界/不可通行不移动、不消耗回合 | `map_blocked_step_is_noop`（待 S8.3.c 补单测落地） |
| 移动一格必触发一次落格结算 | `map_step_triggers_settle_once`（待 S8.3.c 补单测落地） |
| BOSS 格进入战斗 | `map_boss_tile_enters_combat`（待 S8.3.c 补单测落地） |
| 同 seed 同路径 ⇒ 同卡牌串 | `cards_same_seed_same_sequence`（待 S8.3.c 补单测落地） |
| 效果后 HP/金钳制在合法域 | `cards_effects_clamped`（待 S8.3.c 补单测落地） |
| 权重抽卡分布合理（单测只断言「可复现」，分布靠统计脚本） | `cards_weighted_pick_reproducible`（待 S8.3.c 补单测落地） |

## 跨图推进（地图解锁 `unlocked_maps`）

游戏共 3 图 m01/m02/m03，难度按 `boss_tier` 升序（m01=0 → m02=1 → m03=2，BOSS 更强）。
推进规则 = **击败当前图 BOSS 才解锁下一图**，非开局三图齐选。

| 阶段 | 规则 |
|---|---|
| 初始解锁 | 仅 `m01`（`unlocked_maps = [m01]`） |
| 推进 | 击败当前图 BOSS（`combat` 结果 = Player 胜）→ 解锁序列下一图，追加进 `unlocked_maps` |
| 序列 | m01 →(击败)→ 解锁 m02 →(击败)→ 解锁 m03；m03 无下一图 |
| 通关 | 击败 m03 的 BOSS = 全图通杀 = 游戏通关（本局结束） |
| 失败 | HP 归零 / 超 `combat.max_rounds` = 失败，**不解锁**；回主菜单后可**重试**当前图 |
| 选择约束 | SelectMap 显示**全部 3 图**，锁定项置灰 + 深色「锁」块（`08_layout_design.md` §锁块）；Prev/Next 可自由导航（含锁定位），仅 **Confirm** 经 `map_unlocked` 校验拒绝（`Sfx::Invalid`，选择态不提交） |

### 与存档一致（`persistence.md`）

- 规则层存档（`saveload`/`saveload_json` 的 `SaveData.unlocked_maps` 字段）**当前无生产写入端**：4 处构造恒 `Vec::new()`，仅测试 fixture 手工置值——解锁链的**唯一事实源 = app 侧 `GameApp.unlocked_maps`**（`ProgressionState` 驱动）。该字段为预留扩展位（接规则层存档需 `capture_state` 增 `unlocked` 入参，c1 契约变更，见台账 S15.2 残留）。
- **app 侧存档槽（`domain::persistence::SaveSlot`）不持解锁链字段**（v2 schema 稳定，不迁移）：读档按存档 `map_slot` 的「最小前置链」恢复（选中 m(i) ⟹ m(0)..m(i-1) 已击败，`ProgressionState::min_progression_to_select`）；旧档（高图直选/多解锁值）按链回读，无需迁移。

### 可测试规则

| 规则 | 单测 |
|---|---|
| 初始仅 m01 解锁 | `progression_initial_unlocks_only_m01` |
| 击败 m01 → 解锁 m02（m03 仍锁） | `progression_defeat_m01_unlocks_m02_only` |
| 击败 m02 → 解锁 m03 | `progression_defeat_m02_unlocks_m03` |
| 击败 m03 = 通关、无下一图 | `progression_defeat_m03_clears_game` |
| 失败（未击败）不解锁 | `progression_loss_does_not_unlock` |
| 解锁链单调（只增不降，m01 恒解锁） | `progression_unlocks_monotonic` |
| 读档链恢复：选中 m(i) 的最小前置 = 击败 m(0)..m(i-1) | `min_progression_to_select_chain` |
| 解锁集重建（战斗结算 / 旧档异常集忠实保留 / 全解锁幂等） | `from_unlocked_feeds_boss_defeat_step`、`from_unlocked_legacy_anomaly_is_faithful`、`from_unlocked_all_then_m03_clears` |

> 实现落点：`crates/ti-core/src/rules/progression.rs`（`ProgressionState`：`initial_unlocked` /
> `unlocked` / `on_boss_defeated` / `is_cleared` / `can_select` / `from_unlocked` / `min_progression_to_select`）。
> **app 层已接线**（`crates/ti-app/src/game_app.rs`）：开档 `unlocked_maps = initial_unlocked()`（仅 m01，非全开）；
> `battle_step` 胜利钩 = 击败当前图 BOSS → 规则层结算解锁下一图（幂等，失败不加）；`restore_save` 按链恢复。
> 验收：`crates/ti-app/tests/app_progression_wiring.rs`（4 条：初始仅 m01 / 胜解锁+幂等 / m01→m02 全链路 / 读档链恢复+gold/位置恢复）。
> 演示：`crates/ti-core/examples/cross_map_demo.rs`（打满 3 图）。
> **残留（S15.2 登记）**：① 通关判定 `is_cleared` 在 app 游玩层暂无表现（击败 m03 后 Back 回主菜单，无通关画面/提示）；
> ② 规则层存档 `SaveData.unlocked_maps` 无生产写入端（见上「与存档一致」）。

## 与实现的接口

- 玩法：`rules::main_map`（`MapSession`）、`rules::cards`（`CardEngine`）（S8.3.c 重写中）；
- 地形/格数据：真实资源解析（`ti-formats`，保留）；
- 摄像机：表现层（`ti_platform::gfx` 的 `FramePresenter`/`Viewport` 承担缩放/呈现；`ti-app` 主地图渲染按上述投影公式绘制），玩法层只管「玩家在哪一格」。