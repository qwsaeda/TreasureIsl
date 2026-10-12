# c3 — 表现层会话 API 契约（`GameApp` / `SceneHost`）

**状态：draft**（P6 表现层包开工前由 S12.0 契约冻结轮置 `frozen`）

表现层（`ti-app/scenes/*`、`widgets`、`gfx`、`input`）与组合根（`GameApp`）之间的
**唯一公共面**。场景 struct 只依赖 `dyn SceneHost`（S9.2 按职责拆 8 个窄接口后
`SceneHost` 退化为空 supertrait 聚合，`dyn SceneHost` 仍可访问全部方法）。

## 1. 场景标识与跳转

| 面 | 位置 | 契约内容 |
|---|---|---|
| `SceneKind` | `crates/ti-app/src/scenes/mod.rs` | 9 变体（`Intro`/`Index`/`SelectRole`/`SelectMap`/`MainMap`/`Vs`/`Message`/`Saveload`/`Exit`）；`discriminant()` 取 0..8 互异 |
| 合法转换 | `scene_transition!` 宏调用表（宏同时生成 `Default for SceneTransition` 与 8 个窄接口的 `SceneControl` 组实现） | 19 条 `SceneTransition<From, To>`（宏调用，见 `scenes/mod.rs` 底部）+ 路由层 1 条派生（`Intro`/`Exit` 无场景 struct，`route_input` 就地 `enter`）= 共 20 条出边；与 [`business/07_ui_flow.md`](../business/07_ui_flow.md) §1 转换表逐条对应（加转换须改宏 + 业务表两处） |
| `Scene` trait | `scenes/mod.rs` | 三方法：`kind()` / `on_input(host, GameInput)` / `on_touch(host, &TouchSample)`（默认不消费）/ `render(host)` |

## 2. `SceneHost`（组合根能力接口，S9.2 拆分为 8 个子 trait）

`GameApp` 实现 8 个子 trait + 空的 [`SceneHost`] supertrait 聚合
（`trait SceneHost: NavHost + RenderHost + SpriteHost + MenuHost +
SelectionHost + MapHost + EconomyHost + SessionHost`）。
每个方法是一行字段读写或一次原子组合，
**业务判定在场景侧，不在宿主侧**。

### 2.1 子 trait 方法清单

| 子 trait | 方法（签名不变） |
|---|---|
| `NavHost` | `current_scene()`、`enter(next: SceneKind)`、`advance(input: GameInput) -> bool` |
| `RenderHost` | `shell()`、`shell_mut()`、`clear_frame(u16)`、`fill_rect(i32,i32,i32,i32,u16)`、`draw_text(&str,i32,i32)` |
| `SpriteHost` | `blit_scene_sprites(&str)`、`draw_terrain() -> bool`、`terrain_grid_w() -> u32`、`terrain_camera() -> (i64,i64)`、`has_terrain_atlas() -> bool` |
| `MenuHost` | `menu_selected()`、`set_menu_selected(usize)`、`reset_selection()`、`open_help()`、`request_quit()` |
| `SelectionHost` | `character()`、`set_character(Option<Character>)`、`map_slot()`、`set_map_slot(Option<MapSlot>)`、`selection()`、`selection_mut()`、`map_unlocked(MapSlot) -> bool`、`enter_main_map()` |
| `MapHost` | `map_pos()`、`set_map_pos(Option<GridPos>)`、`map_terrain() -> Option<MapTerrain>`、`map_landmarks() -> Vec<(u32,u32)>`、`map_session_move(Dir) -> StepResult`、`map_session_dispatch(u32) -> Option<SceneTarget>`、`set_last_dispatch(Option<SceneTarget>)`、`start_battle()`、`open_message(MsgKind)` |
| `EconomyHost` | `price_of(u32)`、`sell_price_of(u32)`、`gold()`、`set_gold(u32)`、`owned_index_of(u32)`、`owned_push(u32,u32)`、`owned_bump(usize)`、`owned_count(u32)`、`owned_total()`、`owned_take_all(u32)`、`bank_gold()`、`spend_bank_gold(u32)`、`build_increment(Building)` |
| `SessionHost` | `play_sfx(Sfx)`、`battle_step()`、`battle_snapshot() -> Option<BattleSnapshot>`、`msg_kind()`、`msg_page()`、`msg_total()`、`msg_next_page()`、`msg_prev_page()`、`close_message()`、`save_slots() -> &[SaveSlot]`、`write_save_slot(usize,SaveSlot)`、`restore_save(SaveSlot)`、`in_map() -> bool`、`in_battle() -> bool` |

**冻结规则**

1. 场景侧只调 `dyn SceneHost`（supertrait 聚合，方法经子 trait 展开）；**禁止**直接摸 `GameApp` 字段（越界即打回）；
2. 新增方法 = 契约变更（全票同步 + 重跑全 workspace test）；
3. S9.2 已按上表 8 个分组完成拆分，签名不变；拆分后 P6 不许改任何签名。

## 3. 渲染/触摸公共面（P6 包内，跨包可见）

| 面 | 契约 |
|---|---|
| `MenuShell` | 帧缓冲 + 字形 + 音频状态 + 缩放模式；`present()/present_hd()` 上屏 |
| `GpuRenderer`（`ti_platform::gpu_renderer`） | `blit_sprite(dst, &SpriteRef)` / `blit_terrain_tile` / `draw_ttf_text`；headless 时只累计 draw call |
| `TouchPad` / `TouchSample` | 逻辑坐标手势（`Tap`/`SwipeUp/Right/Left/Down`），阈值 20 逻辑像素；主地图点按走等距逆变换（`iso_grid_cell_at`，与渲染同坐标系） |
| `WidgetLayer` | 只读 `controls.json`：`visible(scene_id)` / `draw` / `hit`；控件坐标禁止在 `.rs` 硬编码 |

## 校验锚点

```
ti_app::scenes::SceneKind
ti_app::scenes::SceneHost
ti_app::scenes::NavHost
ti_app::scenes::RenderHost
ti_app::scenes::SpriteHost
ti_app::scenes::MenuHost
ti_app::scenes::SelectionHost
ti_app::scenes::MapHost
ti_app::scenes::EconomyHost
ti_app::scenes::SessionHost
ti_app::scenes::Scene
ti_app::scenes::SceneTransition
ti_app::game_app::GameApp
ti_app::menu_shell::MenuShell
ti_app::touch::TouchPad
ti_app::touch::TouchSample
ti_app::widgets::WidgetLayer
ti_platform::gpu_renderer::GpuRenderer
```
