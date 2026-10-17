//! Z-order 层枚举：定义渲染层叠顺序。
//!
//! 数值越小越靠后（先绘制），数值越大越靠前（后绘制覆盖先绘制）。
//! 场景元素按此枚举排序，确保正确的遮挡关系。

/// 渲染层叠顺序（从后到前）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZOrder {
    /// 背景底板（渐变 + 暗角）。
    BgBoard = 0,
    /// 航海网格装饰层（低对比海网格）。
    BgGrid = 1,
    /// 卡片外框。
    Frame = 2,
    /// 卡片标题条。
    TitleBar = 3,
    /// 卡片内容区（菜单项/正文）。
    Content = 4,
    /// 文字层（标题/正文/提示）。
    Text = 5,
    /// 血条槽位。
    BarSlot = 6,
    /// 高亮/选中层。
    Highlight = 7,
}

impl ZOrder {
    /// 层级数值（用于排序比较）。
    #[must_use]
    pub const fn value(self) -> i32 {
        self as i32
    }
}
