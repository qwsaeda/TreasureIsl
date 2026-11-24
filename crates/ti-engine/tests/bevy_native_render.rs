//! S18.7：**Bevy 原生渲染**契约测试（通用原语 → Bevy 实体，无帧缓冲/无纹理上传）。
//! S39：index 罗盘环迁 `bevy_ui`（`ui_layer=bevy_ui`，36 原语）⇒ `spawn_primitives`（1080 生产
//! 路径）**跳过** `bevy_ui` 原语（双画/重复计数回归）；本契约测试断言 1080 层只剩非 `bevy_ui`
//! 原语（底图/标题/舵轮/尾行），`bevy_ui` 数据化路径由 `build_data_chrome` 测试覆盖。
//!
//! 判据：
//! ① `spawn_primitives` 按 `scenes.json` 的 `index.primitives` 生成实体（Sprite/Text2d）；
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

fn index_prims() -> Vec<ti_core::data::Primitive> {
    ti_core::data::scenes()
        .expect("scenes.json")
        .get("index")
        .expect("index 场景")
        .primitives
        .clone()
}

fn spawn(app: &mut App, selected: i64) -> SceneEntities {
    let prims = index_prims();
    spawn_primitives(app.world_mut(), &theme(), None, selected, &prims)
}

#[test]
fn generic_primitives_spawn_bevy_entities() {
    let mut app = app_with_theme();
    let entities = spawn(&mut app, 0);
    // S84 纯横屏：index 木牌/图标 18 原语迁 bevy_ui ⇒ `spawn_primitives`（1080 生产路径）
    // 跳过 bevy_ui 原语 ⇒ 1080 层无可选元素（木牌不双画；`ui_layer` 契约见 `scene_render.rs`）。
    assert_eq!(entities.selectable.len(), 0, "S84 index 木牌迁 bevy_ui ⇒ 1080 层 0 可选元素");
    // 文本走 Text2d（GPU 字体）：S84 横屏 index 仅版本签 1 条非 bevy_ui text 原语
    // （标题已 multiply 烘焙进 index_bg_map_h 底图、木牌文案走 bevy_ui）。
    let text_entity_count = app.world_mut().query::<&Text2d>().iter(app.world()).count();
    assert!(
        text_entity_count >= 1,
        "原生渲染须有 ≥1 Text2d 实体（横屏版本签），实得 {text_entity_count}"
    );
}

// S39：罗盘环 1080 层 when 过滤/图标烘焙/文字色契约已随数据化迁 bevy_ui
// （`build_data_chrome` Node 合并，EXP-168）——等价断言由 `landscape_drawlist.rs`
// index 木牌 rect（S39 新增）+ `ui.rs::tier_font_px` 单测覆盖，此处不再重复。
