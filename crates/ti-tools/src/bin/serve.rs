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

/// 解析 `--dir` / `--port`（也接受 `--dir=web`）。
fn parse_args() -> (PathBuf, u16) {
    let mut dir = PathBuf::from("web");
    let mut port = DEFAULT_PORT;
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
            _ => {}
        }
    }
    (dir, port)
}

/// 把 URL 路径安全地映射到磁盘路径（拒绝 `..` 穿越）。
fn resolve(root: &Path, url_path: &str) -> Option<PathBuf> {
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
    out.exists().then_some(out)
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

    let response = match resolve(root, url) {
        Some(path) => match fs::read(&path) {
            Ok(body) => {
                let mime = mime_for(&path);
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let mut out = header.into_bytes();
                out.extend_from_slice(&body);
                out
            }
            Err(_) => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec(),
        },
        None => b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n".to_vec(),
    };
    let _ = stream.write_all(&response);
    let _ = stream.flush();
}

/// 入口。
fn main() {
    let (dir, port) = parse_args();
    if !dir.is_dir() {
        eprintln!("目录不存在：{}", dir.display());
        eprintln!("先跑 scripts/build_wasm.ps1 生成 web/ 产物");
        return;
    }
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("无法监听 127.0.0.1:{port} —— {e}");
            return;
        }
    };
    println!("静态服务已启动：http://127.0.0.1:{port}/  (root = {})", dir.display());
    println!("按 Ctrl+C 退出");
    for stream in listener.incoming().flatten() {
        handle(stream, &dir);
    }
}
