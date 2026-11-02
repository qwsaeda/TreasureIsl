//! S32.8 文档检索：全文搜索 + EXP/F-ID/任务 ID 反查（只读）。
//!
//! - 搜索根 = 仓库内白名单相对目录（`docs/ crates/ scripts/ plan/`），不扫 `target/`/
//!   `node_modules/`/`.git/`（构建产物/依赖目录跳过）；
//! - 全文搜索 = 关键词大小写不敏感命中（行级），返回 `file:line + snippet`；
//! - EXP/F-ID/任务 ID 反查 = `q` 形如 `EXP-\d+` / `F-\d+` / `S\d+\.\d+` 时按**精确引用**
//!   匹配（同关键词命中，但语义 = 反查该 ID 出现在哪些文档/代码）；
//! - 索引上限防阻塞：文件数 / 单文件大小 / 总命中三限，超限 = `truncated` 标志（不报错、
//!   不阻塞——检索是只读辅助面，22 号 §9「只读体检」口径）；
//! - 本模块 0 写盘（纯读取 + 内存组装），0 游戏依赖（只碰 `std` + `serde_json`）。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// 搜索根（仓库相对目录；白名单面 = 文档/代码/工具/计划；数据驱动替代 = `console_sources.json`
/// 的 `search_roots` 段（当前未登记，S32.8 起按此约定登记））。
const SEARCH_ROOTS: &[&str] = &["docs", "crates", "scripts", "plan"];
/// 文本扩展名白名单（跳过二进制/图片/压缩包；含平台自身 web/index.html + 工具脚本，
/// 使 `/api/build`、`build_apk`、EXP/F 提及等引用可被检索到）。
const TEXT_EXT: &[&str] =
    &["md", "rs", "py", "ps1", "json", "yaml", "yml", "toml", "csv", "html", "js", "sh", "txt"];
/// 扫描文件数上限（防阻塞）。
const MAX_FILES: usize = 3000;
/// 单文件大小上限（跳过超大文件，防一次性读爆内存）。
const MAX_FILE_BYTES: u64 = 512 * 1024;
/// 总命中上限（防响应体膨胀）。
const MAX_HITS: usize = 500;

/// 全文搜索（只读，0 写盘）。`q` 空 = 空结果（不扫盘）。
#[must_use]
pub fn search(repo_root: &Path, query: &str) -> Value {
    let query = query.trim();
    if query.is_empty() {
        return json!({
            "query": "",
            "hits": [],
            "truncated": false,
            "scanned_files": 0,
            "note": "空关键词（query 必填）",
        });
    }
    let ql = query.to_ascii_lowercase();
    let is_ref = looks_like_ref(query);
    let mut hits: Vec<Value> = Vec::new();
    let mut scanned = 0usize;
    let mut truncated = false;

    'roots: for root in SEARCH_ROOTS {
        let dir = repo_root.join(root);
        if !dir.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        collect_files(&dir, &mut files);
        for f in files {
            if scanned >= MAX_FILES {
                truncated = true;
                break 'roots;
            }
            scanned += 1;
            let Ok(meta) = f.metadata() else {
                continue;
            };
            if meta.len() > MAX_FILE_BYTES {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&f) else {
                continue;
            };
            for (i, line) in text.lines().enumerate() {
                let ll = line.to_ascii_lowercase();
                // 命中：普通关键词 = 子串；反查（EXP-/F-/任务 ID）= 精确 token 匹配
                //（`is_ref` 时走 `ref_in_line`，避免 `EXP-116` 误配 `EXP-1160`）。
                let matched = if is_ref { ref_in_line(&ll, query) } else { ll.contains(&ql) };
                if matched {
                    hits.push(hit(repo_root, &f, i + 1, line));
                    if hits.len() >= MAX_HITS {
                        truncated = true;
                        break 'roots;
                    }
                }
            }
        }
    }
    json!({
        "query": query,
        "hits": hits,
        "truncated": truncated,
        "scanned_files": scanned,
        "note": "只读全文检索（docs/ crates/ scripts/ plan/）；EXP-/F-/任务 ID 反查 = 精确引用匹配",
    })
}

/// 单个命中（仓库相对路径 + 行号 + 行片段）。
fn hit(repo_root: &Path, file_path: &Path, line_no: usize, line: &str) -> Value {
    let rel = file_path
        .strip_prefix(repo_root)
        .map_or_else(|_| file_path.display().to_string(), |p| p.display().to_string());
    let snippet: String = line.chars().take(200).collect();
    json!({ "file": rel, "line": line_no, "snippet": snippet })
}

/// 递归收集白名单扩展名文件（跳过 `target/`/`node_modules/`/`.git/` 构建产物目录）。
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name == "target" || name == "node_modules" || name == ".git" {
                    continue;
                }
                stack.push(p);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if TEXT_EXT.contains(&ext) {
                    out.push(p);
                }
            }
        }
    }
}

/// `query` 是否形如 EXP/F/任务 ID 引用（反查模式）。
fn looks_like_ref(query: &str) -> bool {
    let upper = query.to_ascii_uppercase();
    if let Some(rest) = upper.strip_prefix("EXP-") {
        return !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit());
    }
    if upper.starts_with('F') && query.contains('-') {
        return query[1..]
            .strip_prefix('-')
            .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()));
    }
    // 任务 ID：S<num>.<num>
    let b = upper.as_bytes();
    b.len() >= 4
        && b[0] == b'S'
        && b[1..].contains(&b'.')
        && b[1..].iter().all(|&c| c == b'.' || c.is_ascii_digit())
}

/// 反查精确引用：`query` 作为**独立 token** 出现（前后字符不是 ID 延续 = 非字母数字/连字符），
/// 避免 `EXP-116` 误配 `EXP-1160`、`F-319` 误配 `F-3190`。调用方已小写化行与 query。
fn ref_in_line(lower_line: &str, query: &str) -> bool {
    let ql = query.to_ascii_lowercase();
    if ql.is_empty() {
        return false;
    }
    let bytes = lower_line.as_bytes();
    let qb = ql.as_bytes();
    let mut start = 0usize;
    while start + qb.len() <= bytes.len() {
        let Some(pos) = lower_line[start..].find(ql.as_str()) else {
            break;
        };
        let abs = start + pos;
        let before_ok = abs == 0 || !is_ref_cont(bytes[abs - 1]);
        let after = abs + qb.len();
        let after_ok = after >= bytes.len() || !is_ref_cont(bytes[after]);
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
    }
    false
}

/// 反查 token 延续字符（ID 字符 = 字母数字或连字符；前后遇此 = 非独立 token，不匹配）。
fn is_ref_cont(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-'
}
