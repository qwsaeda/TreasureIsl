> ⚠️ **弃用/归档（S8，2026-10）**：本目录为逆向规格参照物，不再是开发依据或验收基准。
> 业务规则唯一事实源 = [`docs/business/`](../business/00_index.md) + `assets/data/*.json`。
> 本文仅作原版形态参考；数值/文本以 `assets/data/*.json` 为准，流程以 `docs/business/` 为准。
> 见 [`docs/plan-s8-refactor.md`](../plan-s8-refactor.md) §1（逆向产物降级）。

# 经济系统设计（贸易 + 建造）

本文档涵盖：贸易系统（商品目录/价格表/买卖结算/利润）+ 建造系统（建筑类型/成本/银行/金上限）。

---

## 1. 贸易系统

### 1.1 商品目录

5 种商品，固定价格表（来自 `贸易场景对象` 初值 `[20,20,76,20,95,114,133,0]`）：

| 商品 ID | 名称 | 价格 | buy_key | sell_key |
|---|---|---|---|---|
| 1 | good_01 | 20 | 202 | 102 |
| 2 | good_02 | 76 | 402 | 302 |
| 3 | good_03 | 95 | 602 | 502 |
| 4 | good_04 | 114 | 802 | 702 |
| 5 | good_05 | 133 | 112 | 902 |

> buy_key/sell_key 是 exe 内 `资源查表` 查表用的哈希键值，价格本身固定。
> 商品名 good_01..05 为占位（推测），需对照 `goods.lpk` 二进制表确认。

### 1.2 价格表结构

```
struct PriceTable {
    entries: Vec<(goods_id, fixed_price)>
}
```

占位表：`[(1,20), (2,76), (3,95), (4,114), (5,133)]`。

查表：`lookup(gid)` → 线性查找 → `Some(price)` 或 `None`。

### 1.3 行情波动占位

`PriceTable::oracle_price(gid, seed)`：

```
price = base_price
if seed != 0:
    price = base_price - (seed % 10)     // 只降不涨
```

固定价格表（`seed == 0`）直接返回 `base_price`。

exe 侧 `主地图帧更新` case 4 的 `rand() % 22 + 1` 用于**商品选择**（22 种货物之一），非价格波动。

### 1.4 买入

```
oracle_buy(table, gid, gold):
    price = table.lookup(gid)
    if price is None: return Err(UnknownGood)
    if gold < price: return Err(InsufficientGold)

    return Ok(TradeResult {
        gid,
        paid: price,
        gold_left: gold - price,
        profit: 0              // 买入不产生利润
    })
```

### 1.5 卖出

卖出**当前全部持有量**（exe 行为，非逐件）：

```
oracle_sell(table, gid, owned, gold, avg_buy_price):
    price = table.lookup(gid)
    if price is None: return Err(UnknownGood)
    if owned == 0: return Err(NoStock)

    gain = price × owned
    if avg_buy_price > 0:
        unit_profit = max(price - avg_buy_price, 0)     // 负值钳 0
    else:
        unit_profit = price                              // 无历史买入 → 全价

    profit = unit_profit × owned
    return Ok(TradeResult {
        gid,
        paid: gain,
        gold_left: gold + gain,
        profit
    })
```

### 1.6 买入均价

`TradeLedger` 维护 `buy_cost: Vec<(gid, 累计成本)>`：

```
avg_buy_price(gid) = (owned(gid) == 0) ? 0 : buy_cost[gid] / owned(gid)
```

- 买入成功：`buy_cost[gid] += price`
- 卖出成功：`buy_cost[gid] -= price × sold_qty`

### 1.7 累计利润

```
profit_since(gid):
    owned = owned(gid)
    if owned == 0: return 0
    price = table.lookup(gid)
    if price is None: return 0
    avg = avg_buy_price(gid)
    unit_profit = (avg > 0) ? price - avg : price
    return unit_profit × owned
```

### 1.8 错误类型

| 错误 | 触发条件 |
|---|---|
| `TradeErr::UnknownGood` | `gid` 不在价格表中 |
| `TradeErr::InsufficientGold` | `gold < price`（买入） |
| `TradeErr::NoStock` | `owned == 0`（卖出） |

失败时状态不变（事务性）。

### 1.9 手续费

伪代码中**无独立手续费扣减**。手续费 = 0（或已含在买卖价差中）。
推测：买卖价差（buy_key ≠ sell_key）本身就是隐性费用。

### 1.10 贸易场景事件码

| 事件码 | 行为 | 说明 |
|---|---|---|
| 22 | 进入贸易场景（买入方向） | `a1+716 = 22` |
| 23 | 进入贸易场景（卖出方向） | `a1+716 = 23` |
| 13 | 买入/卖出确认 | `+1020 == 6` 买入结算 / `+1020 == 7` 卖出结算 |
| 37 | 取消 | 撤销当前操作，回 `单件买入(0)` |
| 38 | 翻上页 | 商品索引递减，到 1 回 6 |
| 39 | 翻页确认 | `+1020` 6 → 7 |
| 40 | 翻页 | 商品索引递增，超 `+1064` 回 1 |

前置条件：`*(a1+716) == 21`（据点结算完成态，进入贸易的入口）。

### 1.11 贸易场景对象布局

```
struct TradeScene {
    +1024  gold            当前金
    +1028  buy_slots       买入槽链表头（8 字节记录 {goods_id, qty}）
    +1040  sell_slots      卖出槽链表头（同结构）
    +1052  total_goods     u16 货物总数
    +1060  mode            模式（0=卖出方向，1=买入方向）
    +1064  owned_count     持有条目数
    +1108  current_slot    当前商品槽指针
    +1112..1128  price[5]  5 个商品价格槽
    +1132  total_price     总价格累积
    +1136  total_qty       总持有数累积
}
```

### 1.12 资源集字段读取器

`资源集字段读取器(resource_set, case)`：

| case | 语义 |
|---|---|
| 0 | u16 总货物数 |
| 2 | u32 金 |
| 4 | 卡牌库存（用于门控） |

---

## 2. 建造系统

### 2.1 建筑类型

| 建筑类型 | 基础成本 | 控件 |
|---|---|---|
| Castle（城堡） | 1000 | `button_aimcastle` + `button_build` |
| Investment（投资设施） | 500 | `button_invest` |
| Bank（银行） | 200 | `button_bank` |
| Fort（据点/堡垒） | 800 | `button_capture` + `button_build` |

> 4 种建筑类型由控件组合推断（推测），exe 无独立类型码常量。

### 2.2 建造成本公式

```
cost = base_cost(kind) × (level + 1)
```

| level | Castle | Investment | Bank | Fort |
|---|---|---|---|---|
| 0（首次） | 1000 | 500 | 200 | 800 |
| 1（升级 1） | 2000 | 1000 | 400 | 1600 |
| 2 | 3000 | 1500 | 600 | 2400 |
| 5 | 6000 | 3000 | 1000 | 4000 |
| 10（上限） | 11000 | 5500 | 2200 | 8800 |

等级上限 10（`level > 10` → `InvalidLevel`）。

### 2.3 建造操作

```
oracle_build(bank, kind, level):
    if level > 10:
        return Err(InvalidLevel)

    cost = base_cost(kind) × (level + 1)
    if bank.gold < cost:
        return Err(Insufficient)

    return Ok(Bank { gold: bank.gold - cost })
```

`BuildManager::build` 成功后：`buildings[kind].level += 1`。

### 2.4 错误类型

| 错误 | 触发条件 |
|---|---|
| `BuildErr::Insufficient` | `bank.gold < cost` |
| `BuildErr::InvalidLevel` | `level > 10` |

失败时状态不变（事务性）。

### 2.5 金量上限

```
MAX_GOLD = 999999    // 0xF423F
```

金量溢出时截断到 999999（`投资` 行 114/116 证据）。

### 2.6 银行系统

#### 双槽位模型

| 槽位 | 偏移 | 语义 |
|---|---|---|
| 流动金 | `+1032` | 可用于建造/贸易/投资 |
| 银行存款金 | `+1040` | 存入银行的金，安全保管 |

其他槽位：

| 槽位 | 偏移 | 语义 |
|---|---|---|
| 货物持有量 | `+1036` | 当前持有 |
| 货物持有量 2 | `+1038` | 第二持有 |
| 库存上限 | `+1044` / `+1046` | `0x3E7` = 999 |
| 资源模式 | `+1052` | 1=货物/2=金/3=其他 |
| 步进累加器 | `+1104` | 模 10 循环 |
| 步进值 | `+1108` | 阶梯递增 |

#### 存款

```
oracle_deposit(bank, amount):
    if amount > bank.gold:
        return Err(Insufficient)
    return Ok(Bank { gold: bank.gold - amount })
    // 流动金 -= amount，银行金 += amount
```

#### 取款

```
oracle_withdraw(bank, amount):
    new_gold = min(bank.gold + amount, 999999)
    return Bank { gold: new_gold }
    // 流动金 += amount，银行金 -= amount
```

取款**永不失败**（金量只增不减，上限截断）。

### 2.7 银行步进器

银行 UI 的金量选择器步进值阶梯：

| 按压次数 | 步进值 |
|---|---|
| 1 | 10 |
| 2 | 100 |
| 3 | 1000 |
| 4 | 10000 |
| 5 | 100000 |
| 6..10 | 保持当前 |

步进值 = 玩家点按的倍数，每次点按 ×10 递增。
用于 UI 快速选择存取金额（推测为 UX 选择器，非固定存取金额）。

### 2.8 投资奖励阶梯

`投资` 按兵种/类型匹配奖励：

```
if 兵种 == A: gold += 30000
if 兵种 == B: gold += 40000
if 兵种 == C: gold += 20000
if 兵种 == D: gold += 15000; level += 5
if 兵种 == E: gold += 50000; exp += 20
```

### 2.9 建造无独立事件码

建造/银行操作在主地图内以按钮交互完成，复用主地图事件码 4（贸易/资源结算），
不触发场景切换。

### 2.10 建造管理器结构

```
struct BuildManager {
    bank: Bank { gold: u32 }
    buildings: Vec<(Building, level: u32)>
}
```

| 方法 | 语义 |
|---|---|
| `new(bank, buildings)` | 构造 |
| `bank()` | 读银行金 |
| `level(kind)` | 某建筑等级（无记录 → 0） |
| `build_cost(kind, level)` | 建造成本 |
| `can_afford(cost)` | 金量是否足够 |
| `build(kind, level)` | 建造（扣金 + 等级 +1） |
| `deposit(amount)` | 存款 |
| `withdraw(amount)` | 取款（永不失败） |

---

## 3. 贸易账本结构

```
struct TradeLedger {
    goods: Vec<(gid, qty)>       // 持有量
    gold: u32                    // 当前金
    table: PriceTable            // 价格表
    buy_cost: Vec<(gid, cost)>   // 累计买入成本
}
```

| 方法 | 语义 |
|---|---|
| `new(gold, table, goods)` | 构造 |
| `gold()` | 当前金 |
| `goods()` | 全部持有量 |
| `owned(gid)` | 某商品持有量（无记录 → 0） |
| `price(gid)` | 查固定价格 |
| `avg_buy_price(gid)` | 买入均价 |
| `buy(gid)` | 买入 1 件 |
| `sell(gid)` | 卖出全部持有 |
| `profit_since(gid)` | 累计利润 |

---

## 4. 完整经济流程示例

### 4.1 开局贸易

```
初始：gold = 5000, 持有量空

1. 买入商品 1（价格 20）：
   gold = 5000 - 20 = 4980
   goods[1] = 1
   buy_cost[1] = 20
   avg_buy_price(1) = 20 / 1 = 20

2. 再买 2 件商品 1：
   gold = 4980 - 40 = 4940
   goods[1] = 3
   buy_cost[1] = 60
   avg_buy_price(1) = 60 / 3 = 20

3. 卖出全部商品 1：
   owned = 3
   price = 20
   avg = 20
   unit_profit = max(20 - 20, 0) = 0
   profit = 0 × 3 = 0
   gain = 20 × 3 = 60
   gold = 4940 + 60 = 5000
   goods[1] = 0
   buy_cost[1] = 60 - 60 = 0
```

固定价格表下买卖无盈亏（利润恒 0）。盈利需价格波动（`oracle_price` 占位公式）或不同据点不同价格。

### 4.2 建造流程

```
初始：bank.gold = 10000, buildings 空

1. 建造 Castle（level 0）：
   cost = 1000 × 1 = 1000
   bank.gold = 10000 - 1000 = 9000
   buildings[Castle] = 1

2. 升级 Castle（level 1）：
   cost = 1000 × 2 = 2000
   bank.gold = 9000 - 2000 = 7000
   buildings[Castle] = 2

3. 存款 3000：
   bank.gold = 7000 - 3000 = 4000

4. 取款 2000：
   bank.gold = min(4000 + 2000, 999999) = 6000
```

### 4.3 金量上限演示

```
bank.gold = 999000
withdraw(2000) → min(999000 + 2000, 999999) = 999999    // 截断
bank.gold = 999999
withdraw(1) → min(999999 + 1, 999999) = 999999            // 保持
```