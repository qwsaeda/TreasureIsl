# 第 1 章：游戏循环

## 学习目标

- [ ] 理解游戏循环的基本结构（update + render）
- [ ] 理解为什么游戏循环与渲染帧率解耦
- [ ] 能识别本项目中的游戏循环位置
- [ ] 理解"固定时间步长"与"可变时间步长"的区别

## 核心概念

### 什么是游戏循环

游戏循环是任何游戏的核心骨架。它是一个无限循环，每轮做两件事：

```
while 游戏未结束:
    update(dt)   // 更新业务数据（输入、物理、AI、规则）
    render()     // 根据最新游戏状态绘制画面
```

- **update**：推进游戏世界。处理输入、更新位置、检测碰撞、推进动画。
- **render**：把游戏状态翻译成像素。不做任何逻辑决策。

### 关键概念：帧、FPS、dt

**帧（Frame）= 一张完整画面。** 60 FPS = 每秒画 60 张，连续播放看到运动。

**dt（Delta Time）= 两帧之间的时间间隔。** 它不是游戏状态，而是 update 的输入参数：

```
游戏状态（全局数据）：
  玩家位置 (50, 30)、金币 1000、当前场景 MainMap

dt = 0.0167 秒（60 FPS 下每帧间隔）
  → 只是"过了多久"，不是状态本身

update(dt) 做的事：
  新位置 = 旧位置 + 速度 × dt
  → (50, 30) + (10, 5) × 0.0167 = (50.17, 30.08)
```

### 业务数据 vs 游戏状态

```
游戏状态（广义）= 业务数据 + 表现状态

业务数据（ti-core 管）：
  玩家位置、金币、地图、战斗回合、卡牌结果...
  → update 修改的就是这些

表现状态（ti-app 管）：
  摄像机偏移、当前帧缓冲、动画帧号、脏行标记、选中高亮...
  → render 读取全部状态来画画面
```

**update 主要改业务数据，render 读取全部状态（业务 + 表现）来画画面。**

### update 每帧都算所有数据吗？

**不是。** update 每帧遍历数据，但不是全部重新计算：

```
玩家没动 → 只读输入状态（1 次比较）→ 跳过物理/碰撞/事件
玩家动了 → 完整管线：移动 + 碰撞 + 事件 + 动画
```

**"跳过"也有成本，但计算量不同**：
- 跳过检查：O(1) — 读一个布尔值比较
- 完整计算：O(n) — 物理/碰撞/事件/动画

**多线程**：游戏主循环通常**单线程**，保证确定性和可复现。多线程引入竞态条件，bug 难复现。本项目是回合制 2D，单线程足够。

### 为什么 update 和 render 要分离

| 耦合的问题 | 分离的好处 |
|---|---|
| 帧率快 = 游戏速度快 | 游戏速度只由 dt 决定 |
| 不同设备速度不一致 | 逻辑确定性，可复现 |
| 渲染卡顿导致逻辑跳帧 | 渲染慢不影响逻辑完整性 |

**本项目的额外策略 — 按需渲染省电**：
- 回合制游戏大部分时间画面静止，没变化就不重绘
- 只有玩家操作/场景切换时才真正 render
- 移动端高 FPS = 烧电，按需渲染是本项目的省电优化

### 为什么必须乘 dt（60 vs 30 FPS 问题）

```
不乘 dt（错误做法）：
  每帧固定移动 10 像素
  60 FPS：每秒 60 帧 × 10 像素 = 600 像素/秒
  30 FPS：每秒 30 帧 × 10 像素 = 300 像素/秒
  → 30 FPS 下玩家慢一倍！

乘 dt（正确做法）：
  60 FPS：dt=0.0167，位移=0.167，每秒 0.167×60 = 10 像素/秒
  30 FPS：dt=0.0333，位移=0.333，每秒 0.333×30 = 10 像素/秒
  → 速度一致！
```

**核心**：dt 随 FPS 变化，速度 × dt 的乘积保证每秒位移恒定。FPS 高 → dt 小 → 每帧位移小 → 帧数多 → 总位移相同，但画面更流畅。

**代价**：高 FPS = 更费电、更费 CPU。60 FPS 下 CPU/GPU 每秒多算 30 次 update + 多画 30 张。移动端尤其敏感。

### 两种时间步长模式

```
可变时间步长（简单但不稳定）:
  dt = 当前帧实际耗时
  update(dt)  // 每次 dt 不同

固定时间步长（复杂但稳定）:
  固定 dt = 1/60s
  累积器 += 实际耗时
  while 累积器 >= 固定dt:
      update(固定dt)
      累积器 -= 固定dt
  render()  // 用最新状态渲染
```

**可变时间步长**：简单，但浮点累积误差会漂移。
**固定时间步长**：复杂，但游戏速度确定，可复现（见第 4 章）。

### 游戏循环的职责边界

```
游戏循环只做三件事：
  1. 计算本帧耗时（dt）
  2. 调用 update(dt)
  3. 调用 render()

游戏循环不做：
  - 具体的业务逻辑（移动/战斗/经济）
  - 具体的绘制细节（精灵/文字/布局）
  - 资源加载/卸载
```

## 项目对照

### 本项目游戏循环位置

> **S18.8a：交付壳 = Bevy 壳**（`crates/ti-engine/src/bin/treasure_bevy.rs`，独立 workspace）。
> ti-app 的 macroquad 桌面/浏览器壳（`treasure_native.rs` / `treasure_wasm.rs`）已退役；
> 下面的循环对照改指 Bevy 壳的 **ECS 系统链**（概念一一对应，只是宿主从手写 `loop` 换成 Bevy schedule）。

```
crates/ti-engine/src/bin/treasure_bevy.rs（Update 调度链）
```

```rust
// Bevy 壳：Update 阶段的系统链（.chain() = 严格按序执行）
(input_system, camera_system, render_system, stats_system).chain()

// 1. 帧耗时：Bevy 内建时钟（Time::delta）——对应手写壳的 tick_frame_time()
let delta_ms = u32::try_from(time.delta().as_millis()).unwrap_or(16).clamp(1, 250);
g.set_frame_delta_ms(delta_ms);

// 2. 收集输入：Bevy ButtonInput<KeyCode> / MouseButton / Touches → GameApp::poll
input_system(...);

// 3. 渲染：GameApp 逻辑渲染进 DrawList（绘制目标可插拒）
g.render();

// 4. 呈现：DrawList → Bevy 实体（Sprite/Text2d），由 Bevy 渲染器上屏
native_index_system(world);   // .after(render_system)
```

### 职责对应

| 游戏循环职责 | 本项目实现 | 文件位置 |
|---|---|---|
| 计算 dt | Bevy `Time::delta()` + `set_frame_delta_ms()` | `treasure_bevy.rs::render_system` |
| 收集输入 | `input_system` → `GameApp::poll` → `route_input()` | `treasure_bevy.rs` + `scenes/mod.rs` |
| 更新状态 | 各场景 `on_input()` / `on_touch()` | `scenes/*.rs` |
| 渲染 | `g.render()` → 各场景 `render()` → DrawList | `game_app.rs` + `scenes/mod.rs` |
| 呈现 | DrawList → Bevy 实体（`native_index_system`）→ Bevy 渲染器 | `ti-engine/src/scene_render.rs` |

### 设计要点

1. **帧耗时注入**：`set_frame_delta_ms` 让业务层知道本帧耗时，驱动动画推进（`game_app.rs`）
2. **绘制目标可插拒**：`set_record_draw(true)` → 绘制命令进 DrawList（交 Bevy 实体化）；
   `false` → CPU 帧缓冲（headless 测试 / 诊断 bin 路径）
3. **静态帧跳帧**（S17.9）：过渡窗口内逐帧渲染，静止帧仍按变化键跳过重绘

## 实践练习

### 练习 1：识别游戏循环

打开 `crates/ti-engine/src/bin/treasure_bevy.rs`，找到 Update 调度链并标注：
- 帧耗时从哪来（Bevy `Time`）
- 哪里收集输入（`ButtonInput` / `Touches`）
- 哪里调用渲染（`g.render()`）
- 绘制命令如何变成屏幕像素（DrawList → 实体 → Bevy 渲染器）

### 练习 2：理解按需渲染

在 `game_app.rs` 的 `last_rendered` / `last_rendered_scene` 双键判断处思考：
- 如果去掉这个条件，每帧都渲染，会发生什么？
- 为什么 `last_rendered_scene != scene_key` 也要检查？

### 练习 3：最小游戏循环

用伪代码写一个最小游戏循环（20 行以内），包含：
- update(dt) 打印当前时间
- render() 打印"绘制"
- 运行 3 秒后退出

## 常见坑

| 坑 | 原因 | 避免 |
|---|---|---|
| 游戏速度随帧率变化 | update 用了可变 dt 但没做时间步长分离 | 用固定时间步长（第 4 章） |
| 首帧空白 | 循环从 update 开始，没先 render | 首帧强制 render（本项目做法） |
| 渲染卡顿导致逻辑丢失 | 渲染和 update 在同一帧 | 固定时间步长 + 累积器 |
| 游戏循环里写业务逻辑 | 违反单一职责 | 业务逻辑放场景/系统 |

## 学习记录

- 状态：已完成
- 完成日期：2026-10-13
- 心得/疑问：dt 是时间间隔非状态；乘 dt 保证速度一致；"跳过"也是计算但 O(1) vs O(n)；单线程保证确定性
