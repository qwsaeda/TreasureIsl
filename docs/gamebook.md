# 金银岛 TreasureIsl — 可玩实现手册（Gamebook）

本文件是「从文档就能实现出一个可玩游戏」的单一事实源。
资源层（S1）与离线规则层（S3 各模块 diff 全绿）已完成；本手册补齐
**交互壳 + 场景状态机 + 全流程接线 + 验收**，使 `ti-app` 能真正玩起来。

> 关系：`docs/spec/*.md` 是各模块逆向规格（输入/数值/事件码的**证据源**）；
> `crates/ti-core/src/rules/*.rs` 是各模块的**正式实现**（已 diff 验证，可直接调用）；
> 本手册是**接线规格**——把这些已验证的 rules 接进 `ti-app` 表现层，
> 让 12 个玩法模块 + 主菜单拼成一条可从头玩到尾的流程。

## 0. 现状与目标

- **已完成**：12 个玩法模块的 `rules/` 正式实现全部 diff 全绿
  （menu/selection/main_map/trade/build/battle/cards/saveload/messages/render/rng/main_loop）。
- **缺口**：可运行产物只停在**主菜单**——`shell_ui.rs` 的 6 个 `event_id` 是占位值，
  点「新游戏」后没有接进 `ti-core::rules` 的场景状态机，玩法没有进入 UI 流程。
- **目标**：实现 `GameApp`（`ti-app`）状态机，把菜单事件 → 选人 → 选图 →
  主地图移动 → 贸易/建造 → 战斗 → 存读档 全链路接通，可从头玩到尾。

原则（用户既定）：**业务逻辑一律调用 `ti-core::rules` 已验证实现，不重抄数值/公式**。
本手册只定义「壳怎么把 rules 串起来 + 表现层怎么画」，数值/规则以 rules 为准。

## 1. 总体架构

```
                    ┌─────────────────────────────────────────┐
                    │            ti-app（表现+交互壳）          │
                    │  GameApp（场景状态机）                     │
                    │   ├─ 持有 ti-core rules 各 Session       │
                    │   ├─ 每场景：update(输入事件) + render(fb) │
                    │   └─ 跨场景跳转（SceneStack advance）     │
                    └──────────────┬──────────────────────────┘
                                   │ 只调用（不重抄数值）
                    ┌──────────────▼──────────────────────────┐
                    │            ti-core::rules（已验证）       │
                    │  MenuStateMachine / SelectionFlow /      │
                    │  MapSession / TradeLedger / BuildManager /│
                    │  BattleSession / CardEngine / SaveManager /│
                    │  MessageSession / SceneStack / GameRng     │
                    └─────────────────────────────────────────┘
                                   │ 读资源
                    ┌──────────────▼──────────────────────────┐
                    │  ti-formats（容器/字体/PCX/存档/音频解析）  │
                    │  ti-platform（gfx 帧缓冲 / input / audio） │
                    └─────────────────────────────────────────┘
```

- `GameApp` 取代 `MenuShell` 成为两壳（native/wasm）共用的交互核心；
  `MenuShell` 退化为 `GameApp` 在 `Index` 场景下的**渲染子视图**（保留其
  帧缓冲/字形/脏行带机制，见 §4b）。
- 两 bin 入口（`treasure_native.rs` / `treasure_wasm.rs`）改为持有 `GameApp`，
  把 `ti_platform::input` 的每帧输入事件喂给 `GameApp::update`，
  `GameApp::render` 画到 `Framebuf`，`FramePresenter` 上屏（机制不变，EXP-045）。

## 2. 场景状态机（GameApp 顶层）

场景枚举（与 `oracle::menu::MenuScene` + `oracle::main_loop::SceneId` 对齐）：

```rust
enum Scene {
    Intro,        // 片头（可选，触发即跳 Index）
    Index,        // 主菜单
    SelectRole,   // 角色选择
    SelectMap,    // 地图选择
    MainMap,      // 主地图（移动/贸易/建造/事件分派）
    Vs,           // 战斗
    Message,      // 消息/帮助/信息（据点/面板/卡片）
    Saveload,     // 存读档（子窗口叠加，非独立场景）
    Exit,
}
```

跳转规则（**直接调用 `rules::main_loop::SceneStack::advance`**，逐位一致）：

| 当前 | 触发 | 目标 | 依据 |
|---|---|---|---|
| `Index` + 菜单项 0（新游戏） | `select_role` 完成 | `SelectRole` | spec/menu.md §c case 0 |
| `SelectRole` + 角色选定 | `select_role` | `SelectMap` | spec/selection.md |
| `SelectMap` + 地图选定（completed） | `main.Map` | `MainMap` | spec/selection.md |
| `MainMap` + 事件码 6 | `dispatch(6)` → `Vs` | `Vs` | spec/main_map.md §d |
| `MainMap` + 事件码 1/4 | `dispatch` → `Msgl` | `Message` | spec/main_map.md §d |
| `MainMap` + 事件码 2/5/7/8 | `dispatch` → `Msgm/Msgs` | `Message` | spec/main_map.md §d |
| `Vs` + 战斗结束（`outcome` 非空） | `advance(2)` → 状态 0 | `Index`（回主菜单） | spec/battle.md §d / main_loop.md case 25→0 |
| `Message` + 关闭 | `close` | 回 `MainMap`（场景源） | spec/messages.md §c |
| 任意 + 返回 | 各 rules 返回路径 | `Index` | 各 spec 返回码 27 |

`GameApp` 持有 `SceneStack`（`rules::main_loop`）做状态码跟踪；
**具体场景对象由 `GameApp` 自身枚举驱动**（`SceneStack` 只保证状态码语义一致，
壳层用 `Scene` 枚举做可玩 UI 路由，两者并行不冲突）。

## 3. 各场景数据流（调用哪些 rules）

> 关键原则：壳层**只编排**，数值/判定全部委托给 `ti-core::rules`。
> 每场景列出「持有的 rules 对象 + 输入事件 → 调用的 rules 方法 → 表现层渲染」。

### 3.1 Index（主菜单）
- 持有：`rules::menu::MenuStateMachine` + 已载入资源（字体/音效计数）。
- 输入：↑/↓/上移选择，Enter/确认。
- 调 `menu.handle(MenuEvent::Click(code))`：
  - `Click(0)` 新游戏 → `sfx = Press`，切 `SelectRole`（初始化 `SelectionFlow`）。
  - `Click(1)` 继续 → `Click(1)` 走 `Saveload` 子窗口（读 `gl.rec`/`gl*.sav`）。
  - `Click(2)` 选项 / `Click(3)/Click(4)` 说明 → `Message`（Help kind）。
  - `Click(5)` 退出 → 置退出标志。
- 渲染：复用 `menu_shell` 现机制（标题+菜单项+选中高亮+状态行）。

### 3.2 SelectRole / SelectMap
- 持有：`rules::selection::SelectionFlow`（`unlocked` 由壳探测 `glmapm01/02/03.Map` 得到）。
- 输入：38 上一个 / 40 下一个（循环），13 确定。
- 调 `flow.select_role(ch)` / `flow.select_map(m)`，返回 `Option<Sfx>`。
  - `completed=true` → 记 `(character, map_slot)`，切 `MainMap`。
- 渲染：角色头像（`Player_*.pcx`）或地图缩略 + 选中高亮 + 名称。

### 3.3 MainMap（核心可玩场景）
- 持有：`rules::main_map::MapSession`（地形/位置/据点）
  + `rules::trade::TradeLedger` + `rules::build::BuildManager`
  + `rules::cards::CardEngine` + `rules::rng::GameRng`。
- 输入：
  - 方向键/拖拽 → `map.move_to(dir)` / `map.walk(dir, steps)`（越界/阻挡返回 `None`）。
  - 据点键 → `if map.at_landmark()` 触发 `map.dispatch(code)`：
    - `Some(Vs)` → 切 `Vs` 场景（建 `BattleSession`）；
    - `Some(Msgl/Msgm/Msgs)` → 切 `Message` 场景（据点/面板/卡片文本）；
    - `None` → 无事件。
  - 贸易键 → 打开贸易面板：`ledger.buy(gid)` / `ledger.sell(gid)`，
    错误 `TradeErr` 弹提示（`Message` Invalid kind）。
  - 建造键 → `build.build(kind, level)`，`BuildErr` 弹提示。
- 渲染：等距网格（`MapTerrain` `grid_w×grid_h`）+ 玩家棋子在 `pos` +
  据点标记 + 金/持有量 HUD（`ledger.gold()` / `ledger.owned(gid)`）。

### 3.4 Vs（战斗）
- 持有：`rules::battle::BattleSession`（`new(seed, player, boss_tier)`，
  `boss_hp` 由 `oracle_boss_hp` 初始化）。
- 输入：「攻击」键 → `session.step()` 单回合；循环直到 `session.outcome()` 非空。
  - 玩家 HP / 敌方 HP / 回合数 / 胜方 渲染为战斗面板。
  - `outcome = Some(Player)` → 播 `victory.wav`，切回 `Index`（`advance(2)`）。
  - `outcome = Some(Enemy)` → 失败面板 → 回 `Index`。
- seed 来源：`GameRng`（全局 LCG，`in_range` 取 BOSS 档 tier 0..=4）。

### 3.5 Message（消息/帮助/据点文本）
- 持有：`rules::messages::MessageSession`。
- 输入：`open(kind)` / `next()` / `prev()` / `close()`（翻页循环，spec/messages.md §c）。
- 渲染：当前页文本（据点事件文本 / 帮助页 / 卡片说明）+ 页码。

### 3.6 Saveload（存读档子窗口）
- 持有：`rules::saveload::SaveManager`（slot 数 5，见 design/constants.md §9.3）。
- 输入：槽位选择 → `read_slot`（读 `gl*.sav`，`validate` 校验魔数）
  或 `write_slot`（写档，`REC_MAGIC` 头）；`migrate` v1→v2。
- 渲染：槽位列表（版本号/时间戳）+ 金/进度摘要。

### 3.7 卡牌/事件（CardEngine）
- `rules::cards::CardEngine`：`event_to_card(event_id, catalog)` → `apply(card, state)`
  → `CardResult`（`Applied(effect)` / `UnknownEvent`）。
- 在 MainMap 事件码 3（卡牌随机）/ 卡片翻面（事件码 5）时调用。
- 效果码 1..4（Damage/Heal/Gold/Move）作用于 `CardState{hp,gold}`（spec/cards.md §c，**推测**映射）。

## 4. 表现层（复用 menu_shell 机制，不重做）

- 帧缓冲：`ti_platform::gfx::Framebuf`（176×220 逻辑，RGB565）+
  `FramePresenter`（native `FullRebuild` / wasm `Partial`，EXP-045/046）。
- 字形：`ti_app::glyph_raster::Raster`（真 `GoldLand.Fnt` 墨点 + 豆腐块回退）。
- 布局常量：`menu_shell` 的 `LOGICAL_*` / `GLYPH_ADVANCE` / `ROW_H` 等沿用。
- 各场景在 `GameApp::render` 内按当前 `Scene` 分发到对应渲染子函数，
  统一画到同一 `Framebuf`（脏行带机制不变，静止帧零 GPU，S4.3）。

## 4b. 输入（键盘 + 触屏双路径，S5.5）

两条**并行**的输入路径，每帧各调一次，最终都汇到 `GameApp::handle`：

| 路径 | 入口 | 产出 | 适用端 |
|---|---|---|---|
| 键盘 | `GameApp::poll(&KeyState)` | `GameInput` | 桌面（浏览器可兼） |
| 触屏 | `GameApp::poll_touch(Option<TouchSample>)` | `GameInput` | Android / 触屏浏览器 / 桌面鼠标 |

### 4b.1 触屏：无虚拟按键，直接触摸界面元素

**理念**（`docs/modernization.md` §5）：不画虚拟方向键/虚拟手柄，
玩家手指点哪就是哪。`TouchPad`（`ti_app::touch`）负责屏幕坐标 → 逻辑坐标反变换 +
手势识别（点按 / 四向滑动），产出 `TouchSample{g, x, y}`；`GameApp::poll_touch`
按场景把命中区域映射成 `GameInput`。

| 场景 | 点按 | 滑动 |
|---|---|---|
| Index 主菜单 | 点菜单行 = **选中该行并立即执行** | — |
| SelectRole / SelectMap | 点某行 = 选中该项 + 确认 | 下滑 = 返回 |
| MainMap | 点格子 = 朝该格逐格走（先走差值大的轴，遇墙/越界即停，单次最多 6 步）<br>点自己所在格 = 触发据点事件 `Landmark(code)` | 滑动 = 朝该方向走一步 |
| Vs 战斗 | 点任意处 = 攻击一回合 | 下滑 = 退出 |
| Message | 左半屏 = 关闭；右半屏 = 下一页 | 上滑 = 下一页；下滑 = 上一页 |
| Saveload | 点有档的槽 = **读档**；点空槽 = **存档** | 下滑 = 关闭 |

### 4b.2 命中区与渲染坐标严格互逆

`ti_app::touch::layout` 的命中区常量**必须**与 `GameApp::render_*` 的绘制坐标一致：

| 命中函数 | 对应渲染式 |
|---|---|
| `menu_row_at` | 高亮矩形 `(18, MENU_TOP+i*ROW_H-12, 140, ROW_H-2)` |
| `list_index_at` | 选中高亮 `(20, 50+i*30-10, 136, 26)` |
| `grid_cell_at` | `px = col*6 + 20` / `py = row*6 + 30` 的**逆变换** |
| `slot_at` | 槽位行 `y = 40 + i*40` |

改了绘制坐标必须同步改 `layout`，否则触屏命中会偏。

### 4b.3 关键不变量

`Viewport` 扩为双轴缩放（`scale_x`/`scale_y`）后，**四档缩放模式下
屏幕→逻辑→屏幕坐标都精确往返**（P4-2），故触屏命中判定不会因拉伸/裁剪而偏移
（`ti-platform` 7 项单测覆盖）。若改 `Viewport` 的坐标变换，必须保持该往返不变式。

### 4b.4 端侧接线

| 端 | 键盘 | 触屏 | bin |
|---|---|---|---|
| 桌面 native | ✅ | ✅（鼠标兼触屏） | `treasure_native` |
| 浏览器 wasm | ✅ | ✅ | `treasure_wasm` |
| Android | ❌（`KeyState::default()`） | ✅（唯一输入） | `treasure-android` |

三壳都调 `refit()` **再** 轮询输入（触摸反变换要用本帧视口）。
Android 构建见 [`../android/README.md`](../android/README.md)。

## 4c. 渲染：占位 → 真实精灵（S5.6）

各 `render_*` 方法仍画占位（`fill_rect` + `draw_text_row`），保证无美术资源时也不黑屏；
`GameApp::render` 在占位渲染**之后**叠加真实精灵：

```
render() {
    render_<scene>()               // 占位：背景色 + 标题 + 选中高亮 + 提示文字
    if atlas && scene_cache[name] {
        atlas.blit_scene(scene, &mut fb)   // 真实 PCX 按 (f2,f3) 坐标 blit 到帧缓冲
    }
}
```

- 寻址：`f1=N` → `ui.pak` filenum INI `[N] filename=NN.pcx` → `name_key("NN.pcx")` →
  PCX 解码 → 调色板转 RGB565 → `(f2,f3)` 坐标 blit（越界裁到 176×220）。
- 缓存：`SpriteAtlas` 解码后存 `HashMap<f1, DecodedSprite>`，避免每帧 RLE 解压。
- 资源注入：三端 bin 在 `GameApp::new` 后调 `set_atlas(ui_pak)` + `add_scene(name, bytes)`。
  wasm 走 `load_file`，native 走 `read_local`，Android 走 `read_asset`。
- 已知限制：`filenum INI` 中 `num>1` 的多帧动画只取第 0 帧；地形瓦片
  （`glmap_m*.pak`）未接入，主地图仍是占位网格。

### 4c.1 多帧动画（S6.4）

`filenum INI` 的 `num` 字段是帧数。实测 `ui.pak` 120 个精灵中 **4 个 `num=5`**
（f1 = 14/31/32/37），它们的 PCX 高度 ≈ 5 × 40 —— 帧**纵向堆叠**在同一张图里。

| API | 作用 |
|---|---|
| `FRAME_MS` | 每帧时长（毫秒），默认 100（约 10 fps） |
| `DecodedSprite::frame_offset(frame)` | 帧号 → 像素行偏移（自动环绕 `frame % frames`） |
| `SpriteAtlas::advance_animation(delta_ms)` | 按毫秒累积推进，返回当前帧号 |
| `SpriteAtlas::frame()` | 当前帧号 = `elapsed_ms / FRAME_MS` |
| `SpriteAtlas::blit_scene_frame(scene, fb, n)` | 指定帧号 blit（测试/定格用） |

**每帧驱动**：native + wasm 用 `macroquad::time::get_time()` 算本帧耗时
（`tick_frame_time`：首帧 16ms 默认步长，后续实际 delta，clamp 1..250ms），
写入 `GameApp::frame_delta_ms`；`GameApp::render` 在 blit 前调
`atlas.advance_animation(self.frame_delta_ms)`。

**脏行带联动**：`blit_sprite` 按可见区域置脏行带，动画每帧重绘不会全帧上传。

**已知限制**：多帧精灵是 UI 控件（战斗动画/提示框等），不是场景装饰；
等距地形瓦片（`glmap_m*.pak`）当前全为静态图（INI 全 `num=1`），不受影响。

固定种子全流程脚本（`tests/scripts/full_run.json`，`--features headless --bin treasure-headless`）必须覆盖：

```
开局 → 选人(select_role) → 选图(select_map)
     → 主地图移动(move_to/walk) → 据点事件分派(dispatch)
     → 贸易(buy/sell) → 建造(build) → 战斗(battle step 到 outcome)
     → 存读档(write_slot/read_slot) → 回主菜单
```

- headless：无窗口、无 macroquad 渲染，只跑 `GameApp::update` 逻辑链，
  固定 `GameRng` 种子（如 `seed = 42`），逐步喂输入事件，断言每步场景/状态正确。
- 这是 P3-3 的**机器可判定**判据（替代 S4.4 曾用 smoke.ps1 冒烟的缺口）。

## 6. 验收命令（落地后填 ledger DoD）

| 任务 | 验收命令 |
|---|---|
| S5.1 交互壳接通 | `cargo test -p ti-app`（GameApp 状态机单测 + 全链路 7 项断言）+ 桌面 exe 实跑：菜单→选人→选图→主地图→贸易→建造→战斗→存读档 全链路可点通 |
| S5.2 全流程 headless | `cargo run -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json`（seed=42）→ 16 步场景流转断言 0 失败（P3-3） |
| S5.3 P4-1 渲染对齐 | `cargo test -p ti-app --test render_parity`（headless 渲染 5 关键场景，确定性/非空白/场景区分/真字形/176×220 逐像素一致） |
| S5.4 业务对齐收敛 | 各 spec `推测` 项收敛 + `docs/release.md` 更新 + `gate.ps1` 8 项全绿（S5.4 时点 8 项，当前 11 项）+ 桌面/浏览器产物 sha256 入 evidence（**桌面/浏览器为当前收口目标**；Android APK 在 Mac 工具链编译，见 S2.6） |

## 7. 实现约束（硬规则）

1. **不重抄数值**：壳层一切判定/数值走 `ti-core::rules`；壳只负责 UI/输入/渲染。
2. **不占位**：`TODO/unwrap/expect/panic!/unimplemented!` 在 `src/` 0 命中（gate 第 5/6 项）。
3. **跨场景状态传递**：`(character, map_slot)`、`gold`、`boss_tier` 等由 `GameApp` 顶层字段
   持有，场景切换时注入对应 rules Session 构造（不跨场景共享可变指针）。
4. **RNG 单一源**：战斗/事件随机全走 `rules::rng::GameRng`（LCG，EXP-040），壳不另开 rand。
5. **每场景可独立渲染**：`GameApp::render` 按 `Scene` 分发，头屏资源预算不破坏（S4.3）。

## 8. 文件改动清单（S5.1）

- `crates/ti-app/src/game_app.rs`（新）：`GameApp` 场景状态机 + 各场景渲染/更新。
- `crates/ti-app/src/menu_shell.rs`：保留（Index 场景渲染子视图 + 帧缓冲/字形机制）。
- `crates/ti-app/src/shell_ui.rs`：`event_id` 从占位改为 `menu.handle` 的真实事件码
  （`Click(0..=5)`，spec/menu.md §a，与 rules 对齐）。
- `crates/ti-app/src/bin/{treasure_native,treasure_wasm}.rs`：入口改持 `GameApp`。
- `crates/ti-app/Cargo.toml`：加 `headless` feature（S5.2）。
- `tests/scripts/full_run.json`（新，S5.2）+ `crates/ti-app/tests/headless_full_run.rs`（新）。
- `plan/ledger.yaml`：登记 S5.1~S5.4。
