# 渲染管线契约（legacy 像素管线，Render Pipeline Contract）

> **S18.7b：像素基线机制已退役（不再存在）**。
> 依旧的 `assets/render_baseline/` / `forward_baseline/` / `hd_baseline/` 三套 golden PNG、
> `render_snapshot` / `render_snapshot_forward` / `visual_regression` 三个 diff 测试、`visual_baseline.ps1`
> 与其门禁项、`baseline-mirror-identity` 校验均**已全部删除**。
> 现行机制：**结构不变式断言**（`tests/render_invariants.rs`，7 场景 × 非黑覆盖率/内容签名互异/暖色板在位/TTF 在位）
> 与 **CPU↔Bevy 内存对拍**（`bevy-cpu-parity` 门禁项：GPU 真实离屏读回 vs CPU 期望逐像素对照）。
> 本文中仍有的「基线」词句为**历史记述**，不再是现行机制。

> ⚠️ **状态：降级为逆向参照（S16 渲染线裁决）**。运行时**零 legacy 像素层**：画面 100%
> 由代码绘制产出，逐像素 PCX blit 已退出热渲染路径。本文的 z-order 文件顺序 / 越界求交裁剪 /
> PCX 透明位 index 0 三类契约**仅作逆向事实参照与诊断工具依据**，不再是运行时验收基准。
> **运行时架构契约见 [`08_render_arch.md`](08_render_arch.md)**（分层 / 绘制原语 / z-order=调用顺序 /
> 单一坐标系）；`ti-render` crate 边界见 [`09_render_framework.md`](09_render_framework.md)。
> 版面数值见 [`../business/08_layout_design.md`](../business/08_layout_design.md)。

> 单一事实源：《金银岛》legacy UI 渲染链路的**路径、z-order、越界裁剪、PCX 透明**四类行为契约。
> 任何改 `sprite_atlas.rs` / `terrain_atlas.rs` / `game_app.rs::render` / `menu_shell.rs` 的代码，
> 必须先读本文档；改完必须过 `tests/render_invariants.rs`（gate 第 19 项）+ `scripts/check_data_source.py`（gate 第 20 项）。
> 本文档是**正向设计契约**（"渲染应当如何"）；逆向实现细节（原版函数地址、伪代码）仅作历史参照，归档 `legacy/`，不作为验收基准。

> **排查原则（EXP-077）**：原版 ARM WinCE 程序本机跑不起来，**禁止向用户索要原版截图**。
> 替代证据链：(a) 用户指认偏移量（看我渲染的 PNG，指出哪个精灵朝哪个方向偏了多少像素）；
> (b) 像素级对比工具（`scripts/compare_render.py`，**TODO 未实现**，当前用 `element-dump` + `render_invariants` 测试替代，见 gate 第 19 项）；
> (c) QEMU 系统模式（需 WinCE 镜像，本机没有）。

---

## 0. 为什么需要这份契约（踩坑史）

| 事件 | 根因 | 后果 |
|---|---|---|
| S13.5 修 HD 路漏 native 路 | 没有路径表说清「哪条路在哪个 quality 下被走」 | 修错对象（EXP-075①） |
| 洋红框 | PCX 透明位（index 0）散在 4 处 blit，无单一契约 | 反复挖 |
| 贴图覆盖菜单 | 越界裁剪 `ox=min(x,FB_W-1)` 拉回右缘，应当是求交为空不画 | 「特定区域错位」 |
| z-order 误判 | 没有「z-order = 文件顺序」的契约，反复猜 f5 分档/分帧 | 两轮绕圈 |

**本契约把这些行为固化成可机器判定的门禁（第 19/20 项），杜绝再犯。**

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

**契约（正向语言）**：
- `.Map` 控件**按文件记录顺序**逐条 blit：第 1 条先画（底层）→ 第 N 条后画（顶层）。
- **不按 f5 控件类型分档**，**不分帧渲染**（f5 是每精灵的局部多帧动画计数，非全局批轮转）。
- `blit_scene` / `blit_scene_rgba` 必须按 `controls` 数组顺序（= 文件顺序）逐个 blit，**不得按 f5 排序、不得分批**。
- 后画的精灵盖先画的（z-order = 文件顺序）。
- 渲染顺序：`route_render`（场景文字）→ `blit_scene_sprites`（.Map 精灵）。**文字在立绘之上的场景，靠 `.Map` 记录顺序保证（文字精灵排在立绘精灵之后）。**

**误读澄清**：
- `index.Map` 全文件流 `w[4]=176` 是表头第 3 字（精灵宽），**不是记录数**。
- `MapScene.words[4]`（跳 16 字节后）= 全文件 `w[8]` = **31** = 记录数。`MapControl::parse_words` 用 `words[4]` 当 count 是正确的。

> **历史参照（非契约依据）**：上述 z-order 结论源自对原版 `.Map` 加载例程（逐条读入 + 头插法入链表 + 沿 prev 遍历）的逆向推演，证据见 `evidence/audit_f5_zorder.md`。逆向函数地址（`sub_44004`/`sub_449C8`/`sub_443FC`）归档 `legacy/ref/`，本契约只保留行为结论。

---

## 3. 越界裁剪契约（求交为空则整条不画，不拉回右缘）

**契约（正向语言）**：
- 精灵 blit 前做**源窗 × 目标窗（176×220）矩形求交**。
- 求交后源宽或高 = 0 → **整条不画**（不写任何像素、不 `mark_dirty`）。
- 因此**原点 x ≥ 176 或 y ≥ 220 的精灵，整条不画**。

**实现（已落地）**：
- 纯函数 `clip_rect(src_w, src_h, ox, oy) -> Option<ClipRect>`：
  - `ox >= 176 || oy >= 220` → `None`（整条跳过）。
  - `ox < 176 && oy < 220` → `Some(ClipRect { x0: ox, y0: oy, w: min(src_w, 176-ox), h: min(src_h, 220-oy) })`（部分越界裁剪）。
- `blit_sprite` / `blit_sprite_rgba` 开头算 `clip_rect`，`None` 直接 `return`。
- `mark_dirty` 只在 `clip_rect` 为 `Some` 时按裁剪后 w/h 调用。
- **禁止** `ox = px_x.min(FB_W-1)` 把越界精灵拉回右缘画 1 列竖条（这是「贴图覆盖菜单」的根因）。

**数据锚点（index.Map 实测）**：
- `f1=111`（105×11 长条）20 条，其中 3 条越界：x=185（y=22 行）、x=191/x=207（y=44 行）→ 不画。
- `f1=116`（122×74）@(0,65)：原点合法，部分越界裁剪（右缘 122<176 全画，底缘 65+74=139<220 全画）。

> **历史参照（非契约依据）**：求交为空早退行为源自对原版 8-bit blit 例程（源窗×目标窗求交，交为空则直接 return）的逆向推演，证据见 `evidence/audit_zorder.md`。逆向函数地址（`sub_5D9C4`/`sub_5EAA8`/`sub_5E9E0`）归档 `legacy/ref/`。

---

## 4. PCX 透明契约（index 0 = 哨兵，4 处同口径）

**契约（正向语言）**：
- PCX 调色板 **index 0 = 哨兵透明位**（`TRANSPARENT_INDEX: u8 = 0`）。blit 时逐像素 `if idx == TRANSPARENT_INDEX { continue; }`，透出背景。
- 4 处 blit 全部同口径跳过 index 0：
  1. `blit_sprite`（RGB565，`sprite_atlas.rs`）
  2. `blit_sprite_rgba` 的 PCX 回退分支
  3. `terrain_atlas.rs::blit_decoded`（RGB565 地形）
  4. `terrain_atlas.rs::blit_decoded_rgba`（RGBA 地形）
- HD PNG 路径（`blit_sprite_rgba` 的 PNG 分支）用 `ALPHA_TRANSPARENT_CUTOFF = 128`（rgba[3] < 128 跳过），**与 PCX index 0 独立**，但语义都是「跳过透明像素，透出背景」。

**回归断言（`tests/render_parity.rs` `blit_pcx_independent`）**：
- 独立 blit 基线必须同步跳过 index 0（否则与管线不一致，基线漂移）。

> **历史参照（非契约依据）**：「index 0 = 洋红 (255,0,255) 哨兵色」源自对 `ui.pak` 94/120 张 PCX 调色板 0 槽的实测（PCX 尾部调色板 0 槽常被填成显眼色），证据见 `evidence/audit_mag_red_cause.md`。

---

## 5. 门禁锚点（改渲染代码必过）

| 门禁项 | 命令 | 判定 |
|---|---|---|
| **第 19 项 渲染结构不变式** | `cargo test -p ti-app render_invariants` | 7 场景 × 结构不变式（非黑覆盖率/内容签名互异/暖色板在位/TTF 在位），**无像素基线** |
| **第 20 项 数据源门禁** | `python scripts/check_data_source.py` | `legacy/data/*.Map` / `*.pak` git diff 非空 → 强制跑第 19 项快照，漂移即 FAIL。 |
| **第 7 项 no-unwrap** | `scripts/gate.ps1` 第 7 项 | 渲染代码（含新增 `render_pipeline.rs`）0 处 `.unwrap()`/`.expect()`（tests/ 除外）。 |
| **元素级验证（诊断 bin）** | `cargo run -p ti-app --features native-shell --bin element-dump -- <scene>` | 逐精灵 blit dump：`记录号 | f1 | 原点 | 精灵w×h | clip_rect 结果 | OK/CLIP/SKIP | 贡献像素`。报「特定区域错位/覆盖菜单」时先跑它定位到具体记录号 + f1 精灵（§7），再对照修复前后清单确认只有越界条从「拉回」变「SKIP」。与第 19 项全帧 diff 互补：全帧锁「整体没漂」，element_dump 定位「漂的是哪个元素」。 |

**数据源变更纪律**：任何 `legacy/data/*.Map` / `*.pak` 改动 → 必须同步跑快照门禁 + 更新基准，否则第 20 项 FAIL。

---

## 6. 证据索引

> 以下为契约结论的**历史证据链**（逆向推演 + 像素实测），仅作「结论怎么来的」溯源。
> 逆向函数地址（`sub_xxxxx`）与伪代码全量归档 `legacy/ref/`，本表只保留行为结论。

| 契约条目 | 结论来源（历史参照） | 置信度 |
|---|---|---|
| 路径表（§1） | `game_app.rs::render`/`render_hd` 源码 + feature 门控 | 高 |
| z-order = 文件顺序（§2） | `evidence/audit_f5_zorder.md`（`.Map` 逐条读入 + 头插链表 + prev 遍历推演） | 高 |
| words[4]=31 非 176（§2 澄清） | `evidence/audit_f5_zorder.md` §3（780 字节流校验） | 高 |
| 越界求交为空不画（§3） | `evidence/audit_zorder.md` A3（8-bit blit 求交早退推演） | 高 |
| Rust 越界拉回是 bug（§3） | `evidence/audit_zorder.md` B4（`sprite_atlas.rs` 越界拉回实测） | 高 |
| PCX index 0 = 洋红哨兵（§4） | `evidence/audit_mag_red_cause.md`（94/120 张调色板实测） | 高 |
| f1=111 越界 3 条（§3 锚点） | `legacy/data/index.Map` 实测（x=185/191/207） | 高 |

**遗留（不阻塞本契约，标推测）**：
- 精灵源裁剪宽/高的确切语义（`a1[10]`）—— 中置信，不影响越界早退逻辑。
- 网格行/列装载值 —— 不影响 .Map 控件 blit。

---

## 7. UI 色彩/动画/交互标准（EXP-079）

### 7.1 色彩单一事实源

- 所有 UI 色彩常量定义在 `crates/ti-app/src/scenes/chrome.rs`（语义色 + v2 chrome 色 + RGBA 变体）。
- `menu_shell.rs` 通过 `pub use crate::scenes::chrome::{...}` 转发，禁止在 `menu_shell.rs` 定义新色。
- 语义色（`COLOR_DANGER`/`COLOR_PLAYER`/`COLOR_LANDMARK`/`COLOR_BG`/`COLOR_TITLE_BAR`/`COLOR_SELECTED`/`COLOR_SECONDARY`）+ chrome 色（`COLOR_PARCH_DARK`/`COLOR_GOLD`/`COLOR_WOOD` 等）统一在 `chrome.rs`。
- HD 8-bit 路径 RGBA 变体以 `_RGBA` 后缀命名，值 `0xAARRGGBB`。

### 7.2 场景切换过渡动画

- `NavHost::enter()` 自动触发 `TransitionState`（`FadingOut` → `FadingIn`），禁止硬切。
- `Framebuf::apply_brightness(factor: f32)` 做全帧亮度调制（per-channel 缩放），`factor` ∈ [0,1]。
- 过渡速度 `step = 0.08`（约 12 帧完成，~200ms @ 60fps）。
- 过渡期间 `GameApp::render` 每帧：渲染场景 → `apply_brightness(当前亮度)` → 更新亮度。
- 静态帧优化兼容：过渡期间每帧亮度不同 → dirty_rows 全置位 → 强制上传。

### 7.3 菜单项三态视觉

- `chrome::menu_item(host, x, y, w, h, chosen, pressed)` 必须传 `pressed` 参数。
- **常态**：羊皮纸底（`COLOR_MENU_BG`）+ bevel 凸起（高光上/左、阴影下/右）。
- **选中**（`chosen=true`）：金底（`COLOR_GOLD`）+ 正弦微光波动（`blend_color` alpha 混合）+ 金色铆钉。
- **按下**（`pressed=true`）：bevel 反转（阴影上/左、高光下/右）= 凹陷实体感。
- 微光动画由 `tick_glow_frame()`（每帧调用）+ `glow_factor()`（正弦）驱动，确定性可复现。

### 7.4 新增 UI 元素必须遵守

- 新色彩 → 加到 `chrome.rs`，同步加 RGBA 变体 + 测试。
- 新可交互元素 → 必须有「常态/选中/按下」三态。
- 新场景切换 → 走 `NavHost::enter()`，自动获得过渡动画，禁止直接赋值 `scene`。
- 新动画 → 用 `AtomicU32` 帧计数器 + 确定性函数（正弦/hash），禁止 `rand::random`；**禁止 `static mut`**（Rust 2024 `static_mut_refs` 编译错误，EXP-079 实测）。

### 7.5 帧级动画与像素快照门禁的分治（EXP-080）

**矛盾**：像素快照门禁（gate 第 18/19/20 项）假设「同场景渲染 = 确定性像素」；帧级动画（微光/过渡/翻页）让像素依赖渲染帧历史。

**分治契约（改任何动画代码前必读）**：
- **体验路径**（实机 `GameApp::render` + 每帧 `tick_glow_frame`）：保留全部动画。
- **快照路径**（`render_to_framebuf`，所有逐像素门禁的唯一入口）：开头必须
  `transition = None` + `chrome::reset_animation_state()`（微光/翻页计数器归零），
  使快照像素与历史帧数解耦。
- **新增帧计数器**：任何新 `AtomicU32` 动画计数器必须在 `reset_animation_state()` 登记归零，否则下一次快照门禁必漂移。
- **基线重生成纪律**：动画改动 = 基线必然漂移（旧基线是旧代码产物，非 bug）。处置 =
  删 （像素基线体系已退役：S18.7b 改结构不变式断言）
  → 首跑门禁自动重新落盘（测试自带「基准缺失→生成 PASS」分支，EXP-076 口径：基线由修复后代码生成）→
  再跑一次全绿才算完。禁止手工画基线 PNG。

---

## 8. 元素级验证法（EXP-076：逐精灵定位「哪个元素错位」）

全帧 diff（`render_invariants` 第 19 项）只能告诉你「有没有漂移」，不能告诉你「哪个元素漂移」。
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
不再临时删。它是渲染契约 §7 的机器工具，与 `render_invariants`（第 19 项全帧 diff）互补：
`render_invariants` 锁「整体没漂」，`element_dump` 定位「漂的是哪个元素」。
