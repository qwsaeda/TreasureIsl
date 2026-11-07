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
use std::process::{Command, Stdio};

use serde_json::{Value, json};

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
    let log_file = std::fs::File::create(log_path).map_err(|e| format!("日志文件创建失败 {e}"))?;
    let mut last_err = String::new();
    for interp in SCRIPT_INTERPS {
        let mut cmd = Command::new(interp);
        cmd.args(args);
        cmd.current_dir(cwd);
        // 22 号 §10.3 最小环境集（env_clear + 白名单）。
        crate::runner::apply_minimal_env(&mut cmd);
        let stdout = log_file.try_clone().map_err(|e| format!("日志克隆失败 {e}"))?;
        let stderr = log_file.try_clone().map_err(|e| format!("日志克隆失败 {e}"))?;
        cmd.stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                // 状态文件：pid + running=true（收割线程随后写 running=false + exit_code）。
                let mut meta = status_meta;
                meta["pid"] = json!(pid);
                meta["running"] = json!(true);
                let _ = std::fs::write(status_path, meta.to_string());
                // 收割线程：等退出 → 记退出码（子进程被 wait ⇒ 无僵尸残留）。
                let meta_path = status_path.to_path_buf();
                std::thread::spawn(move || {
                    let code = child.wait().ok().and_then(|s| s.code()).unwrap_or(-1);
                    let ended_ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs());
                    let mut done = meta;
                    done["running"] = json!(false);
                    done["exit_code"] = json!(code);
                    done["ended_ts"] = json!(ended_ts);
                    let _ = std::fs::write(&meta_path, done.to_string());
                });
                return Ok(SpawnOutcome { pid, interp });
            }
            Err(e) => last_err = format!("{interp}: {e}"),
        }
    }
    Err(format!("后台 spawn 失败（pwsh/powershell 均缺位？）：{last_err}"))
}
