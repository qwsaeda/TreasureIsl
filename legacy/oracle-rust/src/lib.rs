//! legacy-oracle：原版参照实现（S11 归档）。
//!
//! 与 `crates/ti-core/src/oracle/` 镜像同源（区别：本 crate 用 `pub use ti_core::`
//! 绝对路径版，供 ti-core/ti-app 的集成测试 `reference_diff_*` 直接 import 做对拍）。
//! 业务规则唯一事实源 = `docs/business/` + `assets/data/*.json`，本 crate 不是业务依据。

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
