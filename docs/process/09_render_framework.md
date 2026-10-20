# 渲染框架独立契约（ti-render）

**定位**：把渲染框架从 `ti-app` 中独立抽取为 `ti-render` crate，做到**零游戏语义、可复用到其他游戏**。`ti-app` 保留游戏业务渲染（场景/状态机），依赖 `ti-render` 做绘制。

**权威文档**：`docs/process/08_render_arch.md`（渲染管线契约）、`docs/business/08_layout_design.md`（版面/配色/字体单一事实源）。本文只定义 **crate 边界 + 数据 schema + 迁移映射**，不重复架构结论。

## 1. Crate 边界

| Crate | 职责 | 禁止 |
|---|---|---|
| `ti-render` | 帧缓冲 / 绘制原语 / 字体系统 / 主题类型 / 等距投影 / 文本排版 / 场景外壳组件 | 禁止依赖 `ti-app`/`ti-core`/`ti-platform`/`ti-formats`（S15.10 起连资产载体格式也不依赖）；禁止出现游戏语义（场景名/角色/战斗等）。**依赖实测**：仅 `ab_glyph`/`serde`/`serde_json`，由 doccheck2 第 23 项 `render-framework-boundary` 锁定 |
| `ti-app` | 游戏业务渲染（scenes/*.rs）/GameApp 状态机/游戏 UI 组件 | 禁止直接操作帧缓冲像素（须经 `Canvas` trait / `Framebuf` 方法，S15.10）；必须通过 `ti-render` 绘制；**持有格式层 → 框架层的转换点**（如 `menu_shell::parse_font_bytes` 把 `ti_formats::font` 解析结果转成框架中性 `ti_render::font::FontSet`） |
| `ti-platform` | 平台抽象（窗口/输入/文件系统）+ **呈现层**（脏行跟踪 + 纹理上传，帧缓冲本体不在此，S15.10） | 不重新定义帧缓冲类型（会与 `ti-render` 重复） |

## 2. ti-render 模块结构

```
crates/ti-render/
  src/
    lib.rs          // crate 根，re-export
    canvas.rs       // Canvas trait：渲染框架 ↔ 宿主平台的最小抽象（帧缓冲 + 3 档文字）
    framebuffer.rs  // 帧缓冲抽象（RGB565/RGBA8）、1080 空间常量、PixelSink
    font/
      mod.rs        // FontSet/Glyph 中性类型 + TtfFont 转发（S15.10：不认 ti-formats，宿主转换）
      ttf.rs        // TTF 光栅化（ab_glyph）
      bitmap.rs     // 位图字体回退（中性 FontSet → Raster；宿主在转换层喂字形）
    theme.rs        // 主题类型：色板 + 字号 + 间距（数据驱动）
    iso.rs          // 等距投影（IsoProjection，纯数学）
    text.rs         // 文本排版（center_x/center_y/wrap_text/multi_line_metrics）
    primitives.rs   // 组合函数 parchment_bg（gradient_row/vignette 为私有辅助；绘制原语 = Canvas trait 方法，S15.10）
    shell.rs        // 场景外壳（wood_frame/title_bar/menu_item/hint_row/panel/bar_slot/…）
```

> **本 crate 不含 z-order 层枚举**。曾有 `zorder.rs` 的 `ZOrder` 枚举
> （`BgBoard < BgGrid < Frame < TitleBar < Content < Text < BarSlot < Highlight`），
> 但**零消费方**——运行时的层序实际由 `ti-app` 的调用顺序决定
> （`scenes::chrome::scene_shell` 的调用序），该死枚举已删除。
> z-order 契约见 [`08_render_arch.md`](08_render_arch.md) §4。

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
| `menu_shell.rs` 帧缓冲 + 文字绘制 | `framebuffer.rs` + `text.rs` + `canvas.rs` | `Canvas` trait 是宿主实现面的最小抽象（EXP-094） |
| `glyph_raster.rs` TTF + 位图 | `font/ttf.rs` + `font/bitmap.rs` | Font trait 统一 |
| `chrome.rs` 绘制原语 | `primitives.rs` | 组合函数（`parchment_bg`）也算在原语层 |
| `chrome.rs` iso_cell/iso_tile | `iso.rs` | 纯数学，零游戏语义（`chrome` 的本地副本尚未迁，见 §4 注） |
| `chrome.rs` scene_shell/wood_frame/title_bar/menu_item/hint_row | `shell.rs` | 场景外壳组件；`chrome` 侧目前只是 `pub use` 转发 |
| `chrome.rs` 颜色常量 | `theme.rs` ColorPalette | 数据驱动，不硬编码 |
| `menu_shell.rs` 字号常量 | `theme.rs` FontConfig | 数据驱动 |
| `scenes.json` 版面坐标 | ti-render 的 Layout 类型 | 数据实例 |

> **迁移尚未完成的部分**（勿当已完成）：`chrome.rs` 的 `scene_shell`/`wood_frame`/
> `title_bar`/… 仍是转发层而非搬迁；`chrome.rs` 与 `touch.rs` 各保留一份 `iso_cell`/
> `iso_grid_cell_at`，与 `ti-render::iso::IsoProjection` 三份并存。

**留在 ti-app 的**：
- `scenes/*.rs`（游戏业务渲染：index/main_map/vs/saveload/...）
- `game_app.rs`（游戏状态机）
- `forward_nav.rs`（游戏导航）
- `touch.rs`（游戏触摸命中，调用 ti-render 的 iso 投影）
- `shell_ui.rs`（游戏 UI 组件：菜单标签/状态文本）

### 5.1 原语参数语义（`params` / `rect` 约定）

| kind | 关键参数 | 语义（双后端同构） |
|---|---|---|
| `rect` | `rect` | 填充矩形（左上 + 宽高） |
| `text` | `rect` + `tier` + `text`/`text_key` | `rect` **内水平居中**；`tier` = `title`(96px) / `body`(44px) / `hint`(28px)；`text_key` 走 i18n（缺键回退显示 key 本身 → 落库前须核对 `strings.json`） |
| `icon` | `rect` + `params.icon` | `params.icon` = `ui_icons.json` 图标名；**图标按注册表原生 48px 绘制在 `rect` 左上角**，`rect` 的边长仅表示占位盒（CPU 执行器忽略 `params.size`；Bevy 执行器按 `rect` 边长烘焙纹理，字形仍为原生几何）→ 改图标大小须改 `rect` 边长，不改 `params.size` |
| `grad` | `params.from_token`/`to_token`/`steps` | 垂直渐变（`steps` 条色带，2..256）；令牌名须在 `theme.by_name` 名单（拼错 → 静默回退 `text_on_parch`） |
| `glow` | `rect` = **圆心 + 半径** | 径向光晕（`rect.x`/`rect.y` = 圆心，`rect.w` = 半径） |
| `disc` | `rect` = **圆心 + 半径** | 实心圆盘（同上） |

不变式：`color` 必须是 `theme.by_name` 合法令牌名；`params.icon` 必须在 `ui_icons.json` 注册表内；
`text_key` 必须在 `strings.json` 内（缺键 = 界面显示裸 key，非静默失败）。

## 5. 数据分离原则

1. **色板**：**权威 = Rust 默认值**（`ti_render::theme::Theme::default()`）。
   `assets/data/theme.json` 是它的**镜像**，由 `crates/ti-render/tests/theme_json.rs`
   的 `repo_theme_json_matches_default` 断言两者**逐字段恒等**（改 JSON 不改 Rust ⇒ 测试红）。
   加载路径 `Theme::from_json_str` 已实现，运行时当前走 `Theme::default()`。
   ⚠️ 改动顺序：先改 Rust 默认值 → 再同步 `theme.json` → 跑 `cargo test -p ti-render`。
2. **字号**：同色板，随 `Theme::fonts` 三档（title/body/hint + 位图 advance）走 Rust 默认值，
   JSON 镜像在同一个 `theme.json` 的 `fonts` 段。**`assets/data/` 下没有 `font.json`**
   （历史设想未落地），不要再写「字体来自 `scenes.json`/`font.json`」。
3. **icon 类型**：`scenes.json` 各场景段带 `icon {x}`（仅坐标），类型判据仍在代码；
   `ti-render::shell::menu_icon` 按 `kind` 绘制。
4. **间距/行高**：在 `scenes.json` 的 `list`/`extra_scalar` 键中，消费方在 `ti-app::scenes`。
5. **validate_data.py**：校验 `scenes.json` 标量正整数（不校验 theme 键——theme 是 Rust 权威）。
6. **doccheck2**：断言 `scenes.json` 版面键与 `08_layout_design.md` §4 表一致
   （`check_layout_doc_table_cross_check`）；色板恒等由 `ti-render` 测试守。

## 6. 复用性保证

- `ti-render` 不依赖 `ti-app`/`ti-core`/`ti-platform`
- `ti-render` 不出现游戏语义（场景名/角色/战斗/存档等）
- `ti-render` 的 API 是通用的（draw_text/draw_rect/draw_iso 等），不是游戏专用的
- 其他游戏可 `cargo add ti-render` 直接使用，只需提供自己的 Theme/Layout 数据
