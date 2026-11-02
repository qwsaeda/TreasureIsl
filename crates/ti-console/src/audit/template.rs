//! 审计任务书模板（22 号 §7.3 载体口径）。
//!
//! 八要素清单：任务书正文必须**逐条覆盖**（S32.5 兜底「22 号八要素无机器断言」缺口；
//! 后续迭代漏段在生成任务书时暴露 —— `cover_elements` 返回未覆盖清单供机器断言）。

/// 八要素清单（机器断言：`cover_elements` 返回未覆盖要素；空 = 全覆盖）。
pub const TASKBOOK_ELEMENTS: [&str; 8] = [
    "角色声明",
    "只读约束",
    "被审任务",
    "本轮定位",
    "审计面",
    "证据要求",
    "报告格式契约",
    "输出要求",
];

/// 渲染审计任务书正文（八要素逐条覆盖）。
///
/// `task`/`round`/`role` = 台账 id + 轮次 + 侧（`a`=完成度 / `b`=一致性）；
/// `dod` = 台账判据；`evidence` = 证据路径；`prev_findings` = 上一轮 findings（无 = 空串）；
/// `audit_surface` = 审计面说明（A = `DoD` 逐条 / B = 架构规范文档门禁回归安全依赖）。
#[must_use]
pub fn render_taskbook(
    task: &str,
    round: u32,
    role: char,
    dod: &str,
    evidence: &str,
    prev_findings: &str,
    audit_surface: &str,
) -> String {
    let side = if role == 'b' { "B：一致性与风险" } else { "A：完成度审计" };
    format!(
        "# 审计任务书 · {task} · r{round} · {role} 侧\n\n\
> 你是一个**只读**对抗审计子代理（{side}）。不得改文件、不得改台账、不得跑构建。\n\
> 被审任务 = {task}。本轮 = r{round} 收敛验证轮。\n\n\
## 角色声明\n\
你是对抗审计子代理 {role}（{side}）。\n\n\
## 只读约束\n\
不得改文件、不得改台账、不得跑构建（仅允许只读命令）。\n\n\
## 被审任务\n\
台账 id = `{task}`；dod = {dod}\n\
证据路径 = {evidence}。\n\n\
## 本轮定位\n\
本轮 = r{round}（收敛验证轮，复核上一轮 findings 是否闭环）。\n\n\
## 审计面\n\
{audit_surface}\n\n\
## 证据要求\n\
必须引用**文件路径 + 行号**或**命令实跑输出**，禁止「看起来没问题」。\n\n\
## 报告格式契约\n\
报告正文**第一个 ``` 代码块**内必须含四行（缺任一行 ⇒ 判不可解析）：\n\
```\n\
总判定: PASS|FAIL\n\
新发现: <整数>\n\
误报: 0\n\
残留: <整数>\n\
```\n\
- `新发现` 含 P2 建议（子代理报什么算什么）；`误报` = 0；`残留` = 上一轮未闭环数。\n\n\
## 输出要求\n\
把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。\n\n\
## 上一轮 findings（需复核）\n\
{prev_findings}\n"
    )
}

/// 检查正文是否覆盖八要素（返回未覆盖要素清单；空 = 全覆盖）。
#[must_use]
pub fn cover_elements(text: &str) -> Vec<&'static str> {
    TASKBOOK_ELEMENTS.iter().filter(|el| !text.contains(&format!("## {el}"))).copied().collect()
}
