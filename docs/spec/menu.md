# S3.3 index/intro 主菜单 — 逆向规格

数据源：`ref/pseudocode/*.c`（IDA Hex-Rays）、`ref/functions_menu.csv`、`docs/spec/main_loop.md` §4。
结论标注：`证据` = 指到具体伪代码文件 + 行号；`推测` = 需进一步验证。

---

## §a index.Map 主菜单场景结构

### 装载器（证据）

`sub_40D80`（`00040D80_sub_40D80.c`）：
- 行 37：`sub_37EDC(v20, v17, aUiPak)` — 从 `UI.pak` 索引取控件表；
- 行 41：`sub_37EDC(v21, v17, aIndexMap)` — 取 `index.Map` 场景记录；
- 行 45-49：`operator new(0x3FC)` + `sub_43B8C(v7, a1[2])` — 场景对象 ctor；
- 行 53：`sub_43D54(v8, *(a1[1]+16))` — 关联 UI 句柄；
- 行 56-63：`(**(scene)+4)(...)` 虚调用 — 装载资源/控件表；
- 行 64：`sub_37EDC(v22, v17, aGlpicsPak)` — 加载 `glpics.pak`；
- 行 67：`sub_41B5C(a1, v16[0], a1+8)` — 注入角色头像控件表（Player_panda/tichy/kidd/drake/katharina.pcx × 6，`00041B5C_sub_41B5C.c:53-64`）；
- 行 69-87：`CreateFileW(data/gl.rec)` + 魔数校验（`Buffer == 19810929` = 0x01315A47），读 4 字节头（`byte_81194..81197`）。

### 控件表（证据）

`sub_33548`（`00033548_sub_33548.c`）注入 29 个按钮 PCX：
`button_focus/move/view/info/card/system/load/save/option/exit/return/bank/invest/troop/aimland/aimrole/aimcastle/capture/notcapture/assign/build/trade/sell/buy/pay/attack/explore/finish/base`（行 112-169）。

`sub_41B5C` 注入 6 个角色头像：`Target_head.pcx` + `Player_panda/tichy/kidd/drake/katharina.pcx`（行 53-64）。

### 事件 id 清单（`sub_41354`，`00041354_sub_41354.c`）

`sub_41354(a1, a2, a3)`：`a2=0`（帧首/事件完成）且 `a3==13`（**Enter/确定键**）时：

| `a3` 码 | 含义 | 行号 |
|---|---|---|
| `13` | 确定/Enter（触发菜单项选择） | 行 100 |
| `38` | 上一页（`v3[14]` 循环减 1，到 0 回 5） | 行 262-269 |
| `40` | 下一页（`v3[14]` 循环加 1，到 5 回 0） | 行 270-277 |
| `27` | 返回（`sub_3F4E4:112`、`sub_4085C:100` 均有 `a3==27 → sub_3DC04(v3[1], 6/8)` 返回场景） | 行 253（case 5 置 `a1[1]+12=1` 退出标志） |
| `37` | 无效操作（`sub_3F4E4:118`、`sub_4085C:109` 均有 `a3==37` 分支 → 页码调整，**推测** 对应 invalid.wav 音效） | 行 253（case 5） |

`switch (v3[14])`（`v3[14]` = 当前选中菜单项索引，范围 0..=5）：

| case | 行为 | 目标场景 | 行号 |
|---|---|---|---|
| `0` | `sub_3DC04(v3[1], 8)` — **新游戏**（进入角色选择） | `selectrole.Map`（`sub_3ECDC`） | 行 105-109 |
| `1` | `new 0x460` + `sub_4E60C` + 装载 `saveload.Map` + `ui.pak` | `saveload.map`（继续游戏） | 行 110-148 |
| `2` | `new 0x40C` + `sub_4B0B0` + 装载 `Option.map` + `ui.pak` | `Option.map`（选项） | 行 149-187 |
| `3` | `new 0x830` + `sub_3F8FC` + 装载 `help.map` + `ui.pak` | `help.map`（说明/帮助） | 行 188-221 |
| `4` | `new 0x830` + `sub_3F8FC` + 装载 `help.map` + `ui.pak`（第二页） | `help.map`（说明第二页，`word_7F128` 参数） | 行 222-252 |
| `5` | `*(a1[1]+12) = 1` — **退出标志** | 退出游戏 | 行 253-255 |
| `default` | `goto LABEL_49`（无效，释放临时字符串） | 无 | 行 256-257 |

> **推测**：`v3[14]` 的 0..=5 六个值与菜单项的对应关系（新游戏/继续/选项/说明1/说明2/退出）由 case 行为的资源字符串直接推断；菜单项的实际 UI 顺序需对照 `index.Map` 控件表确认。

## §b intro.Map 过场触发条件（`sub_42574`）

`sub_42574`（`00042574_sub_42574.c`）：
- 行 31：`v4 = &aIntroMap[9]` — 取 `intro.Map` 字符串长度；
- 行 54：`sub_37EDC(v19, v16, aUiPak)` — 从 `UI.pak` 取控件表；
- 行 58：`sub_37EDC(v20, v16, aIntroMap)` — 取 `intro.Map` 场景记录；
- 行 62-64：`operator new(0x3FC)` + `sub_43B8C` — 场景对象 ctor；
- 行 70：`sub_43D54` — 关联 UI 句柄；
- 行 73-80：`(**(scene)+4)(...)` 虚调用装载。

**触发条件**（证据 + 推测）：
- `main_loop.md` §4.1：`sub_42574` 是 intro.Map 装载器，与 `sub_40D80`（index.Map）并列；
- `main_loop.md` §4.3 case 4：`*(a1+716)=4` 时 `sub_34A08(*(a1+588))==0` 判定 → 场景完成态检查 → **推测** intro.Map 过场动画播放完成后切换到 index.Map；
- **推测**：游戏启动时先装载 intro.Map（片头动画），动画结束（`sub_34A08` 返回 0 = 未完成 → 继续播放；返回非 0 = 完成 → 切到 index.Map）；也可由 case 26/27 过场动画 phase 触发。

## §c 菜单项选择 → 场景切换映射

由 §a 的 `sub_41354` case 表直接得出（证据，行 105-255）：

| 菜单项（`v3[14]`） | case | 目标 | 装载函数 |
|---|---|---|---|
| 新游戏 | 0 | `selectrole.Map` | `sub_3ECDC`（`0003ECDC_sub_3ECDC.c:36`） |
| 继续游戏 | 1 | `saveload.map` | `sub_4E60C` + `sub_37EDC(saveload.Map)` |
| 选项 | 2 | `Option.map` | `sub_4B0B0` + `sub_37EDC(Option.map)` |
| 说明 | 3 | `help.map` | `sub_3F8FC` + `sub_37EDC(help.Map)`（第一页） |
| 说明（续） | 4 | `help.map` | `sub_3F8FC` + `sub_37EDC(help.Map)`（第二页，`word_7F128`） |
| 退出 | 5 | 退出游戏 | `*(a1[1]+12)=1`（退出标志，消息泵 `sub_62B84` 检测 `a1[40]`） |

> **推测**：「继续游戏」case 1 实际调 `sub_4E60C`（`00041354_sub_41354.c:121`）而非独立场景装载器——saveload.Map 在此被直接嵌入到 index 场景的子对象（`off_8119C`），而非切换到独立场景。这意味着「继续」是**子窗口叠加**而非场景切换。

## §d 音效触发

资源清单（`evidence/S3.2.log`）：`music\press.wav`、`music\invalid.wav`、`music\victory.wav`。

| 事件 | 音效 | 证据/推测 |
|---|---|---|
| 点击菜单项（`a3==13`，case 0..4 任一） | `press.wav` | **推测**：`sub_41354` 无直接 `press.wav` 字符串引用，音效由框架层（`sub_590A8` waveOut 播放队列）在事件分发时统一触发 |
| 无效操作（`a3==37`，`v3[14]` 越界/default 分支） | `invalid.wav` | **推测**：同上，`sub_3F4E4:118` 与 `sub_4085C:109` 的 `a3==37` 分支 |
| 战斗胜利（case 0 → vs.Map 战斗结束） | `victory.wav` | `main_loop.md` §4.3 case 25：战斗结束回主菜单 |

> 三个音效均为 **推测**：伪代码中无直接 `press.wav`/`invalid.wav` 字符串引用（grep 未命中），由 S3.2 资源清单与事件语义推断。

## §e 函数归属表

完整归属见 `ref/functions_menu.csv`（12 行，不改动 `ref/functions.csv` 原文件）。

| 函数 | 地址 | 归属理由 |
|---|---|---|
| `sub_40D80` | 0x40D80 | index.Map 主菜单装载器（§a） |
| `sub_42574` | 0x42574 | intro.Map 过场装载器（§b） |
| `sub_3ECDC` | 0x3ECDC | selectrole.Map 装载器（§c 新游戏） |
| `sub_402D8` | 0x402D8 | selectmap.Map 装载器（§c 继续/选图） |
| `sub_41354` | 0x41354 | index.Map 事件分发器（§a 事件表） |
| `sub_4085C` | 0x4085C | Option.map 事件处理器（§c 选项） |
| `sub_3F4E4` | 0x3F4E4 | Help 翻页事件处理器（§c 说明） |
| `sub_41B5C` | 0x41B5C | index.Map 角色头像控件注入 |
| `sub_33548` | 0x33548 | 通用按钮控件表注入（29 按钮） |
| `sub_29110` | 0x29110 | 场景管理器帧更新/切换总控 |
| `sub_3D430` | 0x3D430 | 全局单例 ctor |
| `sub_3D49C` | 0x3D49C | 全局单例 dtor |

---

## 推测点汇总

1. `v3[14]` 0..=5 与菜单项 UI 顺序的对应（新游戏/继续/选项/说明1/说明2/退出）— 由 case 行为资源字符串推断，需对照 index.Map 控件表确认。
2. 音效 `press.wav`/`invalid.wav` 的触发点 — 伪代码无直接引用，由事件语义 + S3.2 资源清单推断。
3. intro.Map 的精确触发条件（游戏启动自动播放 vs 用户触发）— 由 `sub_34A08` 完成态检查 + case 26/27 phase 机推断。
4. `a3==37` 是否对应 `invalid.wav`（还是其他无效操作音效）— 由分支语义推断。
