//! `specskel` —— 模块 spec 骨架半自动生成（模板化逆向 → 开发流程的增量工具）。
//!
//! 用法：
//! ```sh
//! cargo run -q -p ti-tools --bin specskel -- rng                 # 生成到 stdout
//! cargo run -q -p ti-tools --bin specskel -- rng --out PATH       # 覆盖已有 spec
//! cargo run -q -p ti-tools --bin specskel -- rng --report         # 只打印统计摘要
//! ```
//!
//! 读 `ref/functions_<module>.csv`（自动识别两种列格式：常规 `addr|address,` 与
//! RNG 专用 `kind,site,func,`），按 site 在 `ref/pseudocode/` 定位函数文件，
//! 抽取带 1-based 行号的证据行（字符串常量 / 控制流 / RNG 调用 / 数值赋值 /
//! return），生成 `legacy/docs/spec/<module>.md` 骨架：函数清单表 + 逐函数证据行表
//! （`证据` 列坐标可直接核对伪代码行号），人工只补「语义」列。已存在的 spec
//! 文件需 `--out` 显式覆盖，默认只打印到 stdout 不改动任何文件。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// 每函数证据行默认上限（超出时 `truncated`，可用 `--max-lines` 放宽）。
const DEFAULT_MAX_LINES: usize = 40;

/// 单条证据文本截断长度（字符数）。
const TEXT_CAP: usize = 120;

/// 一条带行号的证据行（`file_name + 1-based 行号 + 截断后文本`）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceLine {
    file_name: String,
    line_no: usize,
    text: String,
}

/// 归一化后的函数行（两种 csv 列格式都折算成这一形态）。
#[derive(Debug, Clone)]
struct FunctionRow {
    site: String,
    name: String,
    size: String,
    module: String,
    notes: String,
}

/// 某函数的证据提取结果。
#[derive(Debug, Default)]
struct FunctionEvidence {
    file_name: Option<String>,
    lines: Vec<EvidenceLine>,
    truncated: bool,
}

/// 一次生成的骨架报告。
#[derive(Debug)]
struct Report {
    module: String,
    csv_kind: String,
    rows: Vec<FunctionRow>,
    evidences: Vec<FunctionEvidence>,
}

impl Report {
    /// 伪代码缺失的函数数。
    #[must_use]
    fn missing_count(&self) -> usize {
        self.evidences.iter().filter(|e| e.file_name.is_none()).count()
    }

    /// 证据行总数。
    #[must_use]
    fn total_evidence_lines(&self) -> usize {
        self.evidences.iter().map(|e| e.lines.len()).sum()
    }

    /// 渲染 markdown 骨架（默认 stdout 模式直接输出）。
    #[must_use]
    fn render_markdown(&self) -> String {
        let mut buf: Vec<String> = Vec::new();

        buf.push(format!("# S3.12 模块 `{}` 逆向规格（specskel 骨架）\n\n", self.module));
        buf.push(format!(
            "> 本文件由 `cargo run -q -p ti-tools --bin specskel -- {}` 生成；\n",
            self.module
        ));
        buf.push(
            "> 人工补齐「语义」列与推测点；`证据` 列为伪代码行号坐标，直接核对即可。\n\n"
                .to_string(),
        );

        buf.push(format!(
            "## 函数清单（ref/functions_{}.csv，列格式 `{}`）\n\n",
            self.module, self.csv_kind
        ));
        buf.push("| site | 函数 | size | 模块 | 备注 |\n|---|---|---|---|---|\n".to_string());
        for row in &self.rows {
            buf.push(format!(
                "| {} | `{}` | {} | {} | {} |\n",
                row.site,
                row.name,
                row.size,
                row.module,
                escape_cell(&row.notes)
            ));
        }

        buf.push("\n## 逐函数证据行（自动抽取，人工补「语义」列）\n\n".to_string());
        for (row, ev) in self.rows.iter().zip(self.evidences.iter()) {
            if let Some(file) = &ev.file_name {
                buf.push(format!(
                    "### `{}`（{}）\n\n| 证据 | 原文 | 语义（人工） |\n|---|---|---|\n",
                    row.name, row.notes
                ));
                for line in &ev.lines {
                    buf.push(format!(
                        "| {file}:{} | `{}` |  |\n",
                        line.line_no,
                        escape_cell(&line.text)
                    ));
                }
                if ev.truncated {
                    buf.push("| …（超 `--max-lines` 上限，仅到上面末行） |  |  |\n".to_string());
                }
                buf.push('\n'.to_string());
            } else {
                buf.push(format!(
                    "### `{}`\n\n⚠️ 伪代码文件缺失（site `{}`），需 IDA headless 重新导出。\n\n",
                    row.name, row.site
                ));
            }
        }

        buf.push(format!(
            "## 待人工处理\n\n- 伪代码缺失：{} 个（上表 ⚠️ 标注）\n- 「语义」列全部待填\n- 推测点：在相应证据行后追加（**推测**：…）\n",
            self.missing_count()
        ));

        buf.join("")
    }

    /// 一行统计摘要（`--report` 模式与 eprintln 用）。
    #[must_use]
    fn summary_line(&self) -> String {
        format!(
            "module={} csv={} 函数={} 缺失={} 证据行={}",
            self.module,
            self.csv_kind,
            self.rows.len(),
            self.missing_count(),
            self.total_evidence_lines()
        )
    }
}

/// 表格单元格转义：竖线与换行会破坏 markdown 表格。
#[must_use]
fn escape_cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\r', '\n'], " ").trim().to_string()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let module = args.iter().find(|a| !a.starts_with('-')).cloned().unwrap_or_default();
    let mut out_path: Option<PathBuf> = None;
    let mut max_lines = DEFAULT_MAX_LINES;
    let mut report_only = false;
    let mut it = args.iter().peekable();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--out" => out_path = it.next().cloned().map(PathBuf::from),
            "--max-lines" => {
                max_lines = it.next().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_MAX_LINES);
            }
            "--report" => report_only = true,
            _ => {}
        }
    }

    if module.is_empty() {
        eprintln!(
            "usage: specskel <module> [--out PATH] [--max-lines N] [--report]\n  module: ref/functions_<module>.csv 的 <module>（如 rng / battle / mainmap）"
        );
        return ExitCode::FAILURE;
    }

    match run(&module, out_path.as_deref(), max_lines, report_only) {
        Ok(summary) => {
            eprintln!("[specskel] {summary}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[specskel] 失败: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(
    module: &str,
    out_path: Option<&Path>,
    max_lines: usize,
    report_only: bool,
) -> Result<String, String> {
    let root = repo_root();
    let csv = root.join("ref").join(format!("functions_{module}.csv"));
    let text = fs::read_to_string(&csv).map_err(|e| {
        format!("读 {}: {e}（确认 ref/functions_{module}.csv 已登记）", csv.display())
    })?;

    let (rows, kind) = parse_module_csv(&text, module);
    if rows.is_empty() {
        return Err(format!("{} 解析出 0 行", csv.display()));
    }

    let pc_dir = root.join("ref").join("pseudocode");
    let evidences =
        rows.iter().map(|row| extract_function_evidence(&pc_dir, row, max_lines)).collect();

    let report = Report { module: module.to_string(), csv_kind: kind, rows, evidences };

    if report_only {
        return Ok(report.summary_line());
    }

    let markdown = report.render_markdown();
    if let Some(path) = out_path {
        fs::write(path, markdown).map_err(|e| format!("写 {}: {e}", path.display()))?;
        Ok(format!("{} → 写 {}", report.summary_line(), path.display()))
    } else {
        print!("{markdown}");
        Ok(report.summary_line())
    }
}

/// 解析模块函数 csv，归一化为 `FunctionRow`，并识别列格式（两种）。
///
/// - 常规（`addr,` 或 `address,` 开头）：`addr/size/name[,module,reason|callers,callers_note]`；
/// - RNG 专用（`kind,` 开头）：`kind/site/func/module/size/notes`，每行一个 RNG 调用点，
///   同 site+func 的多个 kind 行合并 notes。
fn parse_module_csv(text: &str, module: &str) -> (Vec<FunctionRow>, String) {
    let lines: Vec<&str> = text.lines().collect();
    let first = lines.first().copied().map(str::trim);
    let Some(first) = first else {
        return (Vec::new(), "unknown".to_string());
    };
    let cols: Vec<String> = first.split(',').map(|c| c.trim().to_lowercase()).collect();
    let kind: String = if cols[0] == "kind" {
        "kind,site,func".to_string()
    } else if cols[0] == "address" {
        "address,size,name".to_string()
    } else {
        "addr,size,name".to_string()
    };

    let mut rows: Vec<FunctionRow> = Vec::new();
    for raw in &lines[1..] {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let f: Vec<&str> = trimmed.split(',').map(str::trim).collect();
        if cols[0] == "kind" {
            if f.len() < 3 {
                continue;
            }
            let kind_word = f[0].to_string();
            let site_addr = f[1].to_string();
            let func_name = f[2].to_string();
            let row_module = f.get(3).unwrap_or(&"").to_string();
            let size_val = f.get(4).unwrap_or(&"").to_string();
            let notes_val = f.get(5).unwrap_or(&"").to_string();
            let full_notes = if notes_val.is_empty() {
                kind_word.clone()
            } else {
                format!("{kind_word}: {notes_val}")
            };
            if let Some(existing) =
                rows.iter_mut().find(|r| r.site == site_addr && r.name == func_name)
            {
                if !existing.notes.contains(&full_notes) {
                    existing.notes = format!("{} | {full_notes}", existing.notes);
                }
            } else {
                rows.push(FunctionRow {
                    site: site_addr.clone(),
                    name: func_name.clone(),
                    size: size_val.clone(),
                    module: if row_module.is_empty() { module.to_string() } else { row_module },
                    notes: full_notes,
                });
            }
            continue;
        }

        if f.len() < 3 {
            continue;
        }
        let addr = f[0].to_string();
        let size = f.get(1).unwrap_or(&"").to_string();
        let name = f.get(2).unwrap_or(&"").to_string();
        let module_field = f.get(3).unwrap_or(&"").to_string();
        let notes_field = f.get(4).unwrap_or(&"").to_string();
        let has_module = cols.len() > 3 && cols[3] == "module";
        let notes = if has_module {
            notes_field
        } else {
            // callers,callers_note 风格：把第 4、5 列都拼进 notes
            f.get(3).unwrap_or(&"").to_string() + " | " + f.get(4).unwrap_or(&"")
        };
        rows.push(FunctionRow {
            site: addr,
            name,
            size,
            module: if module_field.is_empty() { module.to_string() } else { module_field },
            notes,
        });
    }
    (rows, kind)
}

/// 从一个 site 定位伪代码文件，抽取证据行（带 1-based 行号）。
///
/// 文件名前缀（如 `00013CE4_sub_13CE4.c`）的字母部分是大写，而 csv 里的 site
/// 可能是小写（`functions_rng.csv` 用 `00013ce4`）。匹配时把两边都小写比较，
/// 避免大小写不一致导致漏定位。
fn extract_function_evidence(
    pc_dir: &Path,
    row: &FunctionRow,
    max_lines: usize,
) -> FunctionEvidence {
    let site_lower = row.site.to_lowercase();
    let prefix = format!("{site_lower}_");
    let found_name = match fs::read_dir(pc_dir) {
        Ok(entries) => entries
            .flatten()
            .map(|d| d.file_name())
            .find(|f| {
                let s = f.to_string_lossy().to_lowercase();
                s.starts_with(&prefix)
            })
            .filter(|f| {
                let s = f.to_string_lossy();
                !s.is_empty() && s.to_lowercase().ends_with(".c")
            }),
        Err(_) => None,
    };
    let Some(found_name) = found_name else {
        return FunctionEvidence::default();
    };
    let found_name = found_name.to_string_lossy().into_owned();
    let Ok(text) = fs::read_to_string(pc_dir.join(&found_name)) else {
        return FunctionEvidence {
            file_name: Some(found_name.clone()),
            lines: Vec::new(),
            truncated: false,
        };
    };

    let mut lines: Vec<EvidenceLine> = Vec::new();
    let mut truncated = false;
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("/*") || trimmed.starts_with("//") {
            continue;
        }
        if is_evidence_line(trimmed) {
            let cap = trimmed.char_indices().map(|(i, _)| i).nth(TEXT_CAP).unwrap_or(trimmed.len());
            let snippet = &trimmed[..cap];
            let cut = cap < trimmed.len();
            lines.push(EvidenceLine {
                file_name: found_name.clone(),
                line_no: idx + 1,
                text: if cut { format!("{snippet}…") } else { snippet.to_string() },
            });
        }
        if lines.len() >= max_lines {
            lines.truncate(max_lines);
            truncated = true;
            break;
        }
    }
    FunctionEvidence { file_name: Some(found_name), lines, truncated }
}

/// 证据行判定：含常量 / 控制流 / RNG 调用 / 关键赋值特征的非注释语句行。
#[must_use]
fn is_evidence_line(line: &str) -> bool {
    let s = line.to_lowercase();
    if s.contains('"') {
        return true;
    }
    if s.starts_with("if ")
        || s.starts_with("switch")
        || s.starts_with("while")
        || s.starts_with("for ")
    {
        return true;
    }
    if s.contains("rand(") || s.contains("srand(") {
        return true;
    }
    if s.starts_with("return") {
        return true;
    }
    if let Some(eq_pos) = s.find('=') {
        let rhs = s[eq_pos + 1..].trim_start();
        if rhs.starts_with('0')
            || rhs.starts_with('{')
            || rhs.starts_with('*')
            || rhs.starts_with('&')
            || rhs.starts_with('!')
        {
            return true;
        }
    }
    false
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

// —— 纯函数单测（内联夹具；风格对齐 classify.rs）——
#[cfg(test)]
mod tests {
    use super::{EvidenceLine, FunctionEvidence, escape_cell, is_evidence_line, parse_module_csv};

    const ADDR_CSV: &str = "addr,size,name,module,reason\n00032960,120,sub_32960,cards,rand() x1\n0003D4C8,1040,sub_3D4C8,cards,srand ms seed\n";

    const ADDRESS_CSV: &str = "address,size,name,callers,callers_note\n000572E4,104,sub_572E4,00029110:sub_29110,vs ctor\n";

    const RNG_CSV: &str = "kind,site,func,module,size,notes\nrand,00032960,sub_32960,cards,120,rand() x1\nsrand,00032960,sub_32960,cards,120,seed=ms\n";

    #[test]
    fn parses_addr_style_csv() {
        let (rows, kind) = parse_module_csv(ADDR_CSV, "cards");
        assert_eq!(kind, "addr,size,name");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].site, "00032960");
        assert_eq!(rows[0].name, "sub_32960");
        assert_eq!(rows[0].module, "cards");
        assert_eq!(rows[0].notes, "rand() x1");
        assert_eq!(rows[1].notes, "srand ms seed");
    }

    #[test]
    fn parses_address_style_csv() {
        let (rows, kind) = parse_module_csv(ADDRESS_CSV, "battle");
        assert_eq!(kind, "address,size,name");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].site, "000572E4");
        assert_eq!(rows[0].size, "104");
        assert_eq!(rows[0].name, "sub_572E4");
        // 第 4、5 列（callers、callers_note）都进 notes 尾部，用 " | " 连接
        let notes = &rows[0].notes;
        assert!(notes.contains("00029110:sub_29110"), "notes={notes}");
        assert!(notes.contains("vs ctor"), "notes={notes}");
    }

    #[test]
    fn merges_same_site_kind_rows() {
        let (rows, kind) = parse_module_csv(RNG_CSV, "cards");
        assert_eq!(kind, "kind,site,func");
        assert_eq!(rows.len(), 1, "同 site+func 的 rand/srand 应合并为一条");
        assert!(rows[0].notes.contains("rand() x1"));
        assert!(rows[0].notes.contains("seed=ms"));
    }

    #[test]
    fn evidence_line_detects_strings_and_conditions() {
        assert!(is_evidence_line("v2 = \"hello\";"));
        assert!(is_evidence_line("if (v1 == 1) {"));
        assert!(is_evidence_line("n = 0x1234;"));
        assert!(is_evidence_line("return a1;"));
        assert!(is_evidence_line("v = rand();"));
        assert!(!is_evidence_line("v5 = v6;"));
    }

    #[test]
    fn escape_cell_breaks_no_pipes() {
        let v = escape_cell("a | b\nc");
        assert!(!v.contains('\n'));
        assert_eq!(v, "a \\| b c");
    }

    #[test]
    fn evidence_shape_roundtrips() {
        let missing = FunctionEvidence::default();
        assert!(missing.file_name.is_none());
        assert_eq!(missing.lines, [] as [EvidenceLine; 0]);
        assert!(!missing.truncated);

        let truncated = FunctionEvidence {
            file_name: Some("00032960_sub_32960.c".to_string()),
            lines: vec![EvidenceLine {
                file_name: "00032960_sub_32960.c".to_string(),
                line_no: 10,
                text: "v = 1;".to_string(),
            }],
            truncated: true,
        };
        assert!(truncated.truncated);
        assert_eq!(truncated.lines[0].line_no, 10);

        let complete = FunctionEvidence {
            file_name: Some("000572E4_sub_572E4.c".to_string()),
            lines: vec![
                EvidenceLine {
                    file_name: "000572E4_sub_572E4.c".to_string(),
                    line_no: 6,
                    text: "sub_43B8C(a1, a2);".to_string(),
                },
                EvidenceLine {
                    file_name: "000572E4_sub_572E4.c".to_string(),
                    line_no: 24,
                    text: "return a1;".to_string(),
                },
            ],
            truncated: false,
        };
        assert_eq!(complete.lines.len(), 2);
        assert!(!complete.truncated);
    }
}
