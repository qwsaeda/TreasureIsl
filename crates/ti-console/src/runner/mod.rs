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
pub mod jobs;
pub mod spawn;

/// 子进程环境变量**最小集**白名单（22 号 §10.3 执行面：`PATH`/`HOME`/`USERPROFILE`/
/// `CARGO_HOME`/`RUSTUP_HOME`/`SYSTEMROOT`/`TEMP`/`TMP`/`APPDATA`/`LOCALAPPDATA`/
/// `XDG_CACHE_HOME`/`TI_CHROMIUM_PATH`/`TI_CONSOLE_TOKEN`/`ANDROID_SERIAL`（adb 目标设备 serial，
/// 多设备在线时 P5 真机判据须明确目标）；其余不透传。
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
    "ANDROID_SERIAL",
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
    // cargo 工具链定位兜底（2026-11-10 S49.7 统一作业模型遗留缺口修复）：
    // 平台进程 spawn 时 PATH 不含 ~/.cargo/bin（nohup/systemd 起平台时 PATH 常缺），
    // 白名单里 CARGO_HOME/RUSTUP_HOME 键若父进程未设 = 跳过 → 子进程 `cargo` 500。
    // 修法：父进程未设 CARGO_HOME/RUSTUP_HOME 时，按 HOME 推断 ~/.cargo / ~/.rustup 补进 PATH，
    // 保证 test/build job 的 `cargo` 命令在任意启动方式下都能定位（Linux；Windows 走 USERPROFILE 同构）。
    if std::env::var("CARGO_HOME").is_err() {
        if let Ok(home) = std::env::var("HOME") {
            let cargo_bin = format!("{home}/.cargo/bin");
            if std::path::Path::new(&cargo_bin).is_dir() {
                // 追加到 PATH（不覆盖已有），子进程 `cargo` 可定位
                let base = std::env::var("PATH").unwrap_or_default();
                let merged =
                    if base.is_empty() { cargo_bin.clone() } else { format!("{base}:{cargo_bin}") };
                cmd.env("PATH", merged);
            }
        }
    }
}
