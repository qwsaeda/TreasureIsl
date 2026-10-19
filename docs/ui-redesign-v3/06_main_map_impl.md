# 06 · main_map 主地图落地记录（v3.1 全量 · S17.8）

> 03 提案 §3.4「世界画布优先」的落地版。main_map 是 7 场景中死区最大（v2 棋盘
> y=520..1040 只占中屏一条，上下 280px+ 死区）的场景。v3 = **棋盘当主角**：
> mood `world_explore`，focus.anchor = `world_canvas`，rhythm = `world_first_hud_corner`。
> **双写纪律同 S17.6/7**：v2 键（`grid.*`/`player.size`/`hud`/`hud_trade|sell|build`）
> 坐标一律不动，v3 键（`world_canvas`/`hud_gold`/`hud_player`/`deck_*`/`canvas_entities`/
> `focus_anchor`）顶层平铺叠加；v3 键缺席 → `board_fit` 回退 v2 静态 `grid.*`，版面不变。

## 1. 关键设计裁决（比坐标更重要）

提案 §3.4 给的固定 `tile 56×28 / y0=800` **在真网格上放不下**：
- 真网格 = 15×15 或 20×20（`balance.json`）。等距菱形宽跨度 = `2·G·tile_w`。
- 世界画布宽 1004 → `tile_w ≤ 1004/(2·G)`：G=20 → ≤25.1（8pt 对齐 = 24，与 v2 25 相当）；
  G=15 → ≤33.4（8pt 对齐 = 32，放大 1.28×）。
- 固定 56×28 在 20×20 上横向溢出屏外 → **改用 G 依赖运行时拟合** `chrome::board_fit`
  （唯一能同时对 15/20 两种真网格「放大且不溢出」的数据驱动解）。

## 2. 版面草图（1080×1920 单坐标系，G=20 实例）

```
y=80   ┌ [金币 999]        顶左 hud_gold {80,80,320,96}
       │                 [持有 12 件]  顶右 hud_player {680,80,320,96}
y=192  │ ╔════════ world_canvas {43,192,1004,1478} 装饰框 ════════╗
       │ ║                                                        ║
       │ ║            ◇ ◇ ◇  等距棋盘（G 依赖拟合进画布）          ║
       │ ║        y0≈691..1171（菱形，8pt 对齐，画布中心）          ║
y≈931  │ ║            玩家/据点/BOSS 实体（canvas_entities 比例）   ║
       │ ║                                                        ║
y=1760 │ ║  [贸易] [出售] [建造]   底部命令盘（P-UW4-3）            ║
       │ ╚══════════════════════════════════════════════════════╝
y=1840 │  方向键移动…（hint，v2 保留）
```

z-order（调用序，`main_map.rs render`）：`parchment_bg` → `nautical_grid` → 等距瓦片
（`iso_cell_fit`，G 依赖）→ `world_canvas_frame`（垫底装饰框）→ `wood_frame`/`title_bar`/标题
→ **v3**：`render_hud_v3`（hud_gold/hud_player 顶角 + deck_trade/sell/build 底部）| **v2**：
`render_hud_v2`（羊皮纸 HUD 条 + 三按钮）→ 实体（`iso_cell_fit` + `entity_size_fit`）→ 实体标签 → hint。

## 3. 坐标方案（scenes.json main_map 段，v3 顶层平铺键）

| 键 | 值 | 依据 |
|---|---|---|
| `world_canvas {x:43, y:192, w:1004, h:1478}` | 画布矩形 | = manifest main_map `focus.bbox_pct{4,10,93,77}` × (1080,1920) 取整：4%→43.2→43、10%→192、93%→1004.4→1004、77%→1478.4→1478（N8 对账 ±1px） |
| `hud_gold {x:80, y:80, w:320, h:96}` | 顶左金币角 | P-Hades-1 HUD 边角化 |
| `hud_player {x:680, y:80, w:320, h:96}` | 顶右持有量角 | 显示 `owned_fmt`（不造新 i18n 键；提案「玩家位」留 v3.5） |
| `deck_trade/sell/build {x:240/460/680, y:1760, w:200, h:80}` | 底部命令盘 3 键 | P-UW4-3；语义 = 键盘 T/Y/B / v2 `hud_trade/sell/build` 同 |
| `canvas_entities {player:100, landmark:80, boss:96}` | 实体 = `board_fit.tw` × 百分比 | 15 格 tw=32→32/26/31；20 格 tw=24→24/19/23；缺席回退 v2 `player.size` 等（QA106） |
| `focus_anchor {type:world_canvas, bbox:{43,192,1004,1478}}` | 视觉重心锚 | gate 第 21 项 `check_scene_narrative` N8 对账值（= `world_canvas` 矩形） |
| v2 键不动 | `grid{x0:540,y0:520,tile_w:25,tile_h:13}`、`player/landmark/boss.size`、`hud`、`hud_trade/sell/build` | 双写回退源（v3 键缺席时版面 100% 还原） |

## 4. 实现映射（代码落点）

| 层 | 落点 | 说明 |
|---|---|---|
| 键契约 | `chrome::keys`（单一事实源） | scenes.json ↔ 代码版面键名 const 表：`grid.*`/`world_canvas`/`hud_gold`/`hud_player`/`deck_*`/`hud_*`/`canvas_entities`/`<key>.size`/v3 场景 art 键（`ambient_art`/`title_art`/`battle_arena`）。各 read 站点（chrome/main_map/touch）一律引用 const，勿写裸字面量；改 scenes.json 键名只改 `keys` 一处 |
| 几何 | `chrome::board_fit(layout, G) → BoardGeom{x0,y0,tw,th}` | v3 拟合：`tw=(画布宽/(2·G))/8·8`（≥8 否则回退 v2）、`th=tw/2`、`x0=画布cx−tw`、`y0=画布cy−G·th`（菱形中心对齐画布中心）；缺席/非正回退 v2 `grid.*` |
| 正/逆 | `iso_cell_fit`/`iso_tile_fit`（正）↔ `touch::iso_grid_cell_at`（逆） | 同读 `board_fit(layout, G)`，EXP-067 正逆成对（`iso_cell`/`iso_tile` v2 保留为回退路径） |
| 实体 | `entity_size_fit(layout, key, G)` | `canvas_entities.<key>` 百分比 × `board_fit.tw`，缺席回退 v2 `<key>.size` |
| 装饰 | `ti_render::shell::world_canvas_frame` | 外框 2px `wood` + 内金线 `gold_bright` + 四角 12×12 `gold` 罗盘方位块（纯静态 0 动效，色板 0 外部资产） |
| 命令盘 | `chrome::deck_button` + `main_map::render_hud_v3` | 面板 + 图标（查 `ui_icons`）+ 28px 标签；缺失/非正不画 |
| 触摸 | `main_map::on_touch` | `deck_*` 优先命中 → 回退 v2 `hud_*`（e2e 经济循环仍命中）→ 网格点按（`grid_cell_at` 已 G 依赖）→ 滑动 |
| G 源 | `GameApp::map_grid_width`（新 pub 访问器） | 暴露 `terrain_grid_w` 供集成测试取真实 G（`board_fit` 的 G 依赖源） |
| 8pt | `doccheck2::ui_8pt_walk` 豁免 7 键 | `world_canvas/hud_gold/hud_player/deck_trade/deck_sell/deck_build/canvas_entities`（43/192/1004/1478 非 8pt 整除属预期，v3 键由 DSL 接管） |
| 镜像 | `08_layout_design §4.3` 加 8 行 v3 键 | 表值 = scenes.json 同值（gate 第 9 项 `layout-doc-table-cross-check` 对账） |
| 基线 | `render/forward/hd` 三套 `main_map` 重生 | 其余场景 0 diff（main_map 版面唯一变化方） |

## 5. 像素证据（render_baseline/main.png 抽检，G=20 实例）

| 点 | 采样 | 值 | 判定 |
|---|---|---|---|
| 棋盘中心 (545,931) | 瓦片/描边 | (33,44,140) | 棋盘在画布中心在位 |
| 棋盘瓦片 (545,1100) | `COLOR_MENU_BG` | (24,32,99) | 瓦片填充在位（y≈1100 属新菱形 691..1171） |
| 旧 v2 棋盘位 (545,520) | 底色 | (8,12,41) | 棋盘已离开 v2 `y0=520` 起点（移位实证） |
| 顶左金币角 (120,128) | `menu_bg` 面板 | (24,32,99) | HUD 边角化在位 |
| 命令盘 (340,1800) | `menu_bg` 面板 | (24,32,99) | 底部命令盘在位（y=1760..1840） |
| 画布框顶边 (540,192) | `wood` 发丝 | (57,65,189) | `world_canvas_frame` 外框在位 |

## 6. 重锚的 e2e（同 G 自洽）

v2 棋盘几何 G 无关（静态），v3 `board_fit` G 相关 → 跨 G 比较不再成立。重锚为**同 G**：
- `touch::grid_cell_inverts_iso_cell`：正向 `iso_cell_fit(·,G)` + 逆解 `grid_cell_at(·,G,G)` 同 G round-trip；越界格（G=15 的 (19,19)）同 G 逆解 → None。
- `game_app_flow::iso_touch_matches_move` / `iso_touch_render_cross_layer`：`iso_cell` → `iso_cell_fit(·,20)`。
- `render_intent::dynamic_entities_land_in_declared_portrait_zones`：`iso_cell`/`iso_tile` → `iso_cell_fit`/`iso_tile_fit`，G 取 `app.map_grid_width()`（真实载入图 G）。

## 7. 后续波（v3.4/v3.5，未开工）

- **v3.5 收口**：删 v2 键（`grid.*`/`hud`/`hud_trade|sell|build`/`player.size` 等），`board_fit`
  回退路径退役；`render_intent::main_map_layout_matches_doc` 的 `hud.y+h ≤ 520` / `size ≤ 2·tile_h`
  QA106 锚随 v2 键删除重锚。
- **HUD 右角「玩家位」**：需新 i18n 格式键（`scene.main_map.hud.pos_fmt`），随 i18n 波次做。
- **命令盘真业务扩展**：当前 3 键 = 买 1/卖 1/造 Castle（与 v2 键盘 T/Y/B 同语义）；4 键
  （含「据点」事件）随主地图事件扩展业务功能落地。
- **棋盘动效**：`world_canvas_frame` 纯静态；S17.4 动效波次另议（vignette/罗盘缓转）。
