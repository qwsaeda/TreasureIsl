# 系统架构

本文档定义完整的系统架构：模块依赖、数据流、场景栈管理、资源加载管线、游戏状态管理、帧循环。

---

## 1. 模块依赖

```
ti-formats (资源解析: pak/lpk/pcx/map/font/save)
    ↑
ti-core (核心类型/常量/规则/错误)
    ↑
ti-platform (平台抽象: 输入/存储/音频/定时)
    ↑
ti-app (游戏逻辑: 场景/状态机/经济/战斗/卡牌)
    ↑
ti-tools (工具: ledger/doccheck/controls/serve)
```

### 1.1 依赖规则

- 下层不得依赖上层（单向依赖）
- `ti-formats` 只依赖 `std` + 自有错误类型
- `ti-core` 依赖 `ti-formats`
- `ti-platform` 依赖 `ti-core`
- `ti-app` 依赖前三者
- 跨模块通信通过 `ti-core` 定义的类型

---

## 2. 核心类型定义

### 2.1 场景枚举

```rust
// crates/ti-app/src/game_app.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scene {
    Intro,        // 片头动画 → 自动跳 Index
    Index,        // 主菜单（6 项）
    SelectRole,   // 角色选择（5 角色循环）
    SelectMap,    // 地图选择（已解锁地图循环）
    MainMap,      // 主地图（移动/贸易/建造/事件分派）
    Vs,           // 战斗
    Message,      // 消息/帮助/信息（叠加窗口）
    Saveload,     // 存读档（子窗口叠加）
    Exit,         // 退出
}

impl Scene {
    pub fn resource_file(&self) -> &'static str {
        match self {
            Scene::Intro => "intro.Map",
            Scene::Index => "index.Map",
            Scene::SelectRole => "selectrole.Map",
            Scene::SelectMap => "selectmap.Map",
            Scene::MainMap => "main.Map",
            Scene::Vs => "vs.Map",
            Scene::Message => "msgl.Map",      // 动态选择 msgl/msgm/msgs/help/information
            Scene::Saveload => "saveload.Map",
            Scene::Exit => "",
        }
    }

    pub fn is_overlay(&self) -> bool {
        matches!(self, Scene::Message | Scene::Saveload)
    }
}
```

### 2.2 场景状态码

```rust
// crates/ti-app/src/game_app.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneState {
    Menu = 0,         // 主菜单态
    Unloading = 2,    // 场景卸载中
    BackToMenu = 3,   // 回主菜单
    Transition = 4,   // 过场/坐标切换
    Battle = 25,      // 战斗态
    Anim26 = 26,      // 过场动画 phase 26
    Anim27 = 27,      // 过场动画 phase 27
}
```

### 2.3 方向枚举

```rust
// crates/ti-core/src/oracle/main_map.rs（Dir 枚举；GridPos 同文件）

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    North = 0,  // (-1, 0), 方向位 8, 探测优先级 4
    West = 1,   // (0, -1), 方向位 4, 探测优先级 3
    South = 2,  // (+1, 0), 方向位 2, 探测优先级 1
    East = 3,   // (0, +1), 方向位 1, 探测优先级 2
}

impl Dir {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::North => (-1, 0),
            Dir::West => (0, -1),
            Dir::South => (1, 0),
            Dir::East => (0, 1),
        }
    }

    pub fn bit(self) -> u8 {
        match self {
            Dir::North => 8,
            Dir::West => 4,
            Dir::South => 2,
            Dir::East => 1,
        }
    }

    pub fn priority_order() -> [Dir; 4] {
        [Dir::South, Dir::East, Dir::North, Dir::West]
    }
}
```

### 2.4 网格坐标

```rust
// crates/ti-core/src/oracle/main_map.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPos {
    pub row: u32,
    pub col: u32,
}

impl GridPos {
    pub fn index(self, grid_w: u32) -> u32 {
        self.row * grid_w + self.col
    }

    pub fn from_index(idx: u32, grid_w: u32) -> Self {
        GridPos {
            row: idx / grid_w,
            col: idx % grid_w,
        }
    }
}
```

### 2.5 游戏状态

> **S8.4 起**：`GameState` 已不再作为独立 struct，各状态字段直接内嵌在 `GameApp` 中。
> 以下为 `crates/ti-app/src/game_app.rs` `GameApp` 字段列表（S8.4 后实际结构，字段延伸至 game_app.rs:180-248）：

```rust
// crates/ti-app/src/game_app.rs（GameState 各字段直接内嵌在 GameApp 中，非独立 struct）

// GameApp 游戏状态字段（§3 顶层结构示例，完整 40+ 字段见 game_app.rs:180-248）：
//   character: Option<Character>
//   map_slot: Option<MapSlot>
//   unlocked_maps: Vec<MapSlot>
//   selection: SelectionState
//   map_terrain: Option<MapTerrain>
//   map_pos: Option<GridPos>
//   map_landmarks: Vec<(u32, u32)>
//   gold: u32
//   price_table: PriceTable
//   owned: Vec<(u32, u32)>
//   bank: Bank
//   buildings: Vec<(Building, u32)>
//   battle: Option<BattleView>
//   map_session: MapSession
//   rng: GameRng / rng_seed: u32
//   msg_kind / msg_page / msg_total / save_slots / quit / last_dispatch
```


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Character {
    pub id: u32,        // 1..=5
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapSlot {
    pub id: u32,        // 1..=3
    pub grid_w: u32,
    pub grid_h: u32,
}

#[derive(Debug, Clone)]
pub struct Bank {
    pub gold: u32,       // 流动金
    pub deposit: u32,    // 银行存款金
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Building {
    Castle,       // 城堡, base_cost 1000
    Investment,   // 投资设施, base_cost 500
    Bank,         // 银行, base_cost 200
    Fort,         // 据点/堡垒, base_cost 800
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgKind {
    Msgs, Msgl, Msgm, Help, Information,
}
```

---

## 3. 顶层应用结构

```rust
// crates/ti-app/src/game_app.rs（S8.4 后实际字段，完整 40+ 字段见 game_app.rs:180-240）

pub struct GameApp {
    pub shell: MenuShell,
    pub scenes: SceneStack,
    pub scene: Scene,
    pub menu_selected: usize,
    // 游戏状态（原 GameState 字段已直接内嵌，§2.5 列表）
    pub character: Option<Character>,
    pub map_slot: Option<MapSlot>,
    pub unlocked_maps: Vec<MapSlot>,
    pub selection: SelectionState,
    pub map_terrain: Option<MapTerrain>,
    pub map_pos: Option<GridPos>,
    pub map_landmarks: Vec<(u32, u32)>,
    pub gold: u32,
    pub price_table: PriceTable,
    pub owned: Vec<(u32, u32)>,
    pub bank: Bank,
    pub buildings: Vec<(Building, u32)>,
    pub battle: Option<BattleView>,
    pub map_session: MapSession,
    pub rng: GameRng,
    pub rng_seed: u32,
    pub msg_kind: Option<MsgKind>,
    pub msg_page: u32,
    pub msg_total: u32,
    pub save_slots: Vec<SaveSlot>,
    pub last_sfx: Option<Sfx>,
    pub battle_won: bool,
    pub quit: bool,
    pub last_dispatch: Option<SceneTarget>,
    pub atlas: Option<SpriteAtlas>,
}

impl GameApp {
    pub fn new(screen_w: u32, screen_h: u32) -> Self {
        // 初始化 shell + 加载 ui.pak
        // 初始场景 = Intro
    }

    // 注：S8.4 前 `run()` 为主循环伪代码；S8.4 后各 bin 直接调 `poll(&KeyState)` + `render()`，
    // 不再用 `.state`/`.input` 字段（已内嵌）。
    pub fn poll(&mut self, keys: &KeyState) -> bool { /* 输入处理 */ }
}
```

---

## 4. 场景栈管理

```rust
// crates/ti-core/src/rules/main_loop.rs

/// 注：`rules::main_loop` 的 `SceneStack` 实际定义为
/// `struct SceneStack { state: u32, scene: Option<SceneId>, history: Vec<(u32, Option<SceneId>)> }`
/// （无 `pending` 字段，事件由 `advance(trigger: u32)` 即时处理，见
/// `crates/ti-core/src/rules/main_loop.rs:12`）；
/// 以下伪代码为壳层（ti-app）视角的场景栈抽象，`pending: Vec<GameInput>` 属壳层排队
/// 输入（与 ti-core 无关）。
pub struct SceneStack {
    stack: Vec<Scene>,
    state: SceneState,
    pending: Vec<GameInput>,
}

impl SceneStack {
    pub fn current(&self) -> Option<Scene> {
        self.stack.last().copied()
    }

    pub fn push(&mut self, scene: Scene) {
        self.stack.push(scene);
    }

    pub fn pop(&mut self) -> Option<Scene> {
        self.stack.pop()
    }

    pub fn replace_top(&mut self, scene: Scene) {
        if let Some(top) = self.stack.last_mut() {
            *top = scene;
        }
    }

    pub fn update(&mut self, state: &mut GameState) {
        // 1. 处理 pending 事件
        // 2. 更新当前场景
        // 3. 处理场景切换请求
    }
}
```

---

## 5. 资源加载管线

```
启动
  ↓
加载 ui.pak → SpriteAtlas
  ↓
加载 settings.dat → Settings
  ↓
探测 data/glmapm01/02/03.Map → 已解锁地图列表
  ↓
进入 Intro 场景
  ↓
Intro 完成 → 加载 index.Map
  ↓
进入 Index 场景
```

### 5.1 资源依赖表

| 场景 | 必须资源 |
|---|---|
| Intro | ui.pak + intro.Map |
| Index | ui.pak + index.Map + glpics.pak + data/gl.rec |
| SelectRole | ui.pak + selectrole.Map + glpics.pak |
| SelectMap | ui.pak + selectmap.Map |
| MainMap | glpics.pak + glmapm0x.Map + main.Map + ui.pak |
| Vs | ui.pak + vs.Map |
| Message | ui.pak + msgl/msgm/msgs/help/information.Map |
| Saveload | ui.pak + saveload.Map |

---

## 6. 帧循环

```
fn main():
    game = GameApp::new(176, 220)
    if let Some(ui_pak) = read_file("ui.pak"):
        game.atlas = Some(SpriteAtlas::load(&ui_pak))
    game.render()   // 帧循环：各 bin 按帧调 poll(&KeyState) + render()（S8.4 后入口形态）

    loop:
        if game.quit:   // S8.4 后字段直接内嵌 GameApp，无独立 .state
            break

        game.shell.refit(176, 220)
        keys = poll_keyboard()
        changed = game.poll(&keys)   // S8.4 后：各 bin 调 GameApp::poll(&KeyState)

        if changed:
            // 场景切换由 poll 内部处理（无独立 .state / .input 字段）

        game.render()
        game.shell.present()
        sleep(16ms)
        next_frame().await
```

---

## 7. 输入系统

### 7.1 按键状态

```rust
// crates/ti-app/src/game_app.rs（S8.4 后实际结构：11 个 bool 字段位标志）

pub struct KeyState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub confirm: bool,
    pub back: bool,
    pub landmark: bool,
    pub buy: bool,
    pub sell: bool,
    pub build: bool,
    // 共 10 个 bool（+Default = 11 含 Default）
}
// 注：S8.4 前 KeyState 为 Vec<Key> 结构；S8.4 后改为 bool 位标志，各 bin 直接构造 KeyState 喂给 GameApp::poll。
```

### 7.2 全局按键映射

| 按键 | 语义 |
|---|---|
| Up / W | 上一个 / 上移 |
| Down / S | 下一个 / 下移 |
| Left / A | 左移 |
| Right / D | 右移 |
| Enter / Space | 确定 |
| Esc / Backspace | 返回 |
| F1 | 循环缩放模式 |
| F2 | 循环滤波模式 |
| F3 | 循环边框样式 |

---

## 8. 渲染管线

### 8.1 渲染流程

```
1. 清屏（帧缓冲清零）
2. 渲染场景背景
3. 渲染地图/精灵
4. 渲染 UI 控件
5. 渲染文本
6. present（交换缓冲）
```

### 8.2 缩放模式

| 模式 | 缩放因子 | 说明 |
|---|---|---|
| Mode0 | 2x | 352×440 |
| Mode1 | 3x | 528×660 |
| Mode2 | 4x | 704×880 |
| Mode3 | 自适应 | 窗口大小 |

---

## 9. 平台抽象

### 9.1 平台 trait

```rust
// crates/ti-platform/src/backend.rs（实际：Storage + Clock 分 trait，无统一 Platform trait）
// 注：以下为概念抽象；实际 ti-platform 拆为 Storage + Clock 两个 trait（backend.rs:104-121）。

pub trait Storage {
    fn read(&mut self, key: &str) -> Result<Vec<u8>, Error>;
    fn write(&mut self, key: &str, value: &[u8]) -> Result<(), Error>;
}

pub trait Clock {
    fn now_ms(&mut self) -> u64;
}
```

### 9.2 平台实现

| 平台 | 文件 | 输入 | 存储 |
|---|---|---|---|
| Native | desktop.rs | 键盘 | 文件系统 |
| WASM | wasm.rs | 键鼠+触摸 | IndexedDB/LocalStorage |
| Android | android.rs | 触摸 | 文件系统 |

---

## 10. 错误处理

```rust
// crates/ti-app/src/error.rs

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Formats(#[from] ti_formats::Error),
    #[error(transparent)]
    Platform(#[from] ti_platform::Error),
}

#[derive(Debug)]
pub enum SceneError {
    InvalidTransition,
    ResourceNotFound(&'static str),
    StackUnderflow,
}

#[derive(Debug)]
pub enum TradeError {
    UnknownGood,
    InsufficientGold,
    NoStock,
    QtyOutOfRange,   // S8.3.b 后实际 4 变体（含 qty 越界）
}

#[derive(Debug)]
pub enum BuildError {
    UnknownBuilding,
    Insufficient,
    InvalidLevel,
}
// 注：oracle 占位名 `TradeError`/`BuildError`；当前 `rules::trade::TradeErr`（4 变体，含 QtyOutOfRange）/
// `rules::build::BuildErr`（3 变体，含 UnknownBuilding）为实际实现。
```

---

## 11. 全局单例

| 单例 | 变量 | 说明 |
|---|---|---|
| 游戏状态 | `off_811A0` | 8 个 u32 槽 + 宽字符串 |
| 地图编号 | `dword_7FB6C` | 1/2/3 |
| 角色索引 | `dword_7FB70` | 1..=5 |
| 状态位 | `byte_81194..81197` | 4 字节全局标志 |

---

## 12. 实现检查清单

- [ ] 所有核心类型定义完整
- [ ] 模块依赖单向无环
- [ ] 帧循环 60fps
- [ ] 输入响应 < 16ms
- [ ] 场景切换无死锁
- [ ] 资源加载失败优雅降级
