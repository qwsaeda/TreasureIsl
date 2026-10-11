# 11 — 现代化改造实现规格

> 单一事实源：本文件是 `ti-app` 表现层 + `ti-platform` 渲染后端的现代化实现规格。
> 关联文档：`docs/modernization.md`（方案总纲）、`docs/design/constants.md`（常量表）。

---

## 1. 双层渲染架构

### 1.1 架构概览

```
┌──────────────────────────────────────────────────────────┐
│  HD UI 层（macroquad 原生绘制，矢量字体）                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │  游戏画面（176×220 RGB565 帧缓冲）                   │  │
│  │  → 整数缩放 texture blit                            │  │
│  └────────────────────────────────────────────────────┘  │
│  虚拟手柄 / 状态栏 / 菜单 / 对话框                         │
└──────────────────────────────────────────────────────────┘
```

### 1.2 游戏画面层

| 属性 | 值 |
|---|---|
| 逻辑分辨率 | 176 × 220 |
| 像素格式 | RGB565（16 位色） |
| 缩放方式 | 整数倍（2x/3x/4x） |
| 纹理上传 | 脏行带部分上传（静止帧零 GPU） |
| 渲染后端 | `FramePresenter`（native FullRebuild / wasm Partial） |

### 1.3 UI 外壳层

| 属性 | 值 |
|---|---|
| 渲染方式 | macroquad 原生绘制（`draw_text` + TTF） |
| 字号策略 | 按屏幕 DPI 动态字号 |
| 帧率 | 每帧重绘（现代 GPU 足够） |
| 输入 | 键盘 + 鼠标 + 触屏 + 手柄 |

### 1.4 Rust 类型

> S6.1/S6.2 落地为 `MenuShell` 帧/边框机制（`ti-app/src/menu_shell.rs`），
> `DualLayerRenderer` 为设计占位，代码无此 struct。

```rust
pub struct DualLayerRenderer {
    game_texture: Texture2D,
    ui_font: Font,
    dpi_scale: f32,
    scale_mode: ScaleMode,
    filter_mode: FilterMode,
    border_style: BorderStyle,
}

pub enum ScaleMode {
    Smooth,      // 等比缩放
    Integer,     // 整数倍
    Stretch,     // 拉伸填充
    ZoomCrop,    // 裁剪填充
}

pub enum FilterMode {
    Nearest,     // 像素锐利
    Linear,      // 平滑
}

pub enum BorderStyle {
    Black,       // 纯黑
    DeepBlue,    // 深蓝
    MatchBackdrop, // 同背景
}
```

---

## 2. 分辨率缩放

### 2.1 整数缩放档

> 注：实际整数 2x/3x/4x 缩放走 `MenuShell` refit（见 `impl/16` §3.4），本表为 DPI 参考。

| 屏幕短边 | 缩放倍数 | 游戏画面像素 | UI 层 |
|---|---|---|---|
| ≥ 720px | 4x | 704 × 880 | 原生 |
| ≥ 528px | 3x | 528 × 660 | 原生 |
| ≥ 352px | 2x | 352 × 440 | 原生 |
| < 352px | 自适应填充 | ≤176 × 220 | 原生 |

### 2.2 缩放算法

```
fn compute_scale(screen_w: u32, screen_h: u32) -> (u32, u32, u32):
    short_side = min(screen_w, screen_h)
    if short_side >= 720: scale = 4
    elif short_side >= 528: scale = 3
    elif short_side >= 352: scale = 2
    else: scale = 1
    game_w = 176 * scale
    game_h = 220 * scale
    return (scale, game_w, game_h)
```

### 2.3 高 DPI 感知

```
dpi_scale = screen_dpi / 96.0    // Windows: 1.0/1.25/1.5/2.0
font_size = base_size × dpi_scale
ui_rect = physical_pixels / dpi_scale  // 逻辑像素
```

| 平台 | DPI 来源 |
|---|---|
| Windows | `GetDpiForWindow` |
| macOS | `NSScreen.backingScaleFactor` |
| 浏览器 | `devicePixelRatio` |
| Android | `DisplayMetrics.density` |

---

## 3. 字体策略

### 3.1 双字体方案

| 用途 | 字体 | 大小 | 说明 |
|---|---|---|---|
| 游戏内文本 | `GoldLand.Fnt` 12×12 点阵 | 整数缩放后 24~48px | 保持原版像素感 |
| 菜单/帮助/设置 | TTF 矢量字体（思源黑体） | 16~24px × DPI | 清晰可读 |

### 3.2 TTF 集成

```rust
// 加载 TTF
let font = load_ttf_font("NotoSansCJK-Regular.ttf").await?;

// 使用
draw_text_ex(
    "新游戏",
    x, y,
    TextParams {
        font: Some(&font),
        font_size: (24.0 * dpi_scale) as u16,
        color: WHITE,
        ..Default::default()
    },
);
```

### 3.3 字体资源

| 文件 | 大小 | 加载时机 |
|---|---|---|
| `GoldLand.Fnt` | ~26 KB | 首屏 |
| `NotoSansCJK-Regular.ttf` | ~4 MB | 非首屏分包 |

---

## 4. 虚拟手柄布局

### 4.1 横屏布局（手机/平板）

```
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
```

### 4.2 竖屏布局（手机）

```
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

### 4.3 触屏输入（S5.5 实际方案）

**理念**：不画虚拟方向键，玩家手指点哪就是哪。

| 场景 | 点按 | 滑动 |
|---|---|---|
| Index 主菜单 | 点菜单行 = 选中并执行 | — |
| SelectRole/SelectMap | 点某行 = 选中 + 确认 | 下滑 = 返回 |
| MainMap | 点格子 = 朝该格走 | 滑动 = 走一步 |
| Vs 战斗 | 点任意处 = 攻击 | 下滑 = 退出 |
| Message | 左半屏 = 关闭；右半屏 = 下一页 | 上滑 = 下一页 |
| Saveload | 点有档槽 = 读档；点空槽 = 存档 | 下滑 = 关闭 |

### 4.4 响应式布局判定

```
screen_w = physical_width / dpi_scale
screen_h = physical_height / dpi_scale

if screen_w < 480:
    layout = MobilePortrait    // 竖屏：游戏区上 + 手柄下
elif screen_w < 800:
    layout = MobileLandscape   // 横屏：游戏区左 + 手柄右
else:
    layout = Desktop           // 桌面：游戏区居中 + 浮动控件 + 侧栏
```

---

## 5. PWA 支持

### 5.1 manifest.json

```json
{
  "name": "金银岛 TreasureIsl",
  "short_name": "金银岛",
  "start_url": "./index.html",
  "display": "fullscreen",
  "orientation": "landscape",
  "icons": [
    { "src": "icon-192.png", "sizes": "192x192", "type": "image/png" },
    { "src": "icon-512.png", "sizes": "512x512", "type": "image/png" }
  ]
}
```

### 5.2 Service Worker

```javascript
const CACHE_NAME = 'treasure-isl-v1';
const ASSETS = [
  './index.html',
  './treasure_wasm.wasm',
  './gl.js',
  './data/settings.dat',
  './data/ui.pak',
  './data/glpics.pak',
  // ... 首屏资源
];

self.addEventListener('install', (e) => {
  e.waitUntil(caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS)));
});

self.addEventListener('fetch', (e) => {
  e.respondWith(caches.match(e.request).then((r) => r || fetch(e.request)));
});
```

---

## 6. Android 适配

### 6.1 构建配置

```toml
# Cargo.toml (ti-app)
[features]
android = ["macroquad/android"]
```

### 6.2 资源与存储

| 项 | 方案 |
|---|---|
| 资源读取 | APK assets（`miniquad` 提供 `android-asset`） |
| 存档写入 | scoped storage（`getExternalFilesDir`） |
| 屏幕方向 | `AndroidManifest.xml` 设 `screenOrientation="landscape"` |
| 构建工具 | `cargo-apk build` 或 `cargo-ndk` |

### 6.3 AndroidManifest.xml 片段

```xml
<activity
    android:name="com.treasureisl.MainActivity"
    android:screenOrientation="landscape"
    android:configChanges="orientation|screenSize">
</activity>
```

---

## 7. 设置菜单规格

### 7.1 设置项

| 项 | 选项 | 默认 |
|---|---|---|
| 缩放模式 | Smooth / Integer / Stretch / ZoomCrop | Smooth |
| 上采样滤波 | Nearest / Linear | Nearest |
| 边框装饰 | Black / DeepBlue / MatchBackdrop | DeepBlue |
| 怀旧滤镜 | 开 / 关 | 关 |
| 音效音量 | 0~100% | 80% |
| BGM 音量 | 0~100% | 60% |
| 虚拟手柄 | 开 / 关（触屏自动检测） | 自动 |
| 手柄布局 | 左手 / 右手 / 居中 | 居中 |
| 主题 | 深色 / 浅色 | 深色 |
| 语言 | 中文 / English | 中文 |

### 7.2 快捷键

| 键 | 功能 |
|---|---|
| `F1` | 循环缩放模式 |
| `F2` | 循环上采样滤波 |
| `F3` | 循环边框样式 |

### 7.3 设置持久化

设置保存到 `settings.dat`（8 字节，见 `09_persistence.md`）。

---

## 8. 音频现代化

| 项 | 现状 | 增强 |
|---|---|---|
| BGM | OGG 循环播放 | 音量滑块 + 淡入淡出切换 |
| 音效 | WAV 即时播放 | 音效音量独立 + 通道限制（≤8 并发） |
| 静音 | — | 全局静音开关 + 后台自动静音 |

详见 `12_audio.md`。

---

## 9. 资源体积预算

| 项 | 预算 | 实测 | 说明 |
|---|---|---|---|
| wasm 主体 | ≤ 800 KB | 745 KB | S5.6/S6.1~S6.4 后增长 |
| 首屏资源 | ≤ 490 KB | 308 KB | 字体 + 9 场景 .Map + ui.pak + glpics.pak |
| TTF 字体 | ≤ 4 MB | — | 思源黑体子集（非首屏分包） |
| 非首屏资源 | ≤ 180 KB | 250 KB | glmap_m02/m03 + cards/goods + music.lpk |

---

## 10. 实现路线图

| 阶段 | 内容 | 状态 |
|---|---|---|
| M1 | 双层渲染框架 | 已完成（S5.1 GameApp） |
| M2 | TTF 字体集成 + 响应式布局 | 待做 |
| M3 | 虚拟手柄 + 手势 + 手柄映射 | 已完成（S5.5 触屏） |
| M4 | 设置菜单 + 现代化存档管理 | 待做 |
| M5 | PWA 清单 + service worker | 待做 |
| M6 | Android NDK + APK 构建 | 待 Mac（S2.6 阻塞） |
| M7 | 可选 CRT 滤镜 | 待做 |
| M8 | 上采样滤波 | 已完成（S6.1） |
| M9 | 边框装饰 | 已完成（S6.2） |
| M10 | 地形瓦片 blit | 已完成（S6.3） |
| M11 | 多帧动画 | 已完成（S6.4） |
| M12 | 容器条目业务名修正（key 哈希 → 可读名） | 已完成（S6.5） |
