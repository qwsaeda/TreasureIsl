//! S18.7：**Bevy 原生渲染**契约测试（通用原语 → Bevy 实体，无帧缓冲/无纹理上传）。
//!
//! 判据：
//! ① `spawn_primitives` 按 `scenes.json` 的 `index.primitives` 生成实体（Sprite/Text2d）；
//! ② 文本是 `Text2d`（GPU 字体通道），不是 CPU 位图；
//! ③ `when` 过滤：选中项的专属元素只在选中时出现（`item` + `when`）；
//! ④ 渲染层只认**通用原语**（`rect`/`text`/`icon`/`grad`/`glow`/`disc`）——场景语义在数据侧；
//! ⑤ 图标 = 烘焙纹理（GPU 采样）。

use bevy::prelude::*;
use bevy::text::TextColor;

use ti_engine::scene_render::{SceneEntities, SelectableKind, spawn_primitives};
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
    // 6 个菜单项 ×（选中卡 + 图标 + 文字）= 13 个可选元素（选中 3 + 未选 5×2）。
    assert_eq!(entities.selectable.len(), 13, "可选元素数（6 菜单项 × 图标/文字 + 选中卡）");
    // 文本走 Text2d（GPU 字体）。
    let text_entity_count = app.world_mut().query::<&Text2d>().iter(app.world()).count();
    assert!(
        text_entity_count >= 8,
        "原生渲染须有 Text2d 实体（标题+副标题+6 菜单文字），实得 {text_entity_count}"
    );
    // 精灵实体（背景/木框/星点/图标等）。
    let sprite_count = app.world_mut().query::<&Sprite>().iter(app.world()).count();
    assert!(sprite_count > 50, "原生渲染须有大量 Sprite 实体，实得 {sprite_count}");
}

#[test]
fn selection_switches_visible_elements() {
    let mut app = app_with_theme();
    let entities = spawn(&mut app, 0);
    // 选中卡（when=selected）仅在选中时存在。
    let cards: Vec<_> =
        entities.selectable.iter().filter(|(_, _, k)| *k == SelectableKind::Card).collect();
    assert_eq!(cards.len(), 1, "选中时仅 1 张选中卡");
    let card_entity = cards[0].1;
    let card_color = app.world().get::<Sprite>(card_entity).expect("卡 Sprite").color;
    assert_eq!(
        card_color,
        ti_engine::canvas::rgb565_to_color(theme().gold_warm),
        "选中卡底 = 数据指定令牌（gold_warm）"
    );
}

#[test]
fn icons_baked_to_gpu_images() {
    let mut app = app_with_theme();
    let entities = spawn(&mut app, 0);
    let icons: Vec<_> =
        entities.selectable.iter().filter(|(_, _, k)| *k == SelectableKind::Icon).collect();
    assert!(!icons.is_empty(), "菜单图标须存在");
    let icon_entity = icons[0].1;
    let sprite = app.world().get::<Sprite>(icon_entity).expect("图标 Sprite");
    assert!(
        sprite.image.id() != bevy::asset::Handle::<Image>::default().id(),
        "图标须有烘焙纹理句柄"
    );
    let images = app.world().resource::<Assets<Image>>();
    let baked = images.get(sprite.image.id()).expect("烘焙纹理在位");
    assert_eq!(baked.width(), 64, "图标烘焙尺寸 = 数据 `params.size`（64）");
}

#[test]
fn text_elements_use_text_color_component() {
    let mut app = app_with_theme();
    let entities = spawn(&mut app, 0);
    let labels: Vec<_> =
        entities.selectable.iter().filter(|(_, _, k)| *k == SelectableKind::Label).collect();
    assert!(!labels.is_empty(), "菜单文字须存在");
    // 未选中项的文字色 = theme 令牌（`text_on_parch`）；选中项 = 黑（数据指定）。
    let has_unselected = labels.iter().any(|(_, e, _)| {
        app.world()
            .get::<TextColor>(*e)
            .is_some_and(|c| c.0 == ti_engine::canvas::rgb565_to_color(theme().text_on_parch))
    });
    assert!(has_unselected, "未选中菜单文字色须 = theme 令牌 text_on_parch");
}
