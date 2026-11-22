//! 成就系统规则层（`docs/business/achievements.md` 单一事实源）。
//!
//! 5 个成就 = u64 位掩码 bit 0..=4；`eval(mask, &stats)` 纯函数按 stats 达标置位，
//! 单调递增（只增不减）。阈值固定常量（不随 balance 调，避免 schema 扩散）。
//! 账号级（不按席位）。纯记录，不影响玩法数值/胜负。

/// 成就 id = 位掩码位号（与 `docs/business/achievements.md` 表逐行对应）。
pub mod id {
    /// 首次击败 BOSS。
    pub const FIRST_BOSS: u32 = 0;
    /// 首次占领领地格。
    pub const CONQUEROR: u32 = 1;
    /// 通关 m01。
    pub const CLEAR_M01: u32 = 2;
    /// 金币峰值达标。
    pub const TYCOON: u32 = 3;
    /// 船升满级。
    pub const MASTER_SHIP: u32 = 4;
    /// 全部成就 id（顺序 = 位号）。
    pub const ALL: [u32; 5] = [FIRST_BOSS, CONQUEROR, CLEAR_M01, TYCOON, MASTER_SHIP];
    /// 成就总数。
    pub const COUNT: usize = 5;
}

/// i18n 名称 key（与 `strings.json::achievement.*` 对应）。
#[must_use]
pub fn name_key(id: u32) -> &'static str {
    match id {
        id::FIRST_BOSS => "achievement.first_boss",
        id::CONQUEROR => "achievement.conqueror",
        id::CLEAR_M01 => "achievement.clear_m01",
        id::TYCOON => "achievement.tycoon",
        id::MASTER_SHIP => "achievement.master_ship",
        _ => "",
    }
}

/// 富甲一方成就的金币峰值阈值（规格 §3）。
pub const TYCOON_GOLD: u32 = 10_000;
/// 舰队旗舰成就的船等级阈值（规格 §3）。
pub const MASTER_SHIP_LEVEL: u32 = 4;

/// 成就统计快照（app 层从运行状态汇总；纯值语义，无借用）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AchievementStats {
    /// 累计击败 BOSS 次数。
    pub boss_defeats: u32,
    /// 当前占领的领地格数量。
    pub owned_cells: u32,
    /// m01 是否已通关（CLEARED 位）。
    pub cleared_m01: bool,
    /// 金币峰值（本档历史最高）。
    pub peak_gold: u32,
    /// 船等级（1..=4）。
    pub boat_level: u32,
}

impl AchievementStats {
    /// 某项成就是否达标（规格 §2 触发条件表）。
    #[must_use]
    pub fn qualifies(self, id: u32) -> bool {
        match id {
            id::FIRST_BOSS => self.boss_defeats >= 1,
            id::CONQUEROR => self.owned_cells >= 1,
            id::CLEAR_M01 => self.cleared_m01,
            id::TYCOON => self.peak_gold >= TYCOON_GOLD,
            id::MASTER_SHIP => self.boat_level >= MASTER_SHIP_LEVEL,
            _ => false,
        }
    }
}

/// 计算新掩码 + 本次新解锁的 id 列表（单调递增，已置位永不清除）。
#[must_use]
pub fn eval(mask: u64, stats: &AchievementStats) -> (u64, Vec<u32>) {
    let mut next = mask;
    let mut newly = Vec::new();
    for &a in &id::ALL {
        let bit = 1u64 << a;
        if next & bit == 0 && stats.qualifies(a) {
            next |= bit;
            newly.push(a);
        }
    }
    (next, newly)
}

/// 是否已解锁。
#[must_use]
pub const fn is_unlocked(mask: u64, id: u32) -> bool {
    mask & (1u64 << id) != 0
}
