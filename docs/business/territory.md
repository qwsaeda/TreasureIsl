# 领地系统（Territory）— 业务规则单一事实源

> **本文件 = 领地/副本/对手机制（v5 主地图大富翁化）的业务规则唯一事实源**。
> 数值全部落在 `assets/data/balance.json`（本文件只写「哪个数、为什么」与公式结构）；
> 版面/坐标另见 [`../ui-redesign-v3/10_main_map_v5_spec.md`](../ui-redesign-v3/10_main_map_v5_spec.md)；
> 赛道与跨图推进见 [`progression.md`](progression.md)。
>
> 触发：用户 2026-10-27「参考大富翁但要海盗风，各类岛屿都不知道」+「每一格有岛屿或城镇，
> 落到这格可以进入副本；别人占领的可交钱或攻占；自己的领地可以升级加固——这就是原来的玩法」
> +「每局 4 个 AI 玩家」+「可以选至少一个玩家」。
> 原版依据（逆向参照，非验收基准）：`docs/impl/04_economy.md` §2.1 **Fort（据点/堡垒）**
> 造价 800、控件 `button_capture` + `button_build`；`docs/impl/07_ui_controls.md` 另见
> `button_notcapture`（不占领）⇒ 原版确有「占领据点」这条业务线，移植时丢失，本文件补回。

## 1. 席位与对局结构（1..5 玩家，其余 AI）

| 项 | 规则 | 值来源 |
|---|---|---|
| 席位池 | 5 个角色（`panda` / `tichy` / `kidd` / `drake` / `katharina`） | `balance.selection.characters[].id` |
| 选角 | 玩家在选角界面选 **≥1** 名（1..5）作为**玩家席位**；未选中的角色自动成为 **AI 席位** | `balance.selection` |
| 典型局 | 选 1 名 ⇒ **4 个 AI 对手**（= 原版「每局 4 个 AI 玩家」） | 设计意图 |
| 回合顺序 | 席位按 `characters[].index` 升序循环 | `balance.selection.characters[].index` |
| 热座 | 同一设备依次接管：**每次交接必须显式「开始回合」确认**（防止误操作），AI 回合自动播放 | 设计意图 |
| 独立性 | 每个席位有独立金币 / 领地 / 建筑 / 船只 / 位置；玩家被 AI 收过路费、被夺格都会真实发生 | 设计意图 |

> **不变量**：`players + ai = 5`；`players ≥ 1`；同局内同一席位不得既是玩家又是 AI。

## 2. 赛道格两类（领地格 / 功能格并存）

| 类 | 含义 | 可占领 | 落格结算 |
|---|---|---|---|
| **领地格** `territory` | 10 类海盗岛原型（岛屿 / 城镇 / 要塞） | ✓ | 按 §4 三支分叉 |
| **功能格** `feature` | 事件 / 卡牌 / 宝藏 / BOSS | ✗ | 照旧：抽卡、宝藏金、BOSS 战 |

每图赛道 = **31 格**（大富翁式：每类岛 2 格相邻成区）：

| 格类 | 数量 | 构成 |
|---|---|---|
| 起点 `start` | 1 | 港口城码头（起点即 1 号领地，初始无主） |
| 领地格 | 20 | 10 类岛 × 2 格 |
| 功能格 | 9 | 事件 3 / 卡牌 3 / 宝藏 2 / BOSS 1 |
| 合计 | 31 | — |

> 赛道格数 45/46/33 → 31 是**业务规则变更**（节奏变快），须用 `scripts/board_measure.py`
> 复测并同步 [`progression.md`](progression.md) §落格节奏表 + 必要时调 `balance.board` 产出值。

## 3. 10 类岛原型（每类 = 领地格 2 格 + 一个专属副本场景）

| # | 原型 id | 中文名 | 副本场景 | 守军 | 占领后每圈产出 | 加固上限 | 岛型色（令牌） |
|---|---|---|---|---|---|---|---|
| 1 | `harbour` | 港口城 | 抢滩登陆战 | 海盗水手 | 贸易抽成 | 3 | `isle_coral` |
| 2 | `lighthouse` | 灯塔礁 | 灯塔夺灯 | 走私贩 | 航行加速（少 1 格结算） | 2 | `isle_sand` |
| 3 | `wreck` | 藏宝沉船 | 沉船夺宝 | 守宝骷髅 | 宝藏金 | 3 | `isle_gold` |
| 4 | `pirate_lair` | 海盗老巢 | 攻陷老巢 | 头目亲卫 | 掠夺金 | 3 | `isle_dark` |
| 5 | `shipyard` | 帆船坞 | 夺船坞 | 船坞工头 | 造船折扣 | 2 | `isle_teal` |
| 6 | `trading_post` | 军需商岛 | 商岛护镖 | 私掠队 | 贸易利润 + | 3 | `isle_olive` |
| 7 | `skull_cliff` | 骷髅崖 | 崖上营寨 | 骷髅兵 | 战利品 | 2 | `isle_bone` |
| 8 | `volcano` | 火山岛 | 熔岩洞府 | 熔岩守卫 | 熔金 | 3 | `isle_ember` |
| 9 | `lightship` | 灯船站 | 登船夺旗 | 灯船船员 | 骰点 +1（下回合多走 1 格） | 2 | `isle_mist` |
| 10 | `boss_fort` | 无畏号要塞 | 无畏号炮台 | BOSS 舰队 | 通关关键（击败要塞 = 本图 BOSS） | 3 | `isle_black` |

- 每类的**两格同属一个领地**：占领其中任一格 ⇒ 该类两格同归该席位（Monopoly 式「买下整区」）。
- 副本 = **每类岛一个专属场景**（10 个新场景）；副本胜利 = 占领，失败 = 无归属变化（仅战斗损耗）。
- 岛型色用于棋盘分区色带 + 副本场景氛围 + 据点岛 HUD 图标（三处同源，`balance.territory.isle_tokens`）。

## 4. 落格结算三支（核心规则）

落到**领地格** `T`（owner = `O`，level = `L`）时：

| 情形 | 玩家可选 | 结果 |
|---|---|---|
| **T 无主** | ① 进入副本 | 副本胜利 ⇒ `O = 你`，`L = 1`（该类两格同时易主）；失败 ⇒ 无变化 |
| **T 属于他人 X** | ① 交钱（付过路费）<br>② 攻占（战斗） | 交钱：付 `toll(T,L)` 给 X ⇒ 本圈该格产出归 X，你通行无收益<br>攻占：战斗（守军 = 原型守军 × `defense_per_level^L`）⇒ 胜：`O = 你`，`L = 1`（夺回降级）；败：按现有战斗结算扣 HP / 金 |
| **T 属于你** | ① 升级加固<br>② 领取产出 | 升级：付 `upgrade_cost(T,L)` ⇒ `L+1`（≤ 上限）⇒ 产出/守军/过路费同步提高<br>领取：获得该格本圈产出 `production(T,L)` |

落到**功能格**时，行为与现行实现一致（事件/卡牌 = 抽卡、宝藏 = 加金、BOSS = BOSS 战）。

### 4.1 公式（数值在 `balance.territory`）

```
toll(T, L)        = toll_base(T) * (1 + toll_per_level * L)
upgrade_cost(T,L) = upgrade_cost_base(T) * (1 + upgrade_cost_per_level * L)
production(T, L)  = production_base(T) * (1 + production_per_level * L)
defense(T, L)     = defense_base(T) * (1 + defense_per_level * L)
```

- `toll_base` / `production_base` / `upgrade_cost_base` / `defense_base` **按原型 id** 取值
  （`balance.territory.prototypes[<isle_id>]`），`*_per_level` 为全局系数。
- 加固等级上限 = §3 表「加固上限」（`balance.territory.max_level` 全局 3，`lighthouse/shipyard/skull_cliff/lightship` 降为 2）。

### 4.2 交钱 / 攻占 的选择权

- 选择权归**落到格子的席位**（玩家 = 弹二选一菜单；AI = 按 §6 策略自动选）。
- 交钱后**本圈不再领取该格产出**（产出归所有者），避免「既交钱又拿收益」双吃。

## 5. 中央据点岛（玩家大本营）

- 中央大岛 = **当前玩家席位的据点**（可点击 → 据点菜单：建造 / 造船 / 贸易 / 升级本家）。
- 据点岛本身**不是赛道格**（不参与摇骰前进），但提供「升级本家」入口
  （`territory.home_upgrade` 与领地加固同族、同公式族）。
- 其他席位的据点不显示在中央（避免信息过载）；可在副本/攻占战里见到对方要塞。

## 6. AI 席位回合规则

| 项 | 规则 |
|---|---|
| 触发 | 玩家结束回合 ⇒ 按席位顺序轮转；轮到 AI ⇒ AI 自动行动（动画 + 提示签条） |
| 行动 | AI 与玩家走**同一套**落格结算规则（同一 `rules::territory`），仅**选择策略**不同 |
| 策略参数 | `balance.territory.ai.attack_threshold`（对方领地防守强度低于此值 ⇒ 攻占）、`ai.toll_aversion`（宁可交钱也不攻）、`ai.upgrade_priority`（自己领地收益低于阈值 ⇒ 优先升级） |
| 禁止 | AI 不得读玩家内存、不得无视规则、不得获得玩家不存在的资源 |
| 可见性 | AI 行动结果必须在主地图可见（领地牌换主 + 提示签条 + toast） |

> **不变量**：AI 与玩家共用规则函数（`territory_settle`），策略只提供「选哪个分支」参数；
> 单测断言「AI 走的是同一分支表」。

## 7. 经济接入

| 来源 | 变化 | 数值键 |
|---|---|---|
| 领地产出 | 占领后每圈按 `production(T,L)` 领金 | `balance.territory.prototypes[*].production_base` |
| 过路费 | 他人路过你领地 → 你收 `toll(T,L)` | `territory.toll_base` / `toll_per_level` |
| 副本收益 | 占领成功 = 归属变更 + 该类首占奖励 `first_capture_bonus` | `territory.first_capture_bonus` |
| 建造 | 现有 4 类建筑（银库 / 城堡 / 炮台 / 投资）不变；新增「领地加固」为独立支线（不占建筑槽） | `territory.*` |
| 船只 | 船只仍只影响贸易数量上限与战斗加成；**不影响**领地攻防（赛道是海路） | `economy.boat`（不变） |
| 节奏 | 31 格 ⇒ 每圈更短 ⇒ 产出/圈数需重算（`scripts/board_measure.py`） | `balance.board` |

## 8. 存档（`SaveSlot` v4）

| 字段 | 类型 | 含义 |
|---|---|---|
| `seats` | `Vec<SeatSave>` | 每席位：角色 id / 类型（player/ai）/ 金币 / 位置格索引 / 船只等级 / 建筑 |
| `territories` | `map → [格索引 → {owner_seat, level}]` | 领地图（`owner_seat = 0..4` 席位下标；无主 = 缺省） |
| `current_seat` | `u8` | 当前轮到哪个席位（热座交接用） |
| `unlocked_mask` | `u8` | 沿用 v2/v3（逐图解锁 + 通关位） |

- 迁移：v3 → v4 缺省 = 单席位（玩家）+ 空领地图 + `current_seat = 0`；v2 → v3 迁移链**保持不变**。
- 存档必须能完整还原「谁占哪格 / 各席位金币 / 轮到谁」。

## 9. 与既有系统的关系

| 系统 | 关系 |
|---|---|
| 战斗 `rules::battle` | 副本守军战 / 攻占战 / BOSS 战共用同一战斗模型（`balance.combat` + `boss_tier`）；副本守军 = 原型 `defense_base` |
| 建造 `rules::build` | 建筑是**个人**成长线；领地是**地图**成长线；两者独立叠加（战斗加成相加） |
| 船只 `rules::boat` | 只影响贸易上限与战斗加成，不影响领地 |
| 跨图推进 `rules::progression` | 击败本图 BOSS = 击败「无畏号要塞」（`boss_fort`）⇒ 解锁下一图；**被 AI 击败要塞不影响解锁**（只有玩家胜才解锁） |
| 卡牌 `rules::cards` | 功能格抽卡不变；副本内不再抽卡（避免双重随机） |

## 10. 可测规则表（规则 ↔ 单测双向追溯）

| 规则 | 单测名（`crates/ti-core/tests/` 或模块内） |
|---|---|
| 选 ≥1 玩家，其余补 AI，合计 5 | `seats_split_players_and_ai_to_five` |
| 席位顺序 = 角色 index 升序 | `seat_order_follows_character_index` |
| 无主格副本胜利 ⇒ 易主 + level 1，两格同易主 | `capture_wins_takes_both_cells_of_isle` |
| 副本失败 ⇒ 归属不变 | `capture_loss_keeps_ownership` |
| 他人格交钱 ⇒ 付 toll 给所有者，本圈产出归所有者 | `paying_toll_transfers_output_to_owner` |
| 他人格攻占胜利 ⇒ 夺回且 level 降为 1 | `siege_win_recaptures_with_level_one` |
| 自己格升级 ⇒ level+1 且产出/守军/过路费同步提高 | `upgrade_raises_output_defense_and_toll` |
| 升级不超上限 | `upgrade_caps_at_isle_max_level` |
| 每图赛道 = 31 格 = 起点 1 + 领地 20 + 功能 9 + BOSS 1 | `v5_track_has_31_cells_composition` |
| 每类岛两格同属一个领地 | `isle_cells_share_one_territory` |
| AI 走同一分支表（策略只选分支） | `ai_uses_same_settlement_table` |
| 存档 v3 → v4 迁移 = 单席位 + 空领地 | `save_migrate_v3_to_v4_defaults` |
| 存档往返还原归属/金币/当前席位 | `save_roundtrip_restores_territories` |

## 11. 未决项（待拍板 / 待实测）

| # | 未决 | 影响 | 当前处置 |
|---|---|---|---|
| U1 | 副本场景 10 个的**具体玩法**（战斗 vs 解谜 vs 资源点） | 10 个新场景的工作量与可玩性 | 先按「战斗 + 1 个专属机制」落地：抢滩登陆战（战斗）、沉船夺宝（战斗 + 采集）、灯塔夺灯（战斗 + 点灯 3 处）……逐个定案 |
| U2 | 交钱过路费**具体数值**（`toll_base` 量级） | 与 1000 初始金、trade 单次上限 400 的关系 | 按 `toll_base ≈ 80~150` 起调，用 `board_measure.py` 复测玩家收益曲线后再定稿 |
| U3 | 领地格 20 个 + 功能格 9 个的**分区排布**（每类 2 格是否相邻、BOSS 位置） | 赛道体验（连片 vs 跳跃） | 每类 2 格相邻（Monopoly 式），BOSS 区置于对角；S28.3 数据层落 `board.json` 时定稿 |
| U4 | AI 难度 | 4 AI 同局时的压迫感 | 单档参数（`ai.*`），后续按实测调 |
| U5 | 热座是否保留（选 2..5 人） | 是否要交接 UI 与防误操作 | 保留（选角 ≥1），交接提示按 §1 落 |

## 12. 改动面声明（硬规则 7）

| 块 | 本规格（S28.0） | 落码任务 |
|---|---|---|
| 业务规则 | **有**（领地/副本/对手/经济接入/存档）= 本文件 | — |
| 数据 | 无（数值先落文档，落码时写 `balance.json` / `board.json` / `strings.json`） | S28.3 |
| 规则 | 无（规格先行） | S28.1（`rules/territory.rs`）、S28.2（存档 v4）、S28.7（AI） |
| 表现 | 无 | S27.4（v5 环形棋盘）、S28.6（副本场景） |
| 资产 | 无 | S27.2（10 类岛图 + 中央插画）、S28.6（副本场景美术） |

**跨模块原因**：领地状态同时被「落格结算（规则）」「棋盘渲染（表现）」「副本入口（场景流）」
「存档（持久化）」「AI（回合驱动）」五面读写，任何一面单独改都会出现「显示有主但规则无主」
或「规则易主但存档不存」。故本文件先定案，再按 §10 单测表逐条落地。
