//! HTTP 请求处理（S32.1 骨架 + S32.2 只读数据源 API + S32.3 看板 UI 静态面
//! （设计图/地图页增量，左菜单 7 页））。
//!
//! - `/api/health` = 200 文本（骨架面）；
//! - `/api/overview` `/api/tasks` `/api/evidence`（含 `?sub=<dir>` 子目录枚举，
//!   设计图页 `sub=ui_concept`）`/api/assets` `/api/session` `/api/audit`
//!   `/api/map` = 只读数据源适配（`repo::data`，22 号 §5/§10.2；路径全过 `resolve_guarded`
//!   白名单 + canonicalize 防穿越；0 写盘）；
//! - S32.3 静态面（`web/`）= 看板 UI v1（左菜单分组 7 页 = 概览/任务/证据/资产/设计图/地图/设置；
//!   原生 JS + CSS 全内联无 npm；视觉令牌 = 游戏「航海夜·精修」，内联于 `index.html` `<style>`
//!   （对齐 `docs/process/10_ui_redesign.md` §3 设计令牌 v2；单文件交付，0 外部静态资源依赖）；
//!   图片（证据/资产/设计稿/地图描述符资产）走 `/` 仓库白名单直读，网页内直查 + 灯箱）；
//! - 其余 = 静态映射（`ti-devmeta::http::resolve_or_404`，400/404 语义同 serve.rs 原型）；
//! - 写操作（POST）骨架期走静态映射 ⇒ 非 403；POST 鉴权 + 403 语义 = S32.4 面（22 号 §10.1）。

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;

use serde_json::Value;

use crate::repo::config::{RepoSources, load as load_sources_file};

use crate::repo::data;
use crate::repo::guard::{PathGuardError, resolve_guarded};
use ti_devmeta::http::{mime_for, resolve_or_404};

/// 每连接一次的数据源清单加载（缺文件/非法 JSON = 该请求 500，不阻断静态面）。
fn load_sources_opt(repo_root: &Path) -> Option<crate::repo::config::RepoSources> {
    load_sources_file(repo_root).ok()
}

/// 处理一个连接（每连接一线程；`web_root` = 静态壳，`repo_root` = 仓库根只读数据源面）。
pub fn serve(mut stream: TcpStream, web_root: &Path, repo_root: &Path) {
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
    let clean = url.split(['?', '#']).next().unwrap_or("/");

    let response = match clean {
        "/api/health" => text200("ti-console ok"),
        "/api/session" => json200(&data::session()),
        "/api/whitelist" => {
            let query = url.split('?').nth(1).unwrap_or("");
            let path_param = query.strip_prefix("path=").unwrap_or("");
            json200(&data::whitelist_probe(repo_root, path_param))
        }
        "/api/audit" => api_500_or_json(load_json(repo_root, |s| data::audit_list(repo_root, s))),
        "/api/map" => {
            api_500_or_json(load_json(repo_root, |s| data::map_packs_detail(repo_root, s)))
        }
        "/api/snapshot" => {
            json200(&data::snapshot(repo_root, &load_sources_opt(repo_root).unwrap_or_default()))
        }
        "/api/overview" => api_500_or_json(load_json(repo_root, |s| data::overview(repo_root, s))),
        "/api/tasks" => api_500_or_json(load_json(repo_root, |s| data::tasks(repo_root, s))),
        "/api/evidence" => {
            let query = url.split('?').nth(1).unwrap_or("");
            let sub = query.strip_prefix("sub=").unwrap_or("").to_owned();
            api_500_or_json(load_json(repo_root, |s| data::evidence_list(repo_root, s, &sub)))
        }
        "/api/assets" => api_500_or_json(load_json(repo_root, |s| data::assets(repo_root, s))),
        _ => static_or_repo_file(web_root, clean, repo_root),
    };

    let _ = stream.write_all(&response);
    let _ = stream.flush();
}

/// 有数据源清单 = 跑端点；无清单 = 500（数据源面不可用，如实上报）。
fn load_json(
    repo_root: &Path,
    f: impl FnOnce(&RepoSources) -> Result<Value, String>,
) -> Result<Value, String> {
    match load_sources_opt(repo_root) {
        Some(s) => f(&s),
        None => Err("console_sources.json 不可读（数据源面 500）".to_owned()),
    }
}

fn api_500_or_json(result: Result<Value, String>) -> Vec<u8> {
    match result {
        Ok(value) => json200(&value),
        Err(err) => {
            let body = format!(r#"{{"error":"{}"}}"#, err.replace('"', "\\\""));
            header_body("application/json", body.as_bytes(), "500 Internal Server Error")
        }
    }
}

fn text200(body: &str) -> Vec<u8> {
    let bytes = body.as_bytes();
    header_body("text/plain", bytes, "200 OK")
}

fn json200(value: &Value) -> Vec<u8> {
    let bytes = value.to_string().into_bytes();
    header_body("application/json", &bytes, "200 OK")
}

/// 静态映射；静态缺失时回退数据源白名单内的仓库文件直读（`/plan/ledger.yaml` 等）。
fn static_or_repo_file(web_root: &Path, clean: &str, repo_root: &Path) -> Vec<u8> {
    if clean.starts_with("/api/") {
        // S32.2 未覆盖的 /api/* = 404（无该端点；写操作 POST = S32.4）。
        return header_body("text/plain", b"404: unknown api endpoint", "404 Not Found");
    }
    if let Some(path) = resolve_or_404(web_root, clean) {
        if let Ok(body) = std::fs::read(&path) {
            let mime = mime_for(&path);
            return header_body(mime, &body, "200 OK");
        }
        if let Ok(v) = guarded_file(repo_root, clean) {
            let mime = mime_for(&v.0);
            return header_body(mime, &v.1, "200 OK");
        }
        return header_body("text/plain", b"404: not found", "404 Not Found");
    }
    // `..` 穿越 = 400 拒绝（与 22 号 §10.2 + guard::check_relative 同口径）。
    if clean.contains("..") {
        return header_body("text/plain", b"400: path traversal rejected", "400 Bad Request");
    }
    // 静态面缺失 + 白名单面也拒 = 404。
    header_body("text/plain", b"404: not found", "404 Not Found")
}

/// 白名单内的仓库文件直读（`repo_root` + 相对路径；`PathGuardError` = 400/404 语义由调用方映射）。
fn guarded_file(
    repo_root: &Path,
    relative: &str,
) -> Result<(std::path::PathBuf, Vec<u8>), PathGuardError> {
    // 白名单 = 数据驱动（console_sources.json 混合白名单，22 号 §10.2：
    // 目录项前缀 + 文件项精确；assets/ 未登记文件 = 拒）。
    let sources = load_sources_file(repo_root).unwrap_or_default();
    let path = resolve_guarded(repo_root, &sources.repo_sources, relative.trim_start_matches('/'))?;
    let body = std::fs::read(&path)
        .map_err(|e| PathGuardError { reason: format!("读 {} 失败：{e}", path.display()) })?;
    Ok((path, body))
}

fn header_body(mime: &str, body: &[u8], status: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\r\n",
        body.len()
    )
    .into_bytes()
    .into_iter()
    .chain(body.iter().copied())
    .collect()
}
