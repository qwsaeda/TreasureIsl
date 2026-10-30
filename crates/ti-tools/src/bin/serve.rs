//! S2.5 预览服务器：静态托管 `web/`（wasm 需 HTTP，不能 `file://`）。
//!
//! 用法：`cargo run -p ti-tools --bin serve -- --dir web --port 8080`
//! 然后浏览器打开 <http://localhost:8080/>。
//!
//! 只做最小静态服务：目录索引 + MIME + wasm 的
//! `application/wasm`，不引入任何依赖。

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

/// 默认监听端口。
const DEFAULT_PORT: u16 = 8080;

/// 按扩展名给 MIME 类型（wasm 必须正确，否则浏览器拒绝流式编译）。
fn mime_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

/// 解析 `--dir` / `--port` / `--host`（也接受 `--dir=web` / `--host=0.0.0.0`）。
///
/// `--host` 缺省 `0.0.0.0`（同内网跨机访问；仅本机可用 `--host 127.0.0.1`）。
fn parse_args() -> (PathBuf, u16, String) {
    let mut dir = PathBuf::from("web");
    let mut port = DEFAULT_PORT;
    let mut host = "0.0.0.0".to_owned();
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
            "--dir" => {
                if let Some(v) = value {
                    dir = PathBuf::from(v);
                }
            }
            "--port" => {
                if let Some(v) = value {
                    port = v.parse().unwrap_or(DEFAULT_PORT);
                }
            }
            "--host" => {
                if let Some(v) = value {
                    host = v;
                }
            }
            _ => {}
        }
    }
    (dir, port, host)
}

/// 路径映射：`..` 穿越 = None（400 拒绝）；缺失文件 = Some(path)（读不到由调用方 404）。
fn resolve_or_404(root: &Path, url_path: &str) -> Option<PathBuf> {
    let clean = url_path.split(['?', '#']).next().unwrap_or("/");
    let rel = clean.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    let mut out = root.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return None;
        }
        out.push(part);
    }
    if out.is_dir() {
        out.push("index.html");
    }
    Some(out)
}

/// 处理单个连接。
fn handle(mut stream: TcpStream, root: &Path) {
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let mut parts = line.split_whitespace();
    let _method = parts.next().unwrap_or("");
    let url = parts.next().unwrap_or("/");

    // 路径映射成功（含缺失文件）→ 读不到 = 404（bevy 资产 `.meta` 缺失的正常语义）；
    // 映射失败（`..` 穿越 / 非法路径）→ 400 拒绝。
    // S31.2c：响应头带 `Cache-Control: no-store`（开发期 serve，防浏览器拿旧 wasm/胶水
    // 造成「改了代码但页面还是旧行为」——2026-10-29 用户报「选完地图不能玩」排查确认
    // 静态资源缓存是首要嫌疑；生产部署用带 fingerprint 的 CDN，不用本 serve）。
    let response = match resolve_or_404(root, url) {
        Some(path) => match fs::read(&path) {
            Ok(body) => {
                let mime = mime_for(&path);
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n\
                     Cache-Control: no-store\r\n\r\n",
                    body.len()
                );
                let mut out = header.into_bytes();
                out.extend_from_slice(&body);
                out
            }
            Err(_) => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\
                       Cache-Control: no-store\r\n\r\n"
                .to_vec(),
        },
        None => b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\
                  Cache-Control: no-store\r\n\r\n"
            .to_vec(),
    };
    let _ = stream.write_all(&response);
    let _ = stream.flush();
}

/// 入口。
fn main() {
    let (dir, port, host) = parse_args();
    if !dir.is_dir() {
        eprintln!("目录不存在：{}", dir.display());
        eprintln!("先跑 scripts/build_wasm.ps1 生成 web/ 产物");
        return;
    }
    let bind_addr = format!("{host}:{port}");
    let listener = match TcpListener::bind(&bind_addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("无法监听 {bind_addr} —— {e}");
            return;
        }
    };
    let display_host = if host == "0.0.0.0" {
        // 0.0.0.0 = 全网卡；提示本机 IP（同内网访问用）。`/proc` 仅在 Linux 存在：
        // 可读 = Linux 环境 → 取 `hostname -I` 首个 IP；否则（Windows/macOS）回退 127.0.0.1。
        if std::fs::read_to_string("/proc/net/if_inet6").is_ok() {
            let out = std::process::Command::new("hostname")
                .arg("-I")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| s.split_whitespace().next().map(str::to_owned));
            out.unwrap_or_else(|| "127.0.0.1".to_owned())
        } else {
            "127.0.0.1".to_owned()
        }
    } else {
        host.clone()
    };
    println!("静态服务已启动：http://{display_host}:{port}/  (root = {})", dir.display());
    println!("按 Ctrl+C 退出");
    for stream in listener.incoming().flatten() {
        handle(stream, &dir);
    }
}
