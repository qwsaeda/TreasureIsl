//! S3.11 / S8.3.e / S12.4 消息/帮助/信息界面正式实现（文本 key 化 + 页数数据化）。
//!
//! 本模块的翻页状态机直接落本文件（`docs/business/loop_core.md` §消息 +
//! `07_ui_flow.md` §4 消息分页），类型全部来自 `domain::messages`（S11 类型唯一家）。
//!
//! 边界规则（`docs/business/loop_core.md` 消息界面）：
//! - `next` 到最后一页 → 保持；
//! - `prev` 到第一页 → 回最后一页（循环）；
//! - `close` → 关闭（回到打开源场景，`FlowSource` 由调用方注入）；`open(kind, source)` → `page=0`。
//!
//! 页数来自 `balance.messages.pages`（`total_pages` 读表，缺省回退 3 不 panic）；
//! 文本 key 走 `strings.json`（`msg.<kind>.<n>`，非中文字面量）。

use crate::data as data_alias;
use crate::domain::messages::{FlowSource, MsgEvent, MsgKind, MsgPage};

/// 某类消息界面的总页数（`balance.messages.pages`，缺省回退 3，不 panic）。
///
/// 页数 = 该 kind 的文本 key 条数（`strings.json` 的 `msg.<kind>.<n>`），
/// 单一事实源在 `balance.json → messages`。整表改坏 → 回退 3。
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
/// 越界页 → 回最后一页 key（不 panic，`07_ui_flow.md` §4 首尾钳位）。
#[must_use]
pub fn page_text_key(kind: MsgKind, page: u32) -> String {
    let total = total_pages(kind);
    let p = if page >= total { total.saturating_sub(1) } else { page };
    format!("msg.{}.{}", kind_key(kind), p + 1)
}

/// 某类消息界面第 `page` 页的纯文本 key（`balance.messages.pages` 分页 + 越界钳末页，不 panic）。
///
/// 实现：返回该页的文本 key（`strings.json` 查表由调用方经 `i18n::sget` 完成；
/// 本函数只做 `balance.messages` 分页钳位，保持纯函数无 IO、可单测）。
#[must_use]
pub fn page_text(kind: MsgKind, page: u32) -> String {
    page_text_key(kind, page)
}

/// kind → 短 key（`strings.json` 的 `msg.*` 前缀，稳定字符串，非中文）。
#[must_use]
pub fn kind_key(k: MsgKind) -> &'static str {
    match k {
        MsgKind::Help => "help",
        MsgKind::Info => "info",
        MsgKind::Msgs => "msgs",
        MsgKind::Msgl => "msgl",
        MsgKind::Msgm => "msgm",
    }
}

/// 消息会话（`07_ui_flow.md` §4/§6 规则 10/11/12）：翻页钳位 + 关闭回打开源场景 + 切场景复位。
///
/// 与 `MessageSession`（S8 旧 API）同语义，并补全 `MsgEvent` 分派 + 打开源场景归属
/// （`FlowSource`）+ 场景切换复位（`scene_switch_resets_state` 的域层语义：
/// `open(kind, source)` 强制 `page=0`，不残留旧页码/旧种类）。
#[derive(Debug, Clone)]
pub struct MsgSession {
    page: Option<MsgPage>,
    source: FlowSource,
}

impl MsgSession {
    /// 新建未打开会话（默认打开源 = 主菜单，`07_ui_flow.md` §1 `Message → Index` 出边）。
    #[must_use]
    pub fn new() -> Self {
        Self { page: None, source: FlowSource::FromIndex }
    }

    /// 当前页（`None` = 未打开）。
    #[must_use]
    pub fn current(&self) -> Option<MsgPage> {
        self.page
    }

    /// 是否已打开。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.page.is_some()
    }

    /// 当前种类（`None` = 未打开）。
    #[must_use]
    pub fn kind(&self) -> Option<MsgKind> {
        self.page.map(|p| p.kind)
    }

    /// 打开源场景（未打开时返回默认 `FromIndex`）。
    #[must_use]
    pub fn source(&self) -> FlowSource {
        self.source
    }

    /// 打开某类界面（`page=0`，页数走 `balance.messages`；同时记录打开源场景）。
    ///
    /// `07_ui_flow.md` §6 规则 12（`scene_switch_resets_state`）：打开即复位页码与种类，
    /// 禁止残留上一次会话的 `page`/`kind`。
    pub fn open(&mut self, kind: MsgKind, source: FlowSource) {
        self.page = Some(MsgPage { kind, page: 0, total_pages: total_pages(kind) });
        self.source = source;
    }

    /// 处理消息事件（`MsgEvent` 分派，`07_ui_flow.md` §4 翻页钳位 + §6 规则 10/11）。
    ///
    /// 返回是否产生可见状态变化（表现层据此决定重绘）：
    /// - `Open` / `Close`：恒为 `true`；
    /// - `NextPage`：未到末页时 `true`（到末页保持 → `false`）；
    /// - `PrevPage`：首页循环回末页 → 总页数 > 1 时 `true`，单页时 `false`。
    #[must_use]
    pub fn handle(&mut self, ev: MsgEvent) -> bool {
        match ev {
            MsgEvent::Open(kind) => {
                // 经 `MsgEvent::Open` 进入时源场景未知，保留当前 `source`（不覆盖）。
                let source = self.source;
                self.open(kind, source);
                true
            }
            MsgEvent::NextPage => {
                let Some(page) = self.page else {
                    return false;
                };
                if page.page + 1 < page.total_pages {
                    self.page = Some(MsgPage {
                        kind: page.kind,
                        page: page.page + 1,
                        total_pages: page.total_pages,
                    });
                    true
                } else {
                    false
                }
            }
            MsgEvent::PrevPage => {
                let Some(page) = self.page else {
                    return false;
                };
                let next =
                    if page.page == 0 { page.total_pages.saturating_sub(1) } else { page.page - 1 };
                let changed = next != page.page;
                self.page =
                    Some(MsgPage { kind: page.kind, page: next, total_pages: page.total_pages });
                changed
            }
            MsgEvent::Close => {
                self.page = None;
                true
            }
        }
    }

    /// 关闭后应回到的场景码（`07_ui_flow.md` §6 规则 11 `msg_close_returns_source`）：
    /// 据点/面板/结算从主地图打开 → 回主地图；菜单/帮助从主菜单打开 → 回主菜单。
    /// 未打开时返回 `None`（无界可回）。
    #[must_use]
    pub fn close_target(&self) -> Option<u32> {
        if !self.is_open() {
            return None;
        }
        Some(match self.source {
            FlowSource::FromMainMap => 4,
            FlowSource::FromIndex => 0,
        })
    }

    /// 场景切换复位（`07_ui_flow.md` §6 规则 12 `scene_switch_resets_state`）：
    /// 切场景后消息会话强制关闭，页码/种类/打开源全部不残留。
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for MsgSession {
    fn default() -> Self {
        Self::new()
    }
}
