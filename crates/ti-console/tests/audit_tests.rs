#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! S32.5 双审计编排单测（22 号 §7.3/§7.4；gate tests/ 目录内允许 unwrap/expect）。
//!
//! - 机械判定：四行解析值域（PASS/FAIL 合法、占位轮「中断（无报告）」不可解析、legacy 豁免）；
//! - 收敛规则：三条判据（0 新发现 + 0 误报 + 0 残留）、连续 2 轮 = 通过、3 轮未收敛 = 升级；
//! - 轮次完整性：缺失轮报告；
//! - 聚合：a/b 配对；
//! - 模板：八要素覆盖机器断言；
//! - 文件名解析：`S32.0a` 含 `-`/`.` 任务名；
//! - 归档防穿越负例。

use ti_console::audit::archive;
use ti_console::audit::judge::{
    AuditFields, ConvergeResult, HeadStatus, ReportMeta, aggregate, convergence, parse_head_status,
    parse_report_filename, round_complete, unclosed_findings,
};
use ti_console::audit::template::{TASKBOOK_ELEMENTS, cover_elements, render_taskbook};

/// 构造一份含四行的报告正文（合法可解析）。
fn valid_report(verdict: &str, found: u64, false_pos: u64, residual: u64) -> String {
    format!(
        "# 审计报告\n\n\
         这是正文。\n\n\
         ```\n\
         总判定: {verdict}\n\
         新发现: {found}\n\
         误报: {false_pos}\n\
         残留: {residual}\n\
         ```\n\n\
         详细 findings...\n"
    )
}

fn parsed_fields(verdict: &str, found: u64, false_pos: u64, residual: u64) -> AuditFields {
    AuditFields {
        verdict: Some(verdict.to_owned()),
        found: Some(found),
        false_pos: Some(false_pos),
        residual: Some(residual),
    }
}

fn meta(task: &str, round: u32, side: char, file: &str, st: HeadStatus) -> ReportMeta {
    ReportMeta { task: task.to_owned(), round, side, file: file.to_owned(), status: st }
}

// ---------- 四行解析值域 ----------

#[test]
fn parse_ok_all_four_lines() {
    let text = valid_report("PASS", 0, 0, 0);
    match parse_head_status(&text, false) {
        HeadStatus::Parsed(f) => {
            assert_eq!(f.verdict.as_deref(), Some("PASS"));
            assert_eq!(f.found, Some(0));
            assert_eq!(f.false_pos, Some(0));
            assert_eq!(f.residual, Some(0));
        }
        other => panic!("预期 Parsed，实际 {other:?}"),
    }
}

#[test]
fn parse_fail_verdict_legal() {
    let text = valid_report("FAIL", 3, 0, 2);
    match parse_head_status(&text, false) {
        HeadStatus::Parsed(f) => assert_eq!(f.verdict.as_deref(), Some("FAIL")),
        other => panic!("预期 Parsed，实际 {other:?}"),
    }
}

#[test]
fn parse_placeholder_round_unparseable() {
    // 占位轮「中断（无报告）」：无四行 + 总判定不是 PASS/FAIL → 不可解析（不计收敛轮次）。
    let text = "# 审计报告\n\n```\n总判定: 中断（无报告）\n```\n";
    assert_eq!(parse_head_status(text, false), HeadStatus::Unparseable);
}

#[test]
fn parse_missing_line_unparseable() {
    // 缺「残留」行 → 不可解析。
    let text = "# 审计报告\n\n```\n总判定: PASS\n新发现: 0\n误报: 0\n```\n";
    assert_eq!(parse_head_status(text, false), HeadStatus::Unparseable);
}

#[test]
fn parse_legacy_exempt() {
    // r1~r3（祖父条款）缺四行 → Legacy（人工口径，不计收敛不计违规）。
    let text = "# 审计报告（旧格式，无四行）\n";
    assert_eq!(parse_head_status(text, true), HeadStatus::Legacy);
}

#[test]
fn parse_not_legacy_still_unparseable() {
    // 非 legacy 缺四行 → 不可解析。
    let text = "# 审计报告（无四行）\n";
    assert_eq!(parse_head_status(text, false), HeadStatus::Unparseable);
}

// ---------- 收敛判定 ----------

#[test]
fn converge_two_rounds_pass() {
    // r7/r8 连续 2 轮全 0 = 通过。
    let rounds = aggregate(&[
        meta("S32.4", 7, 'a', "a7", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 7, 'b', "b7", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 8, 'a', "a8", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 8, 'b', "b8", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    assert_eq!(convergence(&rounds), ConvergeResult::Pass { consecutive: 2 });
}

#[test]
fn converge_one_round_in_progress() {
    // 仅 r8 收敛（r7 有 finding）→ InProgress。
    let rounds = aggregate(&[
        meta("S32.4", 7, 'a', "a7", HeadStatus::Parsed(parsed_fields("FAIL", 2, 0, 1))),
        meta("S32.4", 7, 'b', "b7", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 8, 'a', "a8", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 8, 'b', "b8", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    assert_eq!(convergence(&rounds), ConvergeResult::InProgress { consecutive: 1 });
}

#[test]
fn converge_three_unclosed_escalate() {
    // r4/r5/r6 连续 3 轮未收敛 → Escalate（升级拆子任务）。
    let mut rounds = Vec::new();
    for r in [4u32, 5, 6] {
        rounds.push(meta(
            "S32.4",
            r,
            'a',
            &format!("a{r}"),
            HeadStatus::Parsed(parsed_fields("FAIL", 1, 0, 0)),
        ));
        rounds.push(meta(
            "S32.4",
            r,
            'b',
            &format!("b{r}"),
            HeadStatus::Parsed(parsed_fields("FAIL", 1, 0, 0)),
        ));
    }
    let rounds = aggregate(&rounds);
    assert_eq!(convergence(&rounds), ConvergeResult::Escalate { unclosed: 3 });
}

#[test]
fn converge_placeholder_round_not_counted() {
    // r5 是占位轮（不可解析）→ 不计收敛轮次；r6/r7 连续 2 轮全 0 仍应 Pass。
    let rounds = aggregate(&[
        meta("S32.4", 5, 'a', "a5", HeadStatus::Unparseable),
        meta("S32.4", 5, 'b', "b5", HeadStatus::Unparseable),
        meta("S32.4", 6, 'a', "a6", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 6, 'b', "b6", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 7, 'a', "a7", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 7, 'b', "b7", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    assert_eq!(convergence(&rounds), ConvergeResult::Pass { consecutive: 2 });
}

#[test]
fn converge_legacy_breaks_streak() {
    // r3 legacy（人工口径）打断连续计数；r4/r5 连续 2 轮全 0 → Pass（连续从 r4 起）。
    let rounds = aggregate(&[
        meta("S32.4", 3, 'a', "a3", HeadStatus::Legacy),
        meta("S32.4", 3, 'b', "b3", HeadStatus::Legacy),
        meta("S32.4", 4, 'a', "a4", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 4, 'b', "b4", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 5, 'a', "a5", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 5, 'b', "b5", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    assert_eq!(convergence(&rounds), ConvergeResult::Pass { consecutive: 2 });
}

// ---------- 轮次完整性 ----------

#[test]
fn round_complete_missing_round_reported() {
    // r1/r2 在盘、r3 缺、r4 在盘 → 完整性不满足 + 缺失轮 [3]（r4 使 max=4，缺口可被检出）。
    let rounds = aggregate(&[
        meta("S32.4", 1, 'a', "a1", HeadStatus::Legacy),
        meta("S32.4", 1, 'b', "b1", HeadStatus::Legacy),
        meta("S32.4", 2, 'a', "a2", HeadStatus::Legacy),
        meta("S32.4", 2, 'b', "b2", HeadStatus::Legacy),
        meta("S32.4", 4, 'a', "a4", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 4, 'b', "b4", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    let (ok, missing) = round_complete(&rounds);
    assert!(!ok);
    assert_eq!(missing, vec![3]);
}

#[test]
fn round_complete_ok() {
    let rounds = aggregate(&[
        meta("S32.4", 1, 'a', "a1", HeadStatus::Legacy),
        meta("S32.4", 1, 'b', "b1", HeadStatus::Legacy),
        meta("S32.4", 2, 'a', "a2", HeadStatus::Legacy),
        meta("S32.4", 2, 'b', "b2", HeadStatus::Legacy),
    ]);
    let (ok, missing) = round_complete(&rounds);
    assert!(ok);
    assert_eq!(missing, Vec::<u32>::new());
}

#[test]
fn round_complete_split_task_no_false_missing() {
    // 拆分任务（如 S32.0a）轮次从 r4 起（r1~r3 保留旧 id）：min=4 起检查，不误报 1..=3 缺口。
    let rounds = aggregate(&[
        meta("S32.0a", 4, 'a', "a4", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.0a", 4, 'b', "b4", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.0a", 5, 'a', "a5", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.0a", 5, 'b', "b5", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    let (ok, missing) = round_complete(&rounds);
    assert!(ok, "拆分任务不应误报缺口，missing = {missing:?}");
    assert_eq!(missing, Vec::<u32>::new());
}

// ---------- 聚合 a/b 配对 ----------

#[test]
fn aggregate_pairs_a_b() {
    let rounds = aggregate(&[
        meta("S32.4", 2, 'a', "a2", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
        meta("S32.4", 2, 'b', "b2", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    assert_eq!(rounds.len(), 1);
    assert_eq!(rounds[0].round, 2);
    assert_eq!(rounds[0].a_file.as_deref(), Some("a2"));
    assert_eq!(rounds[0].b_file.as_deref(), Some("b2"));
    assert!(rounds[0].complete());
    assert!(rounds[0].converged());
}

#[test]
fn unclosed_findings_latest_only() {
    let rounds = aggregate(&[
        meta("S32.4", 1, 'a', "a1", HeadStatus::Parsed(parsed_fields("FAIL", 3, 0, 2))),
        meta("S32.4", 1, 'b', "b1", HeadStatus::Parsed(parsed_fields("PASS", 0, 0, 0))),
    ]);
    let unclosed = unclosed_findings(&rounds);
    assert_eq!(unclosed.len(), 1);
    assert_eq!(unclosed[0].0, 'a');
    assert_eq!(unclosed[0].1, 3);
    assert_eq!(unclosed[0].2, 2);
}

// ---------- 八要素模板覆盖 ----------

#[test]
fn taskbook_covers_all_eight_elements() {
    let text = render_taskbook(
        "S32.4",
        9,
        'a',
        "gate 全绿 + 双审计通过",
        "evidence/S32.4.log",
        "（无上一轮 findings）",
        "A：DoD 逐条；B：架构/规范/文档/门禁/回归/安全/依赖",
    );
    let uncovered = cover_elements(&text);
    assert!(uncovered.is_empty(), "八要素未全覆盖：{uncovered:?}（清单 = {TASKBOOK_ELEMENTS:?}）");
}

#[test]
fn taskbook_elements_has_eight() {
    assert_eq!(TASKBOOK_ELEMENTS.len(), 8);
}

#[test]
fn cover_elements_detects_missing() {
    // 缺「证据要求」节 → cover_elements 返回该要素。
    let text = "# 审计任务书\n\n## 角色声明\n## 只读约束\n## 被审任务\n## 本轮定位\n## 审计面\n## 报告格式契约\n## 输出要求\n";
    let uncovered = cover_elements(text);
    assert!(uncovered.contains(&"证据要求"), "uncovered = {uncovered:?}");
}

// ---------- 文件名解析（含 `-`/`.` 任务名） ----------

#[test]
fn parse_filename_task_with_dash_dot() {
    // S32.0a 含 `-`/`.`；末两段 = r4/a。
    let (task, round, side) = parse_report_filename("audit-S32.0a-r4-a.md").unwrap();
    assert_eq!(task, "S32.0a");
    assert_eq!(round, 4);
    assert_eq!(side, 'a');
}

#[test]
fn parse_filename_plain_task() {
    let (task, round, side) = parse_report_filename("audit-S32.4-r8-b.md").unwrap();
    assert_eq!(task, "S32.4");
    assert_eq!(round, 8);
    assert_eq!(side, 'b');
}

#[test]
fn parse_filename_invalid_side_none() {
    assert!(parse_report_filename("audit-S32.4-r8-x.md").is_none());
}

#[test]
fn parse_filename_prompt_none() {
    // 任务书（.prompt.md）非报告：parse_report_filename 对 `.prompt.md` 应返回 None（末段非 a/b）。
    assert!(parse_report_filename("audit-S32.4-r8-a.prompt.md").is_none());
}

// ---------- 归档防穿越负例 ----------

#[test]
fn archive_rejects_non_console_prefix() {
    let dir = std::env::temp_dir().join("ti_audit_tests_archive_prefix");
    let _ = std::fs::create_dir_all(&dir);
    let err = archive::archive(&dir, "evidence/x/audit-S32.4-r8-a.md", "正文").unwrap_err();
    assert!(err.contains("evidence/console/"), "{err}");
}

#[test]
fn archive_rejects_subdir() {
    let dir = std::env::temp_dir().join("ti_audit_tests_archive_subdir");
    let _ = std::fs::create_dir_all(&dir);
    let err =
        archive::archive(&dir, "evidence/console/sub/audit-S32.4-r8-a.md", "正文").unwrap_err();
    assert!(err.contains("子目录分隔符"), "{err}");
}

#[test]
fn archive_rejects_empty_content() {
    let dir = std::env::temp_dir().join("ti_audit_tests_archive_empty");
    let _ = std::fs::create_dir_all(&dir);
    let err = archive::archive(&dir, "evidence/console/audit-S32.4-r8-a.md", "").unwrap_err();
    assert!(err.contains("正文不能为空"), "{err}");
}

#[test]
fn archive_rejects_unknown_filename() {
    let dir = std::env::temp_dir().join("ti_audit_tests_archive_name");
    let _ = std::fs::create_dir_all(&dir);
    let err = archive::archive(&dir, "evidence/console/not-audit.md", "正文").unwrap_err();
    assert!(err.contains("audit-"), "{err}");
}

#[cfg(unix)]
#[test]
fn archive_rejects_symlink() {
    // 防 symlink 逃逸：目标文件已是符号链接 → 拒绝（B 侧 F3）。
    let dir = std::env::temp_dir().join("ti_audit_tests_archive_symlink");
    let console = dir.join("evidence/console");
    let _ = std::fs::create_dir_all(&console);
    // 造一个指向仓库外的 symlink（目标 = temp 下另一文件）。
    let outside = dir.join("outside_target.md");
    std::fs::write(&outside, "秘密").unwrap();
    let target = console.join("audit-S32.5-r1-a.md");
    std::os::unix::fs::symlink(&outside, &target).unwrap();
    let err = archive::archive(&dir, "evidence/console/audit-S32.5-r1-a.md", "正文").unwrap_err();
    assert!(err.contains("符号链接"), "{err}");
    // 链接目标未被写入。
    assert_eq!(std::fs::read_to_string(&outside).unwrap(), "秘密");
    let _ = std::fs::remove_dir_all(&dir);
}
