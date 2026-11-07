//! 白名单命令执行（S32.4 起）。
//!
//! - `gate`：门禁执行（`POST /api/gate`，22 号 §4.2 白名单 id `gate`）
//!   = `pwsh -File scripts/gate.ps1` + tier∈{fast,touch,full} → `-Fast`/`-Touch`/无开关；
//!   后台 spawn + 单实例锁（`evidence/console/gate.lock`）+ 轮询日志 + 历史。
//!
//! 与 `repo/` 不同：`repo/` 是只读数据源适配；`runner/` 是**写面**（受控执行），
//! 程序只来自 `console_commands.json` 白名单（禁自由字符串透传，22 号 §4.2）。

pub mod build;
pub mod close;
pub mod gate;
pub mod spawn;

/// 子进程环境变量**最小集**白名单（22 号 §10.3 执行面：`PATH`/`HOME`/`USERPROFILE`/
/// `CARGO_HOME`/`RUSTUP_HOME`/`SYSTEMROOT`/`TEMP`/`TMP`/`APPDATA`/`LOCALAPPDATA`/
/// `XDG_CACHE_HOME`/`TI_CHROMIUM_PATH`/`TI_CONSOLE_TOKEN`；其余不透传。
/// `gate.ps1` 依赖 `PATH`/`HOME` 定位 cargo；node / Playwright 依赖 `SYSTEMROOT`/`TEMP`/`APPDATA`/`XDG_CACHE_HOME`）。
pub const MINIMAL_ENV_KEYS: &[&str] = &[
    "PATH",
    "HOME",
    "USERPROFILE",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "SYSTEMROOT",
    "TEMP",
    "TMP",
    "APPDATA",
    "LOCALAPPDATA",
    "XDG_CACHE_HOME",
    "TI_CHROMIUM_PATH",
    "TI_CONSOLE_TOKEN",
];

/// 对 `Command` 应用最小环境集（22 号 §10.3：`env_clear()` + 白名单逐键透传）。
/// 平台按 OS 取交集（Windows 侧忽略 POSIX 专有项，反之亦然）；键不在父环境 = 跳过。
pub fn apply_minimal_env(cmd: &mut std::process::Command) {
    cmd.env_clear();
    for k in MINIMAL_ENV_KEYS {
        if let Ok(v) = std::env::var(k) {
            cmd.env(k, v);
        }
    }
}
