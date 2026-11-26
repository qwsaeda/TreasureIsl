//! S38 方案 A：横屏世界层 draw list 验证（无头，不依赖 GPU readback）。
//!
//! S85.5：5 纯 UI 场景 chrome 已从 `scenes.json` 死原语迁到 `components_v1.json` schema
//! （CPU 路径不再绘制这些场景），故其断言改为**schema 结构 + `asset_roles` 解析**：
//! 背景/立绘 role 须解析到改前同名资产，列表/面板 action 与 binding 声明须在位。
//! `vs`/`main_map` = 游戏世界，仍走 1080 draw list（原语未迁），判据不动。
//!
//! 判据：
//! - 5 纯 UI 场景：schema 声明齐 + role→资产解析 + action/binding 契约；
//! - `vs` / `main_map`：draw list Image/Rect op 矩形 = 横屏坐标（游戏世界不迁）。

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use ti_app::game_app::GameApp;
use ti_platform::settings::Orientation;
use ti_render::drawlist::DrawOp;

fn game() -> GameApp {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.set_record_draw(true);
    app.shell_mut().set_orientation(Orientation::Landscape);
    app
}

/// 收集 draw list 中的 `Image` op 目标矩形（asset, x, y, w, h）。
fn image_rects(app: &GameApp) -> Vec<(String, i32, i32, i32, i32)> {
    app.draw_list()
        .ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::Image { asset, x, y, w, h, .. } => Some((asset.clone(), *x, *y, *w, *h)),
            _ => None,
        })
        .collect()
}

/// 收集 draw list 中的 `Rect` op（x, y, w, h）。
fn rect_rects(app: &GameApp) -> Vec<(i32, i32, i32, i32)> {
    app.draw_list()
        .ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::Rect { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .collect()
}

/// schema 组件 + role→资产解析（S85.5 纯 UI 场景断言入口）。
fn scene_schema(id: &str) -> ti_core::data::ui_schema::SceneComponents {
    ti_core::data::ui_schema::components()
        .expect("components_v1.json 加载")
        .get(id)
        .expect("components_v1.json 缺场景")
        .clone()
}

fn role_path(role: &str) -> String {
    ti_core::data::ui_schema::asset_roles()
        .expect("asset_roles.json 加载")
        .path(role)
        .expect("asset_roles.json 缺 role")
        .to_owned()
}

/// S39/S85.5：`index` 组件声明（羊皮藏宝图底 + 人物互动层 + 6 木牌菜单 + 尾行）。
#[test]
fn index_landscape_world_layer() {
    let sc = scene_schema("index");
    assert_eq!(role_path("index_bg"), "assets/ui/index_bg_map_h.png", "羊皮底图 role 漂移");
    assert_eq!(role_path("index_hero"), "assets/ui/index_hero_duo.png", "人物互动层 role 漂移");
    let menu = sc.elements.iter().find(|e| e.id == "menu").expect("index 缺 menu 列表组件");
    assert_eq!(menu.action, "menu_select", "首页 6 木牌须声明 menu_select action");
    assert!(menu.binding == "menu", "首页菜单 binding 漂移");
    let bg = sc.elements.iter().find(|e| e.id == "bg").expect("index 缺 bg 组件");
    assert_eq!(bg.layout.intent, "fill", "满屏底图须声明 fill 意图");
    // S81：木牌运行期几何参数（extra_scalar）仍在 scenes.json 数据面，保留防漂移。
    let scenes = ti_core::data::scenes().expect("scenes.json 加载");
    let index = scenes.get("index").expect("scenes.json 缺 index");
    assert!(
        index.extra_scalar.get("menu.x") == Some(&1446)
            && index.extra_scalar.get("menu.plank_rot_dx") == Some(&-2),
        "index 横屏 menu 参数缺失/漂移: {:?}",
        index.extra_scalar
    );
}

/// S85.5：`select_role` 组件声明（夜空底 + 角色列表 + 标题/导航文本）。
#[test]
fn select_role_landscape_world_layer() {
    let sc = scene_schema("select_role");
    assert_eq!(role_path("bg_night_sea"), "assets/ui/bg_night_sea.png", "夜空底 role 漂移");
    let roster = sc.elements.iter().find(|e| e.id == "roster").expect("缺 roster 列表组件");
    assert_eq!(roster.action, "character_select", "选角列表 action 漂移");
    assert_eq!(roster.binding, "characters", "选角 binding 漂移");
}

/// S85.5：`select_map` 组件声明（夜空底 + 地图列表 + 标题/导航文本）。
#[test]
fn select_map_landscape_world_layer() {
    let sc = scene_schema("select_map");
    let maps = sc.elements.iter().find(|e| e.id == "maps").expect("缺 maps 列表组件");
    assert_eq!(maps.action, "map_select", "选图列表 action 漂移");
    assert_eq!(maps.binding, "map_slots", "选图 binding 漂移");
}

/// S85.5：`message` 组件声明（夜空底 + 消息面板 + 导航文本）。
#[test]
fn message_landscape_world_layer() {
    let sc = scene_schema("message");
    let panel = sc.elements.iter().find(|e| e.id == "panel").expect("缺 panel 面板组件");
    assert_eq!(panel.binding, "msg_pages", "消息面板 binding 漂移");
    assert_eq!(panel.kind, "panel", "消息面板 type 漂移");
}

/// S85.5：`saveload` 组件声明（夜空底 + 槽位列表 + 标题/导航文本）。
#[test]
fn saveload_landscape_world_layer() {
    let sc = scene_schema("saveload");
    let slots = sc.elements.iter().find(|e| e.id == "slots").expect("缺 slots 列表组件");
    assert_eq!(slots.action, "slot_write", "槽位列表 action 漂移");
    assert_eq!(slots.binding, "save_slots", "存读档 binding 漂移");
}

/// S38：`vs` 横屏正式战斗（`render_v2_elem_landscape`：bg 框内 + 顶双 HP 框 + 角色对角）。
///
/// S87.4 双路径兼容：`TI_WORLD_SCHEMA=1` 时 `vs.rs::render` 先画 schema 静态层，随后
/// `render_v2_elem` 动态层后画覆盖其上；默认（env 未设置）走旧过程式。本测试的四项
/// 动态断言（bg / crew / hero / enemy，S34.6 起由 `render_v2_elem` 产出）在**默认路径**
/// 全在；S87.5 起 schema 模式动态层只画数值/时序（bg / 框底 / VS 徽标由 schema 提供），
/// 故这些断言锚定 env=0 的旧路径（0 行为变化），schema 模式拆分由 `world_schema_parity` 守。
#[test]
fn vs_landscape_world_layer() {
    // navigate("vs") 有 battle_snapshot → render_v2_elem_landscape（phase=clash）
    let mut app = game();
    ti_app::forward_nav::navigate(&mut app, "vs");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| {
            // S87.5：schema 模式 bg role=vs_bg → `vs_bg_m<N>.png`；旧路径 → `vs_v2/bg_*.png`。
            (a.contains("vs_v2/bg_") || a.contains("vs_bg_"))
                && *x == 44
                && *y == 44
                && *w == 1832
                && *h == 992
        }),
        "vs 横屏 bg 应在羊皮框内 (44,44,1832,992)，got {imgs:?}"
    );
    // S63 候选 A：clash 帧 = hero 左中 (360,280) 480×680 + enemy 右中 (1080,280) + crew 居中前景 (820,560)
    assert!(
        imgs.iter().any(|(a, x, y, ..)| a.ends_with("crew.png") && *x == 820 && *y == 560),
        "vs 横屏 clash crew 应在 (820,560，S63 候选A居中前景)，got {imgs:?}"
    );
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| a.ends_with("hero.png")
            && *x == 360
            && *y == 280
            && *w == 480
            && *h == 680),
        "vs 横屏 clash hero 应在 (360,280,480,680，S63 候选A左中)，got {imgs:?}"
    );
    assert!(
        imgs.iter().any(|(a, x, y, w, h)| a.ends_with("enemy.png")
            && *x == 1080
            && *y == 280
            && *w == 480
            && *h == 680),
        "vs 横屏 clash enemy 应在 (1080,280,480,680，S63 候选A右中)，got {imgs:?}"
    );
    let rects = rect_rects(&app);
    // 顶双 HP 状态框（左攻 64,40 / 右守 976,40，880×110）
    // S87.5：schema 模式框底 = hud_plaque.png image（role=hud），旧路径 = fill_rect；
    // 两种模式都不该在 HP 框位置留洞 ⇒ imgs / rects 任一命中即通过。
    assert!(
        imgs.iter().any(|(_, x, y, w, h)| *x == 64 && *y == 40 && *w == 880 && *h == 110)
            || rects.iter().any(|(x, y, w, h)| *x == 64 && *y == 40 && *w == 880 && *h == 110),
        "vs 横屏 attacker HP 框应在 (64,40,880,110)，got imgs={imgs:?} rects={rects:?}"
    );
    assert!(
        imgs.iter().any(|(_, x, y, w, h)| *x == 976 && *y == 40 && *w == 880 && *h == 110)
            || rects.iter().any(|(x, y, w, h)| *x == 976 && *y == 40 && *w == 880 && *h == 110),
        "vs 横屏 defender HP 框应在 (976,40,880,110)，got imgs={imgs:?} rects={rects:?}"
    );
}

/// S87.5 接管对账：schema 静态底矩形必须与默认动态层画出的静态底**逐一对齐**，
/// 否则 schema 模式跳过动态底后会留黑洞（可视化编辑生效的结构前提）。
#[test]
fn vs_schema_static_bases_align_with_dynamic_drawlist() {
    let mut app = game();
    ti_app::forward_nav::navigate(&mut app, "vs");
    app.render();
    let imgs = image_rects(&app);
    let rects = rect_rects(&app);
    let scenes = ti_core::data::ui_schema::world_components().expect("world_components.json 加载");
    let vs = scenes.get("vs").expect("缺 vs");
    for (id, want) in [
        ("bg_frame", (44, 44, 1832, 992)),
        ("hp_player_frame", (64, 40, 880, 110)),
        ("hp_enemy_frame", (976, 40, 880, 110)),
    ] {
        let element = vs.elements.iter().find(|e| e.id == id).expect("缺 schema 静态元素");
        let rect = element.world_rect.expect("{id} 缺 world_rect");
        assert_eq!((rect.x, rect.y, rect.w, rect.h), want, "{id} schema 矩形漂移");
        let in_dynamic = imgs.iter().any(|(_, x, y, w, h)| (*x, *y, *w, *h) == want)
            || rects.iter().any(|(x, y, w, h)| (*x, *y, *w, *h) == want);
        assert!(in_dynamic, "{id} schema 矩形 ({want:?}) 无对应动态底 op（两层不对齐）");
    }
    let vs_logo = vs.elements.iter().find(|e| e.id == "vs_logo").expect("缺 vs_logo");
    let lr = vs_logo.world_rect.expect("vs_logo 缺 world_rect");
    assert_eq!((lr.x, lr.y, lr.w, lr.h), (890, 470, 140, 140), "vs_logo schema 矩形漂移");
    // S87.5：schema 模式把 vs_logo 画成 image（role=vs_logo → icon_monogram.png），
    // 旧路径画成 fill_rect；两种模式都不该在该矩形留黑洞 ⇒ 两者任一命中即通过。
    assert!(
        imgs.iter().any(|(_, x, y, w, h)| (*x, *y, *w, *h) == (890, 470, 140, 140))
            || rects.iter().any(|(x, y, w, h)| (*x, *y, *w, *h) == (890, 470, 140, 140)),
        "vs_logo schema 矩形无对应徽标底 op（image 或 rect，动态层跳过后会留黑洞）"
    );
}

/// S38：`main_map` v6 横屏（`render_v6_elem` 海图满屏 1920×1080，椭圆格环）。
/// 注：v6 默认路径先画世界层（`V6_OCEAN_BG` 满屏），故断言满屏 `image` 而非 `chart_full` 原语（z=2 <3）。
#[test]
fn main_map_v6_landscape_world_layer() {
    // v6 海图满屏 1920×1080（render_v6_elem V6_OCEAN_BG）
    let mut app = game();
    ti_app::forward_nav::navigate(&mut app, "main_map");
    app.render();
    let imgs = image_rects(&app);
    assert!(
        imgs.iter().any(|(_, x, y, w, h)| *x == 0 && *y == 0 && *w == 1920 && *h == 1080),
        "main_map 横屏 v6 海图应满屏 1920×1080，got {imgs:?}"
    );
}
