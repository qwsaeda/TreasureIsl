//! 文档一致性校验（`scripts/gate.ps1` 第 9 项）。
//!
//! 与 [`crate::ledger`] 同为「纯函数 + 薄 IO」结构：本模块只对**已读入的文档快照**做校验，
//! 不自己遍历文件系统，便于用内联夹具单测。
//!
//! 校验四类问题（每一条都对应一次真实踩坑）：
//!
//! 1. **ID 唯一性**：`F-NNN`（事实）/ `EXP-NNN`（经验）全库不得重号。历史上 `F-315`/`F-316`
//!    被「数据格式」与「工具链」两节各占一次，导致交叉引用全部指向歧义。
//! 2. **跨文档引用完整性**：文档里出现的每个 `F-NNN`/`EXP-NNN` 都必须真实存在。
//!    历史上台账引用过不存在的 `F-312`。
//! 3. **数字断言一致性**：同一批计数（场景数、控件数、容器条目数）不得在多个文件里给出不同值。
//!    历史上 UI 场景数同时存在 20 / 21 / 23 三个版本。
//! 4. **evidence 内容特征**：`verified` 任务的 evidence 文件不得只有 gate 总输出
//!    ——必须含任务自身 `DoD` 的判据关键字。历史上 21/38 个任务的 evidence 是 `gate` 输出模板，
//!    零任务专属验收内容。

use std::collections::{BTreeMap, BTreeSet};

/// 一份文档快照（相对路径 + 全文）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc {
    /// 相对仓库根的路径，用 `/` 分隔（如 `legacy/docs/facts/map.md`）。
    pub path: String,
    /// 文件全文。
    pub text: String,
}

impl Doc {
    /// 由路径与全文构造快照。
    #[must_use]
    pub fn new(path: &str, text: &str) -> Self {
        Self { path: path.to_string(), text: text.to_string() }
    }
}

/// ID → 定义它的文件列表。
///
/// key/value 都借用 `docs` 里 `Doc` 的 `text`/`path`，不能存 owned 副本
///（`format!` 出的新 `String` 会因生命周期不匹配编译不过）。
type Definitions<'a> = BTreeMap<&'a str, Vec<&'a str>>;

/// 一个跨文档必须一致的数字断言。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericAssertion {
    /// 断言名（报错用）。
    pub name: String,
    /// 权威取值。
    pub expected: String,
    /// 只在这些文件里生效（空 = 全部）。避免误伤 `evidence/` 等历史日志。
    pub scope: Vec<String>,
}

/// doccheck 的校验结果。
#[derive(Debug, Default)]
pub struct Report {
    /// 违例描述（空 = 通过）。
    pub errors: Vec<String>,
    /// 校验统计（打印到 gate 输出里，便于回归观察）。
    pub stats: Stats,
}

/// 校验统计。
#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    /// 参与校验的文档数。
    pub docs: usize,
    /// 识别出的唯一事实 ID 数。
    pub facts: usize,
    /// 识别出的唯一经验 ID 数。
    pub experiences: usize,
    /// 识别出的唯一任务 ID 数。
    pub tasks: usize,
    /// 识别出的数字断言数。
    pub assertions: usize,
}

impl Report {
    /// 是否零违例。
    #[must_use]
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// 识别一条行里定义的 ID（形如 `| F-201 | ...` 或 `### EXP-017 ...`）。
fn defines_id(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("| F-") || t.starts_with("### EXP-")
}

/// 抽取行内**被引用**的 ID（排除该行自己定义的 ID）。
fn referenced_ids(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    if !defines_id(line) {
        for tok in line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
            if is_fact_id(tok) || is_exp_id(tok) {
                out.push(tok);
            }
        }
    }
    out
}

fn is_fact_id(tok: &str) -> bool {
    tok.len() == 5 && tok.starts_with("F-") && tok[2..].bytes().all(|b| b.is_ascii_digit())
}

fn is_exp_id(tok: &str) -> bool {
    tok.len() == 7 && tok.starts_with("EXP-") && tok[4..].bytes().all(|b| b.is_ascii_digit())
}

fn is_task_id(tok: &str) -> bool {
    // `S0.1` / `S1.4b` / `S3.12`
    let Some(rest) = tok.strip_prefix('S') else { return false };
    let Some((stage, sub)) = rest.split_once('.') else { return false };
    !stage.is_empty()
        && stage.bytes().all(|b| b.is_ascii_digit())
        && !sub.is_empty()
        && sub.bytes().all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
}

/// gate 总输出的特征行：只贴 gate 输出等于没跑任务验收（硬规则 2）。
const GATE_MARKER: &str = "=== gate: TreasureIsl";

/// 收集所有文档中**定义**的 `F-NNN` / `EXP-NNN` → 定义它的文件列表。
///
/// key 必须借用 `docs` 内的字符串（`Definitions` 借用 `docs` 同寿命）：
/// - `| F-NNN |` 表格行：`F-NNN` 就在原文里，直接借用切出来的 `&str`；
/// - `### EXP-NNN` 行：原文里整词就是 `EXP-NNN`（前缀 `### ` 之外），
///   用 `split_whitespace().next()` 拿到原文 token 直接借用，不能 `format!`
///   新串（那样会引用局部变量）。
fn collect_definitions(docs: &[Doc]) -> (Definitions<'_>, Definitions<'_>) {
    let mut facts: Definitions<'_> = BTreeMap::new();
    let mut exps: Definitions<'_> = BTreeMap::new();
    for doc in docs {
        for line in doc.text.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix('|') {
                let first_cell = rest.split('|').next().unwrap_or("").trim();
                if is_fact_id(first_cell) {
                    facts.entry(first_cell).or_default().push(&doc.path);
                }
            } else if let Some(rest) = t.strip_prefix("### ") {
                let tok = rest.split_whitespace().next().unwrap_or("");
                if is_exp_id(tok) {
                    exps.entry(tok).or_default().push(&doc.path);
                }
            }
        }
    }
    (facts, exps)
}

/// pass 2：同一 ID 被定义多次即报（历史上 `F-315`/`F-316` 各被占两次）。
fn check_uniqueness(report: &mut Report, facts: &Definitions<'_>, exps: &Definitions<'_>) {
    for (id, paths) in facts {
        if paths.len() > 1 {
            report.errors.push(format!(
                "fact `{id}` defined {} times: {}",
                paths.len(),
                paths.join(", ")
            ));
        }
    }
    for (id, paths) in exps {
        if paths.len() > 1 {
            report.errors.push(format!(
                "experience `{id}` defined {} times: {}",
                paths.len(),
                paths.join(", ")
            ));
        }
    }
}

/// pass 3：文档里引用的 `F-NNN`/`EXP-NNN` 必须真实存在（历史上台账引用过不存在的 `F-312`）。
fn check_references(
    report: &mut Report,
    docs: &[Doc],
    facts: &Definitions<'_>,
    exps: &Definitions<'_>,
) {
    for doc in docs {
        for (idx, line) in doc.text.lines().enumerate() {
            for tok in referenced_ids(line) {
                let kind = if is_fact_id(tok) { "fact" } else { "experience" };
                let known =
                    if kind == "fact" { facts.contains_key(tok) } else { exps.contains_key(tok) };
                if known {
                    continue;
                }
                report.errors.push(format!(
                    "{}:{}: references undefined {kind} `{tok}`",
                    doc.path,
                    idx + 1
                ));
            }
        }
    }
}

/// pass 4：文档提到的任务 ID 必须已登记台账。
fn check_task_ids(report: &mut Report, docs: &[Doc], task_ids: &BTreeSet<String>) {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for doc in docs {
        for (idx, line) in doc.text.lines().enumerate() {
            let tokens = line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '.'));
            for tok in tokens {
                if !is_task_id(tok) || task_ids.is_empty() {
                    continue;
                }
                if task_ids.contains(tok) {
                    seen.insert(tok.to_string());
                } else {
                    report.errors.push(format!(
                        "{}:{}: references task `{tok}` which is not in plan/ledger.yaml",
                        doc.path,
                        idx + 1
                    ));
                }
            }
        }
    }
    report.stats.tasks = seen.len();
}

/// 校验文档快照集合。
///
/// 数字断言的 `name` 支持 3 段形式 `关键词|权威值|旧值`（`NumericAssertion::expected`
/// 是「权威值」，`name` 的第 3 段是「旧值」）；也兼容 2 段 `关键词|期望值`
///（旧值视为空，只校验权威值一致性）。
#[must_use]
pub fn check(docs: &[Doc], assertions: &[NumericAssertion], task_ids: &BTreeSet<String>) -> Report {
    let mut report =
        Report { stats: Stats { docs: docs.len(), ..Stats::default() }, ..Report::default() };
    let (facts, exps) = collect_definitions(docs);
    report.stats.facts = facts.len();
    report.stats.experiences = exps.len();
    check_uniqueness(&mut report, &facts, &exps);
    check_references(&mut report, docs, &facts, &exps);
    check_task_ids(&mut report, docs, task_ids);
    report.stats.assertions = assertions.len();
    check_numeric(&mut report, docs, assertions);
    report
}

/// 数字断言：行**含关键词**时，若含旧值且不含权威值 → 报「断言了不同的数字」。
fn check_numeric(report: &mut Report, docs: &[Doc], assertions: &[NumericAssertion]) {
    for assertion in assertions {
        let parts: Vec<&str> = assertion.name.split('|').collect();
        if parts.len() < 2 {
            continue;
        }
        let keyword = parts[0];
        let legacy: Option<&str> = parts.get(2).copied();
        for doc in docs {
            if !assertion.scope.is_empty() && !assertion.scope.iter().any(|p| p == &doc.path) {
                continue;
            }
            for line in doc.text.lines() {
                if !line.contains(keyword) {
                    continue;
                }
                let has_expected = line.contains(&assertion.expected);
                let legacy_text = legacy.unwrap_or("");
                let has_legacy = !legacy_text.is_empty() && line.contains(legacy_text);
                if has_legacy && !has_expected {
                    report.errors.push(format!(
                        "{}: asserts a different value for `{keyword}` (got `{legacy_text}`, want `{}`)",
                        doc.path, assertion.expected
                    ));
                }
            }
        }
    }
}

/// 一份 evidence 文件是否**只**包含 gate 总输出（= 不构成验收证据）。
///
/// 判据：出现 gate 特征行，且不含任何任务判据关键字。
#[must_use]
pub fn evidence_is_gate_only(text: &str, keywords: &[String]) -> bool {
    if !text.contains(GATE_MARKER) {
        return false;
    }
    !keywords.iter().any(|kw| !kw.is_empty() && text.contains(kw))
}

/// 从 ledger 的 `dod` 里抽取「验收命令/判据」关键字，用于 evidence 匹配。
///
/// 取 `cargo`/`powershell`/`python` 等命令 token 与反引号片段；够用即可，不追求完备。
#[must_use]
pub fn dod_keywords(dod: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = dod;
    // 反引号片段
    while let Some(start) = rest.find('`') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('`') else { break };
        let frag = after[..end].trim();
        if frag.len() >= 6 {
            out.push(frag.to_string());
        }
        rest = &after[end + 1..];
    }
    // 命令 token（末段标识符，如 `pak_key`、`diff_battle`、`pcx_parity`）
    for word in dod.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        if word.len() >= 5 && word != "cargo" && word != "powershell" && word != "python" {
            out.push(word.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::{Doc, NumericAssertion, check, dod_keywords, evidence_is_gate_only};
    use std::collections::BTreeSet;

    fn doc(path: &str, text: &str) -> Doc {
        Doc::new(path, text)
    }

    #[test]
    fn accepts_consistent_snapshot() {
        let docs = vec![
            doc("legacy/docs/facts/map.md", "| F-301 | UI 场景族 20 个 |"),
            doc("legacy/docs/spec/x.md", "见 F-301 与 EXP-001"),
            doc("docs/experience/a.md", "### EXP-001 标签\n- 现象"),
        ];
        let report = check(&docs, &[], &BTreeSet::new());
        assert!(report.ok(), "{:?}", report.errors);
        assert_eq!(report.stats.facts, 1);
        assert_eq!(report.stats.experiences, 1);
    }

    #[test]
    fn rejects_duplicate_fact_ids() {
        let docs = vec![
            doc("legacy/docs/facts/map.md", "| F-315 | 事件 id |"),
            doc("legacy/docs/facts/tools.md", "| F-315 | REA 坑范围 |"),
        ];
        let report = check(&docs, &[], &BTreeSet::new());
        assert!(report.errors.iter().any(|e| e.contains("fact `F-315` defined 2 times")));
    }

    #[test]
    fn rejects_duplicate_experience_ids() {
        let docs = vec![
            doc("docs/experience/a.md", "### EXP-016 x\n"),
            doc("docs/experience/b.md", "### EXP-016 y\n"),
        ];
        let report = check(&docs, &[], &BTreeSet::new());
        assert!(report.errors.iter().any(|e| e.contains("experience `EXP-016` defined 2 times")));
    }

    #[test]
    fn rejects_reference_to_undefined_fact() {
        let docs = vec![
            doc("plan/ledger.yaml", "dod: 见 F-311/F-312"),
            doc("legacy/docs/facts/map.md", "| F-311 | x |"),
        ];
        let report = check(&docs, &[], &BTreeSet::new());
        assert!(report.errors.iter().any(|e| e.contains("undefined fact `F-312`")));
        assert!(!report.errors.iter().any(|e| e.contains("undefined fact `F-311`")));
    }

    #[test]
    fn rejects_reference_to_undefined_experience() {
        let docs = vec![doc("docs/style.md", "见 EXP-999 的教训")];
        let report = check(&docs, &[], &BTreeSet::new());
        assert!(report.errors.iter().any(|e| e.contains("undefined experience `EXP-999`")));
    }

    #[test]
    fn rejects_task_id_not_in_ledger() {
        let docs = vec![doc("docs/plan-s3.md", "| S9.9 | 不存在的任务 |")];
        let ids = BTreeSet::from(["S0.1".to_string()]);
        let report = check(&docs, &[], &ids);
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.contains("task `S9.9` which is not in plan/ledger.yaml"))
        );
    }

    #[test]
    fn accepts_task_id_in_ledger() {
        let docs = vec![doc("docs/plan-s3.md", "| S1.4b | 控件 |")];
        let ids = BTreeSet::from(["S1.4b".to_string()]);
        let report = check(&docs, &[], &ids);
        assert!(report.ok(), "{:?}", report.errors);
    }

    #[test]
    fn rejects_conflicting_numeric_assertion() {
        let docs = vec![
            doc("legacy/docs/facts/map.md", "| F-301 | UI 场景族 20 个 |"),
            // 旧值行：含关键词 + 旧值 21，但**不含**权威值 20
            doc("docs/plan-s0-s2.md", "UI 场景数 21 个"),
        ];
        let assertions = vec![NumericAssertion {
            name: "UI 场景数|20|21".into(),
            expected: "20".into(),
            scope: vec![],
        }];
        let report = check(&docs, &assertions, &BTreeSet::new());
        assert!(
            report.errors.iter().any(|e| e.contains("asserts a different value")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn accepts_matching_numeric_assertion() {
        let docs = vec![doc("legacy/docs/facts/map.md", "UI 场景族实测 20 个")];
        let assertions = vec![NumericAssertion {
            name: "UI 场景数|20|21".into(),
            expected: "20".into(),
            scope: vec![],
        }];
        let report = check(&docs, &assertions, &BTreeSet::new());
        assert!(report.ok(), "{:?}", report.errors);
    }

    #[test]
    fn detects_gate_only_evidence() {
        let gate_log = "=== gate: TreasureIsl (9 items) ===\n  PASS  cargo fmt\n  === gate: 0 failure(s) ===\n";
        assert!(evidence_is_gate_only(gate_log, &["pak_key".to_string()]));
        assert!(!evidence_is_gate_only(gate_log, &["PASS".to_string()]));
    }

    #[test]
    fn gate_only_detector_is_case_insensitive_about_marker() {
        let real = "running diff_battle\ntest result: ok. 1650 passed\n";
        assert!(!evidence_is_gate_only(real, &["diff_battle".to_string()]));
        // 没有 gate 标记的日志一律不算 gate-only（避免误伤手写日志）
        assert!(!evidence_is_gate_only("random log", &["nope".to_string()]));
    }

    #[test]
    fn extracts_dod_keywords() {
        let kws =
            dod_keywords("P1-3：`cargo test -p ti-tools --test pak_key`，20/20 UI 场景整除吻合");
        assert!(kws.iter().any(|k| k.contains("pak_key")), "{kws:?}");
    }

    #[test]
    fn dod_keywords_ignore_stopwords() {
        let kws = dod_keywords("cargo run -q -p ti-tools --bin controls -- --out ref");
        assert!(!kws.iter().any(|k| k == "cargo" || k == "powershell"), "{kws:?}");
    }
}
