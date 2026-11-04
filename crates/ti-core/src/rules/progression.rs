//! 跨图推进规则（`docs/business/progression.md`「跨图推进」，唯一事实源）。
//!
//! 设计：20 图 m01~m20 难度按 `boss_tier` 升序（0→19）。
//! **击败当前图 BOSS 才解锁下一图**；m01 恒解锁；击败 m20 = 通关。失败不解锁（可重试当前图）。
//! 玩法层纯规则，无渲染/无 IO；`unlocked_maps` 的持久化/读档由 `persistence` 负责。
//!
//! S37（2026-11-03）：从 3 图扩到 20 图。`unlocked_mask` 从 `u8` 升到 `u32`
//! （bit 0..=18 = 第 1..=19 张图是否已解锁；bit 19 = 通关位 m20 BOSS 已击败）。
//! 旧档 `u8` 掩码读档时扩展为 `u32`（高 24 位 = 0，兼容 3 图旧档）。

use crate::domain::selection::MapSlot;

/// 图推进序列（按 `boss_tier` 升序：m01→m02→…→m20，末位无下一图）。
pub const MAP_SEQUENCE: [MapSlot; 20] = [
    MapSlot::M01,
    MapSlot::M02,
    MapSlot::M03,
    MapSlot::M04,
    MapSlot::M05,
    MapSlot::M06,
    MapSlot::M07,
    MapSlot::M08,
    MapSlot::M09,
    MapSlot::M10,
    MapSlot::M11,
    MapSlot::M12,
    MapSlot::M13,
    MapSlot::M14,
    MapSlot::M15,
    MapSlot::M16,
    MapSlot::M17,
    MapSlot::M18,
    MapSlot::M19,
    MapSlot::M20,
];

/// 地图的 BOSS 难度档位（`balance.selection.maps[].boss_tier`，0 起）。
///
/// 业务依据（`docs/business/progression.md` §跨图难度阶梯）：地图 index 越大 ⇒ tier 越大
/// ⇒ BOSS 越强（强度表 `balance.combat.boss_tiers[tier]` = HP/ATK）。
///
/// 查表失败（balance 不可用 / 目录缺该图）= `None`（**不 fallback 到猜值**）。
#[must_use]
pub fn boss_tier_of(m: MapSlot) -> Option<u32> {
    let b = crate::data::balance().ok()?;
    b.selection.maps.iter().find(|def| def.id == m.id()).map(|def| def.boss_tier)
}

/// 解锁掩码里的**通关位**（bit 19）：m20 BOSS 已击败（通关不新增解锁项，须单独 1 位）。
/// S37：`u32` 掩码（20 图 = bit 0..=18 解锁 + bit 19 通关）。
pub const CLEARED_BIT: u32 = 1u32 << 19;

/// 解锁掩码位宽（S37：20 图 + 1 通关位 = 20 bit）。
pub const MASK_BITS: u32 = 20;

/// 跨图推进状态：记录每张图（按 `MapSlot::all()` 顺序）是否已击败其 BOSS。
///
/// 解锁链单调：`m(i) 已解锁 ⟺ i==0 || defeated[i-1]`（m01 恒解锁，m02 需击败 m01，… m20 需击败 m19）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProgressionState {
    /// 每张图 BOSS 是否已击败（`[m01, m02, …, m20]`）。
    defeated: [bool; 20],
}

impl ProgressionState {
    /// 初始推进状态：无任何图被击败 → 仅 m01 解锁。
    #[must_use]
    pub const fn initial() -> Self {
        Self { defeated: [false; 20] }
    }

    /// 初始解锁集合（= `ProgressionState::initial().unlocked()`，即 `[m01]`）。
    #[must_use]
    pub fn initial_unlocked() -> Vec<MapSlot> {
        Self::initial().unlocked()
    }

    /// 当前解锁集合（m01 恒解锁；m02 需击败 m01；… m20 需击败 m19）。
    #[must_use]
    pub fn unlocked(self) -> Vec<MapSlot> {
        MapSlot::all().iter().copied().filter(|m| self.can_select(*m)).collect()
    }

    /// 某图是否已解锁（可选中）。
    #[must_use]
    pub fn can_select(self, m: MapSlot) -> bool {
        let i = usize::try_from(m.zero_index()).unwrap_or(0);
        i == 0 || self.defeated.get(i.saturating_sub(1)).copied().unwrap_or(false)
    }

    /// 标记某图 BOSS 已击败（推进）；**按值消费 `self`，返回新状态**（功能式，纯）。
    ///
    /// 调用方：`s = s.on_boss_defeated(m); if s.is_cleared() { … }`。幂等（重复标记同一图不变链）。
    #[must_use]
    pub fn on_boss_defeated(mut self, m: MapSlot) -> Self {
        let i = usize::try_from(m.zero_index()).unwrap_or(0);
        if let Some(slot) = self.defeated.get_mut(i) {
            *slot = true;
        }
        self
    }

    /// 是否已通关（m20 的 BOSS 已击败）。
    #[must_use]
    pub fn is_cleared(self) -> bool {
        self.defeated.get(19).copied().unwrap_or(false)
    }

    /// 已击败图数（0..=20，进度对账用）。
    #[must_use]
    pub fn defeated_count(self) -> u32 {
        u32::try_from(self.defeated.iter().filter(|b| **b).count()).unwrap_or(u32::MAX)
    }

    /// 从「当前解锁集合」重建推进状态（app 层战斗结算 / 状态对账用）。
    ///
    /// 单调链口径：`defeated[j]`（`j < 19`）⟺ 解锁集合含 `MAP_SEQUENCE[j + 1]`
    /// （m(j+1) 已解锁 ⟺ m(j) 已击败）；`defeated[19]`（m20 通关标志）**不由解锁链
    /// 持久化**（击败 m20 不新增任何解锁项），默认 `false`。
    ///
    /// 对旧档的异常解锁集（如 `[m01, m03]` 独缺 m02）忠实重建、不强行修正
    /// （解锁集合以存档为事实源，规则只约束后续步进）。
    #[must_use]
    pub fn from_unlocked(unlocked: &[MapSlot]) -> Self {
        let mut defeated = [false; 20];
        for (j, seq_slot) in MAP_SEQUENCE.iter().skip(1).take(19).enumerate() {
            defeated[j] = unlocked.contains(seq_slot);
        }
        Self { defeated }
    }

    /// 使 `m` 可被选中的**最小**推进状态（读档恢复用）：选中 m(i) 要求
    /// m(0)..m(i-1) 全部已击败（单调链前置），m(i) 自身及后续无需已击败。
    #[must_use]
    pub fn min_progression_to_select(m: MapSlot) -> Self {
        let i = usize::try_from(m.zero_index()).unwrap_or(0);
        let mut s = Self::initial();
        for prev in MAP_SEQUENCE.iter().take(i) {
            s = s.on_boss_defeated(*prev);
        }
        s
    }

    /// 某图 BOSS 是否已击败（逐图门禁判据；`on_boss_defeated` 之后为真）。
    #[must_use]
    pub fn is_map_defeated(self, m: MapSlot) -> bool {
        let i = usize::try_from(m.zero_index()).unwrap_or(0);
        self.defeated.get(i).copied().unwrap_or(false)
    }

    /// 解锁位掩码（存档字段 `SaveSlot::unlocked_mask` 的唯一编解码口径，S15.2 / S37 升 `u32`）：
    /// - bit 0..=18 = 第 1..=19 张图是否**已解锁**（bit 0 恒 1：m01 恒解锁）；
    /// - bit 19 = **通关**（m20 BOSS 已击败，通关不新增解锁项，需单独 1 位）。
    #[must_use]
    pub fn unlocked_mask(self) -> u32 {
        let bits = MapSlot::all()
            .iter()
            .copied()
            .enumerate()
            .fold(0u32, |acc, (i, m)| acc | if self.can_select(m) { 1u32 << i } else { 0u32 });
        bits | if self.is_cleared() { CLEARED_BIT } else { 0u32 }
    }

    /// 从解锁位掩码重建推进状态（读档用）。映射与 [`Self::unlocked_mask`] 严格互逆：
    /// `defeated[j]`（j = 0..=18）= bit(j+1)（m(j+1) 已解锁 ⟺ m(j) 已击败），
    /// `defeated[19]`（通关）= bit 19。**忠实还原**，不修正异常掩码
    /// （如 `m02` 未解锁但 `m03` 已解锁 = 外部写入，解锁集合以存档为事实源）。
    ///
    /// `mask == 0`（旧档无此字段）时返回 `initial()`；调用方须用
    /// [`Self::for_save_restore`] 走「最小前置链」兜底。
    ///
    /// S37：`u32` 掩码；旧 `u8` 掩码读档时直接扩展为 `u32`（高 24 位 = 0，3 图旧档兼容）。
    #[must_use]
    pub fn from_mask(mask: u32) -> Self {
        let mut defeated = [false; 20];
        for (j, _) in MAP_SEQUENCE.iter().skip(1).enumerate().take(19) {
            defeated[j] = mask & (1u32 << (j + 1)) != 0;
        }
        defeated[19] = mask & CLEARED_BIT != 0;
        Self { defeated }
    }

    /// 读档重建解锁链：`mask` 非 0 用掩码忠实还原；`mask == 0`（旧档）退回
    /// 「选中 m(i) 的最小前置链」，保证旧档仍可回读到 `map_slot` 所在图。
    #[must_use]
    pub fn for_save_restore(mask: u32, map_slot: MapSlot) -> Self {
        if mask == 0 {
            return Self::min_progression_to_select(map_slot);
        }
        // 掩码必须至少含当前所在图，否则读档后当前图会被锁（档数据自相矛盾）→ 补齐最小链。
        let mut s = Self::from_mask(mask);
        if !s.can_select(map_slot) {
            s = Self::min_progression_to_select(map_slot);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 初始仅 m01 解锁。
    #[test]
    fn progression_initial_unlocks_only_m01() {
        let s = ProgressionState::initial();
        assert_eq!(s.unlocked(), vec![MapSlot::M01]);
        assert_eq!(ProgressionState::initial_unlocked(), vec![MapSlot::M01]);
        assert!(s.can_select(MapSlot::M01));
        assert!(!s.can_select(MapSlot::M02));
        assert!(!s.can_select(MapSlot::M20));
        assert!(!s.is_cleared());
    }

    /// 击败 m01 → 仅解锁 m02（m03~m20 仍锁）。
    #[test]
    fn progression_defeat_m01_unlocks_m02_only() {
        let s = ProgressionState::initial();
        let s = s.on_boss_defeated(MapSlot::M01);
        assert!(!s.is_cleared(), "击败 m01 非通关");
        assert_eq!(s.unlocked(), vec![MapSlot::M01, MapSlot::M02]);
        assert!(s.can_select(MapSlot::M02));
        assert!(!s.can_select(MapSlot::M03));
        assert!(!s.can_select(MapSlot::M20));
    }

    /// 击败 m02 → 解锁 m03。
    #[test]
    fn progression_defeat_m02_unlocks_m03() {
        let s = ProgressionState::initial();
        let s = s.on_boss_defeated(MapSlot::M01);
        let s = s.on_boss_defeated(MapSlot::M02);
        assert!(!s.is_cleared(), "击败 m02 非通关");
        assert_eq!(s.unlocked(), vec![MapSlot::M01, MapSlot::M02, MapSlot::M03]);
        assert!(s.can_select(MapSlot::M03));
    }

    /// 击败 m19 → 解锁 m20。
    #[test]
    fn progression_defeat_m19_unlocks_m20() {
        let s = ProgressionState::initial();
        let mut s = s;
        for i in 0..18 {
            if let Some(slot) = MAP_SEQUENCE.get(i) {
                s = s.on_boss_defeated(*slot);
            }
        }
        let s = s.on_boss_defeated(MAP_SEQUENCE[18]);
        assert!(!s.is_cleared(), "击败 m19 非通关");
        assert!(s.can_select(MapSlot::M20));
        assert_eq!(s.unlocked().len(), 20, "m01~m20 全解锁");
    }

    /// 击败 m20 = 通关、无下一图。
    #[test]
    fn progression_defeat_m20_clears_game() {
        let s = ProgressionState::initial();
        let mut s = s;
        for slot in &MAP_SEQUENCE[0..19] {
            s = s.on_boss_defeated(*slot);
        }
        let s = s.on_boss_defeated(MapSlot::M20);
        assert!(s.is_cleared(), "击败 m20 应判通关");
        assert_eq!(s.defeated_count(), 20);
    }

    /// 失败（未击败）不解锁：不调用 `on_boss_defeated` 则推进不变。
    #[test]
    fn progression_loss_does_not_unlock() {
        let s = ProgressionState::initial();
        assert_eq!(s.unlocked(), vec![MapSlot::M01]);
        assert!(!s.can_select(MapSlot::M02));
        assert!(!s.is_cleared());
    }

    /// 解锁链单调：只增不降，m01 恒解锁。
    #[test]
    fn progression_unlocks_monotonic() {
        let s = ProgressionState::initial();
        let prev = s.unlocked();
        let s1 = s.on_boss_defeated(MapSlot::M01);
        let after_m01 = s1.unlocked();
        let s2 = s1.on_boss_defeated(MapSlot::M02);
        let after_m02 = s2.unlocked();
        let s3 = s2.on_boss_defeated(MapSlot::M03);
        let after_m03 = s3.unlocked();
        assert_subset(&prev, &after_m01);
        assert_subset(&after_m01, &after_m02);
        assert_subset(&after_m02, &after_m03);
        assert!(s3.can_select(MapSlot::M01), "m01 恒解锁");
    }

    /// 辅助：断言 `sub ⊆ sup`。
    fn assert_subset(sub: &[MapSlot], sup: &[MapSlot]) {
        for x in sub {
            assert!(sup.contains(x), "{x:?} 应在 {sup:?} 中（解锁链单调不减）");
        }
    }

    /// `from_unlocked` + 步进：`[m01, m02, m03]`（= 已击败 m01+m02）再击败 m03 → m04 解锁。
    #[test]
    fn from_unlocked_feeds_boss_defeat_step() {
        let unlocked = vec![MapSlot::M01, MapSlot::M02, MapSlot::M03];
        let s = ProgressionState::from_unlocked(&unlocked);
        let s = s.on_boss_defeated(MapSlot::M03);
        assert!(s.can_select(MapSlot::M04), "击败 m03 后 m04 应解锁");
        assert!(!s.is_cleared(), "击败 m03 非通关（通关 = m20）");
    }

    /// 全解锁集重建后击败 m20 = 通关（app 层重复调用结算时状态稳定）。
    #[test]
    fn from_unlocked_all_then_m20_clears() {
        let all: Vec<MapSlot> = MapSlot::all().to_vec();
        let s = ProgressionState::from_unlocked(&all).on_boss_defeated(MapSlot::M20);
        assert!(s.is_cleared());
        assert_eq!(s.defeated_count(), 20);
        // 幂等：全解锁集上重复结算 m20 不变
        let again = ProgressionState::from_unlocked(&all).on_boss_defeated(MapSlot::M20);
        assert_eq!(s.unlocked(), again.unlocked());
    }

    /// 旧档异常解锁集（`[m01, m03]` 独缺 m02）忠实重建：不强行修正，
    /// 后续步进仍走规则（击败 m02 后补齐 m02 的解锁依赖不再缺失）。
    #[test]
    fn from_unlocked_legacy_anomaly_is_faithful() {
        let weird = vec![MapSlot::M01, MapSlot::M03];
        let s = ProgressionState::from_unlocked(&weird);
        assert!(s.can_select(MapSlot::M03), "旧档 m03 已解锁须保留");
        assert!(!s.can_select(MapSlot::M02), "旧档未解锁 m02 不臆造解锁");
        assert_eq!(s.unlocked(), weird, "重建的解锁集合 = 原集合（忠实不修正）");
    }

    /// `min_progression_to_select`：选中 m(i) 的最小前置 = 击败 m(0)..m(i-1)。
    #[test]
    fn min_progression_to_select_chain() {
        assert_eq!(
            ProgressionState::min_progression_to_select(MapSlot::M01).unlocked(),
            vec![MapSlot::M01]
        );
        assert_eq!(
            ProgressionState::min_progression_to_select(MapSlot::M02).unlocked(),
            vec![MapSlot::M01, MapSlot::M02]
        );
        assert_eq!(
            ProgressionState::min_progression_to_select(MapSlot::M03).unlocked(),
            vec![MapSlot::M01, MapSlot::M02, MapSlot::M03]
        );
        // S37：选 m20 = 击败 m01~m19
        let s = ProgressionState::min_progression_to_select(MapSlot::M20);
        assert!(s.can_select(MapSlot::M20), "最小前置链应能选中 m20");
        assert!(!s.is_cleared(), "最小前置链 = m20 解锁但未击败");
    }

    /// 掩码往返：任意推进态 ↔ `unlocked_mask` ↔ `from_mask` 保持解锁集合一致。
    #[test]
    fn unlocked_mask_round_trip() {
        let states = [
            ProgressionState::initial(),
            ProgressionState::initial().on_boss_defeated(MapSlot::M01),
            ProgressionState::initial()
                .on_boss_defeated(MapSlot::M01)
                .on_boss_defeated(MapSlot::M02),
            ProgressionState::initial()
                .on_boss_defeated(MapSlot::M01)
                .on_boss_defeated(MapSlot::M02)
                .on_boss_defeated(MapSlot::M03)
                .on_boss_defeated(MapSlot::M04),
        ];
        for s in states {
            let mask = s.unlocked_mask();
            assert_eq!(
                ProgressionState::from_mask(mask).unlocked(),
                s.unlocked(),
                "mask={mask} 往返不一致"
            );
        }
        assert_eq!(ProgressionState::initial().unlocked_mask(), 0b0000_0001, "初始 = 仅 m01");
        assert_eq!(
            ProgressionState::initial().on_boss_defeated(MapSlot::M01).unlocked_mask(),
            0b0000_0011,
            "击败 m01 = m01+m02 解锁"
        );
        // 击败 m19 → m20 解锁（bit 19 = 1）
        let mut s = ProgressionState::initial();
        for slot in &MAP_SEQUENCE[0..19] {
            s = s.on_boss_defeated(*slot);
        }
        assert!(s.unlocked_mask() & (1u32 << 19) != 0, "击败 m19 后 bit 19 = 1（m20 解锁）");
        // 击败 m20 → 通关位（bit 19 = CLEARED_BIT）
        let s = s.on_boss_defeated(MapSlot::M20);
        assert!(s.is_cleared());
        assert!(s.unlocked_mask() & CLEARED_BIT != 0, "通关位 = CLEARED_BIT");
    }

    /// `mask == 0`（旧档无字段）→ 退回 `map_slot` 最小前置链，不丢当前图。
    #[test]
    fn for_save_restore_legacy_mask_zero_falls_back_to_chain() {
        assert_eq!(
            ProgressionState::for_save_restore(0, MapSlot::M01).unlocked(),
            vec![MapSlot::M01]
        );
        assert_eq!(
            ProgressionState::for_save_restore(0, MapSlot::M02).unlocked(),
            vec![MapSlot::M01, MapSlot::M02],
            "旧档（mask=0）选中 m02 仍须回读到 m02"
        );
        assert_eq!(
            ProgressionState::for_save_restore(0, MapSlot::M03).unlocked(),
            vec![MapSlot::M01, MapSlot::M02, MapSlot::M03]
        );
        // S37：旧档（mask=0）选中 m20 → 最小前置链 m01~m20 全解锁
        let s = ProgressionState::for_save_restore(0, MapSlot::M20);
        assert!(s.can_select(MapSlot::M20), "旧档选中 m20 须能回读");
        assert_eq!(s.unlocked().len(), 20);
    }

    /// 非零掩码忠实还原（不退回最小链）：m01 击败后未切图的档仍保住 m02 解锁。
    #[test]
    fn for_save_restore_nonzero_mask_is_faithful() {
        // mask = 0b011（m01 已击败 → m02 已解锁），当前仍在 m01
        let s = ProgressionState::for_save_restore(0b011, MapSlot::M01);
        assert_eq!(s.unlocked(), vec![MapSlot::M01, MapSlot::M02], "掩码须忠实还原（F23）");
        assert!(s.can_select(MapSlot::M02));
        assert!(!s.can_select(MapSlot::M03));
    }

    /// 掩码与当前所在图自相矛盾（当前图未解锁）→ 补齐最小链，不把玩家锁在档外。
    #[test]
    fn for_save_restore_contradictory_mask_repairs_current_map() {
        // mask = 0b001（仅 m01 解锁）但 map_slot = m20 → 补齐到能选中 m20
        let s = ProgressionState::for_save_restore(0b001, MapSlot::M20);
        assert!(s.can_select(MapSlot::M20), "读档后当前图必须可选");
        assert!(s.can_select(MapSlot::M01));
    }

    /// 逐图门禁判据：`is_map_defeated` 只认被击败的那张图，通关标志另计。
    #[test]
    fn is_map_defeated_is_per_map() {
        let s = ProgressionState::initial().on_boss_defeated(MapSlot::M01);
        assert!(s.is_map_defeated(MapSlot::M01));
        assert!(!s.is_map_defeated(MapSlot::M02), "未击败的图不算");
        assert!(!s.is_map_defeated(MapSlot::M20));
        assert!(!s.is_cleared(), "击败 m01 非通关");
    }

    /// S37：旧 `u8` 掩码（3 图）扩展为 `u32` 兼容。
    #[test]
    fn legacy_u8_mask_extends_to_u32() {
        // 旧 3 图掩码 0b1111（bit 3 = 通关）→ 20 图掩码 = 0x0000_1111（bit 0..=3）
        let legacy_u8: u32 = 0b1111;
        let s = ProgressionState::from_mask(legacy_u8);
        // 旧 bit 3 = m03 通关 → 20 图 bit 3 = m04 解锁（bit 0=m01, bit 1=m02, bit 2=m03, bit 3=m04）
        assert!(s.can_select(MapSlot::M04), "旧 3 图通关 → m04 应解锁");
        assert!(!s.is_cleared(), "旧 3 图通关 ≠ 20 图通关");
    }
}
