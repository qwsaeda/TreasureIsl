//! 成就规则层单测（`docs/business/achievements.md`）；ti-core 公开面 `rules::achievements`。
use ti_core::rules::achievements as ach;

#[test]
fn eval_unlocks_qualified() {
    let stats = ach::AchievementStats {
        boss_defeats: 1,
        peak_gold: ach::TYCOON_GOLD,
        ..Default::default()
    };
    let (mask, newly) = ach::eval(0, &stats);
    assert_eq!(newly, vec![ach::id::FIRST_BOSS, ach::id::TYCOON]);
    assert!(ach::is_unlocked(mask, ach::id::FIRST_BOSS) && ach::is_unlocked(mask, ach::id::TYCOON));
    assert!(!ach::is_unlocked(mask, ach::id::CONQUEROR));
}

#[test]
fn eval_monotonic_never_clears() {
    let all = (1u64 << ach::id::COUNT) - 1;
    let (mask, newly) = ach::eval(all, &ach::AchievementStats::default());
    assert_eq!(mask, all);
    assert_eq!(newly.len(), 0);
}

#[test]
fn eval_multi_and_boat_master() {
    let stats = ach::AchievementStats {
        owned_cells: 2,
        cleared_m01: true,
        boat_level: ach::MASTER_SHIP_LEVEL,
        ..Default::default()
    };
    let (_, newly) = ach::eval(0, &stats);
    assert_eq!(newly, vec![ach::id::CONQUEROR, ach::id::CLEAR_M01, ach::id::MASTER_SHIP]);
}

#[test]
fn name_keys_nonempty() {
    for &a in &ach::id::ALL {
        assert!(!ach::name_key(a).is_empty(), "id {a} 须有 i18n key");
    }
}
