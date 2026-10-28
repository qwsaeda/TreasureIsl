# 第 8 章：ECS vs 组件化

## 学习目标

- [ ] 理解 ECS（Entity-Component-System）的概念
- [ ] 理解"域 + 规则"模式（Domain + Rules）与纯 ECS 的区别
- [ ] 理解为什么本项目用 Bevy ECS（渲染层）+ 域/规则（逻辑层）的混合架构
- [ ] 能根据项目规模选择合适的架构

**关键洞察（30 秒版）：**

```
  ① ECS = "积木 + 组装台"（数据分离 + 系统遍历）
     适合：1000+ 同构实体（子弹、单位、建筑）

  ② 域 + 规则 = "一个对象 + 一堆函数"（数据内聚 + 业务函数）
     适合：实体少但行为复杂（RPG、回合制）

  ③ 本项目 = 两层：
     · 渲染层（ti-engine）= Bevy ECS（entity/component/system 真 ECS）
     · 逻辑层（ti-core）= 域 + 规则（domain struct + rules 函数）
     两层不混：逻辑层不依赖 Bevy，渲染层不写业务

  ④ 为什么不用纯 ECS 做逻辑？
     实体少（玩家 + 几个 NPC）+ 回合制（不用每帧更新）
     = 纯 ECS 的"缓存友好/并行"优势用不上
     = 域 + 规则的"数据内聚/好调试"优势正好用上
```

---

## 一、什么是 ECS

**生活类比：乐高积木 + 组装台。**

```
  积木块 = Component（组件）
    轮子、车窗、车门、发动机...
    纯数据，不含逻辑

  玩具车 = Entity（实体）
    只是一堆组件的编号，本身不含数据
    0 号：[轮子, 车窗, 车门, 发动机]
    1 号：[轮子, 车窗, 飞机翼]

  组装台 = System（系统）
    把"有轮子+有发动机"的组装成"能跑的车"
    把"有车门+有车窗"的组装成"能坐人的车"

  好处：想要新车？换几个组件就行，不用重新拼
```

**形式化**：

```
  Entity：只是一个编号（u64），不含数据
  Component：纯数据（struct），不含逻辑
  System：遍历"有某组组件的实体"，执行逻辑

  例：
    跑动系统：for entity in 有[轮子+发动机]的 { 计算速度 }
    载人系统：for entity in 有[车门+车窗]的 { 检查座位 }
```

### ECS 的优势和劣势

| 优势 | 说明 | 劣势 | 说明 |
|---|---|---|---|
| 组合灵活 | 换组件 = 换行为（加螺旋桨 = 直升机） | 学习曲线陡 | 要先理解"组装台"概念 |
| 缓存友好 | 相同组件连续存放（SoA），CPU 命中率高 | 过度设计风险 | 拼一只小狗也要搭组装台 |
| 并行友好 | 不同组装台可同时工作 | 调试困难 | 数据分散，出问题难找 |

**关键：ECS 的核心优势（缓存友好 + 并行）只在"大量同构实体"时才体现。**
只有 10 个实体的游戏，ECS 的优势 = 0，成本（学习/调试）= 全。

---

## 二、"域 + 规则"模式（Domain + Rules）

**生活类比：一个对象 + 一堆操作它的函数。**

```
  域（Domain）= 数据（"有什么"）
    玩家 { 血量, 金币, 等级, 背包 }
    地图 { 格子, 建筑, 资源 }

  规则（Rules）= 逻辑（"怎么变"）
    攻击(玩家, 敌人) → 扣血
    贸易(玩家A, 玩家B) → 转移金币
    落格(玩家, 格子) → 结算收益

  和 OOP 的区别：
    OOP：数据 + 逻辑在一个类里（class Player { hp, attack() }）
    域+规则：数据在 domain，逻辑在 rules（分离）
    好处：数据可以独立序列化/验证，逻辑可以独立测试
```

**为什么不用 OOP（class）而用"域 + 规则"？**

```
  Rust 没有 class，用 struct + 自由函数
  但"数据在 domain，逻辑在 rules"和 OOP 的"数据+逻辑在一个类"不同：
    · domain 的 struct 尽量"纯数据"（少方法）
    · rules 的函数"操作 domain 的 struct"（逻辑集中）
    · 好处：domain 可以独立序列化（存档），rules 可以独立测试（不依赖渲染）
```

---

## 三、ECS vs 域 + 规则 对比

| 维度 | ECS | 域 + 规则 |
|---|---|---|
| 数据存放 | 积木分类存（SoA）：所有"轮子"放一起 | 整幅画放一起（AoS）：玩家的 hp/金币/背包在一个 struct |
| 逻辑位置 | 独立系统（组装台） | 独立函数（规则模块） |
| 缓存友好 | 高（同组件连续） | 中（一个 struct 的字段连续） |
| 适合规模 | 大（1000+ 同构实体） | 中（< 100 实体，行为复杂） |
| 典型项目 | 弹幕、RTS、模拟城市 | RPG、回合制、策略 |
| 调试 | 难（数据分散） | 容易（数据内聚） |

**选择规则（一条就够）**：

```
  实体数量 > 1000 且同构（几千颗子弹/单位）→ ECS
  实体数量 < 100 且异构（每个角色行为不同）→ 域 + 规则
```

---

## 四、本项目：Bevy ECS（渲染）+ 域/规则（逻辑）

**关键认知：Bevy 本身就是 ECS 引擎。但项目只在渲染层用 Bevy ECS，逻辑层不用。**

```
  两层分离：

  渲染层（ti-engine，Bevy ECS）：
    Entity = Bevy 的 Entity（u64 编号）
    Component = 挂到 entity 上的数据（Position、Sprite、Text...）
    System = Bevy 的系统（更新位置、渲染、拾取...）
    → 真 ECS，数据分散，缓存友好
    → 因为渲染层需要"每帧遍历所有实体"

  逻辑层（ti-core，域 + 规则）：
    域（domain/）= 纯数据 struct（地图/经济/战斗/选择...）
    规则（rules/）= 纯函数（trade/battle/board/territory...）
    → 不是 ECS，数据内聚，好调试
    → 因为逻辑层是"回合制"（不用每帧遍历）
```

**为什么不把逻辑层也做成 Bevy ECS？**

```
  理由 1：实体少
    玩家 1 + NPC 几个 + 建筑几十 = 不到 100 个
    ECS 的"缓存友好/并行"优势在 < 100 个实体时 = 0

  理由 2：回合制
    不用每帧更新所有实体（不是 60fps 实时）
    只有"摇骰/落格/结算"时才动逻辑
    ECS 的"每帧遍历所有系统"模式在这里浪费

  理由 3：业务复杂
    贸易/建造/战斗/领地 = 复杂业务规则
    域 + 规则的"数据内聚/函数集中"比 ECS 的"数据分散/系统遍历"好调试
    一条业务线 = 一个 rules 模块，不用跨多个 system 找

  理由 4：逻辑层不依赖 Bevy
    ti-core 是纯 Rust（无 Bevy 依赖）
    可以独立测试（cargo test 不需要 GPU/渲染）
    可以移植到其他引擎（换渲染引擎不用改逻辑）
```

**架构形状（crate 划分）**：

```
  ti-core（逻辑层，纯 Rust，无渲染依赖）：
    domain/（数据）：
      map.rs       — 地图数据（格子/建筑/资源）
      economy.rs   — 经济数据（金币/货物/船）
      combat.rs    — 战斗数据（HP/攻击/防御）
      selection.rs — 选择状态（选中的角色/地图）
      messages.rs  — 消息数据
      flow.rs      — 流程数据
      persistence.rs — 存档数据
      render.rs    — 渲染数据（逻辑层的渲染参数）

    rules/（逻辑）：
      board.rs     — 环岛赛道（摇骰/前进/落格）
      trade.rs     — 贸易/经济规则
      battle.rs    — 战斗规则
      build.rs     — 建造规则
      territory.rs — 领地系统（S28）
      boat.rs      — 船系统（S21.10）
      progression.rs — 进度规则
      selection.rs — 选角/选图规则
      saveload.rs  — 存读档规则
      rng.rs       — 随机数规则
      menu.rs      — 菜单规则
      main_loop.rs — 主循环规则
      ...

  ti-engine（渲染层，Bevy ECS）：
    scene_render.rs — BevyCanvas（Canvas trait 的 Bevy 实现）
    pick.rs        — 拾取系统（bevy_picking）
    ui.rs          — UI chrome（bevy_ui）
    → 真 Bevy ECS（entity/component/system）
    → 只负责"画"，不写业务
```

**两层怎么交互？**

```
  逻辑层（ti-core）
    规则函数执行（摇骰 → 落格 → 结算）
    修改 domain 数据（玩家金币 +100，地图格变红）
    输出"渲染参数"（domain/render.rs）

  渲染层（ti-engine）
    读 domain 数据 → 画到屏幕上
    读渲染参数 → 决定画什么颜色/位置
    不修改 domain 数据（只读）

  交互 = 单向：逻辑改数据 → 渲染读数据画
  渲染不写业务，逻辑不碰 GPU
```

---

## 五、行业对比

| 引擎/框架 | 架构 | 逻辑层 | 渲染层 |
|---|---|---|---|
| Unity | 组件化（GameObject + Component） | C# MonoBehaviour | 内置渲染器 |
| Unreal | 组件化（Actor + Component） | C++ 蓝图/C++ | 内置渲染器 |
| Godot | 节点树 + 组件化 | GDScript/C# | 内置渲染器 |
| **Bevy** | **ECS（真）** | **ECS system** | **ECS + wgpu** |
| **本项目** | **混合：Bevy ECS（渲染）+ 域/规则（逻辑）** | **纯 Rust domain + rules** | **Bevy ECS** |

**关键区别：Bevy 是"全 ECS"（逻辑也用 ECS），本项目"半 ECS"（只有渲染用 ECS，逻辑用域/规则）。**

---

## 六、项目对照

| 概念 | 位置 | 说明 |
|---|---|---|
| 域（数据） | `ti-core/src/domain/` | 10 个文件（map/economy/combat/...） |
| 规则（逻辑） | `ti-core/src/rules/` | 18 个文件（board/trade/battle/...） |
| Bevy ECS（渲染） | `ti-engine/src/` | scene_render/pick/ui |
| 逻辑→渲染交互 | `domain/render.rs` | 渲染参数（颜色/位置/可见性） |

### 与纯 ECS 的区别

| 纯 ECS | 本项目 |
|---|---|
| Entity = 编号 | 没有显式 Entity 编号（逻辑层） |
| Component = 纯数据 | Domain struct 含数据 + 简单方法 |
| System = 独立函数遍历 | Rules 函数操作 domain struct |
| SoA 存储 | AoS 存储（struct 字段） |
| 渲染 + 逻辑都是 ECS | 只有渲染是 ECS（Bevy），逻辑是域/规则 |

---

## 七、实践练习

**练习 1：识别域和规则**

打开 `crates/ti-core/src/domain/` 和 `crates/ti-core/src/rules/`，列出所有文件。
思考：哪些 domain struct 像"Component"（纯数据）？哪些 rules 函数像"System"（操作数据）？

**练习 2：设计 ECS 版本**

如果把 `rules/board.rs`（环岛赛道）改写成 Bevy ECS：
- 定义哪些 Component？（`DiceState`、`BoardPosition`、`LapCount`...）
- 定义哪些 System？（`spin_system`、`advance_system`、`settle_system`...）
- 思考：改造后对本项目有什么好处？（缓存友好？并行？）
- 思考：改造后有什么坏处？（数据分散、调试困难、逻辑层依赖 Bevy？）

**练习 3：选择合适架构**

为以下项目选择 ECS 或域 + 规则，说明理由：
- 弹幕游戏（5000+ 子弹）→ ?
- RPG 游戏（10 个角色，每个技能不同）→ ?
- 模拟城市（50000+ 建筑）→ ?
- 本项目（金银岛，< 100 实体，回合制）→ ?

## 常见坑

| 坑 | 原因 | 避免 |
|---|---|---|
| 小项目用 ECS | 过度设计（< 100 实体，ECS 优势 = 0） | 根据实体规模选择 |
| 逻辑层依赖 Bevy | 换引擎时逻辑全要改 | 逻辑层（ti-core）纯 Rust，不碰 Bevy |
| 渲染层写业务 | 渲染变慢/耦合 | 渲染层（ti-engine）只读 domain，不写业务 |
| 组件/域之间直接引用 | 紧耦合 | 通过编号或事件通信 |
| 系统/规则之间隐式依赖 | 执行顺序不确定 | 显式声明调用顺序 |

## 学习记录

- 状态：未开始
- 完成日期：
- 心得/疑问：
