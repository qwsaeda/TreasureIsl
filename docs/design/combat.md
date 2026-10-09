# 战斗与卡牌系统设计

本文档涵盖：战斗系统（BOSS 档/伤害公式/回合制流程/胜负判定）+ 卡牌事件系统（目录/效果/触发/映射）。

---

## 1. 战斗系统

### 1.1 BOSS 档 HP 表

| 档（tier） | HP |
|---|---|
| 0 | 50,000 |
| 1 | 100,000 |
| 2 | 150,000 |
| 3 | 200,000 |
| 4 | 300,000 |
| > 4 | 0（无效档，战斗立即结束，玩家胜） |

```
oracle_boss_hp(tier):
    match tier:
        0 → 50_000
        1 → 100_000
        2 → 150_000
        3 → 200_000
        4 → 300_000
        _ → 0
```

`tier > 4` → HP = 0 → 视为 BOSS 已死 → 玩家立即获胜，0 回合。

### 1.2 战斗单位

```
struct Unit {
    hp: u32,     // 当前 HP
    atk: u32,    // 攻击力
}
```

| 单位 | HP | atk |
|---|---|---|
| 玩家 | 由角色状态决定 | 由角色状态决定 |
| BOSS | `oracle_boss_hp(tier)` | 10（固定） |

### 1.3 伤害公式

```
damage(seed, atk) = (seed × 2654435761) % (atk + 1) + 1
```

- `atk = 0` → 恒 1（`% 1 = 0` + 1）。
- `seed = 0` → 恒 1。
- 纯确定性（同 seed 同 atk 同结果），无 rand 库。
- 常数 `2654435761`（K = 0x9E3779B1，Knuth 乘子）。

> 该公式为 oracle 占位（推测），exe 实际 RNG 归 `docs/design/persistence.md` §RNG 对齐。

### 1.4 回合制流程

每回合双方各行动一次，**玩家先手**：

```
oracle_battle_round(seed, player, enemy):
    // 1. 玩家行动
    player_dmg = damage(seed, player.atk)
    enemy_hp_after = enemy.hp - player_dmg          // saturating_sub

    // 2. 敌方行动（seed+1 避免同回合双方伤害相同）
    enemy_dmg = damage(seed + 1, enemy.atk)
    player_hp_after = player.hp - enemy_dmg         // saturating_sub

    // 3. 胜负判定（先判敌方归零 = 玩家优势）
    if enemy_hp_after == 0:
        winner = Player
    else if player_hp_after == 0:
        winner = Enemy
    else:
        winner = None                              // 继续

    return (player_hp_after, enemy_hp_after, winner)
```

### 1.5 整场战斗

```
oracle_full_battle(seed, player, boss_tier):
    boss_hp = oracle_boss_hp(boss_tier)
    if boss_hp == 0:
        return (空轨迹, Player, 0)                 // 立即结束

    enemy = Unit { hp: boss_hp, atk: 10 }
    player_cur = player
    enemy_cur = enemy
    trace = []
    round_seed = seed
    round_num = 0

    for _ in 0..1000:                               // 最大 1000 回合
        round_num += 1
        (ph, eh, outcome) = oracle_battle_round(round_seed, player_cur, enemy_cur)
        trace.push((ph, eh))
        player_cur = Unit { hp: ph, atk: player_cur.atk }
        enemy_cur = Unit { hp: eh, atk: enemy_cur.atk }
        round_seed = round_seed + 1

        if outcome != None:
            return (trace, outcome, round_num)

    return (trace, Enemy, round_num)                // 1000 回合未分胜负 → 敌方胜
```

### 1.6 胜负判定优先级

| 情况 | 胜方 | 说明 |
|---|---|---|
| `enemy.hp == 0` | Player | BOSS 被击败 |
| `player.hp == 0` | Enemy | 玩家被击败 |
| 双方同时 0 | Player | 玩家优势（先判敌方归零） |
| 1000 回合未结束 | Enemy | 超时判负 |

### 1.7 战斗会话

```
struct BattleSession {
    seed: u32
    player_hp: u32
    player_atk: u32
    enemy_hp: u32
    enemy_atk: u32
    round: u32
    outcome: Option<Side>
}
```

| 方法 | 语义 |
|---|---|
| `new(seed, player, boss_tier)` | 构造（`enemy_hp` 由 `oracle_boss_hp` 初始化，`enemy_atk = 10`） |
| `boss_hp()` | 当前 BOSS HP |
| `round()` | 当前回合数 |
| `outcome()` | 战斗结果（`None` = 未结束） |
| `step()` | 单步回合，返回 `(player_hp, enemy_hp, Option<Side>)` |
| `finish()` | 整场战斗，返回 `(轨迹, 胜方, 回合数)` |

`new` 时 `boss_tier > 4` → `enemy_hp = 0` → `outcome = Some(Player)`，`round = 0`。

### 1.8 战斗场景对象

```
new vs 对象 (0x474 bytes)
sub_572E4 构造:
    sub_43B8C 构造基类（设 vtable）
    *a1 = &off_73A2C              // vs 场景 vtable
    a1[256..261] = 0              // 6 个战斗状态字段清零
    a1[277..284] = 0              // 8 个兵种/HP 字段清零
    a1[255] = 1                   // 战斗标志
    a1[276] = 1                   // 第二标志
```

### 1.9 战斗事件流

| 步骤 | 动作 | 状态变化 |
|---|---|---|
| 1 | 主地图事件码 6 触发 | `sub_31408` 判定 |
| 2 | 建 vs 对象 | `sub_572E4` + `ui.pak+vs.Map` + `sub_4BB34` 初始化 |
| 3 | 进入战斗态 | `a1+716 = 25` |
| 4 | 玩家点「攻击」 | `BattleSession::step()` 单回合 |
| 5 | 循环直到 `outcome != None` | 渲染 HP/回合数/胜方 |
| 6 | 胜 → 播 `victory.wav` | `advance(2)` → 状态 0 |
| 7 | 败 → 失败面板 | `advance(2)` → 状态 0 → 回主菜单 |

### 1.10 兵种

5 种兵种/类型（`troop_all.pcx` / `ship_all.pcx` 图鉴）：

| 槽位偏移 | 宽字符串常量 | 推测语义 |
|---|---|---|
| +0 | `asc_7E014` | 通用（主兵种名） |
| +12 | `aAstx3t` | 兵种 A（陆军） |
| +24 | `aKq` | 兵种 B（海军） |
| +36 | `asc_7DFF8` | 兵种 C（空军/炮兵） |
| +48 | `aGy` | 兵种 D（骑兵/快速） |
| +60 | `byte_7DFE4` | 兵种 E（重甲/攻城） |

exe 内无独立兵种数值表，推测兵种 HP/atk 由 BOSS 档决定。

---

## 2. 卡牌/事件系统

### 2.1 卡牌效果类型

`sub_3D1B4` 效果码 1..4 → 语义（推测映射）：

| 效果码 | Rust 枚举 | 行为 |
|---|---|---|
| 1 | `CardEffect::Damage(n)` | `hp = hp.saturating_sub(n)` |
| 2 | `CardEffect::Heal(n)` | `hp = hp.saturating_add(n)` |
| 3 | `CardEffect::Gold(n)` | `gold = gold.saturating_add(n)` |
| 4 | `CardEffect::Move` | 不改 state（地图移动归主地图模块） |
| 0 | `CardEffect::Unknown` | 无效果 |

### 2.2 占位卡牌目录

```
CARD_CATALOG (5 张卡):
    CardDef { id: 1, effect: Damage(10) }
    CardDef { id: 2, effect: Heal(15) }
    CardDef { id: 3, effect: Gold(20) }
    CardDef { id: 4, effect: Move }
    CardDef { id: 5, effect: Unknown }
```

> 效果数值占位，需对照 `cards.lpk` 的 `cards\cards.ini` 实际内容确认。

### 2.3 卡牌状态

```
struct CardState {
    hp: u32,      // 当前 HP（Damage/Heal 目标）
    gold: u32,    // 当前金（Gold 目标）
}
```

### 2.4 事件 id → 卡牌映射

```
event_to_card(event_id, catalog):
    if catalog is empty: return None
    if event_id == 0: return None

    if event_id <= catalog.len():
        return catalog[event_id - 1]
    else:
        return catalog[(event_id - 1) % catalog.len()]     // 循环映射
```

示例（5 张卡）：

| event_id | 映射索引 | 卡牌 ID |
|---|---|---|
| 1 | 0 | 1 |
| 2 | 1 | 2 |
| 5 | 4 | 5 |
| 6 | (6-1)%5 = 0 | 1 |
| 25 | (25-1)%5 = 4 | 5 |

### 2.5 事件生成

主地图帧更新 case 3（`sub_2101C` 行 2841-2851）：

```
// 1. 生成原始 event_id
raw_id = rand() % 25              // 0..24
event_id = raw_id + 1             // 1..25

// 2. 重映射修正
if event_id ∈ {4, 13, 14, 15}:
    event_id = event_id + 4        // 4→8, 13→17, 14→18, 15→19

// 3. 写全局
MEMORY[0x100081170] = event_id

// 4. 卡牌库存门控
card_state = sub_313D8(ctrl + 72)
if 卡牌库存 >= 10:
    装载 ui.pak + card.Map + bigcard.Map
    a1+716 = 4                     // 卡牌态
```

### 2.6 事件 id 重映射表

| 原始 id | 重映射后 |
|---|---|
| 4 | 8 |
| 13 | 17 |
| 14 | 18 |
| 15 | 19 |

其余 id 不变。

### 2.7 全部 25 个 event id 清单

| event_id | 来源 | 处理路径 |
|---|---|---|
| 1..3 | 直接 | ✓ |
| 4 | 重映射 → 8 | ✓ |
| 5..7 | 直接 | ✓ |
| 8 | 由 4 重映射 / 直接 | ✓ |
| 9..12 | 直接 | ✓ |
| 13 | 重映射 → 17 | ✓ |
| 14 | 重映射 → 18 | ✓ |
| 15 | 重映射 → 19 | ✓ |
| 16 | 直接 | ✓ |
| 17 | 由 13 重映射 | ✓ |
| 18 | 由 14 重映射 | ✓ |
| 19 | 由 15 重映射 | ✓ |
| 20..25 | 直接 | ✓ |

全部 25 个 id 都有处理路径（`sub_32960` 统一存储），覆盖率 100%。

### 2.8 卡牌效果应用

```
apply_card(card, state):
    match card.effect:
        Damage(n) → state.hp = state.hp.saturating_sub(n)
                     return Applied(Damage(n))
        Heal(n)   → state.hp = state.hp.saturating_add(n)
                     return Applied(Heal(n))
        Gold(n)   → state.gold = state.gold.saturating_add(n)
                     return Applied(Gold(n))
        Move      → return Applied(Move)              // state 不变
        Unknown   → return Applied(Unknown)           // state 不变
```

`CardEngine::apply(event_id, &mut state)`：
```
match event_to_card(event_id, catalog):
    Some(card) → apply_card(card, state)
    None → CardResult::UnknownEvent
```

### 2.9 卡牌价格表

`sub_19F9C` 初始化 17 组 (price, stock) 对：

```
[7, 9, 62, 9, 117, 9, 7, 53, 62, 53, 117, 53, 7, 97, 62, 97, 117, 97, 62, 141, 0]
```

| 索引 | 价格 | 库存上限 |
|---|---|---|
| 0 | 7 | 9 |
| 2 | 62 | 9 |
| 4 | 117 | 9 |
| 6 | 7 | 53 |
| 8 | 62 | 53 |
| 10 | 117 | 53 |
| 12 | 7 | 97 |
| 14 | 62 | 97 |
| 16 | 117 | 97 |
| 18 | 62 | 141 |
| 20 | 0 | — |

偶数下标 = 价格，奇数下标 = 库存上限。

### 2.10 卡牌场景

| 组件 | 内容 |
|---|---|
| 场景资源 | `card.Map` + `bigcard.Map` |
| 状态码 | `a1+716 = 4` |
| 卡牌对象 ctor | `sub_19F9C` |
| 场景 ctor | `sub_3D4C8`（`srand` + `sub_3EB24` 读 settings） |
| 场景入口 wrapper | `sub_45E78` |
| 翻页按钮 | `switchpage.pcx` |

### 2.11 卡牌引擎结构

```
struct CardEngine {
    catalog: Vec<CardDef>
}
```

| 方法 | 语义 |
|---|---|
| `new(catalog)` | 构造 |
| `placeholder()` | 占位目录（5 张卡） |
| `catalog()` | 卡牌目录 |
| `event_to_card(event_id)` | 事件 id → 卡牌定义 |
| `apply(event_id, &mut state)` | 应用效果 |
| `all_event_ids()` | 全部有处理路径的 id（1..=25） |
| `card_by_id(id)` | 按卡牌 id 查定义 |

---

## 3. 战斗 + 卡牌联动

### 3.1 卡牌影响战斗

```
主地图 → dispatch(3) → 卡牌事件生成
    ↓ event_id
CardEngine::apply(event_id, &mut CardState)
    ↓
CardState.hp 变化（Damage/Heal）
    ↓
进入战斗 vs.Map 时，BattleSession::new(seed, Unit { hp: CardState.hp, atk }, tier)
```

### 3.2 战斗结果影响经济

战斗胜利后：
- 播 `victory.wav`
- 兵种奖励阶梯（`sub_503F0`）：+15000 ~ +50000 金
- 回主菜单（`advance(2)` → 状态 0）