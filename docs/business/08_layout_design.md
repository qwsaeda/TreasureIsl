# 界面布局重设计（全量，1080 现代版面）

> **S18.7b：像素基线机制已退役（不再存在）**。
> 依旧的 `assets/render_baseline/` / `forward_baseline/` / `hd_baseline/` 三套 golden PNG、
> `render_snapshot` / `render_snapshot_forward` / `visual_regression` 三个 diff 测试、`visual_baseline.ps1`
> 与其门禁项、`baseline-mirror-identity` 校验均**已全部删除**。
> 现行机制：**结构不变式断言**（`tests/render_invariants.rs`，7 场景 × 非黑覆盖率/内容签名互异/暖色板在位/TTF 在位）
> 与 **CPU↔Bevy 内存对拍**（`bevy-cpu-parity` 门禁项：GPU 真实离屏读回 vs CPU 期望逐像素对照）。
> 本文中仍有的「基线」词句为**历史记述**，不再是现行机制。

**定位**：场景版面与配色/字体的**单一事实源**。`scenes.json`（`assets/data/scenes.json`）
坐标以本文为准（正向设计），不依赖任何 legacy 资源记录作为布局依据。

> **与渲染架构的边界**：运行时架构分层、绘制原语、z-order 枚举、坐标契约在
> `docs/process/08_render_arch.md`。本文只管**版面/内容层** = `scenes.json` 坐标 +
> 场景 render 接线 + 控件语义 + 配色 + 字体，不重复架构结论。
>
> **legacy 像素层已退出运行时**（S16 渲染线裁决）：运行时像素 100% 来自
> `scenes/chrome.rs` 代码绘制 + `assets/fonts/` TTF 字体。`.Map`/`ui.pak`/PCX 降为
> **逆向事实参照与诊断输入**，不参与渲染（详见 §6）。

## 1. 设计约束

| 约束 | 值 | 说明 |
|---|---|---|
| 呈现坐标系 | **1080×1920**（9:16 竖屏） | 唯一权威呈现坐标系（`menu_shell::LOGICAL_W_I32/H_I32` = 1080/1920、`gfx::CANVAS_W/H`）。所有绘制原语入参都是 1080 空间坐标 |
| 业务逻辑网格 | 无像素绑定 | 移动/战斗/经济跑在 `ti_core` 的网格与状态上（`GridPos` 等），**不携带像素坐标**；逻辑与渲染分离（`08_render_arch.md` 契约 4） |
| 版式骨架 | 1080 现代栅格 | 标题条 h=96、外框 12px 边距（`chrome::wood_frame`）、底部提示行 y=1840；S17.2 起版面坐标 8pt 整除（`10_ui_redesign.md` §4 增量，`grid.*` 业务几何除外）；**不再有 16px 网格对齐约束**（176 时代遗留，已废） |
| 渲染路径 | 纯代码绘制 | 背景层 + 面板矩形 + 文本 + 进度条 + 代码棋子；**不依赖** PCX 精灵做定位（棋子/据点/BOSS 全走代码绘制） |
| 主地图 | 代码等距菱形格 + 代码棋子 | 棋盘 = `chrome::iso_cell` 按 `grid.*` 计算的菱形格（`terrain_atlas` legacy 瓦片已退出）；玩家/据点/BOSS = 代码色块 |
| 字号 | 正文/标题 TTF 44px | `menu_shell::FONT_SIZE_PX` = 44；位图字体回退 advance `GLYPH_ADVANCE` = 12 |
| 输入映射 | 不变 | `07_ui_flow.md` §3 键盘/触摸映射保持，只改版面不改交互逻辑 |
| 场景流 | 不变 | `07_ui_flow.md` §1 场景转换表保持，只改版面 |

## 2. 配色（羊皮纸/木纹/金色系）

设计基准：AI 概念图 `evidence/ui_concept/index_v2.png`（用户选定）+ 模型自审
`evidence/ui_concept/review_render_index.txt`、`review_all.txt`。

> **色值必须脚本换算，勿手算**：S15 首版手算色值全部错位，渲染出「橙粉塑料色」被模型
> 判为最廉价（见 `review_render_index.txt`）。用 `scripts/rgb565.py` 从 8-bit hex 换算
> R11G5B10（round 量化）。

| 角色 | hex | RGB565 | 常量（`chrome` theme 令牌） | 用途 |
|---|---|---|---|---|
| 深底 bg0 | `#0C0E16` | `0x0865` | `COLOR_PARCH_DARK` / `COLOR_BG` | 场景底色、标题条内块、bevel 凹陷 |
| 深底亮 bg1 | `#151827` | `0x10C9` | `COLOR_PARCH_LIGHT` | 背景顶部层次 |
| 描边 stroke | `#39415E` | `0x3A17` | `COLOR_WOOD` / `COLOR_MENU_BORDER` / `COLOR_TITLE_BAR` | 整屏外框、卡片细边、标题条外框 |
| 金高光 gold | `#FFC857` | `0xFE55` | `COLOR_GOLD` / `COLOR_LANDMARK` / `COLOR_TEXT_ON_WOOD` | 主强调、据点标记、标题墨 |
| 航海蓝 blue | `#4FC3F7` | `0x4E3D` | `COLOR_GOLD_GLOW` / `COLOR_GOLD_BRIGHT` / `COLOR_SELECTED` | 交互高亮、选中项底 |
| 卡片底 surface | `#1B2033` | `0x190C` | `COLOR_MENU_BG` | 菜单卡片面、正文/提示面板 |
| 卡片亮 bevel_hi | `#262C45` | `0x2171` | `COLOR_BEVEL_HI` | 卡片顶部/左侧（凸起） |
| 槽位底 info | `#45B7D8` | `0x45B6` | `COLOR_HILITE` | 血条/进度条槽底（`chrome::bar_slot`） |
| 副文字 text_mid | `#9AA3C0` | `0x9D30` | `COLOR_SECONDARY` | 状态/提示行次级文字（深底上 AA ≈7.1:1；置灰/锁定态复用本 token） |
| 正文墨 text_hi | `#F4F1E8` | `0xF7BA` | `chrome::COLOR_TEXT_ON_PARCH` | 深色底上的正文/提示面板墨字（QA005 三方对齐 = 代码/测试/文档） |
| 玩家绿 success | `#69D58C` | `0x6EA3` | `COLOR_PLAYER` | 玩家血条、玩家棋子 |
| BOSS 红 danger | `#FF6B5E` | `0xFB57` | `COLOR_DANGER` | BOSS 血条、BOSS 格 |

> **S17.1 航海夜·精修 v2**（`docs/process/10_ui_redesign.md` §3 令牌表为数值源）：底色加深拉层次、
> 金/蓝提亮、次级文字提亮达 AA；旧表（`#12141C`/`#F5B841`/`#1E88E5` 系）随 S17.2 作废。
> 对比度表与禁用态口径见 `10_ui_redesign.md` §3.1/§5。

**场景底色**（S15.6 B1 裁决①）：运行时画布底色 = `chrome` 的 `COLOR_PARCH_DARK` 深底
`#0C0E16`（`0x0865`，航海夜 v2 底色口径）；`scenes.json` 的 `bg` 键已删（运行时零消费，
旧 `65440` 浅米白与实绘深底相反，三方失配 QA029）。`render_intent` 清屏/密度判据底色
基准 = `COLOR_PARCH_DARK`（QA029 重锚）。

> **配色表只写一次**（本文），代码常量引用 ID 不复述（AGENTS.md 硬规则 6）。
> `scenes/chrome.rs` 另有一组**局部**呈现常量（如标题条/高亮色），属 chrome 内部实现，
> 不在本文事实源范围；其取值须与上表语义一致。

## 3. 字体

- 方案：开源 CJK 黑体（Noto Sans CJK / 思源黑体），TTF 加载器已支持
  （`ti_render::font::TtfFont`，ab_glyph 软件光栅化；S18.8a 起 ti-app 的 macroquad 字体包装
  `ttf_font.rs` 随 macroquad 壳一并退役，现行 TTF 口径唯一事实源 = `ti_render::font::TtfFont`）
- 1080 呈现层 TTF 字号 **44px**（`menu_shell::FONT_SIZE_PX`）；位图字体回退 advance 12px
  （`GLYPH_ADVANCE`），缺失字体不静默空白（回退并可诊断）
- 字体文件放 `assets/fonts/`，TTF 加载走 `TtfFont::from_bytes`（Bevy 壳）/ `load_ttf_from_path`（native）

## 4. 各场景版面（1080×1920）

> 全场景公共结构（`chrome::wood_frame` 12px 外框 + `chrome::title_bar`）：
> 外框 `y=0..12` 顶 / `y=1908..1920` 底 / `x=0..12` 左 / `x=1068..1080` 右；
> 标题条 = `(12,12,1056,h)` 木纹外框 + `(16,16,1048,h-8)` 羊皮纸内块 +
> `(20,20,8,h-16)` 金色 accent 条，`h` 走 `scenes.json` `title.h` = 96（7 场景同值，S17.2）。
>
> **标题文字的 `x` 不在 `scenes.json` 里**：`title` 只有 `y`/`h` 两个键。绘制链是
> `chrome::scene_shell` → `centered_title_x(host, title)` = `(LOGICAL_W_I32 -
> shell().title_text_width_px(title)) / 2`（96px 标题档按**当前文本实测宽度**居中）
> → `draw_title_text(host, title, cx, layout.title.y, text_on_wood)`。
> `title.y = 8` 的语义是**标题墨迹顶边锚点**（S17.2 8pt 整备，旧值 6）：`MenuShell::draw_ttf_row` 用
> `-ceil(px_bounds.min.y)` 作 y 偏移使字形墨迹顶边正好落在 `y`（位图回退路径则把
> `y` 直接当首行位图字顶边）。`title.h` 只喂给 `title_bar` 决定标题条高度，不决定文字位置。

> **S18.7d 例外（4 场景夜空版）**：select_role / select_map / message / saveload 版面已改走
> `scenes.json` `primitives`（夜空渐变 + 英雄面板 + 选项列 + 海平面 + z20 木框，同 index
> 视觉语言；`chrome::scene_shell` 不再被这 4 场景调用，标题条 `title_bar` 仅 main_map/vs
> 保留）。下表 §4.2/§4.5/§4.6 的坐标 = 新数据槽值（运行时内容仍按这些槽落位）。

### 4.1 index（主菜单）

| 元素 | 坐标（`scenes.json`） | 说明 |
|---|---|---|
| 标题 | `title {y:8, h:96}` | **无 `x` 键**（按文本宽度居中，见上）；标题条高 96（`y=12..108`，S17.2）；标题文字「金银岛」金墨 |
| 菜单列表 | `list {x:240, y:704, w:600, row_h:176, item_h:160, text_x:240, highlight_offset:0}` | 6 项（新游戏/载入游戏/读取存档/选项设置/帮助说明/离开游戏）；**S18.8b 贴图版式**（每行 = 纹章贴图 120px 居中 + 金色标签居中，行距 176）；首块顶 = **704**；末块底 = `704+5*176+160` = **1744**（尾行 1840 之上） | | 6 项（新游戏/载入游戏/读取存档/选项设置/帮助说明/离开游戏）；**紧凑列**（S18.6 暖金版式：居中窄列 600px、行高 88、卡高 72）；首块顶 = `960-40` = **920**；末块底 = `920+5*88+72` = **1432** |
| 图标列 | `icon {x:144}` | 菜单项图标左缘（S17.3 换 `ui_icons` 罗盘/宝箱/锚/齿轮/书/门） |
| 状态行 | `status {x:540, y:1840}` | 浅底条 + 深墨字（字体数/音效/模块数）；`status` 走 `extra` 字段；`x=540` 为屏心锚点 |
| 立绘/动态区 | `portrait_zones [{x:0,y:360,w:1080,h:1310}]` | **代码绘制动态实体**允许落区（QB102：收紧到实渲内容区 360..1670，无 legacy 立绘） |
| 仪式标题面板（v3） | `title_art {x:112, y:192, w:856, h:672}` | S18.6 暖金版：`ornate_frame` 精致木框（木色外框 + 金色内线 + 四角卷角 + 置顶指南针玫瑰），与标题/副标题同处 192..864 仪式区 |
| 星空氛围层（v3） | `ambient_art {x:0, y:0, w:1080, h:1920, center_x:50, center_y:52, exclude:12}` | v3.1 双写：`chrome::star_sky` 确定性 26 星（无 RNG，快照可复现），绘制夹 12px 外框内，z-order 在外壳之后/菜单之前（P-SoT-2）；飞轮 R1 ④-05/①-32：星位表 + 屏心净空参数（`center_x`/`center_y`/`exclude` 百分数）数据化 |
| 星空氛围层 · 星位表（v3） | `ambient_art.stars [{x:8, y:9}, {x:22, y:5}, {x:36, y:14}, {x:52, y:7}, {x:71, y:11}, {x:88, y:6}, {x:15, y:21}, {x:64, y:19}, {x:90, y:23}, {x:6, y:33}, {x:30, y:29}, {x:79, y:31}, {x:94, y:40}, {x:10, y:47}, {x:86, y:51}, {x:18, y:60}, {x:92, y:63}, {x:7, y:72}, {x:38, y:68}, {x:84, y:75}, {x:14, y:84}, {x:62, y:81}, {x:91, y:88}, {x:25, y:91}, {x:48, y:94}, {x:74, y:92}]` | 26 星（百分数 x/y；亮星 = `size:2` + 四向光晕，暗星 = `size:1`；飞轮 R1 ④-05：星位表 = 数据，渲染层只遍历） |
| 视觉重心锚（v3） | `focus_anchor {bbox:{x:108, y:192, w:864, h:672}}` | 场景叙事 DSL 重心区（`ui_schema/scene_manifest_v3.json` index `focus.bbox_pct` × (1080,1920)）；gate 第 22 项 `check_scene_narrative` N8 对账锚点 |

不变式：`920 ≥ 108`（首块不压标题条）、`1432 ≤ 1840`（末块不撞提示行）。


**原语表（`scenes.json` `index.primitives`，有序 = 绘制顺序；S18.7 通用原语集）**：

界面 = **通用原语**数组（`rect`/`text`/`icon`/`grad`/`glow`/`disc` 六种）；渲染层只认这六种，
**不认识**"菜单""木框"等场景语义（场景语义全在数据里）。渲染层契约由
`scripts/check_data_driven.py` 门禁守护（gate 第 24 项）。

| kind | 用途 | 关键参数 |
|---|---|---|
| `rect` | 实心矩形（夜空条带/木框/卡片/星点） | `rect` + `color`（theme 令牌名）+ `z`（层序，负值）+ `when`/`item`（选中态） |
| `text` | 文本（标题/副标题/菜单项） | `rect`（内居中）+ `tier`（title/body/hint）+ `text`/`text_key` |
| `icon` | 图标（`ui_icons.json` 注册表） | `params.icon`（图标名）+ `params.size` |
| `grad` | 垂直渐变 | `params.from_token`/`to_token`/`steps` |
| `glow` | 径向光晕 | `rect`（圆心 + 半径） |
| `disc` | 实心圆盘 | `rect`（圆心 + 半径） |

**声明式原语表（S18.6）**：index 场景界面元素 = `scenes.json` `index.primitives` 数组
（有序 = 绘制顺序；每项 = `kind` + `rect` + `param` + `color` 令牌名 + 文本）。
- **改版面/顺序/参数/配色 = 只改 `scenes.json`**（0 代码）；新增**原语种类**才需 1 处
  代码（`chrome::render_primitives_bg/fg` dispatcher 加一个 arm）——数据驱动边界。
- `color` 字段 = `theme` **令牌名**（如 `sky_navy`/`wood_warm`/`gold_warm`/`text_on_parch`）；
  渲染层零裸色值（`check_hardcode` 门禁）。
- z-order 两批：背景类原语（`night_sky`/`star_sky`/`moon_glow`/`ocean_horizon`）在
  `scene_shell` **之前**画，其余（`ornate_frame`/文本/`menu_items`）在其后。

### 4.2 select_role / select_map（选择类）

> S18.7d：两场景版面改走 `scenes.json` `primitives`（夜空 + 左英雄面板 + 右选项列 + 海平面 +
> z20 木框）；下表坐标 = 运行时内容数据槽（行名/简介/图标/选中高亮全在 `primitives`）。

| 元素 | select_role | select_map |
|---|---|---|
| 标题 | `title {x:296, y:48, w:440, h:96}`（S20.2：`x` = 左立绘列中心点，顶部金标题随立绘画布左移） | `title {x:352, y:48, w:400, h:96}`（S18.7d：`x` = 拼贴区中心点，避开右上月亮） |
| 列表 | `list {x:600, y:480, w:400, row_h:192, item_h:192, text_x:744, highlight_offset:0}` | `list {x:720, y:480, w:280, row_h:320, item_h:320, text_x:808, highlight_offset:0}` |
| 图标列 | `icon {x:584}`（行首 monogram 圆环 → JSON 原语；键留作数据槽） | `icon {x:744}`（折叠地图 → JSON 原语） |
| 首块顶 | `480-0` = **480** | **480** |
| 项数 × 末块底 | 5 角色卡 → `480+4*192+192` = **1440** | 3 地图卡（`MAP_COUNT=3`，QA139）→ `480+2*320+320` = **1440**（海平面带 1704 之上） |
| 锁定标记列 | — | `locked {text_x:920, inset:6, max_w:40, min_w:4, h:12}`（QA033 钳制公式不变：卡片右缘 = 1000；飞轮 R1 ③-13：几何参数组入 JSON） |
| BOSS 难度星（S19.5） | — | `stars {size:14, gap:6, inset:12, top:10, max:3, bevel_px:2}`（飞轮 R1 ③-14：星几何组入 JSON；`max` 仅几何上限，星数钳制规则在 `rules::progression`） |
| 立绘锚（S18.7d） | `hero_art {x:76, y:184, w:440, h:664}`（S18.20：立绘**贴图** 2:3 区） | —（地图拼贴区 = 重心区，chrome 全在 `primitives`） |
| 视觉重心锚（v3） | `focus_anchor {bbox:{x:76, y:130, w:356, h:534}}`（S18.20 = 立绘区） | `focus_anchor {bbox:{x:76, y:230, w:562, h:1190}}`（gate 第 22 项 `check_scene_narrative` N8 对账，manifest `focus.bbox_pct` × (1080,1920)） |
| 提示行 | `hint {x:540, y:1840}` | 同左（JSON 原语 tier hint） |
| 动态区 | `portrait_zones [{0,240,1080,1460}]` | `portrait_zones [{0,240,1080,960}]` |

不变式：首块顶 `480 ≥ 108`、末块底 `1440 ≤ 1704`（不越海平面带）、列表右缘 `≤ 1068`（木框内）。

### 4.3 main_map（主地图，代码等距菱形格 + 代码棋子）

> **S21.3（main_map v4「羊皮海图桌面」）起坐标单一事实源 = [`ui-redesign-v3/08_main_map_v4_spec.md`](../ui-redesign-v3/08_main_map_v4_spec.md)**
> （满屏 `chart_full` + 顶角双铜牌 + 岛只画陆/礁/宝 + 黄铜航线多形状节点牌 + 船玩家 + 舵轮 CTA；
> `world_canvas {43,250,994,826}` ↔ manifest `bbox_pct {4,13,92,43}`（N8），落码 S21.4~S21.7/S21.12）。
> 下表为 **v3/v2 现行值（回退口径）**，v4 上线后本表 §4.3 坐标随 `scenes.json` 同步替换。

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {y:8, h:96}`（无 `x` 键，居中算） | 「地图」标题 |
| HUD | `hud {x:80, y:152, w:920, h:128}` | 羊皮纸面板 + 描边细边（金字 + 持有量，两行 44px 字）；底 = **280**（S17.2 8pt） |
| 等距棋盘 | `grid {x0:540, y0:520, tile_w:25, tile_h:13}`（QB105 重锚：旧 y0=380/tile 24×12；业务几何非 8pt 版面令牌，S17 不动） | `chrome::iso_cell(row,col)` = `(x0+(col-row)*25, y0+(row+col)*13)`；`tile_w/h` 走 `extra_scalar` 的 `grid.*` 键 |
| HUD 三按钮 | `hud_trade {x:536, y:168, w:136, h:88}` / `hud_sell {x:704, y:168, w:136, h:88}` / `hud_build {x:856, y:168, w:136, h:88}` | QA149：HUD 三经济臂（点按 = 买商品 1 / 全卖商品 1 / 造 Castle，与键盘 T/Y/B 同语义，`main_map.rs` on_touch 命中分派 + `e2e_touch_economy_loop` 纯触摸判据；S17.2 8pt 整备：按钮宽 140→136、sell x 696→704、按钮 y 166→168） |
| 提示行 | `hint {x:540, y:1840}` | 方向键移动/据点事件 |
| 动态区 | `portrait_zones [{0,150,1080,1680}]` | 玩家/据点/BOSS 代码方块允许落区 |
| 世界画布（v3） | `world_canvas {x:43, y:192, w:1004, h:710}` | v3.1 双写：`chrome::world_canvas_frame` 装饰框 + `board_fit` 把 G 依赖等距棋盘**拟合进画布**（2:1 菱形、8pt 对齐、画布中心对齐）；v2 `grid.*` 保留为回退（键缺席即 v2 版面） |
| HUD 顶左角 · 金币（v3） | `hud_gold {x:80, y:80, w:320, h:96}` | P-Hades-1 HUD 边角化：金币面板（v3 模式不画 v2 HUD 条；v2 `hud`/三按钮键保留为数据 + 触摸回退，双写） |
| HUD 顶右角 · 持有量（v3） | `hud_player {x:680, y:80, w:320, h:96}` | P-Hades-1：持有量面板（原提案「玩家位」改显 `owned_fmt`，不造新 i18n 键） |
| 命令盘 · 贸易（v3） | `deck_trade {x:240, y:1600, w:200, h:80}` | P-UW4-3 底部命令盘：`chrome::deck_button`（面板 + 图标 + 28px 标签）；点按 = 买商品 1（与键盘 T / v2 `hud_trade` 同语义） |
| 命令盘 · 出售（v3） | `deck_sell {x:460, y:1600, w:200, h:80}` | 命令盘第 2 键 = 全卖商品 1（键盘 Y / v2 `hud_sell`） |
| 命令盘 · 建造（v3） | `deck_build {x:680, y:1600, w:200, h:80}` | 命令盘第 3 键 = 造 Castle（键盘 B / v2 `hud_build`） |
| 摇骰前进 · 主操作（S19） | `deck_spin {x:336, y:1328, w:400, h:120}` | S19 环岛赛道主操作：摇骰 → 沿赛道前进 N 格（`rules::board`，`progression.md` §环岛赛道）；面板 + 骰子/舵轮图标（`assets/ui/dice.png`/`helm.png`）+「摇骰前进」标签（`scene.main_map.hud.spin`）；点按 / 空格 = `GameInput::Spin`；8pt 栅格（x336=42×8、y1328=166×8）；S19.7 构图：骰子 `dice.png` (360,1352,72,72) + 舵轮 `helm.png` (444,1352,72,72) **相邻成组**（原 helm 在右端 844，中间 280px 空档像两张不相干的图），标签 (536,1328,190,120) |
| 航程条（S19.7） | `voyage_bar {x:120, y:1056, w:840, h:120}` | S19.7 D1：把环岛赛道**摊平成进度条**（v3.5 §3「空出来的下方空间 → 放航程条（进度条 + 目的地标记）」）：底板 `brass_dark` + 黄铜槽 + 每格一点（分区色，特殊格加亮）+ 玩家标记（`compass` 暖金）+ BOSS 标记（`sword` danger）；底板色**不用 `panel_warm`**（RGB565 (24,24,16) 与深海底图同色 ⇒ 隐形）；8pt 栅格（x120=15×8、y1056=132×8、w840=105×8、h120=15×8） |
| 落格提示条（S19.7） | 运行时绘制，中心 y=1520（面板 1484..1556） | S19.7 D3：提示条加 `brass_dark` 底板 + 黄铜边框（宽度按文字自适应，最小 240；`panel_warm` 在深海上隐形）；纵向落在摇骰按钮下沿 1448 与命令盘上沿 1600 之间；骰点提示「掷出 N」在 y=1296 |
| 画布实体比例（v3） | `canvas_entities {player:100, landmark:80, boss:96}` | 实体边长 = `board_fit.tw` × 百分比（100/80/96）：15 格图 tw=32 → 32/26/31；20 格图 tw=24 → 24/19/23；缺席回退 v2 `player.size` 等（QA106） |
| 金币光 FX 触发点（S18.5） | `fx_coin {x:900, y:140}` | HUD 金币显示区（飞轮 R1 ③-02 数据化：FX 坐标 = 版面数值，代码读键不裸写） |
| 视觉重心锚（v3） | `focus_anchor {bbox:{x:43, y:192, w:1004, h:710}}` | 场景叙事 DSL 重心区（`ui_schema/scene_manifest_v3.json` main_map `focus.bbox_pct{4,10,93,77}` × (1080,1920)）；gate 第 22 项 `check_scene_narrative` N8 对账锚点 |

不变式：HUD 底 `280 ≤ 520`（不与等距区 `y0=520` 重叠）；**实体 size ≤ 瓦片行高 `2*tile_h` = 26**（QA106：棋子/据点/BOSS 不越格覆盖邻居——player.size=26 为临界值）。

- 玩家棋子：`COLOR_PLAYER` 方块（玩家绿 success，§2 v2），中心落在 `iso_cell(row,col)`，
  边长走 `scenes.json` `player.size` = 26（QA030 数据驱动，QA106 重锚 32→26；键缺失回退 `tile*2/3` 公式）
- 据点：`COLOR_LANDMARK` 方块（金高光 gold，§2 v2），边长走 `scenes.json` `landmark.size` = 20
- BOSS 格：`COLOR_DANGER` 方块（BOSS 红 danger，§2 v2），边长走 `scenes.json` `boss.size` = 24
- **瓦片来源**：`terrain_atlas` legacy 瓦片已退出运行时，棋盘 100% 代码菱形格绘制
  （`main_map.rs` 无地形资源分支；`has_terrain_atlas()` 恒假）

### 4.4 vs（战斗）

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {y:8, h:96}`（无 `x` 键，居中算） | 「战斗」标题 |
| 玩家血条 | `player_hp {x:64, y:168, w:448, h:48}` | `chrome::bar_slot` 槽底 `COLOR_HILITE` + 前景血量 `COLOR_PLAYER`（玩家绿，§2 v2）；右缘 **512**（S17.2 8pt：64..512） |
| BOSS 血条 | `boss_hp {x:576, y:168, w:448, h:48}` | 槽底 + 前景 `COLOR_DANGER`（BOSS 红，§2 v2）；右缘 **1024 ≤ 1080**（S17.2：576..1024） |
| 回合数 | `rounds {x:540, y:416}` | 屏心文字（`x=540` = 屏心锚点；S17.3 数字用 label 档 32px） |
| 战斗结果 | `outcome {x:540, y:960}` | 胜/负 |
| 空手提示 | `empty {x:540, y:960}` | 无卡时提示。`empty` 与 `outcome` **同 y 但互斥**（空手持牌态 vs 战斗结算态，同帧只出现一个）；`y=960` 远在血条带 `168..216` 之下（QA140） |
| 战斗命中 FX 触发点（S18.5） | `fx_hit {x:540, y:880}` | 战斗场景角色区中心（飞轮 R1 ③-01 数据化：FX 坐标 = 版面数值，代码读键不裸写） |
| 提示行 | `hint {x:540, y:1840}` | 攻击/返回 |
| 动态区 | `portrait_zones [{0,170,1080,1680}]` | 玩家/BOSS 代码方块允许落区 |
| 战斗竞技区（v3） | `battle_arena {x:76, y:230, w:918, h:998}` | v3.1 双写：`chrome::battle_arena` 三层甲板阶梯 + 罗盘饰 + 饰线垫底（z-order 外壳之后、血条/回合文本之前）；血条带 `168..216` 在竞技区顶之上不重叠；8pt 豁免（v3 键，ui-redesign-v3/02 §10） |
| 视觉重心锚（v3） | `focus_anchor {bbox:{x:76, y:230, w:918, h:998}}` | 场景叙事 DSL 重心区（`ui_schema/scene_manifest_v3.json` vs `focus.bbox_pct` × (1080,1920)）；gate 第 22 项 `check_scene_narrative` N8 对账锚点 |

不变式：`player_hp.x+w ≤ 1080`、`boss_hp.x+w ≤ 1080`；**双条互斥**：`boss_hp.x - (player_hp.x+w) ≥ 40`（64..512 / 576..1024，gap=64，QA140/QB120）；`rounds`/`empty`/`outcome` 均不在血条带 `y=168..216` 内。

### 4.0 v4 面板语言（S20.2）

> 三屏（message / saveload / select_role）统一：**深棕木底**（`panel_wood` = RGB565 0x3943）
> + **黄铜细边**（`chrome::wood_panel`：brass 2px 四边 + brass_light 内上 1px + brass_dark 内下 1px）
> + **选中态 = 3px 黄铜边**（`chrome::selected_border`，此前是整块金色填充）。
> 底色令牌 `panel_wood` 替代 `panel_warm`（后者 RGB565 (24,24,16) ≈ 近黑 ⇒ 面板在夜空上像"挖了个洞"）。
> 面板边框统一由 `chrome::wood_panel` 出（代码侧同源 ⇒ 三屏语言一致，改一处全局生效）；
> 底色仍在 `scenes.json` 原语（`panel_wood`），符合"UI = 数据"。

### 4.5 message（消息/帮助）

> S18.7d：版面改走 `scenes.json` `message.primitives`（夜空 + 左立绘面板 + 右 text_panel
> + 海平面 + z20 木框；面板/金饰/分割线全 JSON）。下表坐标 = 运行时内容数据槽。

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {x:704, y:704, w:600, h:96}`（S20.2 v4：`x` = 卡片中心点，标题随面板按内容收缩下移；标题文本 = 动态 i18n `msg.<kind>`，代码按槽落位） | 顶 wood_warm 头带（原语 z4）内 |
| 卡片底板 | `panel {x:400, y:704, w:608, h:336}` | S20.2 v4 面板语言：深棕木底（`panel_wood`）+ 黄铜细边（`chrome::wood_panel`）|
| 正文区 | `body {x:408, y:768, w:584, h:176, clip_reserve:48, line_h:52, x_pad:4, edge_pad:8}`（S18.7d v3 text_panel 内区；面板 chrome 在 `primitives`；飞轮 R1 ②-07：衬边/行高/溢出保留数据化） | 多页文本折行（QA111） |
| 页点（QA144） | `page_dots {size:12, gap:36, dy:48, inner:8}` | 页点行 = `page.y - dy`；未翻 = 暗底 + `inner` 内框（飞轮 R1 ②-07：页点几何组数据化） |
| 页码 | `page {x:704, y:1008, w:0, h:0}`（S20.2 v4：随卡片收缩上移） | 页码/总页按 `page.x` 中心点排布（QA144 页点行 = `page.y - 48`） |
| 提示行 | `hint {x:540, y:1840}` | 切页/关闭（JSON 原语 tier hint） |
| 视觉重心锚（v3） | `focus_anchor {bbox:{x:400, y:230, w:605, h:1210}}` | gate `check_scene_narrative` N8 对账（manifest `focus.bbox_pct{37,12,56,63}` × (1080,1920)） |
| 动态区 | `portrait_zones [{0,220,1080,1680}]` | 代码绘制动态实体允许落区 |

> FIND-026 设计裁决（S18.7d 重锚，S20.2 v4 数值随面板收缩更新）：`body.h` 是**正文渲染区上界**
> （`body.y+body.h = 944 ≤ page.y = 1008`），**不是行数**；翻页由 `msg_page`/`msg_total` 驱动，
> `body.h` 仅作版面区间约束。

### 4.6 saveload（存读档）

> S18.7d：版面改走 `scenes.json` `saveload.primitives`（夜空 + 左立绘锚 + 中槽列表 +
> 右元数据栏 chrome + 海平面 + z20 木框）；光标高亮 = 原语 `when="selected"` 行。

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {x:600, y:48, w:640, h:96}`（S20.2：`x` = 左移后列表+元数据区中心点 280..920） | 「存读档」金标题（原语 tier title） |
| 槽位列表 | `list {x:280, y:240, w:440, row_h:256, item_h:224, text_x:312, highlight_offset:8, gold_x:792, summary_dy:52}` | 5 槽；首块顶 `240-8` = **232**；末块底 `240+4*256-8+224` = **1480**（海平面 1704 之上）；`summary_dy` = 第二行摘要相对主行文字偏移（飞轮 R1 ③-15 数据化） |
| 金币次列 | `list.gold_x` = 792（S18.7d v3 右元数据栏 840..1000 内 +32 衬边；元数据栏 chrome 在 `primitives`） | 每槽金币数（S17.3 数字 label 档） |
| 提示行 | `hint {x:540, y:1840}` | 选位/读写（JSON 原语 tier hint） |
| 视觉重心锚（v3） | `focus_anchor {bbox:{x:356, y:230, w:637, h:1210}}` | gate `check_scene_narrative` N8 对账（manifest `focus.bbox_pct{33,12,59,63}` × (1080,1920)，覆盖中列表 + 右元数据栏） |
| 动态区 | `portrait_zones [{0,240,1080,1450}]` | 代码绘制动态实体允许落区 |

## 5. 不动的部分

- `08_render_arch.md` 契约（分层/原语/z-order/坐标不变量）——渲染架构
- `07_render_pipeline.md` §1 渲染路径表（RGB565 默认 + HD RGBA + GPU）——上屏通路
- `07_ui_flow.md` §1 场景流 + §3 输入映射——交互逻辑
- `combat.md`/`economy.md`/`progression.md`/`persistence.md` 玩法规则——业务
- `ti_core::rules/*` 全部规则模块——逻辑层，与像素解耦

## 6. 实施范围（代码改动点）

**legacy 像素层退出运行时**（本轮渲染线核心，替代旧"坐标重排"实施范围）：

| 改动 | 文件 | 说明 |
|---|---|---|
| 删逐像素 PCX blit | `sprite_atlas.rs` | 删除 `blit_sprite`/`blit_sprite_rgba`（写帧的逐像素路径）；PCX/图集**解码**保留供诊断工具 |
| 删场景精灵上屏 | `game_app/render.rs` | `render`/`render_hd` 不再调 `blit_scene*` |
| 诊断 bin 去 blit | `bin/element_dump.rs` | 改 decode-only 几何诊断（不产像素） |
| 删 legacy 地形分支 | `terrain_atlas.rs` / `main_map.rs` | 主地图只走代码菱形格 + 代码棋子 |
| 移除坐标转换契约 | `render_pipeline.rs` | 删 legacy↔1080 缩放函数（无 legacy 像素即无转换需求） |
| 版面数据源 | `scenes.json` | 已 1080 化；**`scenes.json` 是机器权威**，本文 §4 表是它的**人读镜像**（由 `doccheck2::check_layout_doc_table_cross_check` 逐项对账，漂移即 FAIL） |
| 配色/字体常量 | `menu_shell.rs` / `chrome.rs` | 引用 §2/§3，代码常量不硬编码版面 |

> `terrain_atlas.rs` 若仍需**解析** legacy 地形数据供业务（网格尺寸/地图），解析保留，
> 仅删「写像素」分支；业务读网格与渲染解耦（契约 4）。

## 7. 可测试规则

| 规则 | 单测（`render_intent.rs` / `render_parity.rs`） |
|---|---|
| **各场景版面坐标 == `scenes.json` 实值**（1080 空间）。单测断言的是 `scenes.json` 的**实测值**（如 `row_h=300`、`hint.y=1840`、`title.h=90`），§4 表是该数据的人读镜像 | `index_layout_matches_doc` / `select_scenes_layout_matches_doc` / `main_map_layout_matches_doc` / `vs_layout_matches_doc` / `message_layout_matches_doc` / `saveload_layout_matches_doc` |
| 血条 x+w ≤ 1080、HUD 与等距区不重叠 | `vs_layout_matches_doc` / `main_map_layout_matches_doc` |
| 代码绘制动态实体落在 `portrait_zones` 声明区内 | `large_portraits_respect_layout_zones`（**重锚**：判据 = 代码棋子/据点矩形 ∈ 声明区，不再判 legacy 精灵像素） |
| 关键矩形渲染后有非背景像素（标题条/血条/HUD/正文） | `key_rects_have_pixels`（含 vs `bar_slot` 槽底 fill 断言） |
| 各场景文字区场景层有独立贡献像素（非全被背景淹没） | `scene_layer_contributes_text_region_pixels`（**重锚**：场景层 vs 背景 XOR，无 legacy 精灵层参与） |
| 场景渲染确定性 / 非空 / 场景区分 | `render_parity.rs`（`render_is_deterministic_per_scene` / `each_scene_renders_real_content` / `distinct_scenes_produce_distinct_frames`） |
| 呈现坐标系唯一（禁 legacy 像素/坐标泄漏） | `doccheck2` grep 不变量：`ti-platform`/`chrome`/`scenes` 禁字面 176/220 |
| **§4 表 == `scenes.json` 同名键**（本文不漂移） | `doccheck2::check_layout_doc_table_cross_check` 逐项对账本文 §4 表数值对与 `assets/data/scenes.json` |

> **legacy z 语义降级**：`e2e_render_alignment.rs` 原「.Map 文件序 z-order 逐像素」判据随
> legacy 像素层退出而失效，改为 `ti_formats` 层的**纯数据**断言（`.Map` 解析保留文件序，
> z 语义作为逆向事实留档，见 `legacy/docs/facts/`），不再作为像素门禁。

## 8. 验收

- 各场景 `element_dump`/几何自检：代码绘制元素无越界、无错乱（§4 版面）
- `render_snapshot` 7 场景基准 diff = 0（纯代码渲染器重生成基线）
- `visual_regression` 场景基准 diff = 0
- `render_intent` 全绿（重锚后）
- gate 全绿；台账登记渲染线任务后 verified
