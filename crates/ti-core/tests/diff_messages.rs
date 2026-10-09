//! S3.11 消息/帮助/信息界面差分测试。
//!
//! 遍历全部 (`MsgKind` × page 0..=3 × `MsgEvent` 全组合 + 首/末页边界)，
//! oracle 与 impl 的输出逐位一致（`Option<MsgPage>` 全等）。
//! 另含 2 个语义测试。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::messages::{
    MsgEvent, MsgKind, MsgPage, oracle_message_transition, oracle_reachable_kinds,
    oracle_total_pages,
};
use ti_core::rules::messages::MessageSession;

/// 将 oracle 的 `Option<MsgPage>` 与 impl 的 `current()` 逐位对比。
fn assert_page_equal(oracle: Option<MsgPage>, impl_page: Option<MsgPage>) {
    assert_eq!(oracle, impl_page, "翻页状态不一致: oracle={oracle:?} impl={impl_page:?}");
}

/// 按 `MsgEvent` 驱动 impl 会话（构造与 oracle 同输入路径）。
fn apply_event(session: &mut MessageSession, ev: MsgEvent) {
    match ev {
        MsgEvent::Open(kind) => session.open(kind),
        MsgEvent::NextPage => session.next(),
        MsgEvent::PrevPage => session.prev(),
        MsgEvent::Close => session.close(),
    }
}

/// 遍历全部 (`MsgKind` × page 0..=3 × 4 `MsgEvent`) 组合，oracle vs impl 逐位一致。
#[test]
fn diff_all_combinations() {
    for kind in MsgKind::all() {
        for page in 0..=3u32 {
            // 构造当前页（page < total_pages 时取实际 page，否则保持末页，模拟真实状态）
            let total = oracle_total_pages(kind);
            let clamped_page = if page < total { page } else { total - 1 };
            let cur = MsgPage { kind, page: clamped_page, total_pages: total };

            for ev in MsgEvent::all() {
                let oracle_out = oracle_message_transition(Some(cur), ev);

                let mut session = MessageSession::new();
                apply_event(&mut session, MsgEvent::Open(kind));
                // 手动推进到 clamped_page（模拟已打开并翻页到位的状态）
                for _ in 0..clamped_page {
                    session.next();
                }
                apply_event(&mut session, ev);

                assert_page_equal(oracle_out, session.current());
            }
        }
    }
}

/// `cur=None`（界面未打开）时的全事件组合，oracle vs impl 逐位一致。
#[test]
fn diff_none_cur() {
    for ev in MsgEvent::all() {
        let oracle_out = oracle_message_transition(None, ev);
        let mut session = MessageSession::new();
        apply_event(&mut session, ev);
        assert_page_equal(oracle_out, session.current());
    }
}

/// 边界：`Help`（`total_pages` = 3）`NextPage` 到末页保持。
#[test]
fn diff_help_nextpage_boundary() {
    let total = oracle_total_pages(MsgKind::Help);
    assert_eq!(total, 3, "Help 总页数应为 3（sub_41354 行 202 证据）");

    let mut session = MessageSession::new();
    session.open(MsgKind::Help);
    session.next();
    session.next();
    // page=2（末页），再 NextPage 应保持 page=2
    session.next();
    let page = session.current().expect("打开 Help 后应有当前页");
    assert_eq!(page.page, 2, "末页 NextPage 应保持 page=2");

    let cur = MsgPage { kind: MsgKind::Help, page: 2, total_pages: 3 };
    let oracle_out = oracle_message_transition(Some(cur), MsgEvent::NextPage);
    assert_page_equal(oracle_out, session.current());
}

/// 边界：`Help` `PrevPage` 到首页回末页（循环语义）。
#[test]
fn diff_help_prevpage_boundary() {
    let mut session = MessageSession::new();
    session.open(MsgKind::Help);
    // page=0（首页），PrevPage 应回 page=2（末页）
    session.prev();
    let page = session.current().expect("打开 Help 后应有当前页");
    assert_eq!(page.page, 2, "首页 PrevPage 应循环回末页 page=2");

    let cur = MsgPage { kind: MsgKind::Help, page: 0, total_pages: 3 };
    let oracle_out = oracle_message_transition(Some(cur), MsgEvent::PrevPage);
    assert_page_equal(oracle_out, session.current());
}

/// 可达性：`oracle_reachable_kinds` 全部 5 类都可 Open，oracle vs impl 逐位一致。
#[test]
fn diff_reachable_kinds() {
    let reachable = oracle_reachable_kinds();
    assert_eq!(reachable.len(), 5, "应有 5 类可 Open");
    for kind in reachable {
        let mut session = MessageSession::new();
        session.open(kind);
        assert_eq!(session.kind(), Some(kind), "Open({kind:?}) 后 kind() 应返回 {kind:?}");
    }
}

/// 语义测试 (a)：`Open(Msgs)` → `NextPage` × 2 → `cur.page == 0`（单页保持）
/// 且 `Open(Help)` → `NextPage` × 2 → `cur.page == 2`（3 页 Help 翻到末页）。
#[test]
fn semantic_open_next_two() {
    // (a-1) Msgs 单页：NextPage ×2 均保持 page=0
    let mut session = MessageSession::new();
    session.open(MsgKind::Msgs);
    session.next();
    session.next();
    let page = session.current().expect("打开 Msgs 后应有当前页");
    assert_eq!(page.kind, MsgKind::Msgs);
    assert_eq!(page.page, 0, "Msgs 单页 NextPage×2 应保持 page=0");
    assert_eq!(page.total_pages, 1);

    // (a-2) Help 3 页：NextPage ×2 到 page=2（末页）
    let mut session_help = MessageSession::new();
    session_help.open(MsgKind::Help);
    session_help.next();
    session_help.next();
    let page_help = session_help.current().expect("打开 Help 后应有当前页");
    assert_eq!(page_help.kind, MsgKind::Help);
    assert_eq!(page_help.page, 2, "Help NextPage×2 应到 page=2（末页）");
    assert_eq!(page_help.total_pages, 3);
}

/// 语义测试 (b)：`Close` → `current() == None`，且关闭后 `NextPage` 仍 `None`。
#[test]
fn semantic_close_then_next_still_none() {
    let mut session = MessageSession::new();
    session.open(MsgKind::Help);
    session.next();
    assert!(session.current().is_some(), "Open+NextPage 后应有当前页");

    session.close();
    assert!(session.current().is_none(), "Close 后 current() 应为 None");
    assert_eq!(session.kind(), None, "Close 后 kind() 应为 None");

    session.next();
    assert!(session.current().is_none(), "关闭后 NextPage 仍应 None");

    // 与 oracle 对照
    let cur_closed = None;
    let oracle_out = oracle_message_transition(cur_closed, MsgEvent::NextPage);
    assert_eq!(oracle_out, None, "oracle 关闭后 NextPage 应 None");
}
