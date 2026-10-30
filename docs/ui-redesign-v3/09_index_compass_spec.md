# 09 · index 首页重设计「罗盘环标题页」版面与交互规格（S22.0 定案）

> **本文件 = index 首页重设计的版面 / 交互 / 资产接线单一事实源**（坐标 1080 坐标系）。
> `scenes.json` index 块、`scene_manifest_v3.json` index 段、`index.rs` / `chrome.rs` 画法**都必须**对齐本表。
>
> 触发：用户 2026-10-23「首页界面是个屎」→ 选 **C 方向「游戏化罗盘导航」** + **羊皮海图底（与 main_map v4 统一主题）**
> （question 工具拍板留痕）。概念稿 = `evidence/ui_concept/index_compass_layout.png`（版面线图，`ui_layout_sheet.py`）
> + `evidence/ui_concept/index_compass_concept.png`（AI 氛围参考，gitignored，仅参考不入仓）。

## 0. 设计意图（C 罗盘环 + 羊皮海图统一）

- 现状首页 = 稀疏**单列** icon+字竖排 + 大片夜空留白 + 图标糊（④-04 前的代码矢量），无构图重心、无游戏感。
- 重做 = **一张摊开的羊皮海图满屏**（`chart_full`，与 main_map v4 同底同主题，玩家从首页进主地图不跳戏）：
  - **中心 = 黄铜舵轮**（`helm.png` 264×264，**空舵轮**：轮毂不放 logo 字，「金银岛」由上方木牌承载）；
  - **6 项菜单 = 罗盘环 6 方向位的铜质圆形徽章（medallion）**：`compass_ring` 一圈，每枚 = 圆形铜徽章底
    （`medallion_ring.png`）+ 白蒙版图标（④-04 的 14 白蒙版，按 `menu_index` 映射）+ gold tint + 标签；
  - 纵向节奏：**标题木牌（顶）→ 罗盘环（主焦点，中心舵轮 + 6 徽章）→ 版本签（底）**。

## 1. 层结构（z 升序 = 调用序，`08_render_arch.md` 契约 1）

| z | 内容 | 载体 |
|---|---|---|
| 2 | 满屏羊皮海图 `chart_full`（0,0,1080,1920） | primitive `image`（资产，与 main_map 同底） |
| 3 | 标题木牌 `title_plaque`（金银岛 logo，沿用现木牌图） | primitive `image` + `text` `scene.index.title` |
| 5 | 罗盘环：中心舵轮（`helm`）+ 6 铜徽章（`medallion_ring` + 白蒙版图标 tint）+ 标签 | primitive `image`（`helm`/`medallion_ring`/`icon_*`）+ `text`（标签） |
| 9 | 版本签条（`scene.index.footer`） | primitive `rect`/`text`（最上层） |

> index 无运行时动态层（不像 main_map 有棋盘/赛道），全部走 JSON 原语 + 文本；命中区在 `index.rs` on_touch。

## 2. 逐元素坐标表（1080×1920）

> **径向键 8pt 豁免**：`compass_ring` / `helm` / 6 枚 `med_*`（罗盘环 6 方向位）= 径向布局键，
> 与 main_map `world_canvas`（v3 叙事键）**同口径 8pt 豁免**（`ui_8pt_walk` 豁免表登记），其余键须 8 整除。

| 元素 | 键 | rect `{x,y,w,h}` | z | 载体 | 说明 |
|---|---|---|---|---|---|
| 满屏羊皮海图 | （z2） | `{0,0,1080,1920}` | 2 | `image` asset=`chart_full` | 与 main_map 同底；替代现 `bg_night_sea` 夜海 |
| 标题木牌 | `title_plaque` | `{160,128,760,320}` | 3 | `image` asset=`title_art`（现木牌图）+ `text` `scene.index.title`/`.subtitle` | 沿用现木牌，上移；760/320 8 整除 ✓（160=20·8 / 128=16·8 / 760=95·8 / 320=40·8） |
| **罗盘环焦点** | `compass_ring` | `{65,500,950,941}`（N8 锚点，见 §5） | 5 | 无（N8 focus bbox，与 manifest 对账） | 环中心 `(540, 970)`；径向键 8pt 豁免 |
| 中心舵轮 | `helm` | `{408,838,264,264}`（中心 540,970） | 5 | `image` asset=`helm` | **空舵轮**（name 由 `title_plaque` 承载）；径向键 8pt 豁免 |
| 徽章·新游戏 | `m0_newgame` | `{468,498,144,144}` | 5 | `image` `medallion_ring` + 白蒙版 `icon_compass` gold tint + `text` | 环上 -90°（顶） |
| 徽章·载入游戏 | `m1_load` | `{814,698,144,144}` | 5 | 同上 + `icon_anchor` | -30°（上右） |
| 徽章·读取存档 | `m2_save` | `{814,1098,144,144}` | 5 | 同上 + `icon_chest` | +30°（下右） |
| 徽章·选项设置 | `m3_options` | `{468,1298,144,144}` | 5 | 同上 + `icon_gear` | +90°（底） |
| 徽章·帮助说明 | `m4_help` | `{122,1098,144,144}` | 5 | 同上 + `icon_book` | +150°（下左） |
| 徽章·离开游戏 | `m5_quit` | `{122,698,144,144}` | 5 | 同上 + `icon_door` | +210°（上左） |
| 版本签 | `footer` | `{344,1816,392,64}` | 9 | `rect`（`brass_dark`）+ `text` `scene.index.footer` | 344=43·8 / 1816=227·8 / 392=49·8 / 64=8·8 ✓ |

**6 徽章几何**：环中心 `(540,970)`，徽章中心半径 `R=400`，徽章 144×144（18·8 整除 ✓），
6 方向位 = 顶部起顺时针 -90°/-30°/+30°/+90°/±150°（`menu_index` 0..5 = compass/chest/anchor/gear/book/door，
见 `ui_icons.json` `bindings.index_menu`）。标签文字 = 徽章正下方 `text`（`scene.index.menu.<i>`）。

**图标 tint**：6 白蒙版（`icon_compass/chest/anchor/gear/book/door`，④-04 已生成）经 `medallion_ring` 铜底 +
`blit_image_tint` gold 上色（跨层 tint，S21.14）；缺资产 = 代码矢量回退（crash-safe）。

## 3. 信息架构（场景叙事 DSL，`scene_manifest_v3.json` index 段）

- **mood** = `epic_entry`（沿用；游戏标题页）；
- **focus** = `compass_ring`（改自现 `title_art`），`bbox_pct {x:6, y:26, w:88, h:49}`（N8 对账，见 §5）；
- **rhythm.beats**（权重和 = 1.0，beats[0] = focus.anchor，权重 ≥ 0.30）：

| beat | weight | y_pct | 语义 |
|---|---|---|---|
| `compass_ring` | 0.55 | 50 | 罗盘环（主焦点：舵轮 + 6 徽章） |
| `title_plaque` | 0.30 | 11 | 标题木牌 |
| `footer` | 0.15 | 95 | 版本签 |

- **info_density** / **permanent_hud_count_max**：index 无常驻 HUD（footer = 版本签，非 HUD）→ `permanent_hud_count_max = 0`。

## 4. 交互契约（`index.rs` on_touch；命中判定与渲染同坐标系，`EXP-067`）

命中优先级（6 徽章 + 标题；无滑动）：

| 命中区（rect） | 动作 | 语义 |
|---|---|---|
| `m0_newgame` | `MenuAction::NewGame` | 新游戏（键盘 Enter / 1） |
| `m1_load` | `MenuAction::Load` | 载入游戏 |
| `m2_save` | `MenuAction::ReadSave` | 读取存档 |
| `m3_options` | `MenuAction::Options` | 选项设置 |
| `m4_help` | `MenuAction::Help` | 帮助说明 |
| `m5_quit` | `MenuAction::Quit` | 离开游戏 |
| `title_plaque` | 无 | 标题（纯展示，不命中） |

> 现 index 为**线性列表键盘高亮**（上下键 + Enter）；罗盘环版保留键盘 `↑↓/1..6/Enter` 但命中区 = 6 徽章 rect
> （`on_touch` 逆投影同坐标系），高亮态 = 该 medallion 外圈 `gold_warm` 2px + 放大 1.1×（选中 tint=`text_hi`）。

## 5. 门禁耦合点（改版面必须同改，否则门禁 FAIL）

| 门禁 | 锚点 | 本规格目标值 |
|---|---|---|
| N8（`check_scene_narnarative`） | `scenes.json focus_anchor.bbox` ≡ `manifest.focus.bbox_pct × (1080,1920)` ±1px | `compass_ring {65,500,950,941}` ↔ `bbox_pct {x:6,y:26,w:88,h:49}`（6×10.8=64.8→65 / 26×19.2=499.2→500 / 88×10.8=950.4→950 / 49×19.2=940.8→941，全 ±1 ✓） |
| N3（焦点面积 ≥20%） | `w×h/100` | 88×49 = 43.12% ✓ |
| N4（beats[0]=anchor，权重≥0.30，和=1.0） | `compass_ring` 0.55 / 和 1.00 ✓ | |
| 8pt（`doccheck2::ui_8pt_walk`） | 非豁免键 8 整除 | `title_plaque {160,128,760,320}` + `footer {344,1816,392,64}` + 徽章 144 ✓；`compass_ring`/`helm`/`m0..m5` 入 8pt 豁免表（径向键，同 `world_canvas`） |
| 原语闭集（`check_data_driven`） | 7 种（rect/text/icon/grad/glow/disc/image） | 只用 `image`（海图/木牌/舵轮/徽章/白蒙版图标）+ `text` + `rect`（签条），**不新增原语 kind** |
| 资产登记（`ui_assets.py verify`） | `image` asset 必须入 `ui_manifest.json` | `chart_full`/`helm` 已入仓；`medallion_ring` **待 S22.1 生成 + 登记**；`icon_*` 已入仓（S21.14） |

## 6. 资产接线表（运行时 `blit_image_tint` / primitive `image`；`ui_manifest.json` 单一事实源）

| 用途 | 资产 | 状态 |
|---|---|---|
| 满屏羊皮海图底 | `assets/ui/chart_full.png`（`background` 9:16） | 已入仓（S21.1，与 main_map 共用） |
| 标题木牌 | `assets/ui/title_art`（沿用现 index 木牌图） | 已入仓 |
| 中心舵轮 | `assets/ui/helm.png`（`medallion` luma） | 已入仓 |
| 6 徽章铜底（共用 1 枚） | `assets/ui/medallion_ring.png`（`medallion` 1:1 luma） | **待 S22.1 生成** |
| 6 徽章图标（白蒙版 + gold tint） | `assets/ui/icon_{compass,chest,anchor,gear,book,door}.png` | 已入仓（S21.14，经 `blit_image_tint` 上色） |

> `medallion_ring` = 一枚圆形黄铜徽章（luma 抠图，纯白/品红底），6 枚菜单共用；图标按 `menu_index` 叠其中心，
> tint=`gold`（选中态 `text_hi`）。同 `main_map` 节点牌口径（1 类底 + 白蒙版图标 tint），不逐徽章出 6 张。

## 7. 改动面声明（硬规则 7）

| 块 | 本规格（S22.0） | 落码任务 |
|---|---|---|
| 业务规则 | **无**（菜单项 = 6 个既有 `MenuAction`，不改规则） | — |
| 数据 | **无**（坐标表先落文档；`scenes.json` index 块 + `manifest` index 段 + 8pt 豁免表改在 S22.2） | S22.2 |
| 规则 | 无 | — |
| 表现 | 规格定义层序 / 罗盘环命中 / 徽章→图标映射（不写码） | S22.2 |
| 资产 | 规格引用已入仓 `chart_full`/`helm`/`icon_*` + 登记 `medallion_ring` 待 S22.1 | S22.1 |

**跨模块原因**：满屏海图（z2 资产）+ 罗盘环（z5 径向命中）+ 徽章→图标映射（z5 资产映射）+ N8 焦点 bbox
（数据 `manifest`↔`scenes.json` 硬耦合）必须**同一版坐标表**，否则「徽章错位 / N8 断链 / 图标没上色」。
故本规格作为单一事实源先定案，S22.1（出 `medallion_ring` 资产）+ S22.2（落码）逐个只读本表。

## 8. 与旧首页的差异（现状 → S22）

| 元素 | 现状 | S22（本规格） |
|---|---|---|
| 底 | `bg_night_sea` 夜海照片 + 星空 | `chart_full` 满屏羊皮海图（与 main_map 统一） |
| 菜单 | 单列 6 项 icon+字竖排（`list {240,704,600}`，稀疏留白） | 罗盘环 6 铜徽章（`compass_ring` 径向）+ 中心舵轮 |
| 图标 | 代码矢量（48px 最近邻，糊、风格不一） | 白蒙版 + gold tint（④-04，`blit_image_tint`） |
| 构图重心 | 无（全夜空） | 罗盘环（`compass_ring` N8 focus 43%） |
| 命中 | 线性列表键盘高亮 | 6 徽章 rect 命中（同坐标系，保留键盘） |

## 9. 出图验收口径（S22.3）

- `ui_preview.ps1 index` 出 `evidence/bevy_native_index.png`：满屏羊皮海图 + 中心舵轮 + 6 铜徽章可辨 + 图标 tint 上色（非白块/非糊矢量）；
- 6 徽章 e2e 命中（点 m0..m5 各触发对应 `MenuAction`）+ 键盘 ↑↓/1..6/Enter 高亮态正确；
- 双后端 CPU↔Bevy 像素对拍 ≤0.5% + N8/N3/N4 0 失败 + 8pt 全过（径向键豁免）+ gate 全 项 0 failure；
- **用户视觉验收通过（消息留痕，A 纪律）**。
