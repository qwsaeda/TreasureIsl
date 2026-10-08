//! 逆向与资源管线工具。
//!
//! 当前已落地：循环台账校验（`gate.ps1` 第 8 项）。解包/转换类工具随
//! S1 各任务加入，加入前本 crate 不放占位实现。

pub mod ledger;

pub use ledger::{Report, Status, Task, missing_evidence, validate};
