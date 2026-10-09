# 现代化改造方案（Modernization Plan）

原版：Windows Mobile ARM，176×220 单点触屏 + D-Pad。
目标：浏览器（WASM）+ Android + 桌面三端，现代触屏 + 键盘 + 手柄。

---

## 1. 双层渲染架构

```
┌──────────────────────────────────────────┐
│  HD UI 层（macroquad 原生绘制，矢量字体）    │
│  ┌────────────────────────────────────┐  │
│  │  游戏画面（176×220 RGB565 帧缓冲）  │  │
│  │  → 整数缩放 texture blit             │  │
│  └────────────────────────────────────┘  │
│  虚拟手柄 / 状态栏 / 菜单 / 对话框         │  ← 原生分辨率
└──────────────────────────────────────────┘
```

- **游戏画面**：保持 176×220 逻辑帧缓冲（`Framebuf`），整数倍缩放（2x/3x/4x）后 blit 到 macroquad texture。保证像素级忠实原版。
- **UI 外壳**：用 macroquad 原生绘制（`draw_text` + TTF），按屏幕 DPI 动态字号，不经过 176×220 帧缓冲。
- **脏行带**：游戏画面复用现有 `FramePresenter`（S4.3 脏行带部分上传，静止帧零 GPU）；UI 外壳每帧重绘（现代 GPU 足够）。

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

---

## 9. 资源体积预算

| 项 | 预算 | 说明 |
|---|---|---|
| wasm 主体 | ≤ 800 KB | S2.5 实测 670KB |
| 首屏资源 | ≤ 150 KB | 字体 + settings.dat |
| TTF 字体 | ≤ 4 MB | 思源黑体子集（非首屏分包） |
| 非首屏资源 | ≤ 100 KB | music.lpk |
| PWA service worker | ≤ 10 KB | 预缓存清单 |
