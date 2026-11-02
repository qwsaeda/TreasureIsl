//! S32.6 任务收口 HTTP 路由（`POST /api/close` + `GET /api/close?audit=1`）。
//!
//! 从 `routes.rs` 抽出（S32.6 收口；routes.rs 超 audit_code 900 行上限，抽子模块减行）。
//! 业务执行逻辑在 `runner::close`；本模块只做 HTTP 前置（鉴权/CT/body/mode 分派）与响应组帧。
//! 私有 helper（`header_body`/`json200`/`auth_ok`/`query_param`）经 `super::` 访问（同 crate 根 routes 模块）。

use std::path::Path;

use serde_json::Value;

use super::{auth_ok, header_body, json200, query_param};
use crate::runner;

/// `GET /api/close`（S32.6）：`?audit=1` = 只读白名单对账断言（22 号 §4.2 人读视图 id 集合
/// == `console_commands.json`；免 token）；无 `audit` 参数 = 400（写走 POST）。
pub fn handle_close_get(repo_root: &Path, url: &str, want_gzip: bool) -> Vec<u8> {
    if query_param(url, "audit") == Some("1") {
        match runner::close::whitelist_audit(repo_root) {
            Ok(v) => json200(&v, want_gzip),
            Err(e) => {
                let body = format!(r#"{{"ok":false,"error":"{}"}}"#, e.replace('"', "\\\""));
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
    } else {
        header_body(
            "application/json",
            r#"{"ok":false,"error":"400: GET /api/close 仅支持 ?audit=1（只读白名单对账）；写走 POST"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        )
    }
}

/// `POST /api/close`（S32.6 任务收口）：`close_check` 只读体检 / `close_run` 写（需
/// `--enable-write`）/ `exp_check` EXP 查重。
/// 写操作鉴权（403）→ Content-Type application/json（400）→ body ≤ 64KiB（413）→
/// body `{"mode":"check|run|exp","task","cmd_ids","enable_write"}` → `runner::close::*`。
/// 未登记 id → 400（22 号 §4.2 禁止 1）；`whitelist_audit` 对账断言失败 → 500（不静默）。
pub fn handle_close(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
) -> Vec<u8> {
    let payload = match parse_write_json(token, content_type, body, want_gzip) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // 白名单对账断言（S32.6 DoD）：人读视图 id 集合 == console_commands.json；不一致 = 500 不静默。
    if let Err(e) = runner::close::whitelist_audit(repo_root) {
        let body = format!(r#"{{"ok":false,"error":"500: {e}"}}"#);
        return header_body(
            "application/json",
            body.as_bytes(),
            "500 Internal Server Error",
            "no-store",
            None,
            want_gzip,
        );
    }
    let mode = payload.get("mode").and_then(Value::as_str).unwrap_or("");
    let task = payload.get("task").and_then(Value::as_str).unwrap_or("").to_owned();
    let cmd_ids: Vec<String> = payload
        .get("cmd_ids")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default();
    let enable_write = payload.get("enable_write").and_then(Value::as_bool).unwrap_or(false);
    let volume = payload.get("volume").and_then(Value::as_str).unwrap_or("").to_owned();
    let exp_id = payload.get("id").and_then(Value::as_str).unwrap_or("").to_owned();
    let result = match mode {
        "check" => runner::close::close_check(repo_root, &task),
        "run" => runner::close::close_run(repo_root, &task, &cmd_ids, enable_write),
        "exp" => runner::close::exp_check(repo_root, &volume, &exp_id),
        _ => Err((400u16, format!("mode 必须为 check|run|exp（收到 `{mode}`）"))),
    };
    match result {
        Ok(v) => json200(&v, want_gzip),
        Err((code, msg)) => {
            let body = format!(r#"{{"ok":false,"error":"{code}: {msg}"}}"#);
            header_body(
                "application/json",
                body.as_bytes(),
                &format!("{code} {}", code_text(code)),
                "no-store",
                None,
                want_gzip,
            )
        }
    }
}

/// 写请求前置（S32.4/S32.5/S32.6 共用口径）：鉴权 403 → Content-Type application/json 400 →
/// body ≤ 64KiB 413 → JSON 解析 400。返回 `Ok(payload)` 或 `Err(已组好的响应)`。
fn parse_write_json(
    token: Option<&str>,
    content_type: Option<&str>,
    body: &[u8],
    want_gzip: bool,
) -> Result<Value, Vec<u8>> {
    if !auth_ok(token) {
        return Err(header_body(
            "application/json",
            r#"{"ok":false,"error":"403: 写操作需 X-TI-Token（22 号 §10.1）"}"#.as_bytes(),
            "403 Forbidden",
            "no-store",
            None,
            want_gzip,
        ));
    }
    let ct_ok = content_type.is_some_and(|c| {
        let base = c.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        base == "application/json"
    });
    if !ct_ok {
        return Err(header_body(
            "application/json",
            r#"{"ok":false,"error":"400: Content-Type 必须为 application/json（22 号 §10.3）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        ));
    }
    if body.len() > 65_536 {
        return Err(header_body(
            "application/json",
            r#"{"ok":false,"error":"413: 请求体超 64 KiB 上限（22 号 §10.1）"}"#.as_bytes(),
            "413 Payload Too Large",
            "no-store",
            None,
            want_gzip,
        ));
    }
    serde_json::from_slice::<Value>(body).map_err(|e| {
        let text = format!(r#"{{"ok":false,"error":"400: body 非合法 JSON：{e}"}}"#);
        header_body(
            "application/json",
            text.as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        )
    })
}

/// HTTP 状态短语（400/403/413/500；其余兜底 400）。
fn code_text(code: u16) -> &'static str {
    match code {
        403 => "Forbidden",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        _ => "Bad Request",
    }
}
