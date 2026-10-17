# 业务设计文档（Business Design）

本目录是**游戏业务规则的唯一事实源**。它描述**游戏应该怎么玩**（设计意图 +
平衡数值），不描述原版 exe 怎么实现。

> **定位（2026-10 决策）**
> 本目录是**正向设计**的业务规则唯一事实源：描述「游戏应该怎么玩」（设计意图 + 平衡数值），
> 不依赖任何逆向产物。逆向归档物（原版资源/伪代码）在 `legacy/`，仅作历史参照，
> **不作为业务规则来源、不作为验收基准**。业务规则以本目录 + `assets/data/*.json` 为准。

## 为什么这样切分

业务规则与「怎么实现」的规格必须独立：规则（设计意图 + 数值）落在本目录 + `assets/data/*.json`，
实现细节落在 `crates/`。本目录把业务规则**独立出来、可读、可调、可评审**，
数值落到 `assets/data/*.json`，实现由工程师按 `docs/process/` 规范落地。

## 分层：设计意图 vs 数值

| 层 | 写在哪 | 谁改 | 例 |
|---|---|---|---|
| **设计意图** | 本目录（散文 + 规则 + 公式结构） | 策划 / 评审 | 「伤害 = f(atk, rng)，玩家先手，回合制」 |
| **平衡数值** | `assets/data/balance.json` | 策划，改 JSON 即可 | 「boss_tiers[0].hp = 2400」（示例值，实际以 balance.json 为准） |
| **文本** | `assets/data/strings.json` | 策划 / 本地化 | 「scene.select_role.title = 选择角色」 |
| **实现** | `crates/ti-core/src/rules/` | 工程师 | `struct CombatSession`（S8.3.a–e 已 verified；`battle.rs` 数值全由 `balance.combat` 注入，不再含原版公式残留） |

改数值 = 改 JSON（不碰代码）；改机制 = 先改本目录（评审）再改代码。

## 清单

| 文件 | 系统 | 内容 |
|---|---|---|
| [`loop_core.md`](loop_core.md) | 核心循环 | 探索→贸易→建造→卡牌→战斗 的目标与节奏 |
| [`economy.md`](economy.md) | 贸易 / 建造 / 银行 | 商品、价格、买卖、建筑升级、金上限 |
| [`combat.md`](combat.md) | 战斗 | BOSS 档、伤害模型、回合流程、胜负 |
| [`progression.md`](progression.md) | 地图 / 据点 / 卡牌事件 / 跨图推进 | 网格地图、移动、探索事件、卡牌、地图解锁链与通关判定 |
| [`persistence.md`](persistence.md) | 存读档 / 设置 / RNG | 存档字段、迁移、RNG 可复现 |
| [`07_ui_flow.md`](07_ui_flow.md) | **UI 流程（S11 新增）** | 场景流/控件/键盘触摸输入/消息分页/UI 约束；表现层并行开发唯一依据 |
| [`08_layout_design.md`](08_layout_design.md) | **界面版面（S14 新增）** | 9 场景版面 + 配色表 + 字体 + 可测规则；**版面/坐标/配色的单一事实源**，改 `scenes.json` 前必读 |

## 编写约定

1. 每条规则写成**可执行的断言**（能翻成单测），而非散文描述；
2. 数值**不写在本目录**（本目录写「哪个数、为什么」，值在 `balance.json` 并在文中引用其 key）；
3. 标注**设计意图**（「为什么这么设计」）——逆向规格只有「是什么」，没有「为什么」；
4. 与实现不一致时，以本目录为准，改实现或改本目录（评审），不留两个真相；
5. 每条规则配一个单测名，便于「规则 ↔ 测试」双向追溯。

## 与其他文档的边界

- **工程怎么做**：[`docs/process/`](../process/00_index.md)（分层/约定/模板/工具）；
- **架构约束**：[`docs/process/01_architecture.md`](../process/01_architecture.md) §1.1（SceneHost 窄接口隔离）+ §5.6（Builder/Type-State/Command 模式）；
- **代码规范**：[`docs/style.md`](../style.md) §5.6（Trait 设计 + 接口隔离 + 设计模式）；
- **资源字节格式**：`docs/formats/`（pak/lpk/pcx/字体/存档的真实布局，这些是实测事实，保留）；
- **玩法意图（本文档）**：`docs/business/`；
- **原版实现（仅参照，出处标注一律带 `legacy/` 前缀）**：`legacy/docs/`、`legacy/ref/`。

## 架构约束（S9 起）

### 接口隔离（现状宽接口 → 目标窄接口，S9.2 拆分中）

`SceneHost` 现状是单一宽接口（约 60 方法，`scenes/mod.rs`），S9.2 目标按职责拆分为 8 个窄接口
（`SceneControl`/`RenderHost`/`EconomyHost`/`BattleHost`/`SelectionHost`/`MessageHost`/`SaveHost`/`MapHost`），
场景只依赖它需要的接口。详见 [`docs/process/01_architecture.md`](../process/01_architecture.md) §1.1。

### 设计模式

| 模式 | 用途 | 详见 |
|---|---|---|
| Builder | `GameApp` 复杂对象构造 | [`docs/style.md`](../style.md) §5.6.2 |
| Type-State | 场景转换编译期安全 | [`docs/style.md`](../style.md) §5.6.3 |
| Command | 输入可序列化/重放 | [`docs/style.md`](../style.md) §5.6.4 |
| Observer | 事件驱动（规划中） | — |
| ECS | 动态组合行为（规划中） | — |

### 模块边界

- `ti-formats`：纯二进制格式解析/序列化，零依赖；
- `ti-core`：纯玩法逻辑，零 IO，只依赖 `ti-formats`；
- `ti-platform`：平台抽象 trait + macroquad 后端，不依赖 `ti-core`；
- `ti-app`：组装层 + 三端 bin，依赖 `ti-core` + `ti-platform` + `ti-formats`；
- `ti-tools`：离线工具，依赖 `ti-formats`。

**禁止反向依赖**：`ti-core` 不得依赖 `ti-platform`；`ti-platform` 不得依赖 `ti-app`。