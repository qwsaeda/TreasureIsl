# UI 流程（场景流 / 控件 / 输入 / 消息）

**定位**：表现层（`crates/ti-app`）**并行开发的唯一业务依据**。
表现层改动前先读本文件；本文件与代码冲突时以本文件为准，改代码或改本文件（评审），不留两个真相。

配套：玩法规则见 [`loop_core.md`](loop_core.md)/[`economy.md`](economy.md)/[`combat.md`](combat.md)/
[`progression.md`](progression.md)/[`persistence.md`](persistence.md)；接线手册见 [`docs/gamebook.md`](../gamebook.md)。

## 1. 场景流

9 个场景 + 20 条合法转换（`scene_transition!` 宏 19 条 + 路由层 `Intro`/`Exit` 就地 `enter` 派生 1 条；`Intro`/`Exit` 无场景 struct，在路由层处理）。**本节是转换表的唯一事实源**；代码侧
`crates/ti-app/src/scenes/mod.rs` 的 `scene_transition!` 宏逐条对应同一张表
（加转换必须两处同改；删转换同理——集成轮由该双向一致性判据把关）。

| From | To | 触发输入 | 前置条件 |
|---|---|---|---|
| `Intro` | `Index` | 任意确认 | 片头播放结束 |
| `Index` | `SelectRole` | 确认菜单项 0 | 菜单项 0「新游戏」高亮 |
| `Index` | `Saveload` | 确认菜单项 1 | 菜单项 1「继续」高亮 |
| `Index` | `Message` | 确认菜单项 2/3/4 | 以 `MsgKind::Help` 打开 |
| `Index` | `Exit` | 确认菜单项 5 | 先置退出请求 |
| `SelectRole` | `SelectMap` | 确认角色 | 角色已选中（地图未定） |
| `SelectRole` | `MainMap` | 确认角色 | 角色 + 地图齐全（读档或速通） |
| `SelectRole` | `Index` | 返回 | — |
| `SelectMap` | `MainMap` | 确认地图 | 地图已选中 |
| `SelectMap` | `SelectRole` | 返回 | — |
| `MainMap` | `Vs` | 落格事件码 6 | 战斗触发条件成立 |
| `MainMap` | `Message` | 事件码 1/4/2/5/7/8 | 据点/地标/结算面板 |
| `MainMap` | `Index` | 返回（`MapBack`） | — |
| `Vs` | `Index` | 确认/返回 | 战斗结束或中途退出 |
| `Message` | `MainMap` | 关闭 | 打开源场景是主地图 |
| `Message` | `Index` | 关闭 | 打开源场景是主菜单 |
| `Saveload` | `Index` | 关闭（返回） | — |
| `Saveload` | `MainMap` | 读档成功 | 存档含角色 + 地图 |
| `Saveload` | `SelectRole` | 读档成功 | 存档缺角色或缺地图 |
| `Intro` | `Index`（派生） | 片头结束 | 路由层 `route_input` 就地 `enter`（无场景 struct） |

**硬规则**

1. 非法转换（如 `SelectRole → Vs`、`MainMap → Saveload`）**不存在**：类型层面无实现，编译期报错；
2. 场景判别值 `SceneKind::discriminant()` 取 0..8，**一场景一值**（禁两个场景共用一个键，S10.2 修复项）；
3. 任何跳转都必须更新 `scenes` 状态码（`rules::main_loop` 的 `set_state`），否则静态帧优化跳帧；
4. 切场景后关闭态必须复位（选中项/页码/当前棋子位置），禁止残留（S10.2 修复项）。

## 2. 控件规格

- 控件表 = `assets/data/controls.json`（20 场景 / 2041 条记录 / `kind` 取值 0..=44），
  由 `legacy/scripts/gen_layout.py` 从原版控件记录生成；**代码禁止硬编码控件位置**；
- **可见性** = `flags & 1 != 0` 且 `w > 0 && h > 0`（不可见控件不绘制、不参与 hit）；
- **命中边界** = 控件矩形 `(x, y, w, h)`，触摸点在其内即命中该控件；
- `kind = 0` 表示无控件装饰（透明底），`kind` 目前只决定色板索引（`kind % 8`），
  44 种类型的业务语义归类**尚未定案**——不定案前禁止臆造映射表（新增归类必须走评审）；
- 场景级布局（标题/列表/提示行坐标）走 `assets/data/scenes.json`，不在 `.rs` 里写魔法数。

## 3. 输入映射

输入**先归一**为 `GameInput` 再进场景，键盘与触摸共用同一套业务判定
（因此一个场景只实现一次输入，见 [`docs/process/01_architecture.md`](../process/01_architecture.md) §2）。

### 3.1 键盘（`KeyState` → `GameInput`）

| 场景 | 键 | 事件 |
|---|---|---|
| `Index` | ↑/↓ | `MoveUp`/`MoveDown`（6 项循环） |
| `Index` | 确认 | 菜单项 0..=5 → 见 §1 |
| `SelectRole`/`SelectMap` | ↑/←、↓/→ | `Prev`/`Next` |
| `SelectRole`/`SelectMap` | 确认、返回 | `Confirm`/`Back` |
| `MainMap` | ↑/↓/←/→ | `Walk(North/South/West/East)` |
| `MainMap` | Shift+↑/↓ | 切换选中商品与选中建筑 |
| `MainMap` | T / Y / B | `TradeBuy`/`TradeSell`/`Build`（作用在当前选中项） |
| `MainMap` | 据点键 | `Landmark(6)` |
| `MainMap` | 返回 | `MapBack` |
| `Vs` | 确认/据点键、返回 | `Attack`/`Back` |
| `Message` | ↓/↑、确认或返回 | `MsgNext`/`MsgPrev`/`MsgClose` |
| `Saveload` | 确认、返回 | `WriteSave(0)`/`SaveloadClose` |
| `Intro`/`Exit` | 确认 | 跳 `Index` |

### 3.2 触摸（直接触摸界面元素，不画虚拟按键）

| 场景 | 点按 | 滑动 |
|---|---|---|
| `Index` | 点菜单行 = 选中并立即执行 | — |
| `SelectRole`/`SelectMap` | 点行 = 选中并确认 | 下滑 = 返回 |
| `MainMap` | 点格子 = 朝该格走；点自身格 = 触发据点 | 滑动 = 朝该方向走一步 |
| `Vs` | 点任意处 = 攻击一回合 | 下滑 = 退出 |
| `Message` | 左半屏 = 关闭；右半屏 = 下一页 | 上滑 = 下一页；下滑 = 上一页 |
| `Saveload` | 点有档槽 = 读档；点空槽 = 存档 | 下滑 = 关闭 |

**坐标约定（硬规则）**

1. 表现层一切坐标都是 **176×220 逻辑坐标**；屏幕坐标由
   `Viewport::screen_to_logical` 反变换后再进业务层；
2. 主地图点按**必须**走等距投影逆变换（与渲染同一坐标系），
   公式与 [`progression.md`](progression.md) §地图结构一致；渲染与触摸用两套坐标系 = 界错乱（S10.1）；
3. 手势阈值 = 20 逻辑像素（`TouchPad::default_threshold`），越阈才判滑动，否则判点按。

## 4. 消息分页

- 5 类界面：`Msgs`（通用提示/退出对话框）、`Msgl`（据点对话框 + 结算面板）、
  `Msgm`（多页面板/卡片/贸易结果）、`Help`（多页帮助）、`Info`（信息/角色状态）；
- **页数来自 `balance.messages.pages`**（`rules::messages::total_pages` 读表），禁止硬编码；
- 当前页 `page ∈ 0..total_pages`，上/下页**钳位**（首尾不环绕）；
- 关闭时回到**打开源场景**（见 §1 `Message` 的两条出边），不得回固定场景；
- 标题与正文文本走 `strings.json`（`msg.<kind>.label` + 页正文 key），禁止中文面量。

## 5. UI 约束

| # | 约束 | 理由 |
|---|---|---|
| 1 | 逻辑分辨率固定 176×220，缩放只改呈现不改逻辑 | 三端坐标一致，P4-2 互逆不变式 |
| 2 | 滤波 Nearest/Linear 由设置切换，坐标换算不变 | 画质切换不得改变判定位置 |
| 3 | 等距朝向由 `iso_y = 16*(row + grid_w - 1 - col)` 决定 | 上下颠倒会让人以为走反方向 |
| 4 | 场景资源键 = `SceneKind::asset_name()`（`Intro` 复用 `index.Map`，`Exit` 无资源） | HD 注入按名寻址，键必须唯一 |
| 5 | HD 与 CPU 双路径并存，PNG 缺资源时回退不 panic | S7.5 降级约定 |
| 6 | 全部中文进 `strings.json`，全部数值进 `balance.json` | 改文案/数值不碰代码 |

## 6. 可测试规则表

| # | 规则 | 单测名（`crates/ti-app/tests/`） |
|---|---|---|
| 1 | 菜单项 0..5 上下循环不越界 | `menu_cursor_wraps` |
| 2 | 菜单项 0/1/5 分别跳 `SelectRole`/`Saveload`/退出 | `menu_item_targets` |
| 3 | 非法转换不可构造（编译期） | `scenes/mod.rs` 的 `compile_fail` 文档测试 |
| 4 | 9 个场景判别值互不相同 | `scene_discriminants_unique` |
| 5 | 控件可见性判定（`flags&1` + 尺寸） | `widget_visible_filter` |
| 6 | 控件 hit 边界矩形内外各一点 | `widget_hit_bounds` |
| 7 | 触摸坐标反变换往返一致 | `touch_logical_roundtrip` |
| 8 | 主地图点按与棋子移动方向一致（同坐标系） | `iso_touch_matches_move` |
| 9 | 手势阈值：位移 <20 判点按、≥20 判滑动 | `gesture_threshold_split` |
| 10 | 消息翻页首尾钳位 | `msg_page_clamped` |
| 11 | 消息关闭回到打开源场景 | `msg_close_returns_source` |
| 12 | 切场景后选中项/页码复位（无残留） | `scene_switch_resets_state` |