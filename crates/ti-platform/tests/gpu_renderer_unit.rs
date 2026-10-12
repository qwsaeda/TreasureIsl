//! `GpuRenderer` 单测（S10.4，EXP-052：内联测试移至 tests/ 目录）。

use ti_platform::gpu_renderer::{GpuRenderer, SpriteRef};

#[test]
fn headless_renderer_counts_draw_calls() {
    let mut r = GpuRenderer::new_headless();
    assert!(r.is_headless());
    let ref0 = SpriteRef { texture: 1, rect: (0, 0, 16, 16) };
    r.blit_sprite((10, 10), &ref0);
    r.blit_terrain_tile((20, 20), &ref0);
    assert_eq!(r.draw_call_count(), 2, "headless 模式应累加 draw call 计数");
}

#[test]
fn new_renderer_is_not_headless() {
    let r = GpuRenderer::new();
    assert!(!r.is_headless());
    assert_eq!(r.draw_call_count(), 0);
}

#[test]
fn blit_without_atlas_increments_headless_count() {
    let mut r = GpuRenderer::new_headless();
    let ref0 = SpriteRef { texture: 1, rect: (0, 0, 8, 8) };
    r.blit_sprite((5, 5), &ref0);
    assert_eq!(r.draw_call_count(), 1);
}
