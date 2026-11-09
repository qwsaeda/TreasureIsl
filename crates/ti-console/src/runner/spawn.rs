//! S46.1 整理：后台 spawn **共用实现**（门禁执行面 + 发布面同构抽取）。
//!
//! 抽取自 `runner/gate.rs` 与 `content/publish.rs` 的重复段：
//! 「候选解释器回退 → 日志落盘 → 后台 spawn → 状态文件写 pid/running → 收割线程记退出码」。
//!
//! 口径（两处原本各自实现，现统一在此）：
//! - **执行面最小环境集**（22 号 §10.3 `apply_minimal_env`：`env_clear` + 白名单；解释器依赖 PATH/HOME）；
//! - **解释器回退** `pwsh` → `powershell`（22 号 §4.3；Windows 仅 PowerShell 5.1 时回退）；
//! - **状态文件** = 调用方给的 JSON：spawn 后写入 `pid` + `running=true`；
//!   子进程退出 ⇒ 收割线程更新为 `running=false` + `exit_code` + `ended_ts`
//!   （**信号终止无退出码 = `-1`**，非 0 如实上报硬失败，不静默当成功）；
//! - **收割线程**保证子进程被 `wait` ⇒ 无僵尸残留（平台进程存活期间状态文件始终最新）。

use std::path::Path;

use serde_json::Value;

/// 脚本解释器候选（22 号 §4.3 回退序）。
pub const SCRIPT_INTERPS: [&str; 2] = ["pwsh", "powershell"];

/// 后台 spawn 结果。
#[derive(Debug, Clone, Copy)]
pub struct SpawnOutcome {
    /// 子进程 pid。
    pub pid: u32,
    /// 实际使用的解释器名（`pwsh` / `powershell`）。
    pub interp: &'static str,
}

/// 后台 spawn 一条脚本命令（解释器回退 + 日志落盘 + 收割线程记退出码）。
///
/// # 参数
///
/// - `args`：传给脚本的参数（如 `["-File", "scripts/gate.ps1", "-Fast"]`）；
/// - `cwd`：工作目录（仓库根）；
/// - `log_path`：stdout/stderr 落盘目标（**覆盖**创建）；
/// - `status_path`：状态文件（JSON）路径；
/// - `status_meta`：状态文件初始内容（helper 追加 `pid` / `running`）。
///
/// # Errors
///
/// 日志文件创建失败 / 克隆失败 ⇒ 报错；两个解释器均缺位 ⇒ 报错（附最后一次失败原因）。
pub fn spawn_script_logged(
    args: &[&str],
    cwd: &Path,
    log_path: &Path,
    status_path: &Path,
    status_meta: Value,
) -> Result<SpawnOutcome, String> {
    // S49.7：收割/状态/超时实现**单一来源** = `runner::jobs::spawn_program_logged`
    // （23 号 §6.2 统一作业模型）；本函数保留薄封装语义（解释器回退 + 日志 + 收割）。
    let owned: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
    let outcome = crate::runner::jobs::spawn_program_logged(
        &SCRIPT_INTERPS,
        &owned,
        cwd,
        Some(log_path),
        status_path,
        status_meta,
        None,
        None,
    )?;
    let interp: &'static str = if outcome.program == "powershell" {
        "powershell"
    } else {
        "pwsh"
    };
    Ok(SpawnOutcome { pid: outcome.pid, interp })
}
