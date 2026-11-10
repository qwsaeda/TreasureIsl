//! HTTP 请求处理（S32.1 骨架 + S32.2 只读数据源 API + S32.3 看板 UI 静态面
//! （设计图/地图页增量，左菜单 7 页））。
//!
//! - `/api/health` = 200 文本（骨架面）；
//! - `/api/overview` `/api/tasks` `/api/evidence`（含 `?sub=<dir>` 子目录枚举，
//!   设计图页 `sub=ui_concept`）`/api/assets` `/api/tests` `/api/session` `/api/audit`
//!   `/api/map` = 只读数据源适配（`repo::data`，22 号 §5/§10.2；路径全过 `resolve_guarded`
//!   白名单 + canonicalize 防穿越；0 写盘）；
//! - S32.4 门禁执行面：`GET /api/gate` = 运行状态 + 历史（只读，免 token）；
//!   `POST /api/gate` = 写操作（X-TI-Token 鉴权 403 + 非法 tier 400 + 锁占用 409 + 后台 spawn，
//!   22 号 §4.2 白名单 id `gate`；`runner::gate`）；
//! - S32.3 静态面（`web/`）= 看板 UI v1（左菜单分组 8 页 = 概览/任务/证据/资产/设计图/地图/测试/设置；
//!   原生 JS + CSS 全内联无 npm；视觉令牌 = 游戏「航海夜·精修」，内联于 `index.html` `<style>`
//!   （对齐 `docs/process/10_ui_redesign.md` §3 设计令牌 v2；单文件交付，0 外部静态资源依赖）；
//!   图片（证据/资产/设计稿/地图描述符资产）走 `/` 仓库白名单直读，网页内直查 + 灯箱）；
//! - 其余 = 静态映射（`ti-devmeta::http::resolve_or_404`，400/404 语义同 serve.rs 原型）；
//! - 写操作（POST）骨架期走静态映射 ⇒ 非 403；POST 鉴权 + 403 语义 = S32.4 面（22 号 §10.1）。

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;

use serde_json::{Value, json};

use crate::repo::build_ops::build_artifacts;
use crate::repo::config::{RepoSources, load as load_sources_file};

use crate::repo::data;
use crate::repo::guard::{PathGuardError, resolve_guarded};
use crate::repo::import;
use crate::runner;
use ti_devmeta::http::{mime_for, resolve_or_404};

/// S32.6 任务收口路由（`POST /api/close` + `GET /api/close?audit=1`）；从本文件抽出减行。
mod close_api;

/// 每连接一次的数据源清单加载（缺文件/非法 JSON = 该请求 500，不阻断静态面）。
fn load_sources_opt(repo_root: &Path) -> Option<crate::repo::config::RepoSources> {
    load_sources_file(repo_root).ok()
}

/// 取查询串里某个键的值（`?run=<id>` 形态；无该键 = `None`；`url` 含 `?` 才拆）。
pub(crate) fn query_param<'a>(url: &'a str, key: &str) -> Option<&'a str> {
    url.split('?').nth(1)?.split('&').find_map(|kv| kv.strip_prefix(&format!("{key}=")))
}

/// 百分号解码（query 参数；前端 `encodeURIComponent` 输出 `%XX`，服务端须解码后再匹配，
/// 否则含空格/中文/保留字符的查询按字面 `%XX` 匹配 = 0 命中）。实现 = `repo::percent_decode`。
fn percent_decode(input: &str) -> String {
    crate::repo::percent_decode(input)
}

/// 请求头解析结果（S32.4：缓存/压缩协商 + POST body 长度 + 写操作 token + content-type）。
struct ReqHeaders {
    if_none_match: Vec<String>,
    accept_gzip: bool,
    content_length: u64,
    x_ti_token: Option<String>,
    content_type: Option<String>,
}

/// 逐行读请求头（首行已在 serve 读过；空行结束）。
fn read_headers(reader: &mut BufReader<TcpStream>) -> ReqHeaders {
    let mut hdrs = ReqHeaders {
        if_none_match: Vec::new(),
        accept_gzip: false,
        content_length: 0,
        x_ti_token: None,
        content_type: None,
    };
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
            hdrs.if_none_match.push(v.trim().to_owned());
        }
        // 压缩协商：`Accept-Encoding: gzip`（大小写不敏感，含 `gzip` 即认）。
        if let Some(v) = tl.to_lowercase().strip_prefix("accept-encoding:") {
            hdrs.accept_gzip = v.trim().contains("gzip");
        }
        // POST body 长度（`Content-Length`；上限 256KB 防内存暴涨）。
        if let Some(v) = tl.to_lowercase().strip_prefix("content-length:") {
            hdrs.content_length = v.trim().parse().unwrap_or(0);
        }
        // 写操作 token（S32.4）：X-TI-Token（值原样，不 trim 空串为 None 语义在 auth_ok 判）。
        if let Some(v) = tl.strip_prefix("X-TI-Token:") {
            hdrs.x_ti_token = Some(v.trim().to_owned());
        }
        if let Some(v) = tl.strip_prefix("Content-Type:") {
            hdrs.content_type = Some(v.trim().to_owned());
        }
        hline.clear();
    }
    hdrs
}

/// 读 POST 请求体（≤ 256KiB；超限部分由调用方 `content_length > 65_536` 早判 413）。
/// 完整读到 = 返回 body，否则空（非 `POST` / `content_length` 0 亦空）。
fn read_post_body<R: std::io::Read>(reader: &mut R, method: &str, content_length: u64) -> Vec<u8> {
    if method == "POST" && content_length > 0 && content_length <= 262_144 {
        let mut buf = vec![0u8; content_length.min(262_144) as usize];
        let complete = std::io::Read::read_exact(reader, &mut buf).is_ok();
        if complete { buf } else { Vec::new() }
    } else {
        Vec::new()
    }
}

/// 处理一个连接（每连接一线程；`web_root` = 静态壳，`repo_root` = 仓库根只读数据源面）。
pub fn serve(mut stream: TcpStream, web_root: &Path, repo_root: &Path) {
    // 读超时（2026-11-09 修复）：客户端连接后不发数据 / keep-alive 空闲连接会阻塞 `read_line`
    // 长占线程。10s：正常浏览器请求远快于此；超时 = 连接被服务端关闭（客户端重连即可），
    // 防慢客户端线程泄漏（与 main.rs 并发化配套：单线程长占不再冻全局，但线程数仍受控）。
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(10)));
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
    // 请求头面（S32.4：缓存/压缩协商 + POST body 长度 + 写操作 token + content-type）。
    let hdrs = read_headers(&mut reader);
    let if_none_match = hdrs.if_none_match;
    let accept_gzip = hdrs.accept_gzip;
    let content_length = hdrs.content_length;
    let x_ti_token = hdrs.x_ti_token;
    let content_type = hdrs.content_type;

    // POST /api/map/import：读 body（编辑器出描述符 → 落盘 + 回注，S32.14 ② 唯一写点）。
    let request_body = read_post_body(&mut reader, &method, content_length);
    // 统一 22 号 §10.1「请求体上限 64 KiB」口径：超限（含 >256KB 未读入者）一律 413，
    // 而非落入空 body → 400（S32.5 B 侧 F4 修复）。
    let gz = accept_gzip;
    if method == "POST" && content_length > 65_536 {
        let response = post_too_large(gz);
        let _ = stream.write_all(&response);
        let _ = stream.flush();
        return;
    }

    // POST /api/gate|/api/map/import|/api/audit/* = 写操作面（鉴权在各 handler；返回 None 即不匹配）。
    if method == "POST" {
        if let Some(response) = handle_post(
            clean,
            repo_root,
            &request_body,
            x_ti_token.as_deref(),
            content_type.as_deref(),
            gz,
        ) {
            let _ = stream.write_all(&response);
            let _ = stream.flush();
            return;
        }
    }
    let response = dispatch_get(clean, url, web_root, repo_root, &if_none_match, gz);
    let _ = stream.write_all(&response);
    let _ = stream.flush();
}

/// GET 路由分发（只读面；POST 写操作面在 `handle_post`）。
/// `GET /api/jobs`（统一作业模型查询面：列表 / 详情 + 日志尾 + 证明；23 号 §6.2）。
fn jobs_get(repo_root: &Path, url: &str, gz: bool) -> Vec<u8> {
    let kind = percent_decode(query_param(url, "kind").unwrap_or(""));
    let id = percent_decode(query_param(url, "id").unwrap_or(""));
    json200(&crate::runner::jobs::jobs_view(repo_root, Some(kind.as_str()), Some(id.as_str())), gz)
}

/// `GET /api/tests`（测试清单 + `?run=<id>` 白名单执行）。
///
/// 拆出独立函数：`dispatch_get` 行数超限（CS-3 局部函数规模门禁）。
fn tests_get(repo_root: &Path, url: &str, gz: bool) -> Vec<u8> {
    let run = query_param(url, "run");
    api_500_or_json(
        load_json(repo_root, |s| match run {
            Some(id) => data::run_test(repo_root, s, id),
            None => data::tests(repo_root, s),
        }),
        gz,
    )
}

fn dispatch_get(
    clean: &str,
    url: &str,
    web_root: &Path,
    repo_root: &Path,
    if_none_match: &[String],
    gz: bool,
) -> Vec<u8> {
    match clean {
        "/api/health" => text200("ti-console ok", gz),
        "/api/session" => json200(&data::session(), gz),
        "/api/whitelist" => {
            let path_param = percent_decode(query_param(url, "path").unwrap_or(""));
            json200(&data::whitelist_probe(repo_root, &path_param), gz)
        }
        "/api/search" => {
            // S32.8 文档检索（只读全文搜索 + EXP/F-ID/任务 ID 反查；q 必填，空 = 空结果）。
            // 前端 encodeURIComponent → 服务端百分号解码（中文/空格查询才命中；上限 256 字符防长串扫盘）。
            let q: String =
                percent_decode(query_param(url, "q").unwrap_or("")).chars().take(256).collect();
            json200(&crate::repo::search::search(repo_root, &q), gz)
        }
        "/api/audit" => {
            api_500_or_json(load_json(repo_root, |s| data::audit_list(repo_root, s)), gz)
        }
        "/api/map" => {
            api_500_or_json(load_json(repo_root, |s| data::map_packs_detail(repo_root, s)), gz)
        }
        // 单包只读详情（编辑器「编辑已有包」载入面；0 写盘，免 token）。
        "/api/map/pack" => {
            let id = percent_decode(query_param(url, "id").unwrap_or(""));
            api_500_or_json(load_json(repo_root, |s| data::map_pack_detail(repo_root, s, &id)), gz)
        }
        // S44.4 发布面只读状态（运行中标志 + 末次 exit_code + 日志历史 + ?log= 尾部；免 token，镜像 GET /api/gate）。
        "/api/publish" => publish_get(repo_root, url, gz),
        "/api/snapshot" => json200(
            &data::snapshot(repo_root, &load_sources_opt(repo_root).unwrap_or_default()),
            gz,
        ),
        "/api/overview" => {
            api_500_or_json(load_json(repo_root, |s| data::overview(repo_root, s)), gz)
        }
        "/api/tasks" => api_500_or_json(load_json(repo_root, |s| data::tasks(repo_root, s)), gz),
        "/api/evidence" => {
            let sub = percent_decode(query_param(url, "sub").unwrap_or(""));
            api_500_or_json(load_json(repo_root, |s| data::evidence_list(repo_root, s, &sub)), gz)
        }
        "/api/assets" => api_500_or_json(load_json(repo_root, |s| data::assets(repo_root, s)), gz),
        "/api/tests" => tests_get(repo_root, url, gz),
        // 统一作业模型查询面（23 号 §6.2；S49.7）：?kind=<kind>[&id=<id>]
        // 列表 = 各作业状态；详情 = 状态 + 日志尾 + 证明（evidence[]）。只读，免 token。
        "/api/jobs" => jobs_get(repo_root, url, gz),
        "/api/gate" => {
            let log = query_param(url, "log");
            json200(&runner::gate::gate_status(repo_root, log), gz)
        }
        "/api/close" => close_api::handle_close_get(repo_root, url, gz),
        "/api/content/map" => {
            // S44.2 地图向导状态（只读）。
            let id = query_param(url, "id").unwrap_or("");
            match crate::content::map_wizard::state(repo_root, id) {
                Ok(v) => json200(&v, gz),
                Err((code, msg)) => status_json(code, &json!({"ok": false, "error": msg}), gz),
            }
        }
        "/api/content/ui" => {
            // S44.3 UI 向导状态（只读）。
            let id = query_param(url, "id").unwrap_or("");
            match crate::content::ui_wizard::state(repo_root, id) {
                Ok(v) => json200(&v, gz),
                Err((code, msg)) => status_json(code, &json!({"ok": false, "error": msg}), gz),
            }
        }
        "/api/content/mapping" => {
            // S44.5 固化收口：29 号 §2/§3 流程 ↔ 平台向导 1:1 映射断言（只读免 token）。
            json200(&crate::content::docmap::mapping_report(repo_root), gz)
        }
        "/api/build" => {
            // S32.9：?check=1 = 只读体检（build_wasm.ps1 -Check + 冻结路径 0 游戏产物变更断言）。
            // S35.1：?run 白名单 apk|wasm|native（后台 spawn）；非法 = 400（22 号 §10.3 禁自由字符串）。
            if query_param(url, "check") == Some("1") {
                api_500_or_json(
                    load_json(repo_root, |s| crate::runner::build::build_check(repo_root, s)),
                    gz,
                )
            } else if let Some(kind) = query_param(url, "run") {
                if matches!(kind, "apk" | "wasm" | "native") {
                    api_500_or_json(
                        load_json(repo_root, |s| build_artifacts(repo_root, s, Some(kind))),
                        gz,
                    )
                } else {
                    header_body(
                        "application/json",
                        r#"{"ok":false,"error":"400: run 只接受 apk|wasm|native（22 号 §10.3 白名单；非自由字符串）"}"#
                            .as_bytes(),
                        "400 Bad Request",
                        "no-store",
                        None,
                        gz,
                    )
                }
            } else {
                api_500_or_json(load_json(repo_root, |s| build_artifacts(repo_root, s, None)), gz)
            }
        }
        "/api/themes" => crate::repo::themes::themes_get(repo_root, url, gz),
        "/api/apk-size" => {
            api_500_or_json(load_json(repo_root, |s| data::apk_size(repo_root, s)), gz)
        }
        _ => static_or_repo_file(web_root, clean, repo_root, if_none_match, gz),
    }
}

/// `GET /api/themes`（S49.3 主题管理）：`?verify=<tid>` 只读校验；无参数 = list（全部主题包 + applied 态）。
/// `GET /api/publish`（S44.4 发布面只读状态）：运行中标志 + 末次状态（含 `exit_code`）+ 日志历史
/// + `?log=<name>` 尾部（白名单形状校验 + 防穿越由 `publish_status` 内部负责）。
fn publish_get(repo_root: &Path, url: &str, gz: bool) -> Vec<u8> {
    let log = query_param(url, "log").map(crate::repo::percent_decode);
    json200(&crate::content::publish::publish_status(repo_root, log), gz)
}

/// 有数据源清单 = 跑端点；无清单 = 500（数据源面不可用，如实上报）。
pub(crate) fn load_json(
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
    header_body(
        "application/json",
        result.to_string().as_bytes(),
        "200 OK",
        "no-store",
        None,
        want_gzip,
    )
}

/// 请求体超 64 KiB 上限的 413 响应（22 号 §10.1；`serve` 读取前早判，统一口径）。
fn post_too_large(want_gzip: bool) -> Vec<u8> {
    header_body(
        "application/json",
        r#"{"ok":false,"error":"413: 请求体超 64 KiB 上限（22 号 §10.1）"}"#.as_bytes(),
        "413 Payload Too Large",
        "no-store",
        None,
        want_gzip,
    )
}

/// 生图 JSON 错误响应（gen-island 内联 helper，统一 `{"ok":false,"error":...}`）。
fn gen_island_err(want_gzip: bool, status: &str, msg: &str) -> Vec<u8> {
    let text = format!(r#"{{"ok":false,"error":"{}"}}"#, msg.replace('"', "'"));
    header_body("application/json", text.as_bytes(), status, "no-store", None, want_gzip)
}

/// 从项目 `.env`（gitignored，规范 = key 存这里）读环境变量；`std::env` 优先，`.env` 兜底。
/// 解析：逐行 `KEY=VALUE`，跳过 `#` 注释/空行，去成对引号。key 缺失 = None（如实上报）。
/// 登记 AI 生成格子卡图到 `assets/ui_manifest.json`（disk→registry 双向同步）。
///
/// **唯一清单写入路径 = `scripts/ui_assets.py register-tile`**（`04_tooling` §4）：该命令用
/// `json.dumps(indent=2)` 保序写出 = 与仓库清单同格式 → **零 churn**（旧实现用
/// `serde_json::to_string_pretty` 会 2 空格 + 键序重排，整文件 churn）。
/// roles.tile 缺则补（1:1 透明徽章模板）；assets 缺 `tile_<isle>` 则追加（不覆盖已有 AI 图）；
/// 幂等（已登记 = 0 改动）；**只登记不生图**（图片由 `scripts/agnes_gen.py` 唯一入口生成）；
/// 读/写失败 = 静默（`validate_data` 门禁兜底报错，不静默绕过）。
fn register_tile_in_manifest(repo_root: &Path, isle: &str, prompt: &str) {
    let script = repo_root.join("scripts").join("ui_assets.py");
    let mut cmd = std::process::Command::new("python3");
    cmd.arg(&script)
        .arg("register-tile")
        .arg("--isle")
        .arg(isle)
        .arg("--what")
        .arg(prompt)
        .current_dir(repo_root);
    crate::runner::apply_minimal_env(&mut cmd);
    let _ = cmd.output();
}

fn read_env_file(repo_root: &Path, key: &str) -> Option<String> {
    let env_path = repo_root.join(".env");
    let content = std::fs::read_to_string(&env_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (k, v) = line.split_once('=')?;
        if k.trim() == key {
            let mut v = v.trim().to_string();
            if (v.starts_with('"') && v.ends_with('"'))
                || (v.starts_with('\'') && v.ends_with('\''))
            {
                v = v[1..v.len() - 1].to_string();
            }
            return Some(v);
        }
    }
    None
}

/// AI 资产自生成目标（用户 2026-11-10「中央岛/格子不能自己生成」）：
/// `Island` → `assets/ui/island_home_<map_id>.png`（真渲染命名约定，0 游戏改动）；
/// `Tile` → `assets/ui/tile_<isle>.png`（领地格 AI 整卡图，游戏 if-blit 回退程序绘制）。
#[derive(Clone, Copy, PartialEq)]
enum GenAssetKind {
    Island,
    Tile,
}

impl GenAssetKind {
    /// 生成参数键名（body JSON 字段；`Island=map_id` / `Tile=isle`）。
    fn key_name(self) -> &'static str {
        match self {
            GenAssetKind::Island => "map_id",
            GenAssetKind::Tile => "isle",
        }
    }

    /// 输出资产文件（相对 `assets/ui/`）。
    fn file_name(self, key: &str) -> String {
        match self {
            GenAssetKind::Island => format!("island_home_{key}.png"),
            GenAssetKind::Tile => format!("tile_{key}.png"),
        }
    }

    /// 展示名（错误信息用）。
    fn label(self) -> &'static str {
        match self {
            GenAssetKind::Island => "中央岛图",
            GenAssetKind::Tile => "格子整卡图",
        }
    }
}

/// 10 类岛型白名单（`assets/data/board.json`/`scenes.json` isle 值域；编辑器 `ED_ISLES` 同集）。
const GEN_ISLES: &[&str] = &[
    "boss_fort",
    "harbour",
    "lighthouse",
    "lightship",
    "pirate_lair",
    "shipyard",
    "skull_cliff",
    "trading_post",
    "volcano",
    "wreck",
];

/// `POST /api/gen-island` / `POST /api/gen-tile`：AI 提示词 → Agnes 生图 →
/// `assets/ui/<kind>_<key>.png`（真渲染命名约定；Island 0 游戏改动，Tile 游戏 if-blit 回退）。
/// body `{prompt, map_id|isle, size?, ratio?}`；key 校验（Island 仅 `[a-z0-9_]` 防路径穿越 /
/// Tile 仅 10 类岛型）、prompt 非空、size/ratio 白名单；`AGNES_API_KEY` 项目 `.env` 或环境
/// 变量缺失 = 400（如实上报）；子进程非 0 / 输出未落盘 = 500。
fn handle_gen_asset(repo_root: &Path, body: &[u8], want_gzip: bool, kind: GenAssetKind) -> Vec<u8> {
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            return gen_island_err(want_gzip, "400 Bad Request", &format!("body 非合法 JSON：{e}"));
        }
    };
    let prompt = payload.get("prompt").and_then(|v| v.as_str()).unwrap_or("").trim();
    let key = payload.get(kind.key_name()).and_then(|v| v.as_str()).unwrap_or("").trim();
    let size = payload.get("size").and_then(|v| v.as_str()).unwrap_or("1K");
    let ratio = payload.get("ratio").and_then(|v| v.as_str()).unwrap_or("1:1");
    // key 校验：Island = [a-z0-9_]（防路径穿越）；Tile = 10 类岛型白名单。
    let key_ok = match kind {
        GenAssetKind::Island => {
            !key.is_empty()
                && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        }
        GenAssetKind::Tile => GEN_ISLES.contains(&key),
    };
    if !key_ok {
        let hint = match kind {
            GenAssetKind::Island => "map_id 非法（仅 [a-z0-9_]）".to_string(),
            GenAssetKind::Tile => format!("isle 非法（10 类岛型：{}）", GEN_ISLES.join("/")),
        };
        return gen_island_err(want_gzip, "400 Bad Request", &hint);
    }
    if prompt.is_empty() {
        return gen_island_err(want_gzip, "400 Bad Request", "prompt 为空");
    }
    if !["1K", "2K", "3K", "4K"].contains(&size) {
        return gen_island_err(want_gzip, "400 Bad Request", "size 非法（1K/2K/3K/4K）");
    }
    if !["1:1", "3:4", "4:3", "16:9", "9:16", "2:3", "3:2", "21:9"].contains(&ratio) {
        return gen_island_err(want_gzip, "400 Bad Request", "ratio 非法（1:1 等）");
    }
    let api_key = std::env::var("AGNES_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .or_else(|| read_env_file(repo_root, "AGNES_API_KEY"))
        .unwrap_or_default();
    if api_key.is_empty() {
        return gen_island_err(
            want_gzip,
            "400 Bad Request",
            "AGNES_API_KEY 未配置（项目 .env 或环境变量缺失，无法调用 Agnes 生图）",
        );
    }
    let file_name = kind.file_name(key);
    let out = repo_root.join("assets").join("ui").join(&file_name);
    let script = repo_root.join("scripts").join("agnes_gen.py");
    let mut cmd = std::process::Command::new("python3");
    cmd.arg(&script)
        .arg("--prompt")
        .arg(prompt)
        .arg("--out")
        .arg(&out)
        .arg("--size")
        .arg(size)
        .arg("--ratio")
        .arg(ratio)
        .current_dir(repo_root);
    crate::runner::apply_minimal_env(&mut cmd);
    cmd.env("AGNES_API_KEY", &api_key);
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return gen_island_err(
                want_gzip,
                "500 Internal Server Error",
                &format!("spawn agnes_gen.py 失败：{e}"),
            );
        }
    };
    let exit = output.status.code().unwrap_or(-1);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if exit != 0 || !out.is_file() {
        return gen_island_err(
            want_gzip,
            "500 Internal Server Error",
            &format!("{} 生图失败 exit={exit}：{combined}", kind.label()),
        );
    }
    // Tile：生成后自动登记 ui_manifest（AI 卡图落盘 → disk↔registry 双向同步；岛型卡图
    // 独立可换 = 换 tile_<isle>.png 即零引擎改动；validate_data::check_asset_registry 守门）。
    if kind == GenAssetKind::Tile {
        register_tile_in_manifest(repo_root, key, prompt);
    }
    let resp = json!({
        "ok": true,
        "path": format!("assets/ui/{file_name}"),
        "size": size,
        "ratio": ratio,
        "bytes": std::fs::metadata(&out).map_or(0, |m| m.len()),
    });
    let text = resp.to_string();
    header_body("application/json", text.as_bytes(), "200 OK", "no-store", None, want_gzip)
}

/// `POST /api/*` 写操作分派（S32.4/S32.5）：gate / map/import / audit 匹配返回响应，否则 `None`。
fn handle_post(
    clean: &str,
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    gz: bool,
) -> Option<Vec<u8>> {
    match clean {
        // S32.4 门禁执行面（写操作：X-TI-Token 鉴权 + tier 分档 + 单实例锁 + spawn）。
        "/api/gate" => Some(handle_gate(repo_root, body, token, content_type, gz)),
        // S32.14 ② 唯一写点（编辑器描述符 → 落 assets/maps/<id>/ + 回注）。
        "/api/map/import" => Some(handle_map_import(repo_root, body, gz)),
        // 用户 2026-11-10「中央岛图/格子自己生成」：AI 提示词 → Agnes 生图 →
        // `assets/ui/island_home_<map_id>.png`（真渲染命名约定）/ `assets/ui/tile_<isle>.png`
        //（领地格 AI 整卡图，游戏 if-blit 回退程序绘制）。
        "/api/gen-island" => Some(handle_gen_asset(repo_root, body, gz, GenAssetKind::Island)),
        "/api/gen-tile" => Some(handle_gen_asset(repo_root, body, gz, GenAssetKind::Tile)),
        // S32.5 双审计编排（任务书生成 + 粘贴归档）。
        "/api/audit/taskbook" => {
            Some(handle_audit_taskbook(repo_root, body, token, content_type, gz))
        }
        "/api/audit/archive" => {
            Some(handle_audit_archive(repo_root, body, token, content_type, gz))
        }
        // S49.3 主题管理：apply（写 scenes.json + 部署资产，X-TI-Token + enable_write 门控）。
        "/api/themes/apply" => {
            Some(crate::repo::themes::handle_themes_apply(repo_root, body, token, content_type, gz))
        }
        // S32.6 任务收口（close_check 只读体检 / close_run 写 + EXP 查重）。
        "/api/close" => Some(close_api::handle_close(repo_root, body, token, content_type, gz)),
        // S44.2/S44.3 地图/UI 向导执行一步（写动作需 enable_write；不可跳步 409；失败即停）。
        // F-1 修复：写路由须 X-TI-Token（22 号 §10.1）；F4：Content-Type application/json。
        "/api/content/map/step" => Some(crate::content::handle_wizard_step(
            repo_root,
            body,
            token,
            content_type,
            gz,
            false,
        )),
        "/api/content/ui/step" => {
            Some(crate::content::handle_wizard_step(repo_root, body, token, content_type, gz, true))
        }
        // S44.4 发布面（30 号 §2.3）：写操作鉴权（403）+ Content-Type（400）+ mode 白名单（400）
        // + run 无 enable_write（400）；check = 只读体检，run = 后台 spawn。
        "/api/publish" => {
            Some(crate::content::publish::handle_publish(repo_root, body, token, content_type, gz))
        }
        _ => None,
    }
}

/// `POST /api/audit/taskbook`（S32.5 双审计编排）：生成审计任务书 `.prompt.md`。
/// 写操作鉴权（403）→ Content-Type application/json（400）→ body ≤ 64KiB（413）→
/// body `{"task","round","role","surface?"}` → `audit::taskbook::generate`。
fn handle_audit_taskbook(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    if !auth_ok(token) {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"403: 写操作需 X-TI-Token（22 号 §10.1）"}"#.as_bytes(),
            "403 Forbidden",
            "no-store",
            None,
            want_gzip,
        );
    }
    let ct_ok = content_type.is_some_and(|c| {
        let base = c.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        base == "application/json"
    });
    if !ct_ok {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"400: Content-Type 必须为 application/json（22 号 §10.3）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        );
    }
    if body.len() > 65_536 {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"413: 请求体超 64 KiB 上限（22 号 §10.1）"}"#.as_bytes(),
            "413 Payload Too Large",
            "no-store",
            None,
            want_gzip,
        );
    }
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            let text = format!(r#"{{"ok":false,"error":"400: body 非合法 JSON：{e}"}}"#);
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
    let task = payload.get("task").and_then(Value::as_str).unwrap_or("");
    let Some(round) =
        payload.get("round").and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok())
    else {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"400: round 必须为 ≤ u32::MAX 的非负整数"}"#.as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        );
    };
    let role =
        payload.get("role").and_then(Value::as_str).and_then(|s| s.chars().next()).unwrap_or(' ');
    let surface = payload
        .get("surface")
        .and_then(Value::as_str)
        .unwrap_or("A：DoD 逐条；B：架构/规范/文档/门禁/回归/安全/依赖");
    match crate::audit::taskbook::generate(repo_root, task, round, role, surface) {
        Ok((file, uncovered)) => json200(
            &json!({ "ok": true, "file": file, "uncovered_elements": uncovered }),
            want_gzip,
        ),
        Err(e) => {
            let body = format!(r#"{{"ok":false,"error":"{}"}}"#, e.replace('"', "\\\""));
            header_body(
                "application/json",
                body.as_bytes(),
                "400 Bad Request",
                "no-store",
                None,
                want_gzip,
            )
        }
    }
}

fn handle_audit_archive(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    if !auth_ok(token) {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"403: 写操作需 X-TI-Token（22 号 §10.1）"}"#.as_bytes(),
            "403 Forbidden",
            "no-store",
            None,
            want_gzip,
        );
    }
    let ct_ok = content_type.is_some_and(|c| {
        let base = c.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        base == "application/json"
    });
    if !ct_ok {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"400: Content-Type 必须为 application/json（22 号 §10.3）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        );
    }
    if body.len() > 65_536 {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"413: 请求体超 64 KiB 上限（22 号 §10.1）"}"#.as_bytes(),
            "413 Payload Too Large",
            "no-store",
            None,
            want_gzip,
        );
    }
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            let text = format!(r#"{{"ok":false,"error":"400: body 非合法 JSON：{e}"}}"#);
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
    let file = payload.get("file").and_then(Value::as_str).unwrap_or("");
    let content = payload.get("content").and_then(Value::as_str).unwrap_or("");
    match crate::audit::archive::archive(repo_root, file, content) {
        Ok(path) => json200(&json!({ "ok": true, "file": path }), want_gzip),
        Err(e) => {
            let body = format!(r#"{{"ok":false,"error":"{}"}}"#, e.replace('"', "\\\""));
            header_body(
                "application/json",
                body.as_bytes(),
                "400 Bad Request",
                "no-store",
                None,
                want_gzip,
            )
        }
    }
}

/// `POST /api/gate`（S32.4 门禁执行面）：X-TI-Token 鉴权（403）→ tier 校验（400）
/// → 单实例锁（409）→ `runner::gate::run_gate` 后台 spawn（200）。
/// body = `{"tier": "fast"|"touch"|"full"}`；Content-Type 只接受 application/json（22 号 §10.3）。
fn handle_gate(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    // 写操作鉴权（22 号 §10.1）：X-TI-Token 缺失或（env 设了但）不匹配 = 403。
    if !auth_ok(token) {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"403: 写操作需 X-TI-Token（22 号 §10.1；设置页粘贴 TI_CONSOLE_TOKEN）"}"#.as_bytes(),
            "403 Forbidden",
            "no-store",
            None,
            want_gzip,
        );
    }
    // 只接受 application/json（22 号 §10.3；精确匹配 + 可带 charset 参数）。
    let ct_ok = content_type.is_some_and(|c| {
        let base = c.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        base == "application/json"
    });
    if !ct_ok {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"400: Content-Type 必须为 application/json（22 号 §10.3）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        );
    }
    // 请求体上限 64 KiB（22 号 §10.1；gate body 极小，超限 = 413）。
    if body.len() > 65_536 {
        return header_body(
            "application/json",
            r#"{"ok":false,"error":"413: 请求体超 64 KiB 上限（22 号 §10.1）"}"#.as_bytes(),
            "413 Payload Too Large",
            "no-store",
            None,
            want_gzip,
        );
    }
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            let text = format!(r#"{{"ok":false,"error":"400: body 非合法 JSON：{e}"}}"#);
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
    let tier = payload.get("tier").and_then(Value::as_str).unwrap_or("");
    match runner::gate::run_gate(repo_root, tier) {
        Ok(value) => json200(&value, want_gzip),
        Err((code, msg)) => {
            let status = match code {
                400 => "400 Bad Request",
                409 => "409 Conflict",
                _ => "500 Internal Server Error",
            };
            let body = format!(r#"{{"ok":false,"error":"{}"}}"#, msg.replace('"', "\\\""));
            header_body("application/json", body.as_bytes(), status, "no-store", None, want_gzip)
        }
    }
}

/// 写操作鉴权（22 号 §10.1）：
/// - `TI_CONSOLE_TOKEN` env 设了 → 头必须存在且精确匹配；
/// - env 未设（本地内网自用）→ 头存在且非空即可（S32.14 ② 免鉴权例外之外，写面一律要求头在场）。
pub(crate) fn auth_ok(token: Option<&str>) -> bool {
    match (std::env::var("TI_CONSOLE_TOKEN").ok(), token) {
        (Some(env), Some(hdr)) => !hdr.is_empty() && env == hdr,
        (_, None) => false,
        (None, Some(hdr)) => !hdr.is_empty(),
    }
}

pub(crate) fn api_500_or_json(result: Result<Value, String>, want_gzip: bool) -> Vec<u8> {
    match result {
        Ok(value) => json200(&value, want_gzip),
        Err(err) => {
            let body = format!(r#"{{"error":"{}"}}"#, err.replace('"', "\\\""));
            header_body(
                "application/json",
                body.as_bytes(),
                "500 Internal Server Error",
                "no-store",
                None,
                want_gzip,
            )
        }
    }
}

fn text200(body: &str, want_gzip: bool) -> Vec<u8> {
    let bytes = body.as_bytes();
    header_body("text/plain", bytes, "200 OK", "no-store", None, want_gzip)
}

pub(crate) fn json200(value: &Value, want_gzip: bool) -> Vec<u8> {
    let bytes = value.to_string().into_bytes();
    header_body("application/json", &bytes, "200 OK", "no-store", None, want_gzip)
}

/// 带状态码的 JSON 响应（S44.2 向导 400/409 判据回显）。
pub(crate) fn status_json(code: u16, value: &Value, want_gzip: bool) -> Vec<u8> {
    let status = match code {
        400 => "400 Bad Request",
        403 => "403 Forbidden",
        409 => "409 Conflict",
        413 => "413 Payload Too Large",
        500 => "500 Internal Server Error",
        _ => "200 OK",
    };
    let bytes = value.to_string().into_bytes();
    header_body("application/json", &bytes, status, "no-store", None, want_gzip)
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
        return header_body(
            "text/plain",
            b"404: unknown api endpoint",
            "404 Not Found",
            "no-store",
            None,
            want_gzip,
        );
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
        return header_body(
            "text/plain",
            b"404: not found",
            "404 Not Found",
            "no-store",
            None,
            want_gzip,
        );
    }
    // `..` 穿越 = 400 拒绝（与 22 号 §10.2 + guard::check_relative 同口径）。
    if clean.contains("..") {
        return header_body(
            "text/plain",
            b"400: path traversal rejected",
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        );
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
pub(crate) fn header_body(
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
    use flate2::Compression;
    use flate2::write::GzEncoder;
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
        vn == etag
            || vn.trim_matches('"') == norm
            || vn.trim_start_matches("W/").trim_matches('"') == norm
    });
    if hit {
        // 304 空 body（不压；浏览器已缓存原对象）。
        return header_body(
            "text/plain",
            b"",
            "304 Not Modified",
            "max-age=604800, public",
            Some(etag),
            false,
        );
    }
    header_body(mime, body, "200 OK", "max-age=604800, public", Some(etag), want_gzip)
}
