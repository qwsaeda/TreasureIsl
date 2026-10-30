# 场景状态机与事件分派

本文档定义完整的场景状态机、所有事件码、场景装载/卸载规则、全局单例管理。

---

## 1. 场景枚举与资源

| 场景 | 资源文件 | 作用 | 装载函数 |
|---|---|---|---|
| Intro | `intro.Map` | 片头动画，自动跳 Index | `intro` |
| Index | `index.Map` | 主菜单（6 项） | `index` |
| SelectRole | `selectrole.Map` | 角色选择（5 角色循环） | `selectrole` |
| SelectMap | `selectmap.Map` | 地图选择（已解锁地图循环） | `selectmap` |
| MainMap | `main.Map` + `glmapm0x.Map` | 主地图探索 | `main` |
| Vs | `vs.Map` | 战斗场景 | `战斗对象构造` |
| Message | `msgl/msgm/msgs/help/information.Map` | 消息/帮助/信息 | 动态选择 |
| Saveload | `saveload.Map` | 存读档 | 子窗口叠加 |
| Exit | — | 退出游戏 | 终态 |

### 1.1 装载器共同模式

```
new 场景对象 → 场景对象 构造（设 vtable） → 场景对象关联 关联 UI → 虚调用 +4 装载资源
```

---

## 2. 场景状态码

### 2.1 状态码定义

| 状态码 | 语义 | SceneState 枚举 |
|---|---|---|
| 0 | 主菜单态 | Menu |
| 2 | 场景卸载中 | Unloading |
| 3 | 回主菜单 | BackToMenu |
| 4 | 过场/坐标切换 | Transition |
| 25 | 战斗态 | Battle |
| 26 | 过场动画 phase 26 | Anim26 |
| 27 | 过场动画 phase 27 | Anim27 |

### 2.2 状态转换规则

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

---

## 3. 全局事件码

### 3.1 事件码定义

| 事件码 | 语义 | 适用场景 |
|---|---|---|
| 13 | 确定/Enter | 所有场景 |
| 27 | 返回/取消 | 所有子场景 |
| 37 | 无效操作（播 invalid 音效） | 所有场景 |
| 38 | 上一页/上一个 | 菜单/选角/选图/帮助/贸易 |
| 39 | 翻页确认（贸易专用） | 贸易 |
| 40 | 下一页/下一个 | 菜单/选角/选图/帮助/贸易 |

### 3.2 主菜单事件分派

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

### 3.3 音效表

| 音效 | 文件 | 触发 |
|---|---|---|
| Press | `music\press.wav` | 点击有效菜单项 |
| Invalid | `music\invalid.wav` | 无效操作（事件码 37） |
| Victory | `music\victory.wav` | 战斗胜利 |

---

## 4. 主菜单

### 4.1 菜单项

| 索引 | 标签 | 目标场景 | 音效 |
|---|---|---|---|
| 0 | 新游戏 | SelectRole | Press |
| 1 | 继续游戏 | Saveload（子窗口叠加） | Press |
| 2 | 选项 | Option | Press |
| 3 | 说明（第 1 页） | Help | Press |
| 4 | 说明（续，第 2 页） | Help | Press |
| 5 | 退出 | Exit | 无 |

### 4.2 菜单状态机

```
主菜单 6 项，上下键循环选择，Enter 确认
```

---

## 5. 角色选择

### 5.1 角色枚举

| 索引 | 名称 | 头像 | 立绘 | 精灵 |
|---|---|---|---|---|
| 0 | panda（熊猫海盗） | `face_panda.pcx` | `start_panda.pcx` | `Player_panda.pcx` |
| 1 | tichy（蒂奇） | `face_tichy.pcx` | `start_tichy.pcx` | `Player_tichy.pcx` |
| 2 | kidd（基德） | `face_kidd.pcx` | `start_kidd.pcx` | `Player_kidd.pcx` |
| 3 | drake（德雷克） | `face_drake.pcx` | `start_drake.pcx` | `Player_drake.pcx` |
| 4 | katharina（凯瑟琳娜） | `face_katharina.pcx` | `start_katharina.pcx` | `Player_katharina.pcx` |

### 5.2 选择流程

1. 进入 SelectRole 场景，显示 5 个角色头像 + `Target_knife.pcx` 选中标记。
2. 事件码 38 = 上一个角色（索引 0 → 4 循环），40 = 下一个（4 → 0 循环）。
3. 事件码 13 = 确认选择，记录角色到全局变量。
4. 事件码 27 = 返回 Index。

### 5.3 选择状态

```rust
struct SelectionState {
    character: Option<Character>,
    map_slot: Option<MapSlot>,
    completed: bool,
}
```

完成条件：角色非 None **且** 地图非 None → `completed = true`。

---

## 6. 地图选择

### 6.1 地图枚举

| 槽位 | 文件 | 网格 | 编号 | UI 参数 |
|---|---|---|---|---|
| M01 | `glmapm01.Map` | 20×20 | 1 | X=73, Y=100 |
| M02 | `glmapm02.Map` | 20×20 | 2 | X=67, Y=64（`selectmap` 行 89-100 实测写参，见 spec/selection.md §b） |
| M03 | `glmapm03.Map` | 15×15 | 3 | X=117, Y=81（推测，见 spec/selection.md §b 行 103-115） |

### 6.2 解锁条件

地图解锁 = 对应 `data/glmapm0x.Map` 文件存在（`CreateFileW` 探测）。

### 6.3 选择流程

1. 进入 SelectMap 场景，显示已解锁地图按钮。
2. 事件码 38/40 在已解锁地图间循环切换。
3. 事件码 13 = 确认，设置全局地图编号 `dword_7FB6C`（1/2/3）。
4. 未解锁地图 → `Sfx::Invalid`（状态不变）。
5. 角色已选 + 地图已选 → `completed = true` → 触发 `main.Map` 装载。

---

## 7. 主地图事件分派

### 7.1 事件码计算

```
event_code = (*(u8*)(*(u32*)(ctrl+12) + 22) >> 2) + 1
// 高 6 位 + 1，值域 1..31
```

### 7.2 主分派表

| 事件码 | 行为 | 目标场景 | 细节 |
|---|---|---|---|
| 1 | 据点/地标 | Msgl | 定位 + 判定命中 → 显示据点名 |
| 2 | 面板/对话框 | Msgm | 设坐标 + 定位 |
| 3 | 卡牌/战斗准备 | 无场景 | `rand()%25+1` 生成 event_id |
| 4 | 贸易/货物 | Msgl | 判 + `rand()` 哈希选商品 + 结算 |
| 5 | 卡片翻转 | Msgm | 动画序列 |
| 6 | 战斗触发 | Vs | 建 vs 对象 + 初始化 |
| 7 | 消息/提示 | Msgm | 方向计算 |
| 8 | 退出/返回 | Msgs/Msgm | 虚调用判断 |

### 7.3 事件码 9..31

主分派无 case（由场景管理器排队事件消费路径处理）。排队事件码 1..27（哨兵 27）。

### 7.4 事件 id 回填

| 来源 | 写入的 id |
|---|---|
| `主地图帧更新` | {1,2,3,4,5,11,12,13,17,18,19,20,21,24,25,26,27} |
| `场景管理器帧更新总控` | {6,7,8,9,10,14,15,16,22,23} |

---

## 8. 场景码切换

### 8.1 切换函数

```
00029110:场景管理器;0003D4C8:设置;0003F4E4:帮助;0004085C:选项;00041354:主菜单;0004220C:存读档;00042378:继续;000427E8:退出;000428F0:新游戏(ctx, scene_code):
    取场景管理器单例
    虚调用+4 判断是否接受切换
    接受 → 销毁旧场景 → 写入新场景码 → *(ctx+8) = 场景管理器
```

### 8.2 场景码表

| 场景码 | 目标 | 来源 |
|---|---|---|
| 4 | 回主地图/探索 | selectrole/selectmap 完成后跳转 |
| 6 | 回 index.Map 主菜单 | 子场景返回 |
| 8 | 新游戏（selectrole） | 主菜单 case 0 |
| 10 | help.map 返回 | 帮助场景 |
| 12 | 场景 12 | 选项场景 |

---

## 9. 全局单例管理

### 9.1 off_811A0（全局游戏状态单例）

```
构造：清零 8 个 u32 槽（+4..+28），+0 = -1，+32/+33 = 1，构造宽字符串（+36）
析构：清 off_811A0 = nullptr，析构子对象（+48）和宽字符串（+36）
```

### 9.2 dword_7FB6C（全局地图编号）

`selectmap` 确认选择时写入（1/2/3）。`main.Map` 装载时 `switch(dword_7FB6C)` 选地形文件。

### 9.3 dword_7FB70（全局角色索引）

`selectrole` 确认选择时写入（1..5）。存档 `gl*.sav` 写入/读出此字段。

### 9.4 byte_81194..81197（全局状态位）

`data/gl.rec` 读取的 4 个字节。全非零 → 5 角色初始化完成。

---

## 10. 输入映射（全场景）

### 10.1 Index 场景

| 按键 | 行为 |
|---|---|
| Up/Down | 循环选择菜单项 |
| Enter | 执行选中项 |
| Esc | 退出 |

### 10.2 SelectRole 场景

| 按键 | 行为 |
|---|---|
| Left/38 | 上一个角色 |
| Right/40 | 下一个角色 |
| Enter/13 | 确认角色 |
| Esc/27 | 返回 Index |

### 10.3 SelectMap 场景

| 按键 | 行为 |
|---|---|
| Left/38 | 上一个地图 |
| Right/40 | 下一个地图 |
| Enter/13 | 确认地图 |
| Esc/27 | 返回 Index |

### 10.4 MainMap 场景

| 按键 | 行为 |
|---|---|
| 方向键 | 朝该方向走一步 |
| Space | 触发据点事件 |
| Esc | 返回 Index |

### 10.5 Vs 场景

| 按键 | 行为 |
|---|---|
| Enter/Space | 攻击一回合 |
| Esc | 退出战斗 |

### 10.6 Message 场景

| 按键 | 行为 |
|---|---|
| Up/Down | 翻页 |
| Enter/Esc | 关闭 |

### 10.7 Saveload 场景

| 按键 | 行为 |
|---|---|
| Up/Down | 选槽位 |
| Enter | 读/写档 |
| Esc | 关闭 |

---

## 11. 消息界面常量

| 种类 | 总页数 |
|---|---|
| Msgs | 1 |
| Msgl | 1 |
| Msgm | 1 |
| Help | 3 |
| Info | 1 |

### 11.1 对话框对象大小

| 族 | 大小（字节） | vtable |
|---|---|---|
| 对话框族（msgl/msgm/msgs） | 1552 | `off_6FDFC` |
| Help 族 | 2096 | `off_6F47C` |
| Info 族 | 1164 | `off_6F4FC` |

---

## 12. 实现检查清单

- [ ] 所有场景可进入/返回
- [ ] 事件码分派完整
- [ ] 全局单例正确管理
- [ ] 输入映射全场景覆盖
- [ ] 状态转换无死锁
