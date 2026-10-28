# AGENTS.md — 路由表

项目：Windows Mobile 游戏《金银岛 TreasureIsl》(ARM 原生) 的**逆向 + Rust 现代化移植**，目标 WASM(浏览器) + Android 双端。
背景计划不在此处，按任务路由读文档，**不要凭默认习惯开工**。

## 当前状态

- 根目录：`TreasureIsl.exe`（ARM PE32）、`TreasureIsl.exe.i64`（IDA Professional 9.4，已自动分析、**无自定义命名**）、`legacy/`（逆向产物归档：`exe`/`i64`/`data`/`ref`/`docs`）、`Cargo.toml`（6 workspace member = 6 开发 crate）、`scripts/`、`docs/`、`android/`、`plan/ledger.yaml`、`evidence/`、`ref/`。
- 工程已 bootstrap：`crates/{ti-formats,ti-core,ti-platform,ti-app,ti-tools}`；进度只由 `plan/ledger.yaml` 驱动。
- **S15 业务场景穷举测试阶段**：S0–S14 全 verified（含 S9.1–S9.5 全 verified、S10.0–S10.5 全 verified、S11.1–S11.4 全 verified、S12.0–S12.8 全 verified、S13.1–S13.6 全 verified、S14.0 verified）；S15.1 verified，S15.1 verified、**S15.2 verified**（跨图推进规则定稿 + 落地，S15.2 残留 2 项已闭环 = 通关门禁 `is_cleared` 逐图 + 存档 `unlocked_mask` 逐图写入，见 `evidence/S15.2.log [8]`）。S16（试玩闭环 + 发布 + 设计 sheet）待开工（`docs/plan-s16-loop.md`，WBS S16.1 起）。
  逆向产物已物理归档 `legacy/`（S11.1 verified）；oracle 参照实现唯一源 = `ti-core/src/oracle`（`pub mod oracle` 公开导出，测试直接 `use ti_core::oracle::`）；逐字节重复 crate `legacy/oracle-rust` 已于 S15.8 删除（C4 反向守卫防重引入）。
  模块并行开发 loop 已固化（`docs/process/05_module_parallel.md`）；飞轮流水线已落地（`docs/process/06_flywheel.md` + gate 第 17 项）。
- **S19 环岛赛道（大富翁式摇骰前进）**：主地图主玩法 = 摇骰定步数 → 沿环岛赛道前进 N 格 → 落格结算（`rules::board` + `assets/data/board.json` + `balance.board`）；**S19.1 方向键自由移动已彻底删除，主地图移动只保留摇骰（`GameInput::Spin`）**。单一事实源 = `docs/business/progression.md` §环岛赛道 + 路由表 S19 条目。
- **S21 main_map 重设计 v4「羊皮海图桌面」+ 船系统**（2026-10-22 用户：main_map 界面很烂 → 出新的设计渲染图）：**S21.0（脚本纪律强制化：doccheck2 第 27 项工具契约门禁 + `ui_asset_batch.py` 批量资产工具 + 硬规则 10）与 S21.1（概念稿 3 变体 + 15 资产多形状节点批次）已 verified**；**S21.2 船系统规格已定案 verified**（B 独立船轴：4 级 小舢板→双桅→三桅→巡防，造价 0/800/2500/6000，货舱 99/150/240/400 = 单次贸易上限，atk/hp 累计叠加建筑轴；单一事实源 = `docs/business/economy.md` §船系统 + `balance.boat`）；**S21.3 main_map v4 版面/交互规格已定案 verified**（满屏 `chart_full` + 顶角双铜牌 + 岛只画陆/礁/宝 + 黄铜航线多形状节点牌 + 船玩家 + 264×264 舵轮 CTA；`world_canvas {43,250,994,826}` ↔ `bbox_pct {4,13,92,43}`；坐标/交互/资产接线单一事实源 = `docs/ui-redesign-v3/08_main_map_v4_spec.md`，镜像 `business/08_layout_design.md` §4.3）；**S21.13 流程固化待固化项落地已 verified**（15_process_ledger §2 待固化 7 行全部转已自动化：①doccheck 读失败即 ERROR ②`scripts/doc_numbers.py` 门禁项数/台账计数实测回写 ③`doccheck2::check_visual_design_signoff` 视觉签收 ④`scripts/layout_spec_check.py` 版面规格三面对账 ⑤`doccheck2::check_node_kind_asset_map` 节点 kind↔资产唯一源 ⑥`scripts/check_ps_scripts.ps1` PS 入口 BOM+语法 + `doccheck2::check_no_inline_test_mod` src 内联单测拦截（47 存量 `process_ratchet.json` 棘轮豁免）；doccheck2 34→12 项）；**S21.12 verified**（建造面板 5 入口造船 + 船贴图×4 + `boat_level` 船标切换，`evidence/S21.12.log`）；**S24.0/S24.1 verified**（相机 `o.scale` 语义 + 点击绝对光标，EXP-116/117）；**代码标准三件套落地**（`docs/process/16_code_standards.md` + `check_code_standards.py` CS-1~4 + 棕轮基线，S25.1 免用清零待开工）；**用户 2026-10-23 拍板：4 条设计分歧全部→图片化**（④-04 = 14 枚代码图标改图片资产 + 跨层 tint 支持，见 `docs/process/06_flywheel.md` §8 飞轮 R1 findings ①-01/①-12/④-04/⑤-04；**④-04 已落地 = S21.14 verified**：14 白蒙版 `assets/ui/icon_*.png` + 几何规格 `assets/ui_batches/icon_v1.json`（抄自 `icons.rs`）+ `blit_image_tint` 跨层 tint（CPU×tint multiply ≡ Bevy `Sprite.color`，白 tint 恒等），生成/校验归 Python `scripts/ui_icons.py`（工具契约 3 处登记）+ `validate_data.py` V3，Rust 只留运行时 tint 契约，`icons.rs` 代码矢量降级为缺图回退）+ 飞轮 R2（渲染硬编码/数据图片分离，停机=0 新发现）。待开工 = S21.4~S21.7（岛屿重绘 → 赛道重绘 → 版面重排 scenes.json/manifest → 交互升级）+ S21.10（规则接线 rules/boat + qty 上限 + 战斗加成）→ S21.11（存档 v3）→ S21.8（存量豁免清零）+ S21.9（出图验收）。**S22 首页重设计「罗盘环标题页」（C 方向 + 羊皮海图底，用户 2026-10-23 拍板：6 菜单沿罗盘环 + 中心舵轮 logo + 羊皮海图与主地图 v4 统一；概念稿 `evidence/ui_concept/index_compass_{layout,concept}.png`）** 待开工。批次规格 = `assets/ui_batches/main_map_v4.json`；概念稿 = `evidence/ui_concept/main_map_v4_{a,b,c}.png`（gitignored，仅参考）。

- **S26 全场景视觉收尾**（2026-10-27 用户：地图是垃圾/裸色块/低对比/黑底面板「全做」）：**S26.1 verified**（ICU4X 报错根治：bevy `complex_script_segmentation` 启用 parley 完整 CJK 字典 + ti-engine `autobins=false` 收口 resizetest，实跑 ICU 行 0 vs 改前 ~24/场景，`evidence/S26.1.log`）；**S26.2 verified**（main_map v4 岛屿图片化：`scripts/ui_islands.py` terrain→3 枚暖沙岛屿剪影 PNG 资产优先 blit + 黄铜轨逐格骰点航线 + 赛道节点/船尺寸放大，`evidence/S26.2.log`）；**S26.3 verified**（七场景视觉：vs 删裸色块日志占位 + HP 槽黄铜描边、saveload 选中态提亮 + 修「位 1」角色前缀、message 帮助正文占位填真实文案 + 标题对比、select_map 黑底面板 panel_warm→panel_wood 暖棕 + 蓝锁条→暖棕、chrome 全场景底部提示统一 menu_bg 底板条；7 场景出图 `evidence/bevy_native_*.png`，`evidence/S26.3.log`，**视觉签收待用户过目**）。待开工 = S21.4~S21.7 残留（若 S26.2 已覆盖则核对）+ S25.1（代码标准棘轮豁免清零）。

- **S27/S28 主地图 v5「大富翁式环形格子 + 领地系统」（2026-10-27 用户：地图设计得差 → 参考大富翁但要海盗风，各类岛屿都不知道；「每格有岛屿/城镇，落到这格可进副本，别人占领可交钱或攻占，自己的领地可升级加固」= 原版玩法；「每局 4 个 AI 玩家」+「可以选至少一个玩家」）**：**S27.0 设计稿已出**（`evidence/ui_concept/main_map_v5_{a,b,c}.png`，选定 = 变体 B 圆环卡片 + 中央大岛插画）；**S27.1 版面规格已定案**（`docs/ui-redesign-v3/10_main_map_v5_spec.md`：31 张 96×120 岛型卡环（环心 540,780 / R=474）+ 中央可点击据点岛 760×760 + 归属色带 + 10 类海盗岛 + 点卡交互契约）；**S28.0 领地业务规格已定案**（`docs/business/territory.md`：5 席位 = 选角 ≥1 玩家 + 4 AI、领地格 vs 功能格、10 类岛专属副本、落格三支分叉（无主→副本占领 / 他人→交钱或攻占 / 自己→升级加固）、经济接入、存档 v4、可测规则表 13 条）；**S27.2 资产已 verified**（10 类岛卡 + 3 张中央插画，`evidence/S27.2.log`）；**S27.4 表现层环形棋盘已 verified**（31 张 96×120 岛型/功能卡环 + 中央据点岛 + CTA/三键下移 + 几何/资产**数据驱动**（`main_map.ring.*` + 命名约定卡面），`evidence/S27.4.log` + `evidence/bevy_native_main_map.png`；点卡命中/分区色带 = 归 S28）；**S30.1 地图打包工具已 verified**（`scripts/map_pack.py` + `assets/maps/{m01,m02,m03,_template}.json`：加图/换图/改环几何 = 改描述符 + 一条 `sync`，不改 Rust、不散改多份 JSON，`evidence/S30.1.log`）。待开工 = S27.3 数据层残留（board.json 分区色令牌；31 格已由 S28.3 落）→ S28.2 存档 v4 → S28.4 选角多席位 → S28.7 AI 回合 → S28.6 副本场景 ×10（直接按 S29 布局树新建，前置 S29.2）→ S27.5 出图签收（v5 视觉终态）。

- **S29 布局引擎「B1 全 bevy_ui 响应式」（2026-10-27 用户：「有引擎确不用的问题，布局还是写死的」→「真响应式」→ 澄清 = **Bevy 引擎自带响应式，非自造求解器** → 拍板 B1：UI + 命中 + 重排全进 Bevy）**：**S29.0 定案已 verified**（`docs/process/17_layout_engine.md` 单一事实源：B1 架构 + 3 条约定松绑签核（逻辑层仅 UI 交互层进 Bevy / `bevy-cpu-parity` 改口径只保游戏世界 / UI chrome 转 Bevy 代码而游戏内容仍 1080 数据驱动）+ 换引擎成本重估）；**S29.0-void 已 verified**（删遗留求解器 `ti-render::layout`，原「引擎无关求解器」方向作废）；**S29.1 已 verified**（bevy_ui 基建 + 最小验证：bevy `features+=["bevy_ui"]` + 新 `src/ui.rs` 零游戏语义〔`spawn_ui_camera` `IsDefaultUiCamera` order=1/clear None + `build_proof_panel`，`ChildOf` 层级、不挂已废弃 `Button`〕+ 壳 `spawn_ui_chrome` 接线 + `ui_probe_system` ComputedNode 探针（`[ui] bevy_ui probe: node=5 text=2 abs=1 computed_layout=5` 证明 taffy 自动布局真跑）；`12_bevy_usage.md` §9.8 坑位；`evidence/S29.1.log`）；**S29.2 已 verified**（bevy_picking 命中层 + touch.rs 手写几何全退 + doccheck2 改口径：`ti-engine/src/pick.rs`〔`PickTarget`/`PickEvent`（`Message`）/观察者/`pick_flush`/`swipe_dir_from`，零游戏语义〕+ `scene_render::spawn_pick_layer`（`scenes.json` `pick_key`/`pick_index` 原语→透明可拾取标记）+ 壳 `register_pick`/`pick_event_system`→`poll_hit(HitData)` + 退役 `TouchPad`；ti-app `HitData`/`poll_hit`/`route_hit` + 7 场景 `on_hit`（index/select_role/select_map/saveload/message/vs 全接，main_map key4~7 基础臂）；`touch.rs` 删 `layout` 几何仅留手势 `TouchPad`/`TouchSample`/`Gesture`；3 headless 测试 + e2e 迁 `HitData`；doccheck2 EXP-071 改口径；活体拾取派发 + 主地图赛道/环格 `pick_key` 归 S29.5/S27.4/S29.3；`evidence/S29.2.log`）；**S29.3a 已 verified**（S29.3 第一增量 = select_role 列表 chrome 迁 bevy_ui + **3 个根因修复**：①**bevy `bevy_ui` 特性只含布局核心，渲染管线在独立特性 `bevy_ui_render`**（`DefaultPlugins` 仅在后者下挂 `UiRenderPlugin`）⇒ S29.1~S29.2 期间 UI 层**从未渲染** = 用户报「跑起来还是老程序」的真根因 ②UI 相机是普通 `Camera2d` 须 `RenderLayers::layer(1)` 隔离（否则重渲 1080 世界/清掉离屏目标）③`TextFont.font` 是 `FontSource` 枚举（不给字体 = 中文豆腐块）；落码 = `ui.rs` `build_list_chrome`（flex column 竖排 + `ImageNode` 缩略图 + 选中底色，返回行实体供挂 `PickTarget`）+ 壳 `chrome_ui_system`（按场景/选中重建）+ ti-app `RenderHost::chrome_in_bevy_ui`（默认 false ⇒ CPU 后端照旧）+ `scenes.json` 20 原语加 `ui_layer` 标记（**坐标未动**）；判据 = `--shot select_role` 出图 bevy_ui 列表与 1080 立绘**同帧正确叠加**（`evidence/bevy_native_select_role.png`；探针 `node=27 text=10 computed_layout=27 max_node=1080x1920`）；文档拆分 = 新增 `19_bevy_ui.md` + `20_bevy_picking.md` 两篇 Bevy 专题（`12_bevy_usage.md` 保留总览+通用）；`evidence/S29.3a.log`）；**S29.3b 已 verified**（S29.3 第二增量 = P0「菜单按键无响应」双根因根治 + saveload 迁 bevy_ui：①bevy_ui 全屏根/容器节点默认 `Pickable` 挡掉下层 1080 命中（`build_hover_map`）⇒ 证明面板/列表 chrome 全节点挂 `Pickable::IGNORE`；②更深根因 = `bevy_picking` mesh 射线只拾取 `Mesh2d/3d/SimplifiedMesh`（`MeshFilter`），1080 拾取标记是 `Sprite` ⇒ **从未被拾取** ⇒ index/select_map/saveload 1080 命中全死。落码 = 纯几何 `pick::hit_1080_pick`（窗口逻辑 px 经 letterbox 换算 1080 → 取包含点且 z 最大 `pick_key` 矩形）+ 壳 `manual_1080_hit_system`（左键 tap → `PickEvent`，与 bevy_ui 命中层不双发）+ saveload 5 槽列表迁 bevy_ui（`build_list_chrome_pos` PanelSide::Left + i18n 槽文案 + 光标选中态 + `PickTarget` key=3）+ `saveload::render` 加 `chrome_in_bevy_ui` 守卫 + scenes.json 5 槽标 `ui_layer`；`tests/pick_1080.rs` 4 条命中几何无头判据；`20_bevy_picking.md` 坑位 20-8/9/10；判据 = gate 全绿（仅 fps 环境项）+ e2e_touch_economy_loop + `--shot saveload` 出图 1080 夜空底 + bevy_ui 左槽面板同帧叠加（`evidence/S29.3b.log`））；**S29.3c 已 verified**（select_map 3 卡片迁 bevy_ui：`build_select_map_chrome` `PanelSide::Right` 卡片名 i18n `glmapm0N.Map` + 锁/开放态 `scene.select_map.locked/open`（新增 2 i18n 键）+ 选中高亮 + `PickTarget` key=2；18 卡片原语标 `ui_layer` + `select_map::render` 守卫；`--shot select_map` node=15 text=7 出图，`evidence/S29.3c.log`）；**S29.3d 已 verified**（index 首页 6 徽章径向环迁 bevy_ui：`build_index_chrome` 参数化径向环（环心/半径/60° 步进算 6 格中心 `%`）+ 徽章底图**数据驱动**（读 index 场景 `params.asset`，壳侧不硬编码资产路径）+ 环尺寸走 `LOGICAL_*` 常量 + 新增 i18n 键 `scene.index.medallion.0..5` + 30 徽章原语标 `ui_layer`；背景/舵轮/标题仍 1080；`--shot index` node=25 text=6 出图，`evidence/S29.3d.log`）；**S29.3e 已 verified**（message 帮助面板迁 bevy_ui：`build_message_chrome` + `msg_text_child` 助手 + 翻页重建；**vs/main_map 遮挡判定 = 留 1080**：bevy_ui UI 相机叠在 1080 世界相机之上 ⇒ 同屏位 bevy_ui 节点遮住 1080 动态内容（vs HP 条 / main_map 棋盘+实体+HUD 值），两场景 chrome 与 world 层叠 = 游戏世界场景，B1「游戏世界保持 1080」⇒ 整体留 1080；判定记 `19_bevy_ui.md` 坑位 9.8-13；`evidence/S29.3e.log`）；**S29.3 主体收口结论 = 5 纯 UI 场景全迁 bevy_ui（index/select_role/select_map/saveload/message）+ vs/main_map 游戏世界留 1080**；**S29.4 已 verified**（门禁改口径：`bevy-cpu-parity` 加「UI 层排除」= 对照结束**机器断言** bevy_ui 节点数=0 + gate 项描述改「仅游戏世界」；`check_scene_narrative` 口径写明（原语 rect 仍是 1080 版面规格，`ui_layer` 只表达呈现层归属，N1~N8 判据照旧）+ 四篇文档与 B1 对齐：12_bevy_usage（§9.4 陈旧项退役 + §9.8 bevy_ui **已启用** + 多相机口径）/ 11_bevy_migration（S29 UI 层补充段）/ 08_render_arch（**契约 0b** 两层不混坐标系 + 叠压遮住约定）/ 09_render_framework（chrome 归属）；`evidence/S29.4.log`）；**S29.5 已 verified**（收口：`--resizetest` 加**第 3 不变量**（bevy_ui 布局根 = 视口尺寸 + 节点数不变 ⇒ **真重排**，实测 540×960 → 1080×1920 且 25 节点不变）+ **抓出并修复潜伏缺陷**「多相机致正交 scale 静默不更新」（S29.1 起 `camera_system`/`resizetest` 的 `single()/single_mut()` 多实体失败 ⇒ 窗口缩放跟随失效；修法 = 显式 `WorldCam` 标记）+ `--shot --viewport=WxH` **多视口出图**（5 场景 × 3 视口 = 15 张；世界 contain 缩放 + chrome 随视口重排）+ 游戏世界像素对拍 **0 超差 0 LSB** + `17_layout_engine.md` §3.1 换引擎成本实测回填（锁定面 + 28 项必重解的坑）；`evidence/S29.5.log`）；**S29 全部 verified**（B1 全场景 UI + 命中 + 真重排达成）。待开工 = S27.4（main_map v5 参数化地图）+ S28.6（10 副本场景直接 bevy_ui 建）+ S30 资产管线延伸。游戏世界保持 1080 正交 + 数据驱动。

- **S30 资产/内容数据管线加固（2026-10-27 用户：「没有统一资产管理/资源加载标准？scenes.json 手写不行，行业怎么做（地图编辑器等）」→ 定案「不手写像素：声明意图/数据，引擎算坐标」）**：**S30.0 已 verified**（`docs/process/18_asset_content_pipeline.md` 单一事实源：行业三档 ①绝对坐标/②意图+自动布局/③参数化 + 总纲「不手写像素」+ 本项目终态 UI=bevy_ui 自动布局 S29.3 / 地图=参数化 S27.4 / scenes.json 退成元素清单；**①统一资产登记** = `assets/registry.json`（字体/音频，UI 图沿用 `ui_manifest.json`）+ gate `validate_data::check_asset_registry` 全类型 disk↔registry 双向同步；**②scenes.json 序列化纪律** = gate `check_scenes_canonical` 守 indent=1 逐字节规范形（防整文件 churn，S29.2 曾误改 5672 行）；**③音频口径** = 发货无声（Bevy 双渲染期关，registry.audio 空，bevy_audio=S18.8）；回写遗留 EXP-124/125；`evidence/S30.0.log`）。**S30.1 地图打包工具已 verified**（`scripts/map_pack.py` + `assets/maps/<id>.json` 描述符 + `_template.json` 共享版面：`sync` 幂等回注 board.json/scenes.json/manifest + 资产登记对账，`--check` 漂移非 0；「不手写像素」③参数化执行面，`evidence/S30.1.log`；**S31.1 起扁平描述符退役**，归档 `legacy/data/maps_s301/`，见 S31 行）；S27.4 地图参数化表现层已 verified（见 S27 行）。待开工 = S29.3 根治（UI 坐标出 scenes.json 绝对像素 → bevy_ui 意图）。

- **S31 地图包（Map Package）标准 v1（2026-10-28 用户：「现在是打包规范，标准在哪，打包的产物在哪」→「我让你制定的标准」→ 拍板 B = 运行期换图全链路 → 执行顺序 ①文档规范 ②打包工具 ③全部对齐）**：**S31.0 规格定案已 verified**（`docs/process/21_map_package_spec.md` 单一事实源：一张图 = 一个可分发包 `assets/maps/<id>/{manifest,map,layout,strings}.json` + `.default` 默认包指针 + `_template/` 骨架；资产双来源 `source=pack/pool`（专属入包自带，共享池 `assets/ui/` 登记 + 哈希照验）；I1~I8 不变量（31 格 = start1+territory20+功能9+boss1 / 10 类岛各 2 格同区 / **BOSS 在环半周 spaces[31//2]**（非末格，实测三图 boss@15）/ 8pt / 资产哈希 / N8 焦点 ±1px）+ 版本兼容（`map-pack/1` + `board-v5` 不符即拒）+ 失败回退（缺包 → 构建期内嵌默认包数据）；§7 运行期加载契约（选 B）+ §8 工具流水线；18/17/14/04/README/路由表全对齐，`evidence/S31.0.log`）；**S31.1 工具 + 产物已 verified**（`scripts/map_pack.py` 8 子命令 pack/verify/install/uninstall/list/add/check/sync + 已装包 `assets/maps/{m01,m02,m03}/` + `dist/maps/*.map-pack(+zip)` 可分发件（含池资产副本 + POOL_INDEX 哈希索引）；verify 0 违规 / check 0 漂移 / dist zip verify 过；S30.1 扁平描述符归档 `legacy/data/maps_s301/`，`evidence/S31.1.log`）；**S31.2 运行期换图·数据注入层已 verified**（`ti-core::data::map_pack`（包路径直读 + manifest schema/版本/id 三拒 + 三段表**内嵌默认表 upsert 合并**（注入 m02 不丢 m01/m03 内嵌图，只替换本包段））+ `table_source` 注入接缝（首访前 `install` 生效 / 迟注入忽略告警；`board()`/`scenes()`/`strings()` 三访问器 hook，未注入 = 内嵌 0 行为变化）+ 壳 `apply_map_pack`（`TI_MAP=<id>` > `.default` 指针 > 纯内嵌，逐段 precheck 缺段回退）+ 7 项集成测试 + `build_wasm.ps1` 4e 段部署链（`assets/maps/<id>/` → `web/assets/maps/<id>/`）；`TI_MAP=m02 --shot main_map` 真渲染 0 硬错误 + 缺包回退 0 panic，`evidence/S31.2.log`）；**S31.2b 残留已 verified**（select_map 包列表 UI（已装包 ∩ 内嵌槽位金边条标记，未注入 0 行为变化）+ 切包会话重建路径（`MapBack`→选图→重进，`enter_main_map_at(None)` 全量重建 board/territory）+ `SelectionHost::map_pack_ids`/`active_map_pack` 窄接缝 + 3 项 e2e，`evidence/S31.2b.log`；wasm fetch 运行期读取 = 后续，部署链已通）。

## 路由表

> 全库索引见 `docs/README.md`。本表只列「开工前必读」的入口。

| 你要做的事 | 读 |
|---|---|
| **理解游戏怎么设计/架构/模块化（第一入口）** | `docs/00_design.md`（正向设计总览：核心循环 + 数据分层 + crate 架构 + 9 场景 + 模块包 + 窄接口；**不读 legacy/**） |
| **新开工 / 参与开发（先读这个）** | `docs/process/00_index.md`（分工表：谁做什么 + 标准工作流）+ `docs/business/00_index.md`（业务唯一事实源） |
| **任何开工动作之前** | `docs/loop.md` + `plan/ledger.yaml` + `docs/README.md` |
| 判断「做完了没有」/ 验收 | `docs/parity.md`（P0~P4 判据 + **门禁项数/口径唯一事实源**） |
| 写 Rust（分层/数据驱动/现成库） | `docs/process/01_architecture.md` + `docs/process/02_conventions.md`（新）；语法层细节 `docs/style.md`（**标准库 trait 选用矩阵 + 重构操作规程 = §5.6.6**：From/Into/TryFrom/Display/Default/AsRef/include_str! 等何时用 + 手写转换补 trait 的五步规程） |
| 套用模版 / 写新场景·控件·模块 | `docs/process/03_templates.md`（§7 = 模块包任务书模版） |
| **代码标准执行矩阵（架构/耦合/内聚/命名/重构；写码·提交前对照，新代码零豁免）** | `docs/process/16_code_standards.md`（各维度 → 机器检查映射：CS-1 资产单源 / CS-2 文件 `//!` 职责声明 / CS-3 参数禁单字母 / CS-4 场景层窄接缝 = `scripts/check_code_standards.py` 12 项 + 规模归 `audit_code.py` + lint 红线归 gate；棘轮基线 `process_ratchet.json::code_standards`） |
| **布局引擎 / 响应式版面 / 改布局·加场景·调视口·换引擎（S29 定案 = B1 全 bevy_ui 响应式）** | `docs/process/17_layout_engine.md`（B1：UI + 命中 + 重排全进 Bevy = `bevy_ui` 自动布局 + 窗口 resize + `bevy_picking` 2D 拾取；3 条约定松绑签核（逻辑层仅 UI 交互层进 Bevy / `bevy-cpu-parity` 改口径只保游戏世界 / UI chrome 转 Bevy 代码而游戏内容仍 1080 数据驱动）+ 换引擎成本重估；原「引擎无关求解器」方向已作废；WBS S29.0~S29.5；S27.4 环形棋盘 / S28.6 副本场景 UI = 其消费者） |
| **加地图 / 换地图 / 出一张地图包（一张图 = 一个可分发的自包含包）** | `docs/process/21_map_package_spec.md`（**地图包规范 v1** = 包格式 + `manifest.json` 字段 + 版本兼容 + sha256 完整性 + 31 格不变量 I1~I8 + **运行期加载契约**（Bevy 资产根直读包 + `ti_core::data` 可注入源 + `activate_pack(id)` 切包）+ 失败回退语义）+ `scripts/map_pack.py`（`pack` 出一包 / `verify` 校验 / `install` 装上 / `uninstall` / `list` / `check` 漂移对账；台账见 `14_tooling_catalog` §3.2）+ 产物 = 已装包 `assets/maps/<id>/` + 可分发件 `dist/maps/<id>.map-pack/` |
| **资产登记 / 资源加载标准 / 版面坐标写在哪 / 加资产·加地图（不手写像素 + 统一资产管理）** | `docs/process/18_asset_content_pipeline.md`（行业三档 ①绝对坐标/②意图+自动布局/③参数化 + 总纲「不手写像素」；本项目终态 UI=bevy_ui 自动布局 S29.3 / 地图=参数化 S27.4 / scenes.json 退成元素清单；**资产登记标准** = UI 图 `ui_manifest.json` + 字体/音频 `assets/registry.json`，gate `validate_data` `check_asset_registry` 全类型 disk↔registry 双向同步 + `check_scenes_canonical` 守 scenes.json indent=1 规范形；音频口径 = 发货无声 S18.8；**加地图/换图 = `scripts/map_pack.py`（`assets/maps/<id>/` 描述符 + 模板，`sync` 一条命令，不改 Rust）**；改资产/资源加载/版面坐标/加地图前必读） |
| 用脚本/codegen 批量处理 | `docs/process/04_tooling.md` |
| **查业务规则（战斗/经济/地图/存档/UI 流）** | `docs/business/` 各系统文档（不依赖逆向产物） |
| **界面版面 / 配色 / 控件坐标（1080 单一事实源）** | `docs/business/08_layout_design.md`（改 scenes.json 坐标/配色前必读；表值 = scenes.json 同值） |
| **模块并行开发（契约→并行→集成）** | `docs/process/05_module_parallel.md`（一轮 4 步 + 纪律 5 条）+ `docs/contracts/00_index.md` + `plan/packages.yaml`（**包划分的机读唯一源**） |
| **对抗流水线飞轮（找问题→审计→修复→回写；开轮编排 = `flywheel-kickoff` skill）** | `docs/process/06_flywheel.md`（3 角色 + 开轮必读 6 条 + 停机条件）+ `scripts/flywheel_readme.ps1 -Check` |
| **界面全量重设计 S17（设计令牌 v2 / 8pt 版面 / 图标 14 项 / 动效 4 参数，航海夜·精修）** | `docs/process/10_ui_redesign.md`（S17 规格单一事实源；改 UI 视觉量前必读，与 `08_layout_design.md` 场景坐标实例互锁） |
| **UI 重设计 v3（真游戏化版面：场景叙事 DSL + 打破 7 场景同模板）** | `docs/ui-redesign-v3/README.md`（01 对标拆解 / 02 场景叙事 DSL 规格 / 03 七场景重设计提案 / 04 index 落地；gate 的 `check_scene_narrative` 项（gate.ps1 内注释编号 21）锚点 `assets/data/ui_schema/scene_manifest_v3.json`；v3 取代 v2 的「7 场景同模板」前提，色板/字体/动效令牌仍沿用 `10_ui_redesign.md`） |
| **UI 重设计 v3.5（画面丑的根因=设计数据：地形/暖色板/画布/实体/HUD）** | `docs/ui-redesign-v3/07_v3_5_pass.md`（地形数据 → 棋盘成地图；暖色板；画布贴合；实体图标化；执行链 S18.9~S18.13） |
| **main_map 重设计 v4「羊皮海图桌面」（S21.3 定案坐标/交互/资产接线；改 main_map 版面/节点牌/船/舵轮 CTA 前必读）** | `docs/ui-redesign-v3/08_main_map_v4_spec.md`（逐元素坐标表 + N8/8pt 门禁耦合 + 资产接线 + 交互契约；单一事实源，镜像 `business/08_layout_design.md` §4.3；落码 = S21.4 岛屿重绘 / S21.5 赛道重绘 / S21.6 版面重排 / S21.7 交互 / S21.12 船贴图） |
| **main_map 重设计 v5「中央据点岛 + 环形格子环」（S27.1 定案坐标/交互/资产接线；改 main_map 版面/环形格子/中央据点岛/点卡交互前必读）** | `docs/ui-redesign-v3/10_main_map_v5_spec.md`（31 张岛型卡环 96×120 + 中央可点击据点岛 760×760 + 10 类海盗岛 + 归属色带 + 逐元素坐标表 + N8/8pt 门禁耦合 + 点卡交互契约（交钱/攻占/进副本/升级加固）；单一事实源，镜像 `business/08_layout_design.md` §4.4；业务规则 = `business/territory.md`；落码 = S27.2 资产 / S27.3 数据 / S27.4 表现 / S27.5 出图） |
| **领地系统（v5 业务单一事实源：每格 = 领地/功能点，占领/交钱/攻占/升级加固 + 10 类岛专属副本 + 4 AI 对手席位）** | `docs/business/territory.md`（席位与选角 ≥1 玩家 / 领地格 vs 功能格 / 31 格赛道 / 10 类岛与副本 / 落格三支分叉 / 经济接入 / 存档 v4 / 可测规则表）+ `assets/data/board.json` + `rules::territory`（`ti-core`）；改领地/副本/AI/存档前必读 |
| **首页 index 重设计「罗盘环标题页」（S22.0 定案坐标/交互/资产接线；改首页版面/罗盘环/徽章/舵轮前必读）** | `docs/ui-redesign-v3/09_index_compass_spec.md`（C 方向 + 羊皮海图底与 main_map v4 统一：满屏 `chart_full` + 中心舵轮 `helm` + 6 铜质 medallion 沿环（白蒙版图标 gold tint）+ 版本签；逐元素坐标表 + N8/8pt 径向键豁免 + 资产接线 + 6 徽章命中契约；单一事实源；落码 = S22.1 出 `medallion_ring` 资产 / S22.2 落码 / S22.3 出图验收） |
| **环岛赛道 / 摇骰前进（S19，主地图主玩法）** | `docs/business/progression.md` §环岛赛道（单一事实源）+ `assets/data/board.json`（赛道数据，`scripts/extract_board.py` 生成）+ `rules::board`（`ti-core`）；改主地图移动/落格结算/轨道数据前必读 |
| **改渲染代码（sprite_atlas/terrain_atlas/game_app render/越界/透明/z-order）前必读** | `docs/process/08_render_arch.md`（**运行时架构契约**：分层 / 绘制原语 / z-order=调用顺序 / 单一 1080 坐标系 / grep 门禁；gate 第 9 项 doccheck2 `check_runtime_no_legacy_coords` 锚点）<br>`docs/process/07_render_pipeline.md`（legacy 像素管线契约，已降级为逆向参照） |
| **改 `crates/ti-render/`（渲染框架 crate）代码前必读** | `docs/process/09_render_framework.md`（**ti-render crate 边界契约**：职责与禁止项 / 模块结构 / Theme schema / 迁移映射 / 数据分离原则；注意本 crate **无 z-order 枚举**，层序在 `chrome::scene_shell` 调用序） |
| **S18 渲染引擎迁移（Bevy 0.20）· 改引擎层/加 bevy 依赖前必读** | `docs/process/11_bevy_migration.md`（**迁移裁决单一事实源**：差距定稿 / 定案推翻签核（macroquad 定案 + 旧 800KB wasm 预算作废，新预算 **≤8MB**）/ 架构形状（新 crate `ti-engine` + Canvas 后端替换 + 双渲染期）/ WBS S18.0–S18.8 / 风险） |
| 可玩实现手册（已验证 rules 接进交互壳的接线规格） | `docs/gamebook.md` |
| 当前可玩范围 / 平台产物 / 已知限制 / 如何运行 | `docs/release.md`（纯 UTF-8，编辑 CJK 内容时勿用 `Set-Content`，见 EXP-051/091） |
| 防虚报：台账 verified ↔ evidence ↔ 代码符号三方对账 | `docs/check_ledger_reality.md`（EXP-058；gate 第 14 项） |
| S10 现代化收口总计划（P0 修复/占位清零/GPU 渲染） | `docs/plan-s10-refactor.md`（历史阶段文档） |
| S11 正向开发收口（逆向归档 legacy/ + 业务闭环） | `docs/plan-s11-forward.md`（历史阶段文档，§7 交接快照） |
| S12 模块化并行 WBS | `docs/plan-s12-parallel.md` |
| S16 试玩闭环 + 发布 + 设计 sheet（WBS，S15.2 后开工） | `docs/plan-s16-loop.md`（S16.1 浏览器真运行时试玩 web_playtest / S16.2 试玩 agent 闭环 loop / S16.3 一键发布 publish-web / S16.4 内容设计 sheet 模版；对标 OhMyGame 只吸收概念） |
| 逆向 exe 或 `data/` 格式（**仅参照物**，不作为验收基准） | `legacy/docs/facts/index.md` |
| 历史阶段计划（已完成阶段，查背景用） | `docs/plan-archive.md`（S0~S8 逐阶段 1~2 行摘要 + 原文件位置） |
| **游戏开发入门教程** | `docs/tutorial/README.md`（10 章 + 进度追踪，紧密绑定项目代码） |
| 避免重复踩坑 | `docs/experience/index.md` ← **每次踩坑后必须回写** |
| 容器格式（pak/lpk）字段与偏移 | `docs/formats/lucencypackage.md` |
| 现代化改造 / 移植 / 多端适配 | `docs/modernization.md` |
| **HD 资产增强 / 超分 / 矢量化 / 多分辨率打包（S7）** | `docs/impl/14_asset_enhancement.md`（方案规格，历史阶段） |
| **写 Bevy 代码前必读（Bevy 0.20 用法规范 / 最佳实践 / 坑位清单）** | `docs/process/12_bevy_usage.md`（**Bevy 使用规范单一事实源·总览/通用**：官方 `DefaultPlugins` 强制 + 1080 坐标换算 + **z 层负值/升序绘制**规则 + GPU Text2d 文本 + 离屏截图验收口径 + `links=alsa` 隔离 + 已知坑位清单；S18.7 起 Bevy = 唯一渲染引擎；用量大的主题已拆独立篇：见下两行） |
| **改 UI chrome / 加控件 / 改版面坐标 / bevy_ui 用法（自动布局·resize 重排·UI 图片）** | `docs/process/19_bevy_ui.md`（**bevy_ui 专题单一事实源**：UI 相机分层 `IsDefaultUiCamera` order=1/clear None + `Node` 字段全 inline + `Text`/`TextFont` + **UI 图片 = `ImageNode`**（0.20 新 API，非旧 `Image`/`Sprite`；`NodeImageMode::Stretch` 铺满节点）+ **taffy 自动布局 = 意图化**（flex column+gap+`%` 定位 ⇒ resize 真重排，不手写像素）+ `ComputedNode` 探针证据口径 + 0.20 坑位清单） |
| **改命中 / 加可点元素 / bevy_picking 用法（指针事件·事件契约）** | `docs/process/20_bevy_picking.md`（**bevy_picking 专题单一事实源**：0.20 拾取结构（无 `PickingSection`/`PickingHandler`；`Pickable`+`MeshPickingSettings`+`MeshPickingCamera`）+ 指针事件（`PointerClick`=tap、`PointerDragEnd.distance`=swipe）+ **0.20 `Message` 取代 `Event`**（`add_message`+`MessageWriter::write`+`MessageReader::read`；`add_event`/`EventWriter` 已删）+ 本项目命中契约（`PickTarget`/`PickEvent`/`HitData`/`register_pick`/`swipe_dir_from`）+ 数据化命中（`scenes.json` `pick_key`/`pick_index`）+ 观察者参数最小化 + 坑位清单） |
| **做界面（改 scenes.json 版面 / 加美术资产 / 出图验收）** | `docs/process/13_ui_asset_pipeline.md`（**UI 资产管线**：资产清单 `assets/ui_manifest.json` = 单一事实源；`python scripts/ui_assets.py add/build/verify` 生图+抠透明+入库；`powershell scripts/ui_preview.ps1 <scene>` 出图验收；界面数据只引用资产路径，`image` 原语见 `09_render_framework.md` §5.1） |
| **工具台账 / 流水线组合（scripts/ 全量登记 + 用法）** | `docs/process/14_tooling_catalog.md`（工具唯一事实源：用途 / 用法 / 幂等与退出码 / UI 资产·赛道数据·门禁·逆向 4 条流水线 / 门禁项↔排障工具对照）；新增工具须登记（doccheck2 第 26 项对账）+ 满足工具契约（第 27 项 `check_tooling_contract`：落地位置 / 表格行 / 登记要素 / 只读模式 + 非 0 退出；一次性脚本落 `scripts/tmp/` 用完即删，**禁止写仓库外**） |
| **流程固化台账（哪些纪律已被机器兜住；收口 / EXP 登记走一条命令）** | `docs/process/15_process_ledger.md`（人反复要做的动作 = 纪律执行状态面：已自动化 / 半自动 / 待固化（必带台账任务号）/ 一次性；载体 = `scripts/task_close.py`（收口一条命令：跑判据→写证据→置 verified→同步计数→三方对账）/ `scripts/exp_log.py`（EXP 3 处改齐）；doccheck2 第 31~34 项守住：BOM/UTF-8/乱码 / 改动面声明 / 工具 3 处登记 / 台账自洽） |
| **批量做一组 UI 资产（多形状图标 / 一批背景）** | `scripts/ui_asset_batch.py`（register/generate/install/build/check/sheet，规格驱动 `assets/ui_batches/*.json`；含**抠图可行性预检**：luma 需纯黑底 / magenta 需品红底）+ `docs/process/13_ui_asset_pipeline.md` |
| **AI 生图（UI 效果图/概念图，S18.7 前置）** | `scripts/agnes_gen.py`（Agnes Image 2.5 Flash；`AGNES_API_KEY` 环境变量；用法见 `docs/process/04_tooling.md` §6） |
| **AI 视频（教程动效 / 概念演示）** | `scripts/agnes_video.py`（Agnes Video 2.5 Flash；`AGNES_API_KEY`；用法见 `docs/process/04_tooling.md` §6） |
| Android 壳落地 + Mac 出 APK（S5.5） | `android/README.md` |
| 引入新依赖 / 改 workspace 结构 | `docs/process/01_architecture.md` §4（现成库清单，先查再引）→ `docs/style.md` §5.9 |
| **报「特定区域贴图错位/覆盖菜单」先逐元素定位** | `cargo run -p ti-app --features native-shell --bin element-dump -- <scene>`（§7 元素级验证法：逐精灵落点/越界判定/像素贡献，定位到具体记录号 + f1 精灵） |
| 推送远端 / 分支与 commit 规范 / **完成即自动推送**（gate 全绿 + verified → commit → 自动 push，无需用户逐次指令） | `docs/git.md` |

## 硬规则（违反 = 返工）

1. **不许留占位**：`TODO/FIXME/unimplemented!/todo!/panic!` 及库代码中的 `unwrap/expect` 一律为 0 命中（`tests/` 除外），门禁在 `scripts/gate.ps1`。
2. **验收才算完成**：任务的验收命令**实跑输出**写入 `evidence/<task>.log` 后，才在 `plan/ledger.yaml` 置 `verified`。⛔ **只贴 gate 总输出不算验收证据** —— `doccheck` 会检查 evidence 是否包含任务自己的判据（第 9 项）。
3. **进度只由台账驱动**：循环 = 取下一个非 `verified` 任务 → 实现 → 验收 → `scripts/gate.ps1` → 更新台账 → 终止条件见 `docs/loop.md`（推进至 S4.4）。
4. **结论必须实测**：二进制/格式相关结论写进 `legacy/docs/facts/<分卷>.md` 前必须有脚本证据；推测一律标注 `推测`。
5. **选型已定案**：Rust edition 2024 + **Bevy 0.20**（渲染引擎；S18.0 推翻旧「macroquad」定案，见 `docs/process/11_bevy_migration.md` §1 + `docs/style.md`）。macroquad 只剩 android 壳在用（S18.8d 退役）。不要重新讨论语言/框架。
6. **文档拆分 + 单一事实源**：细节进 `docs/`，本文件只做路由与硬规则。**同一结论只写一处**，其他文档引用 ID（`F-xxx`/`EXP-xxx`/`P0-x`）而非复述。新增文档必须登记进 `docs/README.md` 与本路由表。ID 唯一性与跨文档引用有效性由 `doccheck` 门禁（第 9 项）自动校验。
7. **改动面必声明**：每次开发开工前声明「改哪一块」（业务规则 / 数据 / 规则实现 / 表现 / 资产，见 `docs/process/00_index.md` §变更面声明），**跨模块必须写原因**（为何必须同改、为何不能拆成可独立验收子任务）；改动面含「业务规则」时**先在 `docs/business/` 定案再动代码**。不声明 = 返工（同 §分工表「LLM 禁区」）。
8. **按模块功能开发 + 批量编辑一次编译**：动手前先定**模块边界与函数清单**（一个文件 = 一个职责，见 `docs/process/02_conventions.md` §9）；同一任务的编辑**全部落完再编译一次**，编译报错**一次性收口**——禁「改一处编一次」的探针节奏（编译成本纪律）。
9. **UI 视觉改动：设计稿先行（A 纪律）**：表现层视觉/版面/配色/控件改动，**先出设计稿/效果图**（`scripts/ui_preview.ps1` 离屏出图或临时原型），**用户确认（消息留痕）后才落码**；任务 dod 必含「用户视觉验收通过」，确认后再改视觉意图须重新过目（`docs/process/00_index.md` §UI 视觉改动：设计稿先行 + EXP-114）；未确认就开工 = 返工。
10. **脚本纪律（工具缺位才是病）**：动手写脚本前**先查 `docs/process/14_tooling_catalog.md`**（工具唯一事实源）；① 现有工具能跑 → 用；② 现有工具能组合 → 组合；③ 缺能力 → **补 `scripts/` 正式工具** + 按台账 §1.2 登记（幂等 + 只读模式 + 非 0 退出 + 台账 + 速查表），**不许**写一次性脚本包一层；④ 真的一次性改写 → 落 `scripts/tmp/` 且**用完即删**。**禁止写到仓库外**（系统临时目录 / 桌面），仓库内不得散落 `.py`/`.ps1`。违反 = `doccheck2` 第 27 项 `check_tooling_contract` FAIL（落地位置 / 台账表格行 / 登记要素 / 只读模式 + 非 0 退出）。

## 命令

```sh
# 常用命令一键入口（16 项命令固化，-List 列全部；含 push 前置 gate 全绿检查）
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 -List
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 run-native     # 桌面版（HD 真彩，F4 切画质）
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 gate         # 全门禁
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 test         # 全 workspace 测试
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 element-dump index 3  # 元素级验证（场景 + 记录号）

# 每轮循环的门禁：gate.ps1 28 项固定 Report（分档 -Fast/-Touch/full）（fmt/clippy/test/build/占位/unwrap/zero-unknown/台账/doccheck/validate_data/gen_layout/check_hardcode/oracle边界/doccheck2/台账对账/业务闭环/模块并行/飞轮/视觉回归/渲染快照/正向渲染快照/数据源/场景叙事约束check_scene_narrative；wasm/android 构建为条件子项不计顶层；zero-unknown 亦条件项（verify.rs 在位才计，QA145））
# 全绿才可把任务置 verified，输出 tee 到 evidence/<id>.log
powershell -ExecutionPolicy Bypass -File scripts/gate.ps1

# 单项调试
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace
cargo run -q -p ti-tools --bin ledger  -- plan/ledger.yaml   # 台账校验（gate 第 8 项）
cargo run -q -p ti-tools --bin doccheck                       # 文档一致性（gate 第 9 项，含跨文档数值断言 + release 台账计数漂移）
python scripts/check_ledger_reality.py                         # 台账三方对账（gate 第 14 项，verified↔evidence↔代码符号，EXP-058）

# S2.5：构建浏览器产物到 web/（S18.8a 起 = Bevy 交付壳 `treasure-bevy-wasm` + 真实资源；
# 浏览器页 wasm-bindgen 胶水 + HTML 属 S18.8e）
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
# 本地预览（wasm 需 HTTP，不能 file://）
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080

# S18.8a：桌面发货版 = Bevy 壳（ti-engine 独立 workspace；macroquad 壳已退役）
cargo run -q --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native

# S2.7：native + wasm 两端冒烟（本机无 bash，务必用 ps1）
powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1

# S1.4b：控件记录全量导出（20 UI 场景 → ref/controls.json，44 种 f5 控件类型）
cargo run -q -p ti-tools --bin controls -- --out ref

# spec 骨架半自动生成（模板化逆向→开发：函数清单表 + 逐函数带行号证据行表，人工只补「语义」列）
cargo run -q -p ti-tools --bin specskel -- <module>              # 打印到 stdout
cargo run -q -p ti-tools --bin specskel -- <module> --out PATH   # 覆盖已有 spec 文件
cargo run -q -p ti-tools --bin specskel -- <module> --report     # 只打印统计摘要
# <module> = ref/functions_<module>.csv 的 <module>（main_loop/render/menu/selectrole/selectmap/
# mainmap/trade/build/battle/cards/saveload/messages/rng）；自动识别两种列格式
# （常规 addr|address,size,name 与 RNG 专用 kind,site,func）

# 反编译导出（S0.2）；IDA 9.x 把 7.5 的 idat64.exe 合并为 idat.exe（32/64 同一程序）
"D:\soft\IDA Professional 9.4\idat.exe" -A -Lref\ida_run.log -Slegacy\scripts\ida_dump.py TreasureIsl.exe
```

- 构建范围随阶段解锁：native（已解锁）→ wasm（**S2.5 已解锁**）→ android（S2.6 在 **Mac** 装 NDK/SDK + cargo-apk 编译 APK，本机 Windows 不产出，gate SKIP）；gate 按已装 target 自动判定并打印 SKIP。
- **发货壳 = Bevy（S18.8a 起）**：桌面 `treasure-bevy-native` / 浏览器 `treasure-bevy-wasm`（`crates/ti-engine`，独立 workspace）；ti-app 的 macroquad native/wasm 壳已删除（`macroquad` 仅剩 android 壳在用，S18.8d 退役）。门禁 wasm 项直接构建 + lint 该 Bevy 交付产物。
- **桌面/浏览器可玩（S5.1–S5.4）已收口**：菜单→选人→选图→主地图→贸易/建造→战斗→存读档全链路可玩，P3-3 全流程 + P4-1 渲染对齐已机器判定；S0–S14 全 verified，当前活动任务 S21（main_map 重设计 v4 + 船系统：S21.0/S21.1 verified、S21.2 船系统规格待拍板，见 `plan/ledger.yaml`）；Android 为后续移动端交付，开发本身不依赖 Android 工具链。
- S5.5 Android 壳已落地（`android-shell` feature + `treasure-android` bin + `android/README.md`）；APK 实际产出在 Mac，本机 host 形态可编译校验。

## REA 逆向 MCP（2026-10 接入）

- 注册：OpenCode `mcp.rea`（`~/.config/opencode/opencode.json`），`environment` 带 `REA_ANALYSIS_PROVIDER=ida` + `REA_IDA_MCP_CONFIG=C:\Users\Administrator\.rea\ida-mcp.json`；headless supervisor = `D:\ProgramData\miniconda3\Scripts\idalib-mcp.exe`（IDA 9.4 + idapro 0.0.9 + ida-pro-mcp 2.0.0）。
- **三层定位**（F-319 波动性可用 + F-321 结构化能力 + EXP-041 分层修正）：
  - ① **批量/离线主路径**（不可替代）：`ref/pseudocode/*.c` grep（S0.2 已全量导出 1827 份）+ `scripts/pe_rva.py` + `idat.exe -A -S`；
  - ② **REA 结构化能力**（补洞，未利用）：CFG/调用图/类型恢复/值追踪/重建验证 —— 冒烟优先，失败即退主路径，见 F-321 + `docs/style.md` §5.11.4；
  - ③ **稳定性**：F-319 波动性可用（S3.2 实测可用 EXP-039 / S3.12 实测失败 EXP-036），逐工具实测表见 `legacy/docs/facts/tools.md`。
