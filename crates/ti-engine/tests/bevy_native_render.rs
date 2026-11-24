//! S18.7：**Bevy 原生渲染**契约测试（通用原语 → Bevy 实体，无帧缓冲/无纹理上传）。
//! S85.5：5 纯 UI 场景 `scenes.json` 死原语已删，chrome 全走 `components_v1.json` schema；
//! 本契约测试改用**合成原语夹具**验证 `spawn_primitives` 的 1080 世界层行为
//! （与场景数据解耦，schema 渲染由 `build_component_chrome` / `--shot` 覆盖）。
//!
//! 判据：
//! ① `spawn_primitives` 按合成原语生成实体（Sprite/Text2d）；
//! ② 文本是 `Text2d`（GPU 字体通道），不是 CPU 位图；
//! ③ `when` 过滤：选中项的专属元素只在选中时出现（`item` + `when`）；
//! ④ 渲染层只认**通用原语**（`rect`/`text`/`icon`/`grad`/`glow`/`disc`）——场景语义在数据侧；
//! ⑤ 图标 = 烘焙纹理（GPU 采样）。

use bevy::prelude::*;
use bevy::text::TextColor;

use ti_engine::scene_render::{SceneEntities, spawn_primitives};
use ti_render::theme::Theme;

/// headless App（**不建窗口 / 不启渲染 app**）：只注册资产与组件类型，
/// 供 `spawn_primitives` 直接往 World 里 spawn 实体（契约测试不需真渲染）。
fn app_with_theme() -> App {
    let mut app = App::new();
    app.insert_resource(Assets::<Image>::default());
    {
        let world = app.world_mut();
        world.register_component::<Sprite>();
        world.register_component::<Transform>();
        world.register_component::<Text2d>();
        world.register_component::<TextColor>();
        world.register_component::<bevy::text::TextFont>();
    }
    app
}

fn theme() -> Theme {
    Theme::default()
}

/// 合成原语夹具（与已删除的 `scenes.json` 死原语解耦，守 1080 层 spawn 契约本身）。
fn fixture_prims() -> Vec<ti_core::data::Primitive> {
    vec![
        ti_core::data::Primitive {
            kind: "text".to_owned(),
            rect: ti_core::data::Rect { x: 0, y: 0, w: 320, h: 48 },
            text: "原生文本".to_owned(),
            item: -1,
            ..Default::default()
        },
        ti_core::data::Primitive {
            kind: "rect".to_owned(),
            rect: ti_core::data::Rect { x: 0, y: 0, w: 200, h: 100 },
            color: "panel_wood".to_owned(),
            item: -1,
            ..Default::default()
        },
    ]
}

fn spawn(app: &mut App, selected: i64) -> SceneEntities {
    let prims = fixture_prims();
    spawn_primitives(app.world_mut(), &theme(), None, selected, &prims)
}

#[test]
fn generic_primitives_spawn_bevy_entities() {
    let mut app = app_with_theme();
    let entities = spawn(&mut app, 0);
    // 夹具无 `pick_key` 原语 ⇒ 1080 层 0 可选元素（`pick_key` 契约见 `scene_render.rs`）。
    assert_eq!(entities.selectable.len(), 0, "无 pick_key 原语 ⇒ 1080 层 0 可选元素");
    // 文本走 Text2d（GPU 字体）：夹具含 1 条 text 原语。
    let text_entity_count = app.world_mut().query::<&Text2d>().iter(app.world()).count();
    assert!(
        text_entity_count >= 1,
        "原生渲染须有 ≥1 Text2d 实体（夹具文本），实得 {text_entity_count}"
    );
}

// S85.5：场景 chrome（when 过滤/图标烘焙/文字色）已全迁 `components_v1.json` schema，
// 等价断言由 `ui.rs::tier_font_px` 单测 + `--shot` 非黑探针 + doccheck2 覆盖，此处不重复。
