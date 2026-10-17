# 贸易 / 建造 / 银行（Economy）

## 设计意图

经济系统是**金币的进出两端**，服务核心循环（探索→贸易→建造→战斗）：

- **贸易**：低买高卖赚差价，是金币主要来源；
- **建造**：花金币提升城堡，提供**金量上限**与**被动收益**；
- **银行**：金币满仓后的溢出处理（提高上限）。

数值全在 `assets/data/balance.json`，调平衡不改代码。

> **命名说明**：本文的商品/建筑名（spice/silk/gem、palisade/harbor/keep 等）是
> **业务可读命名（设计示例）**；实际数值与 id 以 `assets/data/balance.json` 为准
> （trade.goods 1..5 数字 id、build.buildings Castle/Investment/Bank/Fort 字符串 id）。

## 商品与价格

> 以下为**示例节选**（3 种），完整数值（5 种商品，id 1..5）以 `assets/data/balance.json` 为准。
> ⚠️ 下表 id/name_key/数值为设计示例，**与 balance.json 实际值不同**；实际以 assets/data/balance.json 为准（数字 id 1..5、name_key=goods.N、buy=20/76/95/114/133、qty_max=99）。

```jsonc
"trade": {
  "markup": 1.0,          // 售价 = 成本 * markup
  "goods": [              // 商品目录（示例节选）
    { "id": "spice",  "name_key": "goods.spice",  "buy": 10,  "qty_max": 99 },
    { "id": "silk",   "name_key": "goods.silk",   "buy": 25,  "qty_max": 50 },
    { "id": "gem",    "name_key": "goods.gem",   "buy": 60,  "qty_max": 20 }
  ]
}
```

### 买卖结算（设计选择）

```
买入 cost  = goods.buy  * qty
卖出 value = round(goods.buy * markup) * qty      // 售价随目录固定，避免套利
约束       = 0 ≤ qty ≤ goods.qty_max
           = 0 ≤ gold ≤ balance.gold_max
```

**不做「动态行情」**：价格固定，避免出现「囤货等涨价」的复杂博弈——本作定位
轻量经营，贸易是稳定的金币来源，而非投机玩法。

## 建造系统

> 以下为**示例节选**（3 种），完整数值（4 种建筑，id 以 `assets/data/balance.json` `build.buildings` 为准）以 `assets/data/balance.json` 为准。

```jsonc
"build": {
  "buildings": [
    { "id": "Castle",     "name_key": "build.castle",     "base_cost": 1000, "gold_max": 20000, "atk_bonus": 10, "hp_bonus": 200 },
    { "id": "Investment", "name_key": "build.investment", "base_cost": 500,  "gold_max": 12000, "atk_bonus": 5,  "hp_bonus": 100 },
    { "id": "Bank",       "name_key": "build.bank",       "base_cost": 200,  "gold_max": 5000,  "atk_bonus": 0,  "hp_bonus": 50 },
    { "id": "Fort",       "name_key": "build.fort",       "base_cost": 800,  "gold_max": 16000, "atk_bonus": 8,  "hp_bonus": 150 }
  ]
}
```

建造是**单调升级链**（每种建筑可反复升级，`base_cost × (level+1)` 递增），每建一级：

1. 扣金币；
2. **抬升金量上限**到该建筑对应值（`gold_max` 取已建建筑的最大值）；
3. **抬升战斗 atk/hp**（`atk_bonus`/`hp_bonus` × 等级，`combat.md` §建造加成）；
4. 解锁对应玩法（据点/堡垒 → 高阶 BOSS）。

### 经济上限（`gold_cap` + `settle_overflow`）

- 卖出加金钳到 `gold_cap`（`balance.economy.gold_max`，建造可抬升）；
- 溢出部分按 `bank_rate` 进银行累计（`EconomyHost::settle_overflow`）；
- 金币恒 ≤ 上限（`economy_gold_never_exceeds_cap` 单测）。

### 银行

当金币达到上限，溢出部分进银行累计（`balance.economy.bank_rate` 比例）。
银行余额在战后按比例返还为金币，或作为结算加分。这是「上限不是硬墙」的设计。

## 可测试规则

| 规则 | 单测 |
|---|---|
| 买入扣金币、货物 +qty | `trade_buy_debits_gold_adds_goods` |
| 卖出加金币、货物 -qty | `trade_sell_credits_gold_removes_goods` |
| qty 越界拒绝 | `trade_qty_bounds_enforced` |
| 售价 = buy*markup | `trade_sell_price_from_markup` |
| 建造扣金币 + 抬上限 | `build_debits_and_raises_cap` |
| 金币恒 ≤ 上限（银行承接溢出） | `economy_gold_never_exceeds_cap`（已落地 `rules_build.rs`；穷举覆盖见 `crates/ti-core/tests/exhaust_economy.rs`：贸易侧 `sell` 后 `gold_left ≤ gold_cap`、建造侧 `build` 后 `bank().gold ≤ u32::MAX/4`（上界不越初始值，`saturating_sub` 保证无下溢），S15.1 verified） |
| 建筑单调解锁 | `build_unlocks_monotonic` |

（上表单测已落地于 `crates/ti-core/tests/rules_trade.rs` / `rules_build.rs`（S8.3.b verified）；
`economy_gold_never_exceeds_cap` 已由 `rules_build.rs::economy_gold_never_exceeds_cap` +
`crates/ti-core/tests/exhaust_economy.rs` 穷举覆盖双路验证（S15.1 verified））

## 与实现的接口

- 玩法：`rules::trade`（`TradeLedger`）、`rules::build`（`BuildManager`）（已落地，S8.3.b verified）；
- 数值注入：会话构造时接收 `&balance::Balance`，测试可注假值；
- 表现层只读 `TradeLedger` / `BuildManager` 的只读视图来画 UI，不重算价格。