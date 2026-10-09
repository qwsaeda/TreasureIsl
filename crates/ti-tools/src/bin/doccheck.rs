//! `gate.ps1` 第 9 项：文档一致性校验。
//!
//! 四类检查（详见 [`ti_tools::doccheck`]）：
//! 1. `F-NNN` / `EXP-NNN` 全库唯一，且文档里引用的 ID 都真实存在；
//! 2. 文档提到的任务 ID 都已登记 `plan/ledger.yaml`；
//! 3. 跨文档数字断言一致（场景数等计数不得有多个版本）；
//! 4. `verified` 任务的 evidence 文件不得只有 gate 总输出（硬规则 2）。
//!
//! 用法：`cargo run -q -p ti-tools -- doccheck [repo_root]`
//! （缺省 `repo_root = "."`。）

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_tools::doccheck::{Doc, NumericAssertion, check, dod_keywords, evidence_is_gate_only};
use ti_tools::ledger::{Status, Task, validate};

/// 参与扫描的文档根（相对仓库根）。
const DOC_ROOTS: &[&str] = &["docs", "AGENTS.md", "plan"];

/// 跨文档必须一致的数字断言。
///
/// 形式：`关键词|权威值|旧值`。行**含关键词**时参与校验：
/// - 行含权威值 → 一致，通过；
/// - 行含旧值且不含权威值 → 判「数字冲突」。
///
/// 历史：早期 3 段 `关键词|期望值|旧值` 因关键词误配（"UI 场景" 命中 "20 场景" 等）
/// 全库 10+ 文件误报；现收紧为**精确短语关键词**（带量词限定）+ 3 段权威/旧值。
fn numeric_assertions() -> Vec<NumericAssertion> {
    vec![
        // UI 场景数：F-301 权威 20，历史旧值 21（"21 场景"）与 23（"23 个 .Map" 已含 3 地形，UI 是 20）
        NumericAssertion {
            name: "UI 场景数|20|21".to_string(),
            expected: "20".to_string(),
            scope: Vec::new(),
        },
        // 控件记录总数：S1.4b 修订 2042，旧值 2041
        NumericAssertion {
            name: "控件记录总数|2042|2041".to_string(),
            expected: "2042".to_string(),
            scope: Vec::new(),
        },
    ]
}

fn read_docs(root: &Path) -> Vec<Doc> {
    let mut docs = Vec::new();
    for entry in DOC_ROOTS {
        let path = root.join(entry);
        if path.is_file() {
            push_doc(&path, &path, &mut docs);
        } else if path.is_dir() {
            collect(&path, root, &mut docs);
        }
    }
    docs
}

fn collect(dir: &Path, root: &Path, out: &mut Vec<Doc>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut items: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    items.sort();
    for path in items {
        if path.is_dir() {
            collect(&path, root, out);
        } else {
            push_doc(&path, root, out);
        }
    }
}

fn push_doc(path: &Path, root: &Path, out: &mut Vec<Doc>) {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else { return };
    if !matches!(ext, "md" | "yaml") {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else { return };
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
    out.push(Doc::new(&rel, &text));
}

/// 收集台账任务 ID 与 `verified` 任务的 (id, dod, evidence 路径)。
fn read_ledger(root: &Path) -> (BTreeSet<String>, Vec<Task>) {
    let path = root.join("plan/ledger.yaml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return (BTreeSet::new(), Vec::new());
    };
    let report = validate(&text);
    let ids = report.tasks.iter().map(|t| t.id.clone()).collect();
    (ids, report.tasks)
}

/// 检查 `verified` 任务的 evidence 是否只有 gate 总输出。
fn check_evidence(root: &Path, tasks: &[Task]) -> Vec<String> {
    let mut errors = Vec::new();
    for task in tasks {
        if task.status != Status::Verified {
            continue;
        }
        let Some(raw) = task.evidence.as_deref() else { continue };
        let Some(rel) = raw.split_whitespace().next() else { continue };
        let path = root.join(rel);
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let kws = dod_keywords(&task.dod);
        if evidence_is_gate_only(&text, &kws) {
            errors.push(format!(
                "{}: evidence `{rel}` 只有 gate 总输出，没有任务自身判据（硬规则 2）",
                task.id
            ));
        }
    }
    errors
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = args.first().map_or_else(|| PathBuf::from("."), PathBuf::from);

    let docs = read_docs(&root);
    if docs.is_empty() {
        eprintln!(
            "doccheck: no documents found under {} (cwd={:?})",
            root.display(),
            std::env::current_dir()
        );
        return ExitCode::FAILURE;
    }
    let (task_ids, tasks) = read_ledger(&root);
    if task_ids.is_empty() {
        eprintln!("doccheck: cannot read plan/ledger.yaml under {}", root.display());
        return ExitCode::FAILURE;
    }

    let mut report = check(&docs, &numeric_assertions(), &task_ids);
    report.errors.extend(check_evidence(&root, &tasks));

    println!(
        "doccheck: {} docs, {} facts, {} experiences, {} task refs, {} assertions, {} error(s)",
        report.stats.docs,
        report.stats.facts,
        report.stats.experiences,
        report.stats.tasks,
        report.stats.assertions,
        report.errors.len()
    );
    for err in &report.errors {
        println!("  ERROR {err}");
    }

    if report.ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
