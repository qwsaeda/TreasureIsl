//! 审计报告归档（22 号 §7.3 第 3 步）。
//!
//! 主线程把报告正文**粘贴**给平台（或主线程直接写回）；平台落盘
//! `evidence/console/audit-<task>-r<N>-<role>.md`（同文件名，覆盖写）。
//! 文件名校验：`audit-<task>-r<N>-<role>.md`（task 可含 `-`/`.`；末两段 = round/side）。
//! 防穿越：文件必须落在 `evidence/console/` 下（`audit-` 前缀 + 无 `/` `\` + 无 `..`）。
//! 防符号链接：写前 `symlink_metadata` 拒绝**既有** symlink（目标组件）；父目录组件无法经
//! 本 API 替换（file 参数锁定 `evidence/console/` 平铺名 + 无 `/` + 无 `..`，防穿越见上）。

use std::path::Path;

use crate::audit::judge::parse_report_filename;

/// 粘贴归档报告。
/// `file` = `evidence/console/audit-<task>-r<N>-<role>.md`（相对仓库根）。
/// `content` = 报告正文（非空）。
/// 返回落盘路径（相对仓库根）。
///
/// # Errors
///
/// - 空正文 / 文件名形态不符 / 含子目录分隔符 / 含 `..` / 符号链接逃逸 = `Err`
///   （防穿越 + 命名规则 + 防 symlink）。
pub fn archive(repo_root: &Path, file: &str, content: &str) -> Result<String, String> {
    if content.is_empty() {
        return Err("报告正文不能为空".to_owned());
    }
    // 文件名形态校验（task/round/side 结构）+ 防穿越。
    let stem = file.strip_prefix("evidence/console/").ok_or_else(|| {
        "file 必须以 evidence/console/ 前缀（22 号 §7.2 报告命名规则）".to_owned()
    })?;
    if stem.contains('/') || stem.contains('\\') {
        return Err("file 含子目录分隔符（防穿越，只允许 evidence/console/ 平铺）".to_owned());
    }
    let Some((_task, _round, _role)) = parse_report_filename(stem) else {
        return Err(
            "file 名不符合 audit-<task>-r<N>-<role>.md（task 可含 -/.，末两段 = r<N>/a|b）"
                .to_owned(),
        );
    };
    // 防穿越（.. / 绝对路径）；已由 parse 结构 + 前缀 + 无分隔符覆盖。
    if stem.contains("..") {
        return Err("file 含 `..`（防穿越）".to_owned());
    }
    let abs = repo_root.join(file);
    // 防符号链接：目标文件若已是 symlink，`std::fs::write` 会跟随写到链接目标（仓库外）。
    // 用 `OpenOptions` + `O_NOFOLLOW`（Linux）/ 先 `symlink_metadata` 拒绝既有 symlink。
    let meta = std::fs::symlink_metadata(&abs);
    if let Ok(m) = &meta {
        if m.file_type().is_symlink() {
            return Err("file 是符号链接（防 symlink 逃逸）".to_owned());
        }
    }
    std::fs::write(&abs, content).map_err(|e| format!("报告写入失败：{e}"))?;
    Ok(file.to_owned())
}
