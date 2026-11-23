# saveload（存档/读档）场景 · 详细图文拆解

> **单一事实源声明（时点 2026-11-23，实测）**：坐标/原语 = `assets/data/scenes.json` 的 `saveload` 节点（13 竖屏原语 + `landscape.primitives` 8 原语 + 5 槽位）；叙事规格 = `assets/data/ui_schema/scene_manifest_v3.json` 的 `saveload`；文案 = `assets/data/strings.json`；视图源码 = `crates/ti-app/src/scenes/saveload.rs` + `crates/ti-engine/src/ui.rs`（`build_list_chrome_pos`）+ `crates/ti-engine/src/bin/treasure_bevy.rs`（bevy_ui 装配）；命中 = `crates/ti-app/src/scenes/mod.rs::route_hit`；存档结构 = `crates/ti-core/src/domain/persistence.rs`（`SaveSlot` / `SaveSlotV4Ext`）；业务 = `docs/business/persistence.md`；截图 = `evidence/bevy_native_saveload_1920x1080.png`（1920×1080）+ `evidence/bevy_native_saveload.png`（1080×1920），均为引擎 `--shot` 离屏真实渲染图。本文只引用不复述规则 ID（R-persistence-*/EXP 编号见原文）。

## 1. 场景职责与状态机位置

**职责**：存读档中枢——列出 5 个存档槽（位 1~5），每槽显示有档/空态 + 摘要（角色/金币/RNG 步数）；点有档槽 = 读档，点空槽 = 写档，下滑或返回 = 退回主菜单。

**Scene 变体与路由**：`SceneKind::Saveload`（`crates/ti-app/src/scenes/mod.rs` 第 79 行，编号 7）；入口 `SaveloadScene` 实现 `Scene` trait，`on_input` / `on_hit` / `render` 三条由 `route_input` / `route_hit` / `route_render` 的 `SceneKind::Saveload` 分支派发（mod.rs 第 1057/1076/1092 行）。

**在 9 场景态图中的位置**（业务唯一事实源 `docs/business/07_ui_flow.md` §1，合法转换表在 mod.rs 第 180~235 行）：

```
Intro → Index → { SelectRole / Saveload / Message / Exit }
Saveload → Index（SaveloadClose/Back）
Saveload → MainMap（读档恢复：角色 + 地图齐全）
Saveload → SelectRole（读档恢复：角色/地图缺失）
```

- 进入：`Index` 菜单项 1「继续/存读档」→ `GameInput::Confirm` → `SceneKind::Saveload`（`commands_test.rs::dispatch_saveload_write_and_close` 实测）。
- 退出：`SaveloadClose | Back` → `host.enter(SceneKind::Index)`；读档成功后由宿主按角色/地图是否齐全转 `MainMap` 或 `SelectRole`。
- 非法转换（如 `MainMap → Saveload`）不生成 `SceneTransition` impl，编译期即拒绝。

## 2. 真实渲染截图

![saveload 横屏](../evidence/bevy_native_saveload_1920x1080.png)

- 出图：`cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot saveload --viewport=1920x1080`，视口 1920×1080（S38 方案 A：横屏场景走 `landscape.primitives` 1920×1080 坐标）。
- 可见元素：满屏夜海星空底（`bg_night_sea.png`）；左侧 bevy_ui 面板（3% 贴边）5 行槽位，**位 1 行黄铜高亮**（`BRASS_SELECT`）并显示「位 1 有 / 角色 1 · 0 金 · 0 回合」；右侧 5 块深棕木底 meta 面板（世界层原语，x=1680）；顶部金色「存读档」标题；底部提示行。

![saveload 竖屏](../evidence/bevy_native_saveload.png)

- 出图：`cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot saveload`，视口 1080×1920（默认）。
- 可见元素：同一夜海底；左上 bevy_ui 5 槽面板（位 1 选中黄铜底）；中上金色「存读档」标题（数据原语 x=354）；右侧一整条深棕木色竖带（x=760 的 5 块 meta 面板）；底部「上下选位 13 读/写」提示（数据原语 y=1648）。
- 验收口径：引擎离屏真实渲染（GPU 读回 PNG），非 AI 概念图；出图探针日志 `node/text/computed_layout` 证明 bevy_ui taffy 自动布局真跑。

## 3. ASCII 布局图

**竖屏 1080×1920**（图示按 1080 逻辑坐标；实际 bevy_ui 面板用 `%` 定位 ≈ left 3% / width 34%）

```
(0,0) ┌─────────────────────────────────────────────────┐ (1080,0)
      │  bg_night_sea 满屏夜空                           │
      │        ┌──────── 存读档 (354,48,640,96) ───────┐│
      │ ┌──────┴──┐                              ┌────┐│
      │ │存读档    │                              │meta││
      │ │▓▓位1 有▓▓│ (bevy 选中=黄铜底)            │    ││
      │ │位2 有    │ (280,240,440,224 槽1 原语)    │x760││
      │ │位3 有    │                              │160 ││
      │ │位4 有    │                              │×5  ││
      │ │位5 有    │                              │块  ││
      │ └─────────┘                              └────┘│
      │                                                │
      │                                                │
      │      上下选位 13 读/写 (0,1648,1080,56)         │
      └─────────────────────────────────────────────────┘ (1080,1920)
```

**横屏 1920×1080**

```
(0,0) ┌────────────────────────────────────────────────────────────────┐ (1920,0)
      │ bg_night_sea 满屏                                              │
      │ ┌────────────┐         存读档 (760,24,400,96)      ┌───────────┐│
      │ │存读档      │                                    │meta 1680, ││
      │ │▓▓位1 有▓▓  │  ← bevy_ui 左侧列表（taffy 自动重排）│125,180,150││
      │ │位2 有      │                                    │  ×5 竖排  ││
      │ │位3 有      │                                    └───────────┘│
      │ │位4 有      │                                                 │
      │ │位5 有      │                                                 │
      │ └────────────┘                                                 │
      │  上下选位 13 读/写 (0,1030,1920,50)                             │
      └────────────────────────────────────────────────────────────────┘ (1920,1080)
```

## 4. 逐元素坐标表

### 4.1 竖屏原语（13 条，scenes.json `saveload.primitives` 逐条；# = 数据数组下标）

| # | 层 | kind | rect (x,y,w,h) | z | when | params / text |
|---|---|---|---|---|---|---|
| 0 | 背景 | image | 0,0,1080,1920 | 3 | "" | asset=`assets/ui/bg_night_sea.png` |
| 1 | 槽位 | rect | 280,240,440,224 | 4 | "" | color=panel_wood, pick_key=3, pick_index=0, ui_layer=bevy_ui（槽 1） |
| 2 | 元数据 | rect | 760,752,160,224 | 4 | "" | color=panel_wood（meta 面板，无 pick） |
| 3 | 元数据 | rect | 760,240,160,224 | 4 | "" | color=panel_wood（meta 面板，无 pick） |
| 4 | 槽位 | rect | 280,1008,440,224 | 4 | "" | color=panel_wood, pick_key=3, pick_index=3, ui_layer=bevy_ui（槽 4） |
| 5 | 槽位 | rect | 280,496,440,224 | 4 | "" | color=panel_wood, pick_key=3, pick_index=1, ui_layer=bevy_ui（槽 2） |
| 6 | 元数据 | rect | 760,1008,160,224 | 4 | "" | color=panel_wood（meta 面板，无 pick） |
| 7 | 元数据 | rect | 760,496,160,224 | 4 | "" | color=panel_wood（meta 面板，无 pick） |
| 8 | 槽位 | rect | 280,1264,440,224 | 4 | "" | color=panel_wood, pick_key=3, pick_index=4, ui_layer=bevy_ui（槽 5） |
| 9 | 槽位 | rect | 280,752,440,224 | 4 | "" | color=panel_wood, pick_key=3, pick_index=2, ui_layer=bevy_ui（槽 3） |
| 10 | 元数据 | rect | 760,1264,160,224 | 4 | "" | color=panel_wood（meta 面板，无 pick） |
| 11 | HUD | text | 354,48,640,96 | 8 | "" | color=gold_warm, tier=body, text_key=`scene.saveload.title` |
| 12 | HUD | text | 0,1648,1080,56 | 11 | "" | color=text_on_parch, tier=hint, text_key=`scene.saveload.nav_hint` |

> 绘制顺序 = `render_primitives_filtered` 按 `z` 升序；数组下标 ≠ 绘制顺序。`ui_layer=bevy_ui` 的 5 条槽位原语在 Bevy 路径被跳过（chrome.rs 第 1311 行）改由 bevy_ui 绘制，避免双画；CPU 后端照旧走数据原语。

**场景专属布局字段**：`title {x600,y48,w640,h96}`；`list {x280,y240,w440,row_h256,item_h224,text_x312,gold_x792,highlight_offset8,summary_dy52}`；`hint {x540,y1840}`；`portrait_zones [{0,240,1080,1450}]`；`focus_anchor {type:slot_list_with_meta, bbox:{356,230,637,1210}}`。`render_intent.rs::saveload_layout_matches_doc` 锚定 title.h=96 / list.y=240 / row_h=256 / item_h=224 / gold_x=792 / hint.y=1840。

### 4.2 横屏原语（8 条，scenes.json `saveload.landscape.primitives` 逐条）

| # | 层 | kind | rect (x,y,w,h) | z | when | params / text |
|---|---|---|---|---|---|---|
| 0 | 背景 | image | 0,0,1920,1080 | 3 | "" | asset=`assets/ui/bg_night_sea.png` |
| 1 | 元数据 | rect | 1680,125,180,150 | 4 | "" | color=panel_wood |
| 2 | 元数据 | rect | 1680,295,180,150 | 4 | "" | color=panel_wood |
| 3 | 元数据 | rect | 1680,465,180,150 | 4 | "" | color=panel_wood |
| 4 | 元数据 | rect | 1680,635,180,150 | 4 | "" | color=panel_wood |
| 5 | 元数据 | rect | 1680,805,180,150 | 4 | "" | color=panel_wood |
| 6 | HUD | text | 760,24,400,96 | 8 | "" | gold_warm, tier=body, params.text_key=`scene.saveload.title` |
| 7 | HUD | text | 0,1030,1920,50 | 11 | "" | text_on_parch, tier=hint, params.text_key=`scene.saveload.nav_hint` |

> 横屏 5 块 meta 面板无 `pick_key`/`ui_layer` → 世界层装饰；交互槽列表仍由 bevy_ui 在左侧绘制（`PanelSide::Left`）。`landscape_drawlist.rs::saveload_landscape_world_layer` 实测 slot1 世界层面板 (1680,125,180,150)。

### 4.3 选中态（bevy_ui 黄铜底）↔ SaveSlot 字段的对应（5 槽位）

这是本场景最关键的一张映射：屏幕上的「高亮那一行」不是孤立的视觉态，而是 `save_slots()[cursor]` 的投影。

| 屏幕槽位 | cursor / pick_index | bevy_ui 选中底色 | 数据源（host） | 行文案来源（SaveSlot 字段） |
|---|---|---|---|---|
| 位 1 | 0 | `ChromeRow.selected = (0 == cursor)` → `BRASS_SELECT` | `save_slots()[0]` | version>0→「有」+ `slot_summary_fmt(character,gold,rng_steps)`；version==0→「空」+ `empty_line` |
| 位 2 | 1 | `1 == cursor` → 黄铜底，否则透明 | `save_slots()[1]` | 同上 |
| 位 3 | 2 | `2 == cursor` → 黄铜底，否则透明 | `save_slots()[2]` | 同上 |
| 位 4 | 3 | `3 == cursor` → 黄铜底，否则透明 | `save_slots()[3]` | 同上 |
| 位 5 | 4 | `4 == cursor` → 黄铜底，否则透明 | `save_slots()[4]` | 同上 |

链路细节：

1. `treasure_bevy.rs::build_saveload_chrome`（第 1400~1424 行）遍历 `host.save_slots()`，逐槽生成 `ChromeRow { image: None, line1: "位 N 有/空", line2: 摘要, selected: i == cursor }`；`cursor = host.saveload_cursor()`。
2. `ti_engine::ui::build_chrome_row`（ui.rs 第 367~377 行）把 `selected` 转成 `BackgroundColor`：`palette::BRASS_SELECT`（黄铜高亮，截图中的橙黄条）或 `TRANSPARENT`；行高/位置由 taffy flex 算，不写像素。
3. `attach_row_pick(commands, &chrome, 3)` 给每行 `Node` 挂 `PickTarget` key=3、`index` = 行序号 = 槽号，于是「点哪行」= 「选中哪个 SaveSlot」。
4. 点选先 `set_saveload_cursor(slot)`（S15.6 QB136 三态：落光标瞬间黄铜底移到该行），再按 `s.version > 0` 决定 `LoadSave` / `WriteSave`——即黄铜底所指行 = 即将读/写的那个存档槽。
5. CPU 路径（未迁 bevy_ui 的后端）等价视觉是 3px 黄铜描边 `chrome::selected_border`（saveload.rs 第 119~123 行），选中槽号同样是 `cursor`。

**SaveSlot 数据结构（v4 扩展位）**：`crates/ti-core/src/domain/persistence.rs` 的 `SaveSlot { version, character, gold, map_slot, position, rng_steps, unlocked_mask, boat_level, v4 }`；`v4: Option<Box<SaveSlotV4Ext>>` 承载 `seats: Vec<SeatSave>` + `territories: Vec<TerritorySave>` + `current_seat` + `achievements`（席位全量/领地图/热座/成就，见 `territory.md` §8）。行内可见的「角色/金币/回合」直接读该槽的 `character`/`gold`/`rng_steps`；`有/空` 读 `version > 0`。写档时 `saveload.rs::on_input(WriteSave)` 用 `WRITE_VERSION = 3`（S21.11 带 `boat_level`）组装 `SaveSlot`，并把 `host.save_v4_ext()` 挂进 `v4` 位；业务侧完整迁移链 v1→v2→v3→v4→v5 见 `docs/business/persistence.md`（当前 JSON schema = `SaveSlotV5`）。

## 5. 横屏差异

| 维度 | 竖屏（1080×1920） | 横屏（1920×1080） |
|---|---|---|
| 原语条数 | 13（`primitives`） | 8（`landscape.primitives`） |
| 背景 | image 0,0,1080,1920 | image 0,0,1920,1080（同资产） |
| meta 面板 | 5 块 160×224 竖排，x=760，y=240/496/752/1008/1264 | 5 块 180×150 竖排，x=1680，y=125/295/465/635/805 |
| 槽位数据原语 | 5 条 rect 带 `pick_key=3`+`ui_layer=bevy_ui`（280,240 等） | 无槽位数据原语（交互列表全交给 bevy_ui） |
| 标题 | text (354,48,640,96) | text (760,24,400,96)，参数写在 `params` 内 |
| 提示 | text (0,1648,1080,56) | text (0,1030,1920,50)，参数写在 `params` 内 |

**切换逻辑**：`chrome::render_primitives_filtered` 读 `host.shell().settings.orientation == Landscape` → `layout.prims_for(true)` 取横屏原语表（chrome.rs 第 1293~1296 行）；`--shot --viewport=1920x1080` 下 S39 修复保证方向判定早于坐标渲染（treasure_bevy.rs 第 990 行）。bevy_ui 侧不区分横竖原语——`build_list_chrome_pos(..., PanelSide::Left)` 的 `%` 定位让 5 槽列表随视口自动重排（S29.5 实测 540×960→1080×1920 节点数不变）。竖屏数据原语保持 0 行为变化（`landscape_drawlist.rs` 竖屏对照断言 (760,240,160,224)）。

叙事层（`scene_manifest_v3.json` saveload）：`mood=archive`；`focus.anchor=slot_list_with_meta`，`bbox_pct {x:33,y:12,w:59,h:63}`；rhythm 四拍 slot_list_with_meta(0.55)/hero_art(0.2)/meta(0.15)/hint(0.1)，`rule=three_column_left_hero`；`info_density {center_max_per_row:5, edge_min_per_row:1, dead_zone_max_pct:15}`；`permanent_hud_count_max=1`；几何约束 `focus_bbox_must_contain:[list]`、`no_rect_overlap_with:[focus_anchor]`、`bottom_hint_y_min_pct:92`。

## 6. 资产接线

| 数据层资产路径 | 用途 | ui_manifest 登记 | 守门 |
|---|---|---|---|
| `assets/ui/bg_night_sea.png` | 场景满屏背景（竖屏 1080×1920 / 横屏 1920×1080，同一资产） | `ui_manifest.json` 第 161~163 行 `name=bg_night_sea` | `validate_data::check_asset_registry` 双向同步；`export-icon-boundary` 禁把代码矢量当资产 |
| `assets/ui/bg_night_sea_top.png` | 同系列顶部云层（登记在册，非本场景原语引用） | `ui_manifest.json` 第 373~375 行 | 同上 |

- 资产从 `scenes.json` 的 `params.asset` 提取，源码不硬编码路径：`saveload.rs` 只调用 `chrome::render_primitives(host, "saveload", cursor)`，由 `ti-core::data` 解析 `SCENES_JSON`（`include_str!` 内嵌）。
- 本场景无图标/PickTarget 缩略图资产（bevy `ChromeRow.image = None`），无独立 icon 资产接线。
- 文字全部走 `strings.json` i18n 键：`scene.saveload.title`（存读档）、`scene.saveload.nav_hint`（上下选位 13 读/写）、`slot.label_fmt`（位 {0} {1}）、`slot.has/empty`（有/空）、`scene.saveload.slot_summary_fmt`（角色 {0} · {1} 金 · {2} 回合）、`scene.saveload.empty_line`（空位 · 可写入）、`hud.gold_fmt`（金 {0}）。

## 7. 命中契约

- **数据化命中**：竖屏 `scenes.json` 5 条槽位原语带 `pick_key=3` + `pick_index=0..4`（`ui_layer=bevy_ui`）；横屏交互由 bevy_ui 行的 `PickTarget` key=3 提供（`attach_row_pick(commands, &chrome, 3)`），世界层 meta 面板不可点。
- **链路**：指针点击 → `ti-engine::pick` 产生 `HitData {key,index,swipe}`（`game_app/mod.rs` 第 240 行；`swipe=0` 点按 / `1..4` 方向）→ `scenes::route_hit(SceneKind::Saveload)`（mod.rs 第 1076 行）→ `SaveloadScene::on_hit`。
- **on_hit 行为**（saveload.rs 第 70~89 行）：
  - `swipe == 2`（下滑）→ `SaveloadClose`；
  - `swipe != 0` 或 `key != 3` → 忽略；
  - `slot = max(hit.index, 0)`；`has = save_slots[slot].version > 0`；
  - `chrome::flash_press()` 按压闪影 + `set_saveload_cursor(slot)` 落黄铜底 → `has ? LoadSave(slot) : WriteSave(slot)`。
- **on_input 行为**：`LoadSave` → `host.restore_save(data)` + `Sfx::Press`；`WriteSave` → 组装 `SaveSlot`（`character=index()+1`、`map_slot=zero_index`、`position=map_pos().index(grid_w)`、`rng_steps=host.rng_steps()` 精确续流、`unlocked_mask`、`boat_level`、`v4`）写槽 + `Sfx::Press`，不切场景（返回 false 语义见 commands_test）；`SaveloadClose | Back` → `enter(Index)`。
- **键盘/手柄**：上下键移动光标（`selection.saveload` 槽位光标变量由 `GameInput::MenuUp/MenuDown` 推进，见 `game_app/input.rs`），`13` 读/写、`Esc` 退出（提示行文案）。

## 8. 业务规则

只引用不复述（单一事实源）：

- `docs/business/persistence.md`：存档 v5 schema（`SaveSlotV5`，JSON 人可读/可迁移/可复现）；5 槽来自 `balance.save.slot_count`（回退 5）；迁移链 v1→v2→v3（+`boat_level`）→v4（+seats/territories/current_seat）→v5（+人手/武器）；`unlocked_mask: u32` 落盘口径（勘误：规则层 `Vec<String>` 为镜像）；`rng_seed=DEFAULT_RNG_SEED=42` + `rng_steps` 续流；顶层 `hp` v5 退役（R20 Q008/Q015）。可测规则 R-persistence-1~6。
- `docs/business/territory.md` §8：席位/领地图字段（`SeatSave`/`TerritorySave`，5 席 = 主玩家 + 4 AI）。
- `docs/business/progression.md`：跨图解锁位与存档一致口径。
- `docs/business/07_ui_flow.md` §1：场景流转（Saveload→Index/MainMap/SelectRole）。
- 关键常量：`WRITE_VERSION=3`（saveload.rs 第 17 行）、`REC_MAGIC=0x0131_5A47`（persistence.rs）、`PANEL_SELECTED_BORDER_PX`（CPU 路径选中描边）。
- 数值锚点示例（截图 5 槽均为测试夹具预置有档）：「角色 1 · 0 金 · 0 回合」= `SaveSlot{character:1,gold:0,rng_steps:0,version>0}`。

## 9. 测试覆盖

| 测试文件 | 断言要点 |
|---|---|
| `crates/ti-app/tests/commands_test.rs::dispatch_saveload_write_and_close` | 菜单项 1 进 Saveload；`WriteSave(0)` 不切场景；`SaveloadClose` 回 Index |
| `crates/ti-app/tests/render_intent.rs::saveload_layout_matches_doc` | title.h=96 / list.y=240 / row_h=256 / item_h=224 / gold_x=792 / hint.y=1840 锚定 scenes.json |
| `crates/ti-app/tests/landscape_drawlist.rs::saveload_landscape_world_layer` | 横屏 bg 满屏 1920×1080 + meta 面板 (1680,125,180,150)；竖屏 bg 1080×1920 + (760,240,160,224) 0 行为变化 |
| `crates/ti-app/tests/e2e_render_alignment.rs` | `SCENES` 含 `"saveload"`，1080 世界层渲染对齐 |
| `crates/ti-app/tests/e2e_playability.rs` | 7 场景存在性 + 3 场景 chrome 循环含 saveload；`save_slots()[0]` 读回 |
| `crates/ti-app/tests/render_invariants.rs` | `("saveload","saveload")` 渲染不变量 |
| `crates/ti-app/tests/saveload_v4_roundtrip.rs` | 5 席 + 领地图 + 当前席写档→新会话 `restore_save` 往返；version 3/4 槽兼容 |
| `crates/ti-app/tests/fuzz_playtest.rs` | 写档→`save_slots()` 读回→`restore_save`→二次写回稳定字段比对 |
| `crates/ti-app/tests/app_seats_e2e.rs` / `app_progression_wiring.rs` / `e2e_playthrough.rs` | `SaveSlot.v4` 席位出局保持 / 解锁链 / `boat_level` 写档 |
| `crates/ti-core/tests/rules_saveload_json.rs` + `tests/save_migration.rs` | R-persistence-1~6：往返逐字段一致、v1→v2、幂等、同 seed 同流、rng_seed 续读、设置不影响玩法；v2→v3/v4→v5 迁移默认值 |
| gate | `doccheck` 场景叙事断言（`check_scene_narrative` 锚 `scene_manifest_v3.json`）；`bevy-cpu-parity` 只比游戏世界（UI 层排除）；`--shot saveload` 双视口出图门禁 |

## 10. 数据驱动改法

**换这张 UI / 改坐标 / 调槽位（零 Rust）**：

1. 改 `assets/data/scenes.json` 的 `saveload`：调整 `primitives[]` / `landscape.primitives[]` 的 rect/z/color、场景字段 `title`/`list`/`hint`/`focus_anchor`。保持 indent=1 规范形（gate `check_scenes_canonical` 守）。槽位增删同步改 `balance.save.slot_count` 与 bevy 行装配的遍历口径（数据驱动，不改 Rust 常量）。
2. 改文案：改 `assets/data/strings.json` 的 `scene.saveload.*` / `slot.*` / `hud.gold_fmt` 键；`scenes.json` 只持有 `text_key`。
3. 换背景图：替换 `assets/ui/bg_night_sea.png`，在 `assets/ui_manifest.json` 保持登记一致（gate `check_asset_registry` 双向校验 + `export-icon-boundary` 守门）；选中底色来自主题令牌 `ui_brass_select`（`palette::BRASS_SELECT`），属主题数据而非本场景原语，改主题走主题资产/配置。
4. 重新构建：`scenes.json`/`strings.json` 由 `ti-core::data` 用 `include_str!` 编译期内嵌（data/mod.rs 第 48~50 行）⇒ 改完必须 `cargo build -p ti-engine`（或整 workspace 构建）后出图才生效；运行期注入仅覆盖地图包 `main_map` 段，saveload 不吃注入。
5. 出图验证：`cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot saveload`（竖屏）+ `--shot saveload --viewport=1920x1080`（横屏）；产物 `evidence/bevy_native_saveload*.png`，对照本文 §2。
6. 零 Rust 证据：`git diff -- crates/` 为空即达成（内容运营判据）；`render_intent.rs::saveload_layout_matches_doc` + `landscape_drawlist.rs` 锚定数值，改数据后测试随数据走、无需改测试代码。
