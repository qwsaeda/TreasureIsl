# 战斗系统实现规格

本文档是战斗系统的完整实现规格，实现者无需阅读其他文档即可写出正确的战斗逻辑代码。

> 事实来源：`docs/business/combat.md`（伤害模型/胜负判定节）+ `legacy/docs/design/constants.md` §4 + `legacy/docs/facts/exe.md`。

---

## 1. 类型定义

> S8.3.a 起 `rules::battle.rs` 数值走 `CombatCfg` + `GameRng`（battle.rs:30），
> 本结构（`seed/player:Unit/enemy:Unit/round/outcome`）为 oracle 占位。
> 当前实际结构：`BattleSession { cfg, rng, player_hp, player_atk, enemy_hp, enemy_atk, round, outcome }`。

```rust
/// 战斗单位
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    pub hp: u32,
    pub atk: u32,
}

/// 战斗方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Player,
    Enemy,
}

/// 战斗会话状态机
#[derive(Debug, Clone)]
pub struct BattleSession {
    seed: u32,
    player: Unit,
    enemy: Unit,
    round: u32,
    outcome: Option<Side>,
}

/// 单回合结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundResult {
    pub player_hp: u32,
    pub enemy_hp: u32,
    pub winner: Option<Side>,
}

/// 整场战斗结果
#[derive(Debug, Clone)]
pub struct BattleResult {
    pub trace: Vec<RoundResult>,
    pub winner: Side,
    pub rounds: u32,
}
```

---

## 2. BOSS HP 表

各档 HP 见 `legacy/docs/design/constants.md` §4（BOSS 档 0..4 = 50,000/100,000/150,000/200,000/300,000；tier > 4 → 0 无效档）：

```rust
pub fn oracle_boss_hp(tier: u32) -> u32 {
    match tier {
        0 => 50_000,
        1 => 100_000,
        2 => 150_000,
        3 => 200_000,
        4 => 300_000,
        _ => 0,
    }
}
```

`tier > 4` → HP = 0 → 视为 BOSS 已死 → 玩家立即获胜，0 回合。

---

## 3. 伤害公式

> S8.3.a 起当前伤害模型 = `balance.combat.damage`（atk_plus_flat）：
> `damage = round(atk × atk_coef) + rng_offset`（`rng_offset ∈ [0, rng_span)`，下限 `flat`）。
> 本节的 K=2654435761 公式为 oracle 占位（推测），非当前实现。

```
damage(seed, atk) = (seed × 2654435761) % (atk + 1) + 1
```

- `atk = 0` → 恒 1（`% 1 = 0` + 1）
- `seed = 0` → 恒 1
- 纯确定性（同 seed 同 atk 同结果），无 rand 库
- 常数 `2654435761`（K = 0x9E3779B1，占位常数，非标准 Knuth，exe 实测值）

```rust
pub const DAMAGE_K: u64 = 2_654_435_761;

pub fn damage(seed: u32, atk: u32) -> u32 {
    ((seed as u64).wrapping_mul(DAMAGE_K) % (atk as u64 + 1)) as u32 + 1
}
```

> 该公式为 oracle 占位（推测），exe 实际 RNG 归 `legacy/docs/design/persistence.md` §RNG 对齐。

---

## 4. 回合制流程

每回合双方各行动一次，**玩家先手**：

```rust
pub fn battle_round(seed: u32, player: Unit, enemy: Unit) -> RoundResult {
    // 1. 玩家行动
    let player_dmg = damage(seed, player.atk);
    let enemy_hp_after = enemy.hp.saturating_sub(player_dmg);

    // 2. 敌方行动（seed+1 避免同回合双方伤害相同）
    let enemy_dmg = damage(seed.wrapping_add(1), enemy.atk);
    let player_hp_after = player.hp.saturating_sub(enemy_dmg);

    // 3. 胜负判定（先判敌方归零 = 玩家优势）
    let winner = if enemy_hp_after == 0 {
        Some(Side::Player)
    } else if player_hp_after == 0 {
        Some(Side::Enemy)
    } else {
        None
    };

    RoundResult {
        player_hp: player_hp_after,
        enemy_hp: enemy_hp_after,
        winner,
    }
}
```

---

## 5. 整场战斗

> 注：当前 `rules::battle.rs` 的 `max_rounds` 由 `balance.combat.max_rounds` 注入；
> balance 缺失时回退 30（battle.rs:53 `fallback_cfg`），非 1000。本节的 1000 为 oracle 占位。

```rust
pub const MAX_BATTLE_ROUNDS: u32 = 1000;

pub fn full_battle(seed: u32, player: Unit, boss_tier: u32) -> BattleResult {
    let boss_hp = oracle_boss_hp(boss_tier);
    if boss_hp == 0 {
        return BattleResult {
            trace: vec![],
            winner: Side::Player,
            rounds: 0,
        };
    }

    let mut player_cur = player;
    let mut enemy_cur = Unit { hp: boss_hp, atk: 10 }; // 注：当前 rules/battle.rs 从 balance.combat.boss_tiers 对应档取 atk（各档 10/12/14/16/20），此处 10 为 oracle 占位
    let mut trace = Vec::new();
    let mut round_seed = seed;
    let mut round_num = 0u32;

    for _ in 0..MAX_BATTLE_ROUNDS {
        round_num += 1;
        let result = battle_round(round_seed, player_cur, enemy_cur);
        trace.push(result);
        player_cur = Unit { hp: result.player_hp, atk: player_cur.atk };
        enemy_cur = Unit { hp: result.enemy_hp, atk: enemy_cur.atk };
        round_seed = round_seed.wrapping_add(1);

        if let Some(winner) = result.winner {
            return BattleResult { trace, winner, rounds: round_num };
        }
    }

    BattleResult {
        trace,
        winner: Side::Enemy,
        rounds: round_num,
    }
}
```

---

## 6. 胜负判定优先级

| 情况 | 胜方 | 说明 |
|---|---|---|
| `enemy.hp == 0` | Player | BOSS 被击败 |
| `player.hp == 0` | Enemy | 玩家被击败 |
| 双方同时 0 | Player | 玩家优势（先判敌方归零） |
| 1000 回合未结束 | Enemy | 超时判负 |

---

## 7. 战斗会话状态机

### 7.1 状态定义

```
BattleSession 状态:
    Initial → (new) → Running
    Running → (step) → Running | Finished
    Finished → (reset) → Initial
```

### 7.2 方法语义

> **API 变更（S8.3.a）**：`rules/battle.rs` 已实现 `from_balance(seed, player, boss_tier)`（数值走 `balance.combat`）
> 与 `new_with_cfg(cfg, seed, player, boss_tier)`（测试注入），不再有裸 `new`。
> 下表「构造」行描述的是 oracle 历史路径，当前代码入口为 `from_balance`。

| 方法 | 语义 |
|---|---|
| `from_balance(seed, player, boss_tier)` | 构造（数值走 `balance.combat.boss_tiers`，越界档 → `enemy_hp=0` 立即胜） |
| `boss_hp()` | 当前 BOSS HP |
| `round()` | 当前回合数 |
| `outcome()` | 战斗结果（`None` = 未结束） |
| `step()` | 单步回合，返回 `(player_hp, boss_hp, winner)` |

### 7.3 实现（oracle 历史参照，**非当前代码**）

> 以下为 `oracle/battle.rs` 的直译参考（S8.3.a 前）；当前 `rules/battle.rs` 用
> `from_balance`/`new_with_cfg` 构造，数值全走 `balance.combat`。

```rust
impl BattleSession {
    // oracle 历史路径（rules 侧已替换为 from_balance）
    pub fn new(seed: u32, player: Unit, boss_tier: u32) -> Self {
        let boss_hp = oracle_boss_hp(boss_tier);
        let outcome = if boss_hp == 0 { Some(Side::Player) } else { None };
        BattleSession {
            seed,
            player,
            enemy: Unit { hp: boss_hp, atk: 10 },
            round: 0,
            outcome,
        }
    }

    pub fn boss_hp(&self) -> u32 { self.enemy.hp }
    pub fn round(&self) -> u32 { self.round }
    pub fn outcome(&self) -> Option<Side> { self.outcome }

    pub fn step(&mut self) -> RoundResult {
        if self.outcome.is_some() {
            return RoundResult {
                player_hp: self.player.hp,
                enemy_hp: self.enemy.hp,
                winner: self.outcome,
            };
        }
        let result = battle_round(self.seed, self.player, self.enemy);
        self.player = Unit { hp: result.player_hp, atk: self.player.atk };
        self.enemy = Unit { hp: result.enemy_hp, atk: self.enemy.atk };
        self.round += 1;
        self.seed = self.seed.wrapping_add(1);
        if let Some(w) = result.winner {
            self.outcome = Some(w);
        }
        result
    }

    pub fn finish(&mut self) -> BattleResult {
        let mut trace = Vec::new();
        while self.outcome.is_none() {
            trace.push(self.step());
        }
        BattleResult {
            trace,
            // 仅文档示意；实际实现用 Option 分支（门禁禁止 unwrap）
        winner: match self.outcome {
            Some(w) => w,
            None => Side::Player,
        },
            rounds: self.round,
        }
    }
}
```

---

## 8. 战斗场景对象

```
new vs 对象 (0x474 bytes)
战斗对象构造 构造:
    场景对象 构造基类（设 vtable）
    *a1 = &off_73A2C              // vs 场景 vtable
    a1[256..261] = 0              // 6 个战斗状态字段清零
    a1[277..284] = 0              // 8 个兵种/HP 字段清零
    a1[255] = 1                   // 战斗标志
    a1[276] = 1                   // 第二标志
```

---

## 9. 战斗事件流

| 步骤 | 动作 | 状态变化 |
|---|---|---|
| 1 | 主地图事件码 6 触发 | `0002101C:帧更新` 判定 |
| 2 | 建 vs 对象 | `战斗对象构造` + `ui.pak+vs.Map` + `cards` 初始化 |
| 3 | 进入战斗态 | `a1+716 = 25` |
| 4 | 玩家点「攻击」 | `BattleSession::step()` 单回合 |
| 5 | 循环直到 `outcome != None` | 渲染 HP/回合数/胜方 |
| 6 | 胜 → 播 `victory.wav` | `advance(2)` → 状态 0 |
| 7 | 败 → 失败面板 | `advance(2)` → 状态 0 → 回主菜单 |

---

## 10. 兵种

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

## 11. 胜利奖励

战斗胜利后：
- 播 `victory.wav`
- 兵种奖励阶梯（`投资`）：+15,000 ~ +50,000 金（数值表见 `legacy/docs/design/constants.md` §3.4 / `legacy/docs/design/economy.md` §2.8）
- 回主菜单（`advance(2)` → 状态 0）

> 推测：兵种奖励与兵种匹配关系（A..E 各档），exe 无独立数值表。

---

## 12. 战斗场景状态码

| 状态码 | 语义 |
|---|---|
| 0 | 主菜单态 |
| 25 | 战斗态 |
| 2 | 场景卸载中 |
| 3 | 回主菜单 |

---

## 13. 输入映射

| GameInput | 行为 |
|---|---|
| `Confirm` / `Attack` | 攻击一回合 |
| `Cancel` | 退出战斗 → 回 `Index` |
| 方向 | 无（战斗无方向操作） |

---

## 14. 渲染要点

- 每回合后渲染玩家 HP、BOSS HP、当前回合数
- 战斗结束渲染胜方
- 战斗场景资源：`ui.pak + vs.Map`
- 战斗态标志：`a1+716 = 25`
