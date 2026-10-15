# 战斗系统（Combat）

## 设计意图

战斗是**回合制、单场决胜**的高潮场景：玩家 vs 一个 BOSS，靠先前积累的金币/建造
（提升 atk/hp）与卡牌事件（临时增益）决定胜负。战斗刻意做得很短（几十回合内
分胜负），避免拉锯战。数值做「可调」，全部落在 `balance.json`。

## 参数（在 `assets/data/balance.json`）

```jsonc
"combat": {
  "player_first": true,          // 玩家先手
  "player": { "hp": 2000, "atk": 60 },   // 玩家基础单位（balance.combat.player）
  "damage": {
    "model": "atk_plus_flat",    // 伤害模型（见下）
    "flat": 1,                   // 最小伤害
    "atk_coef": 1.0,             // 伤害 = atk * atk_coef + rng_offset
    "rng_span": 3                // 随机浮动幅度
  },
  "boss_tiers": [                // BOSS 档（hp / atk）
    { "tier": 0, "hp": 50000, "atk": 10 },
    { "tier": 1, "hp": 100000, "atk": 12 },
    { "tier": 2, "hp": 150000, "atk": 14 },
    { "tier": 3, "hp": 200000, "atk": 16 },
    { "tier": 4, "hp": 300000, "atk": 20 }
  ]
}
```

## 伤害模型（设计选择，非逆向）

```
damage = round(atk * atk_coef) + rand_offset()    // rand_offset ∈ [0, rng_span]
下限   = flat                                    // 保证每回合至少掉 flat 血，避免死循环
```

- **确定性**：随机来自 `rules::rng::GameRng`（可播种 PCG，`rand_pcg`，S8.7 落地），同 seed 同操作序列
  ⇒ 同结果（可复现、可回放、可测试）。禁用 `thread_rng`。
- **先手优势**：玩家固定先手（`player_first`），双方伤害独立取随机。

## 回合流程

```
玩家确认攻击
  → 玩家按 damage 模型结算对 BOSS 的伤害
  → 若 BOSS.hp == 0 → Victory，战斗结束
  → 否则 BOSS 按 damage 模型结算对玩家的伤害
  → 若 player.hp == 0 → Defeat，战斗结束
  → 否则回合数 +1，回到等待输入
```

## 胜负判定

| 条件 | 结果 |
|---|---|
| `boss_hp == 0` | `Victory`（同回合判玩家胜） |
| `player_hp == 0` 且 `boss_hp > 0` | `Defeat` |
| 超过 `balance.combat.max_rounds` | 仍按 HP 高者判（同回合为 `Defeat`） |

## 可测试规则（每条 → 单测名）

| 规则 | 单测（已落地于 `crates/ti-core/tests/rules_battle.rs`） |
|---|---|
| 同 seed + 同操作序列 ⇒ 同结果 | `combat_same_seed_same_result`（`session_deterministic`） |
| 玩家先手，伤害顺序固定 | `combat_player_first`（`combat_victory_no_counterattack` 覆盖先手顺序） |
| 每回合玩家至少掉 `flat` 血 | `combat_min_damage_floor` |
| BOSS.hp==0 立即 Victory，不再反击 | `combat_victory_no_counterattack` |
| player.hp==0 判 Defeat | `combat_defeat_when_player_hp_zero` |
| max_rounds 兜底判胜负 | `combat_max_rounds_tiebreak` |
| 越界 tier ⇒ 无效战斗 | `combat_invalid_tier_rejected`（`session_invalid_tier`） |

## 数值平衡指引（非规则，供调参参考）

- 一场战斗目标 10–40 回合：玩家 atk 需在 `boss_hp / 40` 量级；
- BOSS 档位每级 hp ≈ ×1.5–2，atk 小幅递增——让高等级靠建造/卡牌硬堆；
- `rng_span` 越大战斗波动越大，3 是个保守起点。

> 这些是**设计意图**，具体值在 `balance.json`，改 JSON 不改本文件。