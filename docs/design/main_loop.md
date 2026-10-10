# 主循环与游戏状态机

本文档是**自包含的实现规格**。结合 `docs/formats/lucencypackage.md`（资源格式）
和本文档，可在不了解原二进制的情况下实现完整游戏。

---

## 1. 架构总览

```
┌─────────────────────────────────────────────────────────┐
│                    GameApp                               │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ shell        │  │ scenes       │  │ scene_cache   │  │
│  │ (Framebuf+   │  │ (SceneStack) │  │ (HashMap)     │  │
│  │  Presenter)  │  │              │  │               │  │
│  └──────┬──────┘  └──────┬───────┘  └───────┬───────┘  │
│         │                │                  │           │
│  ┌──────▼──────┐  ┌──────▼───────┐  ┌──────▼───────┐  │
│  │ atlas        │  │ input        │  │ settings     │  │
│  │ (SpriteAtlas)│  │ (KeyState)   │  │ (Settings)   │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
         │                 │                  │
    ┌────▼────┐      ┌─────▼─────┐      ┌────▼────┐
    │ sprite  │      │ keyboard  │      │ storage │
    │ .pak    │      │ + touch   │      │ backend │
    └─────────┘      └───────────┘      └─────────┘
```

---

## 2. 场景枚举

```rust
enum Scene {
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
```

---

## 3. 场景状态机

```
                    ┌──────────┐
                    │  Intro   │
                    └────┬─────┘
                         │ 任意键 / 自动
                         ▼
                    ┌──────────┐
         ┌─────────│  Index   │─────────┐
         │         └────┬─────┘         │
         │              │               │
    菜单 0          菜单 1..4         菜单 5
    (新游戏)         (继续/选项/说明)    (退出)
         │              │               │
         ▼              ▼               ▼
  ┌────────────┐  ┌──────────┐    ┌────────┐
  │SelectRole  │  │Saveload  │    │ Exit   │
  └──────┬─────┘  └────┬─────┘    └────────┘
         │              │
    角色选定         读档
         │              │
         ▼              ▼
  ┌────────────┐  ┌──────────┐
  │SelectMap   │  │MainMap   │
  └──────┬─────┘  └────┬─────┘
         │              │
    地图选定         移动
         │              │
         ▼              ▼
  ┌────────────┐  ┌──────────┐
  │ MainMap    │  │ 事件分派  │
  └────────────┘  └────┬─────┘
                        │
              ┌─────────┼─────────┐
              │         │         │
          事件码 1   事件码 6   其他
          (据点)    (战斗)
              │         │         │
              ▼         ▼         ▼
         ┌────────┐ ┌──────┐ ┌────────┐
         │Message │ │  Vs  │ │Message │
         │(Msgl)  │ │      │ │(Msgm)  │
         └────────┘ └──┬───┘ └────────┘
                        │
                   战斗结束
                        │
                        ▼
                   ┌────────┐
                   │ Index  │
                   └────────┘
```

---

## 4. 主循环伪代码

```
fn main():
    let mut game = GameApp::new(screen_w, screen_h)

    // 启动时注入资源
    if let Some(ui_pak) = read_file("ui.pak"):
        game.set_atlas(&ui_pak)

    // 首帧渲染
    game.render()

    loop:
        // 1. 视口跟随窗口尺寸
        game.shell.refit(screen_w, screen_h)

        // 2. 轮询输入
        let keys = poll_keyboard()
        changed |= game.poll_input(&keys)

        // 3. 渲染（有变化时）
        if changed:
            game.render()

        // 4. 呈现
        game.shell.present()

        // 5. 帧率控制
        sleep(16ms)  // ~60fps
        next_frame().await
```

---

## 5. 输入映射

### 5.1 全局按键

| 按键 | 语义 | 场景 |
|---|---|---|
| ↑ / W | 上一个 / 上移 | 所有 |
| ↓ / S | 下一个 / 下移 | 所有 |
| ← / A | 左移 | 主地图 |
| → / D | 右移 | 主地图 |
| Enter / 空格 | 确定 | 所有 |
| Esc / Backspace | 返回 | 子场景 |
| F1 | 循环缩放模式 | 所有 |
| F2 | 循环滤波模式 | 所有 |
| F3 | 循环边框样式 | 所有 |

### 5.2 场景特定

| 场景 | 按键 | 行为 |
|---|---|---|
| Index | ↑/↓ | 循环选择菜单项 |
| Index | Enter | 执行选中项 |
| SelectRole | 38/40 | 上一个/下一个角色 |
| SelectRole | 13 | 确认角色 |
| SelectMap | 38/40 | 循环已解锁地图 |
| SelectMap | 13 | 确认地图 |
| MainMap | 方向键 | 朝该方向走一步 |
| MainMap | Space | 触发据点事件 |
| Vs | Enter / Space | 攻击一回合 |
| Vs | Esc | 退出战斗 |
| Message | ↑/↓ | 翻页 |
| Message | Enter/Esc | 关闭 |
| Saveload | ↑/↓ | 选槽位 |
| Saveload | Enter | 读/写档 |
| Saveload | Esc | 关闭 |

---

## 6. 跨场景状态

```rust
struct GameApp {
    // 表现层
    shell: MenuShell,           // 帧缓冲/字形/呈现
    atlas: Option<SpriteAtlas>, // ui.pak 精灵图

    // 场景管理
    scenes: SceneStack,         // 状态码跟踪
    scene: Scene,               // 当前顶层场景

    // 游戏状态
    character: Option<Character>,  // 已选角色
    map_slot: Option<MapSlot>,      // 已选地图
    unlocked_maps: Vec<MapSlot>,    // 已解锁地图
    gold: u32,                      // 当前金
    owned: Vec<(u32, u32)>,         // 持有量
    bank: Bank,                     // 银行
    buildings: Vec<(Building, u32)>, // 建筑列表
    battle: Option<BattleView>,     // 战斗会话
    msg_kind: Option<MsgKind>,     // 消息种类
    msg_page: u32,                  // 当前消息页
    msg_total: u32,                 // 总页数
    save_slots: Vec<SaveSlot>,      // 存档槽位

    // 系统
    rng: GameRng,       // 全局 RNG
    rng_seed: u32,      // RNG 种子
    last_sfx: Option<Sfx>, // 音效
    quit: bool,         // 退出标志
}
```

---

## 7. 实现路由

| 模块 | 本文档章节 | 详细规格 |
|---|---|---|
| 菜单系统 | §3 场景状态机 | [scene_flow.md](scene_flow.md) |
| 地图系统 | §3 MainMap 分支 | [map_system.md](map_system.md) |
| 经济系统 | §6 gold/owned/bank | [economy.md](economy.md) |
| 战斗系统 | §6 battle | [combat.md](combat.md) |
| 输入系统 | §5 输入映射 | [ui_controls.md](ui_controls.md) |
| 持久化 | §6 save_slots | [persistence.md](persistence.md) |

---

## 8. 验收标准

1. **场景全覆盖**：9 个场景（Intro/Index/SelectRole/SelectMap/MainMap/Vs/Message/Saveload/Exit）均可进入/返回，无死锁
2. **输入正确**：每个按键在每个场景的行为符合 §5 映射表
3. **状态传递**：角色/地图/金/战斗结果在场景间正确传递
4. **资源注入**：ui.pak + 9 个 .Map 文件正确加载并渲染
5. **三端编译**：native/wasm/android 均可编译
