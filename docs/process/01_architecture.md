# 架构规范（Architecture）

## 1. 分层与依赖方向

```
┌─────────────────────────────────────────────┐
│ crates/ti-app        表现层：场景 / 控件 / 输入 / 四端 bin │
├─────────────────────────────────────────────┤
│ crates/ti-platform   平台层：窗口 / 帧缓冲 / 音频 / 存储 │
├─────────────────────────────────────────────┤
│ crates/ti-core       玩法层：纯逻辑，零 IO、可单测 │
├─────────────────────────────────────────────┤
│ crates/ti-formats    格式层：pak / PCX / 字体 / 地图 / 存档解析 │
└─────────────────────────────────────────────┘
        ↕ 单向依赖，只能向下，禁止反向依赖
```

> **层序以 `Cargo.toml` 实测为准**（2026-10 校正）：
> `ti-app → {ti-core, ti-platform, ti-formats}`、
> `ti-platform → {ti-core, ti-formats}`、`ti-core → ti-formats`。
> 即 **`ti-platform` 位于 `ti-core` 之上**（仅为 `LOGICAL_WIDTH/HEIGHT` 常量，
> 见 `ti-platform/src/gfx.rs` 的 `use ti_core::{LOGICAL_HEIGHT, LOGICAL_WIDTH};`）。
> 旧版本文档曾把 `ti-core` 画在 `ti-platform` 之上，与实测依赖相反，已更正。

- **玩法层零 IO 硬约束**：`ti-core` 不得依赖任何平台后端（实测只依赖 `ti-formats`），因此能在无窗口环境跑全流程回归（`treasure-headless`）。
- **格式层只做编解码**，不含玩法判断、不含渲染。
- **表现层不含数值**：所有金币/伤害/价格来自 `ti-core::rules`，表现层只画（详见 `02_conventions.md` §1）。
- **跨层直接依赖是既成事实，非禁令**：`ti-app/Cargo.toml` 直接声明 `ti-formats`（读资源需要），
  分层是**设计意图**（玩法判定不越层），不是 Cargo 层面的机械禁令。
- **逆向参照物不进接口**：`docs/facts`/`ref/pseudocode` 不出现在任何 `pub` API 签名或 trait 约束里。

### 1.1 表现层内部分层（`ti-app`）

```
ti-app/src/
  game_app.rs        组合根：状态字段 + 键位/触摸/帧路由 + impl SceneHost
  scenes/mod.rs      SceneHost 能力接口 + Scene trait + 路由（route_*）
  scenes/<scene>.rs  每场景一个 struct：on_input / on_touch / render
  menu_shell.rs      帧缓冲 / 字形 / 呈现机制（渲染子系统）
  bin/               native / wasm / android / headless 四端入口
```

硬边界：

1. 场景 struct **不持有** `GameApp`，只持有 `&mut dyn SceneHost`；
2. `SceneHost` 的每个方法都是**一行状态读写 / 表现转发**，业务判定在 `scenes/*.rs`；
3. `game_app.rs` 不出现 `handle_X` / `touch_X` / `render_X` 场景分支；
4. 场景之间需要同时可变借用两个字段时（如 `map_session` + `rng`），
   由 `SceneHost` 提供**单一能力方法**（`map_session_move`），避免宿主暴露裸字段引用；
5. **现状（过渡期）**：`SceneHost` 是**单一宽接口**（约 60 方法，`scenes/mod.rs` L355+），
   Rust 不允许 trait object 作为 supertrait，动态分发只能落在单接口上；场景统一持
   `&mut dyn SceneHost`。`scenes/mod.rs` 头部已标注此过渡形态。
6. **目标态（S9.2 已落地）**：按**职责拆分为窄接口**（接口隔离原则），场景只依赖它
    需要的窄接口（8 个子 trait，`scenes/mod.rs` S9.2 已落地，与 c3 契约 §2.1 对齐）：
    - `NavHost`：场景切换 + 输入路由（`current_scene`/`enter`/`advance`）
    - `RenderHost`：帧缓冲/字形（`shell`/`shell_mut`/`clear_frame`/`fill_rect`/`draw_text`）
    - `SpriteHost`：精灵/地形渲染（`blit_scene_sprites`/`draw_terrain`/`terrain_grid_w`/`terrain_camera`/`has_terrain_atlas`）
    - `MenuHost`：主菜单状态（`menu_selected`/`set_menu_selected`/`reset_selection`/`open_help`/`request_quit`）
    - `SelectionHost`：角色/地图选择状态（`character`/`set_character`/`map_slot`/`set_map_slot`/`selection`/`selection_mut`/`map_unlocked`/`enter_main_map`）
    - `MapHost`：主地图状态（`map_pos`/`set_map_pos`/`map_terrain`/`map_landmarks`/`map_session_move`/`map_session_dispatch`/`set_last_dispatch`/`start_battle`/`open_message`）
    - `EconomyHost`：金/价格/持有/银行/建筑（`gold`/`set_gold`/`price_of`/`owned_*`/`bank_*`/`build_increment`）
    - `SessionHost`：会话/战斗/消息/存读档（`play_sfx`/`battle_step`/`battle_snapshot`/`msg_*`/`close_message`/`save_slots`/`write_save_slot`/`restore_save`/`in_map`/`in_battle`）
    `SceneHost` 退化为 8 个子 trait 的空 supertrait 聚合（`dyn SceneHost` 经 supertrait 展开仍可访问全部方法）。

## 2. crate 现状（2026-10）

| crate | 职责 | 备注 |
|---|---|---|
| `ti-formats` | 资源格式解析/序列化（pak/lpk、PCX、字体、`.Map`、存档） | 真实资产，保留 |
| `ti-core` | 玩法规则（菜单/选人/选图/地图/贸易/建造/战斗/卡牌/存读档/消息/RNG） | 含 67 处「推测」（2026-10-10 实数，含 oracle+rules+tests），S8 重写 |
| `ti-platform` | 帧缓冲、输入、音频、存储、设置 | |
| `ti-app` | 场景状态机、UI 控件、三端 bin（native/wasm/android/headless） | 内部分层见 §1.1；组合根 `game_app.rs` + 场景模块 `scenes/` |
| `ti-tools` | 离线工具：unpak / map2json / build_assets / controls / classify / specskel / doccheck | 工具链落点 |

## 3. trait 约定

玩法层对每个系统定义一个「会话」对象（可持有状态），表现层只调它的方法，不重抄数值。

```rust
// 惯例：会话 struct + 单方法入口；表现层只见 trait。
pub trait Scene {
    /// 输入事件 → 是否消费了该事件（false = 未消费，交给全局）
    fn handle(&mut self, input: &InputEvent) -> bool;
    /// 渲染到帧缓冲
    fn render(&mut self, fb: &mut Framebuf);
}
```

约定：

1. **单一入口 trait**（`Scene` / `Widget`），不用多态字段堆；
2. 会话状态在 `struct` 里，不用全局 `static mut`；
3. trait 方法返回 `bool`（是否消费事件）或小枚举（`Flow`），不返回 `Result<(), E>` 给表现层——错误在玩法层内部收敛；
4. 新增系统 = 加一个 `rules/<sys>.rs` + 一个会话 struct + 单测，不改其他文件。

## 4. 现成库清单（**不自造轮子**）

引入新依赖前先查此表；表里没有的，先在 `docs/experience/index.md` 记「查过 crates.io 无合适库」再自造。

| 需求 | 用这个 | 版本 | 备注 |
|---|---|---|---|
| 错误类型 | `thiserror` | 2 | 已用 |
| 序列化（存档/配置） | `serde` + `serde_json` | 1 | 已用；**不要** `bincode`（存档要人可读+可迁移） |
| 解压 | `flate2` | 1 | 已用 |
| PNG 解码 | `png` | 0.17 | 已用（HD 资产，可选 feature） |
| 窗口 / 渲染 | `macroquad` | 0.4 | 已用，三端统一 |
| OGG/Opus 编码 | `opus-pure` | 0.2 | 已用，纯 Rust |
| 数据表解析 | `serde_json` + `include_str!` | 1 | 数值/文本表走 JSON，不写进 .rs |
| **随机** | `rand_pcg` + `rand_core` | =0.9.0 / =0.9.5 | 可复现 PCG64；玩法层禁止 `thread_rng`（S8.7 落地引入；旧自造 MSVC LCG 降级 oracle 历史参照）；未引 `rand` 主 crate |
| **哈希** | `fnv` | 1 | 容器名→key；不要手写 FNV（S8.7 落地引入） |
| **文本格式化** | `write!` / `format!` | std | 禁 `format!("{}", x)` 无占位 |
| **时间** | `std::time::Instant` | std | 帧计时 |
| **随机洗牌** | `rand::seq::SliceRandom` | 0.8 | 卡牌抽牌，禁止手写 Fisher-Yates |

> **新增依赖判据**：① crates.io 有活跃维护的库 → 用；② 无 → 自造但必须有单测；
> ③ 引入需在 `Cargo.toml` 注释写明「为什么不用现有方案」（`docs/style.md` §5.9 同规则）。

## 5. 数据驱动（数值与文本一律进 data/）

**原则：调平衡不改代码。**

```
assets/data/
  balance.json      # 金币上限、价格表、BOSS 档、伤害系数、回合上限
  strings.json      # 按 key 的中文文本（i18n 雏形）
  scenes.json       # 场景 id / 名称 / 布局常量
  controls.json     # 控件表（源自 ref/controls.json，接入控件层）
```

加载约定：

1. `include_str!` 进包（wasm 免 IO），或用 `ti_platform::storage` 按平台加载；
2. 反序列化失败 = **启动即报错**，不 fallback 到默认值（避免「能跑但数值错」）；
3. 解析结果进 `OnceLock`/`LazyLock`，每帧不重解析。

**收益**：数值调整 = 改 JSON = 可 diff、可评审、免编译；LLM 不再有机会「顺手改数值」。

## 6. 三端一致性

| 层 | native | wasm | android |
|---|---|---|---|
| 玩法 | 同一 `ti-core` | 同一 `ti-core` | 同一 `ti-core` |
| 表现 | `GameApp` + macroquad 窗口 | `GameApp` + WebAudio | `GameApp` + AAudio |
| 资源 | `data/` | `web/` 打包 | `assets/` |

三端**共用同一 `GameApp`**，端差异只在 bin 入口的资源加载与音频后端。禁止在玩法/表现层写 `#[cfg(target_os)]` 业务分支。