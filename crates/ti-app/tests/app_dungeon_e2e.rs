//! S28.6 副本接线 e2e（`territory.md` §3/§4/§9）：无主落格 → 待决副本入口 →
//! 确认进守军战 → 战至终局 = 胜利占领该岛两格（首占奖励）/ 失败现状不变。
//!
//! 只走公开面（`handle` 输入 + `battle()` 只读 + `territory`/`turn_cycle` pub 字段），
//! 不触 private 结算方法（`settle_dungeon_win`/`toast_dungeon_fail` 由 `battle_step` 内部驱动）。

use ti_app::{GameApp, GameInput, Scene};
use ti_core::domain::combat::Side;
use ti_core::rules::territory::Ownership;

fn main_map_with_board() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    app
}

/// 摇骰到落无主领地格（`pending_dungeon` 在位）。
fn spin_to_unowned(app: &mut GameApp) -> u32 {
    for _ in 0..40 {
        app.handle(GameInput::Spin);
        if let Some(cell) = app.pending_dungeon {
            return cell;
        }
    }
    unreachable!("40 次摇骰必落至少一个无主领地格（31 格含 20 领地格）")
}

/// 无主落格 → 确认进守军战（`battle.dungeon` 在位）。
fn enter_dungeon_battle(app: &mut GameApp) -> (u32, String, u8) {
    let cell = spin_to_unowned(app);
    app.handle(GameInput::Confirm);
    let view = app.battle().expect("确认后应进副本战斗");
    let (c, isle, lvl) = view.dungeon.clone().expect("battle.dungeon 应在位");
    assert_eq!(c, cell, "副本格 = 落格");
    (c, isle, lvl)
}

/// 驱动副本战至终局（反复 `Attack`；`outcome` 必达，战斗模型有界）。
fn fight_to_outcome(app: &mut GameApp) {
    for _ in 0..200 {
        if app.battle().is_some_and(|b| b.outcome().is_some()) {
            return;
        }
        app.handle(GameInput::Attack);
    }
    // 战斗模型有界（`balance.combat.max_rounds` 兜底判胜负），200 回合必达终局。
    assert!(
        app.battle().is_some_and(|b| b.outcome().is_some()),
        "200 回合未出终局（战斗模型应收敛）"
    );
}

/// 副本胜利 = 占领该岛两格（level 1）+ 结算钩消费 `dungeon`。
#[test]
fn dungeon_win_occupies_isle_cells() {
    let mut app = main_map_with_board();
    let (_cell, isle, _lvl) = enter_dungeon_battle(&mut app);
    fight_to_outcome(&mut app);
    let won = app.battle().and_then(ti_app::BattleView::outcome) == Some(Side::Player);
    let terr = app.territory.as_ref().expect("领地账本应在位");
    let my_seat = app.turn_cycle.current;
    if won {
        for idx in terr.isle_cells(&isle) {
            assert_eq!(
                terr.ownership(*idx).copied(),
                Some(Ownership { owner: Some(my_seat), level: 1, garrison: 0 }),
                "副本胜利 = 占领该岛格 {idx}"
            );
        }
    }
    // 终局后 Back 路由回主地图（vs `in_dungeon_battle` = 副本 ⇒ MainMap；BOSS ⇒ Index）
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::MainMap, "副本终局后 Back 回主地图");
}

/// 副本战入口接线：`dungeon` 字段在位 + 守军按原型构建（非 BOSS 战 = `dungeon` 非 None）。
#[test]
fn dungeon_entry_wires_defender_battle() {
    let mut app = main_map_with_board();
    let (_cell, _isle, _lvl) = enter_dungeon_battle(&mut app);
    let view = app.battle().expect("副本战应在位");
    assert!(view.dungeon.is_some(), "dungeon 战斗态标记在位（与 BOSS 战互斥）");
}
