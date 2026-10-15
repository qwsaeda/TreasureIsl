# 正向设计总入口（Forward Design）

> **定位**：《金银岛》正向开发的**单一设计总览**——讲清「这个游戏怎么设计、怎么架构、怎么模块化」，
> 是阅读项目的第一入口（AGENTS.md 路由表首行）。
> 本文只写**设计决策**（为什么 + 是什么），不写实现细节（怎么做在 `docs/process/` + `docs/impl/`），
> 不引用任何逆向产物（`legacy/` 仅作历史参照归档）。

---

## 1. 核心循环（游戏怎么玩）

海盗船长在等距网格地图上往返据点，**回合制策略经营 + 探索冒险**，节奏「小步高频」（每次操作 1–2 秒出结果，无实时、无长动画）。

**循环五环**：探索（移动）→ 贸易（换金）→ 建造（升级城堡，抬金上限 + 产能）→ 卡牌事件（意外增益/损失）→ 战斗挑战 BOSS，环形闭环。

| 项 | 设计 |
|---|---|
| 胜利 | 在任意地图击败该地图的 BOSS |
| 失败 | 角色 HP 归零（金币**不是**失败条件，回主菜单可再挑战） |
| 金币 | 支付建造/升级；不是通关门槛 |
| 回合结算 | 每次玩家操作后固定三段：① 动作生效 → ② 地图/据点响应 → ③ 状态检查 |

**可断言约束**：单次操作至多一次结算流水线；`0 ≤ gold ≤ gold_max`；HP ∈ [0, hp_max]；任意场景 ≤ N 次操作可回主菜单。
详见 `docs/business/loop_core.md`。

---

## 2. 数据驱动分层（数值/文本/机制分离）

| 层 | 写在哪 | 谁改 | 例 |
|---|---|---|---|
| **设计意图** | `docs/business/` | 策划/评审 | 「伤害 = f(atk, rng)，玩家先手，回合制」 |
| **平衡数值** | `assets/data/balance.json` | 策划，改 JSON 即可 | `boss_tiers[0].hp = 50000` |
| **文本** | `assets/data/strings.json` | 策划/本地化 | `scene.select_role.title = 选择角色` |
| **实现** | `crates/ti-core/src/rules/` | 工程师 | `struct CombatSession` |

**纪律**：改数值 = 改 JSON（不碰代码）；改机制 = 先改 `docs/business/`（评审）再改代码。
业务规则唯一事实源 = `docs/business/` + `assets/data/*.json`，描述「游戏应该怎么玩」，不描述实现。
详见 `docs/business/00_index.md`。

---

## 3. 架构分层（5 crate 单向依赖）

| crate | 职责 | 约束 |
|---|---|---|
| `ti-formats` | 资源二进制格式解析/序列化 | 零依赖，纯字节 |
| `ti-core` | 纯玩法逻辑（数值、规则、场景状态机） | **零 IO**，只依赖 `ti-formats`，可无窗口回归 |
| `ti-platform` | 平台抽象（输入/存储/音频/定时 + macroquad 后端） | 不依赖 `ti-app` |
| `ti-app` | 组装层 + 场景状态机 + 三端 bin（native/wasm/android） | 依赖 core + platform + formats |
| `ti-tools` | 离线工具（解包/校验/codegen） | 依赖 formats |

**依赖方向**：`ti-app → ti-platform → ti-core → ti-formats`，单向向下，**禁止反向依赖**。
**零 IO 约束**：`ti-core` 不碰任何平台后端 → 全流程回归可 headless 跑。
**表现层不含数值**：金币/伤害/价格全来自 `ti-core::rules`，`ti-app` 只画。
详见 `docs/impl/01_architecture.md`。

---

## 4. 场景流（9 场景 + 20 合法转换）

9 个场景：`Intro`（片头，自动跳 Index）/ `Index`（主菜单 6 项）/ `SelectRole`（5 角色）/ `SelectMap`（已解锁地图）/ `MainMap`（移动/贸易/建造/事件）/ `Vs`（战斗）/ `Message`（叠加窗口）/ `Saveload`（叠加子窗口）/ `Exit`。

**20 条合法转换**是唯一事实源（`docs/business/07_ui_flow.md` §1），代码侧 `scene_transition!` 宏逐条对应同一张表。
**硬规则**：非法转换（如 `SelectRole → Vs`）在类型层面无实现、编译期报错；场景判别值 0..8 一场景一值；切场景后关闭态必须复位（选中项/页码/棋子位置），禁止残留。
**UI 约束**：逻辑分辨率固定 176×220，缩放只改呈现不改逻辑；所有坐标经 `Viewport` 反变换到逻辑坐标再进业务层。

---

## 5. 模块边界（7 并行包 + 3 跨包契约）

按域划分 7 个并行包，包间白名单文件集合两两交集 = ∅（机器校验），跨包只经 3 份契约通信：

| 包 | 域 | 依赖 |
|---|---|---|
| D | 数据/工具（`assets/data/*.json` + `ti-core/data` 访问器） | 先行 |
| P1 | 战斗（`rules/battle` + `domain/combat`） | D |
| P2 | 经济（`trade` + `build`） | D |
| P3 | 地图/卡牌（`main_map` + `cards`） | D |
| P4 | 流程/菜单/消息（`menu`/`selection`/`messages`/`main_loop`/`render`） | D |
| P5 | 存档+RNG（`saveload` + `rng`） | P1–P4 |
| P6 | 表现层（`ti-app/scenes` + `ti-platform`） | P1–P5 |

**3 份跨包契约**：`c1` = `ti-core` 公共类型（`domain/*` + rng + data 访问器）；`c2` = `assets/data/*.json` 的 key 契约；`c3` = 表现层会话 API（`GameApp`/`SceneHost` 8 窄子 trait）。
**两条跨包通道**：① 类型通道 — 只 use 契约列出的公共类型；② 数据通道 — 数值/文案只读访问器返回的 key，代码禁写死。
详见 `docs/contracts/00_index.md` + `plan/packages.yaml`。

---

## 6. 场景宿主窄接口（设计意图）

场景 struct **不持有** `GameApp`，只持 `&mut dyn SceneHost`（8 个子 trait 的 supertrait 聚合）：
`NavHost`（场景切换/路由）、`RenderHost`（帧缓冲/字形）、`SpriteHost`（精灵/地形）、`MenuHost`（主菜单）、
`SelectionHost`（选角/选图）、`MapHost`（主地图会话）、`EconomyHost`（金/价格/银行/建造）、`SessionHost`（音效/战斗/消息/存读档）。

每个宿主方法是一行状态读写/表现转发，**业务判定在场景侧**。配合三个设计模式保证纪律：
- **Builder**：复杂 `GameApp` 构造走链式（字段 > 3 的 struct 禁 `new()` 散字段）；
- **Type-State**：场景转换编译期安全（非法转换不可构造）；
- **Command**：输入事件封装为可序列化对象，支持重放/回放。

详见 `docs/process/01_architecture.md` §1.1 + `docs/style.md` §5.6。

---

## 7. 阅读路由（设计 → 实现 → 规范）

| 你要 | 读 |
|---|---|
| 游戏怎么设计（玩法/规则/数值） | `docs/business/`（唯一事实源） |
| 怎么架构（5 crate 分层） | `docs/impl/01_architecture.md` |
| 场景怎么连（9 场景 + 转换 + 输入） | `docs/business/07_ui_flow.md` |
| 版面怎么摆（9 场景版面 + 配色） | `docs/business/08_layout_design.md` |
| 模块怎么分（7 包 + 3 契约） | `docs/contracts/` + `plan/packages.yaml` |
| 工程怎么做（分层/约定/模板/工具） | `docs/process/` |
| 渲染契约（路径/z-order/越界/透明） | `docs/process/07_render_pipeline.md` |
| 踩过的坑 | `docs/experience/`（历史参照卷，非正向依据） |

> **逆向归档**：原版 exe / 伪代码 / 逆向规格 / 逆向脚本全量在 `legacy/`（本地归档，不进 git），
> 仅作历史参照。正向开发不读 `legacy/`。
