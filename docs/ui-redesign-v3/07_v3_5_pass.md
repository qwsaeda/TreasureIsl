# 07 · v3.5 界面重设计（真游戏化）：地形数据 + 暖色板 + 画布贴合 + 实体图标化

> 触发：S18.7d~h 后七场景已 **100% 数据驱动**（Bevy ≈ CPU，画面一致），
> 但**画面仍是 v2 的弱设计** → 问题不在分离/渲染，在**设计数据本身**。
> 本篇 = 把"设计数据"重做成真游戏级；每条都落到 `scenes.json` / `theme.json` /
> `balance.json`（代码只留 dispatch）。

## 0. 现状取证（勿凭记忆）

| 症状 | 取证 | 影响 |
|---|---|---|
| 棋盘=平铺单色菱形 | 全库无 per-cell 地形类型；`main_map.rs:376` 瓦片恒定 `COLOR_MENU_BG`(0x190C) | 世界画布（76% 屏）读不出"地图" |
| 中景冷蓝塑料感 | 棋盘/HUD/命令盘用 v2 冷令牌 menu_bg 0x190C / bevel_hi 0x2171 / menu_border 0x3A17 / parch_dark 0x0865 | 暖金语言只覆盖天空+木框，中景割裂 |
| 画布 65% 空区 | `world_canvas` 1004×1478；G=20 时 board_fit tw=24/th=13 → 棋盘 ≈960×520 | v3 提案"杀死区"未达成 |
| 实体=方块+小字 | 玩家/据点/BOSS = 纯色方块 + 1px bevel | 无角色感 |
| 提示行压命令盘 | Bevy 文本锚点与 CPU 差 18px（实测，见 11_bevy_migration §S18.7b 实测） | 交互区重叠 |

## 1. 地形数据（**最高杠杆**：海陆宝藏 → 棋盘成地图）

- `balance.json` `selection.maps[]` 增 `terrain`：**确定性生成**（seed = 地图 id 哈希）
  的每格地形，枚举：`sea` / `land` / `treasure` / `reef`。
  - 生成规则：岛 = 以地图中心为心的**椭圆衰减** + 伪随机扰动（固定 seed → 快照可复现）；
    `treasure` = BOSS 格 + 2 随机陆格；`reef` = 岛缘一圈。
  - 存储：按行字符串（`"SLT"` 之类，`S`=sea `L`=land `T`=treasure `R`=reef）→ 体积小、可读、可 diff。
- 瓦片取色：**地形 → 配色令牌**（新增暖色板，见 §2），`main_map.rs` 取色 1 行：
  `let c = theme.by_name(terrain_color(terrain)).unwrap_or(...)`。
- 效果：世界画布从"蓝菱形"变"航海图"（深蓝海 / 暖沙陆 / 金宝藏 / 礁石描边）。

## 2. 暖色板扩充（`theme.rs` DEFAULT + `assets/data/theme.json` 同值改）

新增（暖，冷色保留给深底与阴影）：
`sea_deep` 0x0846 / `sea_mid` 0x0C6B / `sand_warm` 0xB565 / `sand_light` 0xDDC7 /
`treasure_gold` 0xFEA0 / `brass` 0xA508 / `brass_light` 0xC6AE / `ink_warm` 0x18C3。

映射：HUD/命令盘/画布框 → brass 系；陆块 → sand 系；海块 → sea 系；宝藏 → treasure_gold；
文字 → ink_warm / parch_light。**棋盘/HUD 不再出现冷蓝 menu_bg/bevel_hi**。

## 3. 画布贴合棋盘（杀空区）

- `world_canvas` 高度改 **G 依赖**：`board_fit` 已算出 tw/th → 画布高 = 棋盘外接高 + 上下留白
  （各 96px），键值随 G 变 → 键不再是固定矩形 → **改 `board_fit` 输出**（v3 键，DSL 管理）。
- manifest `main_map.focus.bbox_pct` 同步收窄（N8 对账要求 scenes.json ≡ manifest）。
- 空出来的下方空间 → 放**航程条**（进度条 + 目的地标记）或直接收进命令盘上沿（视 §5）。

## 4. 实体图标化（占位方块 → 角色感）

- 玩家 = 船、据点 = 灯塔（beacon）、BOSS = 剑/骷髅（sword）—— `ui_icons.json` 已有
  compass/castle/beacon/sword/flag；用 `icon` 原语（数据）或 `icons::icon`（代码）绘制，
  取代纯色方块；方块退为**底座**（1px bevel 描边）。
- 实体标签（你/据/BOSS）字号 28 → 32（label 档）+ 描边/阴影提升可读性。

## 5. HUD / 命令盘（黄铜仪表盘感）

- HUD 角块 = 黄铜渐变（`grad` from brass_light → brass）+ 金饰线 2px + 圆角感（bevel 高光/阴影）；
  金币块加 coin 图标、持有块加 chest 图标（数据 `icon` 原语）。
- 命令盘 3 键放大 200×80 → 240×96，选中态金底 + 图标黑，标签 gold_warm。
- 提示行移到命令盘**下方**并加 32px 净空（修 §0 重叠）。

## 6. 顺带修的既有缺陷

- Bevy 文本锚点与 CPU 差 18px → 统一为 CPU 语义（文本中心 + 半字形高补偿）。
- `glmapm0N.Map` 已修（海图一/二/三）；继续清 i18n 孤儿 key（WARN 级）。

## 7. 执行顺序（每步 = 台账任务 + 门禁 + 基线重生成）

1. **S18.9** 地形数据（balance.json 生成器 + 瓦片取色）→ 世界画布成"地图"（最大观感收益）
2. **S18.10** 暖色板扩充 + 中景换色（theme.rs/theme.json + scenes.json + 代码取色）
3. **S18.11** 画布贴合（board_fit + manifest/N8 同步）
4. **S18.12** 实体图标化 + HUD/命令盘黄铜化 + 提示行净空
5. **S18.13** Bevy 文本锚点归零（CPU↔Bevy 像素 diff 归零 → S18.7b 基线源切换解锁）

**不变式**：全部改动走数据（`balance.json` / `theme.json` / `scenes.json`），代码只加
dispatch；每步基线重生成 + 目视验收 + gate 全绿；地形生成必须**确定性**（seed 固定）。

## 8. 实现记录（S18.9 已落地）

- **数据落点**：`assets/data/terrain.json`（渲染侧数据，与 `scenes.json` 同级）=
  `{schema_version, legend: {码 → {name, token, edge, alt}}, maps: {map_id → {rows: [...]}}}`；
  业务表 `balance.json` **不动**（地形不参与规则，海陆不设阻挡）。
- **生成规则**（一次性脚本生成、数据已落库，运行时不重算）：seed = 地图 id 的 sha256 前 8 位
  → 中心衰减椭圆岛（半径 0.42×grid）+ 确定性抖动（LCG，抖动幅 0.35）→ 岛缘环 = `reef`
  → BOSS 格 + 2 个陆格 = `treasure` → 玩家起点强制 `sea`（船从海面起航）。
  ⇒ 同一地图永远同一地形（基线/快照可复现）。
- **图例 → 配色**（新增 6 令牌，theme.rs ↔ theme.json 双写 + `theme_json` 测试锁一致）：
  `sea_deep 0x108A` / `sea_shallow 0x1A30` / `reef 0x4BCD` /
  `land_sand 0xFD4D` / `land_sand_light 0xFE51` / `land_sand_dark 0xD4CA`；
  `treasure` 复用既有 `gold_warm`。
- **渲染侧**：`ti_core::data::terrain()`（新 `data/terrain.rs`，`OnceLock` + 硬错误不 fallback）
  → `main_map.rs` 瓦片循环按格取色（`token` 底色 + `edge` 顶棱；陆地按 (r+c) 奇偶用 `alt` 交错）；
  地形表缺失/越界 = 回退 v2 单色（不 panic、不空图）。
- **门禁**：`doccheck2` 新增第 26 项 `check_terrain_data`（行数/行宽 == balance grid_h/grid_w、
  地形码 ⊆ legend、图例令牌 ∈ theme palette）。
- **效果**：主地图从"单色冷蓝菱形"变为**航海图**（深蓝海 + 岛缘浅礁 + 暖沙岛 + 金色宝藏格）。

## 9. 实现记录（S18.10 已落地：暖色板铺中景）

- **新增 4 令牌**（theme.rs ↔ theme.json 双写 + `theme_json` 测试锁一致）：
  `panel_warm 0x18C2`（深暖面板底）/ `brass 0x9365`（描边+甲板中层）/
  `brass_light 0xCE4A`（凸边/甲板亮层）/ `brass_dark 0x51E4`（凹边/槽底/甲板暗层）。
- **数据侧换色**（`scenes.json` 7 场景 306 处原语色令牌）：
  `menu_bg→panel_warm`、`menu_border→brass`、`bevel_hi→brass_light`、
  `bevel_shadow→brass_dark`、`wood→wood_warm`、`gold→gold_warm`、
  `gold_bright→gold_warm`、`hilite→brass_dark`；
  星点（3–5px 小方块）`parch_light→moon_pale`（夜空星更亮）；vs 竞技区底层甲板 → `brass_dark`
  （形成 暗→亮 的暖色三层阶梯）。
- **HUD 图标**（数据新增）：`hud_gold` 面板右端 `coin`、`hud_player` 面板右端 `chest`（金色 48px）。
- **效果**：HUD / 命令盘 / 画布框 / 英雄面板 / 竞技区甲板全部脱离 v2 冷蓝塑料感，
  统一到"夜空 + 暖金黄铜"语言；主地图读作**黄铜航海仪表盘**。

## 10. 实现记录（S18.11 已落地：画布贴合棋盘）

- `world_canvas` {43,192,1004,**1478**} → {43,192,1004,**710**}：画布收到 **G=20 棋盘外接框**
  （≈960×480 + 上下留白）→ 画布不再是大空框；`focus_anchor` 同步 + manifest
  `main_map.focus.bbox_pct` h 10%→**37%**（N8 对账 round(37%×19.2)=710 ✓，`check_scene_narrative` 0 failure）。
- **图表经纬网**：画布内 7 竖 + 4 横 1px 暖暗线（`brass_dark`）→ 棋盘压其上，露在四角 = 图表纸质感。
- **底部海平面**：与其余 6 场景一致（`sky_horizon` 带 1704..1920 + 5 条 `moon_pale` 波光）；
  命令盘 3 键上移 1760→**1600**（海平面之上）；提示行 1808→**1648**（海平面之上，不与波光重叠）。
- 视觉结果：主地图 = **夜海上的黄铜航海图**（HUD 角板 / 图纸经纬网 / 岛屿地形 / 宝藏格 /
  命令盘 / 海平线），不再是"大空框里浮一块蓝菱形"。

## 11. 实现记录（S18.12 已落地：实体图标化 + 海图装饰 + 提示行净空）

- **实体图标化**（代码 + `ui_icons.json` 注册表，`chrome::entity_icon` 新 helper）：
  玩家 = `flag`（金色 `gold_warm`）/ 据点 = `beacon`（`moon_pale` 灯塔）/ BOSS = `sword`（`danger`）；
  图标按注册表 **48px 原生尺寸**居中于实体盒（实体盒 = 底座，仍带 1px bevel）。
  底色方块保留（颜色语义 = 玩家/据点/BOSS），图标叠加其上。
- **海图装饰**（`scenes.json` 数据）：画布与命令盘之间的空白区补 **罗盘玫瑰**（金十字 + 8 方位点，
  72×72）+ 3 段**洋流**细线（`brass_dark`）。
- **提示行净空**（5 场景数据）：`tier=hint` 原语 y 1808 → **1648**（海平面带 1704 之上），
  不再与 `moon_pale` 波光重叠。
- 注：`ui_icons.json` `bindings` 为**严格结构**（未知键会整体解析失败）→ 实体图标名走
  代码字面量（与 `select_role`/`select_map` 既有模式一致），不新增 bindings 键。
