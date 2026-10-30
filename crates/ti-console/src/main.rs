//! TI Dev Console 入口（S32.1 骨架 + S32.2 只读数据源 API）。
//!
//! 纯开发管理平台（与游戏运行时解耦，定案见 `docs/process/22_dev_platform.md`）：
//! - 静态壳（`web/`，原生 JS+CSS，无 npm 构建）+ `/api/health`；
//! - S32.2：只读数据源 API（`/api/overview` `/api/tasks` `/api/evidence` `/api/assets`
//!   `/api/session`，`repo/` 适配层 + 路径白名单 + canonicalize 防穿越 + 0 写盘）；
//! - 默认监听 `127.0.0.1:8787`（非回环须显式 `--host` + token，22 号第 10.1 节）；
//! - 单实例锁（`evidence/console/.lock`）；
//! - 命令白名单 = `console_commands.json`（机器真源）；数据源 = `console_sources.json`。
//!
//! 用法：`cargo run -p ti-console`（S32.1 起）。

mod repo;
mod routes;

use std::io::Write;
use std::net::TcpListener;
use std::path::Path;

/// 默认监听地址（非回环须显式 `--host`，鉴权负例见 22 号第 10.1 节）。
const DEFAULT_BIND: &str = "127.0.0.1:8787";

/// 单实例锁文件路径（`evidence/console/.lock`；存在即拒启动，避免多实例端口冲突）。
fn lock_path() -> std::path::PathBuf {
    Path::new("evidence/console/.lock").to_path_buf()
}

/// 写单实例锁（锁文件 = 当前 pid；已存在且 pid 不同 = 拒启动）。
fn write_lock(pid: u32) -> Result<std::path::PathBuf, String> {
    let binding = lock_path();
    let dir = binding.parent().ok_or_else(|| "锁路径缺父目录".to_owned())?;
    std::fs::create_dir_all(dir).map_err(|e| format!("无法建 {}：{e}", dir.display()))?;
    let path = lock_path();
    if path.is_file() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let Ok(old) = content.trim().parse::<u32>() else {
            return Err(format!("锁文件 {} 内容非法（期望 pid）；先删再启动", path.display()));
        };
        if old != pid {
            return Err(format!(
                "单实例锁被占用（pid {old}，当前 {pid}）；先删 {} 再启动",
                path.display()
            ));
        }
    }
    let mut f = std::fs::File::create(&path).map_err(|e| format!("无法写锁文件：{e}"))?;
    writeln!(f, "{pid}").map_err(|e| format!("写锁文件失败：{e}"))?;
    Ok(path)
}

/// 解析 `--host` / `--port`（接受 `--host=...` / `--host ...` 两种写法）。
fn parse_args() -> String {
    let mut port: Option<u16> = None;
    let mut host_opt: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let (key, inline) = match a.split_once('=') {
            Some((k, v)) => (k.to_owned(), Some(v.to_owned())),
            None => (a.clone(), None),
        };
        let value = match inline {
            Some(v) => Some(v),
            None => args.next(),
        };
        match key.as_str() {
            "--host" => host_opt = value,
            "--port" => {
                if let Some(v) = value {
                    port = v.parse().ok();
                }
            }
            _ => {}
        }
    }
    let bind = DEFAULT_BIND.to_owned();
    let host = host_opt.unwrap_or_else(|| bind.split(':').next().unwrap_or("127.0.0.1").to_owned());
    let p = port
        .unwrap_or_else(|| bind.rsplit(':').next().and_then(|s| s.parse().ok()).unwrap_or(8787));
    format!("{host}:{p}")
}

fn main() {
    let pid = std::process::id();
    let bind = parse_args();

    // 非回环鉴权（22 号第 10.1 节）：--host 非 127.0.0.1 时**警告**（默认不启用强鉴权，
    // 便于内网开发机直接访问）；设了 TI_CONSOLE_TOKEN 时 = 写操作面（S32.4 起）的 token 校验。
    let host_part = bind.split(':').next().unwrap_or("");
    if !matches!(host_part, "127.0.0.1" | "localhost") && std::env::var("TI_CONSOLE_TOKEN").is_err()
    {
        eprintln!(
            "ti-console: 非回环监听（{host_part}）未设 TI_CONSOLE_TOKEN —— 只读面可用，写操作面（S32.4 起）将 403；生产部署须设 token"
        );
    }

    let lock = match write_lock(pid) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("ti-console: {e}");
            std::process::exit(1);
        }
    };
    let guard = LockGuard { path: lock };

    let web_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
    let listener = match TcpListener::bind(&bind) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("ti-console: 无法监听 {bind} —— {e}");
            std::process::exit(1);
        }
    };

    println!("ti-console 已启动：http://{bind}/  (root = {})", web_root.display());
    println!("按 Ctrl+C 退出（单实例锁 = {}）", guard.path.display());

    // S32.2：仓库根 = CARGO_MANIFEST_DIR 上两级（**隐含假设 = crate 位于 `<repo>/crates/<crate>`**；
    // 目录结构重组〔crate 移出 crates/〕须同步改本段，22 号 §10.2 路径面兜底 = 启动自检）。
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(repo_root) = manifest_dir.parent().and_then(std::path::Path::parent) else {
        eprintln!("[ti-console] 仓库根推导失败（CARGO_MANIFEST_DIR 必在 crates/<crate>/ 下）");
        std::process::exit(1);
    };
    let repo_root = repo_root.to_path_buf();
    // 启动自检（NEW-3 兜底，非 panic 面）：仓库根标志文件 = 台账单一事实源必在。
    if !repo_root.join("plan/ledger.yaml").is_file() {
        eprintln!(
            "ti-console: 仓库根自检失败（{} 下无 plan/ledger.yaml）；目录结构假设（crate 在 crates/ 下）已断裂，拒绝启动",
            repo_root.display()
        );
        std::process::exit(1);
    }
    for stream in listener.incoming().flatten() {
        let web_root = web_root.clone();
        let repo_root = repo_root.clone();
        std::thread::scope(|s| {
            s.spawn(|| routes::serve(stream, &web_root, &repo_root));
        });
    }
}

/// 锁文件守卫：进程正常退出时删锁（Ctrl+C 终止场景由下次启动时的 pid 存活检测兜底，S32.4 补全）。
struct LockGuard {
    path: std::path::PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
