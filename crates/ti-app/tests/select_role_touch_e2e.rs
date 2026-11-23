#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S77 选角页触摸端到端：点未勾选席位 = 加入多选；点已勾选席位 = 确认推进（对齐 `select_map`）。

use ti_app::game_app::{GameApp, GameInput, HitData, Scene};
use ti_app::MapHost;

fn enter_select_role() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(42);
    app.handle(GameInput::Confirm); // Index → SelectRole
    assert_eq!(app.scene(), Scene::SelectRole);
    app
}

fn hit(index: i32) -> HitData {
    HitData { key: 2, index, swipe: 0 }
}

#[test]
fn touch_unmarked_seat_joins_multi_select() {
    let mut app = enter_select_role();
    // 初始无勾选（min_players=2 需勾 2 席）。点第 0 席（潘达）→ 加入多选，不推进。
    let before = app.player_seats().len();
    app.poll_hit(Some(hit(0)));
    assert_eq!(app.scene(), Scene::SelectRole, "点未勾选席位不推进场景");
    assert_eq!(app.player_seats().len(), before + 1, "加入多选席位");
}

#[test]
fn touch_marked_seat_confirms_advances() {
    let mut app = enter_select_role();
    // 先勾 2 席（满足 min_players=2），再点已勾选席位 → 确认推进到选图。
    app.poll_hit(Some(hit(0)));
    app.poll_hit(Some(hit(1)));
    assert!(app.player_seats().len() >= 2, "已勾 2 席");
    app.poll_hit(Some(hit(0))); // 点已勾选席位 = 确认
    assert_eq!(app.scene(), Scene::SelectMap, "点已勾选席位触摸确认推进到选图");
}

#[test]
fn touch_marked_seat_respects_min_players() {
    let mut app = enter_select_role();
    app.poll_hit(Some(hit(0))); // 勾 1 席（不足 2）
    // 点已勾选席位但人数不足 → on_hit 仍 advance Confirm，Confirm 会因不足拒绝推进
    // （见 select_role Confirm 分支的 seat_selection_ready 校验），场景保持 SelectRole。
    app.poll_hit(Some(hit(0)));
    assert_eq!(app.scene(), Scene::SelectRole, "人数不足时确认被拒，不推进");
}
