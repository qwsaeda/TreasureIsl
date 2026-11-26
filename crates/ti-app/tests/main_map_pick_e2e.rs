//! S27.5 主地图命中回归（用户 2026-10-28 报「没有筛子，玩不了」）。
//!
//! 根因：`scenes.json` `main_map` 场景**零** `pick_key` 原语 ⇒ 摇骰 CTA / 贸易 /
//! 出售 / 建造 四键全不可点（B1 拾取层 `spawn_pick_layer` 只认 `pick_key>0` 原语）。
//! 修复 = 数据面补 `pick_key`（4/5/6/7）+ 轴心补骰点贴图。
//!
//! S34 用户 2026-11-01「底部三键取掉」：5/6/7（贸易/出售/建造）随三键原语删除，
//! 命中层只余 4（摇骰 CTA）；判据 ①/② 相应收窄。
//!
//! S87.3 世界层影子期：`pick_key=4` 的载体双源并存——旧 `scenes.json` prim（`manual_1080_hit_system`
//! 读它）与 `world_components.json::main_map::spin_marker.pick.action="spin"`（`UiAction` 派生 key=4）。
//! 判据改为「至少一处提供 spin=4」，不硬绑 `scenes.json`，避免 S87.6 清理旧 prim 时回归；
//! 「缺键 = 不可点」守门（UiAction 闭集登记 + key=4 命中真摇骰）保持不变。
//!
//! 判据：
//! ① 摇骰键 4 至少由一源提供（prim 或 schema），且 5/6/7 prim 恰 0 个（三键已取掉）；
//! ② prim 载体须有非零矩形（命中几何非空）；
//! ③ `on_hit` 消费面（键 4 → Spin）可达（headless 走 `poll_hit` → `GameInput::Spin`）。

use ti_app::game_app::{GameApp, GameInput, HitData, Scene};
use ti_app::scenes::MapHost;

/// 场景数据面 = 命中键来源（`assets/data/scenes.json` `main_map.primitives`）。
fn main_map_pick_keys() -> Vec<(u32, i32, i32, i32, i32)> {
    let scenes = ti_core::data::scenes().expect("scenes.json 应可读");
    let scene = scenes.get("main_map").expect("main_map 场景应存在");
    scene
        .primitives
        .iter()
        .filter_map(|p| {
            let key = p
                .params
                .get("pick_key")
                .and_then(serde_json::Value::as_u64)
                .and_then(|v| u32::try_from(v).ok())?;
            Some((key, p.rect.x, p.rect.y, p.rect.w, p.rect.h))
        })
        .collect()
}

/// 世界层 schema 是否声明 `spin`（→ `UiAction` key=4）= 第二命中源。
///
/// 只读 `world_components.json`，不依赖 env 开关（影子期两源并存，判据对两源同时成立）。
fn schema_declares_spin() -> bool {
    use ti_core::data::ui_schema::world_components;
    use ti_core::domain::ui_action::UiAction;
    let scenes = world_components().expect("world_components.json 应可读");
    scenes.get("main_map").is_some_and(|scene| {
        scene.elements.iter().any(|element| {
            element
                .pick
                .as_ref()
                .is_some_and(|pick| UiAction::from_schema_str(&pick.action) == Some(UiAction::Spin))
        })
    })
}

fn enter_main_map(app: &mut GameApp) {
    // 导航序列与 `e2e_playthrough.rs` 一致：Index →（Confirm）SelectRole →（Next 预选
    // 角色 + Confirm）SelectMap →（Next 选图 + Confirm）MainMap。
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
}

/// ① + ②：摇骰键 4 至少一源在位（prim 或 schema），prim 载体矩形非空（点得到 = 不再「玩不了」）。
#[test]
fn main_map_hud_pick_keys_present() {
    let prims = main_map_pick_keys();
    // S87.3 影子期：旧 prim 与 schema `spin_marker` 两源并存，任一提供 spin=4 即可；
    // 两源全缺 = 摇骰不可点（守门），S87.6 清旧 prim 后由 schema 独立承担。
    let hits4: Vec<_> = prims.iter().filter(|(k, ..)| *k == 4).collect();
    assert!(
        !hits4.is_empty() || schema_declares_spin(),
        "main_map 两命中源均无 spin=4（摇骰 CTA 不可点）"
    );
    // prim 若仍在位，须唯一且矩形非空（命中几何守门；S87.6 删 prim 后本段自然跳过）。
    if !hits4.is_empty() {
        assert_eq!(hits4.len(), 1, "main_map prim 应恰有 1 个 pick_key=4");
        let (_, x, y, w, h) = *hits4[0];
        assert!(w > 0 && h > 0, "pick_key=4 矩形须非空（命中几何）");
        assert!(x >= 0 && y >= 0, "pick_key=4 矩形须在屏内");
    }
    // S34 用户 2026-11-01「底部三键取掉」：5/6/7（TradeBuy/TradeSell/建造）随三键
    // 原语删除，main_map 命中层只余 4（摇骰 CTA）。版面规格守门：三键 prim 恰 0 个。
    for key in [5u32, 6, 7] {
        let hits: Vec<_> = prims.iter().filter(|(k, ..)| *k == key).collect();
        assert_eq!(hits.len(), 0, "pick_key={key} 三键已取掉（S34 用户拍板），应 0 个");
    }
}

/// S85.5：缺 `action` = 不可点。摇骰 CTA 的语义 action `spin` 必须在 schema 闭集表
/// `UiAction::ACTION_TABLE` 中登记且映射到 key=4（与 ① 的数据面 `pick_key` 对账）。
#[test]
fn spin_action_is_declared_and_maps_to_key_four() {
    use ti_core::domain::ui_action::UiAction;
    assert_eq!(
        UiAction::from_schema_str("spin").map(UiAction::key),
        Some(4),
        "schema action `spin` 缺登记或 key≠4（摇骰 CTA 不可点）"
    );
}

/// ③：摇骰 CTA（键 4）经归一命中 → `GameInput::Spin` ⇒ 骰点 > 0（能摇）。
#[test]
fn spin_cta_hit_rolls_dice() {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.set_rng_seed(42);
    enter_main_map(&mut app);
    assert_eq!(app.board_last_roll(), 0, "初始未摇骰");
    let changed = app.poll_hit(Some(HitData { key: 4, index: -1, swipe: 0 }));
    assert!(changed || app.board_last_roll() > 0, "pick_key=4 命中应触发摇骰");
    assert!(app.board_last_roll() > 0, "摇骰后应有骰点");
}

/// 边界：非 0 键（如 9）不触发摇骰（命中键严格按数据面约定）。
#[test]
fn unknown_pick_key_does_not_roll() {
    let mut app = GameApp::new(1920.0, 1080.0);
    app.set_rng_seed(42);
    enter_main_map(&mut app);
    app.poll_hit(Some(HitData { key: 99, index: -1, swipe: 0 }));
    assert_eq!(app.board_last_roll(), 0, "未知键不应摇骰");
}
