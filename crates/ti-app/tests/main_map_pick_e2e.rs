//! S27.5 主地图命中回归（用户 2026-10-28 报「没有筛子，玩不了」）。
//!
//! 根因：`scenes.json` `main_map` 场景**零** `pick_key` 原语 ⇒ 摇骰 CTA / 贸易 /
//! 出售 / 建造 四键全不可点（B1 拾取层 `spawn_pick_layer` 只认 `pick_key>0` 原语）。
//! 修复 = 数据面补 `pick_key`（4/5/6/7）+ 轴心补骰点贴图。
//!
//! 判据：
//! ① `main_map` 场景数据面必须同时存在 `pick_key` = 4（摇骰）/ 5（买）/ 6（卖）/ 7（建造）；
//! ② 每个 `pick_key` 原语须有非零矩形（命中几何非空）；
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

/// ① + ②：四个 HUD 键的 `pick_key` 原语在位且矩形非空（点得到 = 不再「玩不了」）。
#[test]
fn main_map_hud_pick_keys_present() {
    let prims = main_map_pick_keys();
    for key in [4u32, 5, 6, 7] {
        let hits: Vec<_> = prims.iter().filter(|(k, ..)| *k == key).collect();
        assert_eq!(hits.len(), 1, "main_map 应恰有 1 个 pick_key={key} 原语");
        let (_, x, y, w, h) = *hits[0];
        assert!(w > 0 && h > 0, "pick_key={key} 矩形须非空（命中几何）");
        assert!(x >= 0 && y >= 0, "pick_key={key} 矩形须在屏内");
    }
}

/// ③：摇骰 CTA（键 4）经归一命中 → `GameInput::Spin` ⇒ 骰点 > 0（能摇）。
#[test]
fn spin_cta_hit_rolls_dice() {
    let mut app = GameApp::new(1080.0, 1920.0);
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
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(42);
    enter_main_map(&mut app);
    app.poll_hit(Some(HitData { key: 99, index: -1, swipe: 0 }));
    assert_eq!(app.board_last_roll(), 0, "未知键不应摇骰");
}
