//! S87.2 世界层 schema 渲染器判据（影子期：默认路径不变，仅本测试直接调用渲染器）。
//!
//! - 数据消费：`world_components.json` 两场景 56 元素全部可解析、不 panic；
//! - 中间层：`z` 升序 / `state` / `visible_when` / `geometry` / `binding` / `action` 解析可观测；
//! - 后端等价：`render_world_components` 经 `RenderHost` → `DrawList` 记录三型 op，
//!   证明不是「编译绿但 0 调用」的假接入；
//! - 影子开关：`env_enabled` 仅 `"1"` 开启，默认关闭 ⇒ 0 行为变化。

use ti_app::game_app::{GameApp, HitData, Scene as GameScene};
use ti_app::scenes::vs_anim_proto_landscape::render_v2_elem_landscape;
use ti_app::scenes::world_schema::{
    DrawKind, DrawRect, ResolvedDrawCmd, TextTier, env_enabled, render_world_components,
    resolve_world_draw_cmds,
};
use ti_app::scenes::{EconomyHost, MapHost, RenderHost, Scene as SceneTrait, SceneHost, VsScene};
use ti_core::data::ui_schema::{BindingSpec, Pick, WorldComponent, asset_roles, world_components};
use ti_core::domain::ui_action::UiAction;
use ti_render::drawlist::DrawOp;

fn app() -> GameApp {
    GameApp::new(1920.0, 1080.0)
}

fn rect(x: i32, y: i32, w: i32, h: i32) -> DrawRect {
    DrawRect { x, y, w, h }
}

fn component(id: &str, kind: &str, z: i32, bounds: DrawRect) -> WorldComponent {
    WorldComponent {
        id: id.to_owned(),
        kind: kind.to_owned(),
        role: String::new(),
        binding: None,
        geometry: None,
        world_rect: Some(ti_core::data::ui_schema::WorldRect {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
        }),
        z,
        pick: None,
        state: Vec::new(),
        tier: String::new(),
        visible_when: None,
        anim_ref: String::new(),
        visible_in_phase: Vec::new(),
    }
}

fn ids(commands: &[ResolvedDrawCmd]) -> Vec<&str> {
    commands.iter().map(|command| command.id.as_str()).collect()
}

/// 两场景 56 元素全部可消费（0 panic）+ 矩形合法 + 空/缺场景回退信号。
#[test]
fn consumes_all_world_components_without_panic() {
    let scenes = world_components().expect("world_components.json 加载");
    assert_eq!(scenes.len(), 2, "世界层场景数漂移");
    assert_eq!(scenes.get("main_map").expect("缺 main_map").elements.len(), 35);
    assert_eq!(scenes.get("vs").expect("缺 vs").elements.len(), 21);
    let roles = asset_roles().expect("asset_roles.json 加载");
    let game = app();
    let theme = game.theme().clone();
    for scene_id in ["main_map", "vs"] {
        let elements = &scenes.get(scene_id).expect("场景在场").elements;
        let commands =
            resolve_world_draw_cmds(elements, roles, &game as &dyn SceneHost, &theme, -1);
        assert!(!commands.is_empty(), "{scene_id} 未产出绘制命令");
        for command in &commands {
            assert!(command.rect.w >= 0 && command.rect.h >= 0, "{} 负尺寸矩形", command.id);
        }
    }
    let mut game = app();
    assert!(!render_world_components(&mut game, "index", -1), "未声明场景须回退旧路径");
    assert!(!render_world_components(&mut game, "no_such_scene", -1), "未知场景须回退旧路径");
}

/// z 升序 + 同 z 稳定（与 `build_component_chrome` 同口径）。
#[test]
fn sorts_by_z_with_stable_ties() {
    let bounds = rect(0, 0, 8, 8);
    let components = vec![
        component("c_high", "Rect", 30, bounds),
        component("a_low", "Rect", 10, bounds),
        component("b_mid", "Rect", 20, bounds),
        component("d_same", "Rect", 20, bounds),
    ];
    let roles = asset_roles().expect("asset_roles.json 加载");
    let game = app();
    let theme = game.theme().clone();
    let commands = resolve_world_draw_cmds(&components, roles, &game as &dyn SceneHost, &theme, -1);
    assert_eq!(ids(&commands), ["a_low", "b_mid", "d_same", "c_high"]);
}

/// geometry 两源解析：`world_rect` 直用 / v6 layout 中心化 / `scenes.json` extra 矩形。
#[test]
fn resolves_geometry_from_both_sources() {
    let scenes = world_components().expect("world_components.json 加载");
    let elements = &scenes.get("main_map").expect("缺 main_map").elements;
    let roles = asset_roles().expect("asset_roles.json 加载");
    let game = app();
    let theme = game.theme().clone();
    let commands = resolve_world_draw_cmds(elements, roles, &game as &dyn SceneHost, &theme, -1);

    let ocean = commands.iter().find(|c| c.id == "ocean_bg").expect("缺 ocean_bg");
    assert_eq!(ocean.rect, rect(0, 0, 1920, 1080), "world_rect 直用漂移");

    let plaque = commands.iter().find(|c| c.id == "hud_plaque_left").expect("缺 plaque");
    assert_eq!(plaque.rect, rect(48, 32, 448, 160), "scenes.json 几何引用漂移");

    let island = commands.iter().find(|c| c.id == "island_home").expect("缺 island_home");
    // v6 layout: island.cx=960 / cy=520 / w=560 ⇒ 中心化矩形。
    assert_eq!(island.rect, rect(680, 240, 560, 560), "v6 layout 几何引用漂移");
}

/// binding：i18n key → `sget`；host path → 白名单访问器 + `sgetf` 格式化。
#[test]
fn resolves_i18n_and_host_bindings() {
    let scenes = world_components().expect("world_components.json 加载");
    let elements = &scenes.get("main_map").expect("缺 main_map").elements;
    let roles = asset_roles().expect("asset_roles.json 加载");
    let game = app();
    let theme = game.theme().clone();
    let commands = resolve_world_draw_cmds(elements, roles, &game as &dyn SceneHost, &theme, -1);

    let title = commands.iter().find(|c| c.id == "title_text").expect("缺 title_text");
    assert_eq!(title.text, ti_app::i18n::sget("scene.main_map.title"));
    assert!(matches!(title.kind, DrawKind::Text(TextTier::Title)));

    let mut game = app();
    game.set_gold(1234);
    let mut gold = component("gold_text", "Text", 1, rect(0, 0, 100, 40));
    gold.binding = Some(BindingSpec::Host {
        kind: "host".to_owned(),
        path: "EconomyHost.gold".to_owned(),
        fmt: "hud.gold_fmt".to_owned(),
    });
    gold.tier = "body".to_owned();
    let theme = game.theme().clone();
    let commands = resolve_world_draw_cmds(&[gold], roles, &game as &dyn SceneHost, &theme, -1);
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].text, "金 1234", "host path + sgetf 格式化漂移");
    assert!(theme.by_name(&commands[0].color_token).is_some(), "文本色必须是 Theme token");
}

/// pick.action → 闭集 opaque key（spin=4 / vs 三变体 11/12/13）。
#[test]
fn derives_action_keys_from_ui_action_table() {
    assert_eq!(UiAction::from_schema_str("spin").map(UiAction::key), Some(4));
    assert_eq!(UiAction::from_schema_str("vs_skip").map(UiAction::key), Some(11));
    assert_eq!(UiAction::from_schema_str("vs_confirm").map(UiAction::key), Some(12));
    assert_eq!(UiAction::from_schema_str("vs_reset").map(UiAction::key), Some(13));

    let mut skip = component("btn_skip", "Image", 1, rect(0, 0, 200, 80));
    skip.role = "btn".to_owned();
    skip.pick = Some(Pick { action: "vs_skip".to_owned(), index: 11 });
    let roles = asset_roles().expect("asset_roles.json 加载");
    let game = app();
    let theme = game.theme().clone();
    let commands = resolve_world_draw_cmds(&[skip], roles, &game as &dyn SceneHost, &theme, -1);
    assert!(commands.iter().any(|c| c.action_key == Some(11)), "vs_skip key 未派生");
}

/// `visible_when` 白名单求值（面板未开 → 过滤；打开 → 出现）。
#[test]
fn filters_by_visible_when() {
    use ti_core::data::ui_schema::VisibleWhen;

    let mut panel = component("panel_build", "Rect", 1, rect(0, 0, 100, 100));
    panel.visible_when = Some(VisibleWhen {
        path: "MapHost.panel".to_owned(),
        op: "eq".to_owned(),
        value: Some("build".to_owned()),
    });
    let roles = asset_roles().expect("asset_roles.json 加载");
    let mut game = app();
    let theme = game.theme().clone();
    assert!(
        resolve_world_draw_cmds(&[panel.clone()], roles, &game as &dyn SceneHost, &theme, -1)
            .is_empty(),
        "面板未开时须过滤"
    );
    game.set_build_panel_open(true);
    assert_eq!(
        resolve_world_draw_cmds(&[panel], roles, &game as &dyn SceneHost, &theme, -1).len(),
        1,
        "面板打开后须出现"
    );
}

/// 后端等价实证：`record_draw=true` 时三型 `DrawOp` 均被记录，
/// 证明所有绘制经 `RenderHost` → `Canvas`（mock/记录宿主可观测）。
#[test]
fn records_all_three_draw_op_kinds_through_render_host() {
    let mut game = app();
    game.set_record_draw(true);
    // 世界层不挂 `chrome_in_bevy_ui` 守卫：Bevy 发货壳置 true 也必须照画。
    game.set_chrome_in_bevy_ui(true);
    // S87.3：handoff/toast/outcome 盖板改为运行时可见（is_some），常态不画 ⇒
    // 显式打开建造面板，让 panel_build（Rect）进入 draw list，保持三型断言有效。
    game.set_build_panel_open(true);
    assert!(render_world_components(&mut game, "main_map", -1));

    let ops = &game.draw_list().ops;
    assert!(ops.iter().any(|op| matches!(op, DrawOp::Rect { .. })), "Rect op 未记录");
    assert!(ops.iter().any(|op| matches!(op, DrawOp::Text { .. })), "Text op 未记录");
    assert!(
        ops.iter().any(
            |op| matches!(op, DrawOp::Image { asset, .. } if asset == "assets/ui/chart_full.png")
        ),
        "Image op（羊皮海图底）未记录"
    );

    let mut game = app();
    game.set_record_draw(true);
    game.set_chrome_in_bevy_ui(true);
    assert!(render_world_components(&mut game, "vs", -1));
    let ops = &game.draw_list().ops;
    assert!(
        ops.iter().any(
            |op| matches!(op, DrawOp::Image { asset, .. } if asset == "assets/ui/vs_bg_m04.png")
        ),
        "vs 背景 Image op 未记录"
    );
    assert!(ops.iter().any(|op| matches!(op, DrawOp::Text { .. })), "vs 文本 op 未记录");
}

/// 影子开关：只有精确 `"1"` 开启；默认关闭 ⇒ 不改默认路径、gate 零空窗。
#[test]
fn shadow_switch_is_explicit() {
    assert!(!env_enabled(None));
    assert!(!env_enabled(Some("")));
    assert!(!env_enabled(Some("0")));
    assert!(!env_enabled(Some("true")));
    assert!(env_enabled(Some("1")));
}

/// S87.3：`main_map` 命中重锚的数据载体——schema `spin_marker` 声明 `pick.action="spin"` → key 4，
/// 与 `scenes.json` 旧 prim `pick_key=4` 双源并存（影子期）；缺 action = 不可点（守门）。
#[test]
fn main_map_spin_marker_anchor() {
    let scenes = world_components().expect("world_components.json 加载");
    let main = scenes.get("main_map").expect("缺 main_map");
    let marker = main.elements.iter().find(|e| e.id == "spin_marker").expect("缺 spin_marker");
    let pick = marker.pick.as_ref().expect("spin_marker 缺 pick（摇骰不可点）");
    assert_eq!(UiAction::from_schema_str(&pick.action).map(UiAction::key), Some(4));
    assert_eq!(pick.index, 4);
    // 中间层确已派生 action_key=4（渲染器真消费，非「数据写了但没人读」）。
    let roles = asset_roles().expect("asset_roles.json 加载");
    let game = app();
    let theme = game.theme().clone();
    let commands =
        resolve_world_draw_cmds(&main.elements, roles, &game as &dyn SceneHost, &theme, -1);
    assert!(
        commands.iter().any(|c| c.id == "spin_marker" && c.action_key == Some(4)),
        "spin_marker 未派生 action_key=4（命中链断裂）"
    );
}

/// S87.3：`main_map` env 分支回退信号——schema 在位 → `true`（走新路径）；
/// 缺场景/解析失败 → `false`（`render()` 的 `&&` 短路回退旧过程式，画面非黑）。
#[test]
fn main_map_branch_fallback_signal() {
    let mut game = app();
    assert!(render_world_components(&mut game, "main_map", -1), "main_map schema 须可渲染");
    assert!(!render_world_components(&mut game, "no_such_scene", -1), "缺场景须回退旧路径");
}

/// S87.4：`vs` env 分支回退信号（与 `main_map` 同构）——schema 在位 → `true`（静态层可画）；
/// 缺场景 / 解析失败 → `false`（`render()` 的 `enabled() && ...` 短路回退旧过程式，画面非黑）。
#[test]
fn vs_branch_fallback_signal() {
    let mut game = app();
    assert!(render_world_components(&mut game, "vs", -1), "vs schema 须可渲染");
    assert!(!render_world_components(&mut game, "no_such_scene", -1), "缺场景须回退旧路径");
}

/// S87.4：vs schema 按钮 action key（11/12/13）与旧裸 key 2/3/4 **同语义**实证。
///
/// - `from_key` 闭集反解唯一；旧 key 2/3/4 的代表元是 `CharacterSelect`/`SlotLoad`/`Spin`，
///   已被 `on_hit` 显式匹配截住，不会误入新分支（向后兼容）。
/// - key 12 = `vs_confirm` → `Back` → 离开 Vs 回 Index（可观测的结果）。
/// - key 11/13 = `vs_skip`/`vs_reset` → 只改 `vs_anim_proto` 全局演出时钟
///   （不 advance、不换场景），断言「无命中派发」防误走 Attack/Back。
#[test]
fn vs_action_keys_route_to_legacy_semantics() {
    assert_eq!(UiAction::from_key(11), Some(UiAction::VsSkip));
    assert_eq!(UiAction::from_key(12), Some(UiAction::VsConfirm));
    assert_eq!(UiAction::from_key(13), Some(UiAction::VsReset));
    assert_ne!(UiAction::from_key(2), Some(UiAction::VsSkip), "旧裸 key 2 代表元须不同");

    let hit = |key: u32| HitData { key, index: 0, swipe: 0 };

    // key 12 = vs_confirm → Back → Index（旧 key 3 语义）。
    let mut game = app();
    ti_app::forward_nav::navigate(&mut game, "vs");
    assert_eq!(game.scene(), GameScene::Vs);
    assert!(VsScene.on_hit(&mut game, &hit(12)), "vs_confirm 须消费（Back 语义）");
    assert_eq!(game.scene(), GameScene::Index, "vs_confirm 须回主菜单（旧 key 3 语义）");

    // key 11 / 13 = vs_skip / vs_reset → 演出时钟，不 advance、不换场景。
    for key in [11, 13] {
        let mut game = app();
        ti_app::forward_nav::navigate(&mut game, "vs");
        let before = game.scene();
        assert!(!VsScene.on_hit(&mut game, &hit(key)), "vs_skip/vs_reset 不消费输入");
        assert_eq!(game.scene(), before, "vs_skip/vs_reset 不得切场景");
    }
}

/// S87.5 静态/动态分层判据的录图助手：`record_draw=true` 的 `GameApp` 作为 `SceneHost`，
/// 直接调横屏动态层（显式 flag，避开进程级 env 的并行测试竞态）。
fn dyn_game() -> GameApp {
    let mut game = app();
    game.set_record_draw(true);
    game
}

/// 贴图命中（动画帧的 x/y 带 `tri()` 微位移 ⇒ 按 asset + 尺寸 + 可选轴匹配）。
fn has_image_where(
    ops: &[DrawOp],
    asset_part: &str,
    x: Option<i32>,
    y: Option<i32>,
    w: i32,
    h: i32,
) -> bool {
    ops.iter().any(|op| {
        matches!(op, DrawOp::Image { asset, x: ix, y: iy, w: iw, h: ih, .. }
            if asset.contains(asset_part)
                && *iw == w
                && *ih == h
                && x.is_none_or(|v| *ix == v)
                && y.is_none_or(|v| *iy == v))
    })
}

fn has_rect(ops: &[DrawOp], x: i32, y: i32, w: i32, h: i32) -> bool {
    ops.iter().any(|op| {
        matches!(op, DrawOp::Rect { x: ix, y: iy, w: iw, h: ih, .. }
            if (*ix, *iy, *iw, *ih) == (x, y, w, h))
    })
}

fn has_hint_text(ops: &[DrawOp], part: &str) -> bool {
    ops.iter().any(|op| {
        matches!(op, DrawOp::Text { text, tier: ti_render::drawlist::Tier::Hint, .. }
            if text.contains(part))
    })
}

/// 旧路径（`static_layer_from_schema=false`）：满屏背景 / 4 层边框 / HP 框底 / VS 徽标底
/// 全部照旧产出，0 行为变化（与 S87.4 及改前逐像素一致）。
#[test]
fn vs_dynamic_layer_legacy_draws_all_static_bases() {
    let mut game = dyn_game();
    render_v2_elem_landscape(&mut game, "clash", 40, 30, 35, 25, false, "raid", false);
    let ops = &game.draw_list().ops;
    assert!(
        has_image_where(ops, "vs_v2/bg_", Some(44), Some(44), 1832, 992),
        "旧路径缺满屏背景 blit"
    );
    assert!(has_rect(ops, 64, 40, 880, 110), "旧路径缺攻方 HP 框底");
    assert!(has_rect(ops, 976, 40, 880, 110), "旧路径缺守方 HP 框底");
    assert!(has_rect(ops, 890, 470, 140, 140), "旧路径缺 VS 徽标底");
    assert!(has_hint_text(ops, "VS"), "旧路径缺 VS 字样");
    // 动态数值/时序同样在（两侧兵力数字 + 角色帧）。
    assert!(has_hint_text(ops, "人手 40 / 武器 30 / 武装 30"), "旧路径缺攻方兵力数字");
    assert!(
        has_image_where(ops, "vs_v2/hero.png", Some(360), None, 480, 680),
        "旧路径缺 hero 时序帧"
    );
    assert!(
        has_image_where(ops, "vs_v2/enemy.png", Some(1080), None, 480, 680),
        "旧路径缺 enemy 时序帧"
    );
}

/// schema 模式（`static_layer_from_schema=true`）：静态框/底全部跳过（由
/// `world_components.json` 提供），动态层只剩纯数值/时序——证明「可视化编辑真正生效」
/// 的结构前提：动态层不再重复占据静态元素的位置。
#[test]
fn vs_dynamic_layer_schema_mode_draws_only_numeric_and_timing() {
    let mut game = dyn_game();
    render_v2_elem_landscape(&mut game, "clash", 40, 30, 35, 25, false, "raid", true);
    let ops = &game.draw_list().ops;
    // 静态底必须缺席（都由 schema 静态层画在其下）。
    assert!(
        !has_image_where(ops, "vs_v2/bg_", Some(44), Some(44), 1832, 992),
        "schema 模式仍在重复画背景"
    );
    assert!(!has_rect(ops, 64, 40, 880, 110), "schema 模式仍在重复画 HP 框底");
    assert!(!has_rect(ops, 976, 40, 880, 110), "schema 模式仍在重复画 HP 框底");
    assert!(!has_rect(ops, 890, 470, 140, 140), "schema 模式仍在重复画 VS 徽标底");
    assert!(!has_hint_text(ops, "VS"), "schema 模式仍在重复画 VS 字样（挡住 schema logo）");
    // 动态数值/时序必须完整保留、画在静态层之上。
    assert!(has_hint_text(ops, "人手 40 / 武器 30 / 武装 30"), "schema 模式丢攻方兵力数字");
    assert!(has_hint_text(ops, "人手 35 / 武器 25 / 武装 25"), "schema 模式丢守方兵力数字");
    assert!(
        has_image_where(ops, "vs_v2/hero.png", Some(360), None, 480, 680),
        "schema 模式丢 hero 时序帧"
    );
    assert!(
        has_image_where(ops, "vs_v2/enemy.png", Some(1080), None, 480, 680),
        "schema 模式丢 enemy 时序帧"
    );
    assert!(
        has_image_where(ops, "vs_v2/crew.png", None, Some(560), 280, 240),
        "schema 模式丢 crew 时序帧"
    );
    // HP 填充条（比例）仍在：bw=844 → (bw-8)=836，满兵力 40、crew=40 ⇒ fw=836，落 (86,114,836,14)。
    assert!(has_rect(ops, 86, 114, 836, 14), "schema 模式丢 HP 填充条（动态数值）");
}

/// result 帧同样分层：横幅底图 / 战报框底由 schema 提供，角色时序 + 战报数值文本保留。
#[test]
fn vs_result_frame_static_bases_skipped_in_schema_mode() {
    let mut game = dyn_game();
    render_v2_elem_landscape(&mut game, "result_win", 40, 30, 35, 25, true, "boss", true);
    let ops = &game.draw_list().ops;
    assert!(!has_rect(ops, 240, 720, 1440, 300), "schema 模式仍在重复画战报框底");
    assert!(
        !has_image_where(ops, "vs_v2/banner_win.png", Some(710), Some(80), 500, 300),
        "schema 模式仍在重复画横幅底"
    );
    assert!(
        ops.iter().any(|op| matches!(op, DrawOp::Text { text, .. }
            if text.contains("战报") || text.contains("我方战前"))),
        "schema 模式丢战报数值文本"
    );
    assert!(
        has_image_where(ops, "vs_v2/hero.png", Some(760), None, 400, 560),
        "schema 模式丢 result 角色时序帧"
    );
}

/// 数据面对账：schema vs 静态元素确实覆盖了动态层跳过的全部静态底（避免「两层都跳」黑洞）。
#[test]
fn vs_schema_static_elements_cover_skipped_bases() {
    let scenes = world_components().expect("world_components.json 加载");
    let elements = &scenes.get("vs").expect("缺 vs").elements;
    let has = |id: &str, rect: (i32, i32, i32, i32)| {
        elements
            .iter()
            .any(|e| e.id == id && e.world_rect.map(|r| (r.x, r.y, r.w, r.h)) == Some(rect))
    };
    assert!(has("bg_frame", (44, 44, 1832, 992)), "schema 缺背景底");
    assert!(has("hp_player_frame", (64, 40, 880, 110)), "schema 缺攻方 HP 框底");
    assert!(has("hp_enemy_frame", (976, 40, 880, 110)), "schema 缺守方 HP 框底");
    assert!(has("vs_logo", (890, 470, 140, 140)), "schema 缺 VS 徽标底");
    assert!(has("report_frame", (240, 720, 1440, 300)), "schema 缺战报框底");
    assert!(has("banner", (710, 80, 500, 300)), "schema 缺横幅底");
    for id in ["btn_skip", "btn_confirm", "btn_reset"] {
        assert!(elements.iter().any(|e| e.id == id), "schema 缺按钮 {id}（出场/交互不可编辑）");
    }
}
