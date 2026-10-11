# 地图 / 据点 / 卡牌事件（Progression）

## 设计意图

地图是**探索的舞台**：等距网格 + 摄像机，玩家在格间移动，踩到据点/事件格触发
交互。地图推进玩家与 BOSS 相遇。地图与地形数据来自真实资源（`glmapm0*.Map`，
格式见 `docs/formats/`），但**玩法规则（移动/触发/卡牌）是本目录定义的设计**。

## 地图结构

| 项 | 设计 |
|---|---|
| 网格 | 每地图 `grid_w × grid_h`（来自资源，实测 20×20 / 20×20 / 15×15） |
| 投影 | 等距：`iso_x = 32*(row+col)`，`iso_y = 16*(row + grid_w - 1 - col)`（实测，参照 facts F-307，逆向产物非权威源） |
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
  "deck": [                       // 卡池（示例节选）
    { "id": "treasure", "weight": 3, "effect": { "gold": +200 } },
    { "id": "ambush",   "weight": 2, "effect": { "hp": -10 } },
    { "id": "supply",   "weight": 2, "effect": { "gold": +50, "hp": +5 } }
  ],
  "event_chance": 0.35,           // 移动触发卡牌的概率
  "hand_size": 1                 // 一次抽几张（balance.json 当前值 = 1）
}
```

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

## 与实现的接口

- 玩法：`rules::main_map`（`MapSession`）、`rules::cards`（`CardEngine`）（S8.3.c 重写中）；
- 地形/格数据：真实资源解析（`ti-formats`，保留）；
- 摄像机：表现层（`ti_platform::gfx` 的 `FramePresenter`/`Viewport` 承担缩放/呈现；`ti-app` 主地图渲染按上述投影公式绘制），玩法层只管「玩家在哪一格」。