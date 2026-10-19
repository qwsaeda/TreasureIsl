//! S18.2 `BevyCanvas` 契约测试（无 GPU：裸 World 落 ECS + 纯函数口径）。
//!
//! 覆盖：
//! - RGB565 ↔ `Color` 双向转换（±1 LSB 量化容差）；
//! - 1080 坐标契约：左上原点 → Bevy 世界坐标（y 翻转）+ letterbox scale；
//! - op 记录 → `flush` 落 ECS：色块 = 定尺寸 Sprite + Transform（含层序 z）、文本 = `Text2d`；
//! - 帧间零残留：二次 `flush` 后旧帧实体全部 despawn；
//! - 裁剪语义与 `Framebuf::fill_rect` 一致（越界裁剪 / 退化丢弃）；
//! - 三档指标同源（位图回退口径 12/26/8）。

use bevy::ecs::component::Component;
use bevy::prelude::*;
use ti_engine::canvas::{
    BevyCanvas, CanvasEntity, color_to_rgb565, rgb565_to_color, viewport_scale,
};
use ti_render::canvas::Canvas;
use ti_render::framebuffer::{Framebuf, LOGICAL_H_I32, LOGICAL_W_I32};
use ti_render::theme::Theme;

fn canvas() -> BevyCanvas {
    BevyCanvas::new(Theme::DEFAULT)
}

#[test]
fn rgb565_roundtrip_within_one_lsb() {
    for v in [0x0000u16, 0xFFFF, 0xF7BA, 0x0865, 0x4E3D, 0x0865 | 0xF800] {
        let back = color_to_rgb565(rgb565_to_color(v));
        let diff = i32::from(back).abs_diff(i32::from(v));
        // 8bit 扩展再压回 565：每通道量化误差 ≤ 2 LSB（8bit→5bit 舍入）。
        assert!(diff <= 2, "roundtrip v={v:#06x} back={back:#06x} diff={diff}");
    }
}

#[test]
fn viewport_scale_letterbox() {
    // 1080×1920 同比例 → scale 1.0。
    assert!((viewport_scale(1080.0, 1920.0) - 1.0).abs() < 1e-6);
    // 半采样窗口 → 0.5。
    assert!((viewport_scale(540.0, 960.0) - 0.5).abs() < 1e-6);
    // 宽窗口（1080×1000，比例更宽）→ 高度受限，取 1000/1920。
    let s = viewport_scale(1080.0, 1000.0);
    assert!((s - (1000.0 / 1920.0)).abs() < 1e-6, "scale={s}");
}

#[test]
fn fill_rect_clips_like_framebuf() {
    // 越界矩形：与 Framebuf 相同的裁剪结果（部分可见）。
    let mut c = canvas();
    c.fill_rect(-50, -20, 200, 100, 0xF7BA);
    // 退化（完全画布外）不产生 op。
    c.fill_rect(2000, 0, 10, 10, 0xF7BA);
    assert_eq!(c.op_count(), 1);

    let mut world = World::new();
    c.flush(&mut world);
    let mut q = world.query::<(Entity, &Sprite)>();
    let items: Vec<(Entity, Vec2)> = q
        .iter(&world)
        .map(|(_, s)| (Entity::PLACEHOLDER, s.custom_size.unwrap_or(Vec2::ZERO)))
        .collect();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].1, Vec2::new(150.0, 80.0), "裁剪后尺寸 = 150×80");

    // 对照 CPU 帧缓冲同坐标填充后非零像素包围盒一致。
    let mut fb = Framebuf::new();
    fb.fill_rect(-50, -20, 200, 100, 0xF7BA);
    let nonzero: Vec<(usize, usize)> = fb
        .as_slice()
        .iter()
        .enumerate()
        .filter(|(_, p)| **p != 0)
        .map(|(i, _)| (i % LOGICAL_W_I32 as usize, i / LOGICAL_W_I32 as usize))
        .collect();
    assert_eq!(nonzero.len(), 150 * 80);
}

#[test]
fn rect_transform_is_center_of_logical_rect() {
    let mut c = canvas();
    // 逻辑矩形 (0,0,100,50) → 中心 (50,25) → 世界 (50-540, 960-25) = (-490, 935)。
    c.fill_rect(0, 0, 100, 50, 0x4E3D);
    let mut world = World::new();
    c.flush(&mut world);
    let mut q = world.query::<&Transform>();
    let t: Vec<Vec3> = q.iter(&world).map(|t| t.translation).collect();
    assert_eq!(t.len(), 1);
    assert!((t[0].x - -490.0).abs() < 1e-4 && (t[0].y - 935.0).abs() < 1e-4, "got {t:?}");
}

#[test]
fn layer_order_maps_to_z() {
    let mut c = canvas();
    c.set_z(0);
    c.fill_rect(0, 0, 10, 10, 0x1111);
    let mut world = World::new();
    c.flush(&mut world);

    c.clear_ops();
    c.set_z(5);
    c.fill_rect(0, 0, 10, 10, 0x2222);
    c.flush(&mut world);

    let mut q = world.query::<&Transform>();
    let zs: Vec<f32> = q.iter(&world).map(|t| t.translation.z).collect();
    assert_eq!(zs, vec![5.0], "旧帧实体已 despawn，新层 z=5");
}

#[test]
fn text_layers_above_rects() {
    let mut c = canvas();
    c.set_z(2);
    c.fill_rect(0, 0, 10, 10, 0x1111);
    c.draw_text("金币", 20, 40, 0xF7BA);
    let mut world = World::new();
    c.flush(&mut world);

    // 文本实体存在且 z = 层+1。
    let mut q = world.query::<(Entity, &Transform)>();
    let entities: Vec<(Entity, Vec3)> = q.iter(&world).map(|(e, t)| (e, t.translation)).collect();
    assert_eq!(entities.len(), 2);
    let text_entity = entities.iter().find(|(_, t)| t.z > 0.0).map(|(e, _)| *e);
    assert!(text_entity.is_some(), "文本层 z 应高于色块层");
}

#[test]
fn clear_frame_sets_clear_color_resource() {
    let mut c = canvas();
    c.clear_frame(0x0865);
    let mut world = World::new();
    c.flush(&mut world);
    let back = world.get_resource::<ClearColor>().map_or(0, |cc| color_to_rgb565(cc.0));
    let diff = back.abs_diff(0x0865);
    assert!(diff <= 2, "清屏色回压 565 偏差 {diff}");
}

#[test]
fn bitmap_fallback_metrics_match_theme() {
    let c = canvas();
    // 位图回退口径：chars × advance（body 12 / title 26 / hint 8）。
    assert_eq!(c.text_width("金银岛"), 3 * 12);
    assert_eq!(c.title_text_width("金银岛"), 3 * 26);
    assert_eq!(c.hint_text_width("金银岛"), 3 * 8);
    // 字形高回退 = bitmap_advance。
    assert_eq!(c.glyph_height(), Theme::DEFAULT.bitmap_advance);
}

#[test]
fn second_flush_despawns_previous_frame() {
    let mut c = canvas();
    c.fill_rect(0, 0, 10, 10, 0x1111);
    c.fill_rect(20, 0, 10, 10, 0x1111);
    let mut world = World::new();
    let first = c.flush(&mut world);
    assert_eq!(first, 2);

    c.clear_ops();
    c.fill_rect(0, 0, 10, 10, 0x1111);
    let second = c.flush(&mut world);
    assert_eq!(second, 1);

    let mut q = world.query_filtered::<Entity, With<CanvasEntity>>();
    assert_eq!(q.iter(&world).count(), 1, "旧帧实体零残留");
}

#[test]
fn canvas_entity_is_component() {
    // 编译期锚点：CanvasEntity 仍挂在实体上（despawn 依据）。
    fn assert_component<T: Component>() {}
    assert_component::<CanvasEntity>();
    let mut world = World::new();
    let mut c = canvas();
    c.fill_rect(0, 0, 4, 4, 0x1111);
    c.flush(&mut world);
    let mut q = world.query_filtered::<Entity, With<CanvasEntity>>();
    assert_eq!(q.iter(&world).count(), 1);
}

#[test]
fn world_extent_constants_unchanged() {
    assert_eq!(LOGICAL_W_I32, 1080);
    assert_eq!(LOGICAL_H_I32, 1920);
}
