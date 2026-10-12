//! S10.4 P4-4：GPU 渲染路径基准（headless 退化 no-op + 注入精灵后 draw call > 0）。
//!
//! 判据（`docs/parity.md` P4-4）：
//! 1. headless 模式下 `GpuRenderer` 退化为 no-op（不实际上传 GPU 纹理）；
//! 2. 每个 GPU blit 调用累计 `draw_calls` 计数；
//! 3. **注入 `ui.pak` 精灵后**完整场景渲染 `draw_calls > 0`（证明 GPU 路径被走到）。
//!
//! 与 `render_parity.rs`（RGB565 CPU 路径，P4-1）/`render_parity_hd.rs`（RGBA8，P5-2）
//! 并行：feature `gpu-render` 默认开；本测试在 headless 后端跑（不依赖窗口/GPU）。
//!
//! 注：本测试文件由 S10.4 落地；macroquad GPU API 不确定处用伪代码跳过，
//! 下轮编译时验证（EXP：API 确认点见 `gpu_renderer.rs` 注释）。
//!
//! Q009 修复：原断言 `calls >= 0`（u64 恒真，无判据力）改为「注入 ui.pak 后 > 0」。

#[cfg(feature = "gpu-render")]
mod tests {
    use ti_app::game_app::{GameApp, GameInput};

    /// 渲染当前场景（CPU 路径 + GPU 路径），返回 `GameApp` 供断言。
    fn render_current(app: &mut GameApp) -> &GameApp {
        app.render();
        app
    }

    /// 读本地 `data/ui.pak`（仓库根 `data/`，native 形态）；缺则返回 None（测试仍跑，
    /// 但跳过 > 0 断言——P4-4 判据以「有素材环境」为准，缺素材只验路径不 panic）。
    fn read_ui_pak() -> Option<Vec<u8>> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ui.pak");
        std::fs::read(path).ok()
    }

    /// 主菜单（Index 场景）GPU 渲染基准。
    fn render_index(app: &mut GameApp) {
        render_current(app);
    }

    /// 主地图场景 GPU 渲染基准。
    fn walk_to_main_map(app: &mut GameApp) {
        app.handle(GameInput::Confirm); // Index → SelectRole
        app.handle(GameInput::Next); // 选角
        app.handle(GameInput::Confirm); // → SelectMap
        app.handle(GameInput::Next); // 选图
        app.handle(GameInput::Confirm); // → MainMap
    }

    /// P4-4 判据 1：headless 模式下 GPU 渲染器为 no-op（`is_headless()` 为 `true`）。
    #[test]
    fn gpu_renderer_is_headless_in_default_build() {
        let app = GameApp::new(176.0, 220.0);
        assert!(
            app.gpu_renderer().is_headless(),
            "默认 headless 构建中 GpuRenderer 应退化为 no-op"
        );
    }

    /// P4-4 判据 2/3（Q009 修复）：注入 `ui.pak` 后主菜单 GPU blit `draw_calls > 0`，
    /// 证明 GPU 路径被走到（非整帧跳过）。
    #[test]
    fn index_scene_gpu_draw_calls_positive_with_ui_pak() {
        let mut app = GameApp::new(176.0, 220.0);
        if let Some(bytes) = read_ui_pak() {
            app.set_atlas(&bytes);
        }
        render_index(&mut app);
        let calls = app.gpu_draw_calls();
        if read_ui_pak().is_some() {
            // Q009：有素材时 GPU blit 必 > 0（主菜单至少 1 精灵）
            assert!(calls > 0, "注入 ui.pak 后主菜单 GPU draw_calls 应 > 0，实际 {calls}");
        } else {
            // 缺素材：只验路径不 panic（P4-4 以有素材环境为准）
            let _ = calls;
        }
    }

    /// P4-4 判据：主地图场景 GPU 渲染确定性（两次渲染 draw call 数相同）。
    #[test]
    fn main_map_scene_gpu_render_deterministic() {
        let mut app1 = GameApp::new(176.0, 220.0);
        let mut app2 = GameApp::new(176.0, 220.0);
        if let Some(bytes) = read_ui_pak() {
            app1.set_atlas(&bytes);
            app2.set_atlas(&bytes);
        }
        walk_to_main_map(&mut app1);
        walk_to_main_map(&mut app2);
        render_current(&mut app1);
        render_current(&mut app2);
        assert_eq!(
            app1.gpu_draw_calls(),
            app2.gpu_draw_calls(),
            "两次渲染的 GPU draw call 数应相同（确定性）"
        );
    }
}
