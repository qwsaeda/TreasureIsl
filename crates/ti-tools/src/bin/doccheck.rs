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
        // 控件记录总数：S1.4b 实测 2041（ref/controls.json record_total，evidence/S1.4b.log 实跑）
        NumericAssertion {
            name: "控件记录总数|2041".to_string(),
            expected: "2041".to_string(),
            scope: Vec::new(),
        },
        // LCG 乘数 hex（第 4-5 轮审计固化：214013 = 0x343FD；0x3443D 为误算旧值）
        NumericAssertion {
            name: "0x3443D|0x343FD".to_string(),
            expected: "0x343FD".to_string(),
            scope: Vec::new(),
        },
        // save.magic（REC_MAGIC 十进制）：balance.json / 文档必须 = 20011591（0x01315A47）；
        // 旧值 20125007 为笔误（第 5 轮审计 Q092 发现，已修正 balance.json）
        NumericAssertion {
            name: "20125007|20011591".to_string(),
            expected: "20011591".to_string(),
            scope: Vec::new(),
        },
    ]
}

/// 参与数据一致性扫描的 JSON 数据文件（相对仓库根）。
const DATA_FILES: &[&str] = &["assets/data/balance.json", "assets/data/strings.json"];

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
    // 数据文件（JSON）也参与数值断言扫描，保证 balance.json 与文档数值一致
    for entry in DATA_FILES {
        let path = root.join(entry);
        if path.is_file() {
            push_doc(&path, &path, &mut docs);
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
    if !matches!(ext, "md" | "yaml" | "json") {
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

/// 台账计数漂移检查（第 5 轮审计 Q021 固化）：
/// 文档里「N 任务：M verified」统计行必须与 ledger 实测一致。
/// 防 release.md 等文档在台账演进后计数漂移（已复现 2 次：65→74→75）。
fn check_ledger_counts(docs: &[Doc], tasks: &[Task]) -> Vec<String> {
    let mut errors = Vec::new();
    let total = tasks.len();
    let verified = tasks.iter().filter(|t| t.status == Status::Verified).count();
    // 逐行提取「(\d+) 任务」与「(\d+) verified」的声明值，与实测比对
    for doc in docs {
        if !doc.path.contains("release.md") && !doc.path.contains("ledger") {
            continue;
        }
        for (i, line) in doc.text.lines().enumerate() {
            if !line.contains("任务") || !line.contains("verified") {
                continue;
            }
            // 提取「N 任务」：找「任务」前的数字
            if let Some(declared_total) = num_before(line, "任务") {
                if declared_total != total {
                    errors.push(format!(
                        "{}:{} 台账任务总数声明 {declared_total}，ledger 实测 {total}（第 5 轮审计 Q021 固化）",
                        doc.path,
                        i + 1
                    ));
                }
            }
            // 提取「M verified」：找「verified」前的数字
            if let Some(declared_ver) = num_before(line, "verified") {
                if declared_ver != verified {
                    errors.push(format!(
                        "{}:{} verified 计数声明 {declared_ver}，ledger 实测 {verified}",
                        doc.path,
                        i + 1
                    ));
                }
            }
        }
    }
    errors
}

/// 提取字符串中 `word` 紧邻前方的完整十进制数（如「78 任务」→ 78，取完整多位数）。
fn num_before(line: &str, word: &str) -> Option<usize> {
    let idx = line.find(word)?;
    let before = &line[..idx];
    // 从 word 前向回扫，取连续的 ASCII 数字串（允许数字与 word 之间有空白）
    let mut end = before.len();
    while end > 0 {
        let c = before[..end].chars().last().unwrap_or('\0');
        if c.is_ascii_whitespace() {
            end -= 1;
            continue;
        }
        break;
    }
    let mut start = end;
    while start > 0 {
        let c = before[..start].chars().last().unwrap_or('\0');
        if c.is_ascii_digit() {
            start -= 1;
        } else {
            break;
        }
    }
    if start >= end {
        return None;
    }
    before[start..end].parse().ok()
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
    report.errors.extend(check_ledger_counts(&docs, &tasks));

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
