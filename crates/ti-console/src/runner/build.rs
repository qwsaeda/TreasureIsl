//! 构建只读体检 runner（S32.9）：`GET /api/build?check=1`。
//!
//! 只跑 `scripts/build_wasm.ps1 -Check`（**不构建/不拷贝/不写 evidence**，22 号 §9 只读体检口径）
//! + 0 游戏产物变更断言（22 号 §8 冻结路径内容级 sha256 指纹差为空；增量口径，会话相对）。
//!
//! 冻结路径 = `crates/ti-core crates/ti-app crates/ti-engine crates/ti-render crates/ti-platform
//! crates/ti-formats assets/ android/ tests/`（`repo::snapshot::FrozenSnapshot::take` 采集，
//! sha256 逐文件）。`pwsh` 子进程走 `apply_minimal_env`（22 号 §10.3 最小环境集）。

use std::path::Path;

use serde_json::{Value, json};

use crate::repo::config::RepoSources;
use crate::repo::snapshot::FrozenSnapshot;

/// `/api/build?check=1`：只读体检 + 0 游戏产物变更断言。
///
/// # Errors
///
/// `pwsh` 执行失败（-Check 只读体检，pwsh 缺位）= `Err`（带原因，数据源面如实上报）。
pub fn build_check(repo_root: &Path, _sources: &RepoSources) -> Result<Value, String> {
    let before = FrozenSnapshot::take(repo_root);
    let mut cmd = std::process::Command::new("pwsh");
    cmd.arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(repo_root.join("scripts/build_wasm.ps1"))
        .arg("-Check");
    // 22 号 §10.3 最小环境集（env_clear + 白名单；PATH 透传，-Check 只读体检可跑）。
    crate::runner::apply_minimal_env(&mut cmd);
    let out = cmd
        .output()
        .map_err(|e| format!("pwsh 执行失败（build_wasm.ps1 -Check 只读体检）：{e}"))?;
    let check_output =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let after = FrozenSnapshot::take(repo_root);
    let diff = before.diff(&after);
    Ok(json!({
        "ok": out.status.success() && diff.is_empty(),
        "check_exit": out.status.code().unwrap_or(-1),
        "check_output": check_output.chars().take(2000).collect::<String>(),
        "frozen_diff": diff,
        "frozen_diff_empty": before.no_change(&after),
        "note": "只读体检：build_wasm.ps1 -Check（不构建/不拷贝/不写 evidence）+ 冻结路径内容级 sha256 指纹差为空（22 号 §8 增量口径）",
    }))
}
