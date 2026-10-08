//! 玩法核心：数值、规则、场景状态机。
//!
//! 硬约束（`docs/style.md` §5.1）：**不依赖任何平台/IO 后端**，只依赖
//! [`ti_formats`]，因此可以在无窗口环境跑差分测试与全流程回归。

pub mod consts;

pub use consts::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
