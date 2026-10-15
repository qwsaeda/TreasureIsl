# 第 2 章：状态与状态机

## 学习目标

- [ ] 理解有限状态机（FSM）的概念
- [ ] 能区分"状态"和"状态转换"
- [ ] 理解堆栈式状态机（_scene stack_）的用途
- [ ] 能识别本项目中的状态管理方式

## 核心概念

### 本质：业务场景枚举 + 交互约束

本章重点不是状态机理论，而是：

```
1. 按业务定义场景枚举
   开结 / 选项 / 游戏中 / 对战中 / 死亡 / 重新开始

2. 每个场景约束用户能做什么
   主菜单：只能选选项
   对战中：只能攻击/防御/逃跑
   死亡画面：只能重新开始或回主菜单

3. 每个场景处理自己的业务
   选项场景：改设置，不涉及战斗
   战斗场景：伤害计算，不涉及地图移动
```

**状态机 = 业务场景的枚举 + 每个场景的交互约束 + 场景间的转换规则。**

### 形式化定义

有限状态机 = **状态** + **转换规则** + **当前状态**。

```
状态（State）：游戏在某一时刻的模式
  - 主菜单、角色选择、地图选择、主地图、战斗、消息、存档

转换（Transition）：从一个状态到另一个状态的条件
  - 主菜单 → 角色选择（按确认键）
  - 角色选择 → 地图选择（选完角色）
  - 地图选择 → 主地图（选完地图）

当前状态（Current State）：游戏此刻处于哪个状态
```

### 为什么用枚举 + 分发，而不是 if-else

**不是语法区别，是代码组织区别**：

```
if-else 地狱：所有场景的逻辑堆在一个函数里（1000+ 行）
  改一个场景可能影响其他场景

枚举 + 分发：每个场景的逻辑在自己文件里
  新建 scenes/xxx.rs + 匹配加 1 行，不影响已有场景
```

任何语言都能实现（接口/函数对象/lambda/Map），本质都是：**逻辑分散 + 一个分发器**。Rust 的 enum + match 只是多了编译期穷举检查。

### 堆栈式状态机

普通状态机：状态之间直接切换（A → B → C）。
堆栈式状态机：状态可以"压栈"和"弹栈"（A → B → 回 A）。

```
场景堆栈示例：
  [MainMap]           // 玩家探索地图
  [MainMap, Message]  // 遇到 NPC，打开消息
  [MainMap]           // 关闭消息，回到地图

堆栈的好处：
  - 子状态（消息/存档）关闭后自动回到父状态
  - 不需要"返回到哪里"的硬编码
```

### 状态转换的两种模式

| 模式 | 做法 | 适合 |
|---|---|---|
| **直接切换** | `current_state = Scene::Battle` | 线性流程 |
| **压栈/弹栈** | `stack.push(Scene::Message)` / `stack.pop()` | 可返回的子界面 |

## 项目对照

### 本项目状态管理

```
crates/ti-core/src/rules/main_loop.rs  — SceneStack 定义
crates/ti-app/src/game_app.rs           — Scene 枚举
crates/ti-app/src/scenes/mod.rs        — Scene trait + 路由
```

### Scene 枚举（game_app.rs:52）

```rust
pub enum Scene {
    Intro,      // 片头
    Index,      // 主菜单
    SelectRole, // 角色选择
    SelectMap,  // 地图选择
    MainMap,    // 主地图
    Vs,         // 战斗
    Message,    // 消息/帮助
    Saveload,   // 存读档
    Exit,       // 退出
}
```

### Scene trait（scenes/mod.rs）

```rust
pub trait Scene {
    /// 处理输入，返回是否消费
    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool;
    /// 处理触摸
    fn on_touch(&mut self, host: &mut dyn SceneHost, touch: TouchSample);
    /// 渲染到帧缓冲
    fn render(&mut self, host: &mut dyn SceneHost);
}
```

### 路由机制（scenes/mod.rs）

```rust
pub fn route_input(host: &mut dyn SceneHost, kind: SceneKind, input: GameInput) -> bool {
    match kind {
        SceneKind::Index => index::on_input(host, input),
        SceneKind::SelectRole => select_role::on_input(host, input),
        SceneKind::MainMap => main_map::on_input(host, input),
        // ...
    }
}
```

### 本项目的状态管理特点

1. **场景枚举驱动**：`Scene` 枚举定义所有合法状态
2. **路由分发**：`route_input`/`route_render`/`route_touch` 按场景分发
3. **组合根持有状态**：`GameApp` 持有全部跨场景状态
4. **场景不持有彼此**：每个场景 struct 只持有 `&mut dyn SceneHost`

## 实践练习

### 练习 1：绘制状态转换图

为本项目绘制完整的场景转换图（Scene Transition Diagram）：
- 节点 = Scene 枚举值
- 边 = 转换条件（按键/触摸）
- 标注哪些是"压栈"（消息/存档），哪些是"切换"

### 练习 2：理解路由

在 `scenes/mod.rs` 中找到 `route_input`，思考：
- 如果新增一个场景，需要改哪些地方？
- 为什么不直接 `match` 在 `GameApp` 里，而要用独立的路由函数？

### 练习 3：设计新场景

设计一个"设置"场景（Settings），需要：
- 在 `Scene` 枚举里加变体
- 实现 `Scene` trait
- 在路由函数里加分发分支
- 思考：从哪个场景进入设置？返回时回到哪里？

## 常见坑

| 坑 | 原因 | 避免 |
|---|---|---|
| 状态爆炸 | 每个界面都做成独立状态 | 合并同类（如"消息"通用化） |
| 忘记处理所有输入 | 新状态没处理所有按键 | 状态机 + 输入映射表（第 6 章） |
| 返回状态丢失 | 直接切换而非压栈 | 用堆栈式状态机 |
| 场景之间直接引用 | 违反依赖方向 | 通过 `SceneHost` trait 通信 |

## 学习记录

- 状态：已完成
- 完成日期：2026-10-13
- 心得/疑问：状态机本质是业务场景枚举+交互约束；分发器用什么语法不重要，逻辑分散才是核心
