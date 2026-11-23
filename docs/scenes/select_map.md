# select_map 场景 · 详细图文拆解

> **单一事实源声明**：本文坐标/原语逐条抄自 `assets/data/scenes.json` 的 `select_map` 节点
> （竖屏 `primitives` 31 条、横屏 `landscape.primitives` 13 条，时点 2026-11-23 实测）；
> 叙事锚点抄自 `assets/data/ui_schema/scene_manifest_v3.json` 的 `select_map`；
> 行为/命中抄自 `crates/ti-app/src/scenes/select_map.rs` + `crates/ti-engine/src/bin/treasure_bevy.rs`；
> 截图为引擎真实渲染 `evidence/bevy_native_select_map.png`（1080×1920）与
> `evidence/bevy_native_select_map_1920x1080.png`（1920×1080），出图命令见 §2。
> 业务规则引用 `docs/business/01_game_logic_overview.md` §3 与 `docs/process/21_map_package_spec.md`，不复述。

## 1. 场景职责与状态机位置

`select_map`（选图场景，`Scene::SelectMap` / `SceneKind::SelectMap`）是 9 场景态图中
「选角 → 选图 → 主地图」链路的中继节点（`01_game_logic_overview.md` §1.2，唯一入口
`docs/business/07_ui_flow.md` §1）。

- **进入**：`SelectRole --确认角色--> SelectMap`；`Saveload --读档(缺地图)--> SelectRole
  → SelectMap`（`01_game_logic_overview.md` §1.2）。`Entering` 时 `map_slot=None`
  （选角阶段只设 character，`select_map.rs:66-75` 的 Confirm 兜底选第 0 张）。
- **退出**：
  - `SelectMap --Confirm(map_unlocked 校验)--> MainMap`：校验通过且角色+地图齐全 →
    `enter_main_map()`；数据校验失败（balance 缺条目/坐标越界，FIND-204）→ 停留原场景 + `Sfx::Invalid`。
  - `SelectMap --Back--> SelectRole`；点按命中下滑（`hit.swipe==2`）也走 Back（`on_hit`）。
- **职责**：Prev/Next 环形导航 3 槽（`MAP_COUNT = MapSlot::count() = 3`，QA139），
  未解锁图只置灰 + 锁块、Confirm 才拒绝（`Sfx::Invalid`，选择态不提交）；
  选图卡右上角显示 BOSS 难度星（`boss_tier_of` → tier+1 颗，1~3 星）。
- **态图不变式**：场景判别值 0..8 一场景一值；非法转换（如 `SelectRole → Vs`）
  类型层不存在（`scene_transition!` 宏编译期拦截，S10.2）。

> 注：业务层「全部 20 图」推进链（`01_game_logic_overview.md` §3）是跨图解锁语义；
> **本 UI 场景当前只呈现 3 个地图槽**（内嵌 m01/m02/m03），运行期可由地图包注入扩槽（§5/§10）。

## 2. 真实渲染截图

出图口径 = 引擎离屏截图（`12_bevy_usage.md` 离屏截图验收），视口经 `--viewport` 指定：

- 竖屏：`cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot select_map`
  → `evidence/bevy_native_select_map.png`（1080×1920）

![select_map 竖屏引擎渲染](../evidence/bevy_native_select_map.png)

- 横屏：`cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot --viewport=1920x1080 select_map`
  → `evidence/bevy_native_select_map_1920x1080.png`（1920×1080）

![select_map 横屏引擎渲染](../evidence/bevy_native_select_map_1920x1080.png)

**图中可见元素**：夜海背景 `bg_night_sea` + 左侧木纹拼贴面板（3 张错落海图
harbour / open_sea / reef）+ 右侧 3 张地图选项卡（地图图标 + 中文图名「m01/海图二/海图三」
+ 选中金边 + 未解锁锁条 + BOSS 难度星）+ 顶部金标题「选择地图」+ 底部操作提示「38/40 切换  13 确认」。

## 3. ASCII 布局图

**竖屏 1080×1920**（坐标 = scenes.json 实测）：

```
┌────────────────────────────────────────────┐ y=0
│        标题「选择地图」(352,48,400,96)      │  z8 金
│  ┌───────────────────────────┐             │
│  │ 面板木纹 (76,130,562,1190)│  ┌────────┐ │ y=480
│  │ ┌──────────────┐          │  │卡0 m01 │ │ x=720 w280 h320
│  │ │harbour(96,240│          │  │icon+名 │ │ pick_index=0
│  │ │ 400×280)     │          │  └────────┘ │
│  │ └──────────────┘          │  ┌────────┐ │ y=800
│  │    ┌──────────────┐       │  │卡1 海图二│ │ pick_index=1
│  │    │open_sea(156, │       │  └────────┘ │
│  │    │560,400×280)  │       │  ┌────────┐ │ y=1120
│  │    └──────────────┘       │  │卡2 海图三│ │ pick_index=2
│  │       ┌──────────────┐    │  └────────┘ │
│  │       │reef(216,880, │    │             │
│  │       │ 400×280)     │    │             │
│  │       └──────────────┘    │             │
│  └───────────────────────────┘             │ y=1320
│                                            │
│  提示行 (0,1648,1080,56)  z11              │ y=1648
└────────────────────────────────────────────┘ y=1920
```

**横屏 1920×1080**：

```
┌────────────────────────────────────────────────────────────────────┐ y=0
│                    标题「选择地图」(760,24,400,96) z8               │
│ ┌──────────────────────────────────────┐  ┌─────────────────────┐  │
│ │ 面板木纹 (20,140,1160,760)           │  │ bevy_ui 卡片列      │  │
│ │ ┌──────────┐┌──────────┐┌──────────┐│  │ (PanelSide::Right,  │  │
│ │ │harbour   ││open_sea  ││reef      ││  │  taffy 自动布局，   │  │
│ │ │40,200    ││420,200   ││800,200   ││  │  非 scenes.json 原语)│  │
│ │ │360×280   ││360×280   ││360×280   ││  │  ┌────────────────┐ │  │
│ │ └──────────┘└──────────┘└──────────┘│  │  │卡0/1/2         │ │  │
│ └──────────────────────────────────────┘  │  └────────────────┘ │  │
│                                            │  └─────────────────────┘  │
│  提示行 (0,1030,1920,50) z11                                          │ y=1030
└────────────────────────────────────────────────────────────────────┘ y=1080
```

## 4. 逐元素坐标表

### 4.1 竖屏 `select_map.primitives`（实测 31 条）

> 字段：# / kind / rect(x,y,w,h) / z / when / params(asset / text_key / pick_key / icon)；
> 省略字段取 JSON 空值。`ui_layer=bevy_ui` 表示 1080 原语跳过 CPU 绘制、由 bevy_ui 画（§7）。

**背景层（#1–2）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 1 | image | 0,0,1080,1920 | 3 | — | asset=`assets/ui/bg_night_sea.png` |
| 2 | rect | 76,130,562,1190 | 4 | — | color=`panel_wood` |

**地图拼贴层（#3–11）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 3 | image | 96,240,400,280 | 5 | — | asset=`map_harbour.png` |
| 4 | image | 156,560,400,280 | 5 | — | asset=`map_open_sea.png` |
| 5 | image | 216,880,400,280 | 5 | — | asset=`map_reef.png` |
| 6 | rect | 96,240,400,280 | 4 | — | color=`panel_wood`（#3 底垫） |
| 7 | rect | 96,240,400,2 | 5 | — | color=`gold_warm`（#3 金边） |
| 8 | rect | 156,560,400,280 | 4 | — | color=`panel_wood`（#4 底垫） |
| 9 | rect | 156,560,400,2 | 5 | — | color=`gold_warm`（#4 金边） |
| 10 | rect | 216,880,400,280 | 4 | — | color=`panel_wood`（#5 底垫） |
| 11 | rect | 216,880,400,2 | 5 | — | color=`gold_warm`（#5 金边） |

**选项卡列表层（#12–29，每卡 6 原语，右下 `PanelSide::Right`）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 12 | rect | 720,480,280,320 | 4 | — | pick_key=2, pick_index=0, ui_layer=bevy_ui（卡0 底） |
| 13 | rect | 720,480,280,320 | 9 | selected | item=0, pick_key=2, pick_index=0, ui_layer=bevy_ui（选中金边） |
| 14 | icon | 744,608,64,64 | 10 | normal | icon=`map`, size=64, item=0, ui_layer=bevy_ui |
| 15 | icon | 744,608,64,64 | 10 | selected | icon=`map`, size=64, color=black, item=0 |
| 16 | text | 808,608,168,44 | 10 | normal | text_key=`glmapm01.Map`, item=0 |
| 17 | text | 808,608,168,44 | 10 | selected | text_key=`glmapm01.Map`, color=black, item=0 |
| 18 | rect | 720,800,280,320 | 4 | — | pick_key=2, pick_index=1, ui_layer=bevy_ui（卡1 底） |
| 19 | rect | 720,800,280,320 | 9 | selected | item=1, pick_key=2, pick_index=1, ui_layer=bevy_ui |
| 20 | icon | 744,928,64,64 | 10 | normal | icon=`map`, size=64, item=1, ui_layer=bevy_ui |
| 21 | icon | 744,928,64,64 | 10 | selected | icon=`map`, size=64, color=black, item=1 |
| 22 | text | 808,928,168,44 | 10 | normal | text_key=`glmapm02.Map`, item=1 |
| 23 | text | 808,928,168,44 | 10 | selected | text_key=`glmapm02.Map`, color=black, item=1 |
| 24 | rect | 720,1120,280,320 | 4 | — | pick_key=2, pick_index=2, ui_layer=bevy_ui（卡2 底） |
| 25 | rect | 720,1120,280,320 | 9 | selected | item=2, pick_key=2, pick_index=2, ui_layer=bevy_ui |
| 26 | icon | 744,1248,64,64 | 10 | normal | icon=`map`, size=64, item=2, ui_layer=bevy_ui |
| 27 | icon | 744,1248,64,64 | 10 | selected | icon=`map`, size=64, color=black, item=2 |
| 28 | text | 808,1248,168,44 | 10 | normal | text_key=`glmapm03.Map`, item=2 |
| 29 | text | 808,1248,168,44 | 10 | selected | text_key=`glmapm03.Map`, color=black, item=2 |

**文本层（#30–31）**

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 30 | text | 352,48,400,96 | 8 | — | text_key=`scene.select_map.title` |
| 31 | text | 0,1648,1080,56 | 11 | — | text_key=`scene.select_map.nav_hint` |

### 4.2 横屏 `landscape.primitives`（实测 13 条）

| # | kind | rect (x,y,w,h) | z | when | params |
|---|---|---|---|---|---|
| 1 | image | 0,0,1920,1080 | 3 | — | asset=`assets/ui/bg_night_sea.png` |
| 2 | rect | 20,140,1160,760 | 4 | — | color=`panel_wood` |
| 3 | image | 40,200,360,280 | 5 | — | asset=`map_harbour.png` |
| 4 | image | 420,200,360,280 | 5 | — | asset=`map_open_sea.png` |
| 5 | image | 800,200,360,280 | 5 | — | asset=`map_reef.png` |
| 6 | rect | 40,200,360,280 | 4 | — | color=`panel_wood`（#3 底垫） |
| 7 | rect | 40,200,360,2 | 5 | — | color=`panel_wood`（#3 顶线） |
| 8 | rect | 420,200,360,280 | 4 | — | color=`panel_wood`（#4 底垫） |
| 9 | rect | 420,200,360,2 | 5 | — | color=`panel_wood`（#4 顶线） |
| 10 | rect | 800,200,360,280 | 4 | — | color=`panel_wood`（#5 底垫） |
| 11 | rect | 800,200,360,2 | 5 | — | color=`panel_wood`（#5 顶线） |
| 12 | text | 760,24,400,96 | 8 | — | text_key=`scene.select_map.title`, tier=body |
| 13 | text | 0,1030,1920,50 | 11 | — | text_key=`scene.select_map.nav_hint`, tier=hint |

## 5. 横屏差异

- **原语条数**：竖屏 31 → 横屏 13（少 18 条）。横屏 `landscape.primitives` **只含背景 +
  拼贴面板 + 标题/提示**，**没有 3 张选项卡的 18 条原语**（竖屏 #12–29）。
- **坐标重排**：木纹面板从竖向 `(76,130,562,1190)` 改横向 `(20,140,1160,760)`；
  3 张海图从竖屏错落纵排（x=96/156/216、y=240/560/880、各 400×280）改为横屏等高并排
  （x=40/420/800、y=200、各 360×280）；标题 x 从 352→760、提示行 y 从 1648→1030、h 56→50。
- **横屏卡片来源**：右侧卡列由 bevy_ui 按视口自动布局生成
  （`build_select_map_chrome` → `build_list_chrome_pos(..., PanelSide::Right)`，
  `treasure_bevy.rs:1428-1455`），taffy 随视口重排，故不出现在 landscape 原语表。
- **orientation 切换**：`--shot --viewport=WxH` 出图；世界层按 contain 缩放，chrome 层随视口重排
  （`17_layout_engine.md` B1 / `19_bevy_ui.md`）。landscape 节点专属参数 = 仅 `primitives`（无 title/list/locked/stars 等场景参数），
  后者由竖屏拓扑复用（横屏走 bevy_ui 默认几何）。

## 6. 资产接线

`scenes.json` `params.asset` 提取 → `assets/ui_manifest.json` 登记项（CS-1 资产单源）：

| scenes.json asset | manifest 项 | role | prompt / 说明 |
|---|---|---|---|
| `assets/ui/bg_night_sea.png` | `bg_night_sea` | background | 夜海背景，generated_at 2026-10-21 |
| `assets/ui/map_harbour.png` | `map_harbour` | mapcard | "a busy tropical harbour with ships and docks" |
| `assets/ui/map_open_sea.png` | `map_open_sea` | mapcard | "an open ocean with a lone island and a compass rose" |
| `assets/ui/map_reef.png` | `map_reef` | mapcard | "a narrow reef strait between two cliffs with sea monsters" |
| `icon=map`（图标） | `icon_map` → `assets/ui/icon_map.png` | icon | 折叠地图 monogram，`ui_icons.py` 白蒙版 gold tint |

- `text_key=glmapm0N.Map` → `assets/data/strings.json`（默认 m01 / 海图二 / 海图三）；
  **地图包注入时由包内 `strings.json` 覆盖**（如 m21/m22 = 海图一，m03 = m03，见 §10）。
- `text_key=scene.select_map.title` = 「选择地图」，`scene.select_map.nav_hint` = 「38/40 切换  13 确认」
  （`assets/data/strings.json:96-97`）。
- **守门**：`validate_data::check_asset_registry` 全类型 disk↔registry 双向同步；
  `export-icon-boundary` 禁令禁止把图标烘焙进底图（图标一律白蒙版 + 运行时 tint，S21.14）。

## 7. 命中契约

- **数据化命中**：竖屏卡 #12/#18/#24 的 `pick_key=2`，`pick_index=0/1/2`，`ui_layer=bevy_ui`。
  1080 原语标 `ui_layer` 跳过 CPU 绘制与命中；bevy_ui 侧 `build_select_map_chrome` 给每行
  `Node` 挂 `PickTarget { key: 2, index: idx }`（`attach_row_pick`，S29.3c 拆分；S31.3 根治「行鼠标点不动」）。
- **事件链**：`bevy_picking` 指针事件 → `PickPending` → `pick_flush` → `PickEvent{ key, index, swipe }`
  （`ti-engine/src/pick.rs`：`PointerClick`=tap，`PointerDragEnd.distance`=swipe）→ ti-app `route_hit`
  → `HitData` → `SelectMapScene::on_hit`。
- **on_hit 行为**（`select_map.rs:112-123`）：
  - `hit.key==2 && swipe==0`：`set_map_slot(from_index(index))` → `flash_press()` → `advance(Confirm)`
    （选中即确认，与键盘 Confirm 同语义）。
  - `hit.swipe==2`（下滑）：`advance(Back)` → 返回 SelectRole。
  - 未解锁地图点按：`Confirm` 分支判 `map_unlocked` → `Sfx::Invalid`，停留选图。
- **空白 swipe 兜底**：`PickActivity` 门控避免与实体事件双发（`pick.rs:11-13`）。

## 8. 业务规则

仅引用不复述（单一事实源）：

- 场景态图与进入/退出条件：`docs/business/01_game_logic_overview.md` §1.2（07_ui_flow §1）。
- 选图与跨图推进、解锁链单调、BOSS 难度星：`docs/business/01_game_logic_overview.md` §3
  （规则细节 = `docs/business/progression.md`）。
- 槽位口径：`MapSlot::count() = 3`（QA139，`MapSlot::all()` 单一事实源）；
  新席位 ≥2 人类才进选图（`docs/process/territory` + `app_seats_e2e`）。
- 地图包运行期换图（数据可注入、无注入=内嵌 3 槽零回归、选图包金边条）：
  `docs/process/21_map_package_spec.md` §7（S31.2b）；切图重进走 `MapBack → 选图 → enter_main_map_at(None)`
  全量重建 board/territory 会话。
- 版面坐标表与不变式：`docs/business/08_layout_design.md` §4.2（首块顶 ≥108、
  末块底 ≤1704、列表右缘 ≤1068）；视觉重心锚 `focus_anchor {76,230,562,1190}` ↔
  manifest `bbox_pct {7,12,52,62}`（gate `check_scene_narrative` N8 对账）。

## 9. 测试覆盖

| 测试文件 | 断言要点 |
|---|---|
| `crates/ti-app/tests/map_pack_select_e2e.rs` | 未注入包 = `map_pack_ids()/active_map_pack()` 为 `None`（0 行为变化）；切图重进重建 board 会话（起点一致 + 赛道在位）；非导航输入不改场景 |
| `crates/ti-app/tests/e2e_playthrough.rs` (`e2e_full_game_session`) | 阶段 1–3：Index→SelectRole→SelectMap；连续 3 次 Next = M01/M02/M03 |
| `crates/ti-app/tests/app_progression_wiring.rs` | `Next` 环形到 M20、解锁态提交、读档恢复 map_slot |
| `crates/ti-app/tests/app_seats_e2e.rs` | 2 席/带 AI 走真实 UI 路径进 SelectMap → MainMap |
| `crates/ti-app/tests/render_intent.rs` | select_map 标题 h=96、list.y=480、row_h/item_h=320、hint.y=1840（QB115/123） |

- **出图门禁**：`bevy-cpu-parity` 仅比游戏世界、UI 层排除（按机器断言 bevy_ui 节点数）；
  `--shot select_map` / `--shot --viewport=1920x1080 select_map` 双视口截图存在即 §2 验收。
- **叙事门禁**：`check_scene_narrative` N8 焦点锚对账；`doccheck2` 项号/计数等对账。

## 10. 数据驱动改法

**换这张 UI / 改坐标 / 换资产（零 Rust）**：

1. 改版面/坐标/文案：`assets/data/scenes.json` 的 `select_map.primitives`（或 `landscape.primitives`）
   逐条改 `rect`/`color`/`text_key`；改图名/提示 = `assets/data/strings.json` 对应键。
2. 换资产：替换 `assets/ui/{bg_night_sea,map_harbour,map_open_sea,map_reef,icon_map}.png`
   并同步 `assets/ui_manifest.json`（`ui_assets.py add/build/verify`，`13_ui_asset_pipeline.md`）。
3. 重出图验收：`--shot select_map` + `--shot --viewport=1920x1080 select_map`，用户视觉签收
   （A 纪律：`00_index.md` §UI 视觉改动）。
4. **零 Rust 证据**：以上步骤只改 `assets/**` + 数据，`git diff crates/` 为空即可（内容运营判据，`29_content_ops.md`）。

**换地图内容（不加图，仅改包数据）**：走 `scripts/map_pack.py` 的 `add → 填内容 → verify →
pack → install → check → sync → 出图`（`21_map_package_spec.md` §7 / `29_content_ops.md`），
包内 `strings.json` 可覆盖 `glmapm0N.Map` 文案；运行期 `TI_MAP=<id>` 切包或选图包金边条
（`select_map.rs:205-219`）。新增一张图需 `MapSlot` 扩槽 = 规则/代码改动，**不属零 Rust 范围**。
