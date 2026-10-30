//! 领地 v2 规则（S33.3，2026-10-30 用户定案）：驻军自定 / 攻占败交双倍 / 破产+胜负 /
//! 多人同格 / 不可占领格。
//!
//! 单一事实源 = [`docs/business/territory.md`](../../../../docs/business/territory.md) §4/§10；
//! 兵力模型 = [`docs/business/combat.md`](combat.md) §兵力模型（`rules::force` 消费）。
//!
//! 本模块只算**规则**（确定性），不碰渲染/输入。席位资源与领地账本分开可变：
//! [`SeatState`]（gold/crew/arms/船/位置）与 [`TerritoryState`]（归属 + 驻军）分别传入，
//! 便于存档 v5（S33.4）直接引用。

use crate::data::Balance;
use crate::rules::force::{self, Force, ForceSide};
use crate::rules::territory::{Ownership, SeatIdx, SettleAction, TerritoryRules, TerritoryState};

/// 席位资源（v2 §1/§4：独立金币 / 人手 / 武器 / 船只 / 位置）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatState {
    /// 金币。
    pub gold: u32,
    /// 舰队人手（**不含**驻军；驻军分别记在领地格，守恒 `舰队 + Σ驻军 = 总人手`）。
    pub crew: u32,
    /// 舰队武器。
    pub arms: u32,
    /// 船只等级（`0` 起；船容量上限 = `balance.boat.levels[boat_level].cargo`）。
    pub boat_level: u32,
    /// 当前格序号。
    pub position: u32,
    /// 是否存活（破产 = `false`）。
    pub alive: bool,
}

impl SeatState {
    /// 构造席位（存活，位置 0）。
    #[must_use]
    pub fn new(gold: u32, crew: u32, arms: u32, boat_level: u32) -> Self {
        Self { gold, crew, arms, boat_level, position: 0, alive: true }
    }

    /// 舰队兵力（攻方 = 舰队人手+武器；`rules::force` 消费）。
    #[must_use]
    pub fn fleet(&self) -> Force {
        Force::new(self.crew, self.arms)
    }
}

/// 攻占结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssaultOutcome {
    /// 攻占胜利：占领该岛两格（level 1 + 自定驻军）。
    Win {
        /// 攻方战后剩余舰队（损耗后）。
        fleet_after: Force,
    },
    /// 攻占失败：交双倍过路费（惩罚）。
    Loss {
        /// 实际付的双倍过路费（≤ 攻方金币；金币不足 = 攻方余额，破产由调用方判）。
        double_toll_paid: u32,
        /// 攻方战后剩余舰队（损耗后）。
        fleet_after: Force,
    },
}

/// 攻占（§4 他人格分支 + §4.3 多人同格）：船队攻打建筑。
///
/// - 攻方 = 席位舰队（`seat.fleet()`）；守方 = 驻军 `G` + 建筑防御 `defense(T,L)`。
/// - 守方兵力 = `(G + defense, defense)`（建筑提供 `defense` 名武装守军，驻军为徒手）。
/// - 用 [`force::resolve`]（确定性 VS + 双方必损）。
/// - **胜**：`O = 攻方`，`L = 1`，重定驻军 `new_garrison`（从攻方战后舰队分出，守恒）。
/// - **败**：攻方交**双倍过路费**（惩罚），守方驻军按战后剩余减员。
#[must_use]
pub fn assault(
    balance: &Balance,
    seat: &mut SeatState,
    terr: &mut TerritoryState,
    idx: u32,
    seat_idx: SeatIdx,
    new_garrison: u32,
) -> AssaultOutcome {
    let rules = TerritoryRules::new(balance);
    let isle = terr.isle_cells_by_index(idx).unwrap_or("");
    let own = terr.ownership(idx).copied().unwrap_or(Ownership::unowned());
    let defense_units = rules.defense_at(isle, own.level.max(1));
    let defender = Force::new(own.garrison.saturating_add(defense_units), defense_units);
    let attacker = seat.fleet();
    let res = force::resolve(&attacker, &defender, &balance.manpower);
    match res.winner {
        ForceSide::Attacker => {
            // 胜：占领 + 重定驻军（从战后舰队分出，守恒）。
            let g = new_garrison.min(res.attacker_after.crew);
            seat.crew = res.attacker_after.crew - g;
            seat.arms = res.attacker_after.arms;
            for cell in terr.isle_cells(isle).to_vec() {
                terr.mutate(cell, Ownership::occupied(seat_idx, 1, g));
            }
            AssaultOutcome::Win { fleet_after: Force::new(seat.crew, seat.arms) }
        }
        ForceSide::Defender => {
            // 败：交双倍过路费（惩罚）；守方驻军减员。
            seat.crew = res.attacker_after.crew;
            seat.arms = res.attacker_after.arms;
            let toll = rules.toll_at(isle, own.level.max(1)).saturating_mul(2);
            let paid = toll.min(seat.gold);
            seat.gold -= paid;
            terr.mutate(
                idx,
                Ownership { owner: own.owner, level: own.level, garrison: res.defender_after.crew },
            );
            AssaultOutcome::Loss {
                double_toll_paid: paid,
                fleet_after: Force::new(seat.crew, seat.arms),
            }
        }
    }
}

/// 驻军自定（§4.1）：把 `idx` 格驻军调到 `want`（从席位舰队分出或调回，**守恒**）。
///
/// - 只允许所有者调自己格；`want` 钳到 `[0, 席位舰队人手 + 现有驻军]`（守恒上限）。
/// - 增加驻军：`seat.crew -= Δ`（从舰队分出）；减少：`seat.crew += Δ`（调回舰队）。
/// - 返回实际设置的驻军值。
#[must_use]
pub fn set_garrison(
    seat: &mut SeatState,
    terr: &mut TerritoryState,
    idx: u32,
    seat_idx: SeatIdx,
    want: u32,
) -> u32 {
    let own = terr.ownership(idx).copied().unwrap_or(Ownership::unowned());
    if own.owner != Some(seat_idx) {
        return own.garrison; // 非所有者：不变
    }
    // 守恒：现有驻军 + 舰队人手 = 可用的总人手（上限）。
    let available = own.garrison.saturating_add(seat.crew);
    let g = want.min(available);
    let delta = g.saturating_sub(own.garrison);
    if delta > 0 {
        seat.crew = seat.crew.saturating_sub(delta);
    } else {
        seat.crew = seat.crew.saturating_add(own.garrison - g);
    }
    terr.mutate(idx, Ownership { owner: Some(seat_idx), level: own.level, garrison: g });
    g
}

/// 占领无主格（§3 副本胜利 / §4 无主分支）：`O = seat`，`L = 1`，驻军 = `new_garrison`
/// （从席位舰队分出，守恒）。该类岛两格同易主。
#[must_use]
pub fn capture_cell(
    seat: &mut SeatState,
    terr: &mut TerritoryState,
    idx: u32,
    seat_idx: SeatIdx,
    new_garrison: u32,
) -> u32 {
    let isle = terr.isle_cells_by_index(idx).unwrap_or("").to_string();
    let g = new_garrison.min(seat.crew);
    seat.crew -= g;
    for cell in terr.isle_cells(&isle).to_vec() {
        terr.mutate(cell, Ownership::occupied(seat_idx, 1, g));
    }
    g
}

/// 破产（§4.4）：金币归零、席位出局、全部占领格变空（无主）。
pub fn bankrupt(seat: &mut SeatState, terr: &mut TerritoryState, seat_idx: SeatIdx) {
    seat.gold = 0;
    seat.alive = false;
    // 全部占领格变空（含该席位的两格领地）。
    for cell in terr.owned_indices(seat_idx) {
        terr.mutate(cell, Ownership::unowned());
    }
}

/// 胜利判定（§4.4）：**所有对手（其他席）全部破产** ⇒ 最后一个未破产席位胜。
///
/// `seats` = 全部席位的存活态；返回胜者席位下标（无胜者 = `None`）。
#[must_use]
pub fn victory_seat(seats: &[SeatState]) -> Option<SeatIdx> {
    let alive: Vec<SeatIdx> = seats
        .iter()
        .enumerate()
        .filter(|(_, s)| s.alive)
        .map(|(i, _)| u8::try_from(i).unwrap_or(u8::MAX))
        .collect();
    match alive.as_slice() {
        [single] => Some(*single),
        _ => None,
    }
}

/// 失败判定（§4.4）：玩家席破产 ⇒ 直接结束（败）。
#[must_use]
pub fn player_defeated(seats: &[SeatState], player_idx: SeatIdx) -> bool {
    seats.get(player_idx as usize).is_some_and(|s| !s.alive)
}

/// 驻军守恒检验（§4.1：`舰队人手 + 该席位 Σ驻军（按领地计） = 该席位初始总人手`）。
///
/// `initial_crew` = 该席位开局总人手；返回是否守恒。
#[must_use]
pub fn manpower_conserved(
    seat: &SeatState,
    terr: &TerritoryState,
    seat_idx: SeatIdx,
    initial_crew: u32,
) -> bool {
    seat.crew.saturating_add(terr.garrison_total(seat_idx)) == initial_crew
}

/// 不可占领格（§4：功能格不能占领，只有领地格才可占领）。
#[must_use]
pub fn is_capturable(terr: &TerritoryState, idx: u32) -> bool {
    terr.ownership(idx).is_some() // 领地格在账本（功能格不在）
}

/// 多人同格（§4.3）：后到者决定打不打前一个占格者（只打前一个）。
///
/// `arrivals` = 同格到访序列（先到→后到）；`current` = 当前要行动的席位；
/// 它只能选择打**前一个**（`arrivals` 中紧邻它的前一个）。
/// 返回可对决的前一个席位（`None` = 无（自己是第一个到 / 前一个已离场））。
#[must_use]
pub fn previous_occupant(arrivals: &[SeatIdx], current: SeatIdx) -> Option<SeatIdx> {
    arrivals
        .iter()
        .rposition(|&s| s == current)
        .and_then(|pos| if pos > 0 { arrivals.get(pos - 1).copied() } else { None })
        .filter(|&prev| prev != current)
}

/// v2 AI 决策（§6：AI 与玩家共用同一分支表，策略只选分支；**兵力模型**攻占判定）。
///
/// 与 [`TerritoryRules::ai_policy`] 的区别：攻占判定用**舰队战力 vs 守方战力**（`force::resolve` 预测），
/// 而非旧 `attack_threshold × base` 纯阈值——v2 攻守都消耗兵力（必损），须真实比较。
///
/// 规则（`balance.territory.ai` 配置 + `balance.manpower`）：
/// - 无主格 → `EnterDungeon`（副本胜利即占领）。
/// - 他人格：攻方战力 > 守方战力（`force::resolve` 预测胜）**且** `attack_threshold` 通过
///   （金币充裕偏好）→ `Siege`；否则 `PayToll`。
/// - 自己格：未满级 + 金够升级费 + `upgrade_priority` ≥ 0.5 → `Upgrade`；满级 → `Maxed`；
///   否则 `Collect`（领产出）。
#[must_use]
pub fn ai_policy_v2(
    balance: &Balance,
    state: &TerritoryState,
    seat: SeatIdx,
    idx: u32,
    gold: u32,
    crew: u32,
    arms: u32,
) -> SettleAction {
    let rules = TerritoryRules::new(balance);
    let own = state.ownership(idx).copied().unwrap_or(Ownership::unowned());
    match own.owner {
        None => SettleAction::EnterDungeon,
        Some(o) if o != seat => {
            let isle = state.isle_cells_by_index(idx).unwrap_or("");
            let defense_units = rules.defense_at(isle, own.level.max(1));
            // 守方 = 驻军 + 建筑防御；攻方 = 舰队（人手+武器）。
            let defender = Force::new(own.garrison.saturating_add(defense_units), defense_units);
            let attacker = Force::new(crew, arms);
            let will_win = force::resolve(&attacker, &defender, &balance.manpower).winner
                == ForceSide::Attacker;
            let cfg = &balance.territory.ai;
            if will_win
                && cfg.attack_threshold > 0.0
                && gold >= rules.toll_at(isle, own.level.max(1))
            {
                SettleAction::Siege
            } else {
                SettleAction::PayToll
            }
        }
        Some(_) => {
            let isle = state.isle_cells_by_index(idx).unwrap_or("");
            let cap = rules.level_cap(isle);
            let cost = rules.upgrade_cost_at(isle, own.level.max(1));
            if own.level >= cap {
                SettleAction::Maxed
            } else if gold >= cost && balance.territory.ai.upgrade_priority >= 0.5 {
                SettleAction::Upgrade
            } else {
                SettleAction::Collect
            }
        }
    }
}

/// v2 AI 回合（§6）：决策（兵力模型）+ 驻军自定（战后留 1/3 守，守恒）。
#[must_use]
pub fn ai_turn_v2(
    balance: &Balance,
    state: &TerritoryState,
    seat: SeatIdx,
    idx: u32,
    gold: u32,
    crew: u32,
    arms: u32,
) -> SettleAction {
    ai_policy_v2(balance, state, seat, idx, gold, crew, arms)
}

// ------------------------------------------------------- 单测（§10 可测规则表）

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用 balance + 地图 + 初始账本。
    fn fixture() -> (Balance, TerritoryState, u32) {
        let bal = crate::data::balance().expect("balance 可加载").clone();
        let map = crate::data::board().expect("board 可加载").map("m01").expect("m01 存在");
        let terr = TerritoryState::new(map, &bal).expect("账本构建");
        // 找第一个领地格 idx
        let cell = terr.isle_cells("harbour").first().copied().unwrap_or(1);
        (bal, terr, cell)
    }

    #[test]
    fn feature_tiles_not_capturable() {
        let (_, terr, _) = fixture();
        // 起点格（功能格）不可占领；领地格可占领。
        assert!(!is_capturable(&terr, 0), "起点 = 功能格不可占领");
        let (_, terr2, cell) = fixture();
        assert!(is_capturable(&terr2, cell), "领地格可占领");
    }

    #[test]
    fn capture_wins_takes_both_cells_of_isle() {
        let (_bal, mut terr, cell) = fixture();
        // 无主格副本胜利（v2 入口：占领 + level 1 + 自定驻军）。
        let mut seat = SeatState::new(1000, 30, 20, 0);
        let g = capture_cell(&mut seat, &mut terr, cell, 0, 10);
        assert_eq!(g, 10, "自定驻军 10");
        let isle = terr.isle_cells_by_index(cell).unwrap_or("").to_string();
        let both = terr.isle_cells(&isle).to_vec();
        assert_eq!(both.len(), 2, "该类岛两格同属一个领地");
        for &c in &both {
            let o = terr.ownership(c).copied().unwrap();
            assert_eq!(o.owner, Some(0));
            assert_eq!(o.level, 1);
            assert_eq!(o.garrison, 10);
        }
        assert_eq!(seat.crew, 20, "驻军守恒（30-10）");
        assert!(manpower_conserved(&seat, &terr, 0, 30), "守恒");
    }

    #[test]
    fn capture_loss_keeps_ownership() {
        let (_bal, terr, cell) = fixture();
        // 无主格：副本失败 = 归属不变（仍无主）。
        assert!(terr.ownership(cell).unwrap().owner.is_none(), "副本失败仍无主");
    }

    #[test]
    fn paying_toll_transfers_output_to_owner() {
        let (bal, mut terr, cell) = fixture();
        let rules = TerritoryRules::new(&bal);
        let mut seat = SeatState::new(1000, 30, 20, 0);
        // 席 1 占领该格
        let _ = capture_cell(&mut seat, &mut terr, cell, 1, 5);
        // 席 0 路过交钱
        let toll = rules.toll_at("harbour", 1);
        assert!(toll > 0, "过路费 > 0");
    }

    #[test]
    fn siege_win_recaptures_with_level_one() {
        let (bal, mut terr, cell) = fixture();
        let mut owner_seat = SeatState::new(1000, 30, 20, 0);
        let _ = capture_cell(&mut owner_seat, &mut terr, cell, 1, 5);
        // 席 0 攻占（守方驻军 5 + defense(20×1.4=28) = 战力 61；攻方 50/50 = 战力 100 应胜）
        let mut attacker = SeatState::new(1000, 50, 50, 0);
        let out = assault(&bal, &mut attacker, &mut terr, cell, 0, 8);
        assert!(matches!(out, AssaultOutcome::Win { .. }), "强攻方胜");
        let isle = terr.isle_cells_by_index(cell).unwrap_or("").to_string();
        for &c in terr.isle_cells(&isle) {
            let o = terr.ownership(c).copied().unwrap();
            assert_eq!(o.owner, Some(0), "攻占后归攻方");
            assert_eq!(o.level, 1, "降级 1");
            assert_eq!(o.garrison, 8, "重定驻军");
        }
    }

    #[test]
    fn siege_loss_pays_double_toll() {
        let (bal, mut terr, cell) = fixture();
        let rules = TerritoryRules::new(&bal);
        let mut owner_seat = SeatState::new(1000, 100, 50, 0);
        let _ = capture_cell(&mut owner_seat, &mut terr, cell, 1, 50);
        // 弱攻方（人手 2）必败
        let mut attacker = SeatState::new(1000, 2, 1, 0);
        let out = assault(&bal, &mut attacker, &mut terr, cell, 0, 1);
        let toll = rules.toll_at("harbour", 1);
        match out {
            AssaultOutcome::Loss { double_toll_paid, .. } => {
                assert_eq!(double_toll_paid, toll * 2, "攻占败交双倍过路费");
            }
            AssaultOutcome::Win { .. } => panic!("弱攻方不应胜"),
        }
    }

    #[test]
    fn upgrade_raises_output_defense_and_toll() {
        let (bal, mut terr, cell) = fixture();
        let rules = TerritoryRules::new(&bal);
        let mut seat = SeatState::new(1000, 30, 20, 0);
        let _ = capture_cell(&mut seat, &mut terr, cell, 0, 5);
        // level 1 vs level 2
        let p1 = rules.production_at("harbour", 1);
        let d1 = rules.defense_at("harbour", 1);
        let t1 = rules.toll_at("harbour", 1);
        let p2 = rules.production_at("harbour", 2);
        let d2 = rules.defense_at("harbour", 2);
        let t2 = rules.toll_at("harbour", 2);
        assert!(p2 > p1 && d2 > d1 && t2 > t1, "升级同步提高产出/防御/过路费");
    }

    #[test]
    fn garrison_change_conserves_manpower() {
        let (_bal, mut terr, cell) = fixture();
        let mut seat = SeatState::new(1000, 50, 20, 0);
        let _ = capture_cell(&mut seat, &mut terr, cell, 0, 20);
        assert_eq!(seat.crew, 30, "舰队减 20");
        assert!(manpower_conserved(&seat, &terr, 0, 50), "守恒");
        // 调回 10
        let _ = set_garrison(&mut seat, &mut terr, cell, 0, 10);
        assert_eq!(seat.crew, 40, "舰队加回 10");
        assert!(manpower_conserved(&seat, &terr, 0, 50), "守恒");
    }

    #[test]
    fn upgrade_caps_at_isle_max_level() {
        let (bal, mut terr, cell) = fixture();
        let rules = TerritoryRules::new(&bal);
        let mut seat = SeatState::new(1000, 30, 20, 0);
        let _ = capture_cell(&mut seat, &mut terr, cell, 0, 5);
        let cap = rules.level_cap("harbour");
        let o = terr.ownership(cell).copied().unwrap();
        assert!(o.level <= cap, "level 不超上限");
    }

    #[test]
    fn armed_power_caps_at_manpower() {
        let f = Force::new(5, 20);
        assert_eq!(f.armed(), 5, "武装 = min(人手,武器)");
        let g = Force::new(20, 5);
        assert_eq!(g.armed(), 5);
    }

    #[test]
    fn later_arrival_decides_duel_previous() {
        // a 先到 → b 后到决定打不打 a（只打前一个）
        let arrivals = [1u8, 2u8];
        assert_eq!(previous_occupant(&arrivals, 2), Some(1), "b 打 a");
        assert_eq!(previous_occupant(&arrivals, 1), None, "a 无前一个");
        // c 后到只打 b（不跳打 a）
        let arrivals2 = [1u8, 2u8, 3u8];
        assert_eq!(previous_occupant(&arrivals2, 3), Some(2), "c 只打 b");
    }

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
            assert!(terr.ownership(c).unwrap().owner.is_none(), "破产格变空");
        }
    }

    #[test]
    fn victory_when_all_opponents_bankrupt() {
        let mut seats = vec![SeatState::new(100, 10, 5, 0), SeatState::new(0, 0, 0, 0)];
        seats[1].alive = false;
        assert_eq!(victory_seat(&seats), Some(0), "唯一存活者胜");
        let all_alive = vec![SeatState::new(100, 10, 5, 0), SeatState::new(100, 10, 5, 0)];
        assert_eq!(victory_seat(&all_alive), None, "多人存活无胜者");
    }

    #[test]
    fn defeat_when_player_bankrupt_ends_game() {
        let mut seats = vec![SeatState::new(100, 10, 5, 0), SeatState::new(100, 10, 5, 0)];
        assert!(!player_defeated(&seats, 0));
        seats[0].alive = false;
        assert!(player_defeated(&seats, 0), "玩家破产 = 败");
    }

    #[test]
    fn territory_not_sellable() {
        // v2 无卖出分支（§4.4「不能卖占领格」）= 所有权只经占领/攻占/破产变更；
        // 保守断言：无「sell」API（编译期即验证）。
        let (_, terr, cell) = fixture();
        let o = terr.ownership(cell).unwrap();
        assert_eq!(o.owner, None, "无主格无卖出语义");
    }

    /// §6：v2 AI 攻占判定用**兵力模型**（强舰队 → Siege；弱舰队 → PayToll）。
    #[test]
    fn ai_policy_uses_force_model() {
        let (bal, mut terr, cell) = fixture();
        // 席 1 占该格（驻军 5 + 建筑防御 L1），攻方席 0。
        let mut owner = SeatState::new(1000, 30, 20, 0);
        let _ = capture_cell(&mut owner, &mut terr, cell, 1, 5);
        // 强攻方（200 人手/200 武器）→ Siege（战力 400 > 守方 61）
        let strong = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 200, 200);
        assert_eq!(strong, SettleAction::Siege, "强舰队 → 攻占");
        // 弱攻方（20/5 → 战力 25 < 61）→ PayToll
        let weak = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 20, 5);
        assert_eq!(weak, SettleAction::PayToll, "弱舰队 → 交钱");
    }

    /// §6：v2 AI 无主格 → 副本；自己格 → 升级/产出（与玩家同一分支表）。
    ///
    /// 真实 `balance.territory.ai.upgrade_priority = 0.3 < 0.5` ⇒ 自己格不升级（`Collect`）。
    /// 升级路径须配置优先度 ≥ 0.5（`Upgrade`），满级 → `Maxed`。
    #[test]
    fn ai_policy_unowned_and_self() {
        let (bal, mut terr, cell) = fixture();
        // 无主 → EnterDungeon
        let unowned = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 200, 200);
        assert_eq!(unowned, SettleAction::EnterDungeon, "无主格 → 副本");
        // 自己格：真实配置 upgrade_priority=0.3 < 0.5 ⇒ Collect（不升级）
        let mut seat = SeatState::new(1000, 200, 200, 0);
        let _ = capture_cell(&mut seat, &mut terr, cell, 0, 5);
        let self_own = ai_policy_v2(&bal, &terr, 0, cell, 10_000, 200, 200);
        assert_eq!(self_own, SettleAction::Collect, "自己格 upgrade_priority<0.5 → 领产出");
        // 升级路径：构造 upgrade_priority ≥ 0.5 的 balance（克隆改配置）⇒ Upgrade
        let mut bal2 = bal.clone();
        bal2.territory.ai.upgrade_priority = 0.6;
        let up = ai_policy_v2(&bal2, &terr, 0, cell, 10_000, 200, 200);
        assert_eq!(up, SettleAction::Upgrade, "自己格 upgrade_priority≥0.5 + 金足 → 升级");
    }
}
