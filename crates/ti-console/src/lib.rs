//! S32.2 TI Dev Console 共享层（bin `ti-console` + 集成测试双编译单元）。
//!
//! 本 crate 主体是 `main.rs` bin（平台本体），`lib.rs` 只转发 `repo/` 适配层
//! （只读数据源 + 路径白名单 + 快照；0 游戏依赖，`22_dev_platform` §3 边界），
//! 供 `tests/` 集成测试经 `ti_console::repo::*` 引用（gate 硬规则 1：tests/ 目录
//! 允许 unwrap/expect；src 库代码 0 豁免）。
#![allow(dead_code)]

pub mod repo;
