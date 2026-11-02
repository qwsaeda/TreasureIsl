//! 机械判定（22 号 §7.3 报告格式契约 + §7.4 收敛判定）。
//!
//! - **解析**：读文件 → 第一个围栏代码块 → 四行前缀匹配 + 值域校验；
//! - **位置口径**：四行必须在第一个围栏代码块内（标题/引用块之后、findings 表之前）；
//! - **值域口径**：`总判定` ∈ {PASS, FAIL}；否则（含占位轮「中断（无报告）」）→ 不可解析
//!   ⇒ 该轮不计收敛轮次（占位轮不得被机械判定误读成 0/0/0 有效轮）；
//! - **计数口径**：`0 新发现` = 新发现行 = 0（P2 建议同样计入）；
//! - **轮次完整性**：N 轮 × A/B 角色齐全（序号连续、每轮 a/b 在盘；占位文件计完整性不计值域轮）；
//! - **收敛**：0 新发现 + 0 误报 + 0 残留，连续 2 轮 = 通过；连续 3 轮未收敛 → 升级拆子任务；
//! - **祖父条款**：r1~r3（契约引入前）人工口径读，不得因缺四行判「不可解析」误报。

/// 四行机器可解析字段。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuditFields {
    /// `总判定:` 值（`PASS`/`FAIL`）。
    pub verdict: Option<String>,
    /// `新发现:` 值。
    pub found: Option<u64>,
    /// `误报:` 值。
    pub false_pos: Option<u64>,
    /// `残留:` 值。
    pub residual: Option<u64>,
}

impl AuditFields {
    /// 是否收敛：0 新发现 + 0 误报 + 0 残留。
    #[must_use]
    pub fn is_converged(&self) -> bool {
        self.found == Some(0) && self.false_pos == Some(0) && self.residual == Some(0)
    }
}

/// 报告头解析状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadStatus {
    /// 四行齐全 + 值域合法。
    Parsed(AuditFields),
    /// 缺任一行 或 总判定非 PASS/FAIL（含占位轮「中断（无报告）」）。
    Unparseable,
    /// 祖父条款（r1~r3 人工口径）：缺四行但豁免，不计收敛不计违规。
    Legacy,
}

/// 解析报告正文头：取第一个围栏代码块 → 四行前缀匹配 → 值域校验。
/// `legacy` = 祖父条款豁免（r1~r3 人工口径）。
#[must_use]
pub fn parse_head_status(text: &str, legacy: bool) -> HeadStatus {
    // 取第一个围栏代码块内容（标题/引用块之后）。
    let mut block = String::new();
    let mut in_block = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_block {
                break;
            }
            in_block = true;
            continue;
        }
        if in_block {
            block.push_str(line);
            block.push('\n');
        }
    }

    let mut fields = AuditFields::default();
    for line in block.lines() {
        let s = line.trim();
        if let Some(v) = s.strip_prefix("总判定:") {
            fields.verdict = Some(v.trim().to_owned());
        } else if let Some(v) = s.strip_prefix("新发现:") {
            fields.found = v.trim().parse().ok();
        } else if let Some(v) = s.strip_prefix("误报:") {
            fields.false_pos = v.trim().parse().ok();
        } else if let Some(v) = s.strip_prefix("残留:") {
            fields.residual = v.trim().parse().ok();
        }
    }
    let has_all = fields.verdict.is_some()
        && fields.found.is_some()
        && fields.false_pos.is_some()
        && fields.residual.is_some();
    let verdict_ok = matches!(fields.verdict.as_deref(), Some("PASS" | "FAIL"));
    if has_all && verdict_ok {
        HeadStatus::Parsed(fields)
    } else if legacy {
        HeadStatus::Legacy
    } else {
        HeadStatus::Unparseable
    }
}

/// 单报告元数据（供聚合）。
#[derive(Debug, Clone)]
pub struct ReportMeta {
    /// 被审任务 id。
    pub task: String,
    /// 审计轮次。
    pub round: u32,
    /// 侧（`a`/`b`）。
    pub side: char,
    /// 报告文件路径（相对仓库根）。
    pub file: String,
    /// 头解析状态。
    pub status: HeadStatus,
}

/// 轮次聚合：每轮 a/b 配对。
#[derive(Debug, Clone)]
pub struct RoundAgg {
    /// 审计轮次。
    pub round: u32,
    /// A 侧状态。
    pub a: Option<HeadStatus>,
    /// B 侧状态。
    pub b: Option<HeadStatus>,
    /// A 侧文件。
    pub a_file: Option<String>,
    /// B 侧文件。
    pub b_file: Option<String>,
}

impl RoundAgg {
    /// 构造空轮次（a/b 未配对）。
    #[must_use]
    pub fn new(round: u32) -> Self {
        Self { round, a: None, b: None, a_file: None, b_file: None }
    }

    /// 该轮是否收敛（a 与 b 均 Parsed 且均 `is_converged`）。
    #[must_use]
    pub fn converged(&self) -> bool {
        matches!(self.a, Some(HeadStatus::Parsed(ref f)) if f.is_converged())
            && matches!(self.b, Some(HeadStatus::Parsed(ref f)) if f.is_converged())
    }

    /// 该轮是否任一侧不可解析（非 legacy）→ 违规（不计收敛轮次）。
    #[must_use]
    pub fn unparseable(&self) -> bool {
        matches!(self.a, Some(HeadStatus::Unparseable))
            || matches!(self.b, Some(HeadStatus::Unparseable))
    }

    /// 该轮是否任一侧祖父条款（人工口径，不计收敛不计违规）。
    #[must_use]
    pub fn legacy(&self) -> bool {
        matches!(self.a, Some(HeadStatus::Legacy)) || matches!(self.b, Some(HeadStatus::Legacy))
    }

    /// a/b 是否都在盘（轮次完整性；占位文件也计）。
    #[must_use]
    pub fn complete(&self) -> bool {
        self.a_file.is_some() && self.b_file.is_some()
    }
}

/// 按 task 报告聚合为轮次（每轮 a/b 配对，按 round 升序）。
#[must_use]
pub fn aggregate(reports: &[ReportMeta]) -> Vec<RoundAgg> {
    let mut rounds: Vec<RoundAgg> = Vec::new();
    for r in reports {
        let is_a = r.side == 'a';
        if let Some(a) = rounds.iter_mut().find(|x| x.round == r.round) {
            if is_a {
                a.a = Some(r.status.clone());
                a.a_file = Some(r.file.clone());
            } else {
                a.b = Some(r.status.clone());
                a.b_file = Some(r.file.clone());
            }
        } else {
            let mut a = RoundAgg::new(r.round);
            if is_a {
                a.a = Some(r.status.clone());
                a.a_file = Some(r.file.clone());
            } else {
                a.b = Some(r.status.clone());
                a.b_file = Some(r.file.clone());
            }
            rounds.push(a);
        }
    }
    rounds.sort_by_key(|x| x.round);
    rounds
}

/// 轮次完整性：`min..=max` 连续、每轮 a/b 在盘。返回 `(ok, 缺失轮列表)`。
///
/// 检查**观测轮次区间**（min..=max）而非硬从 1 起：拆分任务（如 `S32.0 → S32.0a`）
/// 的拆分前轮次保留旧 id（22 号 §7.2），新 id 序列从拆分后轮次起（如 r4），
/// 硬从 1 检查会把拆分前轮次误报成缺口（S32.0a 曾误报 `missing=[1,2,3]`）。
#[must_use]
pub fn round_complete(rounds: &[RoundAgg]) -> (bool, Vec<u32>) {
    let Some(min) = rounds.iter().map(|r| r.round).min() else {
        return (true, Vec::new());
    };
    let max = rounds.iter().map(|r| r.round).max().unwrap_or(min);
    let mut missing = Vec::new();
    for r in min..=max {
        if !rounds.iter().any(|x| x.round == r && x.complete()) {
            missing.push(r);
        }
    }
    (missing.is_empty(), missing)
}

/// 收敛判定结果（22 号 §7.4）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvergeResult {
    /// 连续 ≥2 轮全收敛 = 通过。
    Pass {
        /// 连续收敛轮数。
        consecutive: u32,
    },
    /// 进行中（最近连续收敛轮 < 2，且未达升级阈值）。
    InProgress {
        /// 最近连续收敛轮数。
        consecutive: u32,
    },
    /// 连续 3 轮未收敛 → 升级拆子任务（`docs/loop.md` §4）。
    Escalate {
        /// 连续未收敛轮数。
        unclosed: u32,
    },
}

/// 收敛判定：从最新轮向前数**连续收敛轮**（`converged()`）；非收敛/legacy 打断；
/// 连续 3 轮未收敛（非 legacy）→ 升级。
///
/// 两个计数独立扫描（语义不互扰）：
/// - `consecutive`：最新轮起连续收敛轮数，遇非收敛或 legacy 停；
/// - `unclosed_streak`：最新轮起连续非收敛（非 legacy）轮数，遇收敛或 legacy 停。
#[must_use]
pub fn convergence(rounds: &[RoundAgg]) -> ConvergeResult {
    let mut consecutive = 0u32;
    for r in rounds.iter().rev() {
        if r.converged() {
            consecutive += 1;
        } else {
            break;
        }
    }
    let mut unclosed_streak = 0u32;
    for r in rounds.iter().rev() {
        if r.converged() {
            break;
        }
        if r.legacy() {
            break;
        }
        unclosed_streak += 1;
    }
    if consecutive >= 2 {
        ConvergeResult::Pass { consecutive }
    } else if unclosed_streak >= 3 {
        ConvergeResult::Escalate { unclosed: unclosed_streak }
    } else {
        ConvergeResult::InProgress { consecutive }
    }
}

/// 未闭环清单：最新轮两侧非零 findings（新发现 > 0 或 残留 > 0 或 不可解析）。
/// 返回 `(side, new, residual, file)`；`Unparseable` 侧 = `(side, u64::MAX, u64::MAX, file)`。
#[must_use]
pub fn unclosed_findings(rounds: &[RoundAgg]) -> Vec<(char, u64, u64, String)> {
    let latest = rounds.last();
    let mut out = Vec::new();
    if let Some(r) = latest {
        for (side, st, file) in [('a', &r.a, &r.a_file), ('b', &r.b, &r.b_file)] {
            let f = file.clone().unwrap_or_default();
            match st {
                Some(HeadStatus::Parsed(fields)) => {
                    let new = fields.found.unwrap_or(0);
                    let res = fields.residual.unwrap_or(0);
                    if new > 0 || res > 0 {
                        out.push((side, new, res, f));
                    }
                }
                Some(HeadStatus::Unparseable) => {
                    out.push((side, u64::MAX, u64::MAX, f));
                }
                _ => {}
            }
        }
    }
    out
}

/// 解析报告文件名：`audit-<task>-r<N>-<role>.md` → `(task, round, role)`。
/// `<task>` 可含 `-`/`.`（如 `S32.0a`）；末段 = side（a/b），倒数第 2 段以 `r` 开头 = round。
/// 非 `audit-*.md` 形态 = `None`。
#[must_use]
pub fn parse_report_filename(name: &str) -> Option<(String, u32, char)> {
    let stem = name.strip_prefix("audit-")?.strip_suffix(".md")?;
    let segs: Vec<&str> = stem.split('-').collect();
    if segs.len() < 3 {
        return None;
    }
    let side = *segs.last()?;
    if side != "a" && side != "b" {
        return None;
    }
    let round_seg = segs[segs.len() - 2];
    let round: u32 = round_seg.strip_prefix('r')?.parse().ok()?;
    let task = segs[..segs.len() - 2].join("-");
    Some((task, round, side.chars().next()?))
}
