//! TI Dev Console 入口（S32.1 骨架 + S32.2 只读数据源 API）。
//!
//! 纯开发管理平台（与游戏运行时解耦，定案见 `docs/process/22_dev_platform.md`）：
//! - 静态壳（`web/`，原生 JS+CSS，无 npm 构建）+ `/api/health`；
//! - S32.2：只读数据源 API（`/api/overview` `/api/tasks` `/api/evidence` `/api/assets`
//!   `/api/tests` `/api/session`，`repo/` 适配层 + 路径白名单 + canonicalize 防穿越 + 0 写盘）；
//! - 默认监听 `0.0.0.0:8787`（局域网可访问；**本机开发工具，不启用鉴权**，22 号第 10.1 节）；
//!   用户 2026-11-06 拍板「必须局域可以访问」⇒ 默认绑定全接口，`--host 127.0.0.1` 可收窄回环。
//! - 单实例锁（`evidence/console/.lock`）；
//! - 命令白名单 = `console_commands.json`（机器真源）；数据源 = `console_sources.json`。
//!
//! 用法：`cargo run -p ti-console`（S32.1 起）。

mod audit;
mod content;
mod netaddr;
mod repo;
mod routes;
mod runner;

use std::io::Write;
use std::net::TcpListener;
use std::path::Path;

/// 默认监听地址（非回环须显式 `--host`；本机开发工具不启用鉴权，22 号第 10.1 节）。
const DEFAULT_BIND: &str = "0.0.0.0:8787";

/// 平台进程锁陈旧接管阈值（22 号 §10.4：起始超 30 分钟 + pid 不存在 ⇒ 接管）。
const LOCK_STALE_SECS: u64 = 1800;

/// 单实例锁文件路径（`evidence/console/.lock`；存在且非陈旧 = 拒启动，避免多实例端口冲突）。
fn lock_path() -> std::path::PathBuf {
    Path::new("evidence/console/.lock").to_path_buf()
}

/// 进程存活（Linux `/proc/<pid>` 存在；非 Linux 退化为「锁存在即活」由 `LockGuard` `Drop` 清锁）。
fn process_alive(pid: u32) -> bool {
    if cfg!(target_os = "linux") {
        return Path::new(&format!("/proc/{pid}")).exists();
    }
    true
}

/// 写单实例锁（JSON：`pid` + `started_ts` + `action`）。
/// 陈旧接管（22 号 §10.4 S32.4 并发面补全）：锁内 pid 不存在 **且** 起始超 30 分钟 ⇒
/// 接管并记 `stale-lock`（进程被 kill -9/崩溃后不残留阻启）；否则拒启动。
fn write_lock(pid: u32) -> Result<std::path::PathBuf, String> {
    let path = lock_path();
    let dir = path.parent().ok_or_else(|| "锁路径缺父目录".to_owned())?;
    std::fs::create_dir_all(dir).map_err(|e| format!("无法建 {}：{e}", dir.display()))?;
    if path.is_file() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        // 兼容旧纯 pid 锁（S32.1 骨架期）与 JSON 锁（S32.4 起：pid + started_ts + action）。
        let parsed = serde_json::from_str::<serde_json::Value>(&content).ok();
        let old_pid = parsed
            .as_ref()
            .and_then(|v| v.get("pid").and_then(serde_json::Value::as_u64))
            .and_then(|p| u32::try_from(p).ok())
            .or_else(|| content.trim().parse::<u32>().ok());
        let started = parsed
            .as_ref()
            .and_then(|v| v.get("started_ts").and_then(serde_json::Value::as_u64))
            .unwrap_or(0);
        if let Some(old) = old_pid {
            if old != pid {
                let alive = process_alive(old);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                // 进程不存在 ⇒ 锁必然无效（进程被杀/崩溃），**立即接管**，不等 30 分钟
                // （旧判定要求「pid 不存在 且 超 30 分钟」才接管 ⇒ 崩后重启必被拦，平台起不来）。
                // 30 分钟门槛只用于「pid 仍存活但疑似卡死」的情形。
                if !alive {
                    eprintln!("ti-console: 陈旧锁接管（pid {old} 已不存在；stale-lock）");
                    let _ = std::fs::remove_file(&path);
                } else if now.saturating_sub(started) > LOCK_STALE_SECS {
                    eprintln!("ti-console: 陈旧锁接管（pid {old} 仍存活但超 30 分钟；stale-lock）");
                    let _ = std::fs::remove_file(&path);
                } else {
                    return Err(format!(
                        "单实例锁被占用（pid {old}，当前 {pid}）；先删 {} 再启动",
                        path.display()
                    ));
                }
            }
        } else {
            return Err(format!(
                "锁文件 {} 内容非法（期望 pid 或 JSON）；先删再启动",
                path.display()
            ));
        }
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let lock = serde_json::json!({
        "pid": pid,
        "started_ts": ts,
        "action": "platform-serve",
    });
    let mut f = std::fs::File::create(&path).map_err(|e| format!("无法写锁文件：{e}"))?;
    writeln!(f, "{lock}").map_err(|e| format!("写锁文件失败：{e}"))?;
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
    let host = host_opt.unwrap_or_else(|| bind.split(':').next().unwrap_or("0.0.0.0").to_owned());
    let p = port
        .unwrap_or_else(|| bind.rsplit(':').next().and_then(|s| s.parse().ok()).unwrap_or(8787));
    format!("{host}:{p}")
}

/// 绑定监听套接字：默认 IPv6 双栈 `[::]:port`（同收 v4），IPv6 不可用则退回 IPv4 通配。
///
/// 返回 (listeners, 人类可读描述)。显式 `--host` 时按给定 host 绑单一 socket。
fn bind_listeners(bind: &str, port: u16) -> (Vec<TcpListener>, String) {
    let host = bind.split(':').next().unwrap_or("0.0.0.0");
    let explicit = !matches!(host, "" | "0.0.0.0" | "::" | "[::]");
    let hosts: Vec<String> = if explicit {
        vec![host.to_owned()]
    } else {
        // 先试 IPv6 双栈，再试 IPv4 通配；双栈可用时同一端口即可服务 v4+v6。
        let v6 = format!("[::]:{port}");
        let v4 = format!("0.0.0.0:{port}");
        match TcpListener::bind(&v6) {
            Ok(listener) => return (vec![listener], format!("IPv6 双栈 [::]:{port}（同收 IPv4）")),
            Err(_) => vec![v4],
        }
    };
    let mut listeners = Vec::new();
    let mut desc = Vec::new();
    for h in hosts {
        let addr = if h.contains(':') && !h.starts_with('[') {
            format!("[{h}]:{port}")
        } else {
            format!("{h}:{port}")
        };
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                desc.push(addr.clone());
                listeners.push(listener);
            }
            Err(e) => eprintln!("ti-console: 无法监听 {addr} —— {e}"),
        }
    }
    if listeners.is_empty() {
        eprintln!("ti-console: 无可用监听地址（{bind}）");
        std::process::exit(1);
    }
    (listeners, desc.join(" + "))
}

fn main() {
    let pid = std::process::id();
    let bind = parse_args();

    let lock = match write_lock(pid) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("ti-console: {e}");
            std::process::exit(1);
        }
    };
    let guard = LockGuard { path: lock };

    let web_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
    // 监听面（S87.5 追加）：默认绑 **IPv6 双栈** `[::]:port`（Linux bindv6only=0 ⇒ 同一 socket
    // 同收 IPv4 连接）⇒ 一份监听同时服务 v4 + v6；若该机 IPv6 不可用则退回 IPv4 通配 `0.0.0.0:port`。
    // 显式 `--host` 指定时按用户给定的 host 绑（不再自动双栈）。
    let port: u16 = bind.rsplit(':').next().and_then(|s| s.parse().ok()).unwrap_or(8787);
    let (listeners, listen_desc) = bind_listeners(&bind, port);
    let (v4_addrs, v6_addrs) = netaddr::local_addresses();

    println!("ti-console 已启动（监听 {listen_desc}，root = {}）", web_root.display());
    if v4_addrs.is_empty() && v6_addrs.is_empty() {
        println!("  可访问地址：未探测到非回环网卡地址（仅本机回环可用）");
    }
    for ip in &v4_addrs {
        println!("  IPv4  → {}", netaddr::url_for(ip, port));
    }
    for ip in &v6_addrs {
        println!("  IPv6  → {}", netaddr::url_for(ip, port));
    }
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
    // 每连接一线程 + 并发处理（2026-11-09 修复）：原实现 `std::thread::scope` 放在 for 循环**内**，
    // 每 accept 一个连接就 scope 阻塞等该连接处理完才 accept 下一个 ⇒ 任一客户端连接
    // （浏览器 keep-alive 空闲连接 / 慢客户端不发数据）在 `read_line` 长占即冻结整个 accept 循环，
    // 用户报「/api/build 构建日志不实时更新」真根因（慢连接复现实测：curl 5s 超时 0 字节）。
    // 修复 = 单 scope 包住 accept 循环：连接并行（互不阻塞），scope 语义不变（线程不逃逸 main）。
    std::thread::scope(|s| {
        for listener in &listeners {
            let web_root = web_root.clone();
            let repo_root = repo_root.clone();
            s.spawn(move || {
                for stream in listener.incoming().flatten() {
                    let web_root = web_root.clone();
                    let repo_root = repo_root.clone();
                    s.spawn(move || routes::serve(stream, &web_root, &repo_root));
                }
            });
        }
    });
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
