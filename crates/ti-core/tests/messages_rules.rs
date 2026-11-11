//! 消息分页集成测试（S52.1；`docs/business/loop_core.md` 消息界面）。
//!
//! S52.1 补测：`messages.rs` `total_pages_for` 此前 0 集成测试（飞轮 R2 找问题 P1-23）。
//! oracle：页数走 `balance.messages.pages`（缺省回退 3 不 panic）。
//! 判据：`cargo test -p ti-core --test messages_rules` 全绿。

use ti_core::rules::messages::total_pages_for;

/// `total_pages_for`（页数数据化）：`balance.messages.pages` 里有的 kind = 表值，
/// 没有的 kind = 缺省回退 3（不 panic）。
#[test]
fn total_pages_for_reads_balance_table_with_fallback() {
    let bal = ti_core::data::balance().expect("balance.json");
    // 表里登记的 kind = 读表值。
    let help = total_pages_for("help");
    let table_help = bal.messages.pages.get("help").copied().unwrap_or(3);
    assert_eq!(help, table_help, "help 页数 = balance 表值（缺省 3）");
    // 表里没有的 kind = 缺省 3。
    assert_eq!(total_pages_for("no_such_kind"), 3, "未知 kind = 缺省 3");
}

/// 各登记 kind 页数 ≥ 1（分页循环不越界；`page_text` 钳位依赖此不变量）。
#[test]
fn all_registered_kinds_have_at_least_one_page() {
    let bal = ti_core::data::balance().expect("balance.json");
    for (kind, pages) in &bal.messages.pages {
        assert!(*pages >= 1, "{kind} 页数 = {pages}（须 ≥ 1，否则分页循环空）");
    }
}

/// `total_pages_for` 与 `total_pages`（`MsgKind` 枚举版）同源：表值一致。
#[test]
fn total_pages_for_matches_enum_variant() {
    use ti_core::domain::messages::MsgKind;
    let bal = ti_core::data::balance().expect("balance.json");
    for kind in [MsgKind::Help, MsgKind::Msgs, MsgKind::Msgl, MsgKind::Msgm] {
        let key = ti_core::rules::messages::kind_key(kind);
        let via_str = total_pages_for(key);
        let via_enum = ti_core::rules::messages::total_pages(kind);
        assert_eq!(via_str, via_enum, "{key} 字符串版 = 枚举版");
        assert!(via_str == bal.messages.pages.get(key).copied().unwrap_or(3), "与 balance 表一致");
    }
}
