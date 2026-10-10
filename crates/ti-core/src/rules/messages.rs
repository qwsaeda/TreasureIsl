//! S3.11 / S8.3.e 消息/帮助/信息界面正式实现（文本 key 化 + 页数数据化）。
//!
//! S8 定位变更：旧 `oracle::messages::oracle_message_transition`/`oracle_total_pages`
//! 降级参照物；本模块的翻页状态机直接落本文件（`docs/business/loop_core.md` §消息），
//! 文本标签走 `strings.json`（`msg.*` key，非中文字面量），页数走 `balance.messages`。
//!
//! 边界规则（`docs/business/loop_core.md` 消息界面，与 oracle 行为一致）：
//! - `next` 到最后一页 → 保持；
//! - `prev` 到第一页 → 回最后一页（循环）；
//! - `close` → 关闭（`None`）；`open(kind)` → `page=0`。
//!
//! [`MsgKind`] 仍从 oracle 重导出（枚举定义非推测公式，S8.3.e 允许）；
//! `MsgPage`/页数/文本标签改由本模块持有（页数来自 `balance.messages.pages`，文本 key 化）。

use crate::data as data_alias;

/// 消息界面种类（`docs/business/loop_core.md`，5 类；从 `oracle::messages` 重导出稳定标识）。
pub type MsgKind = crate::oracle::messages::MsgKind;

/// 当前打开的消息页（`None` = 界面未打开）。
///
/// 页数不再读 oracle，走 `balance.messages.pages`（`total_pages_for` 注入）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgPage {
    /// 界面种类。
    pub kind: MsgKind,
    /// 当前页（0-based）。
    pub page: u32,
    /// 总页数（`balance.messages.pages`，`total_pages_for` 计算）。
    pub total_pages: u32,
}

/// 某类消息界面的总页数（`balance.messages.pages`，缺省回退 3，不 panic）。
///
/// 页数 = 该 kind 的文本 key 条数（`strings.json` 的 `msg.<kind>.<n>`），
/// 单一事实源在 `balance.json → messages`。整表改坏 → 回退 3（与旧 oracle 一致）。
#[must_use]
pub fn total_pages(kind: MsgKind) -> u32 {
    total_pages_for(kind_key(kind))
}

/// 按 kind 短 key 查总页数（`balance.messages.pages` 缺键回退 3，不 panic）。
#[must_use]
pub fn total_pages_for(kind_key: &str) -> u32 {
    match data_alias::balance() {
        Ok(b) => b.messages.pages.get(kind_key).copied().unwrap_or(3),
        Err(_) => 3,
    }
}

/// 某类消息界面第 `page` 页的文本 key（`strings.json` 的 `msg.<kind>.<page+1>`，key 化非中文）。
/// 越界页 → 回最后一页 key（不 panic）。
#[must_use]
pub fn page_text_key(kind: MsgKind, page: u32) -> String {
    let total = total_pages(kind);
    let p = if page >= total { total.saturating_sub(1) } else { page };
    format!("msg.{}.{}", kind_key(kind), p + 1)
}

/// kind → 短 key（`strings.json` 的 `msg.*` 前缀，稳定字符串，非中文）。
fn kind_key(k: MsgKind) -> &'static str {
    match k {
        MsgKind::Help => "help",
        MsgKind::Info => "info",
        MsgKind::Msgs => "msgs",
        MsgKind::Msgl => "msgl",
        MsgKind::Msgm => "msgm",
    }
}

#[allow(dead_code)]
fn kind_key_static(k: MsgKind) -> &'static str {
    kind_key(k)
}

/// 消息/帮助/信息界面会话：当前打开的页（`None` = 界面未打开）。
#[derive(Debug, Clone, Default)]
pub struct MessageSession {
    cur: Option<MsgPage>,
}

impl MessageSession {
    /// 新建空会话（无打开界面）。
    #[must_use]
    pub fn new() -> Self {
        Self { cur: None }
    }

    /// 当前打开的页（只读，`None` = 未打开）。
    #[must_use]
    pub fn current(&self) -> Option<MsgPage> {
        self.cur
    }

    /// 当前界面种类（`None` = 未打开）。
    #[must_use]
    pub fn kind(&self) -> Option<MsgKind> {
        self.cur.map(|p| p.kind)
    }

    /// 打开某类界面（`page=0`，页数走 `balance.messages`）。
    pub fn open(&mut self, kind: MsgKind) {
        self.cur = Some(MsgPage { kind, page: 0, total_pages: total_pages(kind) });
    }

    /// 下一页（到末页保持）。
    pub fn next(&mut self) {
        if let Some(page) = self.cur {
            let next_page =
                if page.page + 1 < page.total_pages { page.page + 1 } else { page.page };
            self.cur =
                Some(MsgPage { kind: page.kind, page: next_page, total_pages: page.total_pages });
        }
    }

    /// 上一页（首页回末页，循环）。
    pub fn prev(&mut self) {
        if let Some(page) = self.cur {
            let next_page =
                if page.page == 0 { page.total_pages.saturating_sub(1) } else { page.page - 1 };
            self.cur =
                Some(MsgPage { kind: page.kind, page: next_page, total_pages: page.total_pages });
        }
    }

    /// 关闭/返回（`cur = None`）。
    pub fn close(&mut self) {
        self.cur = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_pages_from_balance() {
        // `balance.messages.pages` 已配 `msgl = 1`（数据注入优先于缺省回退）
        assert_eq!(total_pages(MsgKind::Msgl), 1, "balance.messages.pages.msgl = 1");
    }

    #[test]
    fn total_pages_falls_back_when_missing() {
        // 缺键 kind 回退 3（与旧 oracle 一致，不 panic）
        assert_eq!(total_pages_for("nonexistent_kind"), 3, "缺省页数回退 3");
    }

    #[test]
    fn open_starts_at_page_zero() {
        let mut s = MessageSession::new();
        s.open(MsgKind::Help);
        let p = s.current();
        assert!(p.is_some(), "open 后有页（会话不变式）");
        let p = p.unwrap_or(MsgPage { kind: MsgKind::Help, page: 0, total_pages: 0 });
        assert_eq!(p.page, 0);
        assert_eq!(p.kind, MsgKind::Help);
        assert_eq!(p.total_pages, total_pages(MsgKind::Help));
    }

    #[test]
    fn next_stops_at_last_page() {
        let mut s = MessageSession::new();
        s.open(MsgKind::Info);
        let total = total_pages(MsgKind::Info);
        for _ in 0..total {
            s.next();
        }
        // 多 next 保持末页
        s.next();
        let p = s.current();
        assert!(p.is_some(), "next 后仍有页（会话不变式）");
        let p = p.unwrap_or(MsgPage { kind: MsgKind::Info, page: 0, total_pages: 0 });
        assert_eq!(p.page, total - 1, "到末页保持");
    }

    #[test]
    fn prev_wraps_to_last_page() {
        let mut s = MessageSession::new();
        s.open(MsgKind::Msgs);
        s.next(); // page 1
        s.prev(); // page 0
        s.prev(); // 回末页
        let p = s.current();
        assert!(p.is_some(), "prev 后仍有页（会话不变式）");
        let p = p.unwrap_or(MsgPage { kind: MsgKind::Msgs, page: 0, total_pages: 0 });
        assert_eq!(p.page, total_pages(MsgKind::Msgs) - 1, "首页 prev 回末页");
    }

    #[test]
    fn close_sets_none() {
        let mut s = MessageSession::new();
        s.open(MsgKind::Help);
        s.close();
        assert!(s.current().is_none(), "close 后无页");
    }

    #[test]
    fn page_text_key_is_i18n_not_cjk() {
        let key = page_text_key(MsgKind::Help, 0);
        assert_eq!(key, "msg.help.1", "文本走 strings key，非中文");
        assert!(!key.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)));
        let _ = kind_key_static(MsgKind::Help); // 保持辅助函数被引用
    }
}
