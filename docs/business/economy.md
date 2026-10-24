# 贸易 / 建造 / 银行（Economy）

## 设计意图

经济系统是**金币的进出两端**，服务核心循环（探索→贸易→建造→战斗）：

- **贸易**：低买高卖赚差价，是金币主要来源；
- **建造**：花金币提升城堡，提供**金量上限**与**被动收益**；
- **船只**（S21.2）：开局小船、单向升级 4 级（小舢板→双桅→三桅→巡防），抬**单次贸易数量上限**（货舱）与**战斗加成**（独立于建造轴）；
- **银行**：金币满仓后的溢出处理（提高上限）。

数值全在 `assets/data/balance.json`，调平衡不改代码。

> **命名说明**：本文的商品/建筑名（spice/silk/gem、palisade/harbor/keep 等）是
> **业务可读命名（设计示例）**；实际数值与 id 以 `assets/data/balance.json` 为准
> （trade.goods 1..5 数字 id、build.buildings Castle/Investment/Bank/Fort 字符串 id）。

## 商品与价格

> 以下为**示例节选**（3 种），完整数值（5 种商品，id 1..5）以 `assets/data/balance.json` 为准。
> ⚠️ 下表 id/name_key/数值为设计示例，**与 balance.json 实际值不同**；实际以 assets/data/balance.json 为准（数字 id 1..5、name_key=goods.N、buy=20/76/95/114/133、qty_max=400）。

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

## 船系统（Boat，S21.2）

> 玩家**开局是小船**，随经营成长换更大的船（用户定案 2026-10-22：B 独立船系统）。
> 船是**与建造链平行的独立成长轴**：`build.buildings` 4 条建筑链不受影响；
> 船不是建筑（不进 `BuildManager`，不走 `base_cost × level` 公式）。

### 等级链（`balance.json` `boat.levels`，4 级，数值单一事实源）

| 级 | 名（`name_key`） | 造价 `cost`（升到本级） | 货舱 `cargo`（单次贸易数量上限） | 战斗 `atk`（累计值） | 战斗 `hp`（累计值） |
|---|---|---|---|---|---|
| 1 | 小舢板（`boat.1`） | 0（开局默认） | **99**（= 现状有效上限，零行为变化） | 0 | 0 |
| 2 | 双桅快帆船（`boat.2`） | 800 | 150 | 4 | 80 |
| 3 | 三桅大帆船（`boat.3`） | 2500 | 240 | 9 | 180 |
| 4 | 巡防战船（`boat.4`） | 6000 | 400 | 15 | 320 |

### 规则（定稿）

1. **单向升级**：只能升不能降；`cost` 一次性付清，**立即生效**；满级后入口置灰
   （无操作）。升级动作 = 建造面板「造船/换船」条目（文案键 `build.boat`，随 S21.12 表现接线登记 `strings.json`；与 4 种建筑并列的第 5 入口）。
2. **货舱容量 = 单次贸易数量上限**：
   ```
   effective_qty_max(goods, boat_level) = min(goods.qty_max, boat.levels[boat_level].cargo)
   ```
   - **1 级 cargo=99 = 现状**（目录 `goods.qty_max` 现为 **400**，min 后 1 级有效上限 = min(400, 99) = 99 ⇒ 与旧行为逐条一致，开局零差异）；
   - `goods.qty_max` 99→400 的目录数据翻转**必须与规则接线同一步落地**（S21.10），
     否则中间态 = 「上限突然变 400」（回归）。S21.10 判据含「cargo[0]=99 时行为 ≡ 现状」。
     **S21.10 已落地**（`rules/boat.rs` + `rules/trade.rs` 有效上限 `min(qty_max, cargo)`
     + `rules/battle.rs` 加成叠加 + 目录翻转 400 同一步）：4 单测绿 + 1 级切片穷举 0 漂移
     （`assert cap_qty == 99` 锁定）。
   - 2~4 级 cargo>99 ⇒ 换船后单次可买/卖更多 ⇒ 贸易效率是换船的核心收益（大船 = 少跑几趟）。
3. **战斗加成**：船 `atk`/`hp` 与建筑加成**叠加**（`combat.md` §建造加成公式扩展：
   总 atk = 建筑 Σ + 船 atk；总 hp 同；BOSS 战与遭遇战均生效）。
4. **不做的事**：船不抬 `gold_max`（金量上限只归建筑/银行轴）；无维修/耐久/海战
   （船不参战，只做加成）；等级不随跨图重置（全局账号级，随存档持久化）。
5. **存档**：`SaveSlot` v3 新增 `boat_level: u8`（1..=4；v2→v3 迁移缺省 1 = 小船，
   旧档零影响；`persistence.md` §迁移 补 v3 节）。
6. **节奏**：初始 1000 金 ⇒ 2 级（800）= 第一张图自然目标（≈ 1.5 圈绕岛收入）；
   3 级（2500）≈ m01 BOSS 战后（见 BOSS 时 ~1339 金 + 战斗后结余）；
   4 级（6000）= 跨图中期。节奏表由 `scripts/board_measure.py --ladder` 扩测回填。

### 不变量（机器判定，`validate_data.py` 校验 `boat` 段）

- `levels` 长度 = 4；`levels[0].cost = 0`；
- `cargo` 严格递增 且 `cargo[0] = 99`（锁现状）；`cost[1..]` 严格递增；
- `atk`/`hp` 非递减；每级 `name_key` 在 `strings.json` 存在。

### 船系统可测试规则（S21.10 落地，单测名先行登记）

| 规则 | 单测 |
|---|---|
| 造船扣金 + 抬 cargo/atk/hp（立即生效） | `boat_upgrade_debits_and_raises` |
| 单次贸易数量上限 = min(qty_max, cargo)；1 级 ≡ 现状 | `trade_qty_capped_by_boat_cargo` |
| 等级单调（不可降级；满级无操作；金不够拒绝） | `boat_level_monotonic` |
| 船加成与建筑加成叠加进战斗 | `combat_bonus_includes_boat` |

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
  **`rules::boat`（船系统）已落地 —— S21.10 verified**（`rules/boat.rs` = `BoatSession`
  单向升级链 + `cargo/atk/hp` 查询，数值读 `balance.boat`；cargo 上限进 `rules/trade`
  有效上限 `min(qty_max, cargo)`，atk/hp 叠加进 `combat.md` §建造加成公式；
  规格 = 本文「船系统（Boat，S21.2）」节；S21.11 存档 v3 持久化 `boat_level`、
  S21.12 表现层船贴图 + 建造面板「造船」入口接线）；
- 数值注入：会话构造时接收 `&balance::Balance`，测试可注假值；
- 表现层只读 `TradeLedger` / `BuildManager` 的只读视图来画 UI，不重算价格。