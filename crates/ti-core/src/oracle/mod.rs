//! oracle 原版参照实现（S11 归档；S15.8 起为本 crate **唯一**镜像源）。
//!
//! 原参照实现曾物理归档为重复 crate `legacy/oracle-rust/`（`legacy-oracle`，
//! 与本目录逐字节同源、零代码引用），已于 S15.8 连同 workspace member 登记一并删除。
//! 现本目录是单点源：经 lib.rs `pub mod oracle` 公开导出，`rules/` 与集成测试
//! （`reference_diff_*` / `diff_*`）统一 `use ti_core::oracle::`，无 feature 门控。
//!
//! 业务规则唯一事实源 = `docs/business/` + `assets/data/*.json`；oracle 仅作
//! 差分测试参照，不是业务依据。

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
