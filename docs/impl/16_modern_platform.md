# 16 — 现代多端实现指南（Mobile / Desktop / Web）

> 本文档定义把《金银岛》从原版 Windows Mobile ARM 176×220 升级为**现代多端可玩游戏**
> 的完整实现规格：架构选型、渲染管线、输入系统、音频、持久化、PWA、Android、性能预算、
> 资源分包策略。读者仅凭本文档 + 00_index.md 架构总览即可搭建三端运行框架。
>
> 关联文档：
> - `docs/modernization.md`（方案总纲，本文件是其**实现级展开**）
> - `docs/impl/11_modernization.md`（双层渲染 + 响应式布局规格）
> - `docs/impl/12_audio.md`（音频系统规格）
> - `docs/impl/09_persistence.md`（存读档规格）
> - `docs/impl/14_asset_enhancement.md`（HD 资产管线）

---

## 1. 目标与范围

### 1.1 三端目标

| 端 | 渲染后端 | 资源 IO | 输入 | 存档 | 状态 |
|---|---|---|---|---|---|
| 桌面（Win/Mac/Linux） | macroquad native（OpenGL） | 本地文件 `data/` | 键盘 + 鼠标 + 手柄 | `./data/gl*.sav` | 已通 |
| 浏览器（WASM） | macroquad miniquad GL | HTTP fetch `web/` | 触屏 + 键盘 | IndexedDB / LocalStorage | 已通 |
| Android | macroquad native（NDK） | APK assets / scoped storage | 触屏 + 手柄 | `getExternalFilesDir` | 待 Mac 构建 |

### 1.2 设计原则

| 原则 | 说明 |
|---|---|
| 单一游戏逻辑 | 玩法层（`ti-core`）零平台依赖，三端共享同一套代码 |
| 平台抽象隔离 | 输入/存储/音频/定时 全部走 `ti-platform` trait，不直接调 OS API |
| 像素忠实 + 现代外壳 | 游戏画面保持 176×220 逻辑帧缓冲，UI 外壳原生分辨率 |
| 数据驱动 | 数值/文本/场景布局 全部进 JSON，改数据不改代码 |
| 确定性优先 | RNG 可播种、可复现，同 seed 同操作序列 ⇒ 同结果 |

---

## 2. 架构分层

### 2.1 模块依赖（单向，无环）

```
ti-formats (资源解析: pak/lpk/pcx/map/font/save)
    ↑
ti-core (核心类型/常量/规则/RNG/错误 — 零平台依赖)
    ↑
ti-platform (平台抽象: 输入/存储/音频/渲染/定时)
    ↑
ti-app (游戏逻辑: 场景状态机/经济/战斗/卡牌/渲染管线)
    ↑
ti-tools (开发工具: ledger/doccheck/serve/controls — 不依赖 ti-app)
```

### 2.2 核心类型（ti-core）

| 类型 | 说明 |
|---|---|
| `Scene` | 9 场景枚举 |
| `GameState` | 全局游戏状态（角色/地图/金量/货物/建筑/战斗/RNG） |
| `SceneStack` | 场景栈（state/scene/history） |
| `GameRng` | 可播种 LCG（确定性） |
| `BattleSession` | 战斗会话状态机 |
| `CardEngine` | 卡牌引擎 |
| `TradeLedger` | 贸易账本 |
| `BuildManager` | 建造管理器 |
| `Bank` | 银行双槽位 |
| `SaveManager` | 存读档管理 |

### 2.3 帧循环（60fps）

```rust
fn main() {
    let mut game = GameApp::new(176, 220);
    game.atlas = Some(SpriteAtlas::load("ui.pak"));
    game.shell.render(&game.scene, &game.state, &game.atlas);

    while !game.state.quit {
        game.shell.refit(screen_w, screen_h);
        let keys = poll_input();
        game.input.update(&keys);
        if game.input.consume_changed() {
            game.scenes.update(&mut game.state);
        }
        game.shell.render(&game.scene, &game.state, &game.atlas);
        game.shell.present();
        next_frame().await;
    }
}
```

---

## 3. 双层渲染架构

### 3.1 架构概览

```
┌──────────────────────────────────────────────────────────┐
│  HD UI 层（macroquad 原生绘制，矢量字体 TTF）               │
│  ┌────────────────────────────────────────────────────┐  │
│  │  游戏画面（176×220 RGB565 帧缓冲）                   │  │
│  │  → 整数缩放 texture blit（2x/3x/4x）                │  │
│  └────────────────────────────────────────────────────┘  │
│  虚拟手柄 / 状态栏 / 菜单 / 对话框                          │
└──────────────────────────────────────────────────────────┘
```

### 3.2 游戏画面层

| 属性 | 值 |
|---|---|
| 逻辑分辨率 | 176 × 220 |
| 像素格式 | RGB565（16 位色）/ RGBA8（HD 模式） |
| 缩放方式 | 整数倍（2x/3x/4x） |
| 纹理上传 | 脏行带部分上传（静止帧零 GPU） |
| 渲染后端 | `FramePresenter`（native FullRebuild / wasm Partial） |

### 3.3 UI 外壳层

| 属性 | 值 |
|---|---|
| 渲染方式 | macroquad 原生绘制（`draw_text` + TTF） |
| 字号策略 | 按屏幕 DPI 动态字号 |
| 帧率 | 每帧重绘（现代 GPU 足够） |
| 输入 | 键盘 + 鼠标 + 触屏 + 手柄 |

### 3.4 缩放档位

| 屏幕短边 | 缩放倍数 | 游戏画面像素 | UI 层 |
|---|---|---|---|
| ≥ 720px | 4x | 704 × 880 | 原生 |
| ≥ 528px | 3x | 528 × 660 | 原生 |
| ≥ 352px | 2x | 352 × 440 | 原生 |
| < 352px | 自适应填充 | ≤ 176 × 220 | 原生 |

**缩放算法**：

```
short_side = min(screen_w, screen_h)
if short_side >= 720: scale = 4
elif short_side >= 528: scale = 3
elif short_side >= 352: scale = 2
else: scale = 1
game_w = 176 × scale
game_h = 220 × scale
```

### 3.5 高 DPI 感知

```
dpi_scale = screen_dpi / 96.0
font_size = base_size × dpi_scale
ui_rect = physical_pixels / dpi_scale   // 逻辑像素
```

| 平台 | DPI 来源 |
|---|---|
| Windows | `GetDpiForWindow` |
| macOS | `NSScreen.backingScaleFactor` |
| 浏览器 | `devicePixelRatio` |
| Android | `DisplayMetrics.density` |

---

## 4. 响应式布局

### 4.1 布局判定

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

### 4.2 横屏布局（手机/平板）

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

### 4.3 竖屏布局（手机）

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

### 4.4 触屏输入（S5.5 实际方案）

**理念**：不画虚拟方向键，玩家手指点哪就是哪。

| 场景 | 点按 | 滑动 |
|---|---|---|
| Index 主菜单 | 点菜单行 = 选中并执行 | — |
| SelectRole/SelectMap | 点某行 = 选中 + 确认 | 下滑 = 返回 |
| MainMap | 点格子 = 朝该格走 | 滑动 = 走一步 |
| Vs 战斗 | 点任意处 = 攻击 | 下滑 = 退出 |
| Message | 左半屏 = 关闭；右半屏 = 下一页 | 上滑 = 下一页 |
| Saveload | 点有档槽 = 读档；点空槽 = 存档 | 下滑 = 关闭 |

**判定参数**：

| 参数 | 值 | 说明 |
|---|---|---|
| 滑动阈值 | > 20 逻辑像素 | 避免误触 |
| 长按时长 | 500ms | 触发 Cancel |
| 短按 | < 200ms | 触发 Confirm |
| 双指捏合 | 距离变化 > 30px | 触发 CycleScale |

---

## 5. 输入系统

### 5.1 物理键映射

| 物理键 | GameInput | 说明 |
|---|---|---|
| ↑ / W | Up | 上移/上一个 |
| ↓ / S | Down | 下移/下一个 |
| ← / A | Left | 左移 |
| → / D | Right | 右移 |
| Enter / Space | Confirm | 确定 |
| Esc / Backspace | Cancel | 返回 |
| F1 | CycleScale | 循环缩放模式 |
| F2 | CycleFilter | 循环滤波模式 |
| F3 | CycleBorder | 循环边框样式 |
| F4 | CycleQuality | 循环画质（原版/HD） |

### 5.2 手柄映射

| 手柄输入 | GameInput | 说明 |
|---|---|---|
| 左摇杆上/下/左/右 | Up/Down/Left/Right | 方向 |
| 十字键上/下/左/右 | Up/Down/Left/Right | 方向 |
| A 键 | Confirm | 确定 |
| B 键 | Cancel | 返回 |

### 5.3 输入去抖

- 同一帧内多键同时按下，按场景语义合并（MainMap 中 Confirm 优先于方向）
- 场景切换时重置输入状态，避免上一帧按键残留导致新场景误触发

---

## 6. 字体策略

### 6.1 双字体方案

| 用途 | 字体 | 大小 | 说明 |
|---|---|---|---|
| 游戏内文本 | `GoldLand.Fnt` 12×12 点阵 | 整数缩放后 24~48px | 保持原版像素感 |
| 菜单/帮助/设置 | TTF 矢量字体（思源黑体/Noto Sans CJK） | 16~24px × DPI | 清晰可读 |

### 6.2 TTF 集成

```rust
let font = load_ttf_font("NotoSansCJK-Regular.ttf").await?;
draw_text_ex(
    "新游戏", x, y,
    TextParams {
        font: Some(&font),
        font_size: (24.0 * dpi_scale) as u16,
        color: WHITE,
        ..Default::default()
    },
);
```

| 文件 | 大小 | 加载时机 |
|---|---|---|
| `GoldLand.Fnt` | ~26 KB | 首屏 |
| `NotoSansCJK-Regular.ttf` | ~4 MB | 非首屏分包 |

### 6.3 字形光栅

- 真 `GoldLand.Fnt` 墨点渲染
- 缺字画**豆腐块**回退（绝不静默空白）
- 字库字符集 1016 字（`font.dat` UTF-16 字符集清单）
- 字形步进 `GLYPH_ADVANCE = 12`，墨色 `INK_COLOR = 0xFFFF`

---

## 7. 渲染管线

### 7.1 帧缓冲

```rust
pub const LOGICAL_W: u32 = 176;
pub const LOGICAL_H: u32 = 220;
pub const FB_PIXELS: usize = (LOGICAL_W * LOGICAL_H) as usize;

pub struct Framebuf {
    pixels: Box<[u16]>,        // RGB565
    // 或 pixels: Box<[u32]>,  // RGBA8（HD 模式，cfg hd-8bit）
    dirty_rows: [bool; 220],
}

impl Framebuf {
    pub fn clear(&mut self, color: u16);
    pub fn mark_dirty(&mut self, x: i32, y: i32, w: u32, h: u32);
    pub fn has_dirty(&self) -> bool;
    pub fn dirty_row_range(&self) -> (usize, usize);
    pub fn reset_dirty(&mut self);
}
```

### 7.2 呈现器

```rust
pub enum UploadMode { FullRebuild, Partial }

pub struct FramePresenter { mode: UploadMode }

impl FramePresenter {
    pub fn present(&self, fb: &Framebuf) {
        if !fb.has_dirty() { return; }  // 静止帧零 GPU
        let (min, max) = fb.dirty_row_range();
        match self.mode {
            UploadMode::FullRebuild => { /* 全帧上传 */ }
            UploadMode::Partial => { /* 只上传 [min, max] 行区间 */ }
        }
    }
}
```

### 7.3 视口坐标变换

```rust
pub struct Viewport { scale: f32, offset_x: f32, offset_y: f32 }

impl Viewport {
    pub fn screen_to_logical(&self, sx: f32, sy: f32) -> (f32, f32);
    pub fn logical_to_screen(&self, lx: f32, ly: f32) -> (f32, f32);
}
```

等比缩放（letterbox/pillarbox），保证任意窗口尺寸下坐标互逆。

### 7.4 绘制顺序（主地图）

```
1. 清屏（背景色）
2. 绘制地形瓦片（painter's 算法，按 iso_y 非降序）
3. 绘制连线（道路/路线）
4. 绘制据点标记
5. 绘制单位精灵
6. 绘制 UI 覆盖层（HUD 状态行）
```

### 7.5 多帧动画

`filenum INI` 的 `num` 字段是帧数。实测 `ui.pak` 120 个精灵中 4 个 `num=5`
（14/31/32/37.pcx，高度 199 = 5×40 帧纵向堆叠）。

```rust
SpriteAtlas::advance_animation(delta_ms)  // 推进帧号
SpriteAtlas::frame()                     // 取当前帧
blit 时按 frame_offset(frame) 切帧
每帧驱动：壳层用 get_time() 算本帧耗时写入 GameApp::frame_delta_ms
```

### 7.6 静态帧优化

- `last_rendered: Option<usize>` 未变化时跳过整帧
- `last_rendered_scene: Option<i32>` 场景切换时强制整帧重渲染

---

## 8. 音频系统

### 8.1 音频资源

| 文件 | 类型 | 触发 | 加载时机 |
|---|---|---|---|
| `music/press.wav` | WAV | 事件码 13（有效确认） | 首屏 |
| `music/invalid.wav` | WAV | 事件码 37（无效操作） | 首屏 |
| `music/victory.wav` | WAV | 战斗胜利 | 战斗场景按需 |
| `music/glmusic.ogg` | OGG | 主菜单/主地图 BGM | 主菜单 |

### 8.2 通道架构

| 通道类型 | 数量 | 用途 | 并发限制 |
|---|---|---|---|
| BGM 通道 | 1 | 背景音乐 | 1（独占） |
| 音效通道 | 8 | 音效播放 | ≤ 8 并发 |

### 8.3 音量控制

| 音量通道 | 范围 | 默认 | 存储位置 |
|---|---|---|---|
| BGM 音量 | 0~100% | 60% | `settings.dat` offset 6 |
| 音效音量 | 0~100% | 80% | `settings.dat` offset 7 |
| 主音量 | 0~100% | 100% | 运行时计算 |

```
effective_volume = channel_volume × master_volume × mute_factor
mute_factor = 0.0 if muted else 1.0
```

### 8.4 BGM 播放

```
play_bgm(bgm):
    if bgm_channel.state == Playing:
        fade_out(bgm_channel, 500ms)
    load_source(bgm_channel, bgm)
    set_volume(bgm_channel, bgm_volume × master_volume)
    start_loop(bgm_channel)
```

### 8.5 音效播放

```
play_sfx(sfx):
    channel = find_idle_channel()
    if channel == None:
        channel = steal_oldest_channel()
    load_source(channel, sfx)
    set_volume(channel, sfx_volume × master_volume)
    play_once(channel)
```

### 8.6 浏览器自动播放策略

| 浏览器 | 策略 |
|---|---|
| Chrome/Safari/Firefox | 需要用户交互后才能播放音频 |

**处理方案**：

```
init_audio():
    if browser && audio_context.state == "suspended":
        show_overlay("点击任意处开始")
        on_first_click:
            audio_context.resume()
            start_bgm()
```

### 8.7 静音控制

| 触发 | 行为 |
|---|---|
| 用户切换静音 | `mute_factor = 0.0`，所有通道静音 |
| 浏览器 `visibilitychange` | 后台自动静音 |
| 恢复前台 | 恢复原音量 |

### 8.8 平台音频后端

| 端 | 后端 | 说明 |
|---|---|---|
| 桌面 native | `rodio` | 跨平台音频 |
| 浏览器 wasm | Web Audio API | 浏览器原生 |
| Android | OpenSL ES | NDK 音频 |

---

## 9. 持久化（多端）

### 9.1 文件清单

| 文件 | 大小 | 用途 | 桌面路径 | 浏览器路径 | Android 路径 |
|---|---|---|---|---|---|
| `settings.dat` | 8 B | 全局设置 | `./data/` | `web/` | scoped storage |
| `gl0.sav` ~ `gl4.sav` | 26 B (v2) | 5 角色存档槽 | `./data/` | IndexedDB | scoped storage |
| `gl.rec` | 10 B | 全局进度记录 | `./data/` | `web/` | scoped storage |

### 9.2 平台 IO 抽象

```rust
pub trait SaveIo {
    async fn read(&self, path: &str) -> Result<Vec<u8>, SaveErr>;
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), SaveErr>;
    async fn exists(&self, path: &str) -> bool;
    async fn delete(&self, path: &str) -> Result<(), SaveErr>;
}
```

| 端 | 实现 |
|---|---|
| 桌面 native | `LocalFileIo`（`std::fs`） |
| 浏览器 wasm | `WasmFileIo`（localStorage / IndexedDB） |
| Android | `AndroidFileIo`（scoped storage） |

### 9.3 数据完整性

1. **写前读**：写档前读旧文件，保留备份 `gl{N}.sav.bak`
2. **原子写入**：先写 `.tmp`，成功后 `rename` 覆盖
3. **首次启动**：无 `settings.dat` 创建默认；无 `gl.rec` 创建全零
4. **CRC 校验**（v3 预留）：v2 末尾可追加 4B CRC32

### 9.4 现代化存档管理

- 存档列表显示：槽位号 + 角色名 + 地图名 + 金量 + 时间戳
- 存档缩略图：主地图当前画面截图
- 导出/导入：JSON 文件下载/上传（浏览器）/ 文件分享（Android）
- 云同步接口（预留，不实现）

---

## 10. 设置菜单

### 10.1 设置项

| 项 | 选项 | 默认 | 快捷键 |
|---|---|---|---|
| 缩放模式 | Smooth / Integer / Stretch / ZoomCrop | Smooth | F1 |
| 上采样滤波 | Nearest / Linear | Nearest | F2 |
| 边框装饰 | Black / DeepBlue / MatchBackdrop | DeepBlue | F3 |
| 画质模式 | 原版 / HD | 原版 | F4 |
| 音效音量 | 0~100% | 80% | — |
| BGM 音量 | 0~100% | 60% | — |
| 虚拟手柄 | 开 / 关（触屏自动检测） | 自动 | — |
| 手柄布局 | 左手 / 右手 / 居中 | 居中 | — |
| 主题 | 深色 / 浅色 | 深色 | — |
| 语言 | 中文 / English | 中文 | — |

### 10.2 设置持久化

设置保存到 `settings.dat`（8 字节）：

| 偏移 | 字段 | 值 |
|---|---|---|
| 0 | 前缀 | `5D 4A` |
| 2 | 负载 | `71 4A 2E 01` |
| 6 | BGM 音量 | 0..=100 |
| 7 | 音效开关 | 0 或 1 |

---

## 11. PWA 支持（浏览器端）

### 11.1 manifest.json

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

### 11.2 Service Worker

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

### 11.3 启动画面

```
启动 → 显示游戏标题图 + 进度条
进度 = 首屏资源加载（wasm + 字体 + settings.dat）
进度条满 → 主菜单
```

---

## 12. Android 适配

### 12.1 构建配置

```toml
# ti-app Cargo.toml
[features]
android = ["macroquad/android"]
```

- `cargo-apk build` 或 `cargo-ndk` → APK
- 构建在 Mac（S2.6 阻塞项），本机 Windows 不产出

### 12.2 资源与存储

| 项 | 方案 |
|---|---|
| 资源读取 | APK assets（`miniquad` 提供 `android-asset`） |
| 存档写入 | scoped storage（`getExternalFilesDir`） |
| 屏幕方向 | `AndroidManifest.xml` 设 `screenOrientation="landscape"` |

### 12.3 AndroidManifest.xml

```xml
<activity
    android:name="com.treasureisl.MainActivity"
    android:screenOrientation="landscape"
    android:configChanges="orientation|screenSize">
</activity>
```

### 12.4 输入

- 触屏：同浏览器端手势方案（§4.4）
- 手柄：Android Game Controller API（蓝牙/USB 手柄）
- 横屏锁定：`screenOrientation="landscape"`

---

## 13. 资源分包策略

### 13.1 首屏资源（≤ 490 KB 预算，实测 308 KB）

| 类别 | 文件 | 字节 |
|---|---|---|
| 字体 | GoldLand.Fnt + font.dat + settings.dat | 26 KB |
| 精灵 | ui.pak + glpics.pak | 185 KB |
| 场景布局 | 8 个 .Map | 31 KB |
| 地形 | glmap_m01.pak + glmapm01.Map | 55 KB |
| **合计** | | **308 KB** |

### 13.2 非首屏资源（按需加载）

| 类别 | 文件 | 字节 |
|---|---|---|
| 地形 | glmap_m02.pak + glmapm02/m03.Map | 57 KB |
| 卡牌 | cards.lpk | 33 KB |
| 货物 | goods.lpk | 0.5 KB |
| 音乐 | music.lpk | 66 KB |
| 场景 | 其余 18 场景 .Map | 47 KB |
| **合计** | | **250 KB** |

### 13.3 加载策略

```
启动
  → 加载首屏资源（wasm + 字体 + settings + ui.pak + glpics + m01 地形）
  → Intro 片头
  → Index 主菜单

进入 MainMap（m02/m03）
  → 按需加载对应地形 pak + .Map
  → 加载 cards.lpk / goods.lpk

进入战斗
  → 加载 victory.wav（如未加载）

BGM 切换
  → 按需加载 glmusic.ogg
```

### 13.4 体积预算总表

| 项 | 预算 | 实测 | 说明 |
|---|---|---|---|
| wasm 主体 | ≤ 800 KB | 745 KB | 含 sprite_atlas + terrain_atlas + 触屏/滤波/边框 |
| 首屏资源 | ≤ 490 KB | 308 KB | 见 §13.1 |
| TTF 字体 | ≤ 4 MB | — | 思源黑体子集（非首屏分包） |
| 非首屏资源 | ≤ 180 KB | 250 KB | 超限，需重列预算 |
| PWA service worker | ≤ 10 KB | — | 预缓存清单 |
| HD 资产（S7，单列） | 1x ~1.45 MB / 2x ~1.85 MB / 4x ~2.38 MB | 实测 | 见 14_asset_enhancement.md §6 |

---

## 14. 性能预算

| 指标 | 预算 | 说明 |
|---|---|---|
| 帧率 | 60 fps | 桌面 + 中端手机 |
| 帧时间 | ≤ 16.7 ms | 含输入 + 逻辑 + 渲染 |
| 输入响应 | < 16 ms | 按键到画面更新 |
| 场景切换 | < 100 ms | 资源已在内存 |
| 首屏加载（wasm） | < 3 s（宽带） | 745 KB wasm + 308 KB 资源 |
| 存档读写 | < 50 ms | 26 字节 v2 槽 |
| 内存占用 | < 64 MB（运行时） | 桌面；移动端 < 48 MB |

---

## 15. 错误处理

### 15.1 错误类型

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Formats(#[from] ti_formats::Error),
    #[error(transparent)]
    Platform(#[from] ti_platform::Error),
}
```

### 15.2 资源加载失败优雅降级

| 失败场景 | 降级行为 |
|---|---|
| 地形 pak 缺失 | 跳过该地图，SelectMap 只列已加载地图 |
| 精灵缺失 | 画豆腐块（不静默空白） |
| 音频缺失 | 静音继续（不阻塞） |
| 存档损坏 | 显示「存档损坏」，提供重试/新建选项 |
| 字体缺失 | 回退系统默认字体 |

### 15.3 用户可见错误

| 错误 | 提示文案 | 可操作 |
|---|---|---|
| `SaveErr::TooShort` | 「存档损坏」 | 重试 / 新建 |
| `SaveErr::BadMagic` | 「存档损坏」 | 重试 / 新建 |
| `SaveErr::BadVersion` | 「版本不兼容」 | 新建 |
| `SaveErr::Io` | 「读写失败」 | 重试 |
| `TradeErr::InsufficientGold` | 「金量不足」 | 取消 |
| `BuildErr::Insufficient` | 「金量不足」 | 取消 |

---

## 16. 现代化功能清单（状态总览）

| 功能 | 状态 | 说明 |
|---|---|---|
| 双层渲染框架 | ✅ | S5.1 GameApp |
| 上采样滤波（Nearest/Linear） | ✅ | S6.1 |
| 边框装饰（letterbox 主题色） | ✅ | S6.2 |
| 地形瓦片 blit（等距投影 + 摄像机） | ✅ | S6.3 |
| 多帧动画 | ✅ | S6.4 |
| 容器条目业务名修正 | ✅ | S6.5 |
| 触屏直接输入（无虚拟按键） | ✅ | S5.5 |
| TTF 字体 + 响应式布局 | ⏸ 待做 | M2 |
| 设置菜单（完整 10 项） | ⏸ 待做 | M4（F1/F2/F3 已接线） |
| PWA manifest + service worker | ⏸ 待做 | M5 |
| Android NDK + APK | ⏸ 待 Mac | M6 |
| CRT 怀旧滤镜 | ⏸ 待做 | M7 |
| 现代化存档管理（缩略图/导出导入） | ⏸ 待做 | M4 |
| 手柄映射 | ⏸ 待做 | 桌面 + Android |

---

## 17. 实现优先级

| 优先级 | 内容 | 依赖 |
|---|---|---|
| P0 | 场景状态机 + 主菜单 + 角色/地图选择 | ti-core + ti-formats |
| P1 | 主地图渲染 + 移动 + 摄像机 | P0 + 地形 pak |
| P2 | 贸易 + 建造 + 银行 | P1 |
| P3 | 战斗 + 卡牌事件 | P2 |
| P4 | 存读档 + 设置 | P3 |
| P5 | 音频系统 | P0 |
| P6 | 响应式布局 + 触屏 | P1 |
| P7 | PWA + Android | P6 |

---

## 18. 验收标准

1. **三端编译**：native / wasm / android host 均可编译
2. **场景全覆盖**：9 场景均可进入/返回，无死锁
3. **输入正确**：每个按键/手势在每个场景的行为符合映射表
4. **资源注入**：ui.pak + 9 场景 .Map 正确加载并渲染
5. **数值精确**：价格表、BOSS HP、伤害公式、LCG 序列逐位一致
6. **全流程可玩**：开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单
7. **响应式布局**：竖屏/横屏/桌面三档布局正确切换
8. **音频正常**：BGM 循环 + 音效触发 + 音量控制 + 静音
9. **存档跨端**：桌面存档可在浏览器读取（格式一致）
10. **性能达标**：60fps 帧率、< 16ms 输入响应、< 64MB 内存
