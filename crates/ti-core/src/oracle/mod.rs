//! oracle 桥接层（S11）。
//!
//! 原版参照实现已物理归档至 `legacy/oracle-rust/`（独立 crate `legacy-oracle`，
//! 仅测试编译）。本目录是原版 oracle 代码的**镜像**（`pub use crate::` 版），
//! 使 `rules/` 与集成测试的 `crate::oracle::` 路径不变；两份内容同源，修改须同步。
//!
//! 业务规则唯一事实源 = `docs/business/` + `assets/data/*.json`；oracle 仅作
//! 差分测试参照（`reference_diff_*`），不是业务依据。
//!
//! 注：`legacy-oracle` 为独立 crate（`pub use ti_core::` 版，cfg(test) 门控），
//! 供 `reference_diff_*` 集成测试直接 import，做「rules vs 原版参照」对拍。

#![allow(missing_docs)]
pub mod battle;
pub mod build;
pub mod main_loop;
pub mod main_map;
pub mod menu;
pub mod messages;
pub mod render;
pub mod saveload;
pub mod selection;
pub mod trade;
