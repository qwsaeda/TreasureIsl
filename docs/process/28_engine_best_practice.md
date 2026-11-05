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
| 颜色 → `ti_render::Theme` 令牌 | 同上 | ✅ S38 已迁移（§3；加新令牌保视觉 + `theme_color` 编译期派生） |

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
| **F-04a** | 🔴 高 | 位图烘焙颜色字面量——`bake_damage_texture` 描边 `0x20/0x12/0x08` + 芯 `0xFF/0xE8/0xC0`（RGBA8 字节） | `vs_duel.rs` | ✅ 走 `palette::DUEL_DMG_OUTLINE_RGB`/`DUEL_DMG_CORE_RGB`（值不变零视觉变化） |
| **F-04b** | 🟠 中 | 动画/时序魔法数字——伤害上飘 `k*120.0`、锚点 `60.0/0.55`、`ShakeOffset{14.0,160}`、delta `unwrap_or(16)`（vs_duel + treasure_bevy 重复） | `vs_duel.rs` · `bin/treasure_bevy.rs` | ✅ 走 `palette::DUEL_POPUP_RISE_PX`/`DUEL_HIT_ANCHOR_*`/`DUEL_SHAKE_*`/`FRAME_DELTA_FALLBACK_MS`（值不变零视觉变化） |
| **F-04c** | 🟡 低 | 时序常量分散——`STRIKE_REACH`/`STRIKE_MS`/`HIT_STOP_MS`/`POPUP_MS` 在 vs_duel 模块顶部 | `vs_duel.rs` | ✅ 集中 `palette::DUEL_STRIKE_*`/`DUEL_HIT_STOP_MS`/`DUEL_POPUP_MS` |
| **F-05** | 🟡 低 | VS 作废 HP 回合制原型残留——`player_hp=2000`/三动作/4 道具/HP 条，业务已迁兵力模型（`combat.md` v2；S34.6 D5 `rules::force::resolve`），vs_duel 仅 `pub mod` 导出无 live 调用 | `vs_duel.rs` | ✅ 已定案（`combat.md` §S34.6 注）——作废业务残留，**保留作动效骨架参考**（冲刺/hit-stop/上飘/震动可复用到兵力模型自动动效），不删除；硬编码已集中 `palette`（F-04） |

**已验证符合规范（非硬编码）**：场景/地图/平衡/文本数据驱动（px 坐标字面量 0）；`DefaultPlugins`；`bevy_picking`；方向感知（S38 wasm 修复）；`include_str!` 内嵌 = S31 定案（内嵌默认 + 注入解耦，合理）。

## 3. `ui.rs::palette` 单源 + Theme 令牌（F-01/F-02 处置 + S38 Theme 迁移）

**载体** = `crates/ti-engine/src/ui.rs` 顶部 `pub mod palette`：

- **颜色常量**（`Color`，S38 起 = `theme_color(Theme::DEFAULT.<token>, alpha)` 编译期派生）：`PANEL_BG` / `GOLD_TEXT` / `BRASS_SELECT` / `TRANSPARENT` / `CREAM_TEXT` / `MUTED_GOLD` / `DARK_BROWN_TEXT` / `PANEL_WOOD_BG` / `PARCH_BRIGHT` / `PARCH_CREAM` / `DARK_BROWN_06` / `DUEL_SLOT_BG` / `DUEL_BTN_BG` / `DUEL_ROW_BG` / `DUEL_ROUND_BADGE` / `DUEL_POPUP_FLASH` / `DEBUG_RED_BROWN`。
- **尺寸常量**（`f32`，值 = 原字面量）：`PANEL_PAD` / `PANEL_RADIUS` / `PANEL_ROW_GAP` / `PANEL_ROW_GAP_SMALL` / `TITLE_FONT` / `BODY_FONT` / `HINT_FONT` / `LIST_TITLE_FONT` / `BUTTON_FONT` / `ROW_COLUMN_GAP` / `ROW_PAD` / `ROW_RADIUS` / `THUMB_W` / `THUMB_H` / `BUTTON_W` / `BUTTON_H` / `BUTTON_RADIUS`。

**引用约定**：
- `ui.rs`（同模块）：`palette::PANEL_PAD`。
- `vs_duel.rs` / `bevy_app.rs` / `scene_render.rs`（lib 子模块）：`crate::ui::palette::X`。
- `bin/treasure_bevy.rs`（独立 bin crate）：`ti_engine::ui::palette::X`。

**S38 Theme 迁移（palette 颜色 → `ti_render::Theme` 令牌，保视觉）**：

- **为什么加新令牌**：现有 Theme 令牌与 palette 色**语义不匹配**（`menu_bg`=蓝 / `text_on_wood`=粉 / `secondary`=绿 / `brass_light`=黄），naive 值交换会把棕面板变蓝、金文本变粉 → 劣化视觉。故在 `Theme` struct + `Theme::DEFAULT` + `theme.json` + `PaletteJson` 加 **16 个 `ui_*` 令牌**（值 = palette 现值 RGB565 量化，最大误差 0.0155 < 1.6%，零视觉变化近似；用户已确认方向「加新令牌保视觉」，A 纪律对照图 = `theme_migration_swatches.png`）。
- **`theme_color` 编译期派生**：palette 内 `const fn theme_color(token: u16, alpha: f32) -> Color`（RGB565→srgb 数学 + alpha，const 可行——`Color::srgba` 是 const fn，数学无非 const 调用），palette 颜色常量 = `theme_color(Theme::DEFAULT.<token>, <alpha>)` → **编译期真派生**，无值镜像、无 gate 漂移。alpha 因 RGB565 无 alpha 通道单独存于 palette 常量。`theme_color` 数学与 `canvas.rs::rgb565_to_color` 一致。
- **单源**：`theme.json` / `Theme::DEFAULT` = 唯一值源；`ui.rs::palette` 编译期从 `Theme::DEFAULT` 读（不重复存值）。`by_name` 可查 `ui_*` 令牌名（scenes.json `color` 字段可引用）。
- **撞色登记**：`ui_dark_brown_06`==`ui_dark_brown_text`（0x28C2）、`panel_wood`==`ui_duel_btn_bg`==`ui_panel_wood_bg`（0x3943）为有意别名，登记 `gate_data.json::palette_aliases`（飞轮 R1 ①-12）。

**尺寸仍为 palette 常量**（`f32`，不迁 Theme——Theme 字号是 `i32` 逻辑像素、非 bevy_ui 布局 px 语义，迁移会引入单位混淆；保持 palette 单源）。

## 4. 固化守门（doccheck2）

| 检查 | 守什么 |
|---|---|
| **`engine-no-color-literal`**（第 41 项） | ① 引擎层 `Color::srgb/srgba` 字面量只允许出现在 `ui.rs::palette` 常量定义（S38 起 = `theme_color(Theme::DEFAULT.<token>, alpha)` 编译期派生；`theme_color` helper 函数体 `Color::srgba(r,g,b,alpha)` 属合法转换）与 `canvas.rs::rgb565_to_color`（合法 RGB565 转换）；其余出现 = 违背（F-01 防再犯）。② 尺寸裸数字 `Px(<数字>`（`FontSize::Px`/`BorderRadius::px`/`UiRect::px`/`Val::Px`）在 `ui.rs` + `bin/treasure_bevy.rs` = 0，`vs_duel.rs` 精灵尺寸（`Vec2::new`/`custom_size`）裸数字 = 0（F-02/F-02b 防再犯）。③ 动画/时序裸数字：`unwrap_or(16)` 在 `vs_duel.rs` + `bin/treasure_bevy.rs` = 0（`FRAME_DELTA_FALLBACK_MS`）；`vs_duel.rs` `ShakeOffset amplitude: <数字>` 与 `k * <数字>`（上飘）裸数字 = 0（F-04 防再犯）。 |

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
