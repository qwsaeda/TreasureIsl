//! 开发管理平台共享层（S32.1）：
//! - `ledger`：`plan/ledger.yaml` 解析与校验（从 `crates/ti-tools/src/ledger.rs` 上提，单实现）；
//! - `http`：最小静态 HTTP 服务（以 `crates/ti-tools/src/bin/serve.rs` 为原型重写为库，行为对齐；
//!   `std TcpListener` + 每连接一线程，不引 axum/tokio，定案见
//!   `docs/process/22_dev_platform.md` 第 4.1 节）。
//!
//! 本 crate **0 游戏依赖**（纯 std）：`cargo tree -p ti-console` 依赖图断言不含
//! `ti-core`/`ti-platform`/`ti-app`/`ti-engine`/`ti-render`/`bevy`/`macroquad`。

pub mod http;
pub mod ledger;

pub use ledger::{Report, Status, Task, missing_evidence, validate};
