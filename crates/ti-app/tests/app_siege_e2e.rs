//! S33.5 攻占接线 e2e（territory.md §4）：落他人领地格 → 待决攻占二选一
//! （Confirm = 攻占战报 / Back = 交过路费）→ Vs 战报 Attack 一次性结算 →
//! 胜 = 占领该岛两格（守军降级）/ 败 = 交双倍过路费。
//!
//! 只走公开面（`handle` 输入 + `siege_snapshot`/`territory_ownership` + pub 字段），
//! 不触 private 结算方法（`settle_siege` 由 `VsScene::on_input` 驱动）。

use ti_app::{GameApp, GameInput, MapHost, Scene};
use ti_core::rules::territory::Ownership;
use ti_core::rules::territory_v2::SeatState;

fn main_map_with_board() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(11);
    app.set_rng_seed(11);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    app
}

/// 直接让 AI 席占领首格（构造待决攻占：玩家落他人格）。
///
/// 用 `territory_v2::capture_cell` 把某 AI 席设为格 owner（同规则层口径），
/// 再摇骰到该格 ⇒ `pending_siege` 在位。
fn occupy_first_cell(app: &mut GameApp) -> u32 {
    let cell = app
        .territory
        .as_ref()
        .expect("领地账本")
        .isle_cells("harbour")
        .first()
        .copied()
        .expect("m01 有 harbour");
    let ai_seat = 1u8; // AI 席（单玩家局 = 席 0 玩家）
    let mut seat = SeatState::new(1000, 30, 20, 0);
    if let Some(terr) = app.territory.as_mut() {
        let _ = ti_core::rules::territory_v2::capture_cell(&mut seat, terr, cell, ai_seat, 5);
    }
    cell
}

/// 摇骰到落**他人**格（`pending_siege` 在位 = 待决攻占）。
fn spin_to_other_owned(app: &mut GameApp) -> u32 {
    let target = occupy_first_cell(app);
    for _ in 0..80 {
        app.handle(GameInput::Spin);
        if let Some((cell, _owner)) = app.pending_siege {
            if cell == target {
                return cell;
            }
        }
    }
    unreachable!("80 次摇骰必落一次他人格（AI 已占 harbour，玩家绕环必经过）")
}

/// 他人格落格 → `pending_siege` 在位（`Confirm` = 进攻占战报）。
#[test]
fn landing_other_owned_sets_pending_siege() {
    let mut app = main_map_with_board();
    let cell = spin_to_other_owned(&mut app);
    let (c, owner) = app.pending_siege.expect("待决攻占应在位");
    assert_eq!(c, cell);
    assert_eq!(owner, 1, "守方 = AI 席 1");
    // 战报未开（`Confirm` 后才切 Vs）
    assert!(app.siege_snapshot().is_none(), "未确认前无战报");
}

/// 他人格落格 `Confirm` = 切 Vs 攻占战报（`siege_snapshot` 在位）。
#[test]
fn confirm_siege_enters_vs_report() {
    let mut app = main_map_with_board();
    spin_to_other_owned(&mut app);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Vs, "确认攻占 = 切 Vs");
    let s = app.siege_snapshot().expect("攻占战报应在位");
    assert!(s.attacker_crew > 0, "攻方舰队人手 > 0");
    assert!(s.defender_crew > 0, "守方（驻军+防御）> 0");
    assert!(s.won.is_none(), "未结算前无结果");
}

/// 攻占战报 `Attack` 一次性结算：胜 = 占领该岛两格 / 败 = 守方不变（结果已写回账本）。
#[test]
fn siege_attack_settles_once() {
    let mut app = main_map_with_board();
    spin_to_other_owned(&mut app);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Attack);
    let s = app.siege_snapshot().expect("结算后战报仍在");
    let won = s.won.expect("Attack 一次必出结果");
    let cell = s.cell;
    let my_seat = app.turn_cycle.current;
    let terr = app.territory.as_ref().expect("账本");
    let own = terr.ownership(cell).copied().expect("该格在账本");
    if won {
        assert_eq!(own.owner, Some(my_seat), "攻占胜 = 归攻方");
        assert_eq!(own.level, 1, "守军降级 1");
    } else {
        // 败：守方仍 AI 席（不变），攻方交双倍过路费已扣金。
        assert_eq!(own.owner, Some(1), "攻占败 = 守方不变");
    }
    // 再 Attack = no-op（已结算，结果保留）；Back = 回主地图
    app.handle(GameInput::Attack);
    assert!(app.siege_snapshot().is_some(), "已结算再 Attack 不变（no-op）");
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::MainMap, "战报 Back 回主地图");
}

/// 攻占**胜**路径：强攻方（大舰队）打弱守方 ⇒ 占领该岛两格 + 守军降级 + 驻军重定。
///
/// 默认开局 20/5 攻方 vs 守方（驻军 5 + defence L1=28）战力 = 25 vs 61 ⇒ 必败；
/// 抬攻方人手/武器到 200/200 ⇒ 战力 = 200×2 = 400 > 61 ⇒ 必胜（确定性）。
#[test]
fn siege_strong_attacker_wins() {
    let mut app = main_map_with_board();
    let cell = spin_to_other_owned(&mut app);
    let my_seat = app.turn_cycle.current;
    app.seat_crew[my_seat as usize] = 200;
    app.seat_arms[my_seat as usize] = 200;
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Attack);
    let s = app.siege_snapshot().expect("结算后战报仍在");
    assert_eq!(s.won, Some(true), "强攻方必胜（确定性）");
    let terr = app.territory.as_ref().expect("账本");
    // 占领该岛两格（同领地），守军降级 1
    let isle = terr.isle_cells_by_index(cell).expect("该格有岛").to_string();
    for idx in terr.isle_cells(&isle) {
        let o = terr.ownership(*idx).copied().expect("在账本");
        assert_eq!(o.owner, Some(my_seat), "攻占胜 = 两格归攻方（格 {idx}）");
        assert_eq!(o.level, 1, "守军降级 1（格 {idx}）");
    }
    // 驻军重定（战后舰队的 1/3 留驻 = 守恒）
    assert!(terr.garrison_total(my_seat) > 0, "攻占胜 = 重定驻军 > 0（守恒）");
    // 攻方舰队人手 = 战后剩余（守恒：舰队 + 驻军 = 200 - 战损）
    assert!(
        app.seat_crew[my_seat as usize] + terr.garrison_total(my_seat) <= 200,
        "人手守恒（战后剩余 + 驻军 ≤ 初始 200）"
    );
}

/// 他人格落格 `Back` = 放弃攻占交过路费（`pending_siege` 清 + 金转移）。
#[test]
fn back_pays_toll_clears_siege() {
    let mut app = main_map_with_board();
    spin_to_other_owned(&mut app);
    let my_seat = app.turn_cycle.current;
    let gold_before = app.seat_gold[my_seat as usize];
    app.handle(GameInput::Back);
    assert!(app.pending_siege.is_none(), "Back 清待决攻占");
    assert!(app.seat_gold[my_seat as usize] <= gold_before, "交过路费 = 金不增（可能减）");
    assert_eq!(app.scene(), Scene::MainMap, "交钱后仍主地图");
    let _ = Ownership::unowned();
}
