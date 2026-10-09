//! 逆向与资源管线工具。
//!
//! 当前已落地：循环台账校验（`gate.ps1` 第 8 项）与文档一致性校验（第 9 项）。
//! 解包/转换类工具随 S1 各任务加入，加入前本 crate 不放占位实现。

pub mod doccheck;
pub mod hash;
pub mod ledger;

pub use doccheck::{
    Doc, NumericAssertion, Report as DocReport, check, dod_keywords, evidence_is_gate_only,
};
pub use ledger::{Report, Status, Task, missing_evidence, validate};
