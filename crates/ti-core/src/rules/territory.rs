//! 领地系统规则（S28.1；业务单一事实源 `docs/business/territory.md`）。
//!
//! - 席位模型（§1）：5 角色席位 = 玩家 ≥1 + 其余 AI；席位顺序 = 角色 index 升序；
//! - 领地归属（§3/§4）：每类岛两格同属一个领地；落格三支分叉
//!   （无主 → 副本占领 / 他人 → 交钱或攻占 / 自己 → 升级加固）；
//! - 公式（§4.1，数值全走 `balance.territory`）：`toll/upgrade_cost/production/defense`
//!   均 = 基值 ×（1 + 每级系数 × 等级）；
//! - AI 策略（§6）：AI 与玩家共用同一分支表（[`TerritoryRules::settle`] +
//!   [`TerritoryApply`]），策略只决定「选哪个分支」（[`TerritoryRules::ai_policy`]），
//!   不得读玩家内存。
//!
//! 结构：[`TerritoryRules`] = 纯公式（借 `&Balance`，无状态）；[`TerritoryState`] =
//! 归属账本（格序号 → 归属记录 + 岛 → 格映射）；结算结果由调用方（游戏层）扣/加金
//! 并经 [`TerritoryApply`] 回填账本。确定性：纯函数 + 数据表，无随机。

use std::collections::BTreeMap;

use crate::data::{Balance, BoardMap};

/// 席位下标（`0..=4`，共 5 席；`docs/business/territory.md` §1）。
pub type SeatIdx = u8;

/// 席位（5 角色之一；玩家席 / AI 席）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seat {
    /// 角色标识名（`panda`/`tichy`/`kidd`/`drake`/`katharina`，`balance.selection.characters[].id`）。
    pub character: &'static str,
    /// 角色目录 index（0 起；席位顺序 = index 升序）。
    pub index: u32,
    /// 玩家席位（`false` = AI 席，自动行动）。
    pub is_player: bool,
}

/// 席位池（`players + ai = 5`，`players ≥ 1`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatPool {
    seats: [Seat; 5],
}

impl SeatPool {
    /// 由「玩家选中的角色 index 列表（≥1）」构建席位池（§1 不变量）。
    ///
    /// # Panics
    /// 玩家列表为空 / 角色 index 越界 / 重复时 panic（选角界面只产出合法 0..=4 列表；
    /// 本层是规则层不变量兜底，测试面以合法输入为主）。
    #[must_use]
    pub fn build(player_indexes: &[u32]) -> Self {
        const ROLES: [(&str, u32); 5] =
            [("panda", 0), ("tichy", 1), ("kidd", 2), ("drake", 3), ("katharina", 4)];
        let mut chosen = [false; 5];
        assert!(!player_indexes.is_empty(), "at least 1 player seat (territory.md S1)");
        for &idx in player_indexes {
            assert!((0..5).contains(&idx), "character index out of range: {idx}");
            assert!(!chosen[idx as usize], "duplicate player seat: {idx}");
            chosen[idx as usize] = true;
        }
        let seats = ROLES.map(|(character, index)| Seat {
            character,
            index,
            is_player: chosen[index as usize],
        });
        Self { seats }
    }

    /// 全部席位（index 升序 = 回合顺序）。
    pub fn iter(&self) -> impl Iterator<Item = &Seat> {
        self.seats.iter()
    }

    /// 席位总数（恒 5）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.seats.len()
    }

    /// 是否空（恒 `false`：席位池固定 5 席）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seats.is_empty()
    }

    /// 玩家席位数（`1..=5`）。
    #[must_use]
    pub fn player_count(&self) -> usize {
        self.seats.iter().filter(|s| s.is_player).count()
    }

    /// AI 席位数（`0..=4`）。
    #[must_use]
    pub fn ai_count(&self) -> usize {
        self.len() - self.player_count()
    }

    /// 下标 → 席位（越界 `None`）。
    #[must_use]
    pub fn get(&self, idx: SeatIdx) -> Option<&Seat> {
        self.seats.get(idx as usize)
    }

    /// 玩家席位下标（热座交接用；升序）。
    #[must_use]
    pub fn player_seats(&self) -> Vec<SeatIdx> {
        (0..5).filter(|i| self.seats[*i as usize].is_player).collect()
    }
}

/// 一格领地的归属记录（无主 = `owner = None`；`level` 恒 `1..=上限`，无主时 `0`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ownership {
    /// 占领席位（`None` = 无主）。
    pub owner: Option<SeatIdx>,
    /// 加固等级（`0` = 无主 / 未加固；`1..=上限` = 已加固）。
    pub level: u8,
    /// 驻军人手（v2 §4.1：占领者自定，从席位人手池分出；`0` = 无驻军）。
    pub garrison: u32,
}

impl Ownership {
    /// 无主初始态。
    #[must_use]
    pub const fn unowned() -> Self {
        Self { owner: None, level: 0, garrison: 0 }
    }

    /// 占领态（v2：驻军由占领者自定）。
    #[must_use]
    pub const fn occupied(owner: SeatIdx, level: u8, garrison: u32) -> Self {
        Self { owner: Some(owner), level, garrison }
    }
}

/// 领地账本（一张赛道图的归属表：格序号 → 归属记录 + 岛原型 → 格索引映射）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerritoryState {
    cells: BTreeMap<u32, Ownership>,
    isle_map: BTreeMap<String, Vec<u32>>,
}

impl TerritoryState {
    /// 由赛道数据构建初始账本（所有领地格无主；序号 = `board.json` 0 起）。
    ///
    /// # Errors
    /// 赛道缺领地格 / 岛 id 未在 `balance.territory.prototypes` 注册时返回错误（数据硬失败）。
    pub fn new(map: &BoardMap, balance: &Balance) -> Result<Self, String> {
        let mut cells = BTreeMap::new();
        let mut isle_map: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for (i, s) in map.spaces.iter().enumerate() {
            let Some(isle) = s.isle.as_deref() else {
                continue;
            };
            if !balance.territory.prototypes.iter().any(|p| p.id == isle) {
                return Err(format!(
                    "territory: board spaces[{i}] isle `{isle}` not registered in balance.territory.prototypes"
                ));
            }
            let idx = u32::try_from(i).unwrap_or(u32::MAX);
            cells.insert(idx, Ownership::unowned());
            isle_map.entry(isle.to_string()).or_default().push(idx);
        }
        if cells.is_empty() {
            return Err("territory: no territory cells on track (board.json lacks isle metadata)"
                .to_string());
        }
        Ok(Self { cells, isle_map })
    }

    /// 全部领地格归属（按格序号升序）。
    pub fn cells(&self) -> impl Iterator<Item = &Ownership> {
        self.cells.values()
    }

    /// 格序号 → 归属（非领地格 = `None`）。
    #[must_use]
    pub fn ownership(&self, index: u32) -> Option<&Ownership> {
        self.cells.get(&index)
    }

    /// 与某岛同类的全部格序号（§3：两格同属一个领地；确定性升序；未知岛 = 空）。
    #[must_use]
    pub fn isle_cells(&self, isle: &str) -> &[u32] {
        self.isle_map.get(isle).map_or(&[], Vec::as_slice)
    }

    /// 某席位占有的全部格序号（破产清空 / 存档导出用；升序）。
    #[must_use]
    pub fn owned_indices(&self, seat: SeatIdx) -> Vec<u32> {
        self.cells.iter().filter(|(_, o)| o.owner == Some(seat)).map(|(k, _)| *k).collect()
    }

    /// 某席位驻军总量（§4.1 守恒：每**领地**（岛）只计一次驻军，两格同领地不重复计数）。
    #[must_use]
    pub fn garrison_total(&self, seat: SeatIdx) -> u32 {
        self.isle_map
            .iter()
            .filter(|(_, cells)| {
                cells.iter().any(|c| self.cells.get(c).and_then(|o| o.owner) == Some(seat))
            })
            .map(|(_, cells)| {
                cells.first().and_then(|c| self.cells.get(c)).map_or(0, |o| o.garrison)
            })
            .sum()
    }

    /// 全部占领格 → 驻军映射（存档 v5 `garrisons`；无主格不写）。
    #[must_use]
    pub fn garrison_map(&self) -> BTreeMap<u32, u32> {
        self.cells
            .iter()
            .filter(|(_, o)| o.owner.is_some())
            .map(|(k, o)| (*k, o.garrison))
            .collect()
    }

    /// 第 `idx` 格所属岛原型（领地格 = `Some`；功能格 / 起点 = `None`）。
    #[must_use]
    pub fn isle_cells_by_index(&self, idx: u32) -> Option<&str> {
        self.isle_map.iter().find(|(_, cells)| cells.contains(&idx)).map(|(isle, _)| isle.as_str())
    }

    /// 改某格归属记录（格不在账本 = 非领地格，静默忽略）。
    pub fn mutate(&mut self, index: u32, own: Ownership) {
        if self.cells.contains_key(&index) {
            self.cells.insert(index, own);
        }
    }

    /// 序列化快照（存档 v4 `territories`：格序号 → `{owner_seat, level}`，无主 = 缺省不写）。
    #[must_use]
    pub fn snapshot(&self) -> BTreeMap<u32, (Option<u8>, u8)> {
        self.cells.iter().map(|(k, o)| (*k, (o.owner, o.level))).collect()
    }

    /// 从存档快照还原（格须在该图赛道上，越界序号静默丢弃，读档宽松口径）。
    ///
    /// # Errors
    /// 同 [`Self::new`]（赛道缺领地格 / 岛 id 未注册）。
    pub fn restore(
        map: &BoardMap,
        balance: &Balance,
        snapshot: BTreeMap<u32, (Option<u8>, u8)>,
    ) -> Result<Self, String> {
        let mut state = Self::new(map, balance)?;
        for (k, (owner, level)) in snapshot {
            if state.cells.contains_key(&k) {
                state.cells.insert(k, Ownership { owner, level, garrison: 0 });
            }
        }
        Ok(state)
    }
}

/// 落格结算分支（§4 三支分叉）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettleAction {
    /// 无主格：进入副本（胜 ⇒ 占领）。
    EnterDungeon,
    /// 他人格：交过路费（付 `toll`，本圈产出归所有者）。
    PayToll,
    /// 他人格：攻占（战斗；胜 ⇒ 夺回降级 1）。
    Siege,
    /// 自己格：升级加固（付 `cost`，等级 +1，≤ 上限）。
    Upgrade,
    /// 自己格：领取本圈产出。
    Collect,
    /// 自己格：已在上限（不可升级；仍可领产出）。
    Maxed,
}

/// 结算结果（金额；账本变更由调用方经 [`TerritoryApply`] 回填）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettleResult {
    /// 行动分支。
    pub action: SettleAction,
    /// 本次应付金币（过路费 / 升级费；从行动者金币扣，过路费给所有者）。
    pub gold_out: u32,
    /// 本次所得金币（产出领取；0 = 无；首占奖励另计于 [`TerritoryApply::apply_dungeon`] 返回）。
    pub gold_in: u32,
}

/// 副本结果（§3：副本胜利 = 占领，失败 = 归属不变）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonOutcome {
    /// 副本胜利（占领该岛两格）。
    Win,
    /// 副本失败（归属不变，仅战斗损耗由调用方结算）。
    Loss,
}

/// 攻占结果（§4：攻占胜利 = 夺回且等级降 1；败 = 归属不变）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiegeOutcome {
    /// 攻占胜利（夺回该岛两格，等级重置 1）。
    Win,
    /// 攻占失败（归属不变）。
    Loss,
}

/// 公式族四舍五入取整（非负有限值钳制 `u32::MAX`；clippy pedantic 的 f64 转整在此统一
/// `allow`，口径同 `rules::battle` `cast_atk_base`）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn round_to_u32(value: f64) -> u32 {
    let value = value.round();
    if !value.is_finite() || value < 0.0 {
        return 0;
    }
    u32::try_from(value.min(f64::from(u32::MAX)) as u64).unwrap_or(u32::MAX)
}

/// 落格结算公式（§4.1；纯函数，数值全走 `balance.territory`）。
///
/// 与 [`TerritoryState`] 分离：规则公式无状态（借 `&Balance`），账本独立可变，
/// 避免「结算器借账本 + 账本 apply 需 &mut」的借用冲突（AI 与玩家走同一实例）。
pub struct TerritoryRules<'a> {
    balance: &'a Balance,
}

impl<'a> TerritoryRules<'a> {
    /// 构建结算器（`&Balance` = `balance()` 缓存入口返回的静态引用）。
    #[must_use]
    pub fn new(balance: &'a Balance) -> Self {
        Self { balance }
    }

    /// 岛原型配置（未知 id = `None`，调用方视为功能格）。
    #[must_use]
    pub fn proto(&self, isle: &str) -> Option<&crate::data::TerritoryPrototypeCfg> {
        self.balance.territory.prototypes.iter().find(|p| p.id == isle)
    }

    /// 某岛某等级的过路费（§4.1：`toll_base × (1 + toll_per_level × L)`）。
    #[must_use]
    pub fn toll_at(&self, isle: &str, level: u8) -> u32 {
        self.formula(isle, |p| p.toll_base, self.balance.territory.toll_per_level, level)
    }

    /// 某岛从 `level` 升一级的加固费（§4.1：`upgrade_cost_base × (1 + 系数 × L)`）。
    #[must_use]
    pub fn upgrade_cost_at(&self, isle: &str, level: u8) -> u32 {
        self.formula(
            isle,
            |p| p.upgrade_cost_base,
            self.balance.territory.upgrade_cost_per_level,
            level,
        )
    }

    /// 某岛某等级的每圈产出（§4.1）。
    #[must_use]
    pub fn production_at(&self, isle: &str, level: u8) -> u32 {
        self.formula(
            isle,
            |p| p.production_base,
            self.balance.territory.production_per_level,
            level,
        )
    }

    /// 某岛某等级的守军强度（§4.1；副本 / 攻占战的 HP 基线）。
    #[must_use]
    pub fn defense_at(&self, isle: &str, level: u8) -> u32 {
        self.formula(isle, |p| p.defense_base, self.balance.territory.defense_per_level, level)
    }

    /// 某岛等级上限（§3 表按原型可降，且 ≤ 全局 `max_level`；未知岛 = 全局上限）。
    #[must_use]
    pub fn level_cap(&self, isle: &str) -> u8 {
        let global = u8::try_from(self.balance.territory.max_level.min(u32::from(u8::MAX)))
            .unwrap_or(u8::MAX);
        let cap = match self.proto(isle) {
            Some(p) => u8::try_from(p.max_level.max(1)).unwrap_or(u8::MAX),
            None => global,
        };
        cap.min(global)
    }

    /// 首占奖励（§7：占领成功 = 归属变更 + 该类首占奖励；仅首占给奖）。
    #[must_use]
    pub fn first_capture_bonus(&self, was_unowned: bool) -> u32 {
        if was_unowned { self.balance.territory.first_capture_bonus } else { 0 }
    }

    /// §4 落格结算：给定落格 `idx` + 行动席位 `seat` + 行动者金币 `gold` + 分支，
    /// 返回金额（账本变更由调用方按 `action` 回填 [`TerritoryApply`]）。
    /// 非法分支（如对他人的格 Collect / 非所有者 Upgrade）金额归零，分支值不变。
    #[must_use]
    pub fn settle(
        &self,
        state: &TerritoryState,
        idx: u32,
        seat: SeatIdx,
        gold: u32,
        action: SettleAction,
    ) -> SettleResult {
        let isle = state.isle_cells_by_index(idx).unwrap_or("");
        match action {
            SettleAction::EnterDungeon | SettleAction::Siege => {
                SettleResult { action, gold_out: 0, gold_in: 0 }
            }
            SettleAction::PayToll => {
                let own = state.ownership(idx).copied().unwrap_or(Ownership::unowned());
                let toll = self.toll_at(isle, own.level.max(1)).min(gold);
                SettleResult { action, gold_out: toll, gold_in: 0 }
            }
            SettleAction::Upgrade => {
                let own = state.ownership(idx).copied().unwrap_or(Ownership::unowned());
                // 只有所有者可升级（§4 自己格分支）。
                let cost = if own.owner == Some(seat) {
                    self.upgrade_cost_at(isle, own.level.max(1)).min(gold)
                } else {
                    0
                };
                SettleResult { action, gold_out: cost, gold_in: 0 }
            }
            SettleAction::Collect => {
                let own = state.ownership(idx).copied().unwrap_or(Ownership::unowned());
                // 只有所有者可领产出（§4.2：交钱者本圈无收益；他人格 Collect 归零）。
                let income = if own.owner == Some(seat) {
                    self.production_at(isle, own.level.max(1))
                } else {
                    0
                };
                SettleResult { action, gold_out: 0, gold_in: income }
            }
            SettleAction::Maxed => SettleResult { action, gold_out: 0, gold_in: 0 },
        }
    }

    /// AI 策略（§6：只决定「选哪个分支」，走同一结算表；入参 = AI 席可见信息，
    /// 禁止读其他玩家私有资源）。
    #[must_use]
    pub fn ai_policy(
        &self,
        state: &TerritoryState,
        idx: u32,
        seat: SeatIdx,
        gold: u32,
    ) -> SettleAction {
        let cfg = &self.balance.territory;
        let own = state.ownership(idx).copied().unwrap_or(Ownership::unowned());
        match own.owner {
            None => SettleAction::EnterDungeon,
            Some(o) if o != seat => {
                // 他人格：守军 < 攻占阈值 × 守军基值 ⇒ 攻占；金币不足或交钱偏好 ≥0.5 ⇒ 交钱。
                let isle = state.isle_cells_by_index(idx).unwrap_or("");
                let defense = self.defense_at(isle, own.level.max(1));
                let base = self.proto(isle).map_or(0, |p| p.defense_base);
                let threshold = round_to_u32(f64::from(cfg.ai.attack_threshold) * f64::from(base));
                let affordable = gold >= self.toll_at(isle, own.level.max(1));
                if defense < threshold.max(base) {
                    SettleAction::Siege
                } else if !affordable || cfg.ai.toll_aversion >= 0.5 {
                    SettleAction::PayToll
                } else {
                    SettleAction::Siege
                }
            }
            Some(_) => {
                // 自己格：未满级 + 金币够升级费 + 升级优先度 ≥0.5 ⇒ 升级；否则领产出；满级 = Maxed。
                let isle = state.isle_cells_by_index(idx).unwrap_or("");
                let cap = self.level_cap(isle);
                let cost = self.upgrade_cost_at(isle, own.level.max(1));
                if own.level >= cap {
                    SettleAction::Maxed
                } else if gold >= cost && cfg.ai.upgrade_priority >= 0.5 {
                    SettleAction::Upgrade
                } else {
                    SettleAction::Collect
                }
            }
        }
    }

    /// 公式族（§4.1）：`base × (1 + coef × L)`；未知岛 = 0（调用方先判 [`Self::proto`]）。
    fn formula(
        &self,
        isle: &str,
        base: impl Fn(&crate::data::TerritoryPrototypeCfg) -> u32,
        coef: f32,
        level: u8,
    ) -> u32 {
        match self.proto(isle) {
            None => 0,
            Some(p) => {
                round_to_u32(f64::from(base(p)) * (1.0 + f64::from(coef) * f64::from(level)))
            }
        }
    }
}

/// 账本变更操作（把结算结果回填 [`TerritoryState`]；游戏层按 `apply_*` 返回值扣/加金）。
pub trait TerritoryApply {
    /// 副本结算（§3）：`Win` ⇒ 该岛两格归 `seat`（`level = 1`）；返回首占奖励（原无主时）。
    fn apply_dungeon(
        &mut self,
        rules: &TerritoryRules<'_>,
        idx: u32,
        seat: SeatIdx,
        outcome: DungeonOutcome,
    ) -> u32;

    /// 攻占结算（§4）：`Win` ⇒ 夺回该岛两格（`level = 1` 降级）；`Loss` ⇒ 不变。
    fn apply_siege(
        &mut self,
        rules: &TerritoryRules<'_>,
        idx: u32,
        seat: SeatIdx,
        outcome: SiegeOutcome,
    );

    /// 交钱结算（§4.2）：返回应付过路费（调用方扣行动者金币、加所有者金币；
    /// 本圈该格产出归所有者，交钱者不再领产出）。
    fn apply_pay_toll(&mut self, rules: &TerritoryRules<'_>, idx: u32, seat: SeatIdx) -> u32;

    /// 升级结算（§4）：`seat` 付 `cost`（调用方扣金币）⇒ 该岛两格 `level + 1`（≤ 上限）；
    /// 返回是否成功（满级 / 非所有者 = `false`）。
    fn apply_upgrade(&mut self, rules: &TerritoryRules<'_>, idx: u32, seat: SeatIdx) -> bool;

    /// 领取产出（§4）：返回 `seat` 获得的该格本圈产出（调用方加金币；非所有者 = 0）。
    fn apply_collect(&mut self, rules: &TerritoryRules<'_>, idx: u32, seat: SeatIdx) -> u32;
}

/// 副本 / 攻占守军单位（`territory.md` §9：副本守军战 / 攻占战 / BOSS 战共用同一战斗模型；
/// 守军 HP = [`TerritoryRules::defense_at`]，ATK = 守军强度 × `dungeon_attack_coef`）。
impl TerritoryRules<'_> {
    /// 守军 [`Unit`](crate::domain::combat::Unit)（战斗层 `BattleSession::new_with_enemy` 注入）。
    #[must_use]
    pub fn defender_unit(&self, isle: &str, level: u8) -> crate::domain::combat::Unit {
        let defense = self.defense_at(isle, level);
        let coef = self.balance.territory.dungeon_attack_coef;
        let atk = round_to_u32_f32(f64::from(defense) * f64::from(coef));
        crate::domain::combat::Unit { hp: defense, atk }
    }
}

/// 非负值 × 系数取整（`f32` 系数；口径同 [`round_to_u32`]）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn round_to_u32_f32(value: f64) -> u32 {
    let v = value.round().clamp(0.0, f64::from(u32::MAX));
    u32::try_from(v as u64).unwrap_or(u32::MAX)
}

impl TerritoryApply for TerritoryState {
    fn apply_dungeon(
        &mut self,
        rules: &TerritoryRules<'_>,
        idx: u32,
        seat: SeatIdx,
        outcome: DungeonOutcome,
    ) -> u32 {
        if outcome != DungeonOutcome::Win {
            return 0;
        }
        let isle = self.isle_cells_by_index(idx).unwrap_or("");
        let was_unowned = self.ownership(idx).and_then(|o| o.owner).is_none();
        let bonus = rules.first_capture_bonus(was_unowned);
        let cells: Vec<u32> = self.isle_cells(isle).to_vec();
        for cell in cells {
            self.mutate(cell, Ownership { owner: Some(seat), level: 1, garrison: 0 });
        }
        bonus
    }

    fn apply_siege(
        &mut self,
        _rules: &TerritoryRules<'_>,
        idx: u32,
        seat: SeatIdx,
        outcome: SiegeOutcome,
    ) {
        if outcome != SiegeOutcome::Win {
            return;
        }
        let isle = self.isle_cells_by_index(idx).unwrap_or("");
        let cells: Vec<u32> = self.isle_cells(isle).to_vec();
        for cell in cells {
            self.mutate(cell, Ownership { owner: Some(seat), level: 1, garrison: 0 });
        }
    }

    fn apply_pay_toll(&mut self, rules: &TerritoryRules<'_>, idx: u32, seat: SeatIdx) -> u32 {
        let isle = self.isle_cells_by_index(idx).unwrap_or("");
        let own = self.ownership(idx).copied().unwrap_or(Ownership::unowned());
        if own.owner == Some(seat) {
            return 0;
        }
        rules.toll_at(isle, own.level.max(1))
    }

    fn apply_upgrade(&mut self, rules: &TerritoryRules<'_>, idx: u32, seat: SeatIdx) -> bool {
        let isle = self.isle_cells_by_index(idx).unwrap_or("");
        let own = self.ownership(idx).copied().unwrap_or(Ownership::unowned());
        if own.owner != Some(seat) {
            return false;
        }
        let cap = rules.level_cap(isle);
        if own.level >= cap {
            return false;
        }
        let cells: Vec<u32> = self.isle_cells(isle).to_vec();
        for cell in cells {
            self.mutate(
                cell,
                Ownership { owner: Some(seat), level: own.level + 1, garrison: own.garrison },
            );
        }
        true
    }

    fn apply_collect(&mut self, rules: &TerritoryRules<'_>, idx: u32, seat: SeatIdx) -> u32 {
        let isle = self.isle_cells_by_index(idx).unwrap_or("");
        let own = self.ownership(idx).copied().unwrap_or(Ownership::unowned());
        if own.owner != Some(seat) {
            return 0;
        }
        rules.production_at(isle, own.level.max(1))
    }
}

// ------------------------------------------------------- S28.7 AI 回合（§6：AI 与玩家同规则）

/// 席位轮转（§1 回合顺序 = 角色 index 升序；5 席循环）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TurnCycle {
    /// 当前轮到的席位。
    pub current: SeatIdx,
    /// 玩家席位（热座交接：轮到玩家席 = 需要「开始回合」确认；AI 席 = 自动行动）。
    players: [bool; 5],
}

impl TurnCycle {
    /// 由席位池构建（`players[i]` = 席 i 是否玩家席；池顺序 = index 升序）。
    #[must_use]
    pub fn new(pool: &SeatPool) -> Self {
        let mut players = [false; 5];
        for (idx, seat) in pool.iter().enumerate() {
            if seat.is_player {
                players[idx] = true;
            }
        }
        Self { current: 0, players }
    }

    /// 本席是否玩家席（`false` = AI 自动回合）。
    #[must_use]
    pub fn is_player_seat(self, seat: SeatIdx) -> bool {
        self.players[seat as usize]
    }

    /// 推进到下一席位（§1：5 席循环，`players + ai = 5`）。
    pub fn advance(&mut self) {
        self.current = (self.current + 1) % 5;
    }

    /// 下一个**玩家**席位（热座交接：从当前席后找首个玩家席，循环不越 5 次）。
    #[must_use]
    pub fn next_player_seat(self) -> SeatIdx {
        let mut seat = self.current;
        for _ in 0..5 {
            seat = (seat + 1) % 5;
            if self.players[seat as usize] {
                return seat;
            }
        }
        0
    }
}

/// AI 回合结果（§6：AI 走同一结算表；策略只选分支，不得读玩家私有状态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiTurn {
    /// 行动席位（必为 AI 席）。
    pub seat: SeatIdx,
    /// 落格（摇骰前进后；前进由游戏层 `BoardSession` 完成，本层只选分支）。
    pub cell: u32,
    /// 策略选出的分支（= [`TerritoryRules::ai_policy`]，玩家同表）。
    pub action: SettleAction,
    /// 结算金额（= [`TerritoryRules::settle`] 同函数结果）。
    pub result: SettleResult,
}

impl TerritoryRules<'_> {
    /// AI 自动回合（§6）：给定 AI 席落格 + 自身金币，返回「分支 + 金额」（与玩家共用
    /// [`Self::ai_policy`] + [`Self::settle`]，不变量 = 同状态同分支同结果）。
    #[must_use]
    pub fn ai_turn(&self, state: &TerritoryState, seat: SeatIdx, cell: u32, gold: u32) -> AiTurn {
        let action = self.ai_policy(state, cell, seat, gold);
        let result = self.settle(state, cell, seat, gold, action);
        AiTurn { seat, cell, action, result }
    }
}
