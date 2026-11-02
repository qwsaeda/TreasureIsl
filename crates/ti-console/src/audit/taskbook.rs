//! 审计任务书生成（22 号 §7.3 第 1 步）。
//!
//! 平台生成 `evidence/console/audit-<task>-r<N>-<role>.prompt.md`（模板 + 台账 dod + 证据路径）；
//! 主线程用 harness 按任务书起两个子代理（A/B 并行、互不可见）。

use std::path::Path;

use crate::audit::judge::{HeadStatus, aggregate, parse_head_status, parse_report_filename};
use crate::audit::template::{cover_elements, render_taskbook};
use std::fmt::Write as _;

/// 找台账任务（id 精确匹配）。
/// 返回 `(title, dod, evidence)`。
fn find_task(repo_root: &Path, task: &str) -> Result<(String, String, String), String> {
    let path = repo_root.join("plan/ledger.yaml");
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("读 {} 失败：{e}", path.display()))?;
    let report = ti_devmeta::ledger::validate(&text);
    report
        .tasks
        .iter()
        .find(|t| t.id == task)
        .map(|t| (t.title.clone(), t.dod.clone(), t.evidence.clone().unwrap_or_default()))
        .ok_or_else(|| format!("台账无任务 `{task}`（plan/ledger.yaml）"))
}

/// 读指定任务的最新轮 findings（用于「上一轮 findings（需复核）」段；无 = 空串）。
fn latest_findings(repo_root: &Path, task: &str) -> String {
    let dir = repo_root.join("evidence/console");
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return String::new();
    };
    let mut metas = Vec::new();
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.to_lowercase().ends_with(".md") || name.ends_with(".prompt.md") {
            continue;
        }
        let Some((t, round, side)) = parse_report_filename(&name) else {
            continue;
        };
        if t != task {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
        // 祖父条款：r1~r3 人工口径。
        let legacy = round <= 3;
        let status = parse_head_status(&text, legacy);
        metas.push((round, side, status, format!("evidence/console/{name}")));
    }
    let rounds = aggregate(
        &metas
            .into_iter()
            .map(|(round, side, status, file)| crate::audit::judge::ReportMeta {
                task: task.to_owned(),
                round,
                side,
                file,
                status,
            })
            .collect::<Vec<_>>(),
    );
    let mut out = String::new();
    if let Some(r) = rounds.last() {
        for (side, st, file) in [('a', &r.a, &r.a_file), ('b', &r.b, &r.b_file)] {
            let f = file.clone().unwrap_or_default();
            match st {
                Some(HeadStatus::Parsed(fields)) => {
                    let new = fields.found.unwrap_or(0);
                    let res = fields.residual.unwrap_or(0);
                    if new > 0 || res > 0 {
                        let _ = writeln!(
                            out,
                            "- r{} {side} 侧：新发现 {new}，残留 {res}（{f}）",
                            r.round
                        );
                    }
                }
                Some(HeadStatus::Unparseable) => {
                    let _ = writeln!(out, "- r{} {side} 侧：不可解析（{f}）", r.round);
                }
                _ => {}
            }
        }
    }
    if out.is_empty() {
        out.push_str("（无待复核 findings：上一轮已收敛 0/0/0 或本轮为首轮）\n");
    }
    out
}

/// 生成审计任务书（写 `evidence/console/audit-<task>-r<N>-<role>.prompt.md`）。
/// 校验：task 非空、role ∈ {a,b}、round ≥ 1、模板八要素全覆盖。
/// 返回 `(文件路径, 未覆盖要素)`（未覆盖非空 = 模板漂移，任务书仍写出但调用方应告警）。
///
/// # Errors
///
/// - 空 task / role 非 a|b / round = 0 / 台账无该任务 = `Err`。
pub fn generate(
    repo_root: &Path,
    task: &str,
    round: u32,
    role: char,
    audit_surface: &str,
) -> Result<(String, Vec<&'static str>), String> {
    if task.is_empty() {
        return Err("task 不能为空".to_owned());
    }
    if role != 'a' && role != 'b' {
        return Err("role 只接受 a（完成度）/ b（一致性）".to_owned());
    }
    if round == 0 {
        return Err("round 必须 ≥ 1".to_owned());
    }
    let (title, dod, evidence) = find_task(repo_root, task)?;
    let prev = latest_findings(repo_root, task);
    let text = render_taskbook(task, round, role, &dod, &evidence, &prev, audit_surface);
    let uncovered = cover_elements(&text);
    let dir = repo_root.join("evidence/console");
    std::fs::create_dir_all(&dir).map_err(|e| format!("evidence/console 创建失败：{e}"))?;
    let file = dir.join(format!("audit-{task}-r{round}-{role}.prompt.md"));
    std::fs::write(&file, text).map_err(|e| format!("任务书写入失败：{e}"))?;
    let rel =
        format!("evidence/console/{}", file.file_name().unwrap_or_default().to_string_lossy());
    let _ = title;
    Ok((rel, uncovered))
}
