//! `console_sources.json`（仓库根相对路径清单）加载与解析（22 号 §5/§10.2 单一事实源）。
//!
//! 路径清单 = 本文件解析结果的 `RepoSources`；Rust 侧不硬编码 `assets/xxx.json`
//! 字面量（CS-1 口径，22 号 §11.2）。

use std::path::Path;

use serde_json::Value;

/// 数据源路径清单（`console_sources.json` 解析结果）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoSources {
    /// 只读数据源（仓库根相对路径；目录项以 `/` 结尾）。
    pub repo_sources: Vec<String>,
    /// 写点白名单（当前唯一 = `evidence/console/`；平台其余 0 写盘）。
    pub write_whitelist: Vec<String>,
}

impl RepoSources {
    /// 解析 `console_sources.json` 的 `Value`（纯函数，不做文件 IO）。
    /// # Errors
    ///
    /// `repo_sources` / `write_whitelist` 段缺失或类型不符 = 带原因 `Err`。
    pub fn from_json(value: &Value) -> Result<Self, String> {
        let obj = value.as_object().ok_or("console_sources.json 顶层必须是对象")?;
        let repo_sources = obj
            .get("repo_sources")
            .and_then(Value::as_array)
            .ok_or("console_sources.json 缺 `repo_sources` 数组")?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| "repo_sources 项必须是字符串路径".to_owned())
            })
            .collect::<Result<Vec<String>, String>>()?;
        if repo_sources.is_empty() {
            return Err("console_sources.json `repo_sources` 为空".to_owned());
        }
        let write_whitelist = match obj.get("write_whitelist") {
            Some(v) => v
                .as_array()
                .ok_or("write_whitelist 必须是数组")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| "write_whitelist 项必须是字符串路径".to_owned())
                })
                .collect::<Result<Vec<String>, String>>()?,
            None => Vec::new(),
        };
        Ok(Self { repo_sources, write_whitelist })
    }
}

/// 读取并解析 `repo_root` 下的 `crates/ti-console/console_sources.json`（缺文件/非法 JSON 返回 `Err`）。
/// # Errors
///
/// 文件缺/读失败/JSON 非法 = 带原因 `Err`（`console_sources.json` 单文件，不可缺）。
pub fn load(repo_root: &Path) -> Result<RepoSources, String> {
    let path = repo_root.join("crates/ti-console/console_sources.json");
    let raw =
        std::fs::read_to_string(&path).map_err(|e| format!("读 {} 失败：{e}", path.display()))?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("console_sources.json 非法 JSON：{e}"))?;
    RepoSources::from_json(&value)
}
