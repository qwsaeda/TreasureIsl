# main_map 场景 · 详细图文拆解

> **单一事实源声明（时点 2026-11-23）**：坐标/参数逐条抄自 `assets/data/scenes.json` 的 `main_map` 节点（竖屏 primitives 7 条 + `landscape.primitives` 6 条）与场景专属字段实测；叙事元数据抄自 `assets/data/ui_schema/scene_manifest_v3.json` 的 `main_map`；源码 = `crates/ti-app/src/scenes/{main_map.rs,main_map_node.rs,main_map_island.rs,chrome.rs}`；真渲染图 = `evidence/bevy_native_main_map_1920x1080.png`（1920×1080）+ `evidence/bevy_native_main_map.png`（1080×1920），均引擎 `--shot` 实拍；地图包 = `assets/maps/<id>/`（m01~m40）+ `docs/process/21_map_package_spec.md`；业务 = `docs/business/{territory,progression}.md`。结论引用文档 ID（EXP/S/F）不复述推导；坐标直接抄 JSON。

---

## 1. 场景职责与状态机位置

**身份**：`SceneKind::MainMap`（`scenes/mod.rs:73`，判别值 `4`/9，见 `discriminant()` `mod.rs:128`），处理器 = `MainMapScene`（`main_map.rs:35`，`kind()` 返回 `SceneKind::MainMap`，`main_map.rs:1106-1109`）。这是游戏**核心循环场景**：摇骰绕岛 → 落格结算 → 领地经营 → 战斗分派。

**在 9 场景态图中的位置**（`scenes/mod.rs:184-191` 合法转换表）：

```
Intro → Index → SelectRole → SelectMap → MainMap ⇄ Vs / Message
                                            │        │
                                            │        └→ MainMap（Message 关闭返回）
                                            ├→ Index（MapBack 返回键）
                                            └← Saveload（读档恢复：角色+地图齐全）
```

- **进入条件**（合法入边，`mod.rs:223-234`）：`SelectRole → MainMap`（角色 + 地图齐全）、`SelectMap → MainMap`（地图选定）、`Saveload → MainMap`（读档恢复）。非法入边（如 `SelectRole → Vs`）无 `scene_transition!` impl，编译期即禁。
- **退出条件**（合法出边）：`GameInput::Landmark(code)` 经 `map_session_dispatch` → `enter_target` → `Vs`（事件码 6）/ `Message`（事件码 1/4/2/5/7/8）；`GameInput::Spin` 绕圈满 `boss_min_laps` 遇 BOSS → `host.start_battle()` 切 `Vs`（`main_map.rs:1505-1521`）；`GameInput::MapBack` → `Index`（`main_map.rs:1523-1531`）；对局终局 `match_outcome` 在位时 `Confirm` → 清终局 + `Index`（`main_map.rs:1543-1548`）。
- **场景内状态机**（面板子集，互斥打开、`on_input` 按优先级消费）：`build_panel_open` / `trade_panel_open` / `merchant_panel_open` / `event_panel_open` / `ship_panel_open` / `garrison_panel_open` / `upgrade_panel_open` / `dungeon_panel_open` / `card_panel_open` / `achievement_panel_open`，外加 `pending_own` / `pending_siege` / `pending_dungeon`（落格三支分叉待决）、`handoff_pending`（热座交接）、`match_outcome`（终局）。打开任一面板会关其余全部（`main_map.rs:1114-1257`）。
- **渲染路径选择**（`main_map.rs:1581-1761`）：`v3 = has_world_canvas(&layout)`（`world_canvas` 键在位）；默认 `v6_default = v3 && 无 TI_MAP_V6_PROTO && 无 TI_MAP_V5_RING` ⇒ 走 `render_v6_elem`（不规则岛 + 格子紧围岛，S34 终码）。`TI_MAP_V6_PROTO` / `TI_MAP_V5_RING` 为原型保留环境变量；`v2` 回退（dormant）= `main_map_island::draw_board` + `render_track`。**世界层先画，HUD 原语（z≥3）与其后补画**（`render_primitives_filtered(...,3)` + `render_hud_v3`），解决海图满屏盖住铜牌数值的问题。

---

## 2. 真实渲染截图

**横屏（主交付）**：`--shot --scene=main_map --viewport=1920x1080`（S29.5 多视口 + S38 方向感知，`evidence/S38.0.log:32-33`；非黑 2,069,073/2,073,600≈99.8%，无药丸黑边铺满）。

![main_map 横屏真渲染图（1920×1080，v6 不规则岛 + 31 格木片环 + 双铜牌 HUD）](../evidence/bevy_native_main_map_1920x1080.png)

**竖屏（1080×1920）**：`--shot --scene=main_map`（`evidence/S61.0.log:44`，xvfb-run 无头；S38.0 竖屏对拍非动画区 diff=0）。

![main_map 竖屏真渲染图（1080×1920）](../evidence/bevy_native_main_map.png)

**图中可见元素**（两图同构，坐标按各自视口）：满屏羊皮做旧海图底（`chart_full.png`，外框木框 + 边缘暗角）；蓝绿海洋包围**中央不规则岛**（山峦 + 棕榈林 + 白沙滩 + 灯塔 + 海盗村落 + 岸边宝箱金币堆，拆分元素组合，12 号 spec §1.3）；约 **31 枚木片格**（海盗骷髅标记 = 可占领领地格，灰暗锁链/铜钱牌 = 功能格）沿岛轮廓不规则紧围（12 号 §1.1，凸包弧长等分）；格间缆绳路径深棕；底部黄铜**舵轮**摇骰 CTA（左）+ **罗盘**装饰（右）；顶角双铜牌（左「金 1000」金币图标 / 右「持有 0 种」宝箱图标）；玩家船与 `P1~P5` 席位徽标叠在格上（当前席金色船，余席暗灰旗，多席同格 `seat_slots` 扇出，`main_map.rs:300-343`）；海洋动画层（鲸鱼喷水等，S34.6 creatures 程序 bob，`CREATURES_ACC`）。**世界层（岛/格/牌/船/海洋）为代码绘制，非 primitives**——仅双铜牌/图标/标题/摇骰按钮来自 JSON 原语（见 §4）。

---

## 3. ASCII 布局图

**竖屏 1080×1920**（坐标 = scenes.json 竖屏实测；世界画布 `world_canvas (11,250,1048,1056)`）：

```
y=0   ┌──────────────────────────────────────────────┐
      │   标题 text (496,40,88,44)  ·「地图」         │
y=48  │ ┌─────铜牌L (48,48,448,192)────┐ ┌─铜牌R────┐ │
      │ │ icon coin (104,96,96,96) 金1000│ │chest 持有│ │
y=250│ └────────────────────────────────┘ └──────────┘ │
      │ ┌────────── world_canvas (11,250,1048,1056) ─┐ │
      │ │  ~~~~~~ 海洋 / 31 木片格环 / 中央岛 ~~~~~~  │ │
      │ │      [格] [格] [格]   ← 缆绳相连            │ │
      │ │        ╭────中央岛────╮   (岛心≈540,890)   │ │
      │ │        │ 山/棕榈/灯塔/村 │  P1 船 · 宝箱    │ │
      │ │        ╰──────────────╯                   │ │
      │ │      [格] [格] [格]                       │ │
y=1306│ └─────────────────────────────────────────────┘ │
      │  摇骰 CTA: 舵轮 (176,1625,230,230) pick_key=4  │
      │  骰数文本 (540,~1312)  ·  deck_spin(408,1336)  │
y=1760│                 提示 hint (540,1760)          │
y=1920└──────────────────────────────────────────────┘
      x=0   x=540(屏心)                          x=1080
```

**横屏 1920×1080**（`landscape.primitives` 6 条 + `landscape.extra`；世界画布由布局适配铺满）：

```
y=0   ┌────────────────────────────────────────────────────────────────┐
      │ 标题 text (860,24,200,44)「地图」                              │
y=32  │┌─铜牌L(48,32,448,160)─┐              ┌─铜牌R(1424,32,448,160)┐│
      ││ coin(104,80,80,80) 金1000│            │chest(1480,80,80,80)持有││
      │└──────────────────────┘              └───────────────────────┘│
      │ ┌──────────── 世界层：满屏羊皮海图 + 31 格环 + 中央岛 ─────────┐ │
      │ │   [格][格][格][格][格][格][格][格][格][格][格]            │ │
      │ │  [格]   ~~~ 海洋 ~~~    ╭────中央岛────╮      [格]       │ │
      │ │  [格]                   │山·棕榈·灯塔·村│      [格]       │ │
      │ │  [格]      P3/P5 船      ╰──────────────╯  P2 [格]       │ │
      │ │   [格][格][格][格][格][格][格][格][格][格][格]            │ │
      │ │        舵轮(左下)                    罗盘(右下)            │ │
y=1080└────────────────────────────────────────────────────────────────┘
      x=0                    x=960(屏心)                          x=1920
```

> 两图仅示意主区域；逐条精确坐标见 §4，专属几何参数见 §4.3。世界层坐标随视口由 `render_v6_elem(host,&layout,...,landscape)` 重算，横屏读 `main_map_v6_layout_landscape.json`（椭圆格环，S38 方案 A）。

---

## 4. 逐元素坐标表

> `primitives` = **chrome/UI 层**（资源由 `render_primitives_filtered` 绘制）；棋盘、岛、船、海洋等**游戏世界层不在 primitives 中**，走代码绘制（见 §5）。所有 `when=""`、`item=-1`。

### 4.1 竖屏逐元素坐标表（`main_map.primitives`，共 7 条 == 实测 7）

| # | kind | rect (x,y,w,h) | z | when | params（asset / icon / text_key / pick_key） | color |
|---|---|---|---|---|---|---|
| 1 | image | (0, 0, 1080, 1920) | 2 | "" | `asset=assets/ui/chart_full.png` | "" |
| 2 | image | (48, 48, 448, 192) | 3 | "" | `asset=assets/ui/hud_plaque.png` | "" |
| 3 | icon | (104, 96, 96, 96) | 4 | "" | `icon=coin, size=96` | gold_warm |
| 4 | image | (584, 48, 448, 192) | 3 | "" | `asset=assets/ui/hud_plaque.png` | "" |
| 5 | icon | (640, 96, 96, 96) | 4 | "" | `icon=chest, size=96` | gold_warm |
| 6 | text | (496, 40, 88, 44) | 6 | "" | `text_key=scene.main_map.title, tier=body` | gold_warm |
| 7 | image | (176, 1625, 230, 230) | 7 | "" | `asset=assets/ui/btn_spin.png, pick_key=4` | "" |

### 4.2 横屏逐元素坐标表（`main_map.landscape.primitives`，共 6 条 == 实测 6）

| # | kind | rect (x,y,w,h) | z | when | params | color |
|---|---|---|---|---|---|---|
| 1 | image | (0, 0, 1920, 1080) | 2 | "" | `asset=assets/ui/chart_full.png` | "" |
| 2 | image | (48, 32, 448, 160) | 3 | "" | `asset=assets/ui/hud_plaque.png` | "" |
| 3 | icon | (104, 80, 80, 80) | 4 | "" | `icon=coin, size=80` | gold_warm |
| 4 | image | (1424, 32, 448, 160) | 3 | "" | `asset=assets/ui/hud_plaque.png` | "" |
| 5 | icon | (1480, 80, 80, 80) | 4 | "" | `icon=chest, size=80` | gold_warm |
| 6 | text | (860, 24, 200, 44) | 6 | "" | `text_key=scene.main_map.title, tier=body`（在 params 内） | gold_warm |

### 4.3 场景专属几何参数表（`scenes.json main_map` 全量实测，单一事实源）

> 矩形值 = `extra`（点号标量折叠进 `extra_scalar`）；下列均为 scenes.json 实测值。Rust 侧只通过 `chrome::keys::*` 常量读取（`chrome.rs:163-229`），0 裸字面量。

| 键 | 值（x,y,w,h / 标量） | 用途 / 读取点 |
|---|---|---|
| `title` | `{y:8, h:96}` | 标题条高（v2 回退用） |
| `hud` | `{x:80,y:152,w:920,h:128}` | v2 羊皮 HUD 面板 |
| `hud_trade` | `{x:536,y:168,w:136,h:88}` | v2 经济按钮 贸易 |
| `hud_sell` | `{x:704,y:168,w:136,h:88}` | v2 经济按钮 出售 |
| `hud_build` | `{x:856,y:168,w:136,h:88}` | v2 经济按钮 建造 |
| `fx_coin` | `{x:272,y:144}` | 金币 FX 触发点（飞轮 R1 ③-02） |
| `grid` | `{x0:540,y0:520,tile_w:25,tile_h:13}` | v2 静态等距棋盘原点/tile |
| `player` | `{size:26}` | v2 玩家棋子边长回退 |
| `landmark` | `{size:20}` | v2 据点边长回退 |
| `boss` | `{size:24}` | v2 BOSS 边长回退 |
| `hint` | `{x:540,y:1760}` | 落格提示签条中心（`keys::HINT`） |
| `portrait_zones` | `[{x:0,y:150,w:1080,h:1680}]` | 竖屏内容安全带 |
| `world_canvas` | `{x:11,y:250,w:1048,h:1056}` | v3 世界画布（`keys::WORLD_CANVAS`，`board_fit` 拟合基准） |
| `hud_gold` | `{x:80,y:80,w:320,h:96}` | v3 顶左金币角（回退） |
| `hud_player` | `{x:680,y:80,w:320,h:96}` | v3 顶右持有量角（回退） |
| `hud_plaque_l` | `{x:48,y:48,w:448,h:192}` | v4 左铜牌（金币） |
| `hud_plaque_r` | `{x:584,y:48,w:448,h:192}` | v4 右铜牌（船/持有量） |
| `deck_trade` | `{x:48,y:1608,w:312,h:136}` | 底部命令盘 贸易（S34 已移除常驻按钮，数据保留） |
| `deck_sell` | `{x:384,y:1608,w:312,h:136}` | 底部命令盘 出售 |
| `deck_build` | `{x:720,y:1608,w:312,h:136}` | 底部命令盘 建造 |
| `canvas_entities` | `{player:100,landmark:80,boss:96}` | v3 实体边长百分比（× `board_fit.tw`） |
| `track_node` | `{size:120}` | v4 节点牌可见边长（% × tw） |
| `track_ship` | `{size:160}` | v4 玩家船可见边长（% × tw） |
| `focus_anchor` | `{type:world_canvas, bbox:{x:11,y:250,w:1048,h:1056}}` | 场景叙事焦点锚点 |
| `deck_spin` | `{x:408,y:1336,w:264,h:264}` | 摇骰 CTA 数据矩形（v5 残留位，v6 骰点改用 helm 定位） |
| `voyage_bar` | `{x:48,y:1088,w:984,h:48}` | v4 航程进度尺（v6 dormant） |
| `extra_scalar.ring.center_x` | `540` | 环心 x（`ring_geom.cx`，屏心豁免 8pt） |
| `extra_scalar.ring.center_y` | `780` | 环心 y（`ring_geom.cy`） |
| `extra_scalar.ring.radius` | `474` | 环半径（`ring_geom.r`；`2πr/31≈96`） |
| `extra_scalar.ring.card_w` | `96` | 卡宽（`ring_geom.w`） |
| `extra_scalar.ring.card_h` | `120` | 卡高（`ring_geom.h`） |
| `extra_scalar.ring.home_size` | `760` | 中央岛尺寸 |
| `island_home` | `{x:160,y:400,w:760,h:760}` | v5 中央据点岛贴图矩形 |
| `ring` | `{center_x:540,center_y:784,radius:480,card_w:96,card_h:120}` | 包内 `layout.json` 同名字段（地图包注入口径） |

> **注意两处时点差（均实测，非漂移）**：`extra_scalar.ring.center_y=780 / radius=474` 为 scenes.json 当前 v5 环参数；`ring.center_y=784 / radius=480` 与地图包 `layout.json` 一致（Rust `ring_geom` 缺键回退常量亦是 `RING_CENTER=(540,784)`、`RING_RADIUS=480`，`chrome.rs:694-701`）。v6 默认路径不读 `ring.*`，而读 `main_map_v6_layout.json` 的逐格坐标（12 号 §1.1）。

**横屏专属 `landscape.extra`**（`scenes.json main_map.landscape.extra` 实测）：

| 键 | 值 (x,y,w,h) |
|---|---|
| `hud_gold` | (48, 32, 448, 160) |
| `hud_player` | (1424, 32, 448, 160) |
| `hud_plaque_l` | (48, 32, 448, 160) |
| `hud_plaque_r` | (1424, 32, 448, 160) |

---

## 5. 横屏差异

- **原语数量**：竖屏 7 条 → 横屏 6 条（少第 7 条摇骰按钮 `btn_spin` 原语 + `pick_key=4`），即横屏 primitives 无命中键；世界层与 HUD 仍由代码绘制。横屏背景 `chart_full.png` 铺满 1920×1080（`evidence/S38.0.log:32-33` 非黑≈99.8%，无药丸黑边）。
- **坐标变化**：铜牌左右镜像从 `(48,48)/(584,48)` → `(48,32)/(1424,32)`，尺寸 `448×192` → `448×160`；金币/宝箱 icon `96→80`，位置 `(104,96)/(640,96)` → `(104,80)/(1480,80)`；标题 `(496,40,88,44)` → `(860,24,200,44)`（x 随屏心 540→960 平移 + 加宽）。`landscape.extra` 四键给出横屏 HUD 铜牌位（`hud_gold/hud_player/hud_plaque_l/hud_plaque_r`）。
- **orientation 切换逻辑**：`--shot --viewport=1920x1080` 由 `main()` 显式置 `Orientation::Landscape`（`evidence/S39.0.log:79`）；运行时 `window_resize_system` 纵横比变化 → `ti_app::menu_shell::set_orientation`（`evidence/S38.0.log:20-22`）。`main_map.rs` 中 `render_hud_v3` 用 `layout.extra_for(landscape)` 读当前方向 extra（`main_map.rs:128-132`），`render_v6_elem(..., landscape)` 读横屏布局 `main_map_v6_layout_landscape.json` 重算椭圆格环（S38 方案 A）。竖屏 0 行为变化已对拍证明（S38.0 非动画区 diff=0）。

### 5.1 为什么 main_map 的世界层不在 bevy_ui（S29.3e 判定）

B1「全 bevy_ui 响应式」落地时（S29.3a~e），5 个**纯 UI 场景**（index/select_role/select_map/saveload/message）的 chrome 已迁 bevy_ui；但 **vs / main_map 整体留 1080**，原因即 `19_bevy_ui.md` 坑位 9.8-13（`docs/process/19_bevy_ui.md:160`）：

1. **相机层叠遮挡**：bevy_ui 的 UI 相机（`IsDefaultUiCamera`，`order=1`，clear None）叠在 1080 世界相机之上。同屏位的 bevy_ui 节点会**遮住**1080 动态内容。
2. **本场景 chrome 与 world 层叠**：主地图的 HUD 铜牌/标题/摇骰按钮与棋盘格、中央岛、玩家船、HUD 数值同屏**交叠**（不是各自独立区域）。若把 chrome 迁 bevy_ui、动态世界留在 1080，铜牌背景板/节点会盖住岛与格、或世界内容穿透 UI 层。
3. **判据**：B1 约定「游戏世界保持 1080」（`17_layout_engine.md` 三条松绑签核之一）⇒ vs/main_map 整体留 1080；`bevy-cpu-parity` 改口径只保游戏世界、UI 层排除（`12_bevy_usage.md:261-262`，`S29.4`）。

因此 main_map 的 1080 世界层（岛/格/船/海洋/动态实体）与 chrome 原语统一走 CPU→1080 画布→Bevy `Sprite` 呈现；`scenes.json` 的 primitives 只承担**不交叠**的背景层（z=2 `chart_full`）+ 顶部铜牌/图标/标题（z=3~6）+ 摇骰按钮（z=7）。这也是 §2 截图中「岛/格/船清晰」而 primitives 仅 7~6 条的原因。

---

## 6. 资产接线

**primitives 直接引用**（路径来自 `params.asset`，登记项名见 `assets/ui_manifest.json` 的 `assets[].name/file`）：

| 资产路径 | 登记项 | 用途 |
|---|---|---|
| `assets/ui/chart_full.png` | `name:chart_full` / `file:assets/ui/chart_full.png`（`:417`） | 满屏羊皮海图底（z=2） |
| `assets/ui/hud_plaque.png` | `name:hud_plaque`（`:513`） | 顶角双铜牌底（z=3，左右各一） |
| `assets/ui/btn_spin.png` | `name:btn_spin`（`:497`） | 摇骰舵轮 CTA（z=7，pick_key=4） |

**icon 原语**：`icon=coin`（金币，左铜牌）/ `icon=chest`（宝箱，右铜牌），经 `chrome::hud_icon_def` → `assets/ui/icon_*.png` 白蒙版 + `gold_warm` tint（`S21.14`，export-icon-boundary 守门：图标只为 PNG 资产，代码矢量仅缺图回退，跨层 tint ≡ Bevy `Sprite.color`）。

**世界层资产（代码 `blit_image` 按命名约定，非 primitives）**：
- 9 类节点牌 `assets/ui/node_{start,city,port,event,card,treasure,boss,plain}.png` → `chrome::node_asset(kind)`（`chrome.rs:659-675`，唯一事实源 = `08_main_map_v4_spec.md` §6；可见外接框 182~488px 归一化到统一视觉边长 `asset_dest_rect`；缺图静默不画）。
- 中央岛 `assets/ui/island_home_<map_id>.png`（如 `island_home_m01`，manifest 登记 `:708`）→ `chrome::central_island_path(map_id)`（`chrome.rs:776`）。
- 岛型卡 `assets/ui/isle_<isle_id>.png`（10 类，manifest `isle_assets[]`）→ `chrome::isle_card_path`（`chrome.rs:768`）。
- 地块图 `assets/ui/tile_<map_id>_<格号>.png`（1024² 品红抠透明，每格专属）→ `main_map.rs::tile_plot_path` 自动发现，缺图回退程序绘制（`14_plot_tile_spec.md` §1/§3）。
- 玩家船 `assets/ui/boat_<lvl>.png` / `ship_player.png`（缺图回退代码 flag，`chrome::boat_asset`）。
- 元素级组合资产 `isle_land`/`isle_lighthouse`/`isle_treasure`/`isle_village`（12 号 §1.3，A6 对齐）、`creatures.png`（海洋动画层，12 号 §1.4）、舵轮/罗盘/帆船（`assets/ui_batches/main_map_v6.json`）。

**守门**：`validate_data::check_asset_registry` 全类型 disk↔registry 双向同步；新增资产须 `ui_assets.py add/build` 登记进 `ui_manifest.json`（`docs/process/18_asset_content_pipeline.md`）；`check_code_standards` CS-1 资产路径单源（`main_map.rs` 0 资产路径字面量，走 `ti_core::data` + `chrome::keys`）。

---

## 7. 命中契约

- **数据化命中**：`spawn_pick_layer`（`ti-engine`）只认 `pick_key>0` 的 primitives。main_map 竖屏 primitives 中**仅第 7 条 `btn_spin` 带 `pick_key=4`**（横屏 6 条无命中键）；历史 5/6/7（贸易/出售/建造）随 S34「底部三键取掉」删除（`main_map_pick_e2e.rs:7-8`）。
- **派发链**：物理点击 → `manual_1080_hit_system`（bevy_picking mesh 只拾取 Mesh2d/SimplifiedMesh，1080 标记是 Sprite ⇒ 手写几何拾取，`20_bevy_picking.md` 坑位 20-8/9）→ `PickEvent` → `poll_hit(HitData)` → `route_hit` → `Scene::on_hit`。主地图 `on_hit`（`main_map.rs:1566-1577`）：`swipe != 0` 直接吞；`key=4` → `host.advance(GameInput::Spin)`；其它键无动作。
- **键盘等价**：`Space` = 摇骰（`GameInput::Spin`），`T/Y/B` 等经 `Open*Panel` 开面板，面板内 `↑↓/Enter/Back` 导航（`on_input`，`main_map.rs:1112-1559`）。S19 起**移动唯一入口 = 摇骰**，方向键自由移动已彻底删除。
- **赛道/环格/中央岛命中**：v6 终码的 31 格与中央岛为代码绘制的 1080 内容，其交互走 `track_node_sel` 预览选中与落格分叉 `pending_*` + `Confirm/Back`，而非 primitives `pick_index`（`main_map_node::draw_node_card` 信息签只读预览）；e2e 由 `main_map_pick_e2e` + `app_*_panel_e2e` 覆盖。

---

## 8. 业务规则

仅引用单一事实源，不复述推导：

- **环岛赛道**（`docs/business/progression.md` §环岛赛道）：摇骰定步（1..=`dice_sides`，默认 6，`GameRng` 确定性）→ 前进 N 格 → 落格结算；绕圈 +`lap_bonus_gold`（150）；`E[摇骰]=13.6`、`E[见 BOSS 累计金]=4620`（含 initial_gold 5620，`board_measure.py` M6 口径）。BOSS = 经过或落格即遇、满 `boss_min_laps`（1）现身、拦停于 `spaces[31//2]`（第 15 格），击败 → 解锁下一图（m20 通关，`progression.md` §BOSS 触发规则 S19.3 + S61 终局通道）。
- **31 格构成与领地**（`docs/business/territory.md` §2）：起点 1 + 领地 20（10 类岛×2 相邻成区）+ 功能 9（事件 4/卡牌 3/宝藏 2）+ BOSS 1；10 类岛原型见 territory.md §3 表（`harbour`/`lighthouse`/…/`boss_fort`，各有专属副本与每圈产出）。
- **落格三支分叉**（territory.md §4）：无主 → 进副本占领（胜者该岛两格同归 + 留自定驻军）；他人 → 交过路费 or 攻占（败交双倍）；自己 → 升级加固 / 增减驻军（S64 玩家自定，`set_garrison` 守恒钳位）/ 领取产出。`Confirm`/`Back` 在 `pending_own`/`pending_siege`/`pending_dungeon` 上短路消费（`main_map.rs:1538-1557`）。
- **据点/事件/商人**（territory.md §5~§7 / progression.md §据点与事件）：中央大岛 = 当前席据点（非赛道格），7 种海事随机事件（每图特有），武器/奴隶商人（事件格触发）。
- **地图包不变量**（`docs/process/21_map_package_spec.md` §5 I1~I9 + §7 运行期加载契约 B）：31 格 / 10 类各 2 / BOSS 在环半周 / 8pt 环几何 / 资产 sha256 / N8 焦点 ±1px / 内容齐备；失败回退内嵌默认包（可玩性优先不崩）。

---

## 9. 测试覆盖

| 测试文件 | 断言要点 |
|---|---|
| `crates/ti-app/tests/main_map_pick_e2e.rs`（3 测） | ① `pick_key=4` 恰 1 个、5/6/7 恰 0 个且矩形非零且在屏内；② `poll_hit(HitData{key:4})` → `board_last_roll()>0`（摇骰可达）；③ 未知键 99 不摇骰 |
| `crates/ti-app/tests/app_trade_panel_e2e.rs`（4） | 贸易面板开关 / 选中钳位 / 买卖 |
| `app_ship_panel_e2e.rs`（3） | 造船面板开关 / 互斥 |
| `app_event_panel_e2e.rs`（3） | 事件面板开关 / 7 项循环选择 |
| `app_merchant_panel_e2e.rs`（4） | 商人面板开关 / 0↔1 选 / 互斥 |
| `app_garrison_panel_e2e.rs`（6） | 驻军面板 ±1 / 守恒 / 互斥 |
| `app_upgrade_panel_e2e.rs`（7） | 升级面板 金守恒 / 满级退化领产出 |
| `app_dungeon_panel_e2e.rs`（6） | 副本情报面板 进战/放弃待决 |
| `app_card_panel_e2e.rs`（5） | 卡牌图鉴翻页 / 关闭 |
| `app_panel_keyboard_e2e.rs`（7） | 面板键盘路由总表 / 互斥 |
| `crates/ti-app/tests/e2e_playability.rs`（7） | 完整对局到胜利+重开 / 破产失败 / 攻占 / 多轮经济 / AI 确定性 / 存读档 / 版面键控在视口内 |

**门禁覆盖**：`gate.ps1` 中 `bevy-cpu-parity`（仅游戏世界口径，UI 层排除，`S29.4`）以 main_map 为像素对拍主场景，超差 0 LSB；`--shot main_map` 非黑率判据（竖屏 S38.0=diff0 / 横屏 99.8%）；`check_scene_narrative` 锚 `scene_manifest_v3` 的 `main_map`（focus=bbox_pct(1,13,97,55) ↔ world_canvas N8 ±1px）；`validate_data` 守资产/地图包/31 格不变量；`doccheck/doccheck2` 守跨文档数值一致。

---

## 10. 数据驱动改法（换图 / 改坐标 / 换资产，零 Rust）

**A. 换一张地图（内容运营主路径，零 Rust）**：
1. `scripts/map_pack.py add <id>` 建包 → 填 `assets/maps/<id>/{map.json,layout.json,strings.json,manifest.json}`，其中 `layout.json` 的 `ring.{center_x,center_y,radius,card_w,card_h}` / `world_canvas` / `island_home` / `hint` 即几何数据。
2. 资产按命名约定落位：`island_home_<id>.png`、`tile_<id>_<格号>.png`（0..=30）、`isle_<isle_id>.png` —— 无需描述符，代码 `central_island_path` / `tile_plot_path` / `isle_card_path` 自动发现（`chrome.rs:768-778`，`14_plot_tile_spec.md` §3）。
3. `map_pack.py verify`（过 I1~I9）→ `pack` → `install`（写 `assets/maps/<id>/` + 更新 `.default`）→ `check` → `sync` 回注 `board.json`/`scenes.json`/`manifest`。
4. `TI_MAP=<id> --shot --scene=main_map` 出图过目（`evidence/S31.2.log:26-29`：m02/m03 注入过，缺包回退内嵌 0 panic）。

**零 Rust 证据**：Rust 侧只有「读数据 + 按命名约定拼路径 + 缺图回退」，`map_slot_id()`（`game_app/mod.rs:679`）来自运行态选图数据；换图 = 改 JSON + 换 PNG + `sync`，`git diff crates/` 为空（内容运营判据，`29_content_ops.md`）。`ring_geom` 读 `extra_scalar` 点号键、缺键回退常量（`chrome.rs:720-729`），`ring_card_center` 按 `θ=-90°+i·360°/count` 算 31 格中心（`chrome.rs:740-749`）——**环几何与格数全数据驱动**。

**B. 改 UI 坐标 / 换资产（不碰 Rust）**：
- 改铜牌/标题/摇骰按钮位置：直接改 `scenes.json` 的 `main_map.primitives[]` / `landscape.primitives[]` 的 `rect`（min_z=3 原语在世界层之后绘制，见 §1）；横屏改 `landscape.extra`。
- 换底图/铜牌/舵轮贴图：改 `params.asset` 指向的路径并在 `ui_manifest.json` 登记 + `ui_assets.py build`；或同名替换 PNG（注意 8pt/屏心豁免约束，`doccheck2` 8pt 守门）。
- 换节点牌：改 `chrome::node_asset` 对应的 `assets/ui/node_*.png`（9 类形状，可见外接框归一化）；换中央岛/地块图：换 `island_home_<id>.png` / `tile_<id>_<格号>.png`。
- 出图验收：`cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot --scene=main_map`（竖屏）/ 加 `--viewport=1920x1080`（横屏），落 `evidence/bevy_native_main_map*.png` 过目；再跑 `gate.ps1` 全绿。
- 视觉改动遵守 **A 纪律（设计稿先行 + 用户确认）**：12 号 spec / 14 号 spec 定案 → 出图 → 用户过目留痕后才落终码（EXP-141/EXP-114）。
