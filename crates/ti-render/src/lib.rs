//! ti-render：纯渲染框架（零游戏语义，可复用到其他游戏）。
//!
//! 提供帧缓冲抽象、绘制原语、字体系统、主题系统、文本排版、等距投影、场景外壳组件。
//!
//! 设计原则：
//! - **零游戏语义**：不出现场景名/角色/战斗/存档等游戏概念
//! - **数据驱动**：所有颜色/字号从 [`Theme`] 读取，不硬编码
//! - **平台无关**：通过 [`Canvas`] trait 抽象呈现后端，不依赖具体平台
//! - **绘制顺序 = 调用顺序**：本 crate **不提供** z-order 层枚举（曾有 `zorder::ZOrder`
//!   死枚举，零消费方，已删）；层序权威在 `ti-app` 的 `scenes::chrome::scene_shell`
//!   调用序，见 `docs/process/08_render_arch.md` §4

pub mod canvas;
pub mod font;
pub mod framebuffer;
pub mod icons;
pub mod iso;
pub mod primitives;
pub mod shell;
pub mod text;
pub mod theme;

// 便捷重导出
pub use icons::{ICON_COUNT, ICON_SIZE, icon, is_valid as is_valid_icon};
pub use primitives::parchment_bg;
pub use shell::{
    bar_slot, draw_body_text, draw_body_text_secondary, draw_title_text, hint_row, menu_icon,
    menu_item, nautical_grid, panel, title_bar, wood_frame,
};

pub use canvas::Canvas;
pub use font::{FontSet, TtfFont};
pub use framebuffer::{CANVAS_H, CANVAS_W, Framebuf, LOGICAL_H_I32, LOGICAL_W_I32};
pub use iso::IsoProjection;
pub use theme::Theme;
