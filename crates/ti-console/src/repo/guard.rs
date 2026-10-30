//! repo 路径白名单 + `canonicalize` 防穿越（22 号 §10.2）：
//!
//! 比较对象 = **`canonicalize(repo_root)` 自身**（仓库根可能是符号链接，Windows 另有
//! 大小写/分隔符差异 ⇒ 不 canonicalize 会误判逃逸）；被访问路径 canonicalize 后
//! 必须仍以该值为前缀；`..` 段与白名单外路径 = 400/404 语义（返回 `Err`，HTTP 层映射）。

use std::path::{Path, PathBuf};

/// 路径逃逸/白名单外错误（HTTP 层映射 400；缺失文件由调用方映射 404）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathGuardError {
    /// 人类可读原因（日志用）。
    pub reason: String,
}

impl std::fmt::Display for PathGuardError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.reason)
    }
}

/// 判断 `candidate`（仓库根相对路径，可能含 `..`）是否合法且落在白名单目录集合内。
///
/// 白名单 = `console_sources.json` 的 `repo_sources`（目录项以 `/` 结尾 = 前缀匹配；
/// 文件项 = 精确匹配）。
///
/// # Errors
///
/// 返回 `Err` = 逃逸（`..` 段）或白名单外（带原因，供调用方映射 400/404）。
pub fn check_relative(relative: &str, repo_sources: &[String]) -> Result<(), PathGuardError> {
    let clean = relative.trim_start_matches('/');
    if clean.is_empty() {
        return Err(PathGuardError { reason: "空路径".to_owned() });
    }
    // 白名单匹配：目录项 = 前缀；文件项 = 精确。
    let in_whitelist = repo_sources.iter().any(|entry| {
        let entry = entry.trim_start_matches('/');
        if entry.ends_with('/') { clean.starts_with(entry) } else { clean == entry }
    });
    if !in_whitelist {
        return Err(PathGuardError {
            reason: format!("路径 {clean} 不在数据源白名单内")
        });
    }
    // `..` 段 = 逃逸（与 22 号 §10.2 的 canonicalize 双保险：白名单前缀已挡，
    // 但 `docs/../etc` 型前缀命中须再拒）。
    if clean.split('/').any(|part| part == "..") {
        return Err(PathGuardError {
            reason: format!("路径 {clean} 含 `..` 段（穿越拒绝）")
        });
    }
    Ok(())
}

/// 把 `repo_root` 相对路径解析为**已 canonicalize** 的仓库内路径。
///
/// 三步（22 号 §10.2）：
/// 1. `check_relative`（白名单 + `..` 段）；
/// 2. `canonicalize(repo_root.join(relative))`（仓库根自身先 canonicalize，
///    被访问路径 join 后再 canonicalize）；
/// 3. 前缀校验：结果必须以 `canonicalize(repo_root)` 为前缀（符号链接逃逸兜底）。
///
/// 文件缺失 = `Err`（调用方映射 404）；目录缺失同。
///
/// # Errors
///
/// `check_relative` 拒 / canonicalize 失败 / 前缀逃逸 / 路径缺失 = 带原因 `Err`。
pub fn resolve_guarded(
    repo_root: &Path,
    repo_sources: &[String],
    relative: &str,
) -> Result<PathBuf, PathGuardError> {
    check_relative(relative, repo_sources)?;
    let resolved =
        repo_root.join(relative.trim_start_matches('/')).canonicalize().map_err(|e| {
            PathGuardError {
                reason: format!("路径 {} 不存在或不可访问：{e}", relative.trim_start_matches('/')),
            }
        })?;
    let root = repo_root
        .canonicalize()
        .map_err(|e| PathGuardError { reason: format!("仓库根 canonicalize 失败：{e}") })?;
    if !resolved.starts_with(&root) {
        return Err(PathGuardError {
            reason: format!("路径 {relative} 经 canonicalize 后逃逸仓库根 {}", root.display()),
        });
    }
    Ok(resolved)
}
