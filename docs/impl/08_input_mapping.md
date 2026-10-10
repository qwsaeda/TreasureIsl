# 输入系统实现规格

本文档是输入系统的完整实现规格，实现者无需阅读其他文档即可写出正确的输入处理代码。

> 事实来源：`docs/design/input_mapping.md` + `docs/design/ui_controls.md` §5 + `docs/design/constants.md` §7。

---

## 1. 逻辑输入事件

```rust
/// 归一化输入事件（物理键/触摸 → 逻辑动作）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameInput {
    // 方向
    Up,
    Down,
    Left,
    Right,
    // 动作
    Confirm,
    Cancel,
    Attack,
    // 功能键
    CycleScale,
    CycleFilter,
    CycleBorder,
    // 其他
    None,
}
```

---

## 2. 物理按键映射

### 2.1 键盘

| 物理键 | GameInput | 说明 |
|---|---|---|
| ↑ / W | `Up` | 向上移动/上一个 |
| ↓ / S | `Down` | 向下移动/下一个 |
| ← / A | `Left` | 向左移动 |
| → / D | `Right` | 向右移动 |
| Enter / 空格 | `Confirm` | 确定/选择 |
| Esc / Backspace | `Cancel` | 返回/取消 |
| Space | `Confirm` | 空格同时是确定（与 Enter 同） |
| F1 | `CycleScale` | 循环缩放模式 |
| F2 | `CycleFilter` | 循环滤波模式 |
| F3 | `CycleBorder` | 循环边框样式 |

### 2.2 触摸

| 手势 | GameInput | 说明 |
|---|---|---|
| 点按（tap） | `Confirm` | 确认命中区域 |
| 上滑 | `Up` | 方向 |
| 下滑 | `Down` | 方向 |
| 左滑 | `Left` | 方向 |
| 右滑 | `Right` | 方向 |
| 长按 0.5s | `Cancel` | 返回 |
| 双指捏合 | `CycleScale` | 循环缩放 |

### 2.3 手柄

| 手柄输入 | GameInput | 说明 |
|---|---|---|
| 左摇杆上 | `Up` | 方向 |
| 左摇杆下 | `Down` | 方向 |
| 左摇杆左 | `Left` | 方向 |
| 左摇杆右 | `Right` | 方向 |
| A 键 | `Confirm` | 确定 |
| B 键 | `Cancel` | 返回 |
| 十字键上 | `Up` | 方向 |
| 十字键下 | `Down` | 方向 |
| 十字键左 | `Left` | 方向 |
| 十字键右 | `Right` | 方向 |

---

## 3. 虚拟按键

```rust
pub struct VirtualButton {
    pub name: &'static str,
    pub rect: Rect,
}

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl VirtualButton {
    pub fn hit(&self, x: i32, y: i32) -> bool {
        x >= self.rect.x
            && x < self.rect.x + self.rect.w as i32
            && y >= self.rect.y
            && y < self.rect.y + self.rect.h as i32
    }
}
```

命中判定：`x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h`。

---

## 4. 事件翻译管线

```rust
pub enum ScreenEvent {
    PointerDown { x: f32, y: f32 },
    PointerUp { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
    KeyDown { key: KeyCode },
    KeyUp { key: KeyCode },
    Quit,
}

pub fn translate_event(screen_event: ScreenEvent, viewport: &Viewport) -> Event {
    match screen_event {
        ScreenEvent::PointerDown { x, y } => {
            let (lx, ly) = viewport.screen_to_logical(x, y);
            Event::PointerDown { x: lx, y: ly }
        }
        ScreenEvent::PointerUp { x, y } => {
            let (lx, ly) = viewport.screen_to_logical(x, y);
            Event::PointerUp { x: lx, y: ly }
        }
        ScreenEvent::PointerMove { x, y } => {
            let (lx, ly) = viewport.screen_to_logical(x, y);
            Event::PointerMove { x: lx, y: ly }
        }
        ScreenEvent::KeyDown { key } => Event::KeyDown { key },
        ScreenEvent::KeyUp { key } => Event::KeyUp { key },
        ScreenEvent::Quit => Event::Quit,
    }
}
```

---

## 5. 全局事件码

| 事件码 | 语义 | 音效 |
|---|---|---|
| 13 | 确定/Enter | Press |
| 27 | 返回/取消 | 无 |
| 37 | 无效操作 | Invalid |
| 38 | 上一页/上一个 | 无 |
| 39 | 翻页确认（贸易专用） | 无 |
| 40 | 下一页/下一个 | 无 |

---

## 6. 场景行为表

### 6.1 Index（主菜单）

| GameInput | 行为 | 实现 |
|---|---|---|
| `Up` | 菜单索引 -1（循环） | `menu_selected = (menu_selected + 5) % 6` |
| `Down` | 菜单索引 +1（循环） | `menu_selected = (menu_selected + 1) % 6` |
| `Confirm` | 执行选中项 | 见下表 |
| `Cancel` | 无（已在主菜单） | — |

**菜单项 → 目标场景**：

| 索引 | 标签 | Confirm 行为 |
|---|---|---|
| 0 | 新游戏 | 重置选择状态 → 进入 `SelectRole` |
| 1 | 继续游戏 | 进入 `Saveload`（子窗口叠加） |
| 2 | 选项 | 进入 `Message`（Help kind） |
| 3 | 说明（第 1 页） | 进入 `Message`（Help kind） |
| 4 | 说明（续，第 2 页） | 进入 `Message`（Help kind） |
| 5 | 退出 | 置 `quit = true`，循环终止 |

### 6.2 SelectRole（角色选择）

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一个角色（索引 0 → 4 循环） |
| `Down` / `Right` | 下一个角色（索引 4 → 0 循环） |
| `Confirm` | 选定角色 → 进入 `SelectMap` |
| `Cancel` | 返回 `Index` |

### 6.3 SelectMap（地图选择）

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一张地图（在已解锁列表中循环） |
| `Down` / `Right` | 下一张地图 |
| `Confirm` | 选定地图 → 进入 `MainMap` |
| `Cancel` | 返回 `SelectRole` |

### 6.4 Help 帮助

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一页（到首页回末页，循环） |
| `Down` / `Right` | 下一页（到末页保持） |
| `Confirm` / `Cancel` | 关闭 → 回 `Index` |

### 6.5 MainMap（主地图）

| GameInput | 行为 |
|---|---|
| `Up` | 向北走一步（`Dir::North`） |
| `Down` | 向南走一步（`Dir::South`） |
| `Left` | 向西走一步（`Dir::West`） |
| `Right` | 向东走一步（`Dir::East`） |
| `Confirm` / `Attack` | 触发当前位置的据点事件（`Landmark(code)`） |
| `Cancel` | 触发地图返回事件（`MapBack`）→ 回 `Index` |

### 6.6 Trade 贸易

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 商品索引 -1 |
| `Down` / `Right` | 商品索引 +1 |
| `Confirm` | 确认买卖 |
| `Cancel` | 取消 |

贸易场景事件码：

| 事件码 | 行为 |
|---|---|
| 13 | 买入/卖出确认 |
| 37 | 取消 |
| 38 | 翻上页 |
| 39 | 翻页确认 |
| 40 | 翻页 |

进入贸易前置：状态码 21（据点结算完成态）。
进入贸易：事件码 22（买入）/ 23（卖出）。

### 6.7 Saveload（存读档）

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一槽位 |
| `Down` / `Right` | 下一槽位 |
| `Confirm` | 有档 → 读档；空槽 → 存档 |
| `Cancel` | 关闭 → 回 `Index` |

### 6.8 Vs（战斗）

| GameInput | 行为 |
|---|---|
| `Confirm` / `Attack` | 攻击一回合 |
| `Cancel` | 退出战斗 → 回 `Index` |
| 方向 | 无（战斗无方向操作） |

### 6.9 Card（卡牌）

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一张卡 |
| `Down` / `Right` | 下一张卡 |
| `Confirm` | 选中/确认卡牌 |
| `Cancel` | 关闭 → 回 `MainMap` |

---

## 7. 场景码切换值

| 场景码 | 目标 |
|---|---|
| 4 | 回主地图/探索 |
| 6 | 回 index.Map 主菜单 |
| 8 | 新游戏（selectrole） |
| 10 | help.map 返回 |
| 12 | 场景 12 |

---

## 8. 主地图事件分派

| 事件码 | 目标场景 |
|---|---|
| 1 | Msgl（据点） |
| 2 | Msgm（面板） |
| 3 | 无（卡牌随机数） |
| 4 | Msgl（货物结果） |
| 5 | Msgm（卡片翻转） |
| 6 | Vs（战斗） |
| 7 | Msgm（消息） |
| 8 | Msgs / Msgm（退出/返回） |

---

## 9. 场景状态码

| 状态码 | 语义 |
|---|---|
| 0 | 主菜单态 |
| 2 | 场景卸载中 |
| 3 | 回主菜单 |
| 4 | 过场/坐标切换 |
| 25 | 战斗态 |
| 26 | 过场动画 phase 26 |
| 27 | 过场动画 phase 27 |

---

## 10. 实现注意事项

1. **输入去抖**：同一帧内多个键同时按下时，按场景语义合并（如 MainMap 中
   `Confirm` 优先于方向）。
2. **触摸滑动阈值**：滑动距离 > 20 逻辑像素才判定为方向手势，避免误触。
3. **长按判定**：按住 500ms 才触发 `Cancel`（长按），短按仍为 `Confirm`。
4. **场景切换时重置输入**：避免上一帧的按键残留导致新场景误触发。
5. **Saveload 是子窗口**：不是独立场景，而是叠加在 `Index` 上的状态，关闭后回
   `Index`。

---

## 11. 现代化输入映射

| 原操作 | 桌面 | 触屏 | 手柄 |
|---|---|---|---|
| 方向移动 | 方向键 / WASD | 左半区虚拟十字键 / 滑动手势 | 左摇杆 |
| 确定（13） | Enter / 空格 | 右半区「确认」按钮 / 双击 | A 键 |
| 返回（27） | ESC | 「返回」按钮 / 长按 0.5s | B 键 |
| 上一页（38） | ↑ / Q | 上滑 | 十字键上 |
| 下一页（40） | ↓ / E | 下滑 | 十字键下 |

---

## 12. 方向枚举

| 方向码 | 枚举 | Δ(row, col) | 方向位 |
|---|---|---|---|
| 0 | North（北） | (-1, 0) | 8 |
| 1 | West（西） | (0, -1) | 4 |
| 2 | South（南） | (+1, 0) | 2 |
| 3 | East（东） | (0, +1) | 1 |
