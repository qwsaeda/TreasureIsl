# 角色选择（select_role）场景 · 详细图文拆解

> **数据源单一事实源声明**：本文全部数值/坐标来自实测（时点 2026-11-23）：
> - 版面数据：`assets/data/scenes.json` → `select_role` 节点（竖屏 `primitives` 45 条 + `landscape.primitives` 31 条 + `title/list/icon/hint/portrait_zones/hero_art/focus_anchor` 专属槽）。
> - 叙事规格：`assets/data/ui_schema/scene_manifest_v3.json` → `scenes.select_role`（`mood=hero_choice` / `focus.anchor=portrait_carousel` / `rhythm.rule=side_anchor_center_options`）。
> - 源码：`crates/ti-app/src/scenes/select_role.rs`（`SelectRoleScene`；S29.3 起右侧列表 chrome 迁 `bevy_ui`：`build_list_chrome` + `PickTarget`）。
> - 命中：`scenes.json` 原语 `pick_key=2` / `pick_index=0..4`；`crates/ti-engine/src/pick.rs`（`PickTarget` / `PickRect` / `HitData`）；壳 `route_hit` → `SelectRoleScene::on_hit`。
> - 资产登记：`assets/ui_manifest.json`（`role_*` / `bg_night_sea`，见 §6）。
> - 真实截图（引擎出图，验收口径）：`evidence/bevy_native_select_role_1920x1080.png`（横屏）、`evidence/bevy_native_select_role.png`（竖屏 1080×1920）。
> - 视觉规格：`docs/ui-redesign-v3/03_redesign_proposal.md` §3.2 + `docs/business/08_layout_design.md` §4.2/§4.0。
> 本文只引用文档 ID（F-xxx/EXP-xxx/Sxxx）不复述结论；坐标直接抄上述实测值，禁凭记忆。

## 1. 场景职责与状态机位置

- **职责**：选角 = 一次性完成两件事——①选「当前聚焦角色」（`host.character()`，决定左侧立绘/名牌）；②勾选「人类玩家席」（`host.player_seats()` / `seat_marks`，v3 热座，`territory.md` §1）。操作手段三套等价：方向键 Prev/Next 切角色、`MarkSeat` 勾选/取消、`Confirm` 开始对局；触屏点行（`on_hit`）同时完成「聚焦 + 勾选」。
- **状态机位置**：9 场景态图中的第 2 站（`Index → SelectRole → SelectMap → MainMap → Vs/…`）。`SceneKind::SelectRole`，实现类型 `SelectRoleScene`（`select_role.rs:16`）。
- **进入条件**：`Index` 菜单项 0（新游戏，含罗盘环 medallion 0）经 `GameInput::Confirm` → `host.enter(SelectRole)`；`Index` 侧 `reset_selection` 先清角色/地图（首进时 `character=None`，故 `Confirm` 有兜底选第 0 个角色，源码 `:88-96`）。读档后也可能停在 `SelectRole`（角色/地图缺失时，`fuzz_playtest.rs:497`）。
- **退出条件**（`on_input`，`select_role.rs:54-131`）：
  - `Back` / 下滑 swipe=2 → `SceneKind::Index`（返回）。
  - `Confirm | StartMatch`：兜底选角色 → 校验人类席数（`seat_selection_ready`，`min_players = balance.selection.min_players`，缺省 2；空 `seat_marks` = v2 旧单选流豁免下限）→ 若 `selection_done`（角色+地图齐全，`MainMap` 已选过图）则 `enter_main_map()`，否则 `enter(SelectMap)`；数据校验失败 `FIND-204` 留原场景 + `Sfx::Invalid`。
  - 未满足席数下限 → 留场景 + `Sfx::Invalid`（提示补选）。
- **与前后场景流转**：`Index →(Confirm) SelectRole →(Confirm) SelectMap →(Confirm) MainMap`；`SelectMap →(Back) SelectRole`；`SelectRole →(Back) Index`。非法转换 `SelectRole → Vs` 无 `SceneTransition` impl（编译期证明，`game_app_flow.rs:228-239`）。

## 2. 真实渲染截图

**横屏（1920×1080，引擎出图）**

![select_role 横屏真实渲染](../evidence/bevy_native_select_role_1920x1080.png)

**竖屏（1080×1920，引擎出图）**

![select_role 竖屏真实渲染](../evidence/bevy_native_select_role.png)

- 出图口径：`evidence/bevy_native_<scene>_<WxH>.png` = Bevy 原生离屏渲染（项目验收唯一图源，非 AI 概念图）；命令按 `docs/process/12_bevy_usage.md` 离屏截图约定（`--shot select_role` / `--viewport=WxH`），见 S29.5 多视口出图。
- 两文件大小/尺寸实测：横屏 8,296,178 B / 1920×1080 RGBA；竖屏 8,297,018 B / 1080×1920 RGBA。
- **图中可见元素**（对照 §4 坐标）：夜海渐变满屏底 `#0`；左侧木底立绘面板（竖屏 76,184,440,664；横屏 40,90,720,810）+ 当前角色 PNG 立绘 + 黄铜描边 + 名牌标题/简介；右侧 5 行角色选项（竖屏 x=600, y=480..1440，含小图标+名+简介；横屏见 §5 差异）；顶部金标题 `scene.select_role.title`；底部导航提示 `scene.select_role.nav_hint`（竖屏 y=1648 / 横屏 y=1042）。选中行黄铜高亮边框由 `chrome::selected_border` 绘制（Bevy 侧由 `build_list_chrome` 接管，见 §7）。

## 3. ASCII 布局图

**竖屏 1080×1920**（框内数字 = §4 原语序号段）

```
┌──────────────────────────────────────────────┐ y=0
│  bg_night_sea 满屏底 (#0)                     │
│        【标题 scene.select_role.title】       │ y=48  (#43)
│                                              │
│ ┌────────────────────┐                       │
│ │                    │   ┌────────────────┐  │ y=480
│ │  角色立绘 PNG      │   │ ▶ 行0 Panda    │  │   (#23-26)
│ │  role_*.png        │   │   行1 Tichy    │  │ y=672
│ │  (76,184,440,664)  │   │   行2 Kidd     │  │ y=864
│ │  #2-6 + 铜边#7-12  │   │   行3 Drake    │  │ y=1056
│ │                    │   │   行4 Katharina│  │ y=1248
│ └────────────────────┘   └────────────────┘  │ y=1440 底
│ ┌────────────────────┐                       │
│ │ 名牌 panel_wood #1 │                       │ y=860..1100
│ │ 名称/简介 #13-22   │                       │
│ └────────────────────┘                       │
│                                              │
│      【nav_hint #44】                         │ y=1648
└──────────────────────────────────────────────┘ y=1920
```

**横屏 1920×1080**

```
┌──────────────────────────────────────────────────────────────────┐ y=0
│ 【标题 #29 (40,16,720,64)】                                       │
│ ┌────────────────────────────────┐                               │
│ │                                │                               │
│ │   角色立绘 PNG (720×810)       │        （右侧列表区：          │
│ │   (40,90,720,810)              │         竖屏 5 行原语在横屏     │
│ │   #2-6 + 铜边 #7-18             │         landscape 节点未列，   │
│ │                                │         见 §5 差异说明）        │
│ │                                │                               │
│ └────────────────────────────────┘ y=900                        │
│ 【名称 #19/21/… (60,915)】【简介 #20/22/… (60,1000)】             │
│ 【nav_hint #30 (0,1042,1920,32)】                                 │ y=1042
└──────────────────────────────────────────────────────────────────┘ y=1080
```

## 4. 逐元素坐标表

### 4.1 竖屏 primitives（实测 45 条，`select_role.primitives`）

**背景层（1 条）**

| # | kind | rect (x,y,w,h) | z | when | params (asset/text/pick) |
|---|---|---|---|---|---|
| 0 | image | 0,0,1080,1920 | -13 | — | asset=`assets/ui/bg_night_sea.png` |

**内容层 · 左立绘锚（12 条：#1~#12）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 1 | rect | 76,860,440,240 | -12 | — | color=`panel_wood` |
| 2 | image | 76,184,440,664 | -11 | selected | item=0, asset=`role_panda.png` |
| 3 | image | 76,184,440,664 | -11 | selected | item=1, asset=`role_tichy.png` |
| 4 | image | 76,184,440,664 | -11 | selected | item=2, asset=`role_kidd.png` |
| 5 | image | 76,184,440,664 | -11 | selected | item=3, asset=`role_drake.png` |
| 6 | image | 76,184,440,664 | -11 | selected | item=4, asset=`role_katharina.png` |
| 7 | rect | 76,184,440,6 | -10 | — | color=`brass`（上边） |
| 8 | rect | 510,184,6,664 | -10 | — | color=`brass`（右边） |
| 9 | rect | 76,184,6,664 | -10 | — | color=`brass`（左边） |
| 10 | rect | 76,184,440,6 | -10 | — | color=`brass`（下边） |
| 11 | rect | 510,184,6,664 | -12 | — | color=`brass`（右边 deep） |
| 12 | rect | 510,184,6,664 | -11 | — | color=`brass`（右边） |

**内容层 · 名牌文字（10 条：#13~#22，5 角色 × 名称/简介）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 13 | text | 96,886,400,96 | -10 | selected | item=0, tier=title, color=`gold_warm`, key=`role.name.panda` |
| 14 | text | 96,990,400,56 | -10 | selected | item=0, tier=hint, color=`brass_light`, key=`role.desc.panda` |
| 15 | text | 96,886,400,96 | -10 | selected | item=1, tier=title, color=`gold_warm`, key=`role.name.tichy` |
| 16 | text | 96,990,400,56 | -10 | selected | item=1, tier=hint, color=`brass_light`, key=`role.desc.tichy` |
| 17 | text | 96,886,400,96 | -10 | selected | item=2, tier=title, color=`gold_warm`, key=`role.name.kidd` |
| 18 | text | 96,990,400,56 | -10 | selected | item=2, tier=hint, color=`brass_light`, key=`role.desc.kidd` |
| 19 | text | 96,886,400,96 | -10 | selected | item=3, tier=title, color=`gold_warm`, key=`role.name.drake` |
| 20 | text | 96,990,400,56 | -10 | selected | item=3, tier=hint, color=`brass_light`, key=`role.desc.drake` |
| 21 | text | 96,886,400,96 | -10 | selected | item=4, tier=title, color=`gold_warm`, key=`role.name.katharina` |
| 22 | text | 96,990,400,56 | -10 | selected | item=4, tier=hint, color=`brass_light`, key=`role.desc.katharina` |

**内容层 · 右侧 5 行选项（20 条：#23~#42，每行 4 原语；`ui_layer=bevy_ui`）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 23 | rect | 600,480,400,192 | -12 | — | panel_wood, pick_key=2, pick_index=0, ui_layer=bevy_ui |
| 24 | image | 620,504,96,144 | -10 | — | asset=`role_panda.png`, ui_layer=bevy_ui |
| 25 | text | 744,536,280,44 | -10 | — | item=0, tier=body, key=`role.name.panda`, ui_layer=bevy_ui |
| 26 | text | 744,592,280,44 | -10 | — | item=0, tier=hint, key=`role.desc.panda`, ui_layer=bevy_ui |
| 27 | rect | 600,672,400,192 | -12 | — | panel_wood, pick_key=2, pick_index=1, ui_layer=bevy_ui |
| 28 | image | 620,696,96,144 | -10 | — | asset=`role_tichy.png`, ui_layer=bevy_ui |
| 29 | text | 744,728,280,44 | -10 | — | item=1, tier=body, key=`role.name.tichy`, ui_layer=bevy_ui |
| 30 | text | 744,784,280,44 | -10 | — | item=1, tier=hint, key=`role.desc.tichy`, ui_layer=bevy_ui |
| 31 | rect | 600,864,400,192 | -12 | — | panel_wood, pick_key=2, pick_index=2, ui_layer=bevy_ui |
| 32 | image | 620,888,96,144 | -10 | — | asset=`role_kidd.png`, ui_layer=bevy_ui |
| 33 | text | 744,920,280,44 | -10 | — | item=2, tier=body, key=`role.name.kidd`, ui_layer=bevy_ui |
| 34 | text | 744,976,280,44 | -10 | — | item=2, tier=hint, key=`role.desc.kidd`, ui_layer=bevy_ui |
| 35 | rect | 600,1056,400,192 | -12 | — | panel_wood, pick_key=2, pick_index=3, ui_layer=bevy_ui |
| 36 | image | 620,1080,96,144 | -10 | — | asset=`role_drake.png`, ui_layer=bevy_ui |
| 37 | text | 744,1112,280,44 | -10 | — | item=3, tier=body, key=`role.name.drake`, ui_layer=bevy_ui |
| 38 | text | 744,1168,280,44 | -10 | — | item=3, tier=hint, key=`role.desc.drake`, ui_layer=bevy_ui |
| 39 | rect | 600,1248,400,192 | -12 | — | panel_wood, pick_key=2, pick_index=4, ui_layer=bevy_ui |
| 40 | image | 620,1272,96,144 | -10 | — | asset=`role_katharina.png`, ui_layer=bevy_ui |
| 41 | text | 744,1304,280,44 | -10 | — | item=4, tier=body, key=`role.name.katharina`, ui_layer=bevy_ui |
| 42 | text | 744,1360,280,44 | -10 | — | item=4, tier=hint, key=`role.desc.katharina`, ui_layer=bevy_ui |

**HUD 层（2 条：#43~#44）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 43 | text | 76,48,440,96 | -10 | — | tier=title, color=`gold_warm`, key=`scene.select_role.title` |
| 44 | text | 0,1648,1080,56 | -10 | — | tier=body, color=`text_on_parch`, key=`scene.select_role.nav_hint` |

**场景专属槽（非 primitives，来自 `select_role` 顶层键）**

| 槽 | 值（实测） | 说明 |
|---|---|---|
| title | x=296, y=48, w=440, h=96 | 顶部标题数据槽（S20.2：`x`=左立绘列中心点） |
| list | x=600, y=480, w=400, row_h=192, item_h=192, text_x=744, highlight_offset=0 | 右侧列表几何槽 |
| icon | x=584 | 行首 monogram 圆环槽（键留作数据槽） |
| hint | x=540, y=1840 | 底部提示锚点（原语 #44 实际落 y=1648） |
| portrait_zones | [{x:0,y:240,w:1080,h:1460}] | 立绘区 |
| hero_art | x=76, y=184, w=440, h=664 | 左立绘锚（S18.20 贴图 2:3 区） |
| focus_anchor | type=`portrait_carousel`, bbox={x:76,y:130,w:356,h:534} | v3 视觉重心（N8 门禁对账 manifest `focus.bbox_pct`） |

### 4.2 横屏 primitives（实测 31 条，`select_role.landscape.primitives`）

**背景层（1 条）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 0 | image | 0,0,1920,1080 | -13 | — | asset=`assets/ui/bg_night_sea.png` |

**内容层 · 立绘面板 + 铜边（18 条：#1~#18）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 1 | rect | 40,90,720,810 | -12 | — | color=`panel_wood` |
| 2 | image | 40,90,720,810 | -11 | selected | item=0, asset=`role_panda.png` |
| 3 | image | 40,90,720,810 | -11 | selected | item=1, asset=`role_tichy.png` |
| 4 | image | 40,90,720,810 | -11 | selected | item=2, asset=`role_kidd.png` |
| 5 | image | 40,90,720,810 | -11 | selected | item=3, asset=`role_drake.png` |
| 6 | image | 40,90,720,810 | -11 | selected | item=4, asset=`role_katharina.png` |
| 7 | rect | 40,90,720,6 | -10 | — | color=`brass` |
| 8 | rect | 40,90,720,6 | -10 | — | color=`brass` |
| 9 | rect | 40,90,720,6 | -10 | — | color=`brass` |
| 10 | rect | 40,90,720,6 | -10 | — | color=`brass` |
| 11 | rect | 40,90,720,6 | -12 | — | color=`brass` |
| 12 | rect | 40,90,720,6 | -11 | — | color=`brass` |
| 13 | rect | 758,90,6,810 | -10 | — | color=`brass`（右边） |
| 14 | rect | 758,90,6,810 | -10 | — | color=`brass`（右边） |
| 15 | rect | 758,90,6,810 | -12 | — | color=`brass`（右边 deep） |
| 16 | rect | 758,90,6,810 | -11 | — | color=`brass`（右边） |
| 17 | rect | 40,90,6,810 | -10 | — | color=`brass`（左边） |
| 18 | rect | 40,90,720,6 | -10 | — | color=`brass`（下边） |

**内容层 · 名牌文字（10 条：#19~#28，5 角色 × 名称/简介）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 19 | text | 60,915,720,80 | -10 | selected | item=0, tier=title, key=`role.name.panda` |
| 20 | text | 60,1000,720,50 | -10 | selected | item=0, tier=hint, key=`role.desc.panda` |
| 21 | text | 60,915,720,80 | -10 | selected | item=1, tier=title, key=`role.name.tichy` |
| 22 | text | 60,1000,720,50 | -10 | selected | item=1, tier=hint, key=`role.desc.tichy` |
| 23 | text | 60,915,720,80 | -10 | selected | item=2, tier=title, key=`role.name.kidd` |
| 24 | text | 60,1000,720,50 | -10 | selected | item=2, tier=hint, key=`role.desc.kidd` |
| 25 | text | 60,915,720,80 | -10 | selected | item=3, tier=title, key=`role.name.drake` |
| 26 | text | 60,1000,720,50 | -10 | selected | item=3, tier=hint, key=`role.desc.drake` |
| 27 | text | 60,915,720,80 | -10 | selected | item=4, tier=title, key=`role.name.katharina` |
| 28 | text | 60,1000,720,50 | -10 | selected | item=4, tier=hint, key=`role.desc.katharina` |

**HUD 层（2 条：#29~#30）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 29 | text | 40,16,720,64 | -10 | — | tier=title, color=`gold_warm`, key=`scene.select_role.title` |
| 30 | text | 0,1042,1920,32 | -10 | — | tier=body, color=`text_on_parch`, key=`scene.select_role.nav_hint` |

> 逐条数对账：竖屏 1+12+10+20+2 = **45**；横屏 1+18+10+2 = **31**。表内每条与 `scenes.json` 实测一一对应，无「等」省略。

## 5. 横屏差异

- **原语数量**：竖屏 45 → 横屏 31（少 14 条）。竖屏包含右侧 5 行选项的 20 条原语（rect+image+text×2 ×5），横屏 `landscape` 节点**不含这组 5 行原语**（实测仅有背景 1 + 立绘面板/铜边 18 + 名牌文字 10 + HUD 2 = 31）。横屏下右侧选项列不在这份 `landscape.primitives` 清单内；其渲染走 S29.3 bevy_ui 列表路径（`build_list_chrome` 按视口算行位置，见 §7/§10），故数据是「不手写像素」的意图槽而非逐行 rect。
- **立绘锚尺寸**：竖屏 `hero_art` / 立绘原语 = 440×664（2:3 区，x=76,y=184）；横屏立绘面板/贴图放大到 **720×810**（x=40,y=90，S63.2 横屏立绘规格）。横屏 720×810 是**版面 rect**（PNG 源实测仍是 384×576，见 §6），由 blit 缩放铺入。
- **铜边条数/布局**：竖屏 6 条 brass（#7~#12，上下各 1 + 左右各 2）；横屏 12 条（#7~#18，上边重复 6 + 右边 4 + 左 1 + 下 1），均为重复/深描边变体。
- **名牌文字位置**：竖屏名称 y=886 / 简介 y=990（宽 400）；横屏名称 y=915 / 简介 y=1000（宽 720）。两者 `when=selected` + `item=0..4` 语义一致。
- **标题/提示**：竖屏标题 (76,48,440,96) + nav_hint (0,1648,1080,56)；横屏标题 (40,16,720,64) + nav_hint (0,1042,1920,32)。横屏标题左对齐立绘列顶，竖屏标题居中于立绘列。
- **orientation 切换逻辑**：`scenes.json` 同节点下 `primitives`（竖屏基准 1080×1920）与 `landscape.primitives`（横屏 1920×1080）两套并存；壳按视口宽高比选表。S29.5 实测 `--shot --viewport=WxH` 多视口出图：世界 contain 缩放 + chrome 随视口重排，节点数不变（25 节点，真重排）。`scene_manifest_v3` 的 `focus.bbox_pct` 以竖屏 1080×1920 为基准换算（N8 门禁 `check_scene_narrative` 对账）。
- **landscape 专属参数**：实测 `select_role.landscape` 仅含 `primitives` 一个键，无独立 title/list/icon/hint/portrait_zones/hero_art/focus_anchor 槽；这些槽沿用顶层值。

## 6. 资产接线

`scenes.json` 原语 `params.asset` 提取 → `assets/ui_manifest.json` 登记项（实测）：

| 资产路径 | 引用原语（竖屏/横屏） | ui_manifest 登记 | 实测 PNG 尺寸 |
|---|---|---|---|
| `assets/ui/bg_night_sea.png` | #0 / #0 | name=`bg_night_sea`（line 161-163） | 满屏底图 |
| `assets/ui/role_panda.png` | #2,#24 / #2 | name=`role_panda`, role=`portrait`（line 301-303） | 384×576 |
| `assets/ui/role_tichy.png` | #3,#28 / #3 | name=`role_tichy`, role=`portrait`（line 310-312） | 384×576 |
| `assets/ui/role_kidd.png` | #4,#32 / #4 | name=`role_kidd`, role=`portrait`（line 319-321） | 384×576 |
| `assets/ui/role_drake.png` | #5,#36 / #5 | name=`role_drake`, role=`portrait`（line 328-330） | 384×576 |
| `assets/ui/role_katharina.png` | #6,#40 / #6 | name=`role_katharina`, role=`portrait`（line 337-339） | 384×576 |

- 5 张角色立绘均为 384×576（≈2:3），竖屏铺 440×664、横屏铺 720×810（S63.2 横屏立绘 rect），由 blit 缩放；`role_*` 的 `ui_manifest.params.what` 存角色外观描述（生成提示词来源）。
- **守门**：gate `validate_data::check_asset_registry` 做 disk↔registry 双向同步（新增/漏登即 FAIL）；gate `check_hardcode` / doccheck2 `engine-no-color-literal` 守「视觉量不硬编码」；资产引用集由 `scripts/apk_audit.py --refset` 单源打包（未引用 0 进包，S35）。新增立绘须同时改 scenes.json `params.asset` + ui_manifest 登记 + 跑 sync，否则门禁非零退出。
- 概念图 `evidence/ui_concept/` 仅设计参考，非本场景验收图；验收图以 §2 引擎出图为准。

## 7. 命中契约

- **数据化命中**：竖屏右侧 5 行 rect 原语（#23/27/31/35/39）带 `pick_key=2`、`pick_index=0..4`、`ui_layer=bevy_ui`。`scene_render::spawn_pick_layer` 从原语组装 `PickTarget{key:2,index}` / `PickRect`（`ti-engine/src/pick.rs:28-38,120-140`）。
- **派发链**：触屏点行 → 壳 `manual_1080_hit_system` / bevy_picking → `PickEvent` → `PollHit(HitData{key,index,swipe})` → `GameApp::route_hit` → `SelectRoleScene::on_hit`（`select_role.rs:137-153`）。
- **on_hit 行为**：
  - `swipe=2`（下滑）→ `host.advance(GameInput::Back)` → 回 `Index`。
  - `swipe=0 && key=2` → `index` 转 `Character::from_index(i)` → `set_character` + `flash_press()` + `toggle_seat_mark`（勾选/取消该角色为人类席），返回 false（不切场景；勾选与「开始对局」分离，D5 修复）。
  - 其余 key/swipe → 不消费。
- **键盘等价路径**：`Prev/Next` 切 `host.character()`；`MarkSeat` 对当前角色调 `toggle_seat_mark`（源码 `:76-83`）——与触屏行点击等价（D1 修复：v2 键盘流选不出多人）。点行/按键都只「勾选」，真正开局靠 `Confirm`（`seat_selection_ready` 校验 `min_players`）。
- **Bevy 层归属**：`ui_layer=bevy_ui` 的行 rect 由 `build_list_chrome` 渲染为 bevy_ui 节点并挂 `PickTarget`（`Pickable::IGNORE` 给非行节点，避免挡命中，见 S29.3b）；CPU 路径 `render()` 在 `chrome_in_bevy_ui()` 为真时提前 return，跳过木底/选中边绘制（源码 `:167-169`），Bevy 侧负责视觉与拾取。

## 8. 业务规则

只引用不复述（单一事实源）：

- **席位与角色**：`docs/business/territory.md` §1（v3：2..5 人类热座 + 可配 AI 席；-selection 规则 = `rules::selection`）；角色集 `Character::all()` 5 角色，单一事实源 `Character::count()`（`select_role.rs:13`）。`balance.selection.min_players` = 2（缺省），席位下限在「开始对局」时校验，勾选阶段允许清空（D5）。
- **勾选语义**：`selection::seat_mark_toggle`（可取消到 0 席）；`seat_selection_ready`（`select_role.rs:40-47`）。空 `seat_marks` = v2 兼容（1 人类 + 4 AI，豁免下限）。
- **推进/通关**：`docs/business/progression.md`（选图→主地图环岛赛道）；选角自身不改数值，仅写 `selection.character` / `player_seats`。
- **存档**：存档 v4 写入所选角色与席位（`territory.md` 存档段）；读档后角色/地图缺失回 `SelectRole`。
- **版面规格**：`docs/business/08_layout_design.md` §4.2（选择类坐标表）+ §4.0（三屏 v4 面板语言：深棕木底 + 黄铜细边 + 3px 黄铜选中边）。坐标/槽值以 scenes.json 实测为准，业务文档只做锚点说明。
- **场景叙事**：`scene_manifest_v3.json` `select_role`（mood=`hero_choice`、focus=`portrait_carousel`、rhythm=`side_anchor_center_options`、info_density.center_max_per_row=5、permanent_hud_count_max=0）；`docs/ui-redesign-v3/03_redesign_proposal.md` §3.2（左 1/3 立绘锚 + 右侧窄列，重心从列表转立绘）。

## 9. 测试覆盖

**e2e / 流程测试（`crates/ti-app/tests/`，grep `select_role`/`SelectRole`/`选角` 实测）**

| 文件 | 断言要点 |
|---|---|
| `app_seats_e2e.rs` | 真实 UI 路径 `Index→SelectRole(Next+MarkSeat×N)→Confirm→SelectMap→MainMap`；2 人类席 `seat_marks.len()==2`；2+3AI；旧单选流豁免；取消勾选后 `seat_marks` 空（`app_seats_e2e.rs:21-75,239-271`） |
| `game_app_flow.rs` | 菜单→选角→选图→主地图全链路；选角 Back 回 Index；重进选角角色复位为 None；非法转换 `SelectRole→Vs` 编译期无 impl（`:17,203-239,416-433`） |
| `e2e_playthrough.rs` | 新游戏应进 SelectRole；选角 Back→Index；选图 Back→SelectRole；点卡拾取 `pick_key=2` 为 select_role/select_map 列表行（`:35,204-217,297-321,562-570`） |
| `e2e_playability.rs` | 单玩家（0 MarkSeat）与 2 玩家（MarkSeat×2）进主地图（`:32-54`） |
| `app_adversarial_e2e.rs` | 真实 UI 路径 MarkSeat×2 对抗面（`:27-31`） |
| `fuzz_playtest.rs` | `Scene::SelectRole` 字符串/深度编码；角色+地图缺失 → scene=SelectRole（`:63,79,497`） |
| `render_parity.rs` / `render_parity_hd.rs` / `render_parity_gpu.rs` | `render_select_role()` 帧帧间互异（Index≠SelectRole≠SelectMap）；GPU/CPU 对拍（`:50-133` / `:45-123` / `:40-41`） |
| `render_invariants.rs` | `("select_role","select_role")` 场景渲染不变量锚（`:29`） |
| `e2e_ui_flow_deterministic.rs` / `stability_repeat.rs` / `e2e_playthrough.rs` | `index→select_role→select_map→main_map` 稳定/确定性路径（`:10-22` / `:5-35`） |
| `transition_freeze_fix.rs` | SelectRole 终帧与 Index 帧互异（`:36-66`） |
| 其它导航型 | `app_trade_panel_e2e.rs` / `app_merchant_panel_e2e.rs` / `settle_once_no_chain.rs` / `app_territory_turns.rs` / `main_map_pick_e2e.rs` 均以 `Confirm: Index→SelectRole` 起手 |

**门禁覆盖**：gate `bevy-cpu-parity` 对 select_role 做 CPU↔Bevy 游戏世界像素对拍（UI 层排除口径，S29.4）；`check_scene_narrative`（gate 序号注释 21）对 `scene_manifest_v3.select_role` 做 N1~N8 判据 + focus bbox 对账；`--shot select_role --viewport=WxH` 出图（§2）为视觉验收硬证据；doccheck 校验 i18n key（`role.name.*`/`role.desc.*`/`scene.select_role.title`/`scene.select_role.nav_hint`/`scene.select_role.seat_hint`）无孤儿。

## 10. 数据驱动改法

「换这张 UI / 改坐标 / 换资产」的零 Rust 流程（证据：`git diff crates/` = 空）：

1. **改坐标/版面**：编辑 `assets/data/scenes.json` 的 `select_role.primitives`（逐条 rect/when/item）或顶层槽 `title/list/hero_art/focus_anchor/prompt`；横屏改 `landscape.primitives`。坐标直接抄实测原值（本文 §4 表即现状基线）。
2. **换立绘/底图资产**：替换 `assets/ui/role_*.png` / `bg_night_sea.png`，并在 `assets/ui_manifest.json` 同步登记项；若换文件名，同步改 scenes.json 的 `params.asset`。生成走 `scripts/ui_asset_batch.py`（规格 `assets/ui_batches/*.json`，原图落 `evidence/gen_tmp/`），禁手写像素。
3. **改文案**：i18n key 不改 Rust；改语言表对应 key（`role.name.panda` 等 5×2 + `scene.select_role.title` / `.nav_hint` / `.seat_hint`），doccheck 会校验 key 存在与无孤儿。
4. **改叙事重心**：编辑 `assets/data/ui_schema/scene_manifest_v3.json` 的 `select_role`（mood/focus.bbox_pct/rhythm/info_density）；须同步满足 `check_scene_narrative` N8（focus bbox 与 `hero_art_bbox` 对账）；间距/密度违反即 gate FAIL。
5. **出图验收**：重建（scenes.json 经 `include_str!` 内嵌，改后须 rebuild）→ `--shot select_role --viewport=1920x1080` 与 `--viewport=1080x1920` 出新截图 → 用户视觉签收（A 纪律）→ 跑 `gate.ps1` 全绿。
6. **零 Rust 证据**：以上步骤只动 `assets/data/*.json`、`assets/ui/*.png`、`assets/ui_manifest.json`、语言表；`git diff -- crates/` 输出为空即证明未改 Rust（内容运营核心判据，见 `docs/process/29_content_ops.md`）。
