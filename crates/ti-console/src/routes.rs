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
use crate::repo::import;
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
    let method = parts.next().unwrap_or("").to_uppercase();
    let url = parts.next().unwrap_or("/");
    let clean = url.split(['?', '#']).next().unwrap_or("/");
    // 请求头面（缓存协商读 `If-None-Match` + 压缩协商读 `Accept-Encoding` + POST body 长度读 `Content-Length`）。
    let mut if_none_match: Vec<String> = Vec::new();
    let mut accept_gzip = false;
    let mut content_length: u64 = 0;
    let mut hline = String::new();
    while let Ok(n) = reader.read_line(&mut hline) {
        if n == 0 {
            break;
        }
        let tl = hline.trim();
        if tl.is_empty() {
            break;
        }
        if let Some(v) = tl.strip_prefix("If-None-Match:") {
            if_none_match.push(v.trim().to_owned());
        }
        // 压缩协商：`Accept-Encoding: gzip`（大小写不敏感，含 `gzip` 即认）。
        if let Some(v) = tl.to_lowercase().strip_prefix("accept-encoding:") {
            accept_gzip = v.trim().contains("gzip");
        }
        // POST body 长度（`Content-Length`；上限 256KB 防内存暴涨）。
        if let Some(v) = tl.to_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
        hline.clear();
    }

    // POST /api/map/import：读 body（编辑器出描述符 → 落盘 + 回注，S32.14 ② 唯一写点）。
    let request_body: Vec<u8> = if method == "POST" && content_length > 0 && content_length <= 262_144 {
        let mut buf = vec![0u8; content_length.min(262_144) as usize];
        let complete = std::io::Read::read_exact(&mut reader, &mut buf).is_ok();
        if complete {
            buf
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let gz = accept_gzip;
    // POST /api/map/import = S32.14 ② 唯一写点（编辑器描述符 → 落 assets/maps/<id>/ + 回注）。
    if method == "POST" && clean == "/api/map/import" {
        let response = handle_map_import(repo_root, &request_body, gz);
        let _ = stream.write_all(&response);
        let _ = stream.flush();
        return;
    }
    let response = match clean {
        "/api/health" => text200("ti-console ok", gz),
        "/api/session" => json200(&data::session(), gz),
        "/api/whitelist" => {
            let query = url.split('?').nth(1).unwrap_or("");
            let path_param = query.strip_prefix("path=").unwrap_or("").to_owned();
            json200(&data::whitelist_probe(repo_root, &path_param), gz)
        }
        "/api/audit" => api_500_or_json(load_json(repo_root, |s| data::audit_list(repo_root, s)), gz),
        "/api/map" => {
            api_500_or_json(load_json(repo_root, |s| data::map_packs_detail(repo_root, s)), gz)
        }
        "/api/snapshot" => {
            json200(&data::snapshot(repo_root, &load_sources_opt(repo_root).unwrap_or_default()), gz)
        }
        "/api/overview" => api_500_or_json(load_json(repo_root, |s| data::overview(repo_root, s)), gz),
        "/api/tasks" => api_500_or_json(load_json(repo_root, |s| data::tasks(repo_root, s)), gz),
        "/api/evidence" => {
            let query = url.split('?').nth(1).unwrap_or("");
            let sub = query.strip_prefix("sub=").unwrap_or("").to_owned();
            api_500_or_json(load_json(repo_root, |s| data::evidence_list(repo_root, s, &sub)), gz)
        }
        "/api/assets" => api_500_or_json(load_json(repo_root, |s| data::assets(repo_root, s)), gz),
        _ => static_or_repo_file(web_root, clean, repo_root, &if_none_match, gz),
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

/// `POST /api/map/import`（S32.14 ② 唯一写点）：解析编辑器 body JSON → `import::import_map_pack`
/// （落 `assets/maps/<id>/` 四文件 + 同进程 `map_pack.py sync/verify`）。
/// body 非合法 JSON = 400；写面结果 `ok=false` = 200 带 `errors`（如实上报，不静默）；
/// `ok=true` = 200（sync + verify 全绿）。
fn handle_map_import(repo_root: &Path, body: &[u8], want_gzip: bool) -> Vec<u8> {
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            let text = format!(r#"{{"ok":false,"error":"body 非合法 JSON：{e}"}}"#);
            return header_body(
                "application/json",
                text.as_bytes(),
                "400 Bad Request",
                "no-store",
                None,
                want_gzip,
            );
        }
    };
    let sources = load_sources_opt(repo_root).unwrap_or_default();
    let result = import::import_map_pack(repo_root, &sources, &payload);
    header_body("application/json", result.to_string().as_bytes(), "200 OK", "no-store", None, want_gzip)
}

fn api_500_or_json(result: Result<Value, String>, want_gzip: bool) -> Vec<u8> {
    match result {
        Ok(value) => json200(&value, want_gzip),
        Err(err) => {
            let body = format!(r#"{{"error":"{}"}}"#, err.replace('"', "\\\""));
            header_body("application/json", body.as_bytes(), "500 Internal Server Error", "no-store", None, want_gzip)
        }
    }
}

fn text200(body: &str, want_gzip: bool) -> Vec<u8> {
    let bytes = body.as_bytes();
    header_body("text/plain", bytes, "200 OK", "no-store", None, want_gzip)
}

fn json200(value: &Value, want_gzip: bool) -> Vec<u8> {
    let bytes = value.to_string().into_bytes();
    header_body("application/json", &bytes, "200 OK", "no-store", None, want_gzip)
}

/// 静态映射；静态缺失时回退数据源白名单内的仓库文件直读（`/plan/ledger.yaml` 等）。
/// `if_none_match` = 请求头 `If-None-Match` 值集合（白名单直读面 304 协商用）。
fn static_or_repo_file(
    web_root: &Path,
    clean: &str,
    repo_root: &Path,
    if_none_match: &[String],
    want_gzip: bool,
) -> Vec<u8> {
    if clean.starts_with("/api/") {
        // S32.2 未覆盖的 /api/* = 404（无该端点；写操作 POST = S32.4）。
        return header_body("text/plain", b"404: unknown api endpoint", "404 Not Found", "no-store", None, want_gzip);
    }
    if let Some(path) = resolve_or_404(web_root, clean) {
        if let Ok(body) = std::fs::read(&path) {
            let mime = mime_for(&path);
            // 网页静态壳（index.html）= no-store（改即生效，不缓存旧版）；文本类走 gzip。
            return header_body(mime, &body, "200 OK", "no-store", None, want_gzip);
        }
        if let Ok(v) = guarded_file(repo_root, clean) {
            let mime = mime_for(&v.0);
            // 仓库白名单直读 = 内容级指纹缓存（强缓存：文件不变则 304/本地命中，
            // 改文件即换指纹）。ui_concept 侧车/原图、assets/ui 大图全走这。
            let fp = content_fingerprint(&v.1);
            return cached_response(mime, &v.1, &fp, if_none_match, want_gzip);
        }
        return header_body("text/plain", b"404: not found", "404 Not Found", "no-store", None, want_gzip);
    }
    // `..` 穿越 = 400 拒绝（与 22 号 §10.2 + guard::check_relative 同口径）。
    if clean.contains("..") {
        return header_body("text/plain", b"400: path traversal rejected", "400 Bad Request", "no-store", None, want_gzip);
    }
    // 静态面缺失 + 白名单面也拒 = 404。
    header_body("text/plain", b"404: not found", "404 Not Found", "no-store", None, want_gzip)
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

/// 通用响应组装（`cache_control` + 可选 `etag` + `want_gzip` 协商；304 时 body 为空）。
///
/// gzip 面（S32.13）：仅**文本类 MIME**（`text/*` / `application/json` / yaml 等）在客户端
/// `Accept-Encoding: gzip` 时压；**已压缩二进制（`image/*` / wasm）不压**（PNG 内部已 zlib，
/// 再 gzip ≈ 0 收益还费 CPU，实测省 -0.0%）。`Content-Encoding: gzip` + `Vary: Accept-Encoding`。
fn header_body(
    mime: &str,
    body: &[u8],
    status: &str,
    cache_control: &str,
    etag: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    let do_gzip = want_gzip && is_compressible_mime(mime) && body.len() > 64;
    let payload = if do_gzip { gzip_body(body) } else { body.to_vec() };
    let mut out: Vec<u8> = Vec::with_capacity(payload.len() + 128);
    out.extend_from_slice(
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: {cache_control}\r\n",
            payload.len()
        )
        .as_bytes(),
    );
    if let Some(tag) = etag {
        out.extend_from_slice(format!("ETag: {tag}\r\n").as_bytes());
    }
    if do_gzip {
        out.extend_from_slice(b"Content-Encoding: gzip\r\nVary: Accept-Encoding\r\n");
    }
    out.extend_from_slice("\r\n".as_bytes());
    out.extend_from_slice(&payload);
    out
}

/// 文本类 MIME = gzip 有收益（实测 index.html 省 69% / JSON 省 85~93% / YAML 省 65%）；
/// 二进制（image/* / wasm / octet-stream）= 已压缩或无收益，不压。
fn is_compressible_mime(mime: &str) -> bool {
    mime.starts_with("text/")
        || mime.contains("json")
        || mime.contains("xml")
        || mime.contains("javascript")
        || mime.contains("yaml")
}

/// gzip 压缩（flate2，level 6；0 外部新增依赖 = workspace 已登记）。
fn gzip_body(body: &[u8]) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut enc = GzEncoder::new(Vec::new(), Compression::new(6));
    if enc.write_all(body).is_ok() {
        if let Ok(finished) = enc.finish() {
            return finished;
        }
    }
    // 压失败兜底 = 原样返回（不阻断响应，退化为不压缩）。
    body.to_vec()
}

/// 文件内容指纹（FNV-1a 64 位 + 长度，0 外部依赖；编进 ETag，文件改即换 ⇒ 缓存自动失效）。
fn content_fingerprint(body: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in body {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("W/\"fnv64-{h:016x}-len{}\"", body.len())
}

/// 带缓存协商的响应（白名单直读面）：`If-None-Match` 命中 = 304 空 body（省流量）；
/// 否则 200 + `ETag` + 7 天强缓存（本地/CDN 复用；改文件 ⇒ 指纹变 ⇒ 自动失效）。
/// 文本类 MIME + 客户端 `Accept-Encoding: gzip` 时 body 走 gzip（`Content-Encoding: gzip`）。
fn cached_response(
    mime: &str,
    body: &[u8],
    etag: &str,
    if_none_match: &[String],
    want_gzip: bool,
) -> Vec<u8> {
    let norm = etag.trim_start_matches("W/").trim_matches('"');
    let hit = if_none_match.iter().any(|v| {
        let vn = v.trim();
        vn == etag || vn.trim_matches('"') == norm || vn.trim_start_matches("W/").trim_matches('"') == norm
    });
    if hit {
        // 304 空 body（不压；浏览器已缓存原对象）。
        return header_body("text/plain", b"", "304 Not Modified", "max-age=604800, public", Some(etag), false);
    }
    header_body(mime, body, "200 OK", "max-age=604800, public", Some(etag), want_gzip)
}
