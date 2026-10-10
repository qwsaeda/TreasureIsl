//! oracle：S3.11 消息/帮助/信息界面差分基准，测试专用。
//!
//! 直译 `docs/spec/messages.md` 的翻页状态机：
//! - [`MsgKind`]：5 类界面（`Msgs`/`Msgl`/`Msgm`/`Help`/`Info`）；
//! - [`MsgPage`]（kind, `page`, [`total_pages`]）三元组，`page ∈ 0..total_pages`；
//! - [`MsgEvent`]：`Open(kind)` / `NextPage` / `PrevPage` / `Close`；
//! - [`oracle_message_transition`]：同一 (当前页, 事件) 产生同一 `Option<MsgPage>`，逐位基准；
//! - [`oracle_reachable_kinds`]：全部 5 类都可 `Open`；
//! - [`oracle_text_line`]：确定性占位文本（与 page 无关，文本内容属资源，oracle 只建模翻页状态）。
//!
//! 边界规则（spec §c，定死）：
//! - `NextPage` 到最后一页 → 保持最后一页（`page` 不变，`选项场景帧处理` 行 54 语义）；
//! - `PrevPage` 到第一页（`page==0`）→ 回最后一页（`page = total_pages - 1`，`选项场景帧处理` 行 40-43 循环语义）；
//! - `Close` → `None`；
//! - `Open(kind)` → `page = 0`，`total_pages` 取 [`oracle_total_pages`]；
//! - 无当前页（`None`）时 `NextPage`/`PrevPage` → `None`（界面未打开）。
//!
//! `total_pages`（spec §c 常量表，`Help=3` 有 `saveload` 行 202 证据，其余 1 为推测）：
//! 见 [`oracle_total_pages`]。

/// 消息/帮助/信息界面种类（spec §a 五类）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgKind {
    /// `msgs.map` 通用消息提示/退出对话框（事件码 8 的 `虚调用+80==0` 分支）。
    Msgs,
    /// `msgl.map` 据点名/地标对话框 + 资源结算面板（事件码 1/4）。
    Msgl,
    /// `msgM.map` 多页面板/卡片/贸易结果（事件码 2/5/7/8 的 `!=0` 分支）。
    Msgm,
    /// `help.map` + `switchpage.pcx` 多页帮助文本（主菜单「说明」case 3/4）。
    Help,
    /// `information.map` 信息/角色状态界面（场景管理器 case 5，`00029110:场景管理器` 专属 vtable）。
    Info,
}

impl MsgKind {
    /// 全部种类（按枚举声明顺序，diff 测试遍历用）。
    #[must_use]
    pub const fn all() -> [MsgKind; 5] {
        [Self::Msgs, Self::Msgl, Self::Msgm, Self::Help, Self::Info]
    }

    /// 可读名称（壳层渲染用，`docs/gamebook.md` §3.5）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Msgs => "消息",
            Self::Msgl => "据点",
            Self::Msgm => "面板",
            Self::Help => "帮助",
            Self::Info => "信息",
        }
    }
}

/// 单页状态：界面种类 + 当前页码 + 总页数（`page ∈ 0..total_pages`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgPage {
    /// 界面种类。
    pub kind: MsgKind,
    /// 当前页码（0 起，`选项场景帧处理` 的 `v3[22]` 语义，spec §c）。
    pub page: u32,
    /// 总页数（来自 [`oracle_total_pages`]，spec §c 常量表）。
    pub total_pages: u32,
}

/// 消息/帮助/信息界面事件（spec §c 事件码归一）。
///
/// - `Open(kind)`：打开某类界面（主菜单「说明」/据点事件/场景管理器 case 5），`page=0`；
/// - `NextPage`：下一页（事件码 38，`选项场景帧处理` 行 52-61 语义，到末页保持）；
/// - `PrevPage`：上一页（事件码 37，`选项场景帧处理` 行 40-43 循环语义，首页回末页）；
/// - `Close`：关闭/返回（事件码 27，`帮助`/`选项场景帧处理` 行 112/100 语义）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgEvent {
    /// 打开某类界面（`MsgKind` 参数）。
    Open(MsgKind),
    /// 下一页（事件码 38）。
    NextPage,
    /// 上一页（事件码 37，首页回末页循环）。
    PrevPage,
    /// 关闭/返回 index（事件码 27）。
    Close,
}

impl MsgEvent {
    /// 全部事件变体（diff 测试遍历用）。
    ///
    /// `Open` × 5 类 + `NextPage` + `PrevPage` + `Close` = 8 个独立变体。
    #[must_use]
    pub const fn all() -> [MsgEvent; 8] {
        [
            Self::Open(MsgKind::Msgs),
            Self::Open(MsgKind::Msgl),
            Self::Open(MsgKind::Msgm),
            Self::Open(MsgKind::Help),
            Self::Open(MsgKind::Info),
            Self::NextPage,
            Self::PrevPage,
            Self::Close,
        ]
    }
}

/// 各类界面的总页数（spec §c 常量表）。
///
/// `Help = 3`（证据：`saveload` 行 202 `help(..., 3, aF)` 第 4 参数 = 3 页）；
/// 其余 4 类 `= 1`（**推测**：单页对话框，`对话框文本行写入器` 翻页标志未逐点核对）。
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
