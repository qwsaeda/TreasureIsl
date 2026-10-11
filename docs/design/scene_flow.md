> ⚠️ **弃用/归档（S8，2026-10）**：本目录为逆向规格参照物，不再是开发依据或验收基准。
> 业务规则唯一事实源 = [`docs/business/`](../business/00_index.md) + `assets/data/*.json`。
> 本文仅作原版形态参考；数值/文本以 `assets/data/*.json` 为准，流程以 `docs/business/` 为准。
> 见 [`docs/plan-s8-refactor.md`](../plan-s8-refactor.md) §1（逆向产物降级）。

# 场景流与菜单设计

本文档涵盖：场景状态机、主菜单、角色/地图选择、全局事件码体系。

---

## 1. 场景枚举

| 场景 | 资源文件 | 作用 |
|---|---|---|
| Intro | `intro.Map` | 片头动画，自动跳 Index |
| Index | `index.Map` | 主菜单（6 项） |
| SelectRole | `selectrole.Map` | 角色选择（5 角色循环） |
| SelectMap | `selectmap.Map` | 地图选择（已解锁地图循环） |
| MainMap | `main.Map` + `glmapm0x.Map` | 主地图探索（核心可玩场景） |
| Vs | `vs.Map` | 战斗场景 |
| Message | `msgl/msgm/msgs/help/information.Map` | 消息/帮助/信息（叠加窗口） |
| Saveload | `saveload.Map` | 存读档（子窗口叠加） |
| Exit | — | 退出游戏 |

### 装载器

每个场景有独立的装载器函数，共同模式：`new 场景对象 → 场景对象 构造（设 vtable） → 场景对象关联 关联 UI → 虚调用 +4 装载资源`。

| 场景 | 装载函数 | 额外加载 |
|---|---|---|
| Intro | `intro` | `ui.pak` + `intro.Map` |
| SelectRole | `selectrole` | `ui.pak` + `selectrole.Map` + `glpics.pak` + 11 控件 |
| SelectMap | `selectmap` | `ui.pak` + `selectmap.Map` + 探测 `glmapm01/02/03.Map` |
| Index | `index` | `ui.pak` + `index.Map` + `glpics.pak` + `data/gl.rec` 魔数校验 |
| MainMap | `main` | `glpics.pak` + `switch(dword_7FB6C)` 选 `glmapm0x.Map` + `main.Map` + `ui.pak` + 29 按钮 |
| Vs | `战斗对象构造` | `ui.pak` + `vs.Map` |

---

## 2. 主菜单事件分派

### 2.1 菜单项

主菜单 6 项，上下键循环选择，Enter 确认：

| 索引 | 标签 | 目标场景 | 音效 |
|---|---|---|---|
| 0 | 新游戏 | SelectRole | Press |
| 1 | 继续游戏 | Saveload（子窗口叠加，非场景切换） | Press |
| 2 | 选项 | Option | Press |
| 3 | 说明（第 1 页） | Help | Press |
| 4 | 说明（续，第 2 页） | Help | Press |
| 5 | 退出 | Exit | 无 |

### 2.2 菜单状态机

```
oracle_menu_transition(当前场景, 事件) → (新场景, 可选音效)
```

| 当前场景 | 事件 | 新场景 | 音效 |
|---|---|---|---|
| Intro | 任何 | Index | 无 |
| Intro | Quit | Exit | 无 |
| Index | Click(0) 或 Click(13) | SelectRole | Press |
| Index | Click(1) | Saveload | Press |
| Index | Click(2) | Option | Press |
| Index | Click(3) 或 Click(4) | Help | Press |
| Index | Click(5) 或 Quit | Exit | 无 |
| Index | Click(37) 或 Invalid | Index（保持） | Invalid |
| Index | Click(38) | Index（保持） | Invalid |
| Index | 其他 Click | Index（保持） | 无 |
| SelectRole/SelectMap/... | Click(27) | Index | 无 |
| SelectRole/SelectMap/... | Invalid 或 Click(37) | 保持当前 | Invalid |
| SelectRole/SelectMap/... | Quit | Exit | 无 |
| SelectRole/SelectMap/... | 其他 Click | 保持当前 | 无 |
| Exit | 任何 | Exit（终态） | 无 |

### 2.3 音效

| 音效 | 文件 | 触发 |
|---|---|---|
| Press | `music\press.wav` | 点击有效菜单项 |
| Invalid | `music\invalid.wav` | 无效操作（事件码 37） |
| Victory | `music\victory.wav` | 战斗胜利 |

---

## 3. 角色选择

### 3.1 角色枚举

| 索引 | 名称 | 头像 | 立绘 | 精灵 |
|---|---|---|---|---|
| 0 | panda（熊猫海盗） | `face_panda.pcx` | `start_panda.pcx` | `Player_panda.pcx` |
| 1 | tichy（蒂奇） | `face_tichy.pcx` | `start_tichy.pcx` | `Player_tichy.pcx` |
| 2 | kidd（基德） | `face_kidd.pcx` | `start_kidd.pcx` | `Player_kidd.pcx` |
| 3 | drake（德雷克） | `face_drake.pcx` | `start_drake.pcx` | `Player_drake.pcx` |
| 4 | katharina（凯瑟琳娜） | `face_katharina.pcx` | `start_katharina.pcx` | `Player_katharina.pcx` |

控件注入器共 11 个控件槽位：5 个 `face_*` + 5 个 `Player_*` + 1 个 `Target_knife.pcx`。

### 3.2 选择流程

1. 进入 SelectRole 场景，显示 5 个角色头像 + `Target_knife.pcx` 选中标记。
2. 事件码 38 = 上一个角色（索引 0 → 4 循环），40 = 下一个（4 → 0 循环）。
3. 事件码 13 = 确认选择，记录角色到全局变量。
4. 事件码 27 = 返回 Index。

### 3.3 选择状态

```
struct SelectionState {
    character: Option<Character>,  // 已选角色
    map_slot: Option<MapSlot>,      // 已选地图
    completed: bool,               // 角色+地图均选定 → 触发 main.Map
}
```

完成条件：角色非 None **且** 地图非 None → `completed = true`。

无论先选角色还是先选地图，只要两者均选定即完成。

---

## 4. 地图选择

### 4.1 地图枚举

| 槽位 | 文件 | 网格 | 编号 | UI 参数 |
|---|---|---|---|---|
| M01 | `glmapm01.Map` | 20×20 | 1 | X=73, Y=100 |
| M02 | `glmapm02.Map` | 20×20 | 2 | X=67, Y=64 |
| M03 | `glmapm03.Map` | 15×15 | 3 | X=117, Y=81 |

### 4.2 解锁条件

地图解锁 = 对应 `data/glmapm0x.Map` 文件存在（`CreateFileW` 探测）。

新游戏时可能只有 `glmapm01.Map`，后续关卡解锁 02/03。解锁由存档系统控制（`glmapm0x.Map` 文件由存档写出或预置）。

### 4.3 选择流程

1. 进入 SelectMap 场景，显示已解锁地图按钮。
2. 事件码 38/40 在已解锁地图间循环切换。
3. 事件码 13 = 确认，设置全局地图编号 `dword_7FB6C`（1/2/3）。
4. 未解锁地图 → `Sfx::Invalid`（状态不变，`completed` 保持 `false`）。
5. 角色已选 + 地图已选 → `completed = true` → 触发 `main.Map` 装载。

### 4.4 15 种完整组合

5 角色 × 3 地图 = 15 种合法开局组合。每种组合从空状态出发：
```
select_role(ch) → select_map(m) → completed == true
```

---

## 5. 全局事件码

所有场景共用的事件码：

| 事件码 | 语义 | 适用场景 |
|---|---|---|
| 13 | 确定/Enter | 所有场景 |
| 27 | 返回/取消 | 所有子场景 |
| 37 | 无效操作（播 invalid 音效） | 所有场景 |
| 38 | 上一页/上一个 | 菜单/选角/选图/帮助/贸易 |
| 39 | 翻页确认（贸易专用） | 贸易 |
| 40 | 下一页/下一个 | 菜单/选角/选图/帮助/贸易 |

---

## 6. 场景状态码

场景管理器内部用状态码 `*(a1+716)` 跟踪，合法取值 7 个：

| 状态码 | 语义 | SceneState 枚举 |
|---|---|---|
| 0 | 主菜单态 | Menu |
| 2 | 场景卸载中 | Unloading |
| 3 | 回主菜单 | BackToMenu |
| 4 | 过场/坐标切换 | Transition |
| 25 | 战斗态 | Battle |
| 26 | 过场动画 phase 26 | Anim26 |
| 27 | 过场动画 phase 27 | Anim27 |

### 状态转换规则

```
oracle_next_state(当前状态, 触发码) → (新状态, 目标场景)
```

| 当前 | 触发码 | 含义 | 新状态 | 目标场景 |
|---|---|---|---|---|
| 0 | 1 | 进战斗 | 25 | Vs |
| 0 | 其他 | — | 0 | — |
| 2 | 5 | 动画完成 | 0 | Index |
| 2 | 其他 | — | 2 | — |
| 3 | 任意 | 回主菜单 | 0 | Index |
| 4 | 4 | 触发过场 | 27 | Intro |
| 4 | 其他 | — | 4 | — |
| 25 | 2 | 战斗结束 | 0 | Index |
| 25 | 其他 | — | 25 | — |
| 26 | 5 | 动画完成 | 27 | Intro |
| 26 | 其他 | — | 26 | — |
| 27 | 5 | 动画完成 | 0 | Index |
| 27 | 其他 | — | 27 | — |

触发码含义：0=无触发，1=进战斗，2=战斗结束，3=回菜单，4=触发过场，5=过场完成。

### 场景码切换函数

```
00029110:场景管理器;0003D4C8:设置;0003F4E4:帮助;0004085C:选项;00041354:主菜单;0004220C:存读档;00042378:继续;000427E8:退出;000428F0:新游戏(ctx, scene_code):
  取场景管理器单例
  虚调用+4 判断是否接受切换
  接受 → 销毁旧场景（+8 虚函数 + vtable[0] 传 1）
       → 写入新场景码
       → *(ctx+8) = 场景管理器
```

| 场景码 | 目标 | 来源 |
|---|---|---|
| 4 | 回主地图/探索 | selectrole/selectmap 完成后跳转 |
| 6 | 回 index.Map 主菜单 | 子场景返回 |
| 8 | 新游戏（selectrole） | 主菜单 case 0 |
| 10 | help.map 返回 | 帮助场景 |
| 12 | 场景 12 | 选项场景 |

---

## 7. 兵种奖励

5 种兵种/类型对应 5 档投资奖励：

| 兵种 | 奖励金 | 额外效果 |
|---|---|---|
| 兵种 A（陆军） | +30,000 | — |
| 兵种 B（海军） | +40,000 | — |
| 兵种 C（空军/炮兵） | +20,000 | — |
| 兵种 D（骑兵/快速） | +15,000 | 等级 +5 |
| 兵种 E（重甲/攻城） | +50,000 | 额外 +20 经验 |

兵种图鉴：`troop_all.pcx` / `ship_all.pcx`。

---

## 8. 全局单例

### off_811A0（全局游戏状态单例）

构造：清零 8 个 u32 槽（+4..+28），+0 = -1，+32/+33 = 1，构造宽字符串（+36）。
析构：清 `off_811A0 = nullptr`，析构子对象（+48）和宽字符串（+36）。
推测为存档/设置承载体。

### dword_7FB6C（全局地图编号）

`selectmap` 确认选择时写入（1/2/3）。
`main.Map` 装载时 `switch(dword_7FB6C)` 选地形文件。

### dword_7FB70（全局角色索引）

`selectrole` 确认选择时写入（1..5）。
存档 `gl*.sav` 写入/读出此字段。

### byte_81194..81197（全局状态位）

`data/gl.rec` 读取的 4 个字节。
全非零 → 5 角色初始化完成。
