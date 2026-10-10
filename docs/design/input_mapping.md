# 输入映射层

本文档是**自包含的实现规格**：把物理按键/触摸映射到逻辑 `GameInput` 事件，
再到场景行为。实现者无需阅读其他文档即可写出正确的输入处理代码。

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

---

## 3. 场景行为映射

### 3.1 Index（主菜单）

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

### 3.2 SelectRole（角色选择）

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一个角色（索引 0 → 4 循环） |
| `Down` / `Right` | 下一个角色（索引 4 → 0 循环） |
| `Confirm` | 选定角色 → 进入 `SelectMap` |
| `Cancel` | 返回 `Index` |

### 3.3 SelectMap（地图选择）

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一张地图（在已解锁列表中循环） |
| `Down` / `Right` | 下一张地图 |
| `Confirm` | 选定地图 → 进入 `MainMap` |
| `Cancel` | 返回 `SelectRole` |

### 3.4 MainMap（主地图）

| GameInput | 行为 |
|---|---|
| `Up` | 向北走一步（`Dir::North`） |
| `Down` | 向南走一步（`Dir::South`） |
| `Left` | 向西走一步（`Dir::West`） |
| `Right` | 向东走一步（`Dir::East`） |
| `Confirm` / `Attack` | 触发当前位置的据点事件（`Landmark(code)`） |
| `Cancel` | 触发地图返回事件（`MapBack`）→ 回 `Index` |

### 3.5 Vs（战斗）

| GameInput | 行为 |
|---|---|
| `Confirm` / `Attack` | 攻击一回合 |
| `Cancel` | 退出战斗 → 回 `Index` |
| 方向 | 无（战斗无方向操作） |

### 3.6 Message（消息/帮助/信息）

| GameInput | 行为 |
|---|---|
| `Down` / `Right` | 下一页（到末页保持） |
| `Up` / `Left` | 上一页（到首页保持） |
| `Confirm` / `Cancel` | 关闭 → 回来源场景 |

### 3.7 Saveload（存读档）

| GameInput | 行为 |
|---|---|
| `Up` / `Left` | 上一槽位 |
| `Down` / `Right` | 下一槽位 |
| `Confirm` | 有档 → 读档；空槽 → 存档 |
| `Cancel` | 关闭 → 回 `Index` |

---

## 4. 事件码参考

所有场景共用的事件码体系：

| 事件码 | 语义 | 适用场景 |
|---|---|---|
| 13 | 确定/Enter | 所有 |
| 27 | 返回/取消 | 所有子场景 |
| 37 | 无效操作 | 所有 |
| 38 | 上一页/上一个 | 菜单/选角/选图/帮助/贸易 |
| 39 | 翻页确认 | 贸易 |
| 40 | 下一页/下一个 | 菜单/选角/选图/帮助/贸易 |

---

## 5. 实现注意事项

1. **输入去抖**：同一帧内多个键同时按下时，按场景语义合并（如 MainMap 中
   `Confirm` 优先于方向）。
2. **触摸滑动阈值**：滑动距离 > 20 逻辑像素才判定为方向手势，避免误触。
3. **长按判定**：按住 500ms 才触发 `Cancel`（长按），短按仍为 `Confirm`。
4. **场景切换时重置输入**：避免上一帧的按键残留导致新场景误触发。
5. **Saveload 是子窗口**：不是独立场景，而是叠加在 `Index` 上的状态，关闭后回
   `Index`。
