# 渲染管线契约（Render Pipeline Contract）

> 单一事实源：《金银岛》UI 渲染链路的**路径、z-order、越界裁剪、PCX 透明**四类行为契约。
> 任何改 `sprite_atlas.rs` / `terrain_atlas.rs` / `game_app.rs::render` / `menu_shell.rs` 的代码，
> 必须先读本文档；改完必须过 `tests/render_snapshot.rs`（gate 第 19 项）+ `scripts/check_data_source.py`（gate 第 20 项）。
> 证据锚点全部来自 `evidence/` 审计 + `legacy/ref/pseudocode/`（参照物，非验收基准）。

---

## 0. 为什么需要这份契约（踩坑史）

| 事件 | 根因 | 后果 |
|---|---|---|
| S13.5 修 HD 路漏 native 路 | 没有路径表说清「哪条路在哪个 quality 下被走」 | 修错对象（EXP-075①） |
| 洋红框 | PCX 透明位（index 0）散在 4 处 blit，无单一契约 | 反复挖（`audit_mag_red_cause.md`） |
| 贴图覆盖菜单 | 越界裁剪 `ox=min(x,FB_W-1)` 拉回右缘，原版是求交为空不画 | 「特定区域错位」（`audit_zorder.md` B4，高置信） |
| z-order 误判 | 没有「z-order = 文件顺序」的契约，反复猜 f5 分档/分帧 | 两轮绕圈（`audit_f5_zorder.md`，已坐实） |

**本契约就是把这些结论固化成可机器判定的门禁（第 19/20 项），杜绝再犯。**

---

## 1. 渲染路径表（哪条路在什么条件下被走）

| 路径 | 入口 | 帧缓冲 | 调色板/透明判定 | feature 门控 | quality 模式 |
|---|---|---|---|---|---|
| **RGB565（native 默认）** | `GameApp::render` → `route_render` + `blit_scene_sprites` | `MenuShell.fb`（`Framebuf`，RGB565 u16） | PCX `palette[idx]`，**index 0 = 哨兵透明** | 无（默认构建） | `QualityMode::Original` |
| **RGBA（HD）** | `GameApp::render_hd` → `render`（复用）+ `blit_scene_rgba` | `MenuShell.fb_rgba`（`FramebufRgba`） | HD PNG `rgba[3] < 128` 透明；PCX 回退时 index 0 透明 | `feature = "hd-8bit"` | `QualityMode::Hd` |
| **GPU** | `GpuRenderer` 直接纹理 | 显存 | 纹理 alpha | `feature = "gpu-render"` | S10.4 实验 |

**关键结论（杜绝 S13.5 修错路径）**：
- native 默认走 **RGB565 路**，靠 **PCX palette index 0** 判透明（**不是** RGBA 的 a 通道）。
- HD 路靠 **RGBA alpha < 128** 判透明（`ALPHA_TRANSPARENT_CUTOFF`，EXP-075②）。
- 两条路**独立**：修 HD 路不影响 native，修 RGB565 路不影响 HD。任何透明/越界改动必须**两路同步**（`sprite_atlas.rs` 的 `blit_sprite` 与 `blit_sprite_rgba` 口径一致）。

---

## 2. z-order 契约（文件顺序，非 f5 分档、非分帧）

**结论（高置信，`audit_f5_zorder.md`）**：
- 原版 `sub_44004` 按文件顺序逐条读入 `.Map` 控件，`sub_449C8` 头插法入链表，`sub_443FC` 沿 `prev` 指针遍历 → **第 1 条先画（底层）→ 第 N 条后画（顶层）**。
- **不按 f5 控件类型分档**，**不分帧渲染**（`a1[11]/a1[12]` 是每精灵的局部多帧动画计数，非全局批轮转）。
- Rust `blit_scene_frame` 按 `MapControl::parse_words` 的顺序（= 文件顺序）全 blit，**与原版一致**。

**契约**：
- `blit_scene` / `blit_scene_rgba` 必须按 `controls` 数组顺序（文件顺序）逐个 blit，**不得按 f5 排序、不得分批**。
- 后画的精灵盖先画的（z-order = 文件顺序）。
- 原版调用序：`sub_443FC`（.Map 控件全 blit）→ `sub_44828`×4（数字字形，压在控件顶层）。Rust 等价：`route_render`（场景文字）→ `blit_scene_sprites`（.Map 精灵）→（数字由场景画）。**文字在立绘之上的场景，靠 `.Map` 记录顺序保证（文字精灵排在立绘精灵之后）。**

**误读澄清**：
- `index.Map` 全文件流 `w[4]=176` 是表头第 3 字（精灵宽），**不是记录数**。
- `MapScene.words[4]`（跳 16 字节后）= 全文件 `w[8]` = **31** = 记录数，与原版 `sub_44004` 读的 `v28` 一致。**`MapControl::parse_words` 用 `words[4]` 当 count 是正确的。**

---

## 3. 越界裁剪契约（求交为空则整条不画，不拉回右缘）

**原版行为（高置信，`audit_zorder.md` A3 + B4）**：
- `sub_5D9C4` 第一步 `sub_5E9E0`/`sub_5EAA8` 做**源窗 × 目标窗矩形求交**。
- 求交后源宽/高 = 0（`v27==v28 && v30==v29`）→ `sub_5D9C4` 直接 return，**不写任何像素**。
- 因此**原点 x ≥ 场景宽（176）或 y ≥ 场景高（220）的精灵，原版整条不画**。

**Rust 现状（待修）**：
- `blit_sprite` 的 `ox = px_x.min(FB_W-1)` 把 x=185/191/207 的越界精灵**拉回第 175 列**画 1 列宽竖条 → 与原版相反，是「贴图覆盖菜单」的直接根因。

**契约（第 2 步落地）**：
- 纯函数 `clip_rect(src_w, src_h, ox, oy) -> Option<ClipRect>`：
  - `ox >= 176 || oy >= 220` → `None`（整条跳过，对齐原版求交为空早退）。
  - `ox < 176 && oy < 220` → `Some(ClipRect { x0: ox, y0: oy, w: min(src_w, 176-ox), h: min(src_h, 220-oy) })`（部分越界裁剪，与原版一致）。
- `blit_sprite` / `blit_sprite_rgba` 开头算 `clip_rect`，`None` 直接 `return`（不写像素、不 `mark_dirty`）。
- `mark_dirty` 只在 `clip_rect` 为 `Some` 时按裁剪后 w/h 调用。

**数据锚点（index.Map 实测）**：
- `f1=111`（105×11 长条）20 条，其中 3 条越界：x=185（y=22 行）、x=191/x=207（y=44 行）。原版不画，Rust 必须不画。
- `f1=116`（122×74）@(0,65)：原点合法，部分越界裁剪（右缘 122<176 全画，底缘 65+74=139<220 全画）。

---

## 4. PCX 透明契约（index 0 = 哨兵，4 处同口径）

**原版行为（高置信，`audit_mag_red_cause.md` + F-311）**：
- `ui.pak` 94/120 张 PCX 的调色板 **index 0 = 洋红 (255,0,255)**（哨兵色，PCX 尾部调色板 0 槽常被填成显眼色）。
- 原版 8-bit blit 例程 `sub_5D9C4` 逐像素 `if (idx != 0) write` —— **跳过 index 0（透明位）**。
- 因此 index 0 在原版 = 透明，透出背景 `COLOR_BG`。

**契约（单一常量，4 处引用）**：
- 常量 `TRANSPARENT_INDEX: u8 = 0`（`sprite_atlas.rs` 顶部或 `ti_formats::pcx`）。
- 4 处 blit 全部 `if idx == TRANSPARENT_INDEX { continue; }`：
  1. `blit_sprite`（RGB565，`sprite_atlas.rs` ~L260）
  2. `blit_sprite_rgba` 的 PCX 回退分支（~L403）
  3. `terrain_atlas.rs::blit_decoded`（RGB565 地形）
  4. `terrain_atlas.rs::blit_decoded_rgba`（RGBA 地形）
- HD PNG 路径（`blit_sprite_rgba` 的 PNG 分支）用 `ALPHA_TRANSPARENT_CUTOFF = 128`（rgba[3] < 128 跳过），**与 PCX index 0 独立**，但语义都是「跳过透明像素，透出背景」。

**回归断言（`tests/render_parity.rs` `blit_pcx_independent`）**：
- 独立 blit 基线必须同步跳过 index 0（否则与管线不一致，基线漂移）。

---

## 5. 门禁锚点（改渲染代码必过）

| 门禁项 | 命令 | 判定 |
|---|---|---|
| **第 19 项 渲染快照** | `cargo test -p ti-app render_snapshot` | 7 场景 × 2 路（RGB565+RGBA）= 14 张 176×220 快照，与 `assets/render_baseline/<scene>_<path>.png` diff = 0。基准由**越界修复后**代码生成（杜绝 EXP-074/075 自证循环）。 |
| **第 20 项 数据源门禁** | `python scripts/check_data_source.py` | `legacy/data/*.Map` / `*.pak` git diff 非空 → 强制跑第 19 项快照，漂移即 FAIL。 |
| **第 7 项 no-unwrap** | `scripts/gate.ps1` 第 7 项 | 渲染代码（含新增 `render_pipeline.rs`）0 处 `.unwrap()`/`.expect()`（tests/ 除外）。 |

**数据源变更纪律**：任何 `legacy/data/*.Map` / `*.pak` 改动 → 必须同步跑快照门禁 + 更新基准，否则第 20 项 FAIL。

---

## 6. 证据索引

| 契约条目 | 证据 | 置信度 |
|---|---|---|
| 路径表（§1） | `game_app.rs::render`/`render_hd` 源码 + feature 门控 | 高 |
| z-order = 文件顺序（§2） | `evidence/audit_f5_zorder.md`（sub_44004/sub_449C8/sub_443FC 链表遍历推演） | 高 |
| words[4]=31 非 176（§2 澄清） | `evidence/audit_f5_zorder.md` §3（780 字节流校验） | 高 |
| 越界求交为空不画（§3） | `evidence/audit_zorder.md` A3（sub_5EAA8/sub_5D9C4 早退） | 高 |
| Rust 越界拉回是 bug（§3） | `evidence/audit_zorder.md` B4（`sprite_atlas.rs:244`） | 高 |
| PCX index 0 = 洋红哨兵（§4） | `evidence/audit_mag_red_cause.md`（94/120 张实测） | 高 |
| f1=111 越界 3 条（§3 锚点） | `legacy/data/index.Map` 实测（x=185/191/207） | 高 |

**遗留（不阻塞本契约，标推测）**：
- `a1[10]` 的确切语义（精灵源裁剪宽 vs 高/因子）—— 中置信，不影响越界早退逻辑。
- `sub_44004` 装载 index.Map 时 `a1[248]/a1[249]`（网格行/列）实测值 —— 不影响 .Map 控件 blit。

---

## 7. 元素级验证法（EXP-076：逐精灵定位「哪个元素错位」）

全帧 diff（`render_snapshot` 第 19 项）只能告诉你「有没有漂移」，不能告诉你「哪个元素漂移」。
定位到具体元素用**逐精灵 dump**：

### 方法

1. **逐精灵 blit 清单**：对场景每个 `.Map` 控件，单独 blit 到干净帧缓冲，记录
   `记录号 | f1 | 原点(f2,f3) | 精灵w×h | clip_rect 结果 | 越界判定(OK/CLIP/SKIP) | 贡献像素数`。
   越界（`f2>=176 || f3>=220`）→ SKIP（修复后行为）；部分越界 → CLIP 到新尺寸；全在屏内 → OK。
2. **修复前后对照**：同场景跑两遍（修复前 `ox=min(x,175)` 越界拉回 vs 修复后 `clip_rect` 越界早退），
   diff 两条清单——SKIP 条数增加 + 其贡献像素 = 0 = 越界条已消除。
3. **像素级断言**：越界条落点（旧版拉回的 x=175 列目标行）修复后必须 == 背景 `COLOR_BG`。

### index 场景实测（修复后，`element_dump index`）

| 记录 | f1 | 原点 | 判定 | 说明 |
|---|---|---|---|---|
| 11 | 111 | (185,22) | **SKIP** | 越界条，旧版拉回 x=175 画 1 列竖条 → 修复后 0 像素 |
| 20 | 111 | (191,44) | **SKIP** | 越界条 |
| 21 | 111 | (207,44) | **SKIP** | 越界条 |
| 19 | 111 | (175,44) | OK | 屏内最后一列（x=175<176），合法条保留 1 像素宽（右缘裁剪） |
| 25 | 117 | (115,77) | OK | 立绘 90×137 → 裁剪到 61×137（右缘 176），压菜单区（z-order 文件顺序，第 26 条文字盖其上） |

**判据**：SKIP 数 = 3 且其 f2 ∈ {185,191,207}（>=176）= 越界早退生效；x=175,y=22 行（记录 11 拉回落点）修复后 == 背景。

### 何时用

- 报「特定区域贴图错位/覆盖菜单」→ 先跑 `element_dump <scene>` 看哪些记录 SKIP/CLIP/OK，定位到具体记录号 + f1 精灵。
- 改了 `.Map` 数据或越界裁剪逻辑后 → 重跑 element_dump 对照修复前清单，确认只有越界条从「拉回」变「SKIP」。
- 全帧 diff 红了但不知道哪个元素 → element_dump 逐条像素贡献数对照，贡献数突变那条就是 culprit。

### 固化

`element_dump` 作为 ti-app 长期诊断 bin（`crates/ti-app/src/bin/element_dump.rs`，Cargo.toml 声明），
不再临时删。它是渲染契约 §7 的机器工具，与 `render_snapshot`（第 19 项全帧 diff）互补：
`render_snapshot` 锁「整体没漂」，`element_dump` 定位「漂的是哪个元素」。
