//! 领域模型（Domain Model）：游戏业务类型定义的**唯一权威来源**。
//!
//! ## 为什么有这个模块（S11 归档改造）
//!
//! 改造前，全部 27 个业务类型（`Unit`/`Building`/`GridPos`/`SaveSlot`/`Character`/
//! `PriceTable`/…）住在 `crates/ti-core/src/oracle/`——那个名义上「冻结为逆向参照物」的
//! 目录，实际上承载着正式实现 `rules/` 与整个表现层 `ti-app/` 的类型依赖
//! （37 处 `use ti_core::domain::…`）。这是架构倒置：**参照物不应是实现的依赖**。
//!
//! 改造后：
//! - [`domain`]：**类型定义的唯一出处**（纯类型 + 纯函数，零业务编排）；
//! - [`rules`]：业务行为实现（依赖 `domain`，不依赖 `oracle`）；
//! - `oracle`：降级为**纯差分测试基准**（从伪代码直译的行为函数），可整体归档到
//!   `reverse/`，不再被正式实现引用。
//!
//! ## 分层约束（`docs/process/01_architecture.md` §1）
//!
//! ```text
//!   rules/  ──依赖──>  domain/  ──（反向禁止）
//!                         ▲
//!                         │
//!   ti-app/（表现层）───────┘
//!
//!   oracle/（逆向基准，仅 tests/ 可见）
//! ```
//!
//! - `domain` **零依赖**：不依赖 `rules`/`oracle`/`ti-platform`，只依赖 `serde`；
//! - `rules` 只能依赖 `domain`，**禁止**依赖 `oracle`（门禁第 11 项
//!   `check_oracle_boundary.py` 校验）；
//! - `domain` 的语义来源是 `docs/business/`（正向设计），**不是** `docs/reverse/spec/`
//!   （逆向参照）。
//!
//! ## 模块划分
//!
//! | 模块 | 业务域 | 对应业务文档 |
//! |---|---|---|
//! | [`combat`] | 战斗（单位/阵营） | `docs/business/combat.md` |
//! | [`economy`] | 经济（商品/价格/建筑/银行） | `docs/business/economy.md` |
//! | [`map`] | 地图（网格/方向/地形/事件目标） | `docs/business/progression.md` |
//! | [`flow`] | 流程（菜单场景/事件/音效/场景状态） | `docs/business/loop_core.md` |
//! | [`messages`] | 消息（种类/分页） | `docs/business/loop_core.md` §消息 |
//! | [`persistence`] | 持久化（存档槽/错误） | `docs/business/persistence.md` |
//! | [`selection`] | 选择（角色/地图/状态） | `docs/business/progression.md` §选择 |
//! | [`render`] | 渲染资源（容器/引用/图层） | `docs/business/loop_core.md` §资源 |

pub mod combat;
pub mod economy;
pub mod flow;
pub mod map;
pub mod messages;
pub mod persistence;
pub mod render;
pub mod selection;

pub use combat::{Side, Unit};
pub use economy::{Bank, Building, Goods, PriceTable, TradeEvent};
pub use flow::{MenuEvent, MenuScene, SceneId, SceneState, Sfx};
pub use map::{Dir, GridPos, MapTerrain, SceneTarget};
pub use messages::{FlowSource, MsgEvent, MsgKind, MsgPage};
pub use persistence::{MIN_SAVE_SIZE, REC_MAGIC, SaveErr, SaveSlot};
pub use render::{RenderError, RenderPlan, ResourceContainer, ResourceRef, SceneAsset};
pub use selection::{Character, MapSlot, SelectionState};
