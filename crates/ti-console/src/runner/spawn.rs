//! S46.1 整理：脚本解释器常量（22 号 §4.3 回退序）。
//!
//! S49.7（23 号 §6.2 统一作业模型）起，后台 spawn / 状态文件 / 收割线程的**单一实现**
//! = [`crate::runner::jobs::spawn_program_logged`]；本模块只保留解释器候选常量
//! `SCRIPT_INTERPS`（`pwsh` → `powershell`，Windows 仅 PowerShell 5.1 时回退）。
//!
//! 口径（原 S46.1 抽取，现随统一作业模型落在 `jobs.rs`）：
//! - **执行面最小环境集**（22 号 §10.3 `apply_minimal_env`：`env_clear` + 白名单；解释器依赖 PATH/HOME）；
//! - **状态文件** = 调用方给的 JSON：spawn 后写入 `pid` + `running=true`；
//!   子进程退出 ⇒ 收割线程更新为 `running=false` + `exit_code` + `ended_ts`
//!   （**信号终止无退出码 = `-1`**，非 0 如实上报硬失败，不静默当成功）；
//! - **收割线程**保证子进程被 `wait` ⇒ 无僵尸残留（平台进程存活期间状态文件始终最新）。

/// 脚本解释器候选（22 号 §4.3 回退序）。
pub const SCRIPT_INTERPS: [&str; 2] = ["pwsh", "powershell"];
