//! S38 方案 A：`spawn_draw_list` 横屏方向感知（无头，不依赖 GPU）。
//!
//! 横屏 draw list op（1920×1080 坐标）经 `spawn_draw_list` → `to_world_dims` 定位 Bevy 精灵，
//! 断言世界坐标以横屏逻辑画布中心 (960,540) 为原点对折（竖屏路径 = 1080×1920 原点 540,960）。

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use bevy::prelude::*;
use ti_engine::scene_render::{DrawListEntity, spawn_draw_list};
use ti_render::drawlist::{DrawList, DrawOp, Tier};
use ti_render::theme::Theme;

/// 横屏：矩形 (100, 50, 200, 100) 中心 (200,100) → 世界 (200-960, 540-100) = (-760, 440)。
#[test]
fn spawn_draw_list_landscape_centers_on_landscape_extent() {
    let mut world = World::new();
    let mut list = DrawList { theme: Theme::DEFAULT.clone(), ..Default::default() };
    list.ops.push(DrawOp::Rect { x: 100, y: 50, w: 200, h: 100, color: 0xF800 });
    spawn_draw_list(&mut world, None, &list, true);

    let mut q = world.query_filtered::<(&Transform,), With<DrawListEntity>>();
    let (t,) = q.single(&world).expect("spawn 实体应存在");
    // 中心 (200,100) → x = 200-960 = -760；y = 540-100 = 440。
    assert!((t.translation.x - (-760.0)).abs() < 0.01, "x 应 -760, got {}", t.translation.x);
    assert!((t.translation.y - 440.0).abs() < 0.01, "y 应 440, got {}", t.translation.y);
}

/// 竖屏（0 行为变化）：矩形 (100, 50, 200, 100) 中心 (200,100) → 世界 (200-540, 960-100) = (-340, 860)。
#[test]
fn spawn_draw_list_portrait_unchanged() {
    let mut world = World::new();
    let mut list = DrawList { theme: Theme::DEFAULT.clone(), ..Default::default() };
    list.ops.push(DrawOp::Rect { x: 100, y: 50, w: 200, h: 100, color: 0xF800 });
    spawn_draw_list(&mut world, None, &list, false);

    let mut q = world.query_filtered::<(&Transform,), With<DrawListEntity>>();
    let (t,) = q.single(&world).expect("spawn 实体应存在");
    assert!((t.translation.x - (-340.0)).abs() < 0.01, "x 应 -340, got {}", t.translation.x);
    assert!((t.translation.y - 860.0).abs() < 0.01, "y 应 860, got {}", t.translation.y);
}

/// 横屏文本定位（Text2d 实体）同方向感知。
#[test]
fn spawn_draw_list_landscape_text_centers() {
    let mut world = World::new();
    let mut list = DrawList { theme: Theme::DEFAULT.clone(), ..Default::default() };
    list.ops.push(DrawOp::Text {
        text: "标题".to_owned(),
        x: 1000,
        y: 500,
        tier: Tier::Title,
        color: 0xFFFF,
    });
    spawn_draw_list(&mut world, None, &list, true);

    let mut q = world.query_filtered::<(&Transform,), With<DrawListEntity>>();
    let (t,) = q.single(&world).expect("spawn 实体应存在");
    // 文本以中心换算（左上是 (1000,500)，中心 = 左上 + 宽高半量 ⇒ x = (1000+width/2)-960 > 40）。
    // 仅断言横屏方向感知（x=1000 在横屏屏心 960 右侧 ⇒ 世界 x > 0）；精确宽高随字体度量，
    // 由 `spawn_draw_list_portrait_unchanged`/`_landscape_centers_on_landscape_extent` 锁定换算。
    assert!(t.translation.x > 0.0, "横屏文本 x=1000 应在屏心右侧，got {}", t.translation.x);
}
