# 金银岛 TreasureIsl — 游戏设计文档

本文件是**实现级规格**的入口。目标：读者仅凭本目录文档 + 资源格式文档，即可
从头实现出完整可玩游戏。

> **文档分层**
> - 本文件 = 概述与路由
> - `docs/formats/` = 资源字节级格式（pak/lpk/pcx/map/font/save）
> - `docs/design/` = 各玩法系统的实现级规格（含公式、状态机、数据表）
> - `docs/spec/*.md` = 逆向规格（含 IDA 证据行号，用于 diff 验证）
> - `crates/ti-core/src/rules/*.rs` = 已验证正式实现

---

## 1. 游戏概述

| 项 | 值 |
|---|---|
| 游戏名 | 金银岛 TreasureIsl |
| 类型 | 回合制策略经营 + 探索冒险 |
| 原平台 | Windows Mobile（ARM PE32，176×220 逻辑分辨率） |
| 目标平台 | 浏览器（WASM）+ Android + 桌面 |
| 玩家角色 | 5 选 1（panda/tichy/kidd/drake/katharina） |
| 地图 | 3 张等距网格地图（20×20 / 20×20 / 15×15） |

### 1.1 一句话

玩家选择一名海盗角色和一张地图，在等距网格地图上探索据点、买卖货物、
建造城堡、翻卡牌碰事件，最终挑战 BOSS。通过贸易赚金、建造升级实力、
卡牌事件增减 HP/金，在回合制战斗中击败 BOSS。

### 1.2 核心循环

```
探索地图 ──→ 遭遇据点/事件 ──→ 贸易赚金 ──→ 建造升级
    ↑                                         │
    └──────── 卡牌事件（HP/金增减） ←── 战斗挑战 ←──┘
```

### 1.3 全流程图

```
启动 → intro.Map 片头 → index.Map 主菜单
  ├─ [新游戏] → selectrole.Map 选角 → selectmap.Map 选图
  │    → main.Map 主地图探索
  │      ├─ 移动 → 据点事件(msgl) → 贸易 → 建造/银行 → 卡牌事件
  │      └─ 战斗(vs.Map) → 胜/败 → 回主菜单
  ├─ [继续] → saveload 读档
  ├─ [选项] → Option.map
  ├─ [说明] → help.map（3 页）
  └─ [退出]
```

---

## 2. 系统设计索引

| 系统 | 文档 | 内容 |
|---|---|---|
| 主循环/场景状态机 | [`design/main_loop.md`](design/main_loop.md) | 场景枚举、状态机、跨场景状态、实现路由 |
| 输入映射 | [`design/input_mapping.md`](design/input_mapping.md) | 物理键/触摸 → 逻辑事件 → 场景行为 |
| 地图/移动 | [`design/map_system.md`](design/map_system.md) | 地形网格、等距投影、移动规则、摄像机 |
| 贸易/经济 | [`design/economy.md`](design/economy.md) | 商品目录、价格表、买卖公式、银行 |
| 建造 | [`design/economy.md`](design/economy.md) | 建筑类型、成本公式、金量上限 |
| 战斗 | [`design/combat.md`](design/combat.md) | BOSS 档、伤害公式、回合制流程、胜负判定 |
| 卡牌/事件 | [`design/combat.md`](design/combat.md) | 卡牌目录、效果码、触发、事件 id 映射 |
| 消息/界面 | [`design/ui_controls.md`](design/ui_controls.md) | 消息类型、翻页状态机、对话框布局 |
| 控件表 | [`design/ui_controls.md`](design/ui_controls.md) | 29 按钮、6 字段 .Map 记录、资源容器 |
| 持久化/RNG | [`design/persistence.md`](design/persistence.md) | 文件布局、魔数、v1/v2 迁移、LCG 算法（oracle 历史参照）/PCG（当前实现） |

---

## 3. 数据源

| 类别 | 文件 | 说明 |
|---|---|---|
| UI 场景布局 | `*.Map` ×20 | 控件记录 6 字段，头部 16 字节 |
| 精灵 | `ui.pak` / `glpics.pak` | PCX 贴图 |
| 地形 | `glmap_m01/02.pak` | 瓦片 PCX |
| 地形布局 | `glmapm01/02/03.Map` | stride=9 记录流 |
| 音频 | `music.lpk` | MOD → OGG 已转码 |
| 设置 | `settings.dat` | 8 字节 |
| 存档 | `gl0~4.sav` | 可变长度 |
| 记录 | `gl.rec` | 10 字节 |

完整格式规格见 [`docs/formats/lucencypackage.md`](../formats/lucencypackage.md)。

---

## 4. 实现路由

| 要实现的部分 | 调用 | 详细规格 |
|---|---|---|
| 主菜单 | `rules::menu` | main_loop.md |
| 场景管理 | `rules::main_loop` | main_loop.md |
| 角色/地图选择 | `rules::selection` | main_loop.md |
| 主地图移动 | `rules::main_map` | map_system.md |
| 贸易 | `rules::trade` | economy.md |
| 建造/银行 | `rules::build` | economy.md |
| 战斗 | `rules::battle` | combat.md |
| 卡牌 | `rules::cards` | combat.md |
| 存读档 | `rules::saveload` | persistence.md |
| 消息 | `rules::messages` | ui_controls.md |
| RNG | `rules::rng` | persistence.md |
| 渲染 | `rules::render` | ui_controls.md |

---

## 5. 验收标准

1. **场景全覆盖**：9 个场景（Intro..Exit，Exit 为终态）均可进入/返回，无死锁
2. **输入正确**：每个按键在每个场景的行为符合映射表
3. **资源注入**：ui.pak + 9 个场景 .Map 正确加载并渲染
4. **三端编译**：native/wasm/android 均可编译
5. **数值精确**：价格表、BOSS HP、伤害公式、RNG 序列逐位一致（当前实现为 PCG，`rand_pcg`，S8.7；旧 MSVC LCG 序列为 oracle 历史参照）
6. **全流程**：开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单
