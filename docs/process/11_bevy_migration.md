# 11 · Bevy 渲染引擎迁移裁决（S18）

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
