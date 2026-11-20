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
/// **全解锁口径（S62 / 用户 2026-11-20 拍板「全部能玩」）**：`can_select` 恒 `true`
/// （20 图开局全可选），单调解锁链废弃；`defeated[]` / `on_boss_defeated` /
/// `is_cleared`（通关位）/ `defeated_count`（进度计数）语义保留，仅不再作为「选择门禁」。
/// 旧单调链（`m(i) 已解锁 ⟺ i==0 || defeated[i-1]`，m01 恒解锁…m20 需击败 m19）已作废。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProgressionState {
    /// 每张图 BOSS 是否已击败（`[m01, m02, …, m20]`）。
    defeated: [bool; 20],
}

impl ProgressionState {
    /// 初始推进状态：无任何图被击败（全解锁口径下仅表示"尚无通关进度"，不影响可选图）。
    #[must_use]
    pub const fn initial() -> Self {
        Self { defeated: [false; 20] }
    }

    /// 初始解锁集合 = **全部 20 图**（S62 全解锁，用户 2026-11-20 拍板）。
    #[must_use]
    pub fn initial_unlocked() -> Vec<MapSlot> {
        Self::initial().unlocked()
    }

    /// 当前解锁集合 = 全部 20 图（全解锁口径；`can_select` 恒 true）。
    #[must_use]
    pub fn unlocked(self) -> Vec<MapSlot> {
        MapSlot::all().iter().copied().filter(|m| self.can_select(*m)).collect()
    }

    /// 任意图均可选中（S62 全解锁，废弃单调解锁门禁）。
    /// `self`/`_m` 保留 = 维持原方法签名（调用方 + 存档口径兼容），允许 unused。
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn can_select(&self, _m: MapSlot) -> bool {
        true
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
    ///
    /// **S62 全解锁勘误**：解锁恒全开，不再逐图持久化（bit0..18 恒 0，避免读档 `from_mask`
    /// 把「全 1 解锁位」误读成「全图已击败 + 通关」）。掩码仅承载通关位 `CLEARED_BIT`（bit19）。
    #[must_use]
    pub fn unlocked_mask(self) -> u32 {
        if self.is_cleared() { CLEARED_BIT } else { 0 }
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

    /// S62 全解锁：初始 20 图全可选（旧「仅 m01」口径废弃，用户 2026-11-20 拍板）。
    #[test]
    fn progression_initial_all_unlocked() {
        let s = ProgressionState::initial();
        assert_eq!(s.unlocked().len(), 20, "初始全 20 图解锁");
        assert_eq!(ProgressionState::initial_unlocked().len(), 20);
        for m in MapSlot::all() {
            assert!(s.can_select(m), "{m:?} 可选（全解锁）");
        }
        assert!(!s.is_cleared(), "初始未通关");
    }

    /// 击败记录不再门禁：击败 m01 后所有图仍全可选（进度语义 = defeated[] 保留）。
    #[test]
    fn progression_defeat_tracks_progress_not_gate() {
        let s = ProgressionState::initial().on_boss_defeated(MapSlot::M01);
        assert!(!s.is_cleared(), "击败 m01 非通关");
        assert_eq!(s.unlocked().len(), 20, "全解锁： defeating 不改可选集合");
        assert!(s.can_select(MapSlot::M02));
        assert!(s.can_select(MapSlot::M03));
        assert!(s.is_map_defeated(MapSlot::M01), "m01 击败位保留（进度语义）");
        assert!(!s.is_map_defeated(MapSlot::M02), "m02 未击败");
    }

    /// 击败 m19 仍非通关，m20 全可选（门禁废弃）。
    #[test]
    fn progression_defeat_m19_not_cleared() {
        let mut s = ProgressionState::initial();
        for slot in MAP_SEQUENCE.iter().take(19) {
            s = s.on_boss_defeated(*slot);
        }
        assert!(!s.is_cleared(), "击败 m19 非通关");
        assert!(s.can_select(MapSlot::M20), "m20 全解锁可选");
        assert_eq!(s.unlocked().len(), 20);
    }

    /// 击败 m20 = 通关、`defeated_count` 保留（S62 进度语义不删）。
    #[test]
    fn progression_defeat_m20_clears_game() {
        let mut s = ProgressionState::initial();
        for slot in MAP_SEQUENCE.iter().take(19) {
            s = s.on_boss_defeated(*slot);
        }
        let s = s.on_boss_defeated(MapSlot::M20);
        assert!(s.is_cleared(), "击败 m20 应判通关");
        assert_eq!(s.defeated_count(), 20);
    }

    /// 不击败任何图：全解锁仍可选，但无进度（进度语义 = `defeated_count` 0）。
    #[test]
    fn progression_no_defeat_still_all_unlocked() {
        let s = ProgressionState::initial();
        assert_eq!(s.unlocked().len(), 20, "全解锁：不击败也全可选");
        assert_eq!(s.defeated_count(), 0, "但无击败进度");
        assert!(!s.is_cleared());
    }

    /// 全解锁恒成立（幂等、无单调链依赖）。
    #[test]
    fn progression_unlocks_always_all() {
        let s = ProgressionState::initial();
        let s1 = s.on_boss_defeated(MapSlot::M01);
        let s2 = s1.on_boss_defeated(MapSlot::M02);
        for st in [s, s1, s2] {
            assert_eq!(st.unlocked().len(), 20, "任意进度下全 20 图可选");
        }
    }

    /// `from_unlocked`：全解锁恒全可选（旧「解锁集→击败位」重建保留兼容，但非门禁）。
    #[test]
    fn from_unlocked_all_selectable() {
        let s = ProgressionState::from_unlocked(&[MapSlot::M01, MapSlot::M02, MapSlot::M03]);
        assert_eq!(s.unlocked().len(), 20, "旧档重建后仍全可选（S62）");
        assert!(s.can_select(MapSlot::M04));
        assert!(!s.is_cleared());
    }

    /// 全解锁集 + m20 击败 = 通关（app 层重复结算稳定）。
    #[test]
    fn from_unlocked_all_then_m20_clears() {
        let all: Vec<MapSlot> = MapSlot::all().to_vec();
        let s = ProgressionState::from_unlocked(&all).on_boss_defeated(MapSlot::M20);
        assert!(s.is_cleared());
        assert_eq!(s.defeated_count(), 20);
        let again = ProgressionState::from_unlocked(&all).on_boss_defeated(MapSlot::M20);
        assert_eq!(s.unlocked(), again.unlocked(), "幂等");
    }

    /// 旧档异常解锁集：全解锁下不再有「独缺 m02」门禁（任意图可选）。
    #[test]
    fn from_unlocked_legacy_anomaly_all_selectable() {
        let s = ProgressionState::from_unlocked(&[MapSlot::M01, MapSlot::M03]);
        assert_eq!(s.unlocked().len(), 20, "全解锁：旧档异常集也全可选");
        assert!(s.can_select(MapSlot::M02));
    }

    /// `min_progression_to_select`：S62 全解锁下恒全可选（旧「最小前置链」语义废弃）。
    #[test]
    fn min_progression_to_select_all_selectable() {
        for m in MapSlot::all() {
            let s = ProgressionState::min_progression_to_select(m);
            assert_eq!(s.unlocked().len(), 20, "任意选中图的推进态均全可选");
            assert!(!s.is_cleared(), "前置链不含通关位");
        }
    }

    /// 掩码往返：S62 全解锁下 `unlocked_mask` 仅承载通关位（bit19）；解锁恒全开不持久化。
    #[test]
    fn unlocked_mask_cleared_bit_only() {
        let s = ProgressionState::initial().on_boss_defeated(MapSlot::M01).on_boss_defeated(MapSlot::M02);
        let mask = s.unlocked_mask();
        assert_eq!(mask & ((1u32 << 19) - 1), 0, "全解锁：bit0..18 恒 0（不持久化解锁）");
        assert_eq!(mask & CLEARED_BIT, 0, "未通关：bit19=0");
        let back = ProgressionState::from_mask(mask);
        assert_eq!(back.unlocked().len(), 20, "from_mask 后仍全可选");
        // 通关位
        let mut full = ProgressionState::initial();
        for slot in MAP_SEQUENCE.iter().take(19) { full = full.on_boss_defeated(*slot); }
        let full = full.on_boss_defeated(MapSlot::M20);
        assert!(full.is_cleared());
        assert!(full.unlocked_mask() & CLEARED_BIT != 0, "通关位 = CLEARED_BIT");
    }

    /// `for_save_restore`：全解锁下任意 mask/选中图都全可选；通关位由 bit19 还原。
    #[test]
    fn for_save_restore_all_selectable() {
        for mask in [0u32, 0b001, 0b011, CLEARED_BIT, u32::MAX] {
            for m in MapSlot::all() {
                let s = ProgressionState::for_save_restore(mask, m);
                assert_eq!(s.unlocked().len(), 20, "任意 mask/map 全可选");
            }
        }
        assert!(!ProgressionState::for_save_restore(0, MapSlot::M01).is_cleared());
        assert!(ProgressionState::for_save_restore(CLEARED_BIT, MapSlot::M20).is_cleared(), "通关位还原");
    }

    /// 逐图击败位：`is_map_defeated` 忠实反映 defeated[]（进度语义，非门禁）。
    #[test]
    fn is_map_defeated_is_per_map() {
        let s = ProgressionState::initial().on_boss_defeated(MapSlot::M01);
        assert!(s.is_map_defeated(MapSlot::M01));
        assert!(!s.is_map_defeated(MapSlot::M02), "未击败的图不算");
        assert!(!s.is_map_defeated(MapSlot::M20));
        assert!(!s.is_cleared(), "击败 m01 非通关");
    }

    /// S37：旧 `u8` 掩码扩展为 `u32`（`from_mask` 仍按 bit 解析击败位，兼容旧档）。
    #[test]
    fn legacy_u8_mask_extends_to_u32() {
        let s = ProgressionState::from_mask(0b1111u32);
        assert_eq!(s.unlocked().len(), 20, "旧档读回全可选（S62）");
        assert!(!s.is_cleared(), "旧 3 图通关 ≠ 20 图通关");
        assert!(s.is_map_defeated(MapSlot::M01), "旧 bit1 = m01 击败位保留");
    }
}
