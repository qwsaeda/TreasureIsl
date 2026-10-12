# 现代化改造方案（Modernization Plan）

原版：Windows Mobile ARM，176×220 单点触屏 + D-Pad。
目标：浏览器（WASM）+ Android + 桌面三端，现代触屏 + 键盘 + 手柄。

---

## 1. 渲染架构（S10.4 起：GPU 逐精灵 batch + TTF 字形）

> **2026-10 S10 修订**：渲染目标从「176×220 RGB565 CPU 逐像素 blit + 单 quad 上屏」升级为
> **GPU 化逐精灵 batch**（macroquad `draw_texture` + 纹理图集 + alpha 混合）+ **TTF 矢量字形**。
> 176×220 CPU blit 降级为 `legacy-render` feature 回退路径（P4-1 历史基准保留），不再是默认渲染范式。

```
┌──────────────────────────────────────────────┐
│  GPU 渲染管线（S10.4 默认）                     │
│  ┌──────────────────────────────────────────┐ │
│  │  纹理图集（PNG/PCX 解码 → macroquad Texture）│ │
│  │  → draw_texture 逐精灵 + alpha 混合        │ │
│  │  → TTF 矢量字形（替代 14 缺口字符 + 全字符集）│ │
│  └──────────────────────────────────────────┘ │
│  虚拟手柄 / 状态栏 / 菜单 / 对话框（原生分辨率）  │
└──────────────────────────────────────────────┘
┌──────────────────────────────────────────────┐
│  legacy 路径（feature legacy-render，回退）     │
│  176×220 RGB565 CPU 帧缓冲 → 脏行带 → 单 quad  │
│  （P4-1 历史基准，保留用于对比与回退）            │
└──────────────────────────────────────────────┘
```

- **精灵渲染**：`SpriteAtlas`/`TerrainAtlas` 解码结果直接传 macroquad `Texture`，逐精灵 `draw_texture` + 顶点 batch；HD PNG 透明通道走 alpha 混合（`BlendFactor::SrcAlpha/DstAlpha`），不再逐像素 memcpy。
- **TTF 字形**：`macroquad::load_ttf` 加载子集 TTF（思源黑体 / Noto Sans CJK），替代 `font.dat` 14 缺口字符（继续/胜负!/页 等豆腐块）；原版点阵字保留为可选「怀旧字体」模式。
- **HD 默认开**：`assets_hd_pack` 递归加载（S10.2 修复 `load_hd_pack_from_dir` 不递归 bug），缺文件回退 PCX 不 panic。
- **legacy 回退**：feature `legacy-render` 保留 176×220 CPU blit 路径，P4-1 历史基准图（`render_parity`）联动 S7.7 全重生成后，legacy 与 GPU 双路径并存。

---

## 2. 分端适配

### 2.1 三端目标

| 端 | 渲染后端 | 资源 IO | 输入 | 状态 |
|---|---|---|---|---|
| 桌面（Win/Mac/Linux） | macroquad native | 本地文件 `data/` | 键盘 + 鼠标 + 手柄 | 已通（S2.8） |
| 浏览器（WASM） | macroquad miniquad GL | HTTP fetch `web/` | 触屏 + 键盘 | 已通（S2.5） |
| Android | macroquad native（NDK） | APK assets / scoped storage | 触屏 + 手柄 | blocked（S2.6 需 NDK） |

### 2.2 浏览器增强：PWA

将 WASM 站点做成**可安装 PWA**：
- `manifest.json`：App 名、图标（176×220 截图缩放）、全屏模式
- `service_worker`：预缓存 `web/` 全部资源，离线可玩
- 启动画面：加载 wasm + 首屏资源时显示游戏标题图

### 2.3 Android 适配

```toml
# Cargo.toml (ti-app)
[features]
android = ["macroquad/android"]
```

- `cargo-apk build` 或 `cargo-ndk` → APK
- 资源读 APK assets（`miniquad` 提供 `android-asset` 读取）
- 存档写 scoped storage（`getExternalFilesDir`）
- 横屏锁定（`AndroidManifest.xml` 设 `screenOrientation="landscape"`）

---

## 3. 分辨率与缩放

### 3.1 整数缩放档

| 屏幕短边 | 缩放倍数 | 游戏画面像素 | UI 层 |
|---|---|---|---|
| ≥ 720px | 4x | 704×880 | 原生 |
| ≥ 528px | 3x | 528×660 | 原生 |
| ≥ 352px | 2x | 352×440 | 原生 |
| < 352px | 自适应填充 | ≤176×220 | 原生 |

自动选择最大可用整数倍，剩余空间 letterbox（黑边）或填充 UI 元素。

### 3.2 高 DPI 感知

```
dpi_scale = screen_dpi / 96.0    // Windows: 1.0/1.25/1.5/2.0
font_size = base_size × dpi_scale
ui_rect = physical_pixels / dpi_scale  // 逻辑像素
```

macroquad 已支持 `set_high_dpi(true)`，WASM 自带 `devicePixelRatio`。

### 3.3 可选怀旧滤镜

游戏画面 texture 上叠可选 CRT 滤镜（着色器）：
- 扫描线（每 2 行暗化）
- 圆角晕影
- RGB 子像素分离

通过设置菜单开关，默认关闭。

---

## 4. 字体策略

### 4.1 双字体

| 用途 | 字体 | 大小 | 说明 |
|---|---|---|---|
| 游戏内文本（据点名/状态行/面板） | `GoldLand.Fnt` 12×12 点阵 | 整数缩放后 24~48px | 保持原版像素感 |
| 菜单/帮助/设置/对话框 | TTF 矢量字体（思源黑体 / Noto Sans CJK） | 16~24px × DPI | 清晰可读 |

### 4.2 TTF 集成

```toml
# ti-app Cargo.toml
macroquad = { version = "0.4", features = ["font-use-ttf"] }
```

```rust
// 加载 TTF
let font = load_ttf_font("NotoSansCJK-Regular.ttf").await;
// 使用
draw_text_ex("新游戏", x, y, TextParams { font, font_size: 24 * dpi, color });
```

TTF 文件（~4MB 思源黑体子集）作为 wasm 非首屏资源分包加载。

---

## 5. 输入现代化

### 5.1 操作映射表

| 原操作（事件码） | 桌面 | 触屏 | 手柄 |
|---|---|---|---|
| 方向移动（0..3） | 方向键 / WASD | 左半区虚拟十字键 / 滑动手势 | 左摇杆 / 十字键 |
| 确定（13） | Enter / 空格 | 右半区「确认」按钮 / 双击 | A 键 |
| 返回（27） | ESC | 「返回」按钮 / 长按 0.5s | B 键 |
| 上一页（38） | ↑ / Q | 上滑 | 十字键上 |
| 下一页（40） | ↓ / E | 下滑 | 十字键下 |
| 翻页确认（39） | → | 右滑 | 十字键右 |
| 无效（37） | — | — | — |

### 5.2 虚拟手柄布局

```
横屏（手机/平板）：
┌─────────────────────┬──────────────────────┐
│                     │                      │
│   游戏画面           │    ┌──┐  ┌──┐        │
│   (整数缩放)         │    │↑│  │✓│         │
│                     │    └──┘  └──┘        │
│                     │  ┌──┐┌──┐┌──┐       │
│                     │  │←││↓││→│        │
│                     │  └──┘└──┘└──┘       │
│                     │         ┌──┐         │
│                     │         │✕│         │
│                     │         └──┘         │
└─────────────────────┴──────────────────────┘

竖屏（手机）：
┌───────────────────┐
│   游戏画面          │
│   (整数缩放)        │
│                   │
├───────────────────┤
│  ┌──┐  ┌──┐  ┌──┐│
│  │←│  │↓│  │→││
│  └──┘  └──┘  └──┘│
│      ┌──┐  ┌──┐ │
│      │✓│  │✕│ │
│      └──┘  └──┘ │
└───────────────────┘
```

### 5.3 手势

| 手势 | 语义 |
|---|---|
| 滑动（方向） | 主地图移动方向 |
| 双击 | 确定（场景内） |
| 长按 0.5s | 返回 |
| 双指缩放 | 地图缩放（未来增强） |

### 5.4 多帧动画（S6.4）

`filenum INI` 的 `num` 字段是帧数。实测 `ui.pak` 120 个精灵中 4 个 `num=5`
（14/31/32/37.pcx，高度 199 = 5×40 帧纵向堆叠）。`SpriteAtlas` 按
`advance_animation(delta_ms)` 推进帧号，`frame()` 取当前帧，blit 时按
`frame_offset(frame)` 切帧。每帧驱动由壳层用 `macroquad::time::get_time()`
算本帧耗时写入 `GameApp::frame_delta_ms`。

### 5.4 响应式布局判定

```
screen_w = physical_width / dpi_scale
screen_h = physical_height / dpi_scale

if screen_w < 480:
    layout = MobilePortrait    // 竖屏：游戏区上 + 手柄下
elif screen_w < 800:
    layout = MobileLandscape   // 横屏：游戏区左 + 手柄右
else:
    layout = Desktop            // 桌面：游戏区居中 + 浮动控件 + 侧栏
```

---

## 6. UI 外壳现代化

### 6.1 设置菜单（新增）

| 项 | 选项 | 默认 |
|---|---|---|
| 缩放模式 | 等比 Smooth / 整数倍 Integer / 拉伸 Stretch / 裁剪 ZoomCrop | 等比 Smooth |
| 上采样滤波 | 像素锐利 Nearest / 平滑 Linear | 像素锐利 Nearest |
| 边框装饰 | 纯黑 Black / 深蓝 DeepBlue / 同背景 MatchBackdrop | 深蓝 DeepBlue |
| 怀旧滤镜 | 开 / 关 | 关 |
| 音效音量 | 0~100% | 80% |
| BGM 音量 | 0~100% | 60% |
| 虚拟手柄 | 开 / 关（触屏自动检测） | 自动 |
| 手柄布局 | 左手 / 右手 / 居中 | 居中 |
| 主题 | 深色 / 浅色 | 深色 |
| 语言 | 中文 / English | 中文 |

**缩放/滤波/边框快捷键**（已实现，S6.1/S6.2）：`F1` 循环缩放模式、`F2` 循环滤波、`F3` 循环边框样式。三键在 native + wasm 均接线，Android 共用 GameApp 透传。

### 6.2 存档管理（现代化）

- 存档列表显示：槽位号 + 角色名 + 地图名 + 金量 + 时间戳
- 存档缩略图：主地图当前画面截图
- 导出/导入：JSON 文件下载/上传（浏览器）/ 文件分享（Android）
- 云同步接口（预留，不实现）

### 6.3 加载界面

```
启动 → 显示游戏标题图 + 进度条
进度 = 首屏资源加载（wasm + 字体 + settings.dat）
进度条满 → 主菜单
```

---

## 7. 音频现代化

| 项 | 现状 | 增强 |
|---|---|---|
| BGM | OGG 循环播放 | 音量滑块 + 淡入淡出切换 |
| 音效 | WAV 即时播放 | 音效音量独立 + 通道限制（≤8 并发） |
| 静音 | — | 全局静音开关 + 后台自动静音（浏览器 visibilitychange） |

---

## 8. 实现路线图

| 阶段 | 内容 | 状态 |
|---|---|---|
| M1 | 双层渲染框架（游戏区 texture blit + UI 层 macroquad 原生绘制） | ✅ S5.1 GameApp |
| M2 | TTF 字体集成 + 响应式布局判定 | ⏸ 待做（设置类新界面用） |
| M3 | 虚拟手柄 + 手势 + 手柄映射 | ✅ S5.5 触屏（直接触摸，无虚拟按键） |
| M4 | 设置菜单 + 现代化存档管理 | ⏸ 待做（F1/F2/F3 快捷键已接线） |
| M5 | PWA 清单 + service worker + 离线缓存 | ⏸ 待做 |
| M6 | Android NDK + APK 构建 + scoped storage | ⏸ 待 Mac（S2.6 阻塞） |
| M7 | 可选 CRT 滤镜 + 美术增强 | ⏸ 待做 |
| M8 | 上采样滤波（Nearest/Linear） | ✅ S6.1 |
| M9 | 边框装饰（letterbox 主题色填充） | ✅ S6.2 |
| M10 | 地形瓦片 blit（等距投影 + 摄像机） | ✅ S6.3 |
| M11 | 多帧动画（filenum INI num>1） | ✅ S6.4 |
| M12 | 容器条目业务名修正（key 哈希 → 可读名） | ✅ S6.5 |

---

## 9. 资源体积预算

> 2026-10-10 修订（S5.6/S6.1~S6.4 后实测）：首屏含 8 场景 `.Map` + `ui.pak` + `glpics.pak`
> + 地形 `glmap_m01.pak`/`glmapm01.Map`（bin 启动即 fetch）。

| 项 | 预算 | 实测 | 说明 |
|---|---|---|---|
| wasm 主体 | ≤ 800 KB | 745 KB | S5.6/S6.1~S6.4 后 wasm 增长（sprite_atlas + terrain_atlas + 触屏/滤波/边框） |
| 首屏资源 | ≤ 490 KB | 308 KB | 字体 + 8 场景 .Map + ui.pak + glpics.pak + 地形 pak + settings |
| TTF 字体 | ≤ 4 MB | — | 思源黑体子集（非首屏分包，待 M2） |
| 非首屏资源 | ≤ 180 KB | 250 KB（超限，S5.6/S6.1~S6.4 后新增内容，预算重列见 release.md §体积预算） | glmap_m02/m03 + cards/goods + music.lpk + 其余场景 .Map |

**首屏明细**（`build_wasm.ps1` 实测 308,227 B）：

| 类别 | 文件 | 字节 |
|---|---|---|
| 字体 | GoldLand.Fnt + font.dat + settings.dat | 26 KB |
| 精灵 | ui.pak + glpics.pak | 185 KB |
| 场景布局 | 8 个 .Map | 31 KB |
| 地形 | glmap_m01.pak + glmapm01.Map | 55 KB |

**非首屏明细**（250,154 B）：glmap_m02.pak（30 KB）+ glmapm02/m03.Map（27 KB）
+ cards.lpk（33 KB）+ goods.lpk（0.5 KB）+ music.lpk（66 KB）+ 其余 18 场景 .Map（47 KB）。

**HD 资产管线（S7，单列，与原版 380 KB 口径并存）**（`docs/impl/14_asset_enhancement.md` §6）：

| 类别 | 数量（manifest 实测） | 单张预估（2x PNG） | 小计 | 实测落盘（`assets_hd_pack/` 各档） |
|---|---|---|---|---|
| A. UI 控件 | 60（30 基础 + 30 九宫格 `_9s`） | 5~15 KB | ~300 KB | 1x 档 `ui/`+`glpics/`，见 §6 表 |
| B. 角色立绘 | 21 | 30~80 KB（4x 超分后） | ~600 KB | 1x 档 `glpics/`（21 条） |
| C. 兵种/地标 | 152 | 5~20 KB | ~1 MB | 1x 档 `ui/`+`glpics/`（152 条） |
| D. 地形瓦片 | 82 | 15~50 KB | ~3 MB | 1x 档 `glmap_m01/`+`glmap_m02/`（82 条） |
| E. 多帧动画 | 4（×5帧=20帧） | 5~10 KB/帧 | ~150 KB | 1x 档 `ui/*_f0..f4.png`（20 条，分帧命名） |
| **合计** | **335 文件**（A-E，各档相同） | | **~5 MB（2x）**；4x ≈ 20 MB | 1x **1,519,349 B（1.45 MB）** / 2x **1,936,304 B（1.85 MB）** / 4x **2,494,827 B（2.38 MB）** / webp **1,136,388 B（1.08 MB）** |

> E 类多帧精灵（f1=14/31/32/37）在 `assets_hd_pack/1x/ui/` 以**分帧命名** `NN_hd_fK.png`
> （非全图 `NN_hd.png`），Rust 侧 `load_hd_pngs` 按 `(f1, Vec<Vec<u8>>)` 注入帧列表，
> `blit_sprite_rgba` 按 `frame % frames` 取对应 PNG；缺帧回退 PCX 调色板扩色。

- 首屏策略：只载 1x（实测 335 文件 1.45 MB，预算 1.6 MB → PASS）+ 关键场景 2x（1.85 MB）；4x（2.38 MB）仅桌面非首屏分包。
- WebP 转码（`assets_hd_pack/webp/`，实测 1,136,388 B ≈ 1.08 MB，较 1x PNG 减 25%；wasm 非首屏首选）。
- 实际落盘体积以 `evidence/S7.6-size.log`（`build_wasm.ps1` 实测）为准。
- 画质切换：`F4` 键循环（原版/HD），状态行显示当前画质 + 快捷键提示（`QualityMode::label()`）。

**四键快捷键**（native + wasm + android）：`F1` 缩放模式 / `F2` 上采样滤波 / `F3` 边框样式 / `F4` 画质模式（原版/HD，feature `hd-8bit` 门控）。
| PWA service worker | ≤ 10 KB | 预缓存清单 |
