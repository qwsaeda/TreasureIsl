//! S8.3.e / S8.8：`rules::messages` 分页纯函数集成测试。
//!
//! 判据（S8.8）：
//! - `total_pages` 走 `balance.messages.pages`（缺键回退 3，不 panic）；
//! - `page_text` 越界页钳到最后一页 key（与 `oracle_text_line` 的差异：oracle 固定占位文本非分页）；
//! - `page_text_key` 生成 `msg.<kind>.<n>` 字符串 key（`strings.json` 查表由调用方经 `i18n::sget` 完成）。

use ti_core::rules::messages::{MsgKind, page_text, total_pages};

#[test]
fn total_pages_help_from_balance() {
    // `balance.messages.pages.help = 3`（与 oracle `oracle_total_pages(Help)` 一致）
    assert_eq!(total_pages(MsgKind::Help), 3);
}

#[test]
fn total_pages_msgs_single_page() {
    // `balance.messages.pages.msgs = 1`
    assert_eq!(total_pages(MsgKind::Msgs), 1);
}

#[test]
fn page_text_help_page_zero() {
    let key = page_text(MsgKind::Help, 0);
    assert_eq!(key, "msg.help.1", "第 0 页 → key `msg.help.1`");
}

#[test]
fn page_text_help_last_page() {
    let key = page_text(MsgKind::Help, 2);
    assert_eq!(key, "msg.help.3", "第 2 页（末页）→ key `msg.help.3`");
}

#[test]
fn page_text_out_of_bounds_clamped_to_last() {
    let key = page_text(MsgKind::Help, 99);
    assert_eq!(key, "msg.help.3", "越界页钳到最后一页 key（不 panic）");
}

#[test]
fn page_text_single_page_kind() {
    // `msgs` 只 1 页，任何 page → `msg.msgs.1`
    let key = page_text(MsgKind::Msgs, 5);
    assert_eq!(key, "msg.msgs.1");
}
