# 经济系统

本文档定义完整的经济系统：贸易（价格表/买卖公式/利润计算）、建造（建筑类型/成本公式/银行系统/金上限）。

---

## 1. 贸易系统

### 1.1 商品目录

5 种商品，固定价格表（来源 `docs/design/economy.md` §1.1，`资源查表` 哈希键 buy_key/sell_key）：

| 商品 ID | 名称 | 价格 | buy_key | sell_key |
|---|---|---|---|---|
| 1 | good_01 | 20 | 202 | 102 |
| 2 | good_02 | 76 | 402 | 302 |
| 3 | good_03 | 95 | 602 | 502 |
| 4 | good_04 | 114 | 802 | 702 |
| 5 | good_05 | 133 | 112 | 902 |

> 商品名 good_01..05 为占位（**推测**），价格与 buy_key/sell_key 来源 `docs/design/economy.md` §1.1。

### 1.2 价格表结构

```rust
pub struct PriceTable {
    entries: Vec<(u32, u32)>, // (goods_id, fixed_price)
}

impl PriceTable {
    pub fn lookup(&self, gid: u32) -> Option<u32> {
        self.entries.iter().find(|(id, _)| *id == gid).map(|(_, price)| *price)
    }

    pub fn oracle_price(&self, gid: u32, seed: u32) -> Option<u32> {
        let base = self.lookup(gid)?;
        if seed == 0 {
            Some(base)
        } else {
            Some(base - (seed % 10))
        }
    }
}
```

> 来源 `docs/design/economy.md` §1.3（行情波动占位：`base - (seed % 10)` 只降不涨，oracle 占位公式）。

### 1.3 买入

> oracle 逐件占位；当前 `TradeLedger::buy(gid, qty)` 为 qty 批量（`rules/trade.rs`），本节为 oracle 占位/历史参照。

```rust
pub fn oracle_buy(
    table: &PriceTable,
    gid: u32,
    gold: u32,
) -> Result<TradeResult, TradeError> {
    let price = table.lookup(gid).ok_or(TradeError::UnknownGood)?;
    if gold < price {
        return Err(TradeError::InsufficientGold);
    }
    Ok(TradeResult {
        gid,
        paid: price,
        gold_left: gold - price,
        profit: 0,
    })
}
```

### 1.4 卖出

> 同上：oracle 逐件占位；当前 `TradeLedger::sell(gid, qty)` 为 qty 批量（`rules/trade.rs`）。
> `TradeResult` 字段名：oracle 占位用 `paid`/`profit`；当前 `rules::trade::TradeResult` 为 `{ gid, qty, amount, gold_left }`（见 §3）。

卖出**当前全部持有量**：

```rust
pub fn oracle_sell(
    table: &PriceTable,
    gid: u32,
    owned: u32,
    gold: u32,
    avg_buy_price: u32,
) -> Result<TradeResult, TradeError> {
    let price = table.lookup(gid).ok_or(TradeError::UnknownGood)?;
    if owned == 0 {
        return Err(TradeError::NoStock);
    }

    let gain = price * owned;
    let unit_profit = if avg_buy_price > 0 {
        price.saturating_sub(avg_buy_price)
    } else {
        price
    };
    let profit = unit_profit * owned;

    Ok(TradeResult {
        gid,
        paid: gain,
        gold_left: gold + gain,
        profit,
    })
}
```

### 1.5 买入均价

```
avg_buy_price(gid) = (owned(gid) == 0) ? 0 : buy_cost[gid] / owned(gid)
```

- 买入成功：`buy_cost[gid] += price`
- 卖出成功：`buy_cost[gid] -= price × sold_qty`

### 1.6 累计利润

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

### 1.7 错误类型

| 错误 | 触发条件 |
|---|---|
| `TradeError::UnknownGood` | `gid` 不在价格表中 |
| `TradeError::InsufficientGold` | `gold < price`（买入） |
| `TradeError::NoStock` | `owned == 0`（卖出） |

> 注：oracle 侧仅 3 变体；当前 `rules::trade::TradeErr` 实际有 4 个变体（多 `QtyOutOfRange`，qty==0 或超持有上限），见 §3。

失败时状态不变（事务性）。

### 1.8 贸易场景事件码

| 事件码 | 行为 | 说明 |
|---|---|---|
| 22 | 进入贸易场景（买入方向） | `a1+716 = 22` |
| 23 | 进入贸易场景（卖出方向） | `a1+716 = 23` |
| 13 | 买入/卖出确认 | `+1020 == 6` 买入 / `+1020 == 7` 卖出 |
| 37 | 取消 | 撤销当前操作 |
| 38 | 翻上页 | 商品索引递减，到 1 回 6 |
| 39 | 翻页确认 | `+1020` 6 → 7 |
| 40 | 翻页 | 商品索引递增，超 `+1064` 回 1 |

前置条件：`*(a1+716) == 21`（据点结算完成态）。

---

## 2. 建造系统

> 注：§1/§2 代码块中枚举名 `TradeError`/`BuildError` 为 oracle 占位名；当前 `rules::trade`/`rules::build` 实际为 `TradeErr`/`BuildErr`（见 §3）。

### 2.1 建筑类型

| 建筑类型 | 基础成本 | 控件 |
|---|---|---|
| Castle（城堡） | 1000 | `button_aimcastle` + `button_build` |
| Investment（投资设施） | 500 | `button_invest` |
| Bank（银行） | 200 | `button_bank` |
| Fort（据点/堡垒） | 800 | `button_capture` + `button_build` |

### 2.2 建造成本公式

```
cost = base_cost(kind) × (level + 1)       level ∈ [0, 10]
```

base_cost 见 `docs/design/constants.md` §3.2（Castle 1000 / Investment 500 / Bank 200 / Fort 800）；
等级 0..10 展开值 = base_cost × (level+1)，等级上限 10（`level > 10` → `InvalidLevel`）。

### 2.3 建造操作

```rust
pub fn oracle_build(
    bank: &mut Bank,
    kind: Building,
    level: u32,
) -> Result<(), BuildError> {
    if level > 10 {
        return Err(BuildError::InvalidLevel);
    }
    let cost = base_cost(kind) * (level + 1);
    if bank.gold < cost {
        return Err(BuildError::Insufficient);
    }
    bank.gold -= cost;
    Ok(())
}
```

### 2.4 金量上限

```
MAX_GOLD = 999999    // 0xF423F
```

金量溢出时截断到 999999。

> 注：`rules::build::BuildManager::settle_overflow(rate)` 实际为「溢出按 `bank_rate` 比例进银行」（`gold_cap` 由已建建筑的 `gold_max` 抬升），gold_max 截断为上限语义，非简单 999999 截断。

### 2.5 银行系统

#### 双槽位模型

| 槽位 | 语义 |
|---|---|
| 流动金 | 可用于建造/贸易/投资 |
| 银行存款金 | 存入银行的金，安全保管 |

#### 存款

```rust
pub fn oracle_deposit(bank: &mut Bank, amount: u32) -> Result<(), BuildError> {
    if amount > bank.gold {
        return Err(BuildError::Insufficient);
    }
    bank.gold -= amount;
    bank.deposit += amount;
    Ok(())
}
```

#### 取款

```rust
pub fn oracle_withdraw(bank: &mut Bank, amount: u32) {
    bank.gold = (bank.gold + amount).min(999999);
    bank.deposit -= amount;
}
```

取款**永不失败**（金量只增不减，上限截断）。

### 2.6 银行步进器

按压 1/2/3/4/5 次 = 10/100/1000/10000/100000，6..10 保持当前（`docs/design/constants.md` §3.3）。

### 2.7 投资奖励阶梯

兵种 A..E 奖励 +30,000/+40,000/+20,000/+15,000/+50,000（`投资`，来源 `docs/design/economy.md` §2.8 投资奖励阶梯（外部设计源）/ `docs/design/constants.md` §3.4；
数值与本地 §2.7 一致）。

---

## 3. 数据结构定义

```rust
// crates/ti-core/src/rules/trade.rs + rules/build.rs + oracle/build.rs

#[derive(Debug, Clone)]
pub struct TradeLedger {
    pub goods: Vec<(u32, u32)>,      // (gid, qty)
    pub gold: u32,
    pub buy_cost: Vec<(u32, u32)>,   // (gid, 累计成本)
    pub cfg: TradeCfg,              // 价格表/售价系数/数量上限，由 balance.json 注入
}

#[derive(Debug, Clone)]
pub struct BuildManager {
    pub bank: Bank,
    pub buildings: Vec<(String, u32)>, // (id, level)
    pub cfg: BuildCfg,                // 建筑造价/金上限配置，由 balance.json 注入
}

#[derive(Debug, Clone, Copy)]
pub struct Bank {
    pub gold: u32,
    pub deposit: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum Building {
    Castle,
    Investment,
    Bank,
    Fort,
}

// 注：`Building` 枚举为 oracle 侧；当前 `rules::build` 用 balance 字符串 id（`Vec<(String, u32)>`），
// 无 `Building` 枚举参与，建筑由 `BuildCfg`（balance.json）驱动。

#[derive(Debug, Clone, Copy)]
pub struct TradeResult {
    pub gid: u32,
    pub qty: u32,
    pub amount: u32,
    pub gold_left: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum TradeErr {
    UnknownGood,
    InsufficientGold,
    NoStock,
    QtyOutOfRange,
}

#[derive(Debug, Clone, Copy)]
pub enum BuildErr {
    UnknownBuilding,
    Insufficient,
    InvalidLevel,
}
```

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
   owned = 3, price = 20, avg = 20
   unit_profit = max(20 - 20, 0) = 0
   profit = 0 × 3 = 0
   gain = 20 × 3 = 60
   gold = 4940 + 60 = 5000
   goods[1] = 0
   buy_cost[1] = 60 - 60 = 0
```

固定价格表下买卖无盈亏（利润恒 0）。

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
   bank.deposit = 3000

4. 取款 2000：
   bank.gold = min(4000 + 2000, 999999) = 6000
   bank.deposit = 1000
```

### 4.3 金量上限演示

```
bank.gold = 999000
withdraw(2000) → min(999000 + 2000, 999999) = 999999
bank.gold = 999999
withdraw(1) → min(999999 + 1, 999999) = 999999
```

---

## 5. 实现检查清单

- [ ] 价格表解析正确
- [ ] 买入/卖出公式正确
- [ ] 利润计算精确
- [ ] 建造成本公式正确
- [ ] 银行存取正确
- [ ] 金量上限截断
- [ ] 事务性保证
