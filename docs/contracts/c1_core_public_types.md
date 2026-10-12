# c1 — ti-core 公共类型契约（跨包可见面）

**状态：draft**（S12.0 契约冻结轮置 `frozen`）

跨包共享的**类型/枚举/访问器**清单。包内实现细节不进契约；包间只允许 use 以下路径。

## 1. 域模型（`ti_core::domain`，唯一类型家）

| 模块 | 跨包可见类型 | 业务依据 |
|---|---|---|
| `domain/combat` | `Unit`、`Side` | [`business/combat.md`](../business/combat.md) |
| `domain/economy` | `Bank`、`Building`、`Goods`、`PriceTable`、`TradeEvent`、`TradeResult`、`TradeErr`、`BuildErr` | [`business/economy.md`](../business/economy.md) |
| `domain/map` | `Dir`、`GridPos`、`MapTerrain`、`SceneTarget` | [`business/progression.md`](../business/progression.md) |
| `domain/flow` | `MenuScene`、`MenuEvent`、`Sfx`、`SceneId`、`SceneState` | [`business/loop_core.md`](../business/loop_core.md) |
| `domain/messages` | `MsgKind`、`MsgEvent`、`MsgPage` | [`business/loop_core.md`](../business/loop_core.md) §消息 |
| `domain/persistence` | `SaveSlot`、`SaveErr`、`MIN_SAVE_SIZE`、`REC_MAGIC` | [`business/persistence.md`](../business/persistence.md) |
| `domain/selection` | `Character`、`MapSlot`、`SelectionState` | [`business/progression.md`](../business/progression.md) §选择 |
| `domain/render` | `SceneAsset`、`ResourceRef`、`ResourceContainer`、`RenderPlan`、`RenderError` | [`business/loop_core.md`](../business/loop_core.md) §资源 |

## 2. 随机（`ti_core::rules::rng`，唯一随机源）

- `GameRng`：可播种随机数生成器（PCG64 库化，`rand_pcg`）；
  跨包规则 = 玩法层**只经** `GameRng` 取随机，禁 `rand::thread_rng`（不可复现）；
- `RngState` + `srand/rand/seq`：与旧存档续流的种子接口（跨包只经 `GameRng`，不直接传 `RngState`）。

## 3. 数据访问器（`ti_core::data`，数值唯一来源）

| 访问器 | 来源 JSON | 契约含义 |
|---|---|---|
| `balance()` | `balance.json` | 全部玩法数值的唯一注入点；解析失败 = 硬错误（不 fallback） |
| `strings()` / `pairs()` | `strings.json` | 全部玩家可见文本；代码里禁中文面量 |
| `scenes()` | `scenes.json` | 场景布局；表现层坐标走此表 |
| `controls()` | `controls.json` | 控件记录（`gen_layout.py` 生成）；渲染/命中走 `WidgetLayer` |

数据表类型（跨包可见）：`Balance` / `BalanceCfg`、`Strings` / `StringPair`、
`SceneLayout` / `ControlRec` / `ControlsCfg`（均经 `ti_core::data` 重导出）。

## 4. 常量（`ti_core` crate 根）

`LOGICAL_WIDTH` / `LOGICAL_HEIGHT`：176×220 逻辑分辨率（跨包坐标系基准，
[`business/07_ui_flow.md`](../business/07_ui_flow.md) §5 约束 1）。

## 5. 包依赖铁律（跨包只经两条通道）

1. **类型通道**：只 use 本文 §1/§3/§4 列出的公共类型（经 `domain` / `data` 重导出，不直连子模块）；
   会话对象（`rules::battle::BattleSession`、`rules::main_map::MapSession`、
   `rules::main_loop::SceneStack` 等）属包内实现，跨包经 `SceneHost` 方法访问（见 c3），不直接构造；
2. **数据通道**：数值/文案只读 §3 访问器返回的 key，不在代码里写死。

## 校验锚点

```
ti_core::domain::combat::Unit
ti_core::domain::economy::Building
ti_core::domain::map::GridPos
ti_core::domain::map::MapTerrain
ti_core::domain::messages::MsgKind
ti_core::domain::persistence::SaveSlot
ti_core::domain::selection::Character
ti_core::domain::flow::MenuScene
ti_core::rules::rng::GameRng
ti_core::rules::battle::BattleSession
ti_core::rules::main_map::MapSession
ti_core::rules::main_loop::SceneStack
ti_core::data::balance
ti_core::data::strings
ti_core::data::scenes
ti_core::data::controls
ti_core::data::ControlRec
ti_core::data::ControlsCfg
ti_core::LOGICAL_WIDTH
```
