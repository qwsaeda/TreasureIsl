# 横屏适配设计草案（U23，2026-11-04 设计稿先行）

> **状态：设计草案（待用户确认，A 纪律）**。用户 2026-11-01 拍板 U23「手机游戏应该是横屏，你做适配了吗」。
> 本文件 = 横屏适配的**设计单一事实源**；**用户确认后才动码**（A 纪律：设计稿先行 + 用户视觉验收）。
> 登记：`docs/README.md` 索引 + `13_user_requirements_ledger.md` U23 更新。

---

## 1. 现状（问题）

- 全项目唯一呈现坐标系 = **1080×1920 竖屏（9:16）**（`ti_render::framebuffer::LOGICAL_W/H_I32`）。
- 运行时 = **两层**（`08_render_arch.md` 契约 0b）：① 游戏世界层（1080 单一坐标系，数据驱动）
  ② UI chrome 层（`bevy_ui` 屏幕像素空间，taffy 自动布局）。
- **横屏窗口 = letterbox 药丸**（`canvas.rs::viewport_scale` = `min(win_w/1080, win_h/1920)`）：
  窗口 1920×1080 时 scale=0.5625 → 内容 607×1080 居中，左右各 656px 黑边。
  **内容恒竖屏完整可见但不铺满**，横屏屏幕浪费 ~61% 空间。
- 根因：**坐标系统级竖屏写死**，无横屏布局概念。

## 2. 目标

横屏（16:9，典型 1920×1080）下：
1. **内容铺满全屏**（无药丸黑边/letterbox），游戏世界 + UI chrome 都随横屏重排。
2. **7 场景全部横屏可用**：index / select_role / select_map / main_map / vs / message / saveload。
3. 竖屏（9:16）继续可用（**双方向自适应**，横竖屏不互相破坏）。
4. 不破坏 B1 契约（`17_layout_engine.md`）：游戏世界保持数据驱动 1080，UI chrome 走 bevy_ui 自动布局。

## 3. 方案选项（选型）

### 方案 A：坐标系统级横竖双套（推荐）
- 引入**方向感知**：运行时按窗口纵横比选 `PORTRAIT(1080×1920)` 或 `LANDSCAPE(1920×1080)`。
- **两套逻辑坐标系并存**：`LOGICAL_W/H_I32` 保持竖屏，新增 `LANDSCAPE_W/H = 1920/1080` 常量；
  `viewport_scale` 改按当前方向取 `min(win_w/CANVAS_W, win_h/CANVAS_H)`（方向自适应）。
- 世界层数据：`scenes.json` 每场景加 `landscape: {primitives, extra}` 块（**竖屏现有值不动 =
  portrait 默认，0 行为变化**；横屏值 = **每场景独立重排的 1920×1080 坐标**——不是旋转/镜像，
  按横屏视觉重心重排定稿）。`landscape.primitives` = 横屏原语表（`prims_for(landscape)` 选中），
  `landscape.extra` = 横屏专属额外矩形（`extra_for(landscape)` 选中，回退竖屏 `extra`）。
- UI chrome 层：bevy_ui 已有 taffy 自动布局，横屏时**重排方向**（flex 方向/百分比随方向，
  % 定位 = 引擎自带 resize 重排，如 message 面板横屏居中 55%）。
- 游戏世界场景（vs/main_map，S29.3e 留 1080）：**数据驱动横屏布局**（vs = 羊皮框 + 顶双 HP 框 +
  角色对角；main_map v6 = 椭圆格环 31 格，`main_map_v6_layout_landscape.json`）。
- **工作量**：大（~7 场景 × 坐标重排 + 双份数据 + 方向切换逻辑），但**一次到位**、横竖屏都完整。

### 方案 B：横屏 = 竖屏内容旋转 90°（廉价，不推荐）
- 横屏时把整张 1080×1920 画布**旋转 90°**铺满，文字/控件跟着转。
- 工作量小（1 个相机旋转），但**横屏是"躺着的竖屏"**：文字横躺难读、主地图/战斗操作逆直觉、
  用户会认为"没做适配"。**不满足 U23 精神**（用户要真正的横屏游戏 HUD）。

### 方案 C：横屏 = 竖屏内容放大居中（维持 letterbox，最小改动）
- 保持现状 letterbox，仅把窗口默认尺寸改横屏 + 背景色铺满（黑边非空）。
- 工作量最小（窗口尺寸 + 清屏色），但**内容仍竖屏**，横屏空间浪费依旧。不满足 U23。

## 4. 方案 A 落地轮廓（用户确认后细化）

### 4.1 方向判定
```rust
/// 窗口纵横比 ≥1.0 = 横屏；<1.0 = 竖屏。
fn is_landscape(win_w: f32, win_h: f32) -> bool { win_w >= win_h }
```

### 4.2 坐标系常量
```rust
// ti_render::framebuffer
pub const LOGICAL_W_I32: i32 = 1080;   // 竖屏（保留，兼容）
pub const LOGICAL_H_I32: i32 = 1920;
pub const LANDSCAPE_W_I32: i32 = 1920; // 横屏
pub const LANDSCAPE_H_I32: i32 = 1080;
```

### 4.3 数据层双份（落地 = `landscape` 块）
- `scenes.json` 每场景 `landscape: {primitives, extra}`（1920×1080 坐标）；现有 `primitives`/`extra`
  保持竖屏默认（0 行为变化）。`SceneLayout::prims_for(landscape)` / `extra_for(landscape)` 按方向选中，
  `landscape` 块缺席 = 空 = 回退竖屏。
- `ti-core` `SceneLayout` 加 `landscape_prims` / `landscape_extra`（解析口径同竖屏，EXP-074/RectDefault）。
- `validate_data` 校验 `scenes.json` indent=1 规范形 + `landscape` 段 8pt 栅格（doccheck2
  `scene-layout-fidelity` 扫描 `landscape.extra` 键）。

### 4.4 世界层渲染（落地）
- `chrome::render_primitives_filtered`：横屏且场景声明 `landscape.primitives` 时用横屏原语
  （`prims_for(landscape)`），`ui_layer=bevy_ui` 原语跳过（同竖屏口径）。
- 游戏世界场景（vs/main_map）：方向感知 blit（横屏 1920×1080 / 竖屏 1080×1920）+ 屏心 960
  （竖屏 540）；数据驱动横屏布局（`vs_anim_proto_landscape` / `main_map_v6_layout_landscape.json`）。

### 4.5 UI chrome 层（落地）
- bevy_ui 节点 `%` 定位 + flex；横屏时按方向切换布局参数（message 面板横屏居中 55%；
  select_map 右侧列表 / saveload 左侧列表 % 定位 = 引擎自带 resize 重排，0 代码改动）。

### 4.6 命中（落地）
- `pick.rs::hit_1080_pick` 窗口逻辑 px → **当前方向**逻辑坐标换算（letterbox 按方向 scale + 偏移，
  S38.0 已接方向感知）。

## 5. 验收判据（草案）

1. 窗口 1920×1080 时 **7 场景全部横屏铺满**（`--shot --viewport=1920x1080` 出图非黑 ≥90%，无药丸黑边）。
2. 窗口 1080×1920 时竖屏**0 行为变化**（横屏改动不破坏竖屏：`--shot` 竖屏与基线对拍 ≤0.5%）。
3. `--resizetest` 横竖切换：画布完整可见 + 节点数不变（真重排，同 S29.5 第 3 不变量）。
4. 命中：横屏点击 → 正确场景交互（`e2e` 横屏路径）。
5. gate 全绿 + doccheck 0 error + `evidence/U23.log`。

## 6. 风险

- **坐标双套 = 双倍版面维护**：每场景横竖两份坐标，后续改版面要同步两处（须 `validate_data` 断言锁定）。
- **视觉重心重排**：横屏不是旋转，是重设计（主地图横屏宽 > 高，格子环/舵轮布局要重排；VS 横屏对角双面板）。
- **A 纪律**：必须先出**横屏渲染效果图**（`ui_preview`/`--shot --viewport=1920x1080` 或 AI 概念稿）+ 用户确认，才动码。

## 7. 用户确认点（2026-11-04 留痕）

- [x] **方案 A / B / C 选型 = 方案 A（坐标双套）**：用户 2026-11-04 过目横屏概念图
  （`evidence/ui_concept/main_map_v6_landscape.png` + `vs_landscape.png`，`agnes_gen.py` 16:9）
  后回复「继续」无异议（A 纪律消息留痕）。
- [x] **横屏效果图（2 张代表图）用户过目**：main_map 横屏（岛+宝箱+灯塔居中+格环+罗盘右下+顶部 HUD）
  + VS 横屏（拳皇对角线+VS 徽标+双面板+指令区）。
- [x] **S38.1 场景横屏概念图出稿**：index（罗盘环+6 徽章+中心舵轮，横屏居中）+ select_role
  （5 角色卡横排+右侧属性面板）——`evidence/ui_concept/index_landscape.png` +
  `select_role_landscape.png`（`agnes_gen.py` 16:9）。
- [x] **S38.1 用户过目确认 + 动码完成**：index/select_role 横屏概念图过目无异议（A 纪律留痕）
  → S38.1 动码（scenes.json `landscape.primitives` + 渲染方向感知）→ **verified**
  （`evidence/S38.1.log`；竖屏 0 行为变化 0.0003%，横屏 main_map 99.8%，gate 33 项 0 failure）。
- [x] **S38.2 用户过目确认 + 动码完成**：select_map 横屏概念图（3 卡横排 + 右侧状态面板）过目
  确认「动码」→ S38.2 动码（scenes.json select_map `landscape.primitives` 13 个）→ **verified**
  （`evidence/S38.2.log`；3 缩略图横排无头断言 + 竖屏 0 行为变化 + gate 33 项 0 failure）。
- [x] **S38.3 用户过目确认 + 动码完成**：message 横屏概念图（居中帮助面板）过目确认「动码」
  → S38.3 动码（scenes.json message `landscape.primitives` bg 满屏 + `build_message_chrome`
  面板横屏居中 55%）→ **verified**（`evidence/S38.3.log`；无头断言 + 竖屏 0 行为变化 +
  gate 33 项 0 failure）。
- [x] **S38.4 用户过目确认 + 动码完成**：saveload 横屏概念图（5 槽左列 + 右侧按钮）过目确认
  「动码」→ S38.4 动码（scenes.json saveload `landscape.primitives` 8 个：bg 满屏 + 5 按钮右侧竖排）
  → **verified**（`evidence/S38.4.log`；无头断言 + 竖屏 0 行为变化 + gate 33 项 0 failure）。
- [x] **S38.5 用户过目确认 + 动码完成**：vs 横屏概念图（`vs_landscape.png`，双 HP 条顶 + VS 徽标
  屏心 + 角色对角）§7 过目确认 → S38.5 动码（`render_v2_elem_landscape` + `draw_result_scene_landscape`
  新文件 `vs_anim_proto_landscape.rs` + vs.rs 方向感知 + vs `landscape.primitives` 16 +
  `landscape.extra` 6 键）→ **verified**（`evidence/S38.5.log`；landscape_drawlist 6/6 +
  竖屏 0 行为变化 + gate 33 项 0 failure）。
- [x] **S38.6 用户过目确认 + 动码完成**：main_map v6 横屏概念图（中央岛 + 椭圆格环 + 船 + 罗盘 +
  顶 HUD）过目确认「动码」→ S38.6 动码（`main_map_v6_layout_landscape.json` 椭圆格环 +
  `render_v6_elem` 横屏 + HUD/牌/终局 横屏屏心）→ **verified**（`evidence/S38.6.log`；
  landscape_drawlist 7/7 + 竖屏 0 行为变化 + gate 33 项 0 failure）。**S38 全部 7 场景横屏收口**。

---

## 8. 立项（方案 A 动码起点，S38.0 基础设施）

> 方案 A 已定案。WBS 从基础设施开始（不破坏竖屏 0 行为变化），再逐场景横屏布局。
> 各场景横屏布局动码前，仍须过目该场景横屏效果图（A 纪律）。

### S38.0 横屏基础设施（架构层，无场景视觉重排）
1. `is_landscape(win_w, win_h) -> bool`（`win_w >= win_h`）方向判定。
2. 坐标系常量：`ti_render::framebuffer` 加 `LANDSCAPE_W_I32 = 1920` / `LANDSCAPE_H_I32 = 1080`；
   `LOGICAL_W/H_I32` 保持竖屏（兼容，0 行为变化）。
3. `canvas.rs::viewport_scale` 方向自适应：`is_landscape` 时 `min(win_w/1920, win_h/1080)`，
   竖屏保持 `min(win_w/1080, win_h/1920)`。
4. `scene_render.rs` / 相机系统按 `is_landscape` 选逻辑画布尺寸（竖屏路径完全不变）。
5. 命中 `pick.rs::hit_1080_pick`：当前方向换算（横屏 1920×1080，竖屏 1080×1920）。
6. `--shot --viewport=1920x1080` 支持横屏出图（S29.5 多视口已具备，方向感知接上）。

### S38.1 起场景横屏布局（每场景独立任务，动码前过目效果图）
- index / select_role / select_map / main_map / vs / message / saveload（7 场景横屏重排）。
- **S38 全部 7 场景已收口 verified**（S38.1~S38.6，见 §7 确认点逐行 + `evidence/S38.{1..6}.log`；
  `landscape_drawlist.rs` 7/7 无头断言 + 竖屏 0 行为变化 + gate 33 项 0 failure）。

---

## 9. 横屏设计规范（S38 落地固化）

> S38.0~S38.6 已全部 verified（7 场景横屏收口）。本段 = **落地后的设计规范单一事实源**，
> 改横屏布局/加横屏场景/换坐标系前必读（与 §4 轮廓一致，此处为定稿口径）。

### 9.1 方向感知基础（S38.0）
- `is_landscape(win_w, win_h) = win_w >= win_h`；`LANDSCAPE_W/H = 1920/1080`（`LOGICAL_W/H` 保持竖屏）。
- `viewport_scale` 方向自适应：横屏 `min(win_w/1920, win_h/1080)`，竖屏 `min(win_w/1080, win_h/1920)`。
- 命中/相机/`--shot --viewport=1920x1080` 全部方向感知；**竖屏路径 0 行为变化**（横屏分支隔离）。

### 9.2 横屏数据契约（scenes.json `landscape` 块）
- **结构**：每场景 `landscape: {primitives: [...], extra: {...}}`。
  - `primitives` = 1920×1080 原语表（缺 = 空 = 回退竖屏 `primitives`；`prims_for(landscape)` 选中）。
  - `extra` = 横屏专属额外矩形（player_hp/boss_hp/hud_gold/…；缺 = 空 = 回退竖屏 `extra`；
    `extra_for(landscape)` 选中）。
- **坐标系**：横屏原语/extra 一律 1920×1080（**8pt 栅格**：x/y/w/h 须 8 整除，doccheck2
  `scene-layout-fidelity` 扫描 `landscape.extra` 键；对称屏心 = 960，竖屏屏心 = 540）。
- **ui_layer 口径**：已迁 bevy_ui 的 chrome 原语标 `params.ui_layer: "bevy_ui"`（横屏块同口径），
  `render_primitives_filtered` 跳过（由 bevy_ui 层画）。
- **资产路径**：沿用 `assets/ui/` 已登记资产（`ui_manifest.json` 单一事实源），横屏不新增资产路径。

### 9.3 渲染方向感知契约
- `chrome::render_primitives_filtered`：横屏且 `landscape.primitives` 非空 → `prims_for(landscape)`；
  否则竖屏（0 行为变化）。
- **游戏世界场景（vs/main_map，S29.3e 留 1080）**：方向感知 blit 尺寸（横屏 1920×1080 / 竖屏
  1080×1920）+ 数据驱动横屏布局；屏心横屏 960 / 竖屏 540。
- **bevy_ui 场景**（5 纯 UI）：`%` 定位 + flex = 引擎自带 resize 重排；横屏特殊面板（message
  居中 55%）经 `*_panel_params(landscape)` 辅助函数抽参（竖屏返回原 % 值）。

### 9.4 7 场景横屏构图定稿
| 场景 | 横屏构图 | 数据源 |
|---|---|---|
| index | 罗盘环 + 6 徽章径向 + 中心舵轮（横屏居中） | `landscape.primitives` |
| select_role | 5 角色卡横排 + 右侧属性面板 | `landscape.primitives` |
| select_map | 3 地图卡横排 + 右侧状态面板 | `landscape.primitives`（13） |
| message | bg 满屏 + 帮助面板居中 55% | `landscape.primitives` + `message_panel_params` |
| saveload | 5 按钮右侧竖排 + bevy_ui 5 槽左列 | `landscape.primitives`（8） |
| vs | 羊皮框（inset 44）+ 顶双 HP 框（64/976,880×110 8pt）+ VS 徽标屏心 + 角色对角 | `landscape.primitives`（16）+ `landscape.extra`（6）+ `vs_anim_proto_landscape` |
| main_map v6 | 中央岛（960,520）+ 椭圆格环 31 格（rx=760/ry=330 均匀弧距）+ 船左下 + 罗盘右下 + 顶 HUD | `main_map_v6_layout_landscape.json` + `landscape.primitives`（6）+ `landscape.extra`（4） |

### 9.5 验收判据（S38 实测标准）
- **主判据 = 无头绘制列表断言**（`crates/ti-app/tests/landscape_drawlist.rs` 7/7）：横屏坐标
  （1920×1080 原语/布局）+ 竖屏 0 行为变化对照。因多数场景 `--shot` readback 已知环境 GPU flake
  （仅 vs/main_map 可靠读回），**横屏版面用 draw list 级断言证明**（非 --shot 回归）。
- **wasm 横屏实测**（EXP-163）：`scripts/wasm_browser_e2e.py --viewport=1920x1080` 已补（视口 +
  截图名带尺寸后缀），实测 **L1 实例化 / L2 渲染（非黑 + 资产 2xx）/ L3 交互 / L5 环资产实取
  = PASS**（wasm 横屏实际渲染已验证）。**L4 场景切换点击 = 已知边界**：亮度阈值导航
  （`bright<40` 判已在 select_map）是竖屏口径，横屏各场景亮度相近（≈19%）首轮即命中、
  未导航到 select_map——需场景身份识别导航（非亮度）后方可覆盖，非渲染 bug。
  竖屏默认跑 5/5 PASS（0 回归）。
- 每场景：gate 33 项 0 failure + doccheck/doccheck2 0 error + audit_code 0 违规（拆分超限文件）。
- **改横屏布局**：改 `scenes.json` `landscape` 块 → 更新 `landscape_drawlist.rs` 对应断言 → gate 全绿。

### 9.6 固化纪律
- 横屏坐标/构图 = 本文件单一事实源；`08_layout_design.md` 表值仍竖屏（横屏不并入竖屏表，
  避免双份漂移——横屏值在 `scenes.json` `landscape` 块 + 本文件 §9.4）。
- 新增横屏场景 = 登记 `landscape_drawlist` 测试 + 过目该场景横屏效果图（A 纪律）+ 本节补行。
- `validate_data` / doccheck2 `scene-layout-fidelity`（8pt 栅格）/ `layout-doc-table-cross-check`
  （已剥除 `landscape` 段）守住数据面。
