# 界面布局重设计（全量，1080 现代版面）

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
| 版式骨架 | 1080 现代栅格 | 标题条 h=90、外框 12px 边距（`chrome::wood_frame`）、底部提示行 y=1860；**不再有 16px 网格对齐约束**（176 时代遗留，已废） |
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

| 角色 | hex | RGB565 | 常量（`menu_shell`） | 用途 |
|---|---|---|---|---|
| 深底 | `#12141C` | `0x10A3` | `COLOR_PARCH_DARK` | 场景底色、标题条内块 |
| 深底亮 | `#1E2230` | `0x2106` | `COLOR_PARCH_LIGHT` | 背景顶部层次 |
| 卡片描边 | `#3A4058` | `0x3A0B` | `COLOR_WOOD` | 整屏外框、标题条外框 |
| 金高光 | `#F5B841` | `0xF5A8` | `COLOR_GOLD` / `COLOR_LANDMARK` | 高光点缀、据点标记 |
| 航海蓝亮 | `#42A5F5` | `0x453E` | `COLOR_GOLD_GLOW` | 选中项内高光 |
| 航海蓝 | `#1E88E5` | `0x245C` | `COLOR_GOLD_BRIGHT` | 标题条 accent 条、选中项底 |
| 卡片底 | `#242838` | `0x2147` | `COLOR_MENU_BG` | 菜单卡片面、正文/提示面板 |
| 卡片描边 | `#3A4058` | `0x3A0B` | `COLOR_MENU_BORDER` | 卡片/面板细边 |
| 卡片亮 | `#30364A` | `0x31A9` | `COLOR_BEVEL_HI` | 卡片顶部/左侧（凸起） |
| 暗角 | `#0A0C12` | `0x0862` | `COLOR_BEVEL_SHADOW` | 卡片底部/右侧（凹陷） |
| 槽位底 | `#2A3350` | `0x2967` | `COLOR_HILITE` | 血条/进度条槽底（`chrome::bar_slot`） |
| 副文字 | — | `0x8C93` | `COLOR_SECONDARY` | 状态/提示行次级文字 |
| 正文白字 | — | `0xFFFF` | `chrome::COLOR_TEXT_ON_PARCH` | 深色底上的正文/提示面板白字（S15 深色改版；QA005 三方对齐 = 代码/测试/文档） |
| 标题象牙金 | `#FFF0C0` | `0xFF77` | `chrome::COLOR_TEXT_ON_WOOD` | 标题条文字 |
| 玩家青绿 | `#26A69A` | `0x2D33` | `COLOR_PLAYER` | 玩家血条、玩家棋子 |
| BOSS 血红 | `#EF5350` | `0xEAAA` | `COLOR_DANGER` | BOSS 血条、BOSS 格 |

**场景底色**（S15.6 B1 裁决①）：运行时画布底色 = `chrome` 的 `COLOR_PARCH_DARK` 深底
`#12141C`（`0x10A3`，航海图深底口径）；`scenes.json` 的 `bg` 键已删（运行时零消费，
旧 `65440` 浅米白与实绘深底相反，三方失配 QA029）。`render_intent` 清屏/密度判据底色
基准 = `COLOR_PARCH_DARK`（QA029 重锚）。

> **配色表只写一次**（本文），代码常量引用 ID 不复述（AGENTS.md 硬规则 6）。
> `scenes/chrome.rs` 另有一组**局部**呈现常量（如标题条/高亮色），属 chrome 内部实现，
> 不在本文事实源范围；其取值须与上表语义一致。

## 3. 字体

- 方案：开源 CJK 黑体（Noto Sans CJK / 思源黑体），TTF 加载器已支持（`ttf_font.rs`）
- 1080 呈现层 TTF 字号 **44px**（`menu_shell::FONT_SIZE_PX`）；位图字体回退 advance 12px
  （`GLYPH_ADVANCE`），缺失字体不静默空白（回退并可诊断）
- 字体文件放 `assets/fonts/`，TTF 加载走 `load_ttf_from_path`（已有）

## 4. 各场景版面（1080×1920）

> 全场景公共结构（`chrome::wood_frame` 12px 外框 + `chrome::title_bar`）：
> 外框 `y=0..12` 顶 / `y=1908..1920` 底 / `x=0..12` 左 / `x=1068..1080` 右；
> 标题条 = `(12,12,1056,90)` 木纹外框 + `(16,16,1048,82)` 羊皮纸内块 +
> `(20,20,8,74)` 金色 accent 条。标题文字落在 `scenes.json` `title` = `(340,40,h=90)`。

### 4.1 index（主菜单）

| 元素 | 坐标（`scenes.json`） | 说明 |
|---|---|---|
| 标题 | `title {x:340, y:40, h:90}` | 标题条高 90（`y=12..102`）；标题文字「金银岛」象牙金墨 |
| 菜单列表 | `list {x:90, y:400, w:900, row_h:220, item_h:170, text_x:140, highlight_offset:40}` | 6 项（新游戏/载入游戏/读取存档/选项设置/帮助说明/离开游戏）；首块顶 = `400-40` = **360**；末块底 = `360+5*220+170` = **1630**（QB101：6 行重分布 260..1700 带，空带收窄至 1630..1858） |
| 状态行 | `status {x:80, y:1860}` | 浅底条 + 深墨字（字体数/音效/模块数）；`status` 走 `extra` 字段 |
| 立绘/动态区 | `portrait_zones [{x:0,y:360,w:1080,h:1270}]` | **代码绘制动态实体**允许落区（QB102：收紧到实渲内容区 360..1630，无 legacy 立绘） |

不变式：`360 ≥ 102`（首块不压标题条）、`1630 ≤ 1858`（末块不撞提示行）。

### 4.2 select_role / select_map（选择类）

| 元素 | select_role | select_map |
|---|---|---|
| 标题 | `title {x:340, y:40, h:90}` | 同左 |
| 列表 | `list {x:90, y:240, w:900, row_h:270, item_h:230, text_x:140, highlight_offset:40}` | `list {x:90, y:240, w:900, row_h:340, item_h:280, text_x:140, highlight_offset:40}` |
| 首块顶 | `240-40` = 200 | 200 |
| 项数 × 末块底 | 5 角色卡 → `200+4*270+230` = **1510** | 3 地图卡（`MAP_COUNT=3`，QA139 三方分叉销账）→ `200+2*340+280` = **1160**（QB103：3 卡撑满，空带收窄至 1160..1858） |
| 提示行 | `hint {x:80, y:1860}` | 同左 |
| 动态区 | `portrait_zones [{0,240,1080,1560}]` | 同左 |

不变式：`200 ≥ 102`（不压标题条）、末块底 `≤ 1858`。

### 4.3 main_map（主地图，代码等距菱形格 + 代码棋子）

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {x:340, y:40, h:90}` | 「地图」标题 |
| HUD | `hud {x:80, y:150, w:920, h:120}` | 羊皮纸面板 + 木纹细边（金字 + 持有量，两行 44px 字）；底 = **270** |
| 等距棋盘 | `grid {x0:540, y0:520, tile_w:25, tile_h:13}`（QB105 重锚：旧 y0=380/tile 24×12） | `chrome::iso_cell(row,col)` = `(x0+(col-row)*25, y0+(row+col)*13)`；`tile_w/h` 走 `extra_scalar` 的 `grid.*` 键 |
| HUD 三按钮 | `hud_trade {x:536, y:166, w:140, h:88}` / `hud_sell {x:696, y:166, w:140, h:88}` / `hud_build {x:856, y:166, w:140, h:88}` | QA149：HUD 三经济臂（点按 = 买商品 1 / 全卖商品 1 / 造 Castle，与键盘 T/Y/B 同语义，`main_map.rs` on_touch 命中分派 + `e2e_touch_economy_loop` 纯触摸判据；面板无标签，标签归后续批次） |
| 提示行 | `hint {x:80, y:1860}` | 方向键移动/据点事件 |
| 动态区 | `portrait_zones [{0,150,1080,1680}]` | 玩家/据点/BOSS 代码方块允许落区 |

不变式：HUD 底 `270 ≤ 520`（不与等距区 `y0=520` 重叠）；**实体 size ≤ 瓦片行高 `2*tile_h` = 26**（QA106：棋子/据点/BOSS 不越格覆盖邻居——player.size=26 为临界值）。

- 玩家棋子：`COLOR_PLAYER` 方块（青绿，§2 L52），中心落在 `iso_cell(row,col)`，
  边长走 `scenes.json` `player.size` = 26（QA030 数据驱动，QA106 重锚 32→26；键缺失回退 `tile*2/3` 公式）
- 据点：`COLOR_LANDMARK` 方块（金高光，§2 L41），边长走 `scenes.json` `landmark.size` = 20
- BOSS 格：`COLOR_DANGER` 方块（血红，§2 L53），边长走 `scenes.json` `boss.size` = 24
- **瓦片来源**：`terrain_atlas` legacy 瓦片已退出运行时，棋盘 100% 代码菱形格绘制
  （`main_map.rs` 无地形资源分支；`has_terrain_atlas()` 恒假）

### 4.4 vs（战斗）

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {x:340, y:40, h:90}` | 「战斗」标题 |
| 玩家血条 | `player_hp {x:60, y:170, w:440, h:48}` | `chrome::bar_slot` 槽底 `COLOR_HILITE` + 前景血量 `COLOR_PLAYER`（玩家青绿，§2 L52，S15.6 QA038 归位）；右缘 **500**（QB120 解重叠：60..500） |
| BOSS 血条 | `boss_hp {x:580, y:170, w:440, h:48}` | 槽底 + 前景 `COLOR_DANGER`（BOSS 血红，§2 L53）；右缘 **1020 ≤ 1080**（QB120：580..1020） |
| 回合数 | `rounds {x:480, y:260}` | 文字居中 |
| 战斗结果 | `outcome {x:480, y:1400}` | 胜/负 |
| 空手提示 | `empty {x:480, y:960}` | 无卡时提示（QA140：`y=960` 挪出血条带 170..218，不与双条/「击键」撞行） |
| 提示行 | `hint {x:80, y:1860}` | 攻击/返回 |

不变式：`player_hp.x+w ≤ 1080`、`boss_hp.x+w ≤ 1080`；**双条互斥**：`boss_hp.x - (player_hp.x+w) ≥ 40`（60..500 / 580..1020，gap=80，QA140/QB120）；`empty`/`outcome` 不在血条带 `y=170..218` 内。

### 4.5 message（消息/帮助）

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {x:340, y:40, h:90}` | 标题 key = `msg.<kind>` |
| 正文区 | `body {x:90, y:220, w:900, h:1480}` | 羊皮纸浅块 + 木纹细边 + bevel，多页文本；底 = **1700** |
| 页码 | `page {x:480, y:1720}` | 页码/总页 |
| 提示行 | `hint {x:80, y:1860}` | 切页/关闭 |

> FIND-026 设计裁决：`body.h=1480` 是**正文渲染区上界**（`body.y+body.h = 1700 ≤ page.y = 1720`），
> **不是行数**；翻页由 `msg_page`/`msg_total` 驱动，`body.h` 仅作版面区间约束。

### 4.6 saveload（存读档）

| 元素 | 坐标 | 说明 |
|---|---|---|
| 标题 | `title {x:340, y:40, h:90}` | 「存读档」 |
| 槽位列表 | `list {x:90, y:240, w:900, row_h:250, item_h:210, text_x:140, highlight_offset:40, gold_x:820}`（QA105 重锚：旧 920 贴右缘） | 5 槽；首块顶 200；末块底 `200+4*250+210` = **1410** |
| 金币次列 | `list.gold_x` = 820 | 每槽金币数右对齐列 |
| 提示行 | `hint {x:80, y:1860}` | 选位/读写 |

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
| 删场景精灵上屏 | `game_app.rs` | `render`/`render_hd` 不再调 `blit_scene*` |
| 诊断 bin 去 blit | `bin/element_dump.rs` | 改 decode-only 几何诊断（不产像素） |
| 删 legacy 地形分支 | `terrain_atlas.rs` / `main_map.rs` | 主地图只走代码菱形格 + 代码棋子 |
| 移除坐标转换契约 | `render_pipeline.rs` | 删 legacy↔1080 缩放函数（无 legacy 像素即无转换需求） |
| 版面数据源 | `scenes.json` | 已 1080 化；本文 §4 表 = 其同值（单一事实源） |
| 配色/字体常量 | `menu_shell.rs` / `chrome.rs` | 引用 §2/§3，代码常量不硬编码版面 |

> `terrain_atlas.rs` 若仍需**解析** legacy 地形数据供业务（网格尺寸/地图），解析保留，
> 仅删「写像素」分支；业务读网格与渲染解耦（契约 4）。

## 7. 可测试规则

| 规则 | 单测（`render_intent.rs` / `render_parity.rs`） |
|---|---|
| 各场景版面坐标 = §4 表值（1080 空间） | `index_layout_matches_doc` / `select_scenes_layout_matches_doc` / `main_map_layout_matches_doc` / `vs_layout_matches_doc` / `message_layout_matches_doc` / `saveload_layout_matches_doc` |
| 血条 x+w ≤ 1080、HUD 与等距区不重叠 | `vs_layout_matches_doc` / `main_map_layout_matches_doc` |
| 代码绘制动态实体落在 `portrait_zones` 声明区内 | `large_portraits_respect_layout_zones`（**重锚**：判据 = 代码棋子/据点矩形 ∈ 声明区，不再判 legacy 精灵像素） |
| 关键矩形渲染后有非背景像素（标题条/血条/HUD/正文） | `key_rects_have_pixels`（含 vs `bar_slot` 槽底 fill 断言） |
| 各场景文字区场景层有独立贡献像素（非全被背景淹没） | `scene_layer_contributes_text_region_pixels`（**重锚**：场景层 vs 背景 XOR，无 legacy 精灵层参与） |
| 场景渲染确定性 / 非空 / 场景区分 | `render_parity.rs`（`render_is_deterministic_per_scene` / `each_scene_renders_real_content` / `distinct_scenes_produce_distinct_frames`） |
| 呈现坐标系唯一（禁 legacy 像素/坐标泄漏） | `doccheck2` grep 不变量：`ti-platform`/`chrome`/`scenes` 禁字面 176/220 |

> **legacy z 语义降级**：`e2e_render_alignment.rs` 原「.Map 文件序 z-order 逐像素」判据随
> legacy 像素层退出而失效，改为 `ti_formats` 层的**纯数据**断言（`.Map` 解析保留文件序，
> z 语义作为逆向事实留档，见 `legacy/docs/facts/`），不再作为像素门禁。

## 8. 验收

- 各场景 `element_dump`/几何自检：代码绘制元素无越界、无错乱（§4 版面）
- `render_snapshot` 7 场景基准 diff = 0（纯代码渲染器重生成基线）
- `visual_regression` 场景基准 diff = 0
- `render_intent` 全绿（重锚后）
- gate 全绿；台账登记渲染线任务后 verified
