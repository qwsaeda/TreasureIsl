//! 最小静态 HTTP 服务（以 `crates/ti-tools/src/bin/serve.rs` 为原型重写为库，行为对齐）。
//!
//! `std TcpListener` + 每连接一线程（S32.1 定案，`docs/process/22_dev_platform.md`
//! 第 4.1 节：不引 axum/tokio）。只做静态文件映射 + MIME + `..` 穿越拒绝（400）/缺失文件（404），
//! 响应头带 `Cache-Control: no-store`（同 S31.2c 开发期 serve 口径，防浏览器缓存旧产物）。

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

/// 按扩展名给 MIME 类型（wasm 必须正确，否则浏览器拒绝流式编译）。
#[must_use]
pub fn mime_for(path: &Path) -> &'static str {
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

/// 路径映射：`..` 穿越 = `None`（400 拒绝）；缺失文件 = `Some(path)`（读不到由调用方 404）。
#[must_use]
pub fn resolve_or_404(root: &Path, url_path: &str) -> Option<PathBuf> {
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

/// 处理单个连接（同步，跑在独立线程里）。
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

/// 配置（`run` 的入参，库化后不读 `std::env`）。
pub struct Config {
    /// 静态根目录。
    pub root: PathBuf,
    /// 监听地址（`host:port`，默认 `127.0.0.1:8787`）。
    pub bind: String,
}

impl Config {
    /// 默认监听地址（S32.1：默认 `127.0.0.1`，非回环须显式 `--host` + token，见 22 号 §10.1）。
    pub const DEFAULT_BIND: &'static str = "127.0.0.1:8787";
}

/// 启动静态服务（阻塞，每连接起一线程）。
///
/// # Errors
///
/// 返回 `Err` = 根目录缺失或监听绑定失败。
pub fn run(cfg: &Config) -> Result<(), String> {
    if !cfg.root.is_dir() {
        return Err(format!("目录不存在：{}", cfg.root.display()));
    }
    let listener =
        TcpListener::bind(&cfg.bind).map_err(|e| format!("无法监听 {} —— {e}", cfg.bind))?;
    let root = cfg.root.clone();
    for stream in listener.incoming().flatten() {
        let root = root.clone();
        thread::spawn(move || handle(stream, &root));
    }
    Ok(())
}
