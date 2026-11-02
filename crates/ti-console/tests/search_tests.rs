#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S32.8 文档检索 + URL 解码回归测试（第 2 轮双审计 N1：src 内联 mod tests 被门禁拦截，
//! 回归防护落 `tests/`；gate `tests/` 目录内允许 unwrap/expect）。

use ti_console::repo::percent_decode;
use ti_console::repo::search::search;

/// 构造隔离临时仓库根（`docs/` 一个受控文档），测后清理。
fn temp_repo(content: &str) -> std::path::PathBuf {
    let tmp = std::env::temp_dir().join("ti_console_search_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("docs")).unwrap();
    std::fs::write(tmp.join("docs/probe.md"), content).unwrap();
    tmp
}

#[test]
fn percent_decode_handles_encoded_and_literal() {
    // 空格 / 中文（encodeURIComponent 输出）→ 正确还原（S32.8 URL 解码回归）。
    assert_eq!(percent_decode("main_map%20v4"), "main_map v4");
    assert_eq!(percent_decode("%E8%A7%86%E8%A7%89%E7%AD%BE%E6%94%B6"), "视觉签收");
    // 非法 %XX / 孤立 % 原样保留（不崩）。
    assert_eq!(percent_decode("%GG"), "%GG");
    assert_eq!(percent_decode("abc%"), "abc%");
    assert_eq!(percent_decode("%2"), "%2");
    // `+` 保持字面（JS encodeURIComponent 语义：+ → %2B，非空格）。
    assert_eq!(percent_decode("a+b"), "a+b");
    // NUL 字节还原（Rust 字符串安全，仅作检索字面）。
    assert_eq!(percent_decode("%00"), "\0");
    // 纯 ASCII 原样。
    assert_eq!(percent_decode("EXP-116"), "EXP-116");
}

#[test]
fn search_finds_chinese_and_token_boundary() {
    let repo = temp_repo("视觉签收 EXP-116 修\nEXP-1160 单独一行\n");
    // 中文关键词命中（解码后查询的检索面回归）。
    let v = search(&repo, "视觉签收");
    assert_eq!(v["hits"].as_array().unwrap().len(), 1);
    // 反查 token 边界：EXP-116 只命中「EXP-116」行，不误配「EXP-1160」行（第 2 轮审计 F2）。
    let v2 = search(&repo, "EXP-116");
    assert_eq!(v2["hits"].as_array().unwrap().len(), 1);
    // EXP-1160 命中其专属行（与 EXP-116 区分）。
    let v3 = search(&repo, "EXP-1160");
    assert_eq!(v3["hits"].as_array().unwrap().len(), 1);
    // 空关键词 = 0 命中不扫盘。
    let v4 = search(&repo, "");
    assert_eq!(v4["hits"].as_array().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&repo);
}
