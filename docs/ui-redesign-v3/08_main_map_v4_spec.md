# 08 · main_map 重设计 v4「羊皮海图桌面」版面与交互规格（S21.3 定案）

> **本文件 = main_map v4 的版面 / 交互 / 资产接线的单一事实源**（坐标 1080 坐标系，与
> `docs/business/08_layout_design.md` §4.3 镜像互锁）。`scenes.json` main_map 块、
> `scene_manifest_v3.json` main_map 段、`main_map.rs` / `chrome.rs` 的画法**都必须**对齐本表。
>
> 触发：用户 2026-10-22「main_map 设计交互界面很烂，出新的设计渲染图」→ 选定
> **概念稿 A（古典羊皮海图桌面）** + **满屏海图铺满整屏** + **45 节点全显但重做多形状实心黄铜牌**。
> 概念稿见 `evidence/ui_concept/main_map_v4_{a,b,c}.png`（gitignored，仅参考，不入仓）。

## 0. 设计意图（概念稿 A + 船系统 S21.2）

- **整屏 = 一张摊开的羊皮海图**（`chart_full`，含经纬网 / 罗盘玫瑰 / 洋流 / 做旧 / 深木桌沿），
  不再是「照片夜空 + 400 个纯色菱形瓦片 + 45 个小方块」。
- **岛屿是画在海图上的暖砂剪影**（代码按 `terrain.json` 只画 陆/礁/宝 格，**不画海洋格** → 巨型蓝菱形消失），
  **环岛赛道 = 绕岛一周的黄铜航线 + 45 个实心黄铜节点牌**（多形状：圆/六边/圆角方/盾/菱/尖星/宝箱/骷髅/铜钉，
  不再是清一色小方块），**玩家 = 船**（按 `boat_level` 切 4 张船图，S21.2 船系统 / S21.12 贴图）。
- 版面纵向节奏：**海图（重心）→ 航线进度尺 → 大号黄铜舵轮 CTA（摇骰）→ 底部三键铜牌 → 落格提示签条**。

## 1. 层结构（z 升序 = 调用序，`08_render_arch.md` 契约 1）

| z | 内容 | 来源 |
|---|---|---|
| 2 | 满屏海图 `chart_full`（0,0,1080,1920） | primitive `image`（资产） |
| 3 | 顶角双铜牌（金币 / 船·持有） | primitive `image`（`hud_plaque`）+ 运行时数字 |
| 4 | （v2 回退木框 / 装饰，v4 不画；键保留数据） | — |
| 5 | 世界画布：代码岛屿（地形）+ 黄铜航线 + 45 节点牌（`blit_image`）+ 玩家船 | **运行时**（`main_map.rs` + `chrome.rs`） |
| 6 | 航线进度尺（环岛摊平） | 运行时（`chrome::voyage_bar` v4 重做） |
| 7 | 舵轮 CTA（`btn_spin` 大图 + 骰点） | primitive `image` + 运行时骰点 |
| 8 | 底部三键铜牌（贸易/出售/建造，S21.12 + 造船） | primitive `image`（`deck_plaque`）+ 图标 + 标签 |
| 9 | 落格提示签条 / 节点信息卡 / 全部文本（QA142 文本最上层） | 运行时 |

> 运行时内容（z5~z9）**永远画在所有 JSON 原语之上**（层序 = 调用序，`main_map.rs` 先 `render_primitives`
> 再画运行时）。想「棋盘压在命令盘下」只能在 `scenes.json` 里表达，不能靠 z。

## 2. 逐元素坐标表（1080×1920，非豁免键须 8pt 整除；`world_canvas` = v3 豁免键 + N8 耦合）

| 元素 | 键 | rect `{x,y,w,h}` | z | 载体 | 说明 |
|---|---|---|---|---|---|
| 满屏海图 | （z2 原语） | `{0,0,1080,1920}` | 2 | `image` asset=`chart_full` | 替代 v3 的 `bg_night_sea_top`+`chart_wide`+经纬网 rect+罗盘 rect（海图美术已烘焙进 `chart_full`） |
| HUD 顶左铜牌 · 金币 | `hud_plaque_l` | `{48,48,448,192}` | 3 | `image` asset=`hud_plaque` + `coin` 图标 + `hud.gold_fmt` 数字 | 替换 v3 `hud_gold{80,80,320,96}` |
| HUD 顶右铜牌 · 船/持有 | `hud_plaque_r` | `{584,48,448,192}` | 3 | `image` asset=`hud_plaque` + 船图标 + `hud.owned_fmt` 数字 | 替换 v3 `hud_player{680,80,320,96}`；船图标 = `boat_level` 对应船 |
| 标题 | `title` | `{y:8, h:96}`（无 x，居中） | 6 | `text` `scene.main_map.title` | 沿用 |
| **世界画布（岛+赛道焦点）** | `world_canvas` | `{43,250,994,826}` | 5 | `board_fit` G 依赖 + `focus_anchor` | **v4 主焦点**；N8 锚点（见 §5） |
| 航线进度尺 | `voyage_bar` | `{48,1088,984,48}` | 6 | 运行时（环岛摊平：黄铜槽 + 每格点 + 玩家船标 + ☠ BOSS 端标） | 替换 v3 `{120,1056,840,120}`；45 点噪声条 → 1 条进度尺 |
| 摇骰 CTA · 舵轮 | `deck_spin` | `{408,1160,264,264}` | 7 | `image` asset=`btn_spin` + 运行时骰点（`roll_fmt`，CTA 上沿 y≈1136） | 替换 v3 `{336,1328,400,120}`；**大号黄铜舵轮**（概念稿 C 的骰子压罗盘） |
| 命令铜牌 · 贸易 | `deck_trade` | `{48,1440,312,136}` | 8 | `image` asset=`deck_plaque` + `coin` 图标 + `hud.trade` 标签 | 替换 v3 `{240,1600,200,80}` |
| 命令铜牌 · 出售 | `deck_sell` | `{384,1440,312,136}` | 8 | 同上 + `hud.sell` | 替换 v3 `{460,1600,200,80}` |
| 命令铜牌 · 建造 | `deck_build` | `{720,1440,312,136}` | 8 | 同上 + `hud.build`；S21.12 在此面板内加「造船」第 5 入口 | 替换 v3 `{680,1600,200,80}` |
| 落格提示签条 | （运行时） | `{344,1600,392,64}` | 9 | 运行时（`brass_dark` 底板 + 黄铜边，宽按字自适应最小 392） | 替换 v3 y=1520；落在三键下沿 1576 之下 |

**8pt 校验**（非豁免键全过；`world_canvas` 为 v3 豁免键不参与 8pt）：
`48/448/192/584`、`1088/984/48`、`408/1160/264`、`312/136/384/720/1440`、`344/1600/392/64` 均 8 整除 ✓；
屏心锚点 `540` 豁免 ✓。`world_canvas {43,250,994,826}` 非 8pt 但键名在 8pt 豁免表（v3 叙事键）✓。

**board_fit 适配**（`world_canvas {43,250,994,826}`，中心 `(cx,cy)=(540, 250+826/2=663)`）：
- `g=20`（m01/m02）→ `tw=994/(2·20)=24`（8 整）、`th=12`，菱形 `y0=663-20·12=423`，底 `423+40·12=903` ⊂ 画布 `250..1076` ✓
- `g=15`（m03）→ `tw=994/(2·15)=32`、`th=16`，菱形 `423..903` ⊂ 画布 ✓
- 左右 `540±480=60..1020` ⊂ `43..1037` ✓（`EXP-067` 正逆成对：`iso_cell_fit` ↔ `grid_cell_at` 同一 `board_fit`）

## 3. 信息架构（场景叙事 DSL，`scene_manifest_v3.json` main_map 段）

- **mood** = `world_explore`（N2 锁定：`focus.anchor` 必须 `world_canvas`、`rhythm.rule` 必须 `world_first_hud_corner`）；
- **focus** = `world_canvas`，`bbox_pct {x:4, y:13, w:92, h:43}`（N8 对账，见 §5）；
- **rhythm.beats**（权重和 = 1.0，beats[0] = focus.anchor，权重 ≥ 0.30；N4/N5 2~4 拍）：

| beat | weight | y_pct | 语义 |
|---|---|---|---|
| `world_canvas` | 0.50 | 40 | 岛 + 赛道（主焦点） |
| `command_cta` | 0.20 | 66 | 舵轮 CTA（摇骰主操作） |
| `hud_corners` | 0.15 | 6 | 顶角双铜牌（金币 / 船） |
| `hint` | 0.15 | 90 | 底部三键 + 提示签条 |

- **info_density**：`center_max_per_row=3`、`edge_min_per_row=1`、`dead_zone_max_pct=10`（N7 ≤25）；
- **permanent_hud_count_max = 3**（N6 world_explore 上限：顶左铜牌 + 顶右铜牌 + 航线尺 = 3，已满）。

## 4. 交互契约（`main_map.rs` on_touch；命中判定与渲染同坐标系，`EXP-067`）

命中优先级（自上而下，先命中先吃）：

| 命中区（rect） | 动作 | 语义 |
|---|---|---|
| `deck_spin {408,1160,264,264}` | `GameInput::Spin` | 摇骰前进（主操作，最先命中） |
| `deck_trade {48,1440,312,136}` | `TradeBuy(1)` | 买商品 1（键盘 T 同义） |
| `deck_sell {384,1440,312,136}` | `TradeSell(1)` | 全卖商品 1（键盘 Y 同义） |
| `deck_build {720,1440,312,136}` | `Build(Castle)` | 造/升级（键盘 B 同义；S21.12 内加造船入口） |
| `world_canvas` 内（等距逆投影 `grid_cell_at`） | 见下 | 点格 |
| 其余 | 无动作 | 滑动 = 无（S19 收口） |

**点格语义**（`world_canvas` 内，`grid_cell_at` 逆投影 = `chrome::iso_cell_fit` 的逆，数据源同一 `board_fit`）：
- 落在**自身当前格** → `Landmark(1)`（据点菜单，P0-5）；
- 落在**赛道格（`board_spaces` 某 node 中心 ±命中半径）** → **节点高亮 + 信息卡**（S21.7 新增）：
  - 高亮：该节点牌外圈 `gold_warm` 2px + 放大 1.1×；
  - 信息卡：签条式面板（`brass_dark` 底板）锚定在被点节点上方，文案 = `scene.main_map.node.<kind>`（起点/城镇/港口/事件/卡牌/遭遇/宝藏/BOSS）+ 该 kind 的落格产出（复用 `scene.main_map.toast.*_fmt` 数值）；
  - **只读预览，不移动**（移动唯一 = 摇骰，S19 收口不破）；
- 落在非赛道 / 海洋格 → 无动作（可点亮的只有岛 + 航线）。

**摇骰反馈**：落格后目标节点脉冲高亮 1 拍 + 提示签条（z9）+ 骰点文本（CTA 上沿）。

## 5. 门禁耦合点（改版面必须同改，否则门禁 FAIL）

| 门禁 | 锚点 | 本规格目标值 |
|---|---|---|
| N8（`check_scene_narrative`） | `scenes.json` `focus_anchor.bbox` ≡ `manifest.focus.bbox_pct × (1080,1920)` ±1px | `focus_anchor {bbox:{43,250,994,826}}` ↔ `bbox_pct {x:4,y:13,w:92,h:43}`（4×10.8=43.2→43 / 13×19.2=249.6→250 / 92×10.8=993.6→994 / 43×19.2=825.6→826，全 ±1 内 ✓） |
| N3（焦点面积 ≥20%） | `w×h/100` | 92×43 = 39.56% ✓ |
| N4（beats[0] = anchor，权重 ≥0.30，和 =1.0） | `world_canvas` 0.50 / 和 1.00 ✓ | |
| N5 / N6 | beats 4 个（上限）/ 常驻 HUD = 3（上限） | 已满，不加 |
| 8pt（`doccheck2::ui_8pt_walk`） | 非豁免键 8 整除 | §2 全过；`world_canvas` 键名在豁免表 |
| 原语闭集（`check_data_driven` / `check_primitives_data`） | 7 种（rect/text/icon/grad/glow/disc/image） | 只 `image`（节点牌 / 铜牌 / 海图 / 舵轮）+ 运行时，**不新增原语 kind** |
| 资产登记（`ui_assets.py verify` / 孤儿） | `image` 原语 asset 必须入 `ui_manifest.json` | §6 全已登记（`boat_*` 待 S21.12） |

> **改 `world_canvas` 矩形 ⇒ 必须同步改 `manifest.focus.bbox_pct`（N8 硬耦合）**，否则 `check_scene_narrative` FAIL。

## 6. 资产接线表（运行时 `blit_image` / primitive `image`；`ui_manifest.json` 单一事实源）

| 用途 | 资产 | 状态 |
|---|---|---|
| 满屏海图底 | `assets/ui/chart_full.png`（`background` 9:16 key=none） | 已入仓（S21.1） |
| 顶角铜牌 | `assets/ui/hud_plaque.png`（`plaque` 21:9 magenta） | 已入仓（S21.1） |
| 底部命令铜牌 | `assets/ui/deck_plaque.png`（`plaque` 21:9 magenta） | 已入仓（S21.1） |
| 舵轮 CTA | `assets/ui/btn_spin.png`（`medallion` 1:1 luma） | 已入仓（S21.1） |
| 节点牌 ×9 | `assets/ui/node_{start,city,port,event,card,battle,treasure,boss,plain}.png`（`medallion` luma） | 已入仓（S21.1） |
| 玩家船（1 级顶替 / 默认） | `assets/ui/ship_player.png`（`medallion` luma） | 已入仓（S21.1） |
| 玩家船 ×4（按 `boat_level`） | `assets/ui/boat_{1,2,3,4}.png`（`medallion` luma） | **待 S21.12 生成**（`main_map_v4.json` 批次补 4 条） |
| 罗盘玫瑰（若海图未含） | `assets/ui/compass_rose.png`（`medallion` luma） | 已入仓（S21.1，备选） |

> **blit 形变口径**：`image` 原语把资产**拉伸**到目标矩形。`plaque` 资产 = 21:9（1024×439）：
> `hud_plaque` 448×192 = 精确 21:9（无形变）；`deck_plaque` 312×136 = 2.29:1（≤2% 拉伸，可接受）；
> `chart_full` 1080×1920 = 精确 9:16 无形变；`medallion` 资产 1:1 blit 1:1 矩形（`btn_spin` 264×264、
> 节点牌随 `tw` 倍率仍 1:1）无形变。

**节点 kind → 资产映射（唯一事实源，S21.5 落 `chrome.rs` 映射表）**：
`start→node_start / city→node_city / port→node_port / event→node_event / card→node_card /
battle→node_battle / treasure→node_treasure / boss→node_boss / plain→node_plain`；
玩家标记 = `boat_{boat_level}`（缺资产回退 `ship_player`，再回退代码 `flag` 图标）。

## 7. 改动面声明（硬规则 7）

| 块 | 本规格（S21.3） | 落码任务 |
|---|---|---|
| 业务规则 | **无**（船系统规格 S21.2 已定案；本规格不改摇骰 / 落格 / 贸易 / 战斗规则） | — |
| 数据 | **无**（坐标表先落文档；`scenes.json` / `manifest` / `strings` 改在 S21.5/6/7、S21.12） | S21.5/6/7、S21.12 |
| 规则 | 无 | S21.10（rules::boat，独立任务） |
| 表现 | 规格定义层序 / 命中 / 资产映射（不写码） | S21.4~S21.7 |
| 资产 | 规格引用已入仓资产 + 登记 `boat_*` 待 S21.12 | S21.12 |

**跨模块原因**：满屏海图（z2 资产）+ 岛屿（z5 代码）+ 赛道（z5 代码）+ 节点牌（z5 资产映射）+
铜牌 CTA（z7/8 资产）必须**同一版坐标表**，否则「贴图铺满但棋盘错位 / 铜牌压岛 / N8 断链」。
故本规格作为单一事实源先定案，S21.4~S21.7 逐个落码时只读本表。

## 8. 与旧版面的差异（S19.7 v3.5 → S21 v4）

| 元素 | v3（S19.7） | v4（本规格） |
|---|---|---|
| 底 | `bg_night_sea_top` 照片 + `chart_wide` 半幅羊皮 + 经纬网/罗盘 rect | `chart_full` 满屏海图（经纬网/罗盘/洋流烘焙进图） |
| 棋盘 | 400 格全画（含 210 海洋格）巨型蓝菱形 | 只画 陆/礁/宝（`terrain.json`），海洋格不画 → 岛屿剪影 |
| 赛道 | 45 × 20px 小方块 + 28px 图标打架 | 黄铜航线实线 + 45 多形状实心黄铜牌（全显） |
| 玩家 | 绿方块 + flag 图标 | 船（`boat_level` 切 4 图，S21.12） |
| HUD | `hud_gold/hud_player` 320×96 角块 | 顶角双 `hud_plaque` 448×192 铜牌 |
| CTA | `deck_spin` 400×120 面板 | `deck_spin` 264×264 黄铜舵轮 |
| 命令盘 | 3 × 200×80 | 3 × 312×136 铜牌 |
| 航程条 | 45 点噪声条 840×120 | 1 条航线进度尺 984×48 |
| 提示条 | y=1520 | y=1600（三键之下） |
