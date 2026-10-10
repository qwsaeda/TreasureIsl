# 13 — 参考附录

> 单一事实源：本文件汇总所有全局常量、事件码、场景码、错误类型、文件格式和资源清单。
> 关联文档：`docs/design/constants.md`（常量表）、`docs/formats/lucencypackage.md`（容器格式）。

---

## 1. 全局常量总表

> 权威数值表见 `docs/design/constants.md`（§1 显示 / §2 布局 / §3 经济 / §4 战斗 / §10 RNG / §9 存档 / §11 容器 / §13 现代化）；本节仅列 Rust 侧常量名映射，数值以 constants.md 为单一事实源。

### 1.1 显示与分辨率（数值见 constants.md §1）

| 常量 | 说明 |
|---|---|
| `LOGICAL_W` | 逻辑宽 |
| `LOGICAL_H` | 逻辑高 |
| `FB_PIXELS` | 帧缓冲像素数 |
| `PIXEL_FORMAT` | RGB565 16 位色 |
| `GLYPH_ADVANCE` / `GLYPH_W` / `GLYPH_H` | 字形步进与尺寸 |
| `INK_COLOR` | 字形墨色 |

### 1.2 布局常量（数值见 constants.md §2）

`MENU_TOP` / `ROW_H` / `STATUS_X` / `STATUS_Y`（主菜单壳布局）。

### 1.3 经济常量（数值见 constants.md §3）

`MAX_GOLD`（金量上限）/ `MAX_RESOURCE`（资源上限）/ `MAX_BUILD_LEVEL`（建筑等级上限）。

### 1.4 战斗常量（数值见 constants.md §4）

`BOSS_HP_0..4`（各档 HP）/ `BOSS_ATK` / `DAMAGE_K`（Knuth 乘子）/ `BATTLE_MAX_TURNS`。

### 1.5 RNG 常量（数值见 constants.md §10）

`LCG_A` / `LCG_C` / `LCG_MASK` / `LCG_MAX`（MSVC LCG 推测算法，F-320）。

### 1.6 存档常量（数值见 constants.md §9）

`REC_MAGIC`（**仅 gl.rec / gl*.sav，settings.dat 例外，见 F-304**）/ `SAVE_VERSION_V1` / `SAVE_VERSION_V2` / `MIN_SAVE_SIZE_V1` / `MIN_SAVE_SIZE_V2` / `SLOT_COUNT`。

### 1.7 容器常量（清单见 constants.md §11.1）

`CONTAINER_UI` / `CONTAINER_GLPICS` / `CONTAINER_CARDS` / `CONTAINER_GOODS` / `CONTAINER_MUSIC`。

### 1.8 现代化常量（数值见 constants.md §13 + `docs/modernization.md`）

`SCALE_2X/3X/4X` / `DPI_BASE` / `BREAKPOINT_MOBILE` / `BREAKPOINT_DESKTOP` / `SFX_MAX_CHANNELS` / `TTF_FONT_SIZE_MIN/MAX`。

---

## 2. 事件码总表

### 2.1 全局输入事件码

| 事件码 | 常量名 | 语义 | 典型操作 |
|---|---|---|---|
| 0 | `EVENT_NONE` | 无触发 | — |
| 1 | `EVENT_BATTLE`（**触发事件码**，§7.2） | 进战斗 | — |
| 2 | `EVENT_BATTLE_END` | 战斗结束 | — |
| 3 | `EVENT_MENU` | 回菜单 | — |
| 4 | `EVENT_CUTSCENE` | 触发过场 | — |
| 5 | `EVENT_CUTSCENE_DONE` | 过场完成 | — |
| 6 | `EVENT_INVALID` | 无效操作 | 无效点击 |
| 7 | — | 未分配 | — |
| 8 | `EVENT_NEW_GAME` | 新游戏 | 菜单项 0 |
| 9 | — | 未分配 | — |
| 10 | `EVENT_HELP_RETURN` | help 返回 | — |
| 11 | — | 未分配 | — |
| 12 | `EVENT_SCENE_12` | 场景 12 | — |
| 13 | `EVENT_CONFIRM` | 确定/Enter | 菜单确认 |
| 14 | — | 未分配 | — |
| 15 | — | 未分配 | — |
| 16 | — | 未分配 | — |
| 17 | — | 未分配 | — |
| 18 | — | 未分配 | — |
| 19 | — | 未分配 | — |
| 20 | — | 未分配 | — |
| 21 | `EVENT_LANDMARK_DONE` | 据点结算完成 | 贸易前置 |
| 22 | `EVENT_BUY` | 买入 | 贸易确认 |
| 23 | `EVENT_SELL` | 卖出 | 贸易确认 |
| 24 | — | 未分配 | — |
| 25 | `EVENT_BATTLE_STATE` | 战斗态 | — |
| 26 | `EVENT_CUTSCENE_26` | 过场动画 phase 26 | — |
| 27 | `EVENT_CUTSCENE_27` | 过场动画 phase 27 | — |

### 2.2 翻页事件码

| 事件码 | 常量名 | 语义 |
|---|---|---|
| 37 | `EVENT_INVALID` | 无效操作（constants §7.3） |
| 38 | `EVENT_PREV_PAGE` | 上一页/上一个 |
| 39 | `EVENT_PAGE_CONFIRM` | 翻页确认 |
| 40 | `EVENT_NEXT_PAGE` | 下一页/下一个 |

### 2.3 主地图事件分派

> 主地图分派码（§7.5）与 §2.1 触发事件码（§7.2）是两套独立体系，同名数值（如 1）语义不同：
> 触发码 1 = 进战斗，分派码 1 = Msgl 据点。

| 事件码 | 目标场景 | 说明 |
|---|---|---|
| 1 | Msgl（据点） | 据点事件 |
| 2 | Msgm（面板） | 面板事件 |
| 3 | 无（卡牌随机数） | 卡牌抽取 |
| 4 | Msgl（货物结果） | 货物事件 |
| 5 | Msgm（卡片翻转） | 卡片事件 |
| 6 | Vs（战斗） | 进入战斗 |
| 7 | Msgm（消息） | 消息事件 |
| 8 | Msgs/Msgm（退出/返回） | 退出 |

### 2.4 场景队列事件码

| 来源 | 写入的 id 集合 |
|---|---|
| `主地图帧更新` | {1,2,3,4,5,11,12,13,17,18,19,20,21,24,25,26,27} |
| `场景管理器帧更新总控` | {6,7,8,9,10,14,15,16,22,23} |

### 2.5 事件 id 值域

```
event_id ∈ 1..27（哨兵 27）
```

### 2.6 事件重映射

```
{4→8, 13→17, 14→18, 15→19}
```

---

## 3. 场景码总表

### 3.1 场景枚举

| 场景码 | 常量名 | 语义 |
|---|---|---|
| 0 | `SCENE_INDEX` | 主菜单态 |
| 1 | `SCENE_INTRO` | 片头 |
| 2 | `SCENE_UNLOADING` | 场景卸载中 |
| 3 | `SCENE_BACK_MENU` | 回主菜单 |
| 4 | `SCENE_MAIN_MAP` | 主地图/探索 |
| 5 | `SCENE_VS` | 战斗 |
| 6 | `SCENE_INDEX_MAP` | 回 index.Map |
| 7 | `SCENE_SELECT_ROLE` | 角色选择 |
| 8 | `SCENE_SELECT_MAP` | 地图选择 |
| 9 | `SCENE_MESSAGE` | 消息/帮助 |
| 10 | `SCENE_HELP_RETURN` | help 返回 |
| 11 | `SCENE_SAVELOAD` | 存读档 |
| 12 | `SCENE_EXIT` | 退出 |

### 3.2 场景状态码

| 状态码 | 语义 |
|---|---|
| 0 | 主菜单态 |
| 2 | 场景卸载中 |
| 3 | 回主菜单 |
| 4 | 过场/坐标切换 |
| 25 | 战斗态 |
| 26 | 过场动画 phase 26 |
| 27 | 过场动画 phase 27 |

### 3.3 场景跳转规则

| 当前场景 | 触发条件 | 目标场景 |
|---|---|---|
| Index | 菜单项 0（新游戏） | SelectRole |
| SelectRole | 角色选定 | SelectMap |
| SelectMap | 地图选定（completed） | MainMap |
| MainMap | 事件码 6 | Vs |
| MainMap | 事件码 1/4 | Message (Msgl) |
| MainMap | 事件码 2/5/7/8 | Message (Msgm/Msgs) |
| Vs | 战斗结束 | Index |
| Message | 关闭 | MainMap（场景源） |
| 任意 | 返回（事件码 27） | Index |

---

## 4. 错误类型总表

### 4.1 存档错误

| 错误 | 触发条件 | 用户提示 |
|---|---|---|
| `SaveErr::TooShort` | 字节数不足 | 「存档损坏」 |
| `SaveErr::BadMagic` | 魔数不符 | 「存档损坏」 |
| `SaveErr::BadVersion` | version > 2 | 「版本不兼容」 |
| `SaveErr::Io` | 文件系统错误 | 「读写失败」 |

### 4.2 贸易错误

| 错误 | 触发条件 |
|---|---|
| `TradeErr::InsufficientGold` | 金量不足 |
| `TradeErr::UnknownGood` | 商品不在价格表 |
| `TradeErr::NoStock` | 持有列表无匹配 |

### 4.3 建造错误

| 错误 | 触发条件 |
|---|---|
| `BuildErr::Insufficient` | 金量不足（`bank.gold < cost`） |
| `BuildErr::InvalidLevel` | 等级超出有效范围 |

### 4.4 战斗

战斗 oracle（`oracle_battle_round` / `oracle_full_battle`）无独立 `BattleErr` 枚举：
`tier` 越界时 `oracle_boss_hp` 取末档（不报错），战斗结果以 `Option<Side>` / `Side` 表达。

### 4.5 音频错误

| 错误 | 触发条件 |
|---|---|
| `AudioErr::DecodeError` | 解码失败 |
| `AudioErr::PlaybackError` | 播放失败 |
| `AudioErr::ResourceNotFound` | 资源未找到 |

---

## 5. 文件格式总表

### 5.1 存档文件

| 文件 | 格式 | 最小大小 | 魔数 |
|---|---|---|---|
| `settings.dat` | 定长 | 8 B | 无（实测 8 字节 `5D 4A 71 4A 2E 01 01 01`，F-304） |
| `gl.rec` | 定长 | 10 B | `0x01315A47` |
| `gl*.sav` (v1) | 定长 | 18 B | `0x01315A47` |
| `gl*.sav` (v2) | 定长 | 26 B | `0x01315A47` |

> 魔数 `0x01315A47` 仅 `gl.rec` / `gl*.sav`；`settings.dat` 无魔数（F-304 实测）。

### 5.2 容器文件

| 格式 | 磁盘名 | 说明 |
|---|---|---|
| `LucencyPackage` | `*.pak` | UI/图形精灵容器 |
| `LucencyPackage` | `*.lpk` | 卡牌/货物/音乐容器 |

### 5.3 场景文件

| 格式 | 磁盘名 | 说明 |
|---|---|---|
| `.Map` | `index.Map` | 主菜单场景 |
| `.Map` | `help.Map` | 帮助场景 |
| `.Map` | `selectrole.Map` | 角色选择 |
| `.Map` | `selectmap.Map` | 地图选择 |
| `.Map` | `main.Map` | 主地图 |
| `.Map` | `glmapm01.Map` ~ `glmapm03.Map` | 地形图 |
| `.Map` | `vs.Map` | 战斗场景 |
| `.Map` | `trade.Map` | 贸易场景 |
| `.Map` | `build.Map` | 建造场景 |
| `.Map` | `card.Map` | 卡牌场景 |
| `.Map` | `msgs.Map` | 消息场景 |
| `.Map` | `saveload.Map` | 存读档场景 |

### 5.4 资源文件

| 格式 | 磁盘名 | 说明 |
|---|---|---|
| PCX | `*.pcx` | 精灵图 |
| FNT | `GoldLand.Fnt` | 点阵字体 |
| DAT | `font.dat` | 字体数据 |
| WAV | `*.wav` | 音效 |
| OGG | `*.ogg` | BGM |
| INI | `filenum.INI` | 精灵索引 |
| JSON | `manifest.json` | PWA 清单 |

### 5.5 价格表

5 种商品固定价格 20/76/95/114/133，buy_key 202/402/602/802/112、sell_key 102/302/502/702/902（来源 `docs/design/economy.md` §1.1，`资源查表` 查表哈希键）。

### 5.6 建造成本表

```
cost = base_cost × (level + 1)       level ∈ [0, 10]
```

base_cost 见 `docs/design/constants.md` §3.2（Castle 1000 / Investment 500 / Bank 200 / Fort 800）。

### 5.7 银行步进器阶梯

按压 1/2/3/4/5 次 = 10/100/1000/10000/100000，6..10 保持当前（`docs/design/constants.md` §3.3）。

### 5.8 投资奖励阶梯

兵种 A..E 奖励 +30,000/+40,000/+20,000/+15,000/+50,000（`投资`，`docs/design/constants.md` §3.4）。

### 5.9 BOSS HP 速查

`oracle_boss_hp(tier)`：0→50,000 / 1→100,000 / 2→150,000 / 3→200,000 / 4→300,000 / _→0（`docs/design/constants.md` §4.2）。

### 5.10 方向枚举

| 方向码 | 枚举 | Δ(row, col) | 方向位 |
|---|---|---|---|
| 0 | North | (-1, 0) | 8 |
| 1 | West | (0, -1) | 4 |
| 2 | South | (+1, 0) | 2 |
| 3 | East | (0, +1) | 1 |

### 5.11 可通行判定

```
可通行 = (cell.flags & 3) == 1
```

### 5.12 据点触发概率

```
命中 = (rand() % 100 + 1) > 40         // 40% 命中率（来源 docs/design/map_system.md §3.3，constants.md §6.6；exe 实测 `trade` 行 154）
```

### 5.13 等距投影

```
iso_x = 32 × (row + col)
iso_y = 16 × (row + grid_w - 1 - col)
```

### 5.14 cell 索引

```
cell = row × grid_w + col
```

### 5.15 地图网格

| 地图 | 网格 | cell 数 | 连线数 | 缺格 |
|---|---|---|---|---|
| glmapm01 | 20 × 20 | 399 | 75 | cell(0,19) |
| glmapm02 | 20 × 20 | 399 | 125 | cell(0,19) |
| glmapm03 | 15 × 15 | 224 | 23 | cell(0,14) |

### 5.16 消息界面常量

| 种类 | 总页数 |
|---|---|
| Msgs | 1 |
| Msgl | 1 |
| Msgm | 1 |
| Help | 3 |
| Info | 1 |

### 5.17 对话框对象大小

| 族 | 大小（字节） | vtable |
|---|---|---|
| 对话框族（msgl/msgm/msgs） | 1,552 | `off_6FDFC` |
| Help 族 | 2,096 | `off_6F47C` |
| Info 族 | 1,164 | `off_6F4FC` |

### 5.18 文本槽位

| 槽位 | 语义 |
|---|---|
| `a1+255` | 行模式（1=单行，2=双行） |
| `a1+256` | 翻页标志（0/1） |
| `a1+257` | 总页数 - 1 |
| `a1+259` | 条目索引 |
| `a1+260` | 文本缓冲起点 |
| `a1+522` | 翻页按钮位图 |

---

## 6. 资源清单

### 6.1 首屏资源（308 KB）

| 类别 | 文件 | 字节 |
|---|---|---|
| 字体 | GoldLand.Fnt + font.dat + settings.dat | 26 KB |
| 精灵 | ui.pak + glpics.pak | 185 KB |
| 场景布局 | 8 个 .Map | 31 KB |
| 地形 | glmap_m01.pak + glmapm01.Map | 55 KB |

### 6.2 非首屏资源（250 KB）

| 类别 | 文件 | 字节 |
|---|---|---|
| 地形 | glmap_m02.pak + glmapm02/m03.Map | 57 KB |
| 卡牌 | cards.lpk | 33 KB |
| 货物 | goods.lpk | 0.5 KB |
| 音乐 | music.lpk | 66 KB |
| 场景 | 其余 18 场景 .Map | 47 KB |

### 6.3 音频资源

| 文件 | 类型 | 用途 |
|---|---|---|
| `music\press.wav` | WAV | 按键音 |
| `music\invalid.wav` | WAV | 错误音 |
| `music\victory.wav` | WAV | 胜利音 |
| `music\glmusic.ogg` | OGG | BGM |

### 6.4 现代化资源

| 文件 | 类型 | 用途 |
|---|---|---|
| `NotoSansCJK-Regular.ttf` | TTF | 矢量字体 |
| `manifest.json` | JSON | PWA 清单 |
| `sw.js` | JS | Service Worker |
| `icon-192.png` | PNG | PWA 图标 |
| `icon-512.png` | PNG | PWA 图标 |

---

## 7. 卡牌常量

### 7.1 效果码

| 效果码 | 枚举 | 行为 |
|---|---|---|
| 1 | `Damage(n)` | `hp -= n` |
| 2 | `Heal(n)` | `hp += n` |
| 3 | `Gold(n)` | `gold += n` |
| 4 | `Move` | 不改 state |
| 0 | `Unknown` | 无效果 |

### 7.2 占位卡牌目录

| 卡牌 ID | 效果 | 数值 |
|---|---|---|
| 1 | Damage | 10 |
| 2 | Heal | 15 |
| 3 | Gold | 20 |
| 4 | Move | — |
| 5 | Unknown | — |

### 7.3 事件生成常量

| 常量 | 值 | 说明 |
|---|---|---|
| event_id 值域 | 1..25 | `rand() % 25 + 1` |
| 重映射 | {4→8, 13→17, 14→18, 15→19} | 事件 id 修正 |
| 卡牌库存门控 | `>= 10` | 进入卡牌场景条件 |

---

## 8. 实现约束速查

| 约束 | 说明 |
|---|---|
| 不引 `rand` crate | 自实现 LCG |
| 不占位 | `TODO/unwrap/expect/panic!` 0 命中 |
| RNG 单一源 | 全走 `GameRng` |
| 战斗伤害独立 | K 常数不统一 |
| 跨平台一致 | native/wasm/android 序列逐位一致 |
