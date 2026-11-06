//! 内容运营向导（S44.2/S44.3）：地图工作台 + UI 工作台的固定流程状态机。

use std::path::Path;

use serde_json::Value;

pub mod docmap;
pub mod map_wizard;
pub mod publish;
pub mod ui_wizard;

/// `POST /api/content/{map,ui}/step`（S44.2/S44.3 向导）HTTP handler：执行一步（调白名单动作 + 机器判据 + 写点探针）。
/// body：`{"id","step","enable_write"?,"params"?}`。写操作鉴权（`403`）→ Content-Type application/json（`400`）→ 不可跳步（`409`）/ 写动作无 `enable_write`（`400`）。
/// `is_ui` 选择地图/UI 向导（两 handler 同构，合并 DRY；`routes.rs` 行数守 `audit_code` 900 上限）。
pub(crate) fn handle_wizard_step(
    repo_root: &Path,
    body: &[u8],
    token: Option<&str>,
    content_type: Option<&str>,
    want_gzip: bool,
    is_ui: bool,
) -> Vec<u8> {
    // 写路由须 X-TI-Token（22 号 §10.1）；缺 token ⇒ 403。镜像 /api/audit/taskbook 鉴权面；
    // 该路由为写点（assets/ + dist/ + evidence/ + 向导状态），非「免 token 唯一写端点」（22 号 L359 仅 /api/map/import 豁免）。
    if !crate::routes::auth_ok(token) {
        return crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"403: 写操作需 X-TI-Token（22 号 §10.1）"}"#.as_bytes(),
            "403 Forbidden",
            "no-store",
            None,
            want_gzip,
        );
    }
    // F4 修复（B 审计 r1 P2）：Content-Type 必须 application/json（镜像 handle_gate/handle_audit_taskbook，22 号 §10.3）。
    let ct_ok = content_type.is_some_and(|c| {
        let base = c.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        base == "application/json"
    });
    if !ct_ok {
        return crate::routes::header_body(
            "application/json",
            r#"{"ok":false,"error":"400: Content-Type 必须为 application/json（22 号 §10.3）"}"#
                .as_bytes(),
            "400 Bad Request",
            "no-store",
            None,
            want_gzip,
        );
    }
    let payload = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v,
        Err(e) => {
            return crate::routes::status_json(
                400,
                &serde_json::json!({"ok": false, "error": format!("body 非合法 JSON：{e}")}),
                want_gzip,
            );
        }
    };
    let id = payload.get("id").and_then(Value::as_str).unwrap_or("").to_owned();
    let step = payload.get("step").and_then(Value::as_str).unwrap_or("").to_owned();
    let enable_write = payload.get("enable_write").and_then(Value::as_bool).unwrap_or(false);
    let params = payload.get("params").cloned().unwrap_or(serde_json::json!({}));
    let result = if is_ui {
        ui_wizard::run_step(repo_root, &id, &step, enable_write, &params)
    } else {
        map_wizard::run_step(repo_root, &id, &step, enable_write, &params)
    };
    match result {
        Ok(v) => crate::routes::status_json(200, &v, want_gzip),
        Err((code, msg)) => crate::routes::status_json(
            code,
            &serde_json::json!({"ok": false, "error": msg}),
            want_gzip,
        ),
    }
}

/// 白名单动作解析结果：runner + 完整参数（逐参数；禁 shell 拼接）。
type CmdSpec = (String, Vec<String>);

/// 从白名单解析动作 → (runner, 完整参数)（逐参数；禁 shell 拼接）。
/// 返回 `None` 表示动作是平台自身面（self runner），由调用方自行处理。
/// 供 `map_wizard` / `ui_wizard` 共享（30 号 §3.1 白名单解析口径）。
pub(crate) fn resolve_action(
    repo_root: &Path,
    action_id: &str,
) -> Result<Option<CmdSpec>, (u16, String)> {
    let cmds = crate::runner::close::read_commands(repo_root)
        .map_err(|e| (500u16, format!("读白名单失败：{e}")))?;
    let c = cmds
        .iter()
        .find(|c| c.get("id").and_then(Value::as_str) == Some(action_id))
        .ok_or_else(|| (400u16, format!("白名单动作 `{action_id}` 未登记（30 号 §3.1）")))?;
    let runner = c.get("runner").and_then(Value::as_str).unwrap_or("").to_owned();
    if runner == "self" {
        return Ok(None);
    }
    let script = c.get("script").and_then(Value::as_str).unwrap_or("").to_owned();
    let mut args: Vec<String> = c
        .get("args")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default();
    // args 已含 script 时不前置（同 map_cmd_id 口径）。
    let script_in_args = args.iter().any(|a| a == &script);
    if !script.is_empty() && !script_in_args {
        args.insert(0, script);
    }
    Ok(Some((runner, args)))
}
