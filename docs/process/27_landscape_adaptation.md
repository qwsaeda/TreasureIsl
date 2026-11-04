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
- 世界层数据：`scenes.json` 每场景加 `layout: {portrait: {...}, landscape: {...}}`（现有竖屏值 = portrait，
  **横屏值 = 竖屏坐标顺时针旋转 90° + 手工重排**——不是简单镜像，需按横屏视觉重心重排）。
- UI chrome 层：bevy_ui 已有 taffy 自动布局，横屏时**重排方向**（flex 方向/百分比随方向）。
- 游戏世界 `portrait_zones`/`focus_anchor`/`bbox` 每场景**横竖双份**（`scene_manifest_v3.json` 加 landscape 段）。
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

### 4.3 数据层双份
- `scenes.json` 每场景 primitives 加 `orientation: "portrait"|"landscape"` 或按方向切换整组；
  现有值默认 portrait（0 行为变化）。
- `scene_manifest_v3.json` 每场景 `focus.bbox_pct`/`portrait_zones` 加 landscape 值。
- `ui_schema` 校验（`validate_data`）加「方向字段合法性」断言。

### 4.4 世界层渲染
- `scene_render.rs` 按 `is_landscape` 选逻辑画布（1080×1920 或 1920×1080），`viewport_scale` 方向自适应。
- 每场景横屏绘制 = 竖屏坐标**旋转 90° 基准 + 手工重排**（各场景按横屏视觉重心定稿）。

### 4.5 UI chrome 层
- bevy_ui 节点已 `%` 定位 + flex；横屏时按 `orientation` 切换布局参数（如列表 flex 方向、面板位置）。

### 4.6 命中
- `pick.rs::hit_1080_pick` 窗口逻辑 px → **当前方向**逻辑坐标换算（letterbox 按方向 scale + 偏移）。

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
- [ ] 其余场景横屏概念图（vs）出稿 → 用户过目 → 动码（S38.5）。

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
