//! 内容运营向导（S44.2/S44.3）：地图工作台 + UI 工作台的固定流程状态机。

use std::path::Path;

use serde_json::Value;

pub mod map_wizard;
pub mod ui_wizard;

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
