//! 结构化测试结果协议（契约 C1，S74.2）。
//!
//! `result.json = { binary, tests:[{name,passed,failed,msg}], duration_ms, exit_code }`
//! —— 平台测试页结论优先读此形状（W1/W3/W4 判据输出同源）；缺失时从**完整** cargo test
//! 日志解析同形状（根治旧实现只读尾部 400 行导致长日志前段 binary 静默漏检）。
//!
//! 本模块 **0 游戏依赖**（ti-console 红线 §6）：只定义数据形状 + 纯文本解析，不碰进程/文件。
//! 解析器 [`parse_test_blocks`] = 单一事实源，`jobs.rs::parse_test_log`（旧日志格式兼容回退）
//! 复用同一实现，避免两份解析逻辑漂移。

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// C1 `tests[]` 元素：一个测试 binary 的汇总行。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TestCaseResult {
    /// 测试 binary 名（对应 `Running tests/<name>.rs` 去后缀）。
    pub name: String,
    /// 通过断言数。
    pub passed: u32,
    /// 失败断言数。
    pub failed: u32,
    /// 说明（全过 = `"ok"`；有失败 = 失败用例名逗号串）。
    pub msg: String,
}

/// C1 结构化结果（契约冻结形状，勿增删字段）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TestResult {
    /// 测试 binary（单 binary 作业 = binary 名；多 binary 作业 = job id）。
    pub binary: String,
    /// 逐 binary 汇总行。
    pub tests: Vec<TestCaseResult>,
    /// 作业耗时（毫秒，来自作业状态 `ended_ts - started_ts`）。
    pub duration_ms: u64,
    /// 进程退出码（0 = 通过；-1 = 超时/信号终止）。
    pub exit_code: i32,
}

impl TestResult {
    /// 通过断言总数。
    #[must_use]
    pub fn total_passed(&self) -> u32 {
        self.tests.iter().map(|t| t.passed).sum()
    }

    /// 失败断言总数。
    #[must_use]
    pub fn total_failed(&self) -> u32 {
        self.tests.iter().map(|t| t.failed).sum()
    }

    /// 全部通过（0 失败）。
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.total_failed() == 0
    }

    /// 转 JSON（结构化结论 `tests` 段，契约 C1 形状）。
    #[must_use]
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| json!({}))
    }
}

/// 单个 `Running …` 块的解析中间态（无需 serde）。
#[derive(Debug, Clone, Default)]
pub struct ParsedBlock {
    /// binary 名（`Running tests/<name>.rs` 去后缀）。
    pub name: String,
    /// 通过断言数。
    pub passed: u32,
    /// 失败断言数。
    pub failed: u32,
    /// 失败用例名列表。
    pub failures: Vec<String>,
}

/// 分块 flush（内部 helper，收敛重复）。
fn flush_block(
    blocks: &mut Vec<ParsedBlock>,
    cur_name: &mut Option<String>,
    passed: &mut u32,
    failed: &mut u32,
    failures: &mut Vec<String>,
) {
    if let Some(n) = cur_name.take() {
        blocks.push(ParsedBlock {
            name: n,
            passed: *passed,
            failed: *failed,
            failures: std::mem::take(failures),
        });
        *passed = 0;
        *failed = 0;
    }
}

/// 解析完整 cargo test 日志文本 → 逐 binary 块（**完整日志**，非尾部裁剪）。
///
/// 识别 3 类行：`Running tests/<name>.rs`（开块）、`test result: … N passed; M failed`（计数）、
/// `test <case> … FAILED`（失败用例名）。smoke.ps1 简化输出（仅 Running + result 对）同样识别。
#[must_use]
pub fn parse_test_blocks(log_text: &str) -> Vec<ParsedBlock> {
    let mut blocks: Vec<ParsedBlock> = Vec::new();
    let mut cur_name: Option<String> = None;
    let mut cur_passed: u32 = 0;
    let mut cur_failed: u32 = 0;
    let mut cur_failures: Vec<String> = Vec::new();

    for line in log_text.lines() {
        let stripped = line.trim();
        if let Some(rest) = stripped.strip_prefix("Running tests/") {
            flush_block(
                &mut blocks,
                &mut cur_name,
                &mut cur_passed,
                &mut cur_failed,
                &mut cur_failures,
            );
            let fname = rest.split_whitespace().next().unwrap_or("");
            cur_name = Some(fname.strip_suffix(".rs").unwrap_or(fname).to_owned());
        } else if let Some(res) = stripped.strip_prefix("test result:") {
            if cur_name.is_some() {
                let res_str = res.trim();
                cur_passed = extract_count(res_str, " passed");
                cur_failed = extract_count(res_str, " failed");
            }
        } else if stripped.starts_with("test ") && stripped.contains("FAILED") && cur_name.is_some()
        {
            let tname = stripped
                .strip_prefix("test ")
                .and_then(|s| s.split(" ...").next())
                .unwrap_or("")
                .trim()
                .to_owned();
            cur_failures.push(tname);
        }
    }
    flush_block(&mut blocks, &mut cur_name, &mut cur_passed, &mut cur_failed, &mut cur_failures);
    blocks
}

/// 从 `test result: ok. N passed; M failed;` 提取关键词前紧邻的数字（`needle` 带前导空格）。
fn extract_count(line: &str, needle: &str) -> u32 {
    let Some(idx) = line.find(needle) else {
        return 0;
    };
    let before = &line[..idx];
    let digits: String = before.chars().rev().take_while(char::is_ascii_digit).collect();
    digits.chars().rev().collect::<String>().parse::<u32>().unwrap_or(0)
}

/// 从**完整**日志构造 C1 结构化结果。
#[must_use]
pub fn parse_cargo_test_result(
    log_text: &str,
    binary: &str,
    duration_ms: u64,
    exit_code: i32,
) -> TestResult {
    let tests = parse_test_blocks(log_text)
        .into_iter()
        .map(|b| TestCaseResult {
            name: b.name,
            passed: b.passed,
            failed: b.failed,
            msg: if b.failures.is_empty() { "ok".to_owned() } else { b.failures.join(", ") },
        })
        .collect();
    TestResult { binary: binary.to_owned(), tests, duration_ms, exit_code }
}

/// 解析外部落盘的 `result.json`（W1/W3/W4 判据产物）→ C1 结构化结果。
#[must_use]
pub fn parse_result_json(text: &str) -> Option<TestResult> {
    serde_json::from_str::<TestResult>(text).ok()
}
