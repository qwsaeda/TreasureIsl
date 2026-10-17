# 渲染框架独立契约（ti-render）

**定位**：把渲染框架从 `ti-app` 中独立抽取为 `ti-render` crate，做到**零游戏语义、可复用到其他游戏**。`ti-app` 保留游戏业务渲染（场景/状态机），依赖 `ti-render` 做绘制。

**权威文档**：`docs/process/08_render_arch.md`（渲染管线契约）、`docs/business/08_layout_design.md`（版面/配色/字体单一事实源）。本文只定义 **crate 边界 + 数据 schema + 迁移映射**，不重复架构结论。

## 1. Crate 边界

| Crate | 职责 | 禁止 |
|---|---|---|
| `ti-render` | 帧缓冲/3 绘制原语/z-order 层枚举/字体系统/主题类型/等距投影/文本排版 | 禁止依赖 `ti-app`/`ti-core`/`ti-platform`；禁止出现游戏语义（场景名/角色/战斗等） |
| `ti-app` | 游戏业务渲染（scenes/*.rs）/GameApp 状态机/游戏 UI 组件 | 禁止直接操作帧缓冲像素；必须通过 `ti-render` 绘制 |
| `ti-platform` | 平台抽象（窗口/输入/文件系统） | 不动 |

## 2. ti-render 模块结构

```
crates/ti-render/
  src/
    lib.rs          // crate 根，re-export
    framebuffer.rs  // 帧缓冲抽象（RGB565/RGBA8），3 绘制原语
    zorder.rs       // Z-order 层枚举（BgBoard < Bg < Chrome < Entity < Text < Overlay）
    font/
      mod.rs        // Font trait + FontSet（TTF + bitmap fallback）
      ttf.rs        // TTF 光栅化（ab_glyph）
      bitmap.rs     // 位图字体回退（GoldLand.Fnt + font.dat）
    theme.rs        // 主题类型：色板 + 字号 + 间距（数据驱动）
    iso.rs          // 等距投影（iso_cell/iso_tile，纯数学）
    text.rs         // 文本排版（width/height/centering/wrapping）
    primitives.rs   // 绘制原语（fill_rect/gradient_row/vignette）
    shell.rs        // 场景外壳（wood_frame/title_bar/menu_item/hint_row）
```

## 3. 数据 Schema（Theme）

```rust
// ti-render/src/theme.rs
pub struct Theme {
    pub colors: ColorPalette,      // 色板（数据驱动，不硬编码）
    pub fonts: FontConfig,        // 字号配置（title/body/hint 三档）
    pub layout: LayoutConfig,     // 间距/边距/行高
}

pub struct ColorPalette {
    pub parch_dark: u16,      // 场景底色
    pub parch_light: u16,     // 背景顶部层次
    pub wood: u16,            // 整屏外框
    pub gold: u16,            // 高光点缀
    pub gold_glow: u16,       // 选中项内高光
    pub gold_bright: u16,     // 选中项底
    pub menu_bg: u16,         // 菜单卡片面
    pub menu_border: u16,     // 卡片细边
    pub bevel_hi: u16,        // 卡片顶部/左侧
    pub bevel_shadow: u16,    // 卡片底部/右侧
    pub hilite: u16,          // 血条/进度条槽底
    pub secondary: u16,       // 次级文字
    pub text_on_parch: u16,   // 羊皮纸浅块上的深墨字
    pub text_on_wood: u16,    // 标题条文字
    pub player: u16,          // 玩家青绿
    pub danger: u16,          // BOSS 血红
    pub landmark: u16,        // 据点标记
}

pub struct FontConfig {
    pub title_size: i32,      // 标题字号（96）
    pub body_size: i32,       // 正文字号（44）
    pub hint_size: i32,       // 提示字号（28）
    pub bitmap_advance_title: i32,  // 位图回退 advance（26）
    pub bitmap_advance_body: i32,   // 位图回退 advance（12）
    pub bitmap_advance_hint: i32,   // 位图回退 advance（8）
}
```

## 4. 迁移映射

| 从（ti-app） | 到（ti-render） | 备注 |
|---|---|---|
| `menu_shell.rs` 帧缓冲 + 文字绘制 | `framebuffer.rs` + `text.rs` | Framebuf 抽象不变 |
| `glyph_raster.rs` TTF + 位图 | `font/ttf.rs` + `font/bitmap.rs` | Font trait 统一 |
| `chrome.rs` 绘制原语 + z-order | `primitives.rs` + `zorder.rs` | 3 原语不变 |
| `chrome.rs` iso_cell/iso_tile | `iso.rs` | 纯数学，零游戏语义 |
| `chrome.rs` scene_shell/wood_frame/title_bar/menu_item/hint_row | `shell.rs` | 场景外壳组件 |
| `chrome.rs` 颜色常量 | `theme.rs` ColorPalette | 数据驱动，不硬编码 |
| `menu_shell.rs` 字号常量 | `theme.rs` FontConfig | 数据驱动 |
| `scenes.json` 版面坐标 | ti-render 的 Layout 类型 | 数据实例 |

**留在 ti-app 的**：
- `scenes/*.rs`（游戏业务渲染：index/main_map/vs/saveload/...）
- `game_app.rs`（游戏状态机）
- `forward_nav.rs`（游戏导航）
- `touch.rs`（游戏触摸命中，调用 ti-render 的 iso 投影）
- `shell_ui.rs`（游戏 UI 组件：菜单标签/状态文本）

## 5. 数据分离原则

1. **色板**：`scenes.json` 各场景段加 `theme` 键（或全局 `theme.json`），`ti-render` 从数据加载，不硬编码
2. **字号**：`scenes.json` 各场景段加 `font` 键（或全局 `font.json`），`ti-render` 从数据加载
3. **icon 类型**：`scenes.json` 各场景段加 `icon` 键（类型 + 坐标），`ti-render` 的 `menu_icon` 按类型绘制
4. **间距/行高**：已在 `scenes.json` 的 `list`/`extra` 键中，`ti-render` 从数据读取
5. **validate_data.py** 加断言：theme/font/icon 键存在且值为正整数
6. **doccheck2** 加断言：scenes.json 的 theme/font/icon 键与 ti-render 的 Theme/FontConfig 类型一致

## 6. 复用性保证

- `ti-render` 不依赖 `ti-app`/`ti-core`/`ti-platform`
- `ti-render` 不出现游戏语义（场景名/角色/战斗/存档等）
- `ti-render` 的 API 是通用的（draw_text/draw_rect/draw_iso 等），不是游戏专用的
- 其他游戏可 `cargo add ti-render` 直接使用，只需提供自己的 Theme/Layout 数据
