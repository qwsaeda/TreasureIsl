//! S18.7：**Bevy 原生渲染**契约测试（通用原语 → Bevy 实体，无帧缓冲/无纹理上传）。
//! S22：index = 罗盘环标题页（medallion/底图 = `image` 原语走生产 drawlist；本契约测试
//! 只经 `spawn_primitives` spawn rect/icon/text，故 medallion 不在此计数）。
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
    // S22 罗盘环 index：medallion/底图/舵轮 = `image` 原语（spawn_primitives 走 catch-all 不 spawn，
    // 走 drawlist 生产路径）；本契约测试只 spawn rect/icon/text → 选中 0 时可见 =
    // 6 图标（5 常态 item1-5 + 1 选中 item0）+ 6 标签（同 when 过滤）= 12 个可选元素（无 Card）。
    assert_eq!(entities.selectable.len(), 12, "S22 罗盘环 index 可选元素 = 6 图标 + 6 标签");
    // 文本走 Text2d（GPU 字体）：6 标签 + 标题/副标题/尾行 = 9。
    let text_entity_count = app.world_mut().query::<&Text2d>().iter(app.world()).count();
    assert!(
        text_entity_count >= 6,
        "原生渲染须有 Text2d 实体（标题+副标题+标签+尾行），实得 {text_entity_count}"
    );
    // 精灵实体（图标烘焙等）。
    let sprite_count = app.world_mut().query::<&Sprite>().iter(app.world()).count();
    assert!(sprite_count >= 6, "原生渲染须有图标 Sprite 实体（≥6），实得 {sprite_count}");
}

#[test]
fn selection_switches_visible_elements() {
    let mut app = app_with_theme();
    let entities = spawn(&mut app, 0);
    // S22 罗盘环：无「选中卡」（medallion = image 原语，走生产 drawlist，不在此契约 spawn）。
    // 选中态 = 该徽章**标签变金（gold_warm）**；未选中徽章标签 = 常态（`text_on_parch`，
    // 数据 = scenes.json index 段 item1..5 的 text 原语 `when=normal` color）。
    let labels: Vec<_> =
        entities.selectable.iter().filter(|(_, _, k)| *k == SelectableKind::Label).collect();
    let gold_warm = ti_engine::canvas::rgb565_to_color(theme().gold_warm);
    let text_on_parch = ti_engine::canvas::rgb565_to_color(theme().text_on_parch);
    let label_color = |e: Entity| app.world().get::<TextColor>(e).map(|c| c.0);
    let selected_gold =
        labels.iter().filter(|(_, e, _)| label_color(*e) == Some(gold_warm)).count();
    let normal_parch =
        labels.iter().filter(|(_, e, _)| label_color(*e) == Some(text_on_parch)).count();
    assert_eq!(selected_gold, 1, "选中徽章 0 标签 = gold_warm（1 个）");
    assert_eq!(normal_parch, 5, "未选中徽章 1-5 标签 = text_on_parch（5 个）");
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
    assert_eq!(baked.width(), 96, "图标烘焙尺寸 = 数据 `params.size`（96，S21.14 白蒙版）");
}

#[test]
fn text_elements_use_text_color_component() {
    let mut app = app_with_theme();
    let entities = spawn(&mut app, 0);
    let labels: Vec<_> =
        entities.selectable.iter().filter(|(_, _, k)| *k == SelectableKind::Label).collect();
    assert!(!labels.is_empty(), "菜单文字须存在");
    // S22 罗盘环：未选中徽章标签色 = theme 令牌（`text_on_parch`）（scenes.json index 段
    // item1..5 text 原语 `when=normal` color；选中 = `gold_warm`）。
    let has_unselected = labels.iter().any(|(_, e, _)| {
        app.world()
            .get::<TextColor>(*e)
            .is_some_and(|c| c.0 == ti_engine::canvas::rgb565_to_color(theme().text_on_parch))
    });
    assert!(has_unselected, "未选中徽章标签色须 = theme 令牌 text_on_parch");
}
