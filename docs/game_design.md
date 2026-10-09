# 金银岛 TreasureIsl — 游戏设计总纲

本文件是**全库设计文档的入口与路由**。各系统的完整规格在 `docs/design/*.md`，
现代化改造方案在 `docs/modernization.md`。

> **文档分层**：
> - `docs/spec/*.md` = 逆向规格（含 IDA 证据行号，用于 diff 验证）
> - `docs/design/*.md` = **实现设计书**（从文档就能实现游戏的完整规格，不含 IDA 地址）
> - `docs/gamebook.md` = 接线规格（壳怎么把 rules 串起来）
> - `docs/modernization.md` = 现代化改造方案
> - `crates/ti-core/src/rules/*.rs` = 已验证正式实现

---

## 1. 游戏概述

| 项 | 值 |
|---|---|
| 游戏名 | 金银岛 TreasureIsl |
| 类型 | 回合制策略经营 + 探索冒险 |
| 原平台 | Windows Mobile（ARM，176×220 触屏） |
| 目标平台 | 浏览器（WASM）+ Android + 桌面 |
| 逻辑分辨率 | 176×220 像素（RGB565） |
| 字体 | `GoldLand.Fnt`，12×12 点阵汉字 |
| 角色 | 5 选 1（panda/tichy/kidd/drake/katharina） |
| 地图 | 3 张等距网格地图（20×20 / 20×20 / 15×15） |

### 1.1 一句话

玩家选择一名海盗角色和一张地图，在等距网格地图上探索据点、买卖货物、
建造城堡、翻卡牌碰事件，最终挑战 BOSS。通过贸易赚金、建造升级实力、
卡牌事件增减 HP/金，在回合制战斗中击败 BOSS。

### 1.2 全流程图

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

### 1.3 核心循环

```
探索地图 ──→ 遭遇据点/事件 ──→ 贸易赚金 ──→ 建造升级
    ↑                                         │
    └── 卡牌事件（HP/金增减） ←── 战斗挑战 ←──┘
```

---

## 2. 架构

```
                    ┌─────────────────────────────────────────┐
                    │            ti-app（表现+交互壳）          │
                    │  GameApp（场景状态机）                     │
                    │   ├─ 持有 ti-core rules 各 Session       │
                    │   ├─ 每场景：update(输入事件) + render(fb) │
                    │   └─ 跨场景跳转（SceneStack advance）     │
                    └──────────────┬──────────────────────────┘
                                   │ 只调用（不重抄数值）
                    ┌──────────────▼──────────────────────────┐
                    │            ti-core::rules（已验证）       │
                    │  MenuStateMachine / SelectionFlow /      │
                    │  MapSession / TradeLedger / BuildManager /│
                    │  BattleSession / CardEngine / SaveManager /│
                    │  MessageSession / SceneStack / GameRng     │
                    └──────────────┬──────────────────────────┘
                                   │ 读资源
                    ┌──────────────▼──────────────────────────┐
                    │  ti-formats（容器/字体/PCX/存档/音频解析）  │
                    │  ti-platform（gfx 帧缓冲 / input / audio） │
                    └─────────────────────────────────────────┘
```

---

## 3. 设计文档索引

| 文档 | 内容 |
|---|---|
| [`design/scene_flow.md`](design/scene_flow.md) | 场景状态机、主菜单、角色/地图选择、事件码体系 |
| [`design/map_system.md`](design/map_system.md) | 地图数据、地形网格、移动规则、据点/地标、事件分派 |
| [`design/economy.md`](design/economy.md) | 贸易系统（商品/价格/买卖/利润）+ 建造系统（建筑/银行/金上限） |
| [`design/combat.md`](design/combat.md) | 战斗系统（BOSS档/伤害公式/回合制）+ 卡牌事件系统（目录/效果/触发） |
| [`design/persistence.md`](design/persistence.md) | 存读档（文件布局/魔数/迁移）+ RNG（LCG算法/播种/调用点） |
| [`design/ui_controls.md`](design/ui_controls.md) | 消息界面/帮助/信息 + 控件表 + 资源容器 + 渲染管线 + 控制方案 |
| [`design/constants.md`](design/constants.md) | 全局常量表（分辨率/金上限/BOSS HP/RNG 常数/价格表等） |
| [`modernization.md`](modernization.md) | 多端现代化改造方案（双层渲染/响应式/虚拟手柄/PWA） |
| [`gamebook.md`](gamebook.md) | 可玩实现手册（rules 接进交互壳的接线规格） |

---

## 4. 实现路由

| 要实现的部分 | 调用的 rules 模块 | 接口 | 详细规格 |
|---|---|---|---|
| 主菜单 | `rules::menu` | `MenuStateMachine::handle(MenuEvent)` | scene_flow.md |
| 场景管理 | `rules::main_loop` | `SceneStack::advance(trigger)` | scene_flow.md |
| 角色/地图选择 | `rules::selection` | `SelectionFlow::select_role(ch)` / `select_map(m)` | scene_flow.md |
| 主地图移动 | `rules::main_map` | `MapSession::move_to(dir)` / `walk(dir, n)` | map_system.md |
| 主地图事件 | `rules::main_map` | `MapSession::dispatch(code)` / `at_landmark()` | map_system.md |
| 贸易 | `rules::trade` | `TradeLedger::buy(gid)` / `sell(gid)` | economy.md |
| 建造/银行 | `rules::build` | `BuildManager::build(kind, level)` / `deposit(amt)` / `withdraw(amt)` | economy.md |
| 战斗 | `rules::battle` | `BattleSession::new(seed, player, tier)` / `step()` | combat.md |
| 卡牌 | `rules::cards` | `CardEngine::apply(event_id, &mut state)` | combat.md |
| 存读档 | `rules::saveload` | `SaveManager::read_slot(bytes)` / `write_slot(&slot)` | persistence.md |
| 消息 | `rules::messages` | `MessageSession::open(kind)` / `next()` / `prev()` | ui_controls.md |
| RNG | `rules::rng` | `GameRng::new(seed)` / `next()` / `in_range(n)` | persistence.md |
| 渲染 | `rules::render` | `SceneManager::load_scene(...)` / `render_frame(&assets)` | ui_controls.md |

---

## 5. 全流程验收脚本

固定种子全流程（headless，`seed = 42`）：

```
开局 → select_role(Panda) → select_map(M01)
    → main.Map 装载
    → move_to(South) × 3 → move_to(East) × 2
    → at_landmark() == true → dispatch(1) → Msgl 据点
    → dispatch(4) → 贸易 buy(1) → gold -= 20
    → build(Castle, 0) → gold -= 1000
    → dispatch(3) → 卡牌事件 apply(rand_id, &mut state)
    → dispatch(6) → BattleSession::new(42, player, 0) → step() 到 outcome
    → write_slot(slot) → read_slot(bytes) → roundtrip
    → 回主菜单
```

断言每步场景/状态正确，0 失败。

---

## 6. 推测项

以下为推测值，实现时可先按推测，后续用实测收敛（详见各 design 文档标注）：

1. MSVC LCG 常数 214013/2531011 — coredll 导入 thunk无法实测
2. 战斗伤害公式 `(seed × 2654435761) % (atk+1) + 1` — oracle 占位
3. 效果码 1..4 → Damage/Heal/Gold/Move — 需对照 `cards\event.ini`
4. 建造成本公式 `base × (level+1)` — 占位公式
5. 5 种兵种名具体对应 — 宽字符串需对照资源包
6. 手续费 = 0 — 伪代码未见独立扣减
7. 据点触发概率 40% — `rand()%100+1 > 40`
8. 金量上限 999999 — `sub_503F0` 行 114/116
9. 卡牌价格表 17 组 — `sub_19F9C` 初值
10. 建筑类型 4 种 — 控件组合推断
