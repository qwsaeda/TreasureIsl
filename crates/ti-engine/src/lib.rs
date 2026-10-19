//! S18：Bevy 呈现后端（`docs/process/11_bevy_migration.md` §4 架构形状）。
//!
//! 定位 = 与 `ti-render` 对等：**零游戏语义**（场景名/角色/战斗等一律不出现），
//! 只做「Bevy ↔ 渲染框架」桥接：
//! - `bevy_app::SpikeApp`（S18.1）：最小 Bevy App 构造（本 spike 验证三端可跑）；
//! - S18.2 起落 `BevyCanvas`：实现 `ti_render::canvas::Canvas`，把 1080 坐标系
//!   绘制原语转 Bevy immediate 绘制（Mesh2D/Text）+ Camera2D letterbox。
//!
//! 依赖方向：`ti-engine → ti-render + bevy`（doccheck2 框架边界断言在 S18.6 更新）。

mod bevy_app;

pub use bevy_app::SpikeApp;

/// 构建 S18.1 spike 最小 Bevy App（窗口 540×960 竖屏逻辑 1080×1920，
/// 渲染 1 矩形 + 1 段文字；native 实跑 / wasm-webgl2 / android-vulkan 编译验证）。
pub fn spike_app() -> SpikeApp {
    SpikeApp::new()
}
