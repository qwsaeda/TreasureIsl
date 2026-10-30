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
    { "tier": 0, "hp": 2400,  "atk": 10 },
    { "tier": 1, "hp": 4800,  "atk": 12 },
    { "tier": 2, "hp": 7200,  "atk": 14 },
    { "tier": 3, "hp": 9600,  "atk": 16 },
    { "tier": 4, "hp": 12000, "atk": 20 }
  ]
}
```

## 建造加成（成长机制，`docs/business/economy.md` §建造）

> S21.2 起扩展为「建筑 + 船只」双成长轴：总 atk/hp = 建筑 Σ + 船等级加成
> （`balance.boat.levels[boat_level].atk/hp`，**累计值直接加**，不经 `× level` 公式；
> 规格单一事实源 = `economy.md` §船系统，接线 S21.10）。

玩家 atk/hp = 基础值（`balance.combat.player`）+ 已建建筑加成。
加成数值在 `balance.build.buildings[].atk_bonus` / `hp_bonus`（每级加成，
`BuildCfg::atk_bonus_at(id, level)` = `atk_bonus × level`，`Σ` 全部建筑）。

```jsonc
"build": {
  "buildings": [
    { "id": "Castle", "atk_bonus": 10, "hp_bonus": 200, ... },
    { "id": "Investment", "atk_bonus": 5, "hp_bonus": 100, ... },
    { "id": "Bank", "atk_bonus": 0, "hp_bonus": 50, ... },
    { "id": "Fort", "atk_bonus": 8, "hp_bonus": 150, ... }
  ]
}
```

- 无建筑 → 加成 0，退化为 base 战斗（与原行为一致）；
- 卡牌 `Damage` 事件扣 HP 走同一通道（`rules::cards::CardState.hp` 暂存，
  `start_battle` 时并入玩家 hp）。

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
玩家选择动作（攻击 / 道具 / 防御，见 §战斗动作）
  → 玩家动作结算（攻击 = damage 模型；道具 = 该道具效果；防御 = 本回合减伤）
  → 若 BOSS.hp == 0 → Victory，战斗结束
  → 否则 BOSS 按 damage 模型结算对玩家的伤害（烟雾弹落空 / 防御减伤在此生效）
  → 若 player.hp == 0 → Defeat，战斗结束
  → 否则回合数 +1，回到等待输入
```

## 战斗动作（S27.7 起：攻击 / 道具 / 防御）

> 用户 2026-10-29 定案：战斗不再是「点哪都是普攻」，每回合从三种动作里选一种。
> 数值单一事实源 = `assets/data/balance.json → combat.items` / `combat.defend_reduction_pct`；
> 规则实现 = `rules::battle::BattleSession::step_with(CombatAction)`（`step()` ≡ `step_with(Attack)`）；
> 效果词汇 = `domain::combat::ItemEffect`（规则只消费效果，不查表）。

| 动作 | 语义 | 备注 |
|---|---|---|
| 攻击 `Attack` | 按 §伤害模型 结算对敌伤害 | 旧行为，默认动作 |
| 防御 `Defend` | 本回合敌方反击伤害 ×（1 − `defend_reduction_pct`） | 减值只作用于**当回合**反击，不跨回合累积 |
| 道具 `Item(effect)` | 结算该道具效果，**不消耗随机数** | 道具不改变 `GameRng` 状态 ⇒ 同 seed 同操作序列仍同结果 |

### 道具表（4 种，`balance.combat.items[]`）

| id | 名称键 | 效果 `effect` | 携带上限 `carry` | 数值 | 设计意图 |
|---|---|---|---|---|---|
| `keg` | `combat.item.keg` | `damage 300` | 3 | 固定 300 | 稳定爆发，不吃 atk/随机（低成长玩家的兜底伤害） |
| `medkit` | `combat.item.medkit` | `heal_pct 25` | 3 | 回 25% 上限 HP（不超上限） | 拉长回合数，配合防御打消耗 |
| `rum` | `combat.item.rum` | `atk_buff {pct:50, rounds:3}` | 2 | 3 回合内 atk ×1.5 | 增益铺垫，收益随 atk 成长放大 |
| `smoke` | `combat.item.smoke` | `stagger` | 2 | 敌方下一次反击落空 | 关键回合免伤（每个只能挡 1 次） |

- **结算顺序**（每回合固定）：① 玩家动作 → ② 敌 HP 归零即胜（不再反击）→ ③ 敌方反击（`stagger` 消耗 1 次 / `Defend` 减伤 / `atk_buff` 已在①生效）→ ④ 回合末 `atk_buff.rounds` −1（到 0 失效）。
- **buff 时效**：`atk_buff` 从**使用当回合**起算，覆盖当回合的攻击；`rounds` 递减发生在回合末。
- **携带与获得**：携带上限见 `carry`；获取途径 = 贸易港「军械」购买 + 卡牌事件（S28 领地副本同样消耗同一背包）。背包随存档持久化（`persistence.md`）。
- **不做的**：不做冷却/能量条（道具靠数量约束，保持可测）；不做敌方用道具（敌方只有普攻，回合演出另算）。

## 胜负判定

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
| 火药桶伤害固定 300，不吃 atk/随机 | `combat_item_damage_is_fixed` |
| 医疗包回 25% 上限、不超上限 | `combat_item_heal_capped` |
| 朗姆酒 3 回合 atk×1.5，到期失效 | `combat_item_atk_buff_expires` |
| 烟雾弹只挡 1 次反击 | `combat_item_stagger_once` |
| 防御当回合反击减半 | `combat_defend_halves_counter` |
| 道具不消耗随机数（同 seed 同轨迹） | `combat_item_keeps_rng_state` |

## 数值平衡指引（非规则，供调参参考）

- 一场战斗目标 10–40 回合：玩家 atk 需在 `boss_hp / 40` 量级；
- BOSS 档位每级 hp ≈ ×1.5–2，atk 小幅递增——让高等级靠建造/卡牌硬堆；
- `rng_span` 越大战斗波动越大，3 是个保守起点。

> 这些是**设计意图**，具体值在 `balance.json`，改 JSON 不改本文件。