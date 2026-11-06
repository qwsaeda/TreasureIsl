//! S44.5 固化收口：29 号 §2/§3 流程 ↔ 平台向导 **1:1 映射断言**。
//!
//! 两侧单一事实源分工：
//! - **平台侧** = `map_wizard::MAP_STEPS` / `ui_wizard::UI_STEPS`（步骤 id + 白名单动作 + 写面 + 机器判据）；
//! - **文档侧** = `docs/process/29_content_ops.md` §2/§3 步骤表的「平台向导步骤 / 判据 / 写面」三列。
//!
//! 本模块把两侧对账并机器断言（`assert_mapping`）：任一侧漂移 ⇒ 非空问题列表 ⇒ 门禁/单测 FAIL。
//! 断言口径三件（DoD「每步 id/判据/写面登记一致」）：
//! 1. **id** = 文档步骤 id 与平台步骤 id 集合逐一相等，且每个平台步 id 在文档中**恰好出现一次**
//!    （§2 严格序号 1:1；§3 锚点映射：§3.2 拆 `variant`+`layout`，§3.5+§3.6 合 `zero_rust`）；
//! 2. **判据** = 文档「判据」列必须含该步的判据关键词；关键词须同时出现在平台
//!    `parse_criterion` 的判据描述中（由单测断言 ⇒ 关键词不漂移）；
//! 3. **写面** = 文档「写面」列必须列出该步的**有效写点**（白名单动作取 `console_commands.json`
//!    的 `writes`；平台自身步取向导登记写点）；有效写点为空 ⇒ 文档列须标「只读」。
//!
//! 端点：`GET /api/content/mapping`（只读免 token）= 映射报告 + 断言结果。

use std::path::Path;

use serde_json::{Value, json};

use super::map_wizard;
use super::ui_wizard;

/// 29 号流程文档路径（流程 ↔ 平台 1:1 的文档侧单一事实源）。
pub const DOC_PATH: &str = "docs/process/29_content_ops.md";

/// 步骤表「平台向导步骤」列表头（解析锚点）。
const COL_STEP: &str = "平台向导步骤";
/// 步骤表「判据」列表头。
const COL_CRITERION: &str = "判据";
/// 步骤表「写面」列表头。
const COL_WRITE: &str = "写面";

/// §2 章节标题片段（改地图流程）。
const SEC_MAP: &str = "改地图流程";
/// §3 章节标题片段（换 UI 流程）。
const SEC_UI: &str = "换 UI 流程";

/// 29 号 §3 六步 ↔ UI 向导 6 步的**锚点映射**（文档行号 1 基）。
/// - §3.2「改数据」在平台拆为 `variant`（换资产）+ `layout`（改版面原语）；
/// - §3.5「零 Rust」+ §3.6「视觉签收留痕」在平台合为 `zero_rust` 一步（判据含签收关键词）。
const UI_DOC_ANCHORS: [(&str, &[usize]); 6] = [
    ("design", &[1]),
    ("variant", &[2]),
    ("layout", &[2]),
    ("build", &[3]),
    ("shot", &[4]),
    ("zero_rust", &[5, 6]),
];

/// 地图向导每步的判据关键词（须出现在平台 `parse_criterion` 描述 + 文档「判据」列）。
const MAP_CRITERION_KEYS: [(&str, &str); 10] = [
    ("add", "0=成功"),
    ("import", "0 违规"),
    ("syncmap", "0=成功"),
    ("verify", "0 违规"),
    ("pack", "pack 成功"),
    ("install", "install 接线成功"),
    ("check", "0 漂移"),
    ("sync", "0=成功"),
    ("shot", "出图成功"),
    ("zero_rust", "git diff crates/"),
];

/// UI 向导每步的判据关键词（须出现在平台 `parse_criterion` 描述 + 文档「判据」列）。
const UI_CRITERION_KEYS: [(&str, &str); 6] = [
    ("design", "出图成功"),
    ("variant", "ui_manifest 登记一致"),
    ("layout", "scenes.json indent=1 规范形"),
    ("build", "build 成功"),
    ("shot", "出图成功"),
    ("zero_rust", "git diff crates/"),
];

/// 文档步骤表的一行（解析结果）。
#[derive(Debug, Clone)]
struct DocRow {
    /// 文档行号（1 基）。
    row_no: usize,
    /// 该行的平台步骤 id 列表（可多值：§3.2 = variant+layout）。
    step_ids: Vec<String>,
    /// 「判据」列原文。
    criterion: String,
    /// 「写面」列原文。
    write: String,
}

/// 平台一步的映射信息（映射报告用）。
#[derive(Debug, Clone)]
pub struct StepMapping {
    /// 平台步骤 id。
    pub step: String,
    /// 白名单动作 id（`None` = 平台自身面）。
    pub action: Option<String>,
    /// 判据关键词。
    pub criterion_key: String,
    /// 平台机器判据描述（`parse_criterion` 产出）。
    pub criterion: String,
    /// 有效写点集。
    pub write_paths: Vec<String>,
    /// 文档行号（1 基；§3 锚点可多值）。
    pub doc_rows: Vec<usize>,
}

/// 文档中某个章节的步骤表（解析结果）。
#[derive(Debug, Clone)]
struct DocTable {
    rows: Vec<DocRow>,
}

/// 从文档中解析指定章节的步骤表。
fn parse_section_table(doc: &str, section_marker: &str) -> Result<DocTable, String> {
    let lines: Vec<&str> = doc.lines().collect();
    // 定位章节起始行（`## ` 开头且含章节标记）。
    let start = lines
        .iter()
        .position(|l| l.starts_with("## ") && l.contains(section_marker))
        .ok_or_else(|| format!("文档未找到章节「{section_marker}」"))?;
    // 章节结束 = 下一个 `## ` 标题。
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, l)| l.starts_with("## "))
        .map_or(lines.len(), |(i, _)| i);
    let section = &lines[start..end];
    // 找表头含「平台向导步骤」的表格（连续 `|` 行 = 一张表）。
    let mut i = 0usize;
    while i < section.len() {
        if !section[i].trim_start().starts_with('|') {
            i += 1;
            continue;
        }
        // 收集连续表行。
        let tbl_start = i;
        let mut j = i;
        while j < section.len() && section[j].trim_start().starts_with('|') {
            j += 1;
        }
        let header = split_cells(section[tbl_start]);
        if let Some(col_step) = header.iter().position(|c| c.contains(COL_STEP)) {
            let col_crit = header.iter().position(|c| c.contains(COL_CRITERION));
            let col_write = header.iter().position(|c| c.contains(COL_WRITE));
            let (Some(col_crit), Some(col_write)) = (col_crit, col_write) else {
                return Err(format!(
                    "章节「{section_marker}」步骤表缺「{COL_CRITERION}」/「{COL_WRITE}」列"
                ));
            };
            let mut rows = Vec::new();
            let mut row_no = 0usize;
            for line in &section[tbl_start + 1..j] {
                let cells = split_cells(line);
                if is_separator(&cells) {
                    continue;
                }
                if cells.len() != header.len() {
                    return Err(format!(
                        "章节「{section_marker}」第 {row_no} 行列数 {} ≠ 表头列数 {}",
                        cells.len(),
                        header.len()
                    ));
                }
                row_no += 1;
                let raw_step = cells[col_step].trim();
                let step_ids = extract_step_ids(raw_step);
                rows.push(DocRow {
                    row_no,
                    step_ids,
                    criterion: cells[col_crit].trim().to_owned(),
                    write: cells[col_write].trim().to_owned(),
                });
            }
            return Ok(DocTable { rows });
        }
        i = j;
    }
    Err(format!("章节「{section_marker}」未找到含「{COL_STEP}」列的步骤表"))
}

/// 从「平台向导步骤」列提取步骤 id 列表。
///
/// 步骤 id 在文档中**恒以反引号包裹** ⇒ 只取反引号跨度（散文说明不误判为步骤 id）；
/// 无反引号时退化为整格按分隔符切分（使畸形文档报「未知平台步」而非静默通过）。
fn extract_step_ids(cell: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut in_tick = false;
    let mut cur = String::new();
    for ch in cell.chars() {
        if ch == '`' {
            if in_tick {
                let id = cur.trim().to_owned();
                if !id.is_empty() {
                    ids.push(id);
                }
                cur.clear();
            }
            in_tick = !in_tick;
            continue;
        }
        if in_tick {
            cur.push(ch);
        }
    }
    if ids.is_empty() {
        ids = cell
            .split(['+', ',', '、', ' '])
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();
    }
    ids
}

/// 拆分 markdown 表格行的单元格（去首尾竖线）。
#[must_use]
fn split_cells(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    t.split('|').map(|c| c.trim().to_owned()).collect()
}

/// 分隔行判定（`|---|---|`）。
fn is_separator(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

/// 步骤的**有效写点集**：白名单动作步取 `console_commands.json` 的 `writes`；
/// 平台自身步取向导登记写点（`step_write_paths`）。
fn effective_write_paths(
    repo_root: &Path,
    action: Option<&str>,
    platform_paths: &[&str],
) -> Result<Vec<String>, String> {
    match action {
        Some(a) => {
            let cmds = crate::runner::close::read_commands(repo_root)?;
            let cmd = cmds
                .iter()
                .find(|c| c.get("id").and_then(Value::as_str) == Some(a))
                .ok_or_else(|| format!("白名单未登记动作 `{a}`"))?;
            let writes = cmd
                .get("writes")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter().filter_map(Value::as_str).map(str::to_owned).collect::<Vec<String>>()
                })
                .unwrap_or_default();
            // 白名单登记必须与平台写点模板逐项一致（写面登记一致性）。
            let template: Vec<String> =
                map_wizard::write_templates(a).iter().map(|s| (*s).to_owned()).collect();
            if template != writes {
                return Err(format!(
                    "动作 `{a}` 写面登记不一致：console_commands.json writes={writes:?} ≠ 平台模板={template:?}"
                ));
            }
            Ok(writes)
        }
        None => Ok(platform_paths.iter().map(|s| (*s).to_owned()).collect()),
    }
}

/// 地图向导 10 步的映射表（步骤 id / 动作 / 判据关键词 / 写点 / 文档行）。
fn map_mappings(repo_root: &Path) -> Result<Vec<StepMapping>, String> {
    let mut out = Vec::new();
    for (idx, step) in map_wizard::MAP_STEPS.iter().enumerate() {
        let action = map_wizard::STEP_ACTION[idx];
        let key = MAP_CRITERION_KEYS
            .iter()
            .find(|(s, _)| s == step)
            .map(|(_, k)| (*k).to_owned())
            .ok_or_else(|| format!("地图向导步 `{step}` 缺判据关键词登记"))?;
        let platform_paths = map_wizard::platform_write_paths(step);
        let write_paths = effective_write_paths(repo_root, action, platform_paths)?;
        let criterion = map_wizard::parse_criterion(step, 0, "").1;
        // 判据关键词必须出现在平台判据描述中（关键词 ↔ 判据不漂移；否则文档断言失去意义）。
        if !criterion.contains(&key) {
            return Err(format!(
                "地图向导步 `{step}` 的判据关键词「{key}」未出现在平台判据描述「{criterion}」中"
            ));
        }
        out.push(StepMapping {
            step: (*step).to_owned(),
            action: action.map(str::to_owned),
            criterion_key: key,
            criterion,
            write_paths,
            doc_rows: vec![idx + 1],
        });
    }
    Ok(out)
}

/// UI 向导 6 步的映射表。
fn ui_mappings(repo_root: &Path) -> Result<Vec<StepMapping>, String> {
    let mut out = Vec::new();
    for (idx, step) in ui_wizard::UI_STEPS.iter().enumerate() {
        let action = ui_wizard::STEP_ACTION[idx];
        let key = UI_CRITERION_KEYS
            .iter()
            .find(|(s, _)| s == step)
            .map(|(_, k)| (*k).to_owned())
            .ok_or_else(|| format!("UI 向导步 `{step}` 缺判据关键词登记"))?;
        let platform_paths = ui_wizard::step_write_paths(step);
        let write_paths = effective_write_paths(repo_root, action, platform_paths)?;
        let criterion = ui_wizard::parse_criterion(step, 0, "").1;
        // 同地图向导：判据关键词 ↔ 平台判据描述不漂移。
        if !criterion.contains(&key) {
            return Err(format!(
                "UI 向导步 `{step}` 的判据关键词「{key}」未出现在平台判据描述「{criterion}」中"
            ));
        }
        let doc_rows = UI_DOC_ANCHORS
            .iter()
            .find(|(s, _)| s == step)
            .map(|(_, rows)| rows.to_vec())
            .unwrap_or_default();
        out.push(StepMapping {
            step: (*step).to_owned(),
            action: action.map(str::to_owned),
            criterion_key: key,
            criterion,
            write_paths,
            doc_rows,
        });
    }
    Ok(out)
}

/// 文档行 → 该行应含的平台步骤 id 集合（由映射表反查）。
fn expected_rows(mappings: &[StepMapping], row_no: usize) -> Vec<String> {
    mappings.iter().filter(|m| m.doc_rows.contains(&row_no)).map(|m| m.step.clone()).collect()
}

/// 断言文档步骤表 ↔ 平台向导 1:1 映射（id / 判据 / 写面三件）。
/// 返回问题列表（**空 = 全过**）。
#[must_use]
pub fn assert_mapping(doc: &str, repo_root: &Path) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    let map_maps = match map_mappings(repo_root) {
        Ok(v) => v,
        Err(e) => return vec![format!("地图向导映射构建失败：{e}")],
    };
    let ui_maps = match ui_mappings(repo_root) {
        Ok(v) => v,
        Err(e) => return vec![format!("UI 向导映射构建失败：{e}")],
    };

    // ---- §2 地图十步：严格序号 1:1 ----
    match parse_section_table(doc, SEC_MAP) {
        Ok(t) => {
            if t.rows.len() != map_maps.len() {
                problems.push(format!(
                    "§2 步骤表行数 {} ≠ 地图向导步数 {}",
                    t.rows.len(),
                    map_maps.len()
                ));
            }
            for m in &map_maps {
                let row = t.rows.iter().find(|r| r.row_no == m.doc_rows[0]);
                let Some(row) = row else {
                    problems.push(format!("§2 缺第 {} 行（平台步 `{}`）", m.doc_rows[0], m.step));
                    continue;
                };
                // id 一致
                if row.step_ids != vec![m.step.clone()] {
                    problems.push(format!(
                        "§2 第 {} 行：文档步骤 {:?} ≠ 平台步 `{}`",
                        row.row_no, row.step_ids, m.step
                    ));
                }
                // 判据一致（文档列须含判据关键词）
                if !row.criterion.contains(&m.criterion_key) {
                    problems.push(format!(
                        "§2 第 {} 行（`{}`）：文档判据缺关键词「{}」",
                        row.row_no, m.step, m.criterion_key
                    ));
                }
                // 写面一致（文档列须列出有效写点；空 ⇒ 标「只读」）
                check_write_cell(&mut problems, "§2", row, &m.write_paths);
            }
            // 覆盖：平台步 id 恰好出现一次
            check_coverage(&mut problems, "§2", &t.rows, &map_maps, true);
        }
        Err(e) => problems.push(e),
    }

    // ---- §3 UI 六步：锚点映射 ----
    match parse_section_table(doc, SEC_UI) {
        Ok(t) => {
            if t.rows.len() != 6 {
                problems.push(format!("§3 步骤表行数 {} ≠ 29 号 §3 六步", t.rows.len()));
            }
            for m in &ui_maps {
                for row_no in &m.doc_rows {
                    let row = t.rows.iter().find(|r| r.row_no == *row_no);
                    let Some(row) = row else {
                        problems.push(format!("§3 缺第 {row_no} 行（平台步 `{}`）", m.step));
                        continue;
                    };
                    if !row.step_ids.contains(&m.step) {
                        problems.push(format!(
                            "§3 第 {row_no} 行：文档步骤 {:?} 未含平台步 `{}`",
                            row.step_ids, m.step
                        ));
                    }
                    if !row.criterion.contains(&m.criterion_key) {
                        problems.push(format!(
                            "§3 第 {row_no} 行（`{}`）：文档判据缺关键词「{}」",
                            m.step, m.criterion_key
                        ));
                    }
                    // 写面：多步共行时，逐步写点都必须被文档列覆盖。
                    check_write_cell(&mut problems, "§3", row, &m.write_paths);
                }
            }
            // 行 ↔ 步 双向一致（文档行的步骤集合 = 平台锚点反查集合）
            for row in &t.rows {
                let expect = expected_rows(&ui_maps, row.row_no);
                if !expect.is_empty() && row.step_ids != expect {
                    problems.push(format!(
                        "§3 第 {} 行：文档步骤 {:?} ≠ 平台锚点 {:?}",
                        row.row_no, row.step_ids, expect
                    ));
                }
            }
            check_coverage(&mut problems, "§3", &t.rows, &ui_maps, false);
        }
        Err(e) => problems.push(e),
    }
    problems
}

/// 写面一致性检查：文档「写面」列须列出每个有效写点；有效写点为空 ⇒ 须标「只读」。
fn check_write_cell(problems: &mut Vec<String>, sec: &str, row: &DocRow, write_paths: &[String]) {
    if write_paths.is_empty() {
        if !row.write.contains("只读") {
            problems.push(format!(
                "{sec} 第 {} 行：该步无写点，文档写面列须标「只读」（现为「{}」）",
                row.row_no, row.write
            ));
        }
        return;
    }
    for p in write_paths {
        if !row.write.contains(p.as_str()) {
            problems.push(format!(
                "{sec} 第 {} 行：文档写面列缺写点「{p}」（现为「{}」）",
                row.row_no, row.write
            ));
        }
    }
}

/// 覆盖检查：每个平台步骤 id 在文档中至少出现一次；`strict` = §2 严格 1:1（恰好一次）。
/// §3 为锚点映射（§3.5+§3.6 共用 `zero_rust` ⇒ 出现 2 次合法），故非 strict。
fn check_coverage(
    problems: &mut Vec<String>,
    sec: &str,
    rows: &[DocRow],
    maps: &[StepMapping],
    strict: bool,
) {
    let mut counts: Vec<(String, usize)> = maps.iter().map(|m| (m.step.clone(), 0)).collect();
    for row in rows {
        for id in &row.step_ids {
            if let Some(c) = counts.iter_mut().find(|(s, _)| s == id) {
                c.1 += 1;
            } else {
                problems.push(format!("{sec} 第 {} 行：未知平台步 `{id}`", row.row_no));
            }
        }
    }
    for (step, n) in counts {
        let bad = if strict { n != 1 } else { n == 0 };
        if bad {
            problems.push(format!(
                "{sec}：平台步 `{step}` 在文档中出现 {n} 次（应{}）",
                if strict { "恰好 1 次" } else { "至少 1 次" }
            ));
        }
    }
}

/// 映射报告（`GET /api/content/mapping` 数据源）：两侧对照 + 断言结果。
#[must_use]
pub fn mapping_report(repo_root: &Path) -> Value {
    let doc = std::fs::read_to_string(repo_root.join(DOC_PATH)).unwrap_or_default();
    let problems = assert_mapping(&doc, repo_root);
    let rows_of = |sec: &str| parse_section_table(&doc, sec).ok().map(|t| t.rows);
    let map_rows = rows_of(SEC_MAP).unwrap_or_default();
    let ui_rows = rows_of(SEC_UI).unwrap_or_default();
    let dump = |maps: Vec<StepMapping>, rows: Vec<DocRow>| -> Vec<Value> {
        maps.into_iter()
            .map(|m| {
                let doc_row = rows.iter().find(|r| m.doc_rows.contains(&r.row_no));
                json!({
                    "step": m.step,
                    "action": m.action,
                    "criterion_key": m.criterion_key,
                    "criterion": m.criterion,
                    "write_paths": m.write_paths,
                    "doc_rows": m.doc_rows,
                    "doc_step_ids": doc_row.map(|r| r.step_ids.clone()).unwrap_or_default(),
                    "doc_criterion": doc_row.map(|r| r.criterion.clone()).unwrap_or_default(),
                    "doc_write": doc_row.map(|r| r.write.clone()).unwrap_or_default(),
                })
            })
            .collect()
    };
    let map_maps = map_mappings(repo_root).unwrap_or_default();
    let ui_maps = ui_mappings(repo_root).unwrap_or_default();
    json!({
        "doc": DOC_PATH,
        "ok": problems.is_empty(),
        "problems": problems,
        "map": dump(map_maps, map_rows),
        "ui": dump(ui_maps, ui_rows),
        "note": "29 号 §2/§3 流程 ↔ 平台向导 1:1 映射断言（S44.5）：id 恰好一次 + 判据关键词 + 写面登记三者一致；空 problems = 全过",
    })
}
