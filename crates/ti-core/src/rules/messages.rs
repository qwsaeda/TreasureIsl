//! S3.11 消息/帮助/信息界面正式实现。
//!
//! 与 `oracle::messages::oracle_message_transition` 行为逐位一致（同一输入产生逐位相同的
//! `Option<MsgPage>`），但结构不同：用 `MessageSession` 结构体 + 方法承载状态，而非自由函数。
//!
//! 依据 `docs/spec/messages.md`：
//! - 界面种类 [`MsgKind`]（5 类，从 oracle 重导出）；
//! - 事件 [`MsgEvent`]（`Open`/`NextPage`/`PrevPage`/`Close`，从 oracle 重导出）；
//! - [`MessageSession`]：当前打开的界面 + 翻页状态机。
//!
//! 边界规则（spec §c，定死）：
//! - `NextPage` 到最后一页 → 保持（`sub_4085C` 行 54）；
//! - `PrevPage` 到第一页 → 回最后一页（`sub_4085C` 行 40-43 循环）；
//! - `Close` → 关闭（`None`）；`Open(kind)` → `page=0`。

use crate::oracle::messages::{MsgKind, MsgPage, oracle_total_pages};

/// 消息/帮助/信息界面会话：当前打开的页（`None` = 界面未打开）。
#[derive(Debug, Clone)]
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

    /// 打开某类界面（事件码 13/据点/场景管理器 case 5 进入路径）。
    ///
    /// 行为与 `oracle_message_transition(None, Open(kind))` 逐位一致：
    /// `cur = Some(MsgPage { kind, page: 0, total_pages: oracle_total_pages(kind) })`。
    pub fn open(&mut self, kind: MsgKind) {
        self.cur = Some(MsgPage { kind, page: 0, total_pages: oracle_total_pages(kind) });
    }

    /// 下一页（事件码 38，`sub_4085C` 行 52-61 语义，到末页保持）。
    ///
    /// 行为与 `oracle_message_transition(self.cur, NextPage)` 逐位一致。
    pub fn next(&mut self) {
        self.cur = match self.cur {
            None => None,
            Some(page) => {
                let next_page =
                    if page.page + 1 < page.total_pages { page.page + 1 } else { page.page };
                Some(MsgPage { kind: page.kind, page: next_page, total_pages: page.total_pages })
            }
        };
    }

    /// 上一页（事件码 37，`sub_4085C` 行 40-43 循环语义，首页回末页）。
    ///
    /// 行为与 `oracle_message_transition(self.cur, PrevPage)` 逐位一致。
    pub fn prev(&mut self) {
        self.cur = match self.cur {
            None => None,
            Some(page) => {
                let next_page = if page.page == 0 { page.total_pages - 1 } else { page.page - 1 };
                Some(MsgPage { kind: page.kind, page: next_page, total_pages: page.total_pages })
            }
        };
    }

    /// 关闭/返回（事件码 27，`sub_3F4E4`/`sub_4085C` 行 112/100 语义）。
    ///
    /// 行为与 `oracle_message_transition(self.cur, Close)` 逐位一致：`cur = None`。
    pub fn close(&mut self) {
        self.cur = None;
    }
}

impl Default for MessageSession {
    fn default() -> Self {
        Self::new()
    }
}
