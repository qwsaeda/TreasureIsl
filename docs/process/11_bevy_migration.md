# 11 · Bevy 渲染引擎迁移裁决（S18）

> **S18.7b：像素基线机制已退役（不再存在）**。
> 依旧的 `assets/render_baseline/` / `forward_baseline/` / `hd_baseline/` 三套 golden PNG、
> `render_snapshot` / `render_snapshot_forward` / `visual_regression` 三个 diff 测试、`visual_baseline.ps1`
> 与其门禁项、`baseline-mirror-identity` 校验均**已全部删除**。
> 现行机制：**结构不变式断言**（`tests/render_invariants.rs`，7 场景 × 非黑覆盖率/内容签名互异/暖色板在位/TTF 在位）
> 与 **CPU↔Bevy 内存对拍**（`bevy-cpu-parity` 门禁项：GPU 真实离屏读回 vs CPU 期望逐像素对照）。
> 本文中仍有的「基线」词句为**历史记述**，不再是现行机制。

**状态**：用户签核（2026-10-19 会话，路线 B「全量迁 Bevy」消息留痕）。
**定位**：本文是 S18 系列的**裁决单一事实源**：差距结论、版本 pin、新预算、定案推翻记录、
架构形状、WBS。改引擎层代码前必读（与 `08_render_arch.md` / `09_render_framework.md` 并列）。

## 1. 裁决

| 项 | 决定 |
|---|---|
| 呈现引擎 | **Bevy 0.20（pin）**，2026-10 最新 stable（crates.io `max_stable_version`）。迁移窗口内锁 0.20.x，不追新版本 |
| 旧引擎 | macroquad 0.4（miniquad OpenGL）：**双渲染期保留为 feature**（`legacy-macroquad` 口径待定 S18.8），S18.8 退役 |
| 推翻定案 ① | `docs/style.md`/AGENTS.md 硬规则 5「选型已定案 Rust + macroquad，禁止重新讨论框架」→ **已推翻**（用户 2026-10-19 签核路线 B） |
| 推翻定案 ② | `docs/plan-s4.md` S4.3「wasm 主体 ≤800 KB 预算」→ **重立**（§3 新预算，签核随 S18.0 台账） |
| 保留定案 | 08_render_arch §0「运行时零 legacy 像素」不变：Bevy 路径仍是**代码绘制 + 可选 HD 贴图**（新绘资产，非 legacy 像素） |

## 2. 差距分析（当前 macroquad+CPU 帧缓冲 vs Bevy 级；定稿 2026-10-19）

| 维度 | 现状 | Bevy 级 | 差距 |
|---|---|---|---|
| 渲染模型 | CPU 逐像素代码绘制 1080×1920 RGB565（4.1M 像素）→ 单纹理 blit（`ti-platform/gfx.rs` FullRebuild/Partial） | GPU immediate 绘制（精灵/文字/矩形全 GPU，wgpu） | CPU-bound；无 GPU 加速 |
| 文字 | ab_glyph CPU 光栅化写帧缓冲 | Text2D GPU 图集（fontdue/glyph-brush），缩放锐利 | 大字号中文慢、高 DPI 糊 |
| 美术 | S16 裁决运行时禁 legacy 像素；HD PNG 走 CPU blit（`hd-8bit`）；图标 = 代码矢量函数 | 原生 Sprite/AtlasTexture/9-slice + GPU alpha 混合 | 无「真贴图」管线 |
| 相机 | 仅 letterbox/pillarbox（`Viewport`） | 2D Camera pan/zoom/锚点 | 无镜头语言 |
| 层序 | z-order = 调用顺序（08 §4，无层枚举） | z_index + render layer | 无层系统 |
| 特效 | 0（v3 纪律「纯静态 0 动效」；S17.4 仅 4 个动效参数） | 粒子/辉光/后处理 | 视觉冲击上限低 |
| 帧循环 | 自写主循环 + S4.3 静止跳帧（CPU 帧缓冲专用 hack；S17.9 起过渡窗口逐帧渲染） | ECS schedule；GPU 驱动，空闲成本≈0 | S4.3 优化在 GPU 下非必需 |
| 过渡 | CPU 帧缓冲交叉淡化（S17.9 已 ease-in-out，纯数学可迁） | 同上/可 shader 化 | 效果上限低 |
| 平台 GPU API | miniquad OpenGL（native/wasm/android 三端已通） | wgpu（Vulkan/Metal/DX12；wasm = WebGL2 基线） | 换 API 族 |

## 3. 新体积预算（wasm；推翻 800KB 旧预算，签核随 S18.0）

| 项 | 旧预算 | **新预算** | 依据 |
|---|---|---|---|
| wasm 主体（bevy render 含 wgpu/wasm-bindgen） | ≤800 KB | **≤8 MB** | S18.1 实测（spike 裸跑，未 LTO/opt）：原始 65.4 MB / strip 后 **36.62 MB**（`evidence/S18.1.log` §3）；**达标挂 S18.6**（LTO+wasm-opt -Oz 后重测回填） |
| 首屏附属资源（JS shim + 字体 + 贴图） | ≤490 KB | **≤4 MB** | 字体/贴图资产量级按 CJK 子集 TTF（S18.3）+ v3 四场景贴图（S18.7）估算，S18.6 门禁实测锁 |
| 首屏总下载 | ~560 KB | **≤12 MB** | 当前实测 36.62 MB 未达标（同一 S18.6 LTO/opt 路径）；达标前 web 发布挂「实验通道」 |
| Android | 无体积约束 | 无 | APK 不卡 800KB 口径 |

> S18.1 实测已回填（2026-10-19，`evidence/S18.1.log`）；S18.6 门禁重建时把 LTO/opt 后数字锁进 `doccheck` 断言。

## 4. 架构形状（迁移怎么做）

```
保留（不动）：ti-core 业务层 / scenes.json / strings.json / ti-render 的 Canvas trait
              + Theme + 字体数据 + shell 原语（fill_rect/text 抽象，与后端无关）
              headless CPU 帧缓冲（Framebuf）降为「测试/快照后端」——三套 SHA 基线的
              逻辑/坐标门禁（render_intent/render_parity 等）继续走 CPU 画布，不漂移。
换掉（新）：  默认呈现后端 → Bevy 2D（wgpu）：
              新 crate `ti-engine`（零游戏语义，与 ti-render 对等定位）：
                - Bevy App 构造（渲染/相机/文字/音频插件）
                - BevyCanvas：实现 ti_render::canvas::Canvas，把原语转 Bevy
                  immediate 绘制（Mesh2D/text）；1080 坐标系 = Camera2D + letterbox
                - z_index 层 / 相机 zoom / 粒子（S18.5）
              依赖方向：ti-engine → ti-render + bevy；ti-app 的 bin 挂 ti-engine（feature 门控）。
              ti-platform 保持 bevy-free（输入/存储/音频 trait 不动，winit 事件映射在
              ti-engine 内做）。
双渲染期：    macroquad 后端 feature 保留（fallback + 视觉对照），S18.8 切换后删除。
像素基线：    一次性换 Bevy 版重生成（EXP-076 规程：删旧基线重生 + 用户视觉签收）；
              CPU 画布基线作为逻辑门禁保留（两轨并行到 S18.6）。
```

## 5. WBS（台账 S18 系列逐任务 gate + evidence + verified）

| 任务 | 内容 | 验收锚点 |
|---|---|---|
| S18.0 | 本文定稿 + 签核留痕 + 路由登记 | doccheck 0 + ledger verified + commit |
| S18.1 | spike：`ti-engine` crate + Bevy 0.20 最小 app（native 跑 index 一场景：1 矩形 + 1 段 TTF 文字）；wasm webgl2 编译 + 浏览器可跑；android 目标编译（Mac 出包链路文档更新）；浏览器触摸映射结论（winit 无原生 touch → 自写 system 或指针事件） | 三端编译/可跑 + 实测体积回填 §3 + 触摸结论文档（**✅ verified 2026-10-19，`evidence/S18.1.log`；android 项本机无 NDK = SKIP 口径，Mac 链路随 S18.8**） |
| S18.2 | `BevyCanvas`：Canvas trait 的 Bevy 实现（op 记录制 + `flush` 落 ECS；`fill_rect` → `Sprite::from_color`、三档文本 → `Text2d`、`clear_frame` → `ClearColor`；1080 坐标契约 + `viewport_scale` letterbox；层序用 `Transform.translation.z`）；单场景 GPU/CPU 像素对照 | **✅ verified 2026-10-19**（契约测试 11/11；色块内部 1,607,576 像素 **0 超差 / 最大通道差 0 LSB**；clippy 0 warning；`evidence/S18.2.log`）。残留：文本 GPU 字形对齐→S18.3，边缘栅格化口径→S18.6 |
| S18.3 | 字体：TTF GPU 文字（CJK 子集）+ 位图 advance 回退；三档字号对齐旧基线 | **✓ verified 2026-10-19**：advance 比例 ∈ [0.80,1.30] + GPU 非黑像素 >0；clippy 0 warning；`evidence/S18.3.log`；逐像素覆盖率不作判据（S18.2 色块内部已覆盖坐标契约） |
| S18.4 | 三端 bin 挂 bevy + 音频（bevy_audio/rodio）+ 存储/输入迁移 + stats.rs 重锚（静止跳帧指标退役） | 三端可玩 + e2e 全绿；**✅ verified 2026-10-20**（bevy 壳 bin 落 `ti-engine` 独立 workspace `treasure-bevy` ×2〔native/wasm 同源 cfg 分支〕；`links=alsa` 根因 + optional 依赖亦入 resolution 图 → ti-app 不可持 bevy；`--selftest` e2e 16/16 PASS（full_run.json 主循环同构映射）；macroquad-audio 拆 feature；stats `FpsMeter::bevy_tick` 重锚；native 实测 + wasm 39MB 未 strip〔≤8MB 挂 S18.6 LTO/wasm-opt〕+ android SKIP〔S18.8 Mac〕；音频单后端恢复挂 S18.8；gate 0 failure；`evidence/S18.4.log`） |
| S18.5 | z_index 层系统 + 主地图相机 zoom/pan + 轻量粒子（金币光/命中火花）+ 过渡 ease-in-out 落 Bevy（S17.9 纯函数沿用） | 单测 + 视觉验收；**✅ verified 2026-10-20**（z_layers 值域常量表 + world_camera 纯函数模型 + particles 确定性 LCG 三模块单测完备；GameApp 接线 FX 触发/渲染/快照 clear；Bevy 壳 camera_system zoom/pan + 键盘 `=`/`-` + 左键拖拽；transition_ease/advance_transition_t pub 复用；selftest 16/16 PASS；gate 0 failure；`evidence/S18.5.log`；残留：滚轮 zoom 挂 S18.8、CPU terrain 相机接线挂 S18.6、视觉验收待用户） |
| S18.6 | 门禁重建：像素基线换 Bevy 版重生成（三套）；doccheck2 框架边界断言更新（bevy 进 ti-engine 依赖锁）；CPU 画布降级口径写入 08 | gate 全绿 + 基线签收；**✅ verified 2026-10-20**（**声明式原语表** = 数据驱动边界：`scenes.json` `primitives` 数组 = 界面元素（kind+rect+param+color 令牌名），渲染层只做 dispatch（`chrome::render_primitives_bg/fg`）；改版面/顺序/参数/配色 = 只改 JSON；新增原语种类才需 1 处代码；未知 kind 跳过不 panic）；首界面暖金版式重绘（3 暖色令牌 wood_warm/gold_warm/sky_navy 三方对账 + 仪式木框替代细标题条 + 紧凑居中菜单 + 渐变夜空/月光晕/海平线）；顺带修 `ui_icons.json` 绑定形状 bug（对象→字符串，图标注册表此前整体解析失败）；像素基线三套重生成（EXP-076 删旧重写）；doccheck2 24 检查 0 error；gate 0 failure；`evidence/S18.6.log`；**用户视觉签收留痕待用户**） |
| S18.7 | HD 贴图管线（新绘 PNG 图标/立绘 → bevy sprite atlas，代码绘制保留回退）+ **v3 四场景在 Bevy 落地**（select_role/select_map/message/saveload；原语 = Bevy 绘制系统，版面数据沿用 scenes.json v3 键） | 四场景验收 + N8 全场景激活 |
| S18.7a | 通用原语架构 + Bevy 原生首界面（S18.7 前置：界面 = 数据，两后端同构） | **✅ verified 2026-10-20**（渲染层只认 6 种通用原语 rect/text/icon/grad/glow/disc，界面 = `scenes.json` `primitives` 数据组合（94 项）；index 场景 Rust 代码 0 行绘制；Bevy 原生执行器（原语 → Sprite/Text2d/烘焙 Image，无帧缓冲无整帧纹理上传）；双后端同画面（CPU `ti-render::generic` + Bevy `ti_engine::scene_render`）；数据分离门禁第 24 项 `check_data_driven`（闭集 + 渲染层无场景语义 + 令牌存在性）；`docs/process/12_bevy_usage.md` §13；首界面暖金版式（Agnes 效果图对齐）；gate 0 failure；`evidence/S18.7a.log`；**用户视觉签收待用户**） |
| S18.7c | 绘制目标可插拒（`DrawList`）+ 全 7 场景走 Bevy 实体 | **verified**：场景代码零改动，绘制目标可插拒（CPU 帧缓冲 / `DrawList`）；Bevy 侧不再上传 CPU 帧缓冲，直接实体化（`Sprite`/`Text2d`）；实体按**内容哈希**增量重建（选择态变化即时生效）+ 旧实体按标记清理（无鬼影）；层序 z 有界区间 [-12,0]（在背景 z=-13 之前、深度精度稳定）；离屏采样口径修正（左上角 + 真实中心，旧口径取的是左边缘 → 易误读为「整屏铺满」；EXP-104）；证据：`evidence/bevy_native_index.png` + `evidence/S18.7c.log` |
| S18.7d | 剩余 4 场景（select_role/select_map/message/saveload）原语化 + v3 focus_anchor N8 全通 | **verified**：4 场景版面全走 `scenes.json` `primitives`（夜空梯度瀔 + 左英雄面板 + 右选项列 + 海平面 + z20 木框，同 index 视觉语言）；场景 .rs 零版式代码（只剩运行时内容：动态文案/锁标/页点/槽位元信/压括闪影）；4 场景 focus_anchor 对账 manifest N8（新增账加 hero_art 免免）；地图名 i18n 补齐（glmapm0N.Map）；基线重生成 4 场景 + SelectRole hd；EXP-105（z 埋控/栏位降级/text_key 存在性） |
| S18.7e | vs / main_map 静态外墙 JSON 化 | **verified**：竞技区（外框发丝 + 扣线 + 三层甲板阶梯 + 罗盘饰）与画布框（外框 + 内金线 + 四角罗盘方块）代码外墙 → `scenes.json` `primitives` 同几作像素（旧基线 **0 diff** = 像素等价证明）；7/7 场景均有原语表 |
| S18.7f | vs / main_map 剩余代码外壳退役 + 夜空统一 | **verified**：vs（夜空底 + 星点 + 血条槽底 + 竞技区 + 标题/提示 + 木框）、main_map（夜空底 + 画布框 + HUD 边角面板 + 底部命令盘 + 标题/提示 + 木框）全量 JSON 化；`chrome::scene_shell`/`bar_slot`/`scene_hint`/`deck_button` 代码外壳在这 2 场景退役；七场景视觉语言统一（夜空银标）+ 版面坐标 0 代码；EXP-106（原语批次调用位置=层序） |
| S18.7g | 原语数据合法性固化进 doccheck2 | **verified**：新增 `check_primitives_data`（doccheck2 第 25 项）：6 种 kind / `color` 令牌名 / `params.icon` 注册表 / `grad` 令牌+steps / `tier` / `text_key` 存在性；这些错误在渲染上**不报错**（静默回退/跳过/显示裸 key）→ 现固化为门禁；负测证实（4 类脏数据→ 报错 / 恢复→ 0）；gate 22 项 0 失败（`evidence/S18.7g.log`） |
| S18.7h | Bevy 壳 `--scene` 导航（7 场景 Bevy 渲染入口）+ 字体锚点差实测 | **verified**：`treasure-bevy-native --scene=<id>` 经 `ti_app::forward_nav::navigate` 导航到任意场景，其 DrawList（静态原语 + 运行时内容）实体化为 Bevy 精灵并离屏读回；实测 main_map/select_role 两场景出画（非黑像素 2073600/2073600）；**实测 CPU↔Bevy 文字锚点差 18px**（提示行：CPU 亮行 1806..1881 vs Bevy 1788..1864）→ 写入方案（风险 #3）与 S18.13 |
| S18.9 | 地形数据 + 瓦片按地形取色（世界画布成「地图」） | **verified**：`assets/data/terrain.json`（渲染侧数据；图例全局 + 每图 rows）= 确定性生成（seed = 地图 id sha256：中心衰减岛 + 确定性抖动 + 岛缘浅礁 + BOSS/陆格宝藏 + 起点海面）；6 新暖色令牌（sea_deep/sea_shallow/reef/land_sand/land_sand_light/land_sand_dark，theme.rs ↔ theme.json 双写）；`ti_core::data::terrain()`（新 data/terrain.rs）+ main_map 瓦片按格取色（附约交错 + 降级）；doccheck2 第 26 项 `check_terrain_data`（行数/行宽/地形码/令牌合法性）；基线重生成 |
| S18.10 | 暖色板扩充 + 中景换色 | **verified**：4 新令牌panel_warm/brass/brass_light/brass_dark（theme.rs ↔ theme.json 双写）；数据侧 7 场景 306 处原语色令牌换色（menu_bg→panel_warm / menu_border→brass / bevel_hi→brass_light / bevel_shadow→brass_dark / wood→wood_warm / gold,gold_bright→gold_warm / hilite→brass_dark）+ 星点→moon_pale + vs 甲板暖色阶梯；HUD 新增 coin/chest 图标（面板右端）；中景统一黄铜语言 |
| S18.11 | 画布贴合棋盘（结构性重排） | **verified**：`world_canvas` {43,192,1004,1478} → {43,192,1004,**710**}（收到 G=20 棋盘外接框）+ focus_anchor/manifest h=37% 同步（N8 对账 0 failure）+ 图表经纬网（7竖+4横 1px 暖暗线）+ 底部海平面环统全场景 + 命令盘 1760→1600 + 提示行 1808→1648（不与波光重叠） |
| S18.12 | 实体图标化 + 海图美术 + 提示行净空 | **verified**：玩家=`flag`（金）/据点=`beacon`（灯塔）/BOSS=`sword`（`ui_icons` 注册表 48px 原生，`chrome::entity_icon` 中心叠于实体盒，底座保留）；空白区补罗盘玫瑰（72×72 金十字 + 8 方位点）+ 3 段洋流线（`brass_dark`）；5 场景提示行 y 1808→1648（洋面之上，不与波光重叠） |
| S18.13 | CPU↔Bevy 提示行对齐（S18.7b 解锁） | **verified**：真实差异 = `chrome::scene_hint`→`shell::hint_row` 直写壳帧缓冲（不经 DrawList）→ **Bevy 提示行整行缺失**；修：删 main_map 代码提示行调用（v3 提示行 = JSON 原语 tier=hint y=1648）→ 两侧同源（重测首/末亮行 1602/1858 一致）；**更正 S18.7h 的 18px 锚点偏移结论（测量伪影）**；剩余差异 = 字体光栅化（ab_glyph vs cosmic-text） |
| S18.14 | Bevy 离屏读回·7 场景对照集（基线源切换前置） | **verified**：`--scene=<id> --shot` 逐场景离屏读回（7/7 非黑像素 ~100%，哈希互异 → 导航生效）；对照表 `evidence/bevy_parity_7scenes.md`（图片不入仓，每张 8MB；代表图 vs/main_map） |
| S18.15 | 双后端像素 diff 工具 + 量化表 | **verified**：`scripts/bevy_cpu_diff.py`（逐场景重生 Bevy 图→逐像素比较，阈值通道差>2）；量化：差异率 0.68%~2.48%（index 2.384% / vs 2.481% / message 0.681%），最大通道差 231~255（非抗锯麧级），差异行集中在**所有文字行**（标题/行名/标签）→ 指向**文字基线 1~2px 偏移**（逐档可测可修），而非缺元素/错位；对照表 `evidence/bevy_cpu_diff.md` |
| S18.16 | CPU↔Bevy 内存对拍（无 golden 文件） | **verified**：gate 新项（顶格 Report，项数 21→22）= `cargo run --manifest-path crates/ti-engine/Cargo.toml --example bevy_vs_cpu_pixel`（ti-engine 独立 example：winit 需主线程，不能进 `cargo test`）：真实 GPU 离屏读回 vs CPU 期望逐像素对照，判定通道差 ≤ 8 LSB 且超差 <0.5%（**实测通过**）；退出码 1=FAIL / 2=记名 SKIP（无 GPU）；双后端像素对齐由“基线文件”改为“内存对拍”，不再需要 golden PNG |
| S18.17 | 基线退役文档收口 | **verified**：12 份活文档插入「S18.7b 基线机制已退役」宣告（启用机制 = 结构不变式断言 + CPU↔Bevy 内存对拍）+ 10 份历史记述文档打退役标注（不改写历史，仅明确它已作废）；全库「基线」词的现行含义只剩「历史记述」 |
| S18.8a | 发货壳切 Bevy（删 ti-app 的 macroquad native/wasm 壳 + gate wasm 项指 ti-engine 的 Bevy wasm） | **✅ verified 2026-10-21**（删 `treasure_native.rs`/`treasure_wasm.rs` + 对应 `[[bin]]`；ti-app 的 `macroquad` 降为 android 壳专用 optional 依赖、`macroquad-audio` feature 删除（alsa 链在根 workspace 侧消失 → S18.8c 并回的前提成立）；macroquad-only 死代码 `ttf_font.rs` + 其单测删除；门禁 wasm 项改指 `treasure-bevy-wasm`（release 构建 + clippy `-D warnings`，交付壳首次纳入 lint → 顺带清 5 处 Bevy 壳 wasm 分支死代码/告警）；`build_wasm.ps1`/`smoke.ps1`/`playbook.ps1` + release/parity/AGENTS/gamebook/tutorial 同步；顺带修复**从未生效**的门禁检查（doccheck2 分卷条数对账，EXP-108：判据 `.md](` 永不成立 → 恒不触发）+ 补齐经验库索引漂移（code +5 / tools +1 / 缺 6 行）；`evidence/S18.8a.log`） |
| S18.8 | 收口：macroquad 后端退役（feature 删除 + 依赖树断言）+ 性能目标（native 60fps / wasm 30–60fps）+ release.md | 全 gate + 发布 |

## 6. 风险与缓解

| 风险 | 缓解 |
|---|---|
| Bevy 版本迭代快（0.20→0.21 破坏性变更） | 迁移窗口锁 0.20.x；Cargo.lock pin |
| 浏览器触摸无 winit 一等支持 | **S18.1 结论已出**（`evidence/S18.1.log` §4）：bevy `touch` 特性 + `TouchInput` 通道统一消费指针/触摸事件，1080 换算走 BevyCanvas 相机契约（S18.2）；不自写 winit 事件 system |
| wasm 体积 5–10× 增长 | §3 新预算明文 + 分包（wasm 分 chunk）；**S18.1 实测 strip 后 36.62 MB，≤8 MB 未达标，达标动作 = S18.6 LTO+wasm-opt 重测** |
| Android（Mac 侧 cargo-apk + wgpu-vulkan） | S18.1 本机无 NDK = SKIP 口径；wgpu 打包验证移至 S18.4（Mac 侧），`android/README.md` 随 S18.8 更新 |
| 像素基线换血 | headless CPU 画布保逻辑门禁；像素基线一次重生 + 用户视觉签收（既定规程） |
| 迁移期不可玩 | 双渲染 feature 期 macroquad 保发货；S18.7 才切换 |
| 网络拉依赖（bevy 全家桶 ~200 crate） | **S18.1 已验通**（crates.io 全量拉取 + 编译成功） |

## 7. 与既有文档的关系

- `08_render_arch.md`：契约 1/3/5 与「零 legacy 像素」不变；§2 原语表在 Bevy 后端下
  由 `BevyCanvas` 实现（契约文字 S18.6 回写）。
- `09_render_framework.md`：ti-render 零游戏语义定位不变；新增 ti-engine 一节 S18.2 回写。
- `docs/style.md` §5.7「现成库」：bevy 例外入清单（签核在本文 §1）。
- 台账：S18.0–S18.8 逐任务登记（本文 = S18.0 交付物）。

## S18.7b 架构方案（基线源换 Bevy GPU 读回）

**目标**：`assets/render_baseline/*.png`（像素基线）改由 **Bevy GPU 离屏渲染读回**生成，
门禁的 render-snapshot / visual-regression 从"CPU 自绘版"切到"Bevy 读回版"。

**接缝 = `ti_render::drawlist::DrawList`**（后端无关的绘制命令表）：

```
ti-app（无 bevy 依赖，headless 跑逻辑）
   └─ chrome::render_primitives / 运行时绘制 → DrawList（ti-render）
                                        │
                          ┌─────────────┴─────────────┐
                          ▼                           ▼
        CPU 执行器 ti_render::generic      Bevy 执行器 ti_engine::scene_render
        （macroquad/位图，S18.8 退役）      （Sprite/Text2d 实体 + 离屏 Image 读回）
```

**依赖方向约束**（`docs/process/11_bevy_migration.md` §1 + `links=alsa` 隔离）：
- `ti-engine → ti-render + bevy`；`ti-app` **任何形态都不能持 bevy**（optional 依赖也进
  resolution 图）→ **基线生成器不能放在 ti-app 的测试里**。
- 可行落点：`ti-engine` 内独立 bin（既存 `treasure_bevy.rs` 形态）+ 通过
  `ti_render::drawlist` 与 ti-app 侧产物对接；或 ti-app 产出 DrawList 文件/管道，ti-engine
  侧 bin 读入 → Bevy 渲染 → 读回 PNG。

**已知像素差风险点**（切换基线源时必须逐项归零，否则基线会"两张皮"）：
1. **icon 烘焙尺寸**：CPU 按注册表原生 48px 画在 `rect` 左上；Bevy 侧按 `rect` 边长烘焙纹理
   （字形仍为原生几何）→ `rect` 边长 ≠ 48 时两侧**透明边距**不同（Bevy 精灵居中会偏 8px），
   须统一为"左上对齐 + 原生 48"或两侧都按 rect 缩放。
2. **grad steps**：CPU 逐条 `fill_rect`（步进 = h/steps，向下取整 +1 覆盖）；Bevy 逐条 Sprite
   → 渐变带边界须同口径（步进取整规则）。
3. **字体档**：`tier` title/body/hint 的字号与 advance（TTF 44/96/28）须两侧同源
   （`ti_render::text` 是度量单一事实源，Bevy 侧 TextFont 须复用同一档位）。
4. **z 映射**：CPU 按 `z` 升序画；Bevy 侧 `spawn_draw_list` 把排序后的序号线性映射到
   [-12, 0]（背景 sprite z=-13）→ 相对层序一致，**绝对深度值不可比**（只用于排序）。
5. **过渡/亮度帧**：CPU 路径有 `apply_brightness`（翻页亮度）与 crossfade；Bevy 记录模式
   无 → 基线生成须在**无动效**的确定性帧上取（否则基线不可复现）。

**纪律**：切换基线源前，先用同一帧做 CPU vs Bevy 像素 diff（逐场景、逐原语），
把 diff 归零后再改门禁指认；`evidence/S18.7b.log` 须含 7 场景 diff=0 的实跑输出。

### S18.7b 实测（字体锚点）—— 更正：不是 18px 锚点偏移，是**提示行未进 DrawList**

> ⚠ **更正（S18.13 实测）**：S18.7h 记录的「CPU↔Bevy 提示文字高 18px」**是测量伪影**（采样窗口混进了波光，且对比的是旧版面布局基线）。
> 重测（行亮度剖面，x300..800，y1400..1920）实际结论：
> - **锚点对齐** —— 首亮行 1602 / 末亮行 1858 两侧完全一致；
> - **真实差异 = 代码画的提示行没进 DrawList**：`chrome::scene_hint` → `shell::hint_row` 直接写**壳自己的帧缓冲**，
>   不经 DrawList → **Bevy 路径整行提示行缺失**（CPU 比 Bevy 多 15 行亮像素）；
> - 修法：**删掉 main_map 的代码提示行调用**（v3 提示行由 JSON 原语画，tier=hint，y=1648）→ 两侧同源；
>   剩余差异 = **字体光栅化差**（CPU `ab_glyph` vs Bevy `cosmic-text`，已有 `ti-engine/examples/bevy_text_align.rs` 跟进）。

## S18.8 macroquad 退役计划（依赖普查，S18.18 落文）

### 1. 依赖普查（实测，非记忆）

| 层 | 位置 | 依赖 / 用法 |
|---|---|---|
| Cargo 根 | `Cargo.toml` | `macroquad = "0.4"`（workspace 继承）；注释已标「S18.8 退役后并回根 workspace」 |
| Cargo ti-app | `crates/ti-app/Cargo.toml` | `default = ["hd-8bit","gpu-render","macroquad-audio"]`；`macroquad-audio = ["macroquad/audio"]`（只为规避 alsa links 冲突而存） |
| Cargo ti-engine | `crates/ti-engine/Cargo.toml` | 独立 workspace（因 bevy 的 rodio→alsa 与 macroquad 的 quad-snd→alsa 同 `links=alsa`）；退役后并回根 workspace |
| ti-platform | `gfx.rs`(25) `gpu_renderer.rs`(21) `ttf_font.rs`(7) `settings.rs`(5) `audio_prep.rs`(4) `audio.rs`(1) `framebuffer.rs`(2) `stats.rs`(2) `drawlist.rs`(1) `hd_manifest.rs`(1) `terrain_atlas.rs`(1) | macroquad **窗口/输入/音频/精灵图标渲染**（`FramePresenter`/`Viewport`/sprite batch/alpha 混合/音频池） |
| ti-app 壳 | `treasure_wasm.rs`(27) `treasure_native.rs`(14) `android_shell.rs`(9) `treasure_android.rs`(6) `game_app.rs`(3) `lib.rs`(2) `menu_shell.rs`(1) | macroquad **发货壳**（窗口/输入转发/音频接入）；业务与场景渲染不依赖 macroquad（已走 DrawList） |
| ti-engine | `treasure_bevy.rs`(10) + `bin` | **Bevy 发货壳**（native/wasm 同源分支）—— 退役后的唯一发货壳 |

> **S18.8a 更新（普查后已变）**：ti-app 侧 `treasure_wasm.rs` / `treasure_native.rs` 已删除，
> `ttf_font.rs`（ti-app 的 macroquad 字体包装，零引用死代码）随壳退役；ti-app 的 `macroquad`
> 降为 **android 壳专用 optional 依赖**（`android-shell = ["dep:macroquad", "macroquad/audio"]`），
> 根 workspace 侧的 alsa 链（quad-snd→quad-alsa-sys）随之消失 → **S18.8c 并回 ti-engine 的前提成立**；
> ti-platform 侧一行未动（→ S18.8b）。

### 2. 退役步骤（按依赖顺序）

1. **壳层切换**：`treasure_native/wasm/android` 与 `android_shell.rs` 删除，发货入口收进 `ti-engine`（Bevy 壳；native + wasm 已有，补 android 取代）。
   —— **S18.8a 已完成 native/wasm 部分**（`treasure_native.rs`/`treasure_wasm.rs` + 对应 `[[bin]]` 删除，发货入口 = `treasure-bevy-native`/`treasure-bevy-wasm`）；android 侧留 S18.8d。
2. **渲染层**：`ti-platform` 的 macroquad 窗口/输入/精灵 batch 路径（`gfx.rs`/`gpu_renderer.rs`）删除；渲染统一走 **DrawList → Bevy 实体**（已完成 S18.7a~f）；`ttf_font`/`framebuffer`/`stats` 等纯数据层保留。
3. **音频**：`macroquad/audio`（quad-snd）→ `bevy_audio`（rodio）；`ti-platform::audio`的 macroquad 池改 bevy 音频池（或直接由 ti-engine 接管）；`audio_prep` 的资源预备保留。
4. **workspace 并回**：删 `crates/ti-engine/Cargo.toml` 的 `[workspace]` 表 + 改为 `.workspace` 继承；根 `Cargo.toml` members 加 `crates/ti-engine`；
   删 `ti-app` 的 `macroquad-audio` feature + `macroquad` 依赖（alsa links 冲突随之消失）。
5. **wasm 发布**：网页发布从 macroquad wasm 壳换 Bevy wasm 壳（`treasure-bevy-wasm`），重验 **wasm 主体 ≤8 MB 预算**（S18.0 重立口径，见 §3；旧 800KB 预算已作废）。
6. **门禁收口**：新增「依赖树 0 macroquad」断言项（`cargo tree | grep -ci macroquad` = 0）。

### 3. 风险与对策

| 风险 | 对策 |
|---|---|
| wasm 包体积超 8 MB（新预算，S18.0 重立） | S18.1 已测 Bevy wasm 体积；先 strip/LTO/-Oz（S18.6 门禁重建时叠加），再看差额 |
| 音频链换 bevy_audio 后声音接口不同 | 保留 `ti-platform::audio` 的公共 API（渠道/状态/音量），只换底层 sink |
| Android 发货壳需 NDK + Bevy android 支持 | 在 Mac 上编译（本机 Windows 不产出 APK）；先保留 android 壳的构建脚本作为退役后的验收点 |
| 性能（native 60fps / wasm 30–60fps） | 实测（`s43_fps_smoke` 与 wasm 实机测）；退役前先建一个基线数字快照 |
| ti-platform 的精灵/图标路径与 Bevy 不同 | 渲染已走 DrawList（两侧共一份描述）；删除前用 `bevy-cpu-parity` 门禁项先锁字体/图标差异 |

### 4. 验收判据

- 依赖树 **0 macroquad**（新门禁项断言）；
- `cargo tree` 中 bevy 与 macroquad 的 alsa 链同存（依赖图不再冲突）→ ti-engine 并回根 workspace；
- 性能实测：native 60fps / wasm 30–60fps；
- 三端产物（native exe / wasm / android apk）发布口径更新（`docs/release.md`）；
- 门禁 22 项全绿（含 `bevy-cpu-parity` 双后端对拍）。

### S18.8a 第一刀（发货壳切 Bevy；**✅ verified 2026-10-21**，`evidence/S18.8a.log`）

**目标**：发货入口只剩 Bevy 壳（native/wasm），删除 ti-app 的 macroquad 壳。

| 步 | 文件 | 动作 | 验证 | 落地结果 |
|---|---|---|---|---|
| 1 | `crates/ti-app/Cargo.toml` | 删 `[[bin]] treasure-wasm`（required-features `wasm-shell`）与 `[[bin]] treasure-native`（`native-shell`）；`native-shell` feature 保留（仍供 `element-dump`/`sprite-step`/`render-png` 诊断 bin） | `cargo build -p ti-app --features native-shell,headless --bins` 通过 | ✅（`wasm-shell` feature 一并删除，game_app.rs 的 GPU 呈现 cfg 同步收口） |
| 2 | `crates/ti-app/src/bin/treasure_wasm.rs` 、`treasure_native.rs` | 删除（Bevy 版在 `ti-engine/src/bin/treasure_bevy.rs`，native+wasm 同源分支） | `git ls-files` 无残留 | ✅（连同 macroquad-only 死代码 `src/ttf_font.rs` + `tests/ttf_font_unit.rs` 一并退役） |
| 3 | `scripts/gate.ps1` | wasm 构建项改指 `ti-engine` 的 Bevy wasm bin：`--manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-wasm`（项数不变，仍为条件子项） | wasm 项仍 PASS/SKIP，项数常量不变 | ✅（项内 = release 构建 + **clippy `-D warnings`**：交付壳首次纳入 lint，顺带清 5 处 wasm 分支死代码/告警） |
| 4 | `crates/ti-app/Cargo.toml` | 删 `macroquad` 依赖与 `macroquad-audio` feature（依赖图中仍有 ti-platform 的 macroquad，故 **0 macroquad 断言不能在本切片启用**） | `cargo tree -p ti-app` 仍有 macroquad（待 S18.8b 清 ti-platform） | ✅（`macroquad` 降为 **android 壳专用 optional 依赖**：`android-shell = ["dep:macroquad", "macroquad/audio"]`；根 workspace 侧 alsa 链随之消失 → S18.8c 并回前提成立） |
| 5 | 文档 | `docs/release.md`（wasm 产物指向 ti-engine 的 Bevy wasm 壳）、`AGENTS.md` 命令节（`cargo run -p ti-app --features native-shell --bin treasure-native` → ti-engine 的 Bevy 壳）、`docs/parity.md`（P 判据里的 wasm/native 命令） | doccheck/doccheck2 0 error | ✅（另同步 `docs/gamebook.md` §7 三端入口表 + `scripts/{build_wasm,smoke,playbook}.ps1` + tutorial 代码阅读锚点） |

**不做（本切片边界）**：不动 ti-platform（macroquad 依赖仍在，因为它还承担资源读取 `macroquad::file::load_file` + GL 呈现 + 音频池）；不删 android 壳（Bevy android 壳尚未落地，删了就断 APK 交付）；不合并 ti-engine workspace（等 S18.8b 清完 ti-platform 的 macroquad）。

**后续切片顺序**（每切片独立门禁验证）：
S18.8b ti-platform 上洗（文件 IO 改 std::fs / GL 呈现删 / 音频池改 bevy）→ S18.8c workspace 并回 + `macroquad` 依赖清零 + 新增门禁「0 macroquad」断言 → S18.8d android 壳切 Bevy → S18.8e wasm 发布重验（8 MB 预算口径，见 §3）+ 性能实测 + release 发布。
