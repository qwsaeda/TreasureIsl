//! `plan/ledger.yaml` 的解析与校验（`docs/loop.md` §1，门禁第 8 项）。
//!
//! 只支持 `docs/loop.md` 规定的**单行 flow-mapping** 写法，例如：
//!
//! ```yaml
//!   - { id: S0.1, title: "建 workspace", status: pending, deps: [], dod: "gate 全绿", evidence: null, blocked_by: null, rounds: 0 }
//! ```
//!
//! 保持单行是有意为之：无需引入 YAML 依赖即可做严格校验，
//! 格式一旦变更必须同步更新本模块与 `docs/loop.md`。

use std::collections::HashSet;
use std::path::Path;

/// 任务状态（`docs/loop.md` §1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// 未开始。
    Pending,
    /// 进行中。
    InProgress,
    /// 阻塞（必须填 `blocked_by`）。
    Blocked,
    /// 已验收（必须填 `evidence`）。
    Verified,
}

impl Status {
    /// 从台账字面量解析状态；未知字面量返回 `None`。
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "blocked" => Some(Self::Blocked),
            "verified" => Some(Self::Verified),
            _ => None,
        }
    }

    /// 台账字面量表示。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Verified => "verified",
        }
    }
}

/// 台账中的一个任务。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// 任务 ID（如 `S0.1`）。
    pub id: String,
    /// 任务标题。
    pub title: String,
    /// 当前状态。
    pub status: Status,
    /// 依赖任务 ID，必须出现在本任务之前且已 `verified` 才可取用。
    pub deps: Vec<String>,
    /// 验收判据（抄自 WBS）。
    pub dod: String,
    /// 验收证据路径（`verified` 必填）。
    pub evidence: Option<String>,
    /// 阻塞原因（`blocked` 必填）。
    pub blocked_by: Option<String>,
    /// 已失败验收轮数。
    pub rounds: u32,
    /// 在文件中的行号（报错定位）。
    pub line: usize,
}

impl Task {
    /// 是否已验收。
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self.status, Status::Verified)
    }
}

/// 校验结果：解析出的任务 + 规则违例清单。
#[derive(Debug, Default)]
pub struct Report {
    /// 按文件顺序解析出的任务。
    pub tasks: Vec<Task>,
    /// 违例描述（空 = 通过）。
    pub errors: Vec<String>,
}

impl Report {
    /// 是否零违例。
    #[must_use]
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// 已验收任务数。
    #[must_use]
    pub fn verified_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.is_verified()).count()
    }
}

/// 解析并校验台账文本（纯函数，不做文件 IO）。
#[must_use]
pub fn validate(text: &str) -> Report {
    let mut report = Report::default();
    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if !trimmed.starts_with("- {") {
            continue;
        }
        match parse_task(trimmed, line_no) {
            Ok(task) => report.tasks.push(task),
            Err(err) => report.errors.push(format!("line {line_no}: {err}")),
        }
    }
    check_rules(&mut report);
    report
}

/// 返回 `verified` 但证据文件缺失的任务 ID（`evidence` 为空也计入）。
/// `evidence` 字段允许「路径 + 自由说明」，取第一个空白分词作为主路径。
#[must_use]
pub fn missing_evidence(tasks: &[Task], base: &Path) -> Vec<String> {
    tasks
        .iter()
        .filter(|t| t.is_verified())
        .filter(|t| match t.evidence.as_deref() {
            Some(path) if !path.is_empty() => {
                let first = path.split_whitespace().next().unwrap_or(path);
                !base.join(first).is_file()
            }
            _ => true,
        })
        .map(|t| t.id.clone())
        .collect()
}

const KNOWN_KEYS: [&str; 8] =
    ["id", "title", "status", "deps", "dod", "evidence", "blocked_by", "rounds"];

fn parse_task(trimmed: &str, line: usize) -> Result<Task, String> {
    let body = trimmed
        .strip_prefix("- {")
        .and_then(|s| s.strip_suffix('}'))
        .ok_or_else(|| "task line must be `- { ... }`".to_string())?;

    let mut id = None;
    let mut title = None;
    let mut status = None;
    let mut deps: Option<Vec<String>> = None;
    let mut dod = None;
    let mut evidence: Option<Option<String>> = None;
    let mut blocked_by: Option<Option<String>> = None;
    let mut rounds = None;

    for pair in split_flow(body) {
        let Some((key, value)) = pair.split_once(':') else {
            return Err(format!("`{pair}` is not `key: value`"));
        };
        let key = key.trim();
        let value = value.trim();
        if !KNOWN_KEYS.contains(&key) {
            return Err(format!("unknown key `{key}`"));
        }
        match key {
            "id" => id = Some(parse_word(value)?.to_string()),
            "title" => title = Some(parse_string(value)?.to_string()),
            "status" => status = Some(parse_word(value)?.to_string()),
            "dod" => dod = Some(parse_string(value)?.to_string()),
            "evidence" => evidence = Some(parse_opt_string(value)?),
            "blocked_by" => blocked_by = Some(parse_opt_string(value)?),
            "rounds" => rounds = Some(parse_number(value)?),
            "deps" => deps = Some(parse_list(value)?),
            _ => unreachable!("KNOWN_KEYS is exhaustive"),
        }
    }

    let id = id.ok_or("missing key `id`")?;
    let title = title.ok_or("missing key `title`")?;
    let status_raw = status.ok_or("missing key `status`")?;
    let status =
        Status::parse(&status_raw).ok_or_else(|| format!("invalid status `{status_raw}`"))?;
    let deps = deps.ok_or("missing key `deps`")?;
    let dod = dod.ok_or("missing key `dod`")?;
    let evidence = evidence.ok_or("missing key `evidence`")?;
    let blocked_by = blocked_by.ok_or("missing key `blocked_by`")?;
    let rounds = rounds.ok_or("missing key `rounds`")?;

    Ok(Task { id, title, status, deps, dod, evidence, blocked_by, rounds, line })
}

fn check_rules(report: &mut Report) {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut order: Vec<&str> = Vec::new();

    for task in &report.tasks {
        if !seen.insert(task.id.as_str()) {
            report.errors.push(format!("line {}: duplicate id `{}`", task.line, task.id));
        }
        order.push(task.id.as_str());
    }

    for task in &report.tasks {
        let at = format!("line {} ({})", task.line, task.id);

        for dep in &task.deps {
            match order.iter().position(|&id| id == dep) {
                None => report.errors.push(format!("{at}: unknown dependency `{dep}`")),
                Some(pos) => {
                    if pos >= order.iter().position(|&id| id == task.id).unwrap_or(usize::MAX) {
                        report
                            .errors
                            .push(format!("{at}: dependency `{dep}` must be listed earlier"));
                    }
                }
            }
        }

        match task.status {
            Status::Verified => match task.evidence.as_deref() {
                Some(path) if !path.is_empty() => {}
                _ => report.errors.push(format!("{at}: verified requires `evidence`")),
            },
            Status::Blocked => match task.blocked_by.as_deref() {
                Some(text) if !text.is_empty() => {}
                _ => report.errors.push(format!("{at}: blocked requires `blocked_by`")),
            },
            Status::Pending | Status::InProgress => {}
        }

        if task.rounds >= 3 {
            report.errors.push(format!(
                "{at}: rounds={} >= 3，必须拆成子任务后才能继续（docs/loop.md §1）",
                task.rounds
            ));
        }
    }
}

fn split_flow(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;
    let mut depth = 0usize;
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_str = !in_str;
                cur.push(c);
            }
            '[' | '{' if !in_str => {
                depth += 1;
                cur.push(c);
            }
            ']' | '}' if !in_str => {
                depth = depth.saturating_sub(1);
                cur.push(c);
            }
            ',' if !in_str && depth == 0 => {
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

fn parse_word(value: &str) -> Result<&str, String> {
    if value.is_empty() || value.contains(char::is_whitespace) {
        return Err(format!("expected a bare word, got `{value}`"));
    }
    Ok(value)
}

fn parse_string(value: &str) -> Result<&str, String> {
    let inner = value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or_else(|| format!("expected quoted string, got `{value}`"))?;
    Ok(inner)
}

fn parse_opt_string(value: &str) -> Result<Option<String>, String> {
    if value == "null" {
        return Ok(None);
    }
    Ok(Some(parse_string(value)?.to_string()))
}

fn parse_number(value: &str) -> Result<u32, String> {
    value.parse::<u32>().map_err(|_| format!("expected unsigned integer, got `{value}`"))
}

fn parse_list(value: &str) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| format!("expected list, got `{value}`"))?;
    let inner = inner.trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    Ok(inner.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

#[cfg(test)]
mod tests {
    use super::{Status, validate};

    const GOOD: &str = r#"
version: 1
tasks:
  - { id: S0.1, title: "first, with comma", status: pending, deps: [], dod: "gate green", evidence: null, blocked_by: null, rounds: 0 }
  - { id: S0.2, title: "second", status: verified, deps: [S0.1], dod: "csv rows match", evidence: "evidence/S0.2.log", blocked_by: null, rounds: 1 }
"#;

    #[test]
    fn parses_valid_ledger() {
        let report = validate(GOOD);
        assert!(report.ok(), "{:?}", report.errors);
        assert_eq!(report.tasks.len(), 2);
        assert_eq!(report.verified_count(), 1);
        assert_eq!(report.tasks[1].deps, vec!["S0.1".to_string()]);
        assert_eq!(report.tasks[1].evidence.as_deref(), Some("evidence/S0.2.log"));
    }

    #[test]
    fn rejects_unknown_status() {
        let text = GOOD.replace("status: pending", "status: done");
        let report = validate(&text);
        assert!(!report.ok());
        assert!(report.errors.iter().any(|e| e.contains("invalid status `done`")));
    }

    #[test]
    fn rejects_verified_without_evidence() {
        let text = r#"
tasks:
  - { id: S0.1, title: "x", status: verified, deps: [], dod: "d", evidence: null, blocked_by: null, rounds: 0 }
"#;
        let report = validate(text);
        assert!(report.errors.iter().any(|e| e.contains("requires `evidence`")));
    }

    #[test]
    fn rejects_blocked_without_reason() {
        let text = r#"
tasks:
  - { id: S0.1, title: "x", status: blocked, deps: [], dod: "d", evidence: null, blocked_by: null, rounds: 1 }
"#;
        let report = validate(text);
        assert!(report.errors.iter().any(|e| e.contains("requires `blocked_by`")));
    }

    #[test]
    fn rejects_forward_and_unknown_dependencies() {
        let text = r#"
tasks:
  - { id: S0.2, title: "a", status: pending, deps: [S0.1], dod: "d", evidence: null, blocked_by: null, rounds: 0 }
  - { id: S0.1, title: "b", status: pending, deps: [S9.9], dod: "d", evidence: null, blocked_by: null, rounds: 0 }
"#;
        let report = validate(text);
        assert!(report.errors.iter().any(|e| e.contains("must be listed earlier")));
        assert!(report.errors.iter().any(|e| e.contains("unknown dependency `S9.9`")));
    }

    #[test]
    fn rejects_three_failed_rounds() {
        let text = r#"
tasks:
  - { id: S0.1, title: "x", status: in_progress, deps: [], dod: "d", evidence: null, blocked_by: null, rounds: 3 }
"#;
        let report = validate(text);
        assert!(report.errors.iter().any(|e| e.contains("必须拆成子任务")));
    }

    #[test]
    fn rejects_duplicate_ids_and_unknown_keys() {
        let dup = r#"
tasks:
  - { id: S0.1, title: "a", status: pending, deps: [], dod: "d", evidence: null, blocked_by: null, rounds: 0 }
  - { id: S0.1, title: "b", status: pending, deps: [], dod: "d", evidence: null, blocked_by: null, rounds: 0 }
"#;
        assert!(validate(dup).errors.iter().any(|e| e.contains("duplicate id")));

        let unknown = r#"
tasks:
  - { id: S0.1, title: "a", status: pending, deps: [], dod: "d", evidence: null, blocked_by: null, rounds: 0, owner: me }
"#;
        assert!(validate(unknown).errors.iter().any(|e| e.contains("unknown key `owner`")));
    }

    #[test]
    fn status_roundtrips_through_literals() {
        for s in [Status::Pending, Status::InProgress, Status::Blocked, Status::Verified] {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
        assert_eq!(Status::parse("finished"), None);
    }
}
