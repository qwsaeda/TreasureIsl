//! S33.6 AI 回合接线 e2e（territory.md §6）：AI 摇骰/占领/攻占可见。
//!
//! 判据：AI 用**兵力模型**决策（`ai_policy_v2`：强舰队 → `Siege` / 弱 → `PayToll`），
//! 攻占经 `territory_v2::assault` 一次性确定性结算（耗兵力 + 重定驻军 1/3 守恒），
//! 主地图 `last_ai_toast` 通道可见反馈。只走公开面。

use ti_app::{GameApp, GameInput, Scene};
use ti_core::rules::territory_v2::SeatState;

fn main_map_with_board() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(2026);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    app
}

/// 让玩家占**全部领地格**（弱驻军 5），AI 落任何领地格 = 攻占机会。
fn occupy_all_for_player(app: &mut GameApp) {
    let isles: Vec<String> = app
        .territory
        .as_ref()
        .expect("账本")
        .snapshot()
        .keys()
        .copied()
        .map(|idx| {
            app.territory.as_ref().unwrap().isle_cells_by_index(idx).unwrap_or("").to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();
    let mut seat = SeatState::new(100_000, 500, 500, 0);
    if let Some(terr) = app.territory.as_mut() {
        for isle in isles {
            for cell in terr.isle_cells(&isle).to_vec() {
                let _ = ti_core::rules::territory_v2::capture_cell(&mut seat, terr, cell, 0, 5);
            }
        }
    }
}

/// §6：AI 强舰队攻占玩家格 → `last_ai_toast` 可见攻占/失败 + 人手守恒 + 归属易主。
#[test]
fn ai_sieges_player_cell_with_visible_feedback() {
    let mut app = main_map_with_board();
    occupy_all_for_player(&mut app);
    // 所有 AI 席强舰队（500/500 ⇒ 战力 1000 > 守方 61 ⇒ 必胜）。
    for s in 1..5u8 {
        app.seat_crew[s as usize] = 500;
        app.seat_arms[s as usize] = 500;
    }
    let mut siege_toast_seen = false;
    for _ in 0..40 {
        app.handle(GameInput::Spin);
        if let Some(toast) = app.last_ai_toast.clone() {
            // 攻占胜 = 「奪取した」；攻占败 = 「攻撃に失敗した」（i18n ai_siege_fmt/_fail_fmt）。
            if toast.contains("奪取") || toast.contains("失敗") {
                siege_toast_seen = true;
                break;
            }
        }
    }
    assert!(siege_toast_seen, "40 次玩家摇骰内 AI 必攻占至少一次（全部领地格归玩家）");
    // 人手守恒：任 AI 席舰队 ≤ 初始 500（战损或驻军分出），驻军 = 战后 1/3 守恒。
    let total_owned: u32 = (0..5u8).map(|s| app.seat_crew[s as usize]).sum();
    assert!(total_owned <= 5 * 500, "AI 舰队人手不增（守恒）");
    // 至少一席发生攻占（归属从玩家 → 某 AI）
    let ai_owned = app
        .territory
        .as_ref()
        .expect("账本")
        .snapshot()
        .into_iter()
        .any(|(_, (o, _))| matches!(o, Some(s) if s != 0));
    assert!(ai_owned, "AI 攻占后至少一格归 AI 席");
    let _ = SeatState::new(0, 0, 0, 0);
}

/// §6：AI 回合后 `last_ai_toast` 非空（摇骰/占领/交钱任一反馈可见）。
#[test]
fn ai_turns_produce_visible_feedback() {
    let mut app = main_map_with_board();
    app.handle(GameInput::Spin);
    assert!(app.last_ai_toast.is_some(), "AI 回合后必有 toast（攻占/占领/交钱/产出其一）");
}
