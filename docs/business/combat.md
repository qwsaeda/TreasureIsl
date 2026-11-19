# 战斗系统（Combat）

> 2026-10-30 业务逻辑大改（v2，用户定案）：战斗从「HP 回合制（攻击/道具/防御 + 随机伤害）」
> 改为**兵力模型（人手 + 武器，确定性 VS + 双方必损 + 自动动效）**。
> 数值单一事实源 = `assets/data/balance.json → combat`（兵力段新增）；
> 规则实现 = `rules::force` 兵力模型（`rules/battle.rs` 旧 HP 回合制已删 S34.6 D5）；领地/攻占接线 = [`territory.md`](territory.md) §4。

## 设计意图

VS 是**单场定胜负**的高潮场景：**英雄 + 水手对战，自动动效，然后出结果**。
胜负由**人手 + 武器**决定（确定性，可算出来，不靠随机回合制）；**比拼后双方必损**
（武器 + 水手都掉，胜者也损）。战斗刻意短（一次比较结算），避免拉锯。

## 兵力模型（v2）

### 战力合成

```
人手   crew     = 席位水手数（船容量上限 + 流浪儿/孤岛救援/奴隶商人）
武器   arms     = 席位武器数（武器商人 + 宝藏沉船；一人一把）
武装   armed    = min(人手, 武器)          # 武器 > 人手 = 多余武器无效（没人用）
徒手   unarmed  = 人手 − 武装              # 人手 > 武器 = 未武装（战力折减）

战力   power    = armed × weapon_coef + unarmed × barehand_coef
                 # weapon_coef > barehand_coef；数值在 balance.manpower
```

- **一人一把武器**：武器 > 水手没意义（无徒可持），瓶颈 = 水手；水手可 > 或 < 武器。
- 攻占/遭遇战里，进攻方用**舰队人手**，防守方用**驻军人手 `G` + 建筑防御 `defense(T,L)`**
  （`territory.md` §4.1）。

### 胜负判定（确定性）

```
attack_power = 舰队战力（进攻方）
defense_power = 驻军战力（守方）+ 建筑防御加成
attack_power > defense_power ⇒ 进攻方胜（占领，重定驻军）
否则                        ⇒ 守方胜（进攻方败：交双倍过路费）
```

- **不消耗随机数**（同 seed 同兵力 ⇒ 同结果；`rules::rng::GameRng` 可播种复现）。
- **双方必损**：胜败双方都损武器 + 水手（按 `balance.manpower.loss_pct`，
  建议 10~20% 人手 + 部分武器）。

### 结算顺序

```
比较战力 → 判胜败 → 双方必损（武器 + 水手）→ 败方交钱（双倍过路费）/ 胜方占领
```

### 卡牌效果口径（2026-11-19 拍板②：改为人手增减，对齐兵力模型）

> **审计登记**：卡牌 `Heal`/`Damage` 旧口径写 `card_state.hp`（暂存 HP，钳制 `hp_max =
> balance.combat.player.hp`），但兵力模型（`force::resolve`）战斗结算**不用 HP** ⇒ 旧
> 卡牌 HP 效果 = **死效果**（暂存不进 `seat_crew`，不影响兵力战斗）。
> **拍板②（2026-11-19）**：卡牌 `Damage`/`Heal` 改作用**当前席人手 `seat_crew`**（对齐
> 兵力模型，与商人 `buy_crew` 同账本）：`Damage(n)` = `seat_crew.saturating_sub(n)`（扣人手），
> `Heal(n)` = `seat_crew.saturating_add(n)`（加人手）。`card_state` 字段 + `combat.player`
> 消费点 2026-11-19 全删（`host.rs::apply_card_effect` 改写 `seat_crew`，`mod.rs` 删
> `card_state` 字段）。`progression.md` §卡牌事件 / `01_game_logic_overview.md` §4.2 同改口径
> （卡牌金 = 加金；血/伤 = 增减人手，非 HP 暂存）。
> **rules 层口径对齐（飞轮 R18，2026-11-19）**：app `host.rs::apply_card_effect` 改 `seat_crew`
> 后，rules 层 `rules/cards.rs::apply_card` 仍写 `CardState.hp`（HP 暂存死通道）= 两条路径口径
> 分裂（EXP-199 六面漏「rules 层」面）。R18 FIND 抓出 ⇒ `CardState` 加 `crew` 字段（删
> `hp`/`hp_max`）+ `apply_card` `Damage`/`Heal` 写 `state.crew`（与 app `seat_crew` 同口径）+
> doccheck2 第 52 项 `card-crew-alignment` 守门（防回归，EXP-200）。**EXP-199 六面扩七面 = +rules
> 层 app↔rules 双侧一致**。
> **`combat.player` 字段现状**：原「活数据字段」（卡牌 HP 钳制上限）消费点 2026-11-19 删后
> = 无 live 消费（仅 `boss_tiers` 难度阶梯 + `force` 守方 `defender_unit` 仍用 `Unit` 结构体）；
> 后续轮可评估整删 `combat.player`（不属本轮范围）。
> **`combat.item.*` i18n 孤儿（4 条）**：随 `combat.items` 表删（B-01 死内容，S34.6 D5）
> 后 `strings.json` 4 文案（`combat.item.keg/medkit/rum/smoke`）+ `chrome.rs` 4 个 vs 道具
> 图标常量残留（`vs_anim_v4` 渲染回退用）；`doccheck2` i18n 孤儿 WARN 4 条登记待清（与 B-01
> 同源，下一轮清 `strings.json` + `chrome.rs` 4 常量，非本轮）。

## 回合流程（v2 简化为一次结算）

- 攻占 / 多人同格遭遇：**进入 VS 界面 → 英雄 + 水手自动动效（双方列阵对攻）→ 出结果**
  （自动播完即结算，无逐回合玩家输入）。
- 副本 / BOSS 战：同用兵力模型（`territory.md` §11 U3），进 VS 界面自动结算。
- 原「攻击 / 道具 / 防御」三动作回合制**作废**（`combat.md` v1 §战斗动作删除）。

> **S34.6 D5（2026-11-03 用户拍板「迁移兵力模型」）**：VS 战斗（副本/BOSS/遭遇）结算
> **已从 HP 回合制（`rules::battle::BattleSession`，作废）迁移到兵力模型**
> （`rules::force::resolve`：人手+武器确定性 VS + 双方必损）。vs 场景 `Attack` 触发
> 一次性 `force::resolve(seat.fleet(), defender)`（副本/BOSS 敌手 = `defense_at(isle, level)`），
> 写回 `seat_crew/seat_arms`（attrition），结果钩同攻占（副本胜 = 占领 / BOSS 胜 = 通关）。
> 实现：`BattleSnapshot` 带兵力（`attacker_crew/arms` + `defender_crew/arms`）+ 战后
> （`after_crew/arms`）+ `resolved` 标记；vs.rs 渲染 clash 战前 / result 战后。
>
> **2026-11-19 HP 本体彻底删除（用户拍板「全删 HP 本体」，收口 S34.6 D5 迁移残留）**：
> 旧 HP 回合制代码/数据全部清理 —— 规则层 HP 会话（`BattleSession`/`CombatBonus`/`step`）
> 整删 + HP 差分基准文件（无消费方）整删 + `combat.items` 4 道具
> （`CombatItemDef`/`ItemEffect`/`CombatAction` + `balance.combat.items[]` + `combat.item.*` i18n）
> 全删（B-01 死内容：4 道具无获取路径，兵力模型无逐回合道具机制）+ 3 纯 HP 测试
> （`rules_battle`/`battle_rules`/`exhaust_combat`）+ 3 example（`full_run`/`cross_map`/
> `random_session`）战斗段改 `force::resolve` 一次性。保留面：`boss_tiers` 表（难度阶梯数据，
> `defender_force` 用 `boss_tier+1` 查 `boss_fort` 防御）/ `Unit`/`CombatCfg.player`（活数据字段）/
> `combat.anim.items`（道具动效表，渲染 `vs_anim_v4` 缺省回退）。`vs_duel.rs`（引擎渲染原型）
> 仍保留作动效骨架参考（见 F-05，0 live 调用，不参与兵力结算）。
>
> **S38 审计 F-05（2026-11-05）——`vs_duel.rs` 模块状态判定**：`crates/ti-engine/src/vs_duel.rs`
> （S27.7 横版对峙动效骨架）实现的是**作废的 HP 回合制原型**（`DuelStage.player_hp=2000` /
> 三动作 `Attack/Item/Defend` / 4 道具 / HP 条 + 冲刺/定格/震动动效），与兵力模型业务
> **不冲突**（仅 `pub mod` 导出，**无 live 调用点**，不参与 VS 场景结算）。处置：**保留作动效
> 骨架参考**（冲刺/hit-stop/伤害上飘/屏幕震动可复用到兵力模型的自动动效），**不贸然删除**；
> 其硬编码（位图烘焙色/时序参数）已集中 `ui.rs::palette`（F-04）。**活动 VS 业务 =**
> `rules::force::resolve` 兵力模型 + vs.rs 渲染（本文件 §兵力模型）。

## VS 界面（按地图 / 格类变化，好好设计）

> 用户 2026-10-30：「VS 界面是按地图、按不同类格有变化的」。VS 场景不是单一模板，
> 按**战斗类型（格类）**出不同画面 + 按**地图**换主题资产。设计单一事实源 =
> `ui-redesign-v3/12_main_map_v6_vs_v2_spec.md` §2（VS v2 重设计，2026-10-31 用户否决旧稿后定案）；
> `11_vs_spec.md` §4（5 套版式 A/B/C/D/E）已降级为版式选型参考（该文档 §2 注记「被 v2 兵力模型取代」）。

| 战斗类型（格类） | VS 画面 | 主题资产 |
|---|---|---|
| **攻占战**（他人格） | 船队**攻打建筑**（舰队人手 vs 驻军+建筑防御）：海面舰炮 + 建筑攻城 | 地图特有建筑/城墙 |
| **副本战**（无主格） | 英雄 + 水手 **登岛**打守军：滩头/洞府/崖顶 | 地图特有岛屿地形 |
| **BOSS 战**（`boss_fort`） | 英雄 vs 要塞炮台（巨型 BOSS）：炮台特写 | 地图特有 BOSS 要塞 |
| **多人同格遭遇**（后到者） | 英雄 + 水手**对决**（双方列阵）：甲板/海面对峙 | 地图特有甲板/海面 |
| **事件**（功能格） | 不进 VS（随机事件直接结算 + 提示） | — |

- **按地图换主题**：每张图 `map.json`/`manifest` 带 VS 主题（背景/氛围/BOSS 造型），
  VS 场景读包主题（与换图同源，`21_map_package_spec.md`）。
- **自动动效**：双方列阵 → 对攻（炮火/刀光）→ 损耗结算 → 结果横幅；全程自动，无逐回合输入。
- 落地按 `12_main_map_v6_vs_v2_spec.md` §2（VS v2 重设计）逐格类出图。

## 参数（在 `assets/data/balance.json`）

```jsonc
"manpower": {
  "weapon_coef": 2,          // 武装人手战力系数（整数倍）
  "barehand_coef": 1,        // 徒手人手战力系数（整数倍）
  "loss_pct": 1500,          // 每战双方人手损耗比例（**万分比 u32**，1500 = 15%）
  "weapon_loss_pct": 2000    // 每战双方武器损耗比例（万分比，2000 = 20%）
}
```

## 建造/船只加成（成长机制）

> 领地建筑升级增防御（`territory.md` §4.2）；船只等级 = **人手容量上限**
> （`economy.md` §船系统：船越大可带水手越多）。战力本身只由人手 + 武器决定，
> 建筑防御只加防守方 `defense_power`。

## 可测试规则（每条 → 单测名）

| 规则 | 单测（`rules/force.rs` 内联 `#[cfg(test)]` + `tests/territory_v2_rules.rs`；旧 HP `rules_battle.rs` 已删 S34.6 D5，本表指向兵力模型） |
|---|---|
| 武装 = min(人手, 武器)；武器多余无效 | `armed_power_caps_at_manpower`（tests/territory_v2_rules.rs） |
| 人手 > 武器 ⇒ 徒手战力折减 | `power_weights_armed_over_barehand`（force.rs 内联） |
| 战力确定性（同兵力同结果，不耗随机） | `deterministic_same_input_same_result`（force.rs 内联） |
| 攻方战力 > 守方 ⇒ 攻方胜 | `attacker_power_wins`（force.rs 内联） |
| 守方战力 ≥ 攻方 ⇒ 守方胜 | `defender_wins_on_tie_or_higher`（force.rs 内联） |
| 双方必损（人手 + 武器都掉） | `both_sides_attrition`（force.rs 内联） |
| 败方交双倍过路费 | `siege_loss_pays_double_toll` |
| 胜方占领重定驻军 | `siege_win_recaptures_with_level_one` |
| 同兵力同参数 ⇒ 同结果（确定性） | `deterministic_same_input_same_result`（force.rs 内联，兵力模型 `force::resolve` 无随机） |

## 数值平衡指引（非规则，供调参参考）

- 武器系数 >> 徒手系数 ⇒ 武器是战力核心，但受人手上限约束；
- `loss_pct` 调大 ⇒ 战斗更消耗（鼓励少打 / 多备水手）；
- 初始人手 / 船容量 / 武器价 / 水手价见 `territory.md` §11 U4。
