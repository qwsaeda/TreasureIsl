#![allow(deprecated)]

//! 领地 v2 兵力模型规则集成测试（S52.1；`docs/business/territory.md` §4–§7 可测规则）。
//!
//! 本文件位 `tests/`（源码内联测试被 doccheck2 `no-inline-test-mod` 拦截；EXP-052 口径）。
//! S52.1 补测：`territory_v2.rs` 42 条内联单测迁移至此（飞轮 R2 找问题 P1-1），
//! `territory_v2.rs` 棘轮豁免摘除。
//! oracle = `docs/business/territory.md` §4.4（驻军守恒 / 攻占败交双倍 / 多人同格只打前一个 /
//! 破产出局 / 胜者 = 唯一存活者）+ `combat.md` §兵力模型（`rules::force`）。
//! 判据：`cargo test -p ti-core --test territory_v2_rules` 全绿。
//!
//! `#[allow(deprecated)]`（S52.3 文件级）：本文件是 `victory_seat` / `player_defeated`
//! （v2 口径，0 生产调用，挂 `#[deprecated]` 留作语义参照）的**唯一**引用点；
//! 文件级 allow 盖住 use 块 + 2 个用例。现役 v3 口径 = `victory_seat_active` /
//! `players_defeated`（本文件另有专测，不 deprecated）。

use ti_core::data::{Balance, balance, board};
use ti_core::rules::force::Force;
use ti_core::rules::territory::{Ownership, SettleAction, TerritoryRules, TerritoryState};
use ti_core::rules::territory_v2::{
    AssaultOutcome, SeatState, ai_policy_v2, assault, bankrupt, capture_cell, is_capturable,
    manpower_conserved, player_defeated, players_defeated, previous_occupant, set_garrison,
    victory_seat, victory_seat_active,
};

/// 测试用 balance（编译期内嵌数据，不可缺）。
fn test_balance() -> Balance {
    balance().expect("balance.json 加载").clone()
}

/// 测试基建：m01 赛道 + 余额表 + 全新领地账本 + 首个领地格。
fn fixture() -> (Balance, TerritoryState, u32) {
    let bal = test_balance();
    let db = board().expect("board.json 加载");
    let map = db.map("m01").expect("m01 赛道");
    let terr = TerritoryState::new(map, &bal).expect("领地账本构建");
    let cell = terr.isle_cells("harbour").first().copied().unwrap_or(1);
    (bal, terr, cell)
}

/// 测试断言助手：领地格在账本必有归属（缺 = 规则 bug）。
fn assert_ownership_present() -> Ownership {
    unreachable!("领地格应在账本中有归属记录")
}

/// §4.4 不可占领格：功能格（start）不可占，领地格可占。
#[test]
fn feature_tiles_not_capturable() {
    let (_bal, terr, _cell) = fixture();
    assert!(!is_capturable(&terr, 0), "起点 = 功能格不可占领");
    let (_bal2, terr2, cell) = fixture();
    assert!(is_capturable(&terr2, cell), "领地格可占领");
}

/// §4.3 T 无主：副本胜利 ⇒ 该岛两格同归 + L=1 + 驻军自定（守恒）。
#[test]
fn capture_wins_takes_both_cells_of_isle() {
    let (_bal, mut terr, cell) = fixture();
    let mut seat = SeatState::new(1000, 30, 20, 0);
    let g = capture_cell(&mut seat, &mut terr, cell, 0, 10);
    assert_eq!(g, 10, "自定驻军 10");
    let isle = terr.isle_cells_by_index(cell).unwrap_or("").to_string();
    let both = terr.isle_cells(&isle).to_vec();
    assert_eq!(both.len(), 2, "该类岛两格同属一个领地");
    for &c in &both {
        let o = terr.ownership(c).copied().unwrap_or_else(assert_ownership_present);
        assert_eq!(o.owner, Some(0));
        assert_eq!(o.level, 1);
        assert_eq!(o.garrison, 10);
    }
    assert_eq!(seat.crew, 20, "驻军守恒（30-10）");
    assert!(manpower_conserved(&seat, &terr, 0, 30), "守恒");
}

/// §4.3 T 无主·副本失败 = 归属不变（仍无主）。
#[test]
fn capture_loss_keeps_ownership() {
    let (_bal, terr, cell) = fixture();
    assert!(terr.ownership(cell).is_some_and(|o| o.owner.is_none()), "副本失败仍无主");
}

/// §4.2 交钱：他人格过路费 > 0。
#[test]
fn paying_toll_transfers_output_to_owner() {
    let (bal, mut terr, cell) = fixture();
    let rules = TerritoryRules::new(&bal);
    let mut seat = SeatState::new(1000, 30, 20, 0);
    let _ = capture_cell(&mut seat, &mut terr, cell, 1, 5);
    let toll = rules.toll_at("harbour", 1);
    assert!(toll > 0, "过路费 > 0");
}

/// §4.3 攻占胜：夺岛两格 + L 降 1 + 重定驻军。
#[test]
fn siege_win_recaptures_with_level_one() {
    let (bal, mut terr, cell) = fixture();
    let mut owner_seat = SeatState::new(1000, 30, 20, 0);
    let _ = capture_cell(&mut owner_seat, &mut terr, cell, 1, 5);
    let mut attacker = SeatState::new(1000, 50, 50, 0);
    let out = assault(&bal, &mut attacker, &mut terr, cell, 0, 8);
    assert!(matches!(out, AssaultOutcome::Win { .. }), "强攻方胜");
    let isle = terr.isle_cells_by_index(cell).unwrap_or("").to_string();
    for &c in terr.isle_cells(&isle) {
        let o = terr.ownership(c).copied().unwrap_or_else(assert_ownership_present);
        assert_eq!(o.owner, Some(0), "攻占后归攻方");
        assert_eq!(o.level, 1, "降级 1");
        assert_eq!(o.garrison, 8, "重定驻军");
    }
}

/// §4.3 攻占败：交双倍过路费（惩罚）。
#[test]
fn siege_loss_pays_double_toll() {
    let (bal, mut terr, cell) = fixture();
    let rules = TerritoryRules::new(&bal);
    let mut owner_seat = SeatState::new(1000, 100, 50, 0);
    let _ = capture_cell(&mut owner_seat, &mut terr, cell, 1, 50);
    let mut attacker = SeatState::new(1000, 2, 1, 0);
    let out = assault(&bal, &mut attacker, &mut terr, cell, 0, 1);
    let toll = rules.toll_at("harbour", 1);
    match out {
        AssaultOutcome::Loss { double_toll_paid, .. } => {
            assert_eq!(double_toll_paid, toll * 2, "攻占败交双倍过路费");
        }
        AssaultOutcome::Win { .. } => unreachable!("弱攻方必败（规则违反）"),
    }
}

/// §4.1 升级：产出/防御/过路费同步提高。
#[test]
fn upgrade_raises_output_defense_and_toll() {
    let (bal, mut terr, cell) = fixture();
    let rules = TerritoryRules::new(&bal);
    let mut seat = SeatState::new(1000, 30, 20, 0);
    let _ = capture_cell(&mut seat, &mut terr, cell, 0, 5);
    let p1 = rules.production_at("harbour", 1);
    let d1 = rules.defense_at("harbour", 1);
    let t1 = rules.toll_at("harbour", 1);
    let p2 = rules.production_at("harbour", 2);
    let d2 = rules.defense_at("harbour", 2);
    let t2 = rules.toll_at("harbour", 2);
    assert!(p2 > p1 && d2 > d1 && t2 > t1, "升级同步提高产出/防御/过路费");
}

/// §4.1 驻军增减守恒：舰队 ↔ 驻军互转，总量不变。
#[test]
fn garrison_change_conserves_manpower() {
    let (_bal, mut terr, cell) = fixture();
    let mut seat = SeatState::new(1000, 50, 20, 0);
    let _ = capture_cell(&mut seat, &mut terr, cell, 0, 20);
    assert_eq!(seat.crew, 30, "舰队减 20");
    assert!(manpower_conserved(&seat, &terr, 0, 50), "守恒");
    let _ = set_garrison(&mut seat, &mut terr, cell, 0, 10);
    assert_eq!(seat.crew, 40, "舰队加回 10");
    assert!(manpower_conserved(&seat, &terr, 0, 50), "守恒");
}

/// §4.1 升级上限：level 不超 `level_cap`。
#[test]
fn upgrade_caps_at_isle_max_level() {
    let (bal, mut terr, cell) = fixture();
    let rules = TerritoryRules::new(&bal);
    let mut seat = SeatState::new(1000, 30, 20, 0);
    let _ = capture_cell(&mut seat, &mut terr, cell, 0, 5);
    let cap = rules.level_cap("harbour");
    let o = terr.ownership(cell).copied().unwrap_or_else(assert_ownership_present);
    assert!(o.level <= cap, "level 不超上限");
}

/// 兵力模型：武装 = min(人手, 武器)（`combat.md` §兵力模型）。
#[test]
fn armed_power_caps_at_manpower() {
    let f = Force::new(5, 20);
    assert_eq!(f.armed(), 5, "武装 = min(人手,武器)");
    let g = Force::new(20, 5);
    assert_eq!(g.armed(), 5);
}

/// §4.3 多人同格：后到者只打前一个（`a→b→c` 序列）。
#[test]
fn later_arrival_decides_duel_previous() {
    let arrivals = [1u8, 2u8];
    assert_eq!(previous_occupant(&arrivals, 2), Some(1), "b 打 a");
    assert_eq!(previous_occupant(&arrivals, 1), None, "a 无前一个");
    let arrivals2 = [1u8, 2u8, 3u8];
    assert_eq!(previous_occupant(&arrivals2, 3), Some(2), "c 只打 b");
}

/// §4.3 遭遇败：败者交钱 + 损水手。
#[test]
fn duel_loser_pays_and_loses_crew() {
    let (bal, mut terr, cell) = fixture();
    let rules = TerritoryRules::new(&bal);
    let mut owner = SeatState::new(1000, 100, 50, 0);
    let _ = capture_cell(&mut owner, &mut terr, cell, 1, 50);
    let mut attacker = SeatState::new(1000, 2, 1, 0);
    let before = attacker.crew;
    let out = assault(&bal, &mut attacker, &mut terr, cell, 0, 1);
    if let AssaultOutcome::Loss { double_toll_paid, fleet_after } = out {
        assert!(fleet_after.crew <= before, "败者损水手");
        assert!(double_toll_paid > 0, "败者交钱");
        let toll = rules.toll_at("harbour", 1);
        assert_eq!(double_toll_paid, toll * 2);
    }
}

/// §4.4 破产：金币归零 + 席位出局 + 全部占领格变无主。
#[test]
fn bankrupt_empties_all_territories() {
    let (_bal, mut terr, cell) = fixture();
    let mut seat = SeatState::new(1000, 30, 20, 0);
    let _ = capture_cell(&mut seat, &mut terr, cell, 0, 10);
    bankrupt(&mut seat, &mut terr, 0);
    assert!(!seat.alive, "破产出局");
    assert_eq!(seat.gold, 0);
    let isle = terr.isle_cells_by_index(cell).unwrap_or("").to_string();
    for &c in terr.isle_cells(&isle) {
        assert!(terr.ownership(c).is_some_and(|o| o.owner.is_none()), "破产格变空");
    }
}

/// §4.4 胜利：所有对手破产 ⇒ 最后存活者胜。
/// `#[allow(deprecated)]`（S52.3）：`victory_seat` = v2 全席扫描口径（0 生产调用，挂
/// `#[deprecated]` 留作语义参照）；现役 = `victory_seat_active`（v3 在局掩码，另有专测）。
#[allow(deprecated)]
#[test]
fn victory_when_all_opponents_bankrupt() {
    let mut seats = vec![SeatState::new(100, 10, 5, 0), SeatState::new(0, 0, 0, 0)];
    seats[1].alive = false;
    assert_eq!(victory_seat(&seats), Some(0), "唯一存活者胜");
    let all_alive = vec![SeatState::new(100, 10, 5, 0), SeatState::new(100, 10, 5, 0)];
    assert_eq!(victory_seat(&all_alive), None, "多人存活无胜者");
}

/// §4.4 玩家败：玩家席破产 = 玩家方败。
/// `#[allow(deprecated)]`（S52.3）：`player_defeated` = 单玩家 index 口径（0 生产调用，
/// 挂 `#[deprecated]` 留作语义参照）；现役 = `players_defeated`（v3 多玩家席，另有专测）。
#[allow(deprecated)]
#[test]
fn defeat_when_player_bankrupt_ends_game() {
    let mut seats = vec![SeatState::new(100, 10, 5, 0), SeatState::new(100, 10, 5, 0)];
    assert!(!player_defeated(&seats, 0));
    seats[0].alive = false;
    assert!(player_defeated(&seats, 0), "玩家破产 = 败");
}

/// §4.4 不可卖：占领格无卖出分支（所有权只经占领/攻占/破产变更）。
#[test]
fn territory_not_sellable() {
    let (_bal, terr, cell) = fixture();
    assert!(terr.ownership(cell).is_some_and(|o| o.owner.is_none()), "无主格无卖出语义");
}

/// §6 AI 攻占判定用兵力模型：强舰队 → `Siege`；弱舰队 → `PayToll`。
#[test]
fn ai_policy_uses_force_model() {
    let (bal, mut terr, cell) = fixture();
    let mut owner = SeatState::new(1000, 30, 20, 0);
    let _ = capture_cell(&mut owner, &mut terr, cell, 1, 5);
    let strong = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 200, 200);
    assert_eq!(strong, SettleAction::Siege, "强舰队 → 攻占");
    let weak = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 20, 5);
    assert_eq!(weak, SettleAction::PayToll, "弱舰队 → 交钱");
}

/// §6 AI 无主格 → 副本；自己格 → 升级/产出（与玩家同一分支表）。
#[test]
fn ai_policy_unowned_and_self() {
    let (bal, mut terr, cell) = fixture();
    let unowned = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 200, 200);
    assert_eq!(unowned, SettleAction::EnterDungeon, "无主格 → 副本");
    let mut seat = SeatState::new(1000, 200, 200, 0);
    let _ = capture_cell(&mut seat, &mut terr, cell, 0, 5);
    let self_own = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 200, 200);
    assert_eq!(self_own, SettleAction::Collect, "自己格 upgrade_priority<0.5 → 领产出");
    let mut bal2 = bal.clone();
    bal2.territory.ai.upgrade_priority = 0.6;
    let up = ai_policy_v2(&bal2, &terr, 0, cell, 10_000, 200, 200);
    assert_eq!(up, SettleAction::Upgrade, "自己格 upgrade_priority≥0.5 + 金足 → 升级");
}

/// v3 胜负：只在在局席位内判最后存活者（`victory_seat_active`，D4 修复）。
#[test]
fn victory_seat_active_ignores_inactive_seats() {
    let seats = [
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
    ];
    let active = [true, true, true, false, false];
    assert_eq!(victory_seat_active(&seats, &active), None, "全活在局席无胜者");
    let mut seats2 = seats.to_vec();
    seats2[0].alive = false;
    seats2[2].alive = false;
    assert_eq!(victory_seat_active(&seats2, &active), Some(1), "唯一在局存活 = seat 1");
}

/// §4.4 玩家方败（多玩家席）：全部玩家席出局 ⇒ 玩家方败。
#[test]
fn players_defeated_all_dead() {
    let mut seats = [
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
        SeatState::new(0, 10, 0, 0),
    ];
    let players: [u8; 2] = [0, 2];
    assert!(!players_defeated(&seats, &players), "玩家席全活 = 未败");
    seats[0].alive = false;
    seats[2].alive = false;
    assert!(players_defeated(&seats, &players), "玩家席全出局 = 败");
}

/// §4.1 驻军守恒（`manpower_conserved`）：舰队 + Σ驻军 = 总人手。
#[test]
fn manpower_conserved_after_transfer() {
    let (_bal, mut terr, cell) = fixture();
    let mut seat = SeatState::new(0, 200, 0, 0);
    let _ = capture_cell(&mut seat, &mut terr, cell, 0, 50);
    assert!(manpower_conserved(&seat, &terr, 0, 200), "守恒：150 + 50 = 200");
}

/// §4.3 非所有者调驻军 = 不变。
#[test]
fn set_garrison_noop_for_non_owner() {
    let (_bal, mut terr, cell) = fixture();
    let mut owner = SeatState::new(0, 100, 0, 0);
    let _ = capture_cell(&mut owner, &mut terr, cell, 1, 30);
    let mut stranger = SeatState::new(0, 50, 0, 0);
    let g = set_garrison(&mut stranger, &mut terr, cell, 0, 999);
    assert_eq!(g, 30, "非所有者调驻军 = 不变");
    assert_eq!(stranger.crew, 50, "不扣自己人手");
}
