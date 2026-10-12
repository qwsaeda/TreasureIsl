//! 消息域类型（业务文档：`docs/business/loop_core.md` §消息）。
//!
//! 五类消息界面（据点对话框 / 面板 / 通用提示 / 帮助 / 信息）+ 分页状态。
//! **页数**来自 `assets/data/balance.json` 的 `messages.pages`（S10.3 数据化），
//! 不在此硬编码（`rules::messages::total_pages` 读表）。

/// 消息界面种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgKind {
    /// 通用消息提示 / 退出对话框。
    Msgs,
    /// 据点名 / 地标对话框 + 资源结算面板。
    Msgl,
    /// 多页面板 / 卡片 / 贸易结果。
    Msgm,
    /// 多页帮助文本。
    Help,
    /// 信息 / 角色状态界面。
    Info,
}

impl MsgKind {
    /// 全部种类（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [MsgKind; 5] {
        [Self::Msgs, Self::Msgl, Self::Msgm, Self::Help, Self::Info]
    }

    /// 可读名称（shell 层渲染用；正式文案走 `strings.json` key `msg.<kind>.label`）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Msgs => "msgs",
            Self::Msgl => "landmark",
            Self::Msgm => "panel",
            Self::Help => "help",
            Self::Info => "info",
        }
    }
}

/// 单页状态：种类 + 当前页码 + 总页数（`page ∈ 0..total_pages`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgPage {
    /// 界面种类。
    pub kind: MsgKind,
    /// 当前页码（0 起）。
    pub page: u32,
    /// 总页数（来自 `balance.messages.pages`）。
    pub total_pages: u32,
}

impl MsgPage {
    /// 场景切换复位（`07_ui_flow.md` §1 硬规则 4 + §6 规则 12 `scene_switch_resets_state`）：
    /// 页码强制回 0，禁止残留上一次会话的页码。
    #[must_use]
    pub fn with_page_reset(mut self) -> Self {
        self.page = 0;
        self
    }
}

/// 消息界面事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgEvent {
    /// 打开某类界面（`page = 0`）。
    Open(MsgKind),
    /// 下一页（**到末页保持**，不回绕）。
    NextPage,
    /// 上一页（**首页回末页**，循环）。
    PrevPage,
    /// 关闭/返回来源场景。
    Close,
}

/// 场景流标识：区分「打开源场景」归属，决定消息关闭后回到哪个场景
/// （`docs/business/07_ui_flow.md` §1 `Message` 的两条出边：主地图 ↔ 主菜单）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowSource {
    /// 从主地图（`MainMap`）场景打开 → 关闭后回 `MainMap`。
    FromMainMap,
    /// 从主菜单（`Index`）场景打开 → 关闭后回 `Index`。
    FromIndex,
}
