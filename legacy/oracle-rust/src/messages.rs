//! oracle：消息分页状态机差分基准，测试专用（S11 起降级为纯基准函数）。
//!
//! **类型已迁至 [`crate::domain::messages`]**（S11 归档改造）。
//! 本模块只保留从伪代码直译的**差分基准行为函数**（分页转换 / 可达种类）。
//!
//! 业务规则以 `docs/business/loop_core.md` §消息为准；页数以 `balance.messages.pages` 为准。

pub use ti_core::domain::messages::{MsgEvent, MsgKind, MsgPage};

/// oracle：各类界面总页数（逆向初值；权威值在 `balance.messages.pages`）。
#[must_use]
pub const fn oracle_total_pages(kind: MsgKind) -> u32 {
    match kind {
        MsgKind::Help => 3,
        MsgKind::Msgs | MsgKind::Msgl | MsgKind::Msgm | MsgKind::Info => 1,
    }
}

/// oracle：消息/帮助/信息界面翻页状态机。
///
/// 规则（spec §c，边界定死）：
/// - `Open(kind)` → `Some(MsgPage { kind, page: 0, total_pages: oracle_total_pages(kind) })`；
/// - `NextPage`：`cur=None` → `None`；否则 `page+1`，到末页（`page == total-1`）→ 保持（`选项场景帧处理` 行 54）；
/// - `PrevPage`：`cur=None` → `None`；否则 `page==0` → 回末页（`total-1`，`选项场景帧处理` 行 40-43 循环）；
///   `total==1` 时 `page` 恒 0，无变化；
/// - `Close` → `None`（`cur=None` 时仍 `None`，幂等）。
#[must_use]
pub fn oracle_message_transition(cur: Option<MsgPage>, ev: MsgEvent) -> Option<MsgPage> {
    match ev {
        MsgEvent::Open(kind) => {
            Some(MsgPage { kind, page: 0, total_pages: oracle_total_pages(kind) })
        }
        MsgEvent::NextPage => cur.map(|c| MsgPage {
            kind: c.kind,
            page: if c.page + 1 < c.total_pages { c.page + 1 } else { c.page },
            total_pages: c.total_pages,
        }),
        MsgEvent::PrevPage => cur.map(|c| MsgPage {
            kind: c.kind,
            page: if c.page == 0 { c.total_pages - 1 } else { c.page - 1 },
            total_pages: c.total_pages,
        }),
        MsgEvent::Close => None,
    }
}

/// oracle：可 Open 的界面种类全集（spec §a 全部 5 类均可由事件进入）。
#[must_use]
pub fn oracle_reachable_kinds() -> Vec<MsgKind> {
    MsgKind::all().to_vec()
}

/// oracle：某类界面某页的确定性占位文本行（与 page 无关）。
///
/// 注释（spec §b）：文本内容属资源（`word_7E470`/`word_7D694` 等宽字符串常量 + `cards/event.ini`
/// 条目），oracle 只建模翻页状态机，不还原原文；占位文本以界面种类为键，逐位确定。
#[must_use]
pub fn oracle_text_line(kind: MsgKind, _page: u32) -> &'static str {
    match kind {
        MsgKind::Msgs => "msgs: placeholder line",
        MsgKind::Msgl => "msgl: placeholder line",
        MsgKind::Msgm => "msgm: placeholder line",
        MsgKind::Help => "help: placeholder line",
        MsgKind::Info => "info: placeholder line",
    }
}
