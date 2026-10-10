# 卡牌与事件系统实现规格

本文档是卡牌/事件系统的完整实现规格，实现者无需阅读其他文档即可写出正确的卡牌逻辑代码。

> 事实来源：`docs/design/combat.md` §2 + `docs/design/constants.md` §5 + `docs/facts/map.md`。

---

## 1. 类型定义

```rust
/// 卡牌效果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardEffect {
    Damage(u32),
    Heal(u32),
    Gold(u32),
    Move,
    Unknown,
}

/// 卡牌定义
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDef {
    pub id: u32,
    pub effect: CardEffect,
}

/// 卡牌状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardState {
    pub hp: u32,
    pub gold: u32,
}

/// 卡牌引擎
#[derive(Debug, Clone)]
pub struct CardEngine {
    catalog: Vec<CardDef>,
}

/// 应用结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardResult {
    Applied(CardEffect),
    UnknownEvent,
}
```

---

## 2. 卡牌效果类型

`效果类型码解码` 效果码 1..4 → 语义（推测映射）：

| 效果码 | Rust 枚举 | 行为 |
|---|---|---|
| 1 | `CardEffect::Damage(n)` | `hp = hp.saturating_sub(n)` |
| 2 | `CardEffect::Heal(n)` | `hp = hp.saturating_add(n)` |
| 3 | `CardEffect::Gold(n)` | `gold = gold.saturating_add(n)` |
| 4 | `CardEffect::Move` | 不改 state（地图移动归主地图模块） |
| 0 | `CardEffect::Unknown` | 无效果 |

---

## 3. 卡牌目录

```
CARD_CATALOG (5 张卡):
    CardDef { id: 1, effect: Damage(10) }
    CardDef { id: 2, effect: Heal(15) }
    CardDef { id: 3, effect: Gold(20) }
    CardDef { id: 4, effect: Move }
    CardDef { id: 5, effect: Unknown }
```

> 效果数值占位，需对照 `cards.lpk` 的 `cards\cards.ini` 实际内容确认。

---

## 4. 事件 id → 卡牌映射

```rust
pub fn event_to_card(event_id: u32, catalog: &[CardDef]) -> Option<CardDef> {
    if catalog.is_empty() {
        return None;
    }
    if event_id == 0 {
        return None;
    }
    if (event_id as usize) <= catalog.len() {
        Some(catalog[(event_id - 1) as usize])
    } else {
        Some(catalog[((event_id - 1) as usize) % catalog.len()])
    }
}
```

示例（5 张卡）：

| event_id | 映射索引 | 卡牌 ID |
|---|---|---|
| 1 | 0 | 1 |
| 2 | 1 | 2 |
| 5 | 4 | 5 |
| 6 | (6-1)%5 = 0 | 1 |
| 25 | (25-1)%5 = 4 | 5 |

---

## 5. 事件生成算法

主地图帧更新 case 3（`主地图帧更新` 行 2841-2851）：

```rust
pub fn generate_event_id(rand_val: u32) -> u32 {
    // 1. 生成原始 event_id
    let raw_id = rand_val % 25;         // 0..24
    let mut event_id = raw_id + 1;     // 1..25

    // 2. 重映射修正
    match event_id {
        4 => event_id = 8,
        13 => event_id = 17,
        14 => event_id = 18,
        15 => event_id = 19,
        _ => {}
    }

    event_id
}
```

### 5.1 写全局

```
MEMORY[0x100081170] = event_id
```

### 5.2 卡牌库存门控

```
card_state = 0002101C:帧更新(ctrl + 72)
if 卡牌库存 >= 10:
    装载 ui.pak + card.Map + bigcard.Map
    a1+716 = 4                     // 卡牌态
```

---

## 6. 事件 id 重映射表

| 原始 id | 重映射后 |
|---|---|
| 4 | 8 |
| 13 | 17 |
| 14 | 18 |
| 15 | 19 |

其余 id 不变。

---

## 7. 全部 25 个 event id 清单

| event_id | 来源 | 处理路径 |
|---|---|---|
| 1 | 直接 | `cards` 统一存储 |
| 2 | 直接 | `cards` 统一存储 |
| 3 | 直接 | `cards` 统一存储 |
| 4 | 重映射 → 8 | `cards` 统一存储 |
| 5 | 直接 | `cards` 统一存储 |
| 6 | 直接 | `cards` 统一存储 |
| 7 | 直接 | `cards` 统一存储 |
| 8 | 由 4 重映射 / 直接 | `cards` 统一存储 |
| 9 | 直接 | `cards` 统一存储 |
| 10 | 直接 | `cards` 统一存储 |
| 11 | 直接 | `cards` 统一存储 |
| 12 | 直接 | `cards` 统一存储 |
| 13 | 重映射 → 17 | `cards` 统一存储 |
| 14 | 重映射 → 18 | `cards` 统一存储 |
| 15 | 重映射 → 19 | `cards` 统一存储 |
| 16 | 直接 | `cards` 统一存储 |
| 17 | 由 13 重映射 | `cards` 统一存储 |
| 18 | 由 14 重映射 | `cards` 统一存储 |
| 19 | 由 15 重映射 | `cards` 统一存储 |
| 20 | 直接 | `cards` 统一存储 |
| 21 | 直接 | `cards` 统一存储 |
| 22 | 直接 | `cards` 统一存储 |
| 23 | 直接 | `cards` 统一存储 |
| 24 | 直接 | `cards` 统一存储 |
| 25 | 直接 | `cards` 统一存储 |

全部 25 个 id 都有处理路径（`cards` 统一存储），覆盖率 100%。

---

## 8. 卡牌效果应用

```rust
pub fn apply_card(card: CardDef, state: &mut CardState) -> CardResult {
    match card.effect {
        CardEffect::Damage(n) => {
            state.hp = state.hp.saturating_sub(n);
            CardResult::Applied(CardEffect::Damage(n))
        }
        CardEffect::Heal(n) => {
            state.hp = state.hp.saturating_add(n);
            CardResult::Applied(CardEffect::Heal(n))
        }
        CardEffect::Gold(n) => {
            state.gold = state.gold.saturating_add(n);
            CardResult::Applied(CardEffect::Gold(n))
        }
        CardEffect::Move => CardResult::Applied(CardEffect::Move),
        CardEffect::Unknown => CardResult::Applied(CardEffect::Unknown),
    }
}
```

---

## 9. 卡牌引擎状态机

```
CardEngine 状态:
    Initial → (new) → Ready
    Ready → (apply) → Ready
    Ready → (reset) → Initial
```

### 9.1 方法语义

| 方法 | 语义 |
|---|---|
| `new(catalog)` | 构造 |
| `placeholder()` | 占位目录（5 张卡） |
| `catalog()` | 卡牌目录 |
| `event_to_card(event_id)` | 事件 id → 卡牌定义 |
| `apply(event_id, &mut state)` | 应用效果 |
| `all_event_ids()` | 全部有处理路径的 id（1..=25） |
| `card_by_id(id)` | 按卡牌 id 查定义 |

### 9.2 实现

```rust
impl CardEngine {
    pub fn new(catalog: Vec<CardDef>) -> Self {
        CardEngine { catalog }
    }

    pub fn placeholder() -> Self {
        CardEngine::new(vec![
            CardDef { id: 1, effect: CardEffect::Damage(10) },
            CardDef { id: 2, effect: CardEffect::Heal(15) },
            CardDef { id: 3, effect: CardEffect::Gold(20) },
            CardDef { id: 4, effect: CardEffect::Move },
            CardDef { id: 5, effect: CardEffect::Unknown },
        ])
    }

    pub fn catalog(&self) -> &[CardDef] { &self.catalog }

    pub fn event_to_card(&self, event_id: u32) -> Option<CardDef> {
        event_to_card(event_id, &self.catalog)
    }

    pub fn apply(&self, event_id: u32, state: &mut CardState) -> CardResult {
        match self.event_to_card(event_id) {
            Some(card) => apply_card(card, state),
            None => CardResult::UnknownEvent,
        }
    }

    pub fn all_event_ids(&self) -> Vec<u32> {
        (1..=25).collect()
    }

    pub fn card_by_id(&self, id: u32) -> Option<CardDef> {
        self.catalog.iter().find(|c| c.id == id).copied()
    }
}
```

---

## 10. 卡牌价格表

`卡牌对象` 初始化 17 组 (price, stock) 对：

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

---

## 11. 卡牌场景

| 组件 | 内容 |
|---|---|
| 场景资源 | `card.Map` + `bigcard.Map` |
| 状态码 | `a1+716 = 4` |
| 卡牌对象 ctor | `卡牌对象` |
| 场景 ctor | `卡牌场景`（`srand` + `设置读取` 读 settings） |
| 场景入口 wrapper | `卡牌场景入口` |
| 翻页按钮 | `switchpage.pcx` |

---

## 12. 卡牌 + 战斗联动

```
主地图 → dispatch(3) → 卡牌事件生成
     ↓ event_id
CardEngine::apply(event_id, &mut CardState)
     ↓
CardState.hp 变化（Damage/Heal）
     ↓
进入战斗 vs.Map 时，BattleSession::new(seed, Unit { hp: CardState.hp, atk }, tier)
```

---

## 13. RNG 对齐

卡牌场景使用 MSVC LCG（F-320）：

| 常量 | 值 | 说明 |
|---|---|---|
| LCG 乘数 | 214,013（`0x343FD`） | `seed × 214013` |
| LCG 加数 | 2,531,011（`0x269EC3`） | `+ 2531011` |
| 输出掩码 | `0x7FFF` | `(seed >> 16) & 0x7FFF` |
| 值域 | 0..=32,767 | 15 位 |

播种公式：`seed = wMinute + wSecond + wMilliseconds`（3 处播种点：卡牌场景 ctor、存读档入口 x2）。
