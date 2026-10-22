//! S19.5a 跨图难度阶梯不变量（`docs/business/progression.md` §跨图难度阶梯）。
//!
//! 阶梯设计（业务规格的唯一事实源 = `docs/business/progression.md`）：
//! - **难度单调递增**：地图 index 越大 → `boss_tier` 越大 → BOSS 越强；
//! - **强度表单调**：`boss_tiers[]` 的 HP / ATK 随 tier 严格递增；
//! - **tier 有效**：每图 `boss_tier` 必须落在 `boss_tiers` 范围内；
//! - **「攒金→建造→打 BOSS」可闭环**：初始金 ≥ 最低「战力建筑」成本（Castle）
//!   ⇒ 玩家在打 BOSS 前**一定造得起**一座 Castle（m01 余血 82% / m02 58% / m03 28%）。
//!
//! 本文件只测**结构性不变量**（数据层可判定）；各图的生存余量等**模拟口径**写在业务规格里。

use ti_core::data::balance;

/// 取平衡表（加载失败即测试失败 —— 数据表是硬错误，不 fallback）。
fn bal() -> ti_core::data::Balance {
    balance().expect("balance.json 应可加载").clone()
}

#[test]
fn difficulty_tier_increases_with_map_index() {
    let b = bal();
    let mut prev: Option<(u32, u32)> = None;
    for m in &b.selection.maps {
        if let Some((prev_idx, prev_tier)) = prev {
            assert!(
                m.boss_tier > prev_tier,
                "难度阶梯必须随地图 index 递增：m{prev_idx}(tier {prev_tier}) -> m{}(tier {})",
                m.index,
                m.boss_tier
            );
        }
        prev = Some((m.index, m.boss_tier));
    }
    assert!(prev.is_some(), "地图目录不应为空");
}

#[test]
fn every_map_tier_is_within_boss_tier_table() {
    let b = bal();
    let tier_count = u32::try_from(b.combat.boss_tiers.len()).unwrap_or(0);
    for m in &b.selection.maps {
        assert!(
            m.boss_tier < tier_count,
            "地图 {} 的 boss_tier {} 越界（boss_tiers 只有 {tier_count} 档）",
            m.id,
            m.boss_tier
        );
    }
}

#[test]
fn boss_tier_strength_increases_monotonically() {
    let b = bal();
    let tiers = &b.combat.boss_tiers;
    for w in tiers.windows(2) {
        assert!(
            w[1].hp > w[0].hp,
            "BOSS HP 必须随 tier 递增：tier{}={} -> tier{}={}",
            w[0].tier,
            w[0].hp,
            w[1].tier,
            w[1].hp
        );
        assert!(
            w[1].atk > w[0].atk,
            "BOSS ATK 必须随 tier 递增：tier{}={} -> tier{}={}",
            w[0].tier,
            w[0].atk,
            w[1].tier,
            w[1].atk
        );
    }
}

#[test]
fn initial_gold_covers_one_power_building() {
    // 「攒金→建造→打 BOSS」闭环的前置：初始金 ≥ 一座 Castle 成本
    // （Castle = +10 atk / +200 hp，是打高 tier BOSS 的关键 ⇒ 造得起才有意义）。
    let b = bal();
    let castle_cost = b
        .build
        .buildings
        .iter()
        .find(|x| x.id == "Castle")
        .map(|x| x.base_cost)
        .expect("Castle 应在 build.buildings 中");
    assert!(
        b.economy.initial_gold >= castle_cost,
        "初始金 {} < Castle 成本 {castle_cost} ⇒ 打 BOSS 前造不起战力建筑",
        b.economy.initial_gold
    );
}

#[test]
fn map_catalog_has_boss_and_landmarks() {
    // 每图都应有 BOSS 格与据点（难度阶梯的前提：每图都有终点）
    let b = bal();
    for m in &b.selection.maps {
        assert!(m.boss_tile.is_some(), "地图 {} 缺 BOSS 格坐标", m.id);
        assert!(!m.landmarks.is_empty(), "地图 {} 缺据点坐标", m.id);
        assert!(m.grid_w > 0 && m.grid_h > 0, "地图 {} 网格尺寸非法", m.id);
    }
    assert_eq!(
        b.selection.maps.len(),
        b.selection.map_count as usize,
        "地图目录数与 map_count 应一致"
    );
}

#[test]
fn boss_tier_of_matches_balance_catalog() {
    // 规则层查询 `boss_tier_of`（S19.5b：难度档位接线）须与 balance 目录一致
    use ti_core::domain::selection::MapSlot;
    use ti_core::rules::progression::boss_tier_of;
    let b = bal();
    for m in &b.selection.maps {
        let slot = MapSlot::all()
            .iter()
            .find(|s| s.id() == m.id)
            .copied()
            .expect("MapSlot 应覆盖目录里的地图 id");
        assert_eq!(
            boss_tier_of(slot),
            Some(m.boss_tier),
            "地图 {} 的难度档位查询应与 balance 一致",
            m.id
        );
    }
}
