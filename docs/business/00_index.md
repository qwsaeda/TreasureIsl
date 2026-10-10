# 业务设计文档（Business Design）

本目录是**游戏业务规则的唯一事实源**。它描述**游戏应该怎么玩**（设计意图 +
平衡数值），不描述原版 exe 怎么实现。

> **与逆向产物的关系（2026-10 决策）**
> 逆向产物（`docs/facts/`、`docs/spec/`、`ref/pseudocode/*.c`、`crates/ti-core/src/oracle/`）
> 已降级为**参照物**：可以看看原版长什么样，但**不作为业务规则来源、不作为验收基准**。
> 业务规则以本目录 + `assets/data/*.json` 为准。详见 [`plan-s8-refactor.md`](../plan-s8-refactor.md) §1。

## 为什么这样切分

历史上业务规则写在 `oracle/`（差分测试基准）+ `docs/design/`（实现级规格）里，
两者都源于逆向伪代码，导致「猜的公式 → 自己验证自己 → 全绿」的假对齐。
本目录把业务规则**独立出来、可读、可调、可评审**，数值落到 `assets/data/*.json`。

## 分层：设计意图 vs 数值

| 层 | 写在哪 | 谁改 | 例 |
|---|---|---|---|
| **设计意图** | 本目录（散文 + 规则 + 公式结构） | 策划 / 评审 | 「伤害 = f(atk, rng)，玩家先手，回合制」 |
| **平衡数值** | `assets/data/balance.json` | 策划，改 JSON 即可 | 「boss_tiers[0].hp = 50000」 |
| **文本** | `assets/data/strings.json` | 策划 / 本地化 | 「scene.select_role.title = 选择角色」 |
| **实现** | `crates/ti-core/src/rules/` | 工程师 | `struct CombatSession`（S8.3.a–e 重写中，当前 `battle.rs` 仍含 oracle 公式残留，S8.3.a pending） |

改数值 = 改 JSON（不碰代码）；改机制 = 先改本目录（评审）再改代码。

## 清单

| 文件 | 系统 | 内容 |
|---|---|---|
| [`loop_core.md`](loop_core.md) | 核心循环 | 探索→贸易→建造→卡牌→战斗 的目标与节奏 |
| [`economy.md`](economy.md) | 贸易 / 建造 / 银行 | 商品、价格、买卖、建筑升级、金上限 |
| [`combat.md`](combat.md) | 战斗 | BOSS 档、伤害模型、回合流程、胜负 |
| [`progression.md`](progression.md) | 地图 / 据点 / 卡牌事件 | 网格地图、移动、探索事件、卡牌 |
| [`persistence.md`](persistence.md) | 存读档 / 设置 / RNG | 存档字段、迁移、RNG 可复现 |

## 编写约定

1. 每条规则写成**可执行的断言**（能翻成单测），而非散文描述；
2. 数值**不写在本目录**（本目录写「哪个数、为什么」，值在 `balance.json` 并在文中引用其 key）；
3. 标注**设计意图**（「为什么这么设计」）——逆向规格只有「是什么」，没有「为什么」；
4. 与实现不一致时，以本目录为准，改实现或改本目录（评审），不留两个真相；
5. 每条规则配一个单测名，便于「规则 ↔ 测试」双向追溯。

## 与其他文档的边界

- **工程怎么做**：[`docs/process/`](../process/00_index.md)（分层/约定/模板/工具）；
- **资源字节格式**：`docs/formats/`（pak/lpk/pcx/字体/存档的真实布局，这些是实测事实，保留）；
- **玩法意图（本文档）**：`docs/business/`；
- **原版实现（仅参照）**：`docs/facts/` + `ref/pseudocode/`。