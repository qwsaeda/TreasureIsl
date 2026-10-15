# 接线手册（Gamebook）

**定位**：业务规则 → 代码落点的**对照手册**。
业务规则唯一事实源 = [`docs/business/`](business/00_index.md)（+ `assets/data/*.json`）；
本文件只回答「这条规则落在哪个模块、哪个文件、哪条测试」，**不复制规则本身**。

> 面向两类读者：新会话/子代理查落点；改完接线后自检是否漏改。

## 0. 三层结构

```
业务文档 (docs/business/*.md)   ← 规则（为什么这么设计）
   ↓ 实现
ti-core::rules/*  纯逻辑、零 IO、数值全部来自 assets/data/*.json 注入
   ↓ 表现
ti-app::scenes/*  每场景一个 struct：on_input / on_touch / render
   ↓ 呈现
ti-app::menu_shell + ti-platform::gfx  帧缓冲 / 视口 / 缩放 / 上屏
```

**依赖方向铁律**：`ti-core` 不依赖 `ti-platform`/`ti-app`；`ti-platform` 不依赖 `ti-app`；
场景 struct **不持有** `GameApp`，只拿 `&mut dyn SceneHost`（见 [`docs/process/01_architecture.md`](process/01_architecture.md) §2）。

## 1. 业务 → 代码落点

| 业务文档 | rules 模块 | 表现层 | 关键测试 |
|---|---|---|---|
| [`loop_core.md`](business/loop_core.md) | `rules/main_loop.rs`、`domain/flow.rs` | `scenes/mod.rs` 路由 | `tests/reference_diff_main_loop.rs` |
| [`economy.md`](business/economy.md) | `rules/trade.rs`、`rules/build.rs`、`domain/economy.rs` | `scenes/main_map.rs` | `tests/rules_trade.rs`、`tests/rules_build.rs` |
| [`combat.md`](business/combat.md) | `rules/battle.rs`、`domain/combat.rs` | `scenes/vs.rs` | `tests/rules_battle.rs` |
| [`progression.md`](business/progression.md) | `rules/main_map.rs`、`rules/cards.rs`、`rules/cards_draw.rs`、`domain/map.rs` | `scenes/main_map.rs` | `tests/reference_diff_cards.rs` |
| [`persistence.md`](business/persistence.md) | `rules/saveload.rs`、`rules/saveload_json.rs`、`rules/rng.rs`、`domain/persistence.rs` | `scenes/saveload.rs` | `tests/rules_saveload*.rs`、`tests/save_migration.rs` |
| [`07_ui_flow.md`](business/07_ui_flow.md) | —（表现层规范） | 全部 `scenes/*.rs`、`touch.rs`、`widgets.rs` | `tests/game_app_flow.rs`、`tests/e2e_playthrough.rs` |

## 2. 数据源（数值/文本/布局/控件）

| 表 | 内容 | 生成 | 加载 |
|---|---|---|---|
| `assets/data/balance.json` | 全部玩法数值 | 手写（策划） | `ti_core::data::balance()`（`OnceLock`，解析失败 = 硬错误，不 fallback） |
| `assets/data/strings.json` | 全部玩家可见中文 | 手写 | `ti_core::data::strings()` / `ti_app::i18n::sget` |
| `assets/data/scenes.json` | 场景布局（标题/列表/提示行坐标） | 手写 | `ti_core::data::scenes()` |
| `assets/data/controls.json` | 控件记录（20 场景 / 2041 条） | `legacy/scripts/gen_layout.py`（幂等，gate 第 10b 项） | `ti_core::data::controls()` → `ti_app::widgets::WidgetLayer` |

**硬规则**：代码里不写中文面量、不写数值、不写控件坐标。违反即 `check_hardcode` / `check_module_contracts` 门禁 FAIL。

## 3. 场景接线

- 场景枚举与合法转换：`scenes/mod.rs`（`SceneKind` + `scene_transition!` 宏；转换表业务事实源 = [`07_ui_flow.md`](business/07_ui_flow.md) §1）；
- 每场景一个 struct：`IndexScene` / `SelectRoleScene` / `SelectMapScene` / `MainMapScene` /
  `VsScene` / `MessageScene` / `SaveloadScene`（`Intro` 与 `Exit` 无独立资源，由 `Index`/清屏处理）；
- 组合根：`game_app.rs::GameApp` 实现 `SceneHost`，方法全是**一行状态读写**；业务判定在 `scenes/*.rs`；
- 渲染键：`SceneKind::asset_name()`（场景 `.Map` 精灵资源名），判别值 `discriminant()` 用于静态帧优化。

## 4. 输入接线

```
宏（native/wasm/android）读按键/指针
   ├─ KeyState  → GameApp::poll(&k)          → GameInput → Scene::on_input
   └─ TouchSample（逻辑坐标）→ GameApp::poll_touch → Scene::on_touch
```

- 按键沿：壳层把 held 键转成单次沿（`key_edge`），避免每帧连跳；
- 鼠标左键**不**映射 `confirm`：触屏/鼠标走元素命中判定（点菜单行/地图格），避免双触发；
- 触摸坐标：壳层用 `Viewport::screen_to_logical` 反变换后交给 `TouchPad::feed_screen`；
  主地图点按走等距逆变换（与渲染同坐标系，[`07_ui_flow.md`](business/07_ui_flow.md) §3.2 硬规则 2）。

## 5. 渲染接线

- `GameApp::render()`：先场景模块绘制（`Scene::render`），再叠加控件层与 `.Map` 真实精灵；
- 双路径：`render()`（RGB565 帧缓冲）/ `render_hd()`（RGBA8 + HD PNG 注入），按 `settings.quality` 切换；
  对应上屏 `present()` / `present_hd()`；
- 精灵源：`ui.pak`（`set_atlas`）+ 场景 `.Map`（`add_scene`）；地形：`glmap_m01.pak` + `glmapm01.Map`（`set_terrain`）；
- HD 目录 `assets_hd_pack/1x/` 递归注入（`load_hd_pack_from_dir`），缺失即回退 PCX，**不 panic**；
- 静态帧优化：`last_rendered`（选中项）+ `last_rendered_scene`（场景判别值）双键，都不变则跳过重绘。

## 6. 音频接线

- 源 `music.lpk` → `ti_app::audio_prep::prepare_music`（解包 + 混音 + MOD→OGG 转码）→ 宏侧加载；
- BGM 优先 OGG 曲目（`bundle.ogg_tracks`），否则取最长 WAV 混音；SE 取最短 WAV；
- 触发：玩法层置 `last_sfx`，壳层每帧消费后清空（避免连播）；
- 失败策略：曲目数归 0 并在状态行可见，**不在启动路径 panic**。

## 7. 三端入口差异

| 端 | bin | 资源来源 | 备注 |
|---|---|---|---|
| 桌面 | `treasure-native` | `std::fs::read(legacy/data/...)` | F1 缩放 / F2 滤波 / F3 边框 / F4 画质 |
| 浏览器 | `treasure-wasm` | `macroquad::file::load_file("assets/...")` | `scripts/build_wasm.ps1` 产 `web/` |
| Android | `treasure-android` | APK `assets/`（macroquad android target 自动映射） | 编译在 Mac，见 `android/README.md` |
| 无头 | `treasure-headless` | 无（不加载资源） | 跑 `tests/scripts/full_run.json` 回归 |

## 8. 全流程回归（P3-3）

`tests/scripts/full_run.json` = 固定种子（seed=42）的 16 步断言：

| # | 动作 | 场景 | 期望场景 |
|---|---|---|---|
| 1 | 确认菜单项 0 | `Index` | `SelectRole` |
| 2 | 下一项 | `SelectRole` | `SelectRole` |
| 3 | 确认角色 | `SelectRole` | `SelectMap` |
| 4 | 下一项 | `SelectMap` | `SelectMap` |
| 5 | 确认地图 | `SelectMap` | `MainMap` |
| 6 | 南走一步 | `MainMap` | `MainMap` |
| 7/8 | 买入/卖出商品 1 | `MainMap` | `MainMap` |
| 9 | 建造 Castle | `MainMap` | `MainMap` |
| 10 | 据点事件码 6 | `MainMap` | `Vs` |
| 11 | 攻击 ×200 | `Vs` | `Vs` |
| 12 | 退出战斗 | `Vs` | `Index` |
| 13/14 | 下移 + 确认 | `Index` | `Saveload` |
| 15 | 写档 slot 0 | `Saveload` | `Saveload` |
| 16 | 读档 slot 0 | `Saveload` | `MainMap` |

命令：

```
cargo run -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json
```

## 9. 改动检查清单

| 我改了什么 | 必须同步 | 验证命令 |
|---|---|---|
| 玩法规则（`business/*.md`） | `rules/*` + 对应单测 + 本文件 §1 表 | `cargo test -p ti-core` |
| 数值 | `assets/data/balance.json`（不改代码） | `python scripts/validate_data.py` |
| 玩家可见文本 | `assets/data/strings.json`（不改代码） | `python scripts/check_hardcode.py` |
| 场景流/输入/控件 | 本文件 §3–§5 + [`07_ui_flow.md`](business/07_ui_flow.md) + `scenes/*.rs` | `cargo test -p ti-app` |
| 场景转换表 | **两处同改**（本文件/业务文档 ↔ `scene_transition!` 宏） | `cargo test -p ti-app`（含 `compile_fail` 文档测试） |
| 全局门禁 | 任何一项 | `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` |