# 渲染架构（Runtime Render Architecture）

> **单一事实源**：《金银岛》**运行时**渲染架构 —— 分层、绘制原语、z-order、坐标契约。
> 任何改 `ti-platform/gfx.rs` / `ti-app/scenes/*` / `ti-app/menu_shell.rs` /
> `ti-app/game_app.rs::render` 的代码，必须先读本文。
>
> **版面数值**（各场景坐标/配色/字号）在 [`../business/08_layout_design.md`](../business/08_layout_design.md)，
> 本文不重复数值，只定架构。
>
> **legacy 像素管线契约**（PCX 调色板 index 0 透明、`.Map` 文件序 z-order、越界求交裁剪）
> 见 [`07_render_pipeline.md`](07_render_pipeline.md)，但自 S16 渲染线裁决起**降级为逆向参照**
> ——运行时不再走逐像素 PCX blit（见 §1.3）。

---

## 0. 决策与理由（S16 渲染线）

**决策**：运行时**零 legacy 像素层**。运行时画面 100% 由代码绘制（矩形 + 文本）产出；
`legacy/data/*.Map`、`ui.pak`、PCX、地形瓦片降为**逆向事实参照 + 诊断输入**，不参与渲染。

**理由**（对照踩坑史）：

| 事件 | 根因 | 本架构如何消除 |
|---|---|---|
| S15.2 三套坐标系同屏（1080 chrome + 1:1 legacy + 缩放映射） | legacy 像素层与呈现层各自带坐标系 | 契约 1：运行时只有 1080 一个坐标系，legacy 坐标不进运行时 |
| legacy↔1080 缩放公式散落 3 处（RGB565/RGBA/独立基线） | 同一映射多副本，漂移即静默像素分歧 | 契约 1：无 legacy 像素即无映射需求，副本归零 |
| 逐帧 CPU 重采样 1980s 像素 | 把参照物当运行时管线 | 契约 2：像素来源只有代码绘制 |
| 翻转语义与缩放映射纠缠 | 两个概念揉进一个循环 | 契约 3：绘制原语只有 3 个，各自语义单一 |
| 测试以 legacy 像素为判据（密度/立绘区） | 判据锚在参照物上 | 契约 5：门禁判据锚在代码绘制区 + scenes.json |

---

## 1. 分层

### 1.1 四层与依赖方向

```
┌─ 数据层 ────────────────────────────────────────────────────┐
│ legacy/data/*.Map + ui.pak   → 逆向事实（控件语义/派发码）  │
│ assets/data/scenes.json      → 版面唯一事实源（1080 坐标）  │
│ assets/data/strings.json     → 文案                          │
│ assets/fonts/*.ttf           → 字体                          │
└──────────────────────────────┬───────────────────────────────┘
                               │ 只读数据
┌─ 逻辑层（ti_core）───────────┴───────────────────────────────┐
│ rules/* · domain/*：移动/战斗/经济/抽卡/消息分页             │
│ 载体 = GridPos / BattleSnapshot / SaveSlot …（无像素坐标）   │
└──────────────────────────────┬───────────────────────────────┘
                               │ 只读状态
┌─ 视图层（ti-app）────────────┴───────────────────────────────┐
│ scenes/chrome.rs   公共外壳：外框/标题条/面板/血条槽/等距格   │
│ scenes/{index,select_role,select_map,main_map,vs,message,   │
│         saveload}.rs   逐场景接线：读状态 → 调 3 原语          │
│ menu_shell.rs      文字绘制 + 帧缓冲持有 + 上屏               │
└──────────────────────────────┬───────────────────────────────┘
                               │ 1080 坐标 + 3 原语
┌─ 平台层（ti-platform）───────┴───────────────────────────────┐
│ gfx.rs：Framebuf(RGB565) / FramebufRgba(RGBA8) / CANVAS_W/H   │
│ gpu_renderer.rs（feature=gpu-render）：纹理上屏               │
└──────────────────────────────────────────────────────────────┘
```

**依赖方向严格单向向下**：数据 → 逻辑 → 视图 → 平台。禁止反向依赖
（平台层不得引用 `ti_core`/`ti-app`；`ti_core` 不得引用 `ti-app`/`ti-platform`）。

### 1.2 各层职责边界

| 层 | 只做 | 绝不做 |
|---|---|---|
| 数据 | 解析成结构体 | 产像素、做坐标变换 |
| 逻辑 | 状态转移与规则判定 | 画任何东西、持有帧缓冲 |
| 视图 | 读状态 → 调 3 原语 | 跑业务规则、直接写 `Framebuf` 切片 |
| 平台 | 帧缓冲分配/清屏/上传/上屏 | 解释游戏状态、知道 legacy 存在 |

### 1.3 legacy 的运行时角色 = 无

| legacy 资产 | 运行时角色 | 保留用途 |
|---|---|---|
| `*.Map` | **无** | 控件语义/派发码的逆向事实；`element_dump`/`specskel` 诊断输入 |
| `ui.pak` / PCX | **无** | 精灵尺寸/调色板的逆向事实；诊断工具的解码输入 |
| 地形瓦片（`glmap_*`） | **无**（网格尺寸/地图数据若被业务读，属逻辑层数据） | 地图数据的逆向事实 |

> legacy **解码**（`SpriteAtlas`/`Image::decode`/包解析）保留，供诊断工具；
> legacy **写像素**（`blit_sprite`/`blit_sprite_rgba`/`terrain_atlas` 的 blit 分支）删除。

---

## 2. 绘制原语（只有 3 个）

视图层允许且只允许调用这 3 个绘制原语（经 `SceneHost` 转发）：

| 原语 | 签名语义 | 用途 |
|---|---|---|
| `fill_rect(x, y, w, h, color)` | 以 1080 空间左上角 + 尺寸填一个矩形 | 外框/面板/棋盘格/棋子/血条/文字底条 |
| `draw_text(text, x, y)` / `draw_text_colored(...)` | 在 1080 空间起点绘制一行文本（TTF 44px，回退位图 12px advance） | 标题/列表项/状态行/HUD/血量数字 |
| `blit_texture(...)` | 未来若引入 HD 位图资产，一次纹理 blit（GPU 拉伸或 CPU 单次拷贝） | 当前**未启用**；启用需先补契约 |

**语义单一性**：每个原语只做一件事。任何"既要缩放又要翻转又要裁剪还要查像素"的
绘制逻辑都是架构违规——它属于工具链/构建期，不属于运行时。

---

## 3. 五条契约

### 契约 1 · 单一权威呈现坐标系

- 呈现坐标系**只有** 1080×1920（`menu_shell::LOGICAL_W_I32/H_I32`、`gfx::CANVAS_W/H` 三处常量同值）。
- 所有绘制原语入参、帧缓冲索引、快照/基线尺寸，一律 1080 空间。
- **legacy 坐标（176×220）不得出现在运行时**；如需 legacy 尺寸做业务判断，由逻辑层
  以数据形式提供，视图层不得感知。
- **机器保证**：`doccheck2` grep 不变量 —— `ti-platform/gfx.rs`、`ti-app/scenes/**`、
  `ti-app/menu_shell.rs` 内出现字面量 `176` / `220` 即 FAIL（契约 4 解释，§5 门禁）。

### 契约 2 · 像素来源单一

- 运行时帧缓冲的每一个非底色像素，来自 §2 的 3 个原语。
- 禁止在运行时按帧解码/上传/重采样 legacy 位图资产。

### 契约 3 · 逻辑与渲染分离

- 视图层 `render()` **只读**状态、**只调** 3 原语；不跑业务规则、不改状态。
- 业务规则只在 `ti_core::rules/*`；状态推进只在 `GameApp::advance(GameInput)`。
- 判据：把 `render()` 换成"只读快照"，同一次渲染必产生同一帧（确定性门禁覆盖）。

### 契约 4 · 版面数据驱动

- 场景版面坐标只有一处事实源：`assets/data/scenes.json`。
- `chrome.rs` / `scenes/*.rs` 读它，不写死坐标常量（几何常量如 `wood_frame` 的 12px
  外框宽、`iso_cell` 的公式属**实现**，可写死在 `chrome.rs`，但不得重复场景级坐标）。
- 改版面 = 改 `scenes.json`，代码不动；`08_layout_design.md` §4 表与之同值。

### 契约 5 · 门禁判据锚在代码绘制区

- 门禁断言的对象必须是**代码绘制的几何/像素**（`scenes.json` 矩形、`chrome` 原语调用、
  代码棋子坐标），不得以 legacy 位图像素为判据。
- legacy 的语义结论（z-order、控件行为）以**纯数据断言**形式留在 `ti_formats` 层，
  或作为事实记录留在 `legacy/docs/facts/`，不进像素门禁。

---

## 4. z-order 单一枚举

绘制顺序由一个枚举唯一决定（视图层自底向上）：

```
BgBoard  → ChromePanels → DynamicMarkers → Text
```

| 层 | 内容 | 实现 |
|---|---|---|
| `BgBoard` | 场景底色、等距棋盘菱形格 | `chrome::parchment_bg` + `chrome::iso_cell` 循环 |
| `ChromePanels` | 外框、标题条、面板、血条槽、提示条 | `chrome::wood_frame/title_bar/panel/bar_slot/hint_row` |
| `DynamicMarkers` | 玩家棋子、据点、BOSS 格 | `scenes/main_map.rs` 等距坐标上的代码色块 |
| `Text` | 标题文字、列表项、HUD 数值、状态/提示文字 | `chrome::draw_*_text` / `SceneHost::draw_text` |

**纪律**：层内顺序由代码调用顺序决定，层间顺序由本枚举决定；**不得**在层内偷偷
补画属于其他层的内容（例如在 `DynamicMarkers` 里补 HUD 底板）。新增绘制先归层，再实现。

---

## 5. 契约的机器保证（门禁）

| 契约 | 保证方式 | 门禁项 |
|---|---|---|
| 契约 1 坐标系唯一 | `doccheck2` grep：`ti-platform/gfx.rs` + `ti-app/scenes/**` + `menu_shell.rs` 禁字面 `176`/`220`；另断言 `LOGICAL_W_I32 == gfx::CANVAS_W` 等三常量同值 | gate 第 13 项（doccheck2 第 18 检查 runtime-no-legacy-coords） |
| 契约 2 像素来源单一 | `grep` 断言：`game_app::render`/`render_hd` 不调 `blit_scene*`；`sprite_atlas` 无写帧函数；`terrain_atlas` 无 blit 分支 | gate 第 13 项（doccheck2）+ 编译期（函数删除即失效） |
| 契约 3 逻辑/渲染分离 | `render_parity.rs` 确定性/非空/场景区分；`render()` 签名不暴露可变状态 | gate 第 3 项（cargo test） |
| 契约 4 版面数据驱动 | `render_intent.rs` 6 个 `*_layout_matches_doc` 断言 `scenes.json` = §4 表值 | 测试（gate 第 3 项 cargo test） |
| 契约 5 判据锚代码区 | `render_intent.rs` 关键矩形/动态区断言基于代码绘制；`e2e_render_alignment` 的 legacy z 断言迁 `ti_formats` 纯数据 | 测试（gate 第 3 项 cargo test） |

**不变量清单**（常量同值，编译期 + 单测双锁）：

```
menu_shell::LOGICAL_W_I32 == gfx::CANVAS_W == 1080
menu_shell::LOGICAL_H_I32 == gfx::CANVAS_H == 1920
gfx::FB_PIXELS == CANVAS_W * CANVAS_H
```

**基线镜像恒等（S15.6 B1 裁决③ / QA014）**：`render_baseline`（gate 第 19 项 render-snapshot）为**唯一权威基线**；
`forward_baseline`（gate 第 19.5 项 forward-render-snapshot）与 `hd_baseline`（gate 第 18 项 visual-regression，
3 场景抽验 Index/SelectRole/MainMap）是同一纯代码渲染器的**镜像**
（S15.4 删 legacy 写帧层后 forward 开关退役，S15.6 QA017，两态帧恒等）。
对应场景文件逐字节恒等（比 SHA256 恒等更强），由 `doccheck2` 第 19 检查
（`baseline-mirror-identity`，gate 第 13 项）机器锁定，防未来某套基线被单独改动导致三套门禁静默分叉。
gate 三项门禁的**结构合并**（19/19.5 并入 18）留 S15.6 B6 收口，本轮不动 gate 结构。

---

## 6. 测试重锚矩阵

| 门禁测试 | 原判据（legacy 像素） | 重锚后判据 |
|---|---|---|
| `render_intent::large_portraits_respect_layout_zones` | `.Map` 大精灵 clip 矩形 ∈ `portrait_zones` | **代码绘制动态实体**（玩家/据点/BOSS 方块）中心 ∈ `scenes.json` `portrait_zones` 声明区 |
| `render_intent::scene_layer_contributes_text_region_pixels` | 场景层帧 XOR 精灵层帧 > 0 | 场景层（chrome）绘制的关键区 ≠ 底色，且未被其他层整块覆盖 |
| `render_intent::key_rects_have_pixels` | 场景层关键矩形有像素 + vs 槽底 fill | 保留（已是 chrome/bar_slot 判据），坐标 1080 化 |
| `render_intent::text_regions_not_flooded_by_sprites` | 文字区 legacy 精灵密度 ≤ 90% | 改为「文字区被 chrome 面板/底条覆盖，非底色占比 > 0」 |
| `e2e_render_alignment::z_order_matches_lowest_file_index_wins` | `.Map` 文件序 z 逐像素 | 迁 `ti_formats` 纯数据断言（`MapControl::parse_words` 保序），**不再**像素门禁 |
| `e2e_render_alignment::f4_flip_pixel_direction_is_correct` | f4 翻转像素方向 | **删除**（无 legacy 精灵即无 f4 运行时语义） |
| `e2e_render_alignment::disabled_records_never_render` | 关闭标记不产像素 | 迁 `ti_formats` 纯数据断言（`f2/f3` 哨兵解析） |
| `render_parity::render_sprite_region_matches_tiformats_baseline` | 独立 PCX 解码基线逐像素 | **删除**（无运行时精灵层） |
| `render_parity_hd::*` | HD RGBA 精灵层 | 保留"帧缓冲尺寸/确定性/非空"，删精灵层相关断言 |
| `render_snapshot` / `visual_regression` / `render_snapshot_forward` | legacy 精灵层基线 | **重生成**（纯代码渲染器基线） |

---

## 7. 落地分期

| 期 | 内容 | 验收 |
|---|---|---|
| **Stage 0** | 本文 + `08_layout_design.md` §4/§6 定稿；doccheck2 grep 不变量上线；台账登记 | doccheck/ledger 绿 |
| **Stage 1** | 删写像素路径：`sprite_atlas::blit_sprite`/`blit_sprite_rgba`、`terrain_atlas` blit 分支、`game_app::render*` 的 `blit_scene*` 调用；`element_dump` 改 decode-only；删坐标转换函数 | `cargo build` + grep 门禁绿 |
| **Stage 2** | 测试重锚（§6 矩阵）；基线重生成；`legacy/docs/facts` 记「legacy 像素退出运行时」+ 回写 EXP | `cargo test` 全绿 + gate 绿 |

---

## 8. 禁止事项（反模式清单）

1. **禁止**在运行时按帧做 legacy 位图解码/缩放/重采样（`legacy_scaled` 类函数不得复活）。
2. **禁止**同一坐标变换在多处复制实现（"3 处映射"是本次事故根因）。
3. **禁止**视图层跑业务规则（血量百分比、地图步进行走等必须在 `ti_core`）。
4. **禁止**在 `scenes/*.rs` 写死场景级坐标（必须读 `scenes.json`）。
5. **禁止**把 legacy 像素当门禁判据（判据锚代码绘制区，契约 5）。
6. **禁止**新增第 4 个绘制原语而不先补本文契约（扩展须走文档 + 门禁）。
7. **禁止**"顺手"把 scaling/z-order/裁剪语义揉进单个函数（语义单一性，契约 3）。
