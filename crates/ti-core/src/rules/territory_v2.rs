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
    let res = force::resolve(attacker, defender, balance.manpower);
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
///
/// `#[deprecated]`（S52.3）：v2 全 5 席扫描，v3 席位池 2..5 席时有「不在局席永远存活」
/// 缺陷（D4）；0 生产调用方。现役 = [`victory_seat_active`]（按在局掩码判）。
#[deprecated(
    since = "0.2.0",
    note = "v3 在局掩码口径 = `victory_seat_active`；v2 全席扫描仅留作语义参照"
)]
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

/// v3：胜利判定（§1.2 / §4.4）——**只在「在局席位」范围内**判最后一个存活者。
///
/// v2 的 [`victory_seat`] 扫全部 5 席；v3 席位池可只放 2..5 席，其余槽位**不在局**
/// （`alive = true` 但从未参与）⇒ 必须按在局掩码过滤，否则「不在局的席位」会
/// 永远存活 ⇒ 永远判不出胜者（对局打不完，v2 缺陷 D4 的另一半）。
///
/// `active` = 在局席位掩码（长度 = `seats` 长度；越界视为不在局）。
#[must_use]
pub fn victory_seat_active(seats: &[SeatState], active: &[bool]) -> Option<SeatIdx> {
    let alive: Vec<SeatIdx> = seats
        .iter()
        .enumerate()
        .filter(|(i, s)| s.alive && active.get(*i).copied().unwrap_or(false))
        .map(|(i, _)| u8::try_from(i).unwrap_or(u8::MAX))
        .collect();
    match alive.as_slice() {
        [single] => Some(*single),
        _ => None,
    }
}

/// 失败判定（§4.4）：玩家席破产 ⇒ 直接结束（败）。
///
/// `#[deprecated]`（S52.3）：单玩家 index 口径，v3 热座多玩家 = [`players_defeated`]
/// （全部玩家席出局才判败）；0 生产调用方，仅留作语义参照。
#[deprecated(
    since = "0.2.0",
    note = "v3 多玩家席口径 = `players_defeated`；单 index 口径仅留作语义参照"
)]
#[must_use]
pub fn player_defeated(seats: &[SeatState], player_idx: SeatIdx) -> bool {
    seats.get(player_idx as usize).is_some_and(|s| !s.alive)
}

/// v3：失败判定（§1.2）——**全部人类席**都破产 ⇒ 玩家失败。
///
/// `player_seats` = 人类席下标（热座可能有多个人类席）；空列表 = 无人类席 ⇒ 不判失败。
#[must_use]
pub fn players_defeated(seats: &[SeatState], player_seats: &[SeatIdx]) -> bool {
    !player_seats.is_empty()
        && player_seats.iter().all(|&s| seats.get(s as usize).is_some_and(|seat| !seat.alive))
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
            let will_win =
                force::resolve(attacker, defender, balance.manpower).winner == ForceSide::Attacker;
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
///
/// `#[deprecated]`（S52.3）：纯薄包装（本函数体 = 直调 [`ai_policy_v2`]，无独有逻辑）；
/// 0 生产调用方。现役 AI 决策入口 = `ai_policy_v2`。
#[deprecated(
    since = "0.2.0",
    note = "薄包装无独有逻辑，现役入口 = `ai_policy_v2`；仅留作 v2 语义参照"
)]
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
