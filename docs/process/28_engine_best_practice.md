# 28 游戏开发 + Bevy 最佳实践（规范 / 审计基准 · S38）

> 单一事实源：**硬编码禁忌 + 引擎使用规范 + 审计基准**（用户 2026-11-04「发现你用了游戏引擎，但很多还是硬编码，不是遵守游戏开发/引擎使用的最佳规范实践」→ 先总结规范 → 审计项目 → 处置）。
> 相邻：`12_bevy_usage.md`（Bevy 用法规范）/ `16_code_standards.md`（CS-1~4 代码标准）/ `18_asset_content_pipeline.md`（数据驱动）/ `17_layout_engine.md`（B1 意图化）。

## 1. 规范清单（审计对照基准）

### A. 数据驱动（内容/数据不进代码）
| 规范 | 单一事实源 | 禁忌 | 现状 |
|---|---|---|---|
| 场景/UI 坐标 | `scenes.json`（1080 原语） | 代码写死坐标 | ✅ 已达成（px 坐标字面量 0） |
| 地图/格环几何 | `main_map_v6_layout.json` | 代码写死格环 | ✅ 已达成（S34.6 参数化） |
| 平衡/规则数值 | `balance.json` | 代码写死数值 | ✅ |
| 文本/i18n | `strings.json` | 代码写死文案 | ✅ |
| 资产引用 | `ui_manifest.json` / `registry.json` | 代码写死路径 | ✅（chrome.rs `VS_*` 已单源） |

### B. 视觉令牌（Theme / 8pt）—— ⚠️ S38 前主要违背面
| 规范 | 禁忌 | 处置 |
|---|---|---|
| 颜色 → 具名常量单源（`ui.rs::palette`） | `Color::srgb()/srgba()` 字面量 | ✅ S38 已固化（F-01） |
| 尺寸 → `palette` 常量 | `FontSize::Px`/`BorderRadius::px`/`UiRect::px`/`Val::Px` 裸数字 | ✅ S38 已固化（F-02） |
| （后续）颜色/尺寸 → `ti_render::Theme` 令牌 | 同上 | ⏳ 待用户确认设计稿（见 §3） |

### C. 引擎机制（Bevy 原生，不手搓）
| 规范 | 禁忌 | 现状 |
|---|---|---|
| `DefaultPlugins`（禁手搓插件表） | 手搓插件表 | ✅ |
| 方向感知窗口（禁硬编码窗口尺寸） | `WindowResolution(WIN_W,WIN_H)` 竖屏硬编码 | ✅（S38 wasm 修复 EXP-164 + doccheck2 `wasm-viewport-init`） |
| `bevy_picking` 命中（禁手写几何） | 手写命中 | ✅（S29.2） |
| `bevy_ui` 意图化（flex/%/gap） | 写死 px | ✅（F-02 后） |

### D. 代码标准 / 架构
CS-1 资产单源 · CS-2 文件职责 · CS-3 参数禁单字母 · CS-4 场景层窄接缝 · 状态机（Scene enum 非裸 bool）· 规则层（`ti-core::rules`）与表现层分离 · 固定时间步 · LCG 种子封。

## 2. 审计 findings（S38 首轮对照）

| ID | 严重度 | 违背 | 位置 | 处置 |
|---|---|---|---|---|
| **F-01** | 🔴 高 | 颜色未走令牌单源（CS-1）——`Color::srgb()/srgba()` 字面量 ~24 处 | `ti-engine/src/ui.rs`(10) · `bin/treasure_bevy.rs`(7) · `vs_duel.rs`(5) · `scene_render.rs`(1) · `bevy_app.rs`(1) | ✅ 集中 `ui.rs::palette`（值不变零视觉变化） |
| **F-02** | 🟠 中 | 布局尺寸未走令牌/意图化——字体/圆角/内边/gap/宽高 px 字面量 18 处 | `ti-engine/src/ui.rs` | ✅ 集中 `ui.rs::palette` |
| **F-02b** | 🟠 中 | 尺寸字面量延伸——消息面板/索引环标签/页点/VS 精灵尺寸 16 处 | `bin/treasure_bevy.rs`(13) · `vs_duel.rs`(3) | ✅ 集中 `ui.rs::palette`（F-02 续） |
| **F-03** | 🟡 低 | 资产路径字符串 91 处——`chrome.rs VS_*` 已单源（OK）；`hd_manifest.rs` 生成路径（OK）；其余为数据路径（OK） | `ti-app/src/**` + `ti-engine/src/**` | 无违背（核对） |
| **F-03b** | 🟡 低 | chrome.rs 内联资产路径 27 处（boat_1-4/icon_*/node_*） | `ti-app/src/scenes/chrome.rs` | 无违背——集中注册表 + `check_asset_registry` disk↔ui_manifest 双向同步兜底（全部登记在案） |

**已验证符合规范（非硬编码）**：场景/地图/平衡/文本数据驱动（px 坐标字面量 0）；`DefaultPlugins`；`bevy_picking`；方向感知（S38 wasm 修复）；`include_str!` 内嵌 = S31 定案（内嵌默认 + 注入解耦，合理）。

## 3. `ui.rs::palette` 单源（F-01/F-02 处置）

**载体** = `crates/ti-engine/src/ui.rs` 顶部 `pub mod palette`：

- **颜色常量**（`Color`，值 = 原字面量）：`PANEL_BG` / `GOLD_TEXT` / `BRASS_SELECT` / `TRANSPARENT` / `CREAM_TEXT` / `MUTED_GOLD` / `DARK_BROWN_TEXT` / `PANEL_WOOD_BG` / `PARCH_BRIGHT` / `PARCH_CREAM` / `DARK_BROWN_06` / `DUEL_SLOT_BG` / `DUEL_BTN_BG` / `DUEL_ROW_BG` / `DUEL_ROUND_BADGE` / `DUEL_POPUP_FLASH` / `DEBUG_RED_BROWN`。
- **尺寸常量**（`f32`，值 = 原字面量）：`PANEL_PAD` / `PANEL_RADIUS` / `PANEL_ROW_GAP` / `PANEL_ROW_GAP_SMALL` / `TITLE_FONT` / `BODY_FONT` / `HINT_FONT` / `LIST_TITLE_FONT` / `BUTTON_FONT` / `ROW_COLUMN_GAP` / `ROW_PAD` / `ROW_RADIUS` / `THUMB_W` / `THUMB_H` / `BUTTON_W` / `BUTTON_H` / `BUTTON_RADIUS`。

**引用约定**：
- `ui.rs`（同模块）：`palette::PANEL_PAD`。
- `vs_duel.rs` / `bevy_app.rs` / `scene_render.rs`（lib 子模块）：`crate::ui::palette::X`。
- `bin/treasure_bevy.rs`（独立 bin crate）：`ti_engine::ui::palette::X`。

**为什么不直接走 `ti_render::Theme`（RGB565）**：Theme 值与现值有明显偏差（如 `0.72,0.45,0.16` 黄铜高亮 vs `gold 0.581,0.429,0.161`），严格走 Theme 会**改变视觉** → 触发 A 纪律设计稿（硬规则 9）。故先做**常量单源**（消除硬编码、零视觉变化）；**Theme 完整迁移列为后续**，需用户确认设计稿后才动（届时 `palette::X` → `rgb565_to_color(theme.xxx)` 映射）。

## 4. 固化守门（doccheck2）

| 检查 | 守什么 |
|---|---|
| **`engine-no-color-literal`**（第 41 项） | ① 引擎层 `Color::srgb/srgba` 字面量只允许出现在 `ui.rs::palette` 常量定义（单源）与 `canvas.rs::rgb565_to_color`（合法 RGB565 转换）；其余出现 = 违背（F-01 防再犯）。② 尺寸裸数字 `Px(<数字>`（`FontSize::Px`/`BorderRadius::px`/`UiRect::px`/`Val::Px`）在 `ui.rs` + `bin/treasure_bevy.rs` = 0，`vs_duel.rs` 精灵尺寸（`Vec2::new`/`custom_size`）裸数字 = 0（F-02/F-02b 防再犯）。 |

## 5. 审计方法（复跑）

```sh
# 颜色/尺寸字面量快扫（F-01/F-02）
grep -rnE "Color::srgb\(|Color::srgba\(" crates/ti-engine/src | grep -v "pub const"
grep -nE "FontSize::Px\([0-9]|BorderRadius::px\([0-9]|UiRect::px\([0-9]|Val::Px\([0-9]" crates/ti-engine/src/ui.rs

# 机器守门（第 41 项）
cargo run -q -p ti-tools --bin doccheck2
```

## 6. 变更面声明（本次 S38）

改动面 = **表现层**（`ti-engine` bevy_ui chrome / VS 原型视觉量）。值不变等价替换（零视觉变化），不触发 A 纪律设计稿；无业务规则/数据改动。
