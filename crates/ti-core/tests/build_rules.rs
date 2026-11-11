//! 建造规则集成测试（S52.1；`docs/business/economy.md` 建造/银行可测规则）。
//!
//! S52.1 补测：`build.rs` `cost_of` / `can_afford` 此前 0 集成测试（飞轮 R2 找问题 P1-10）。
//! 判据：`cargo test -p ti-core --test build_rules` 全绿。

use ti_core::data::{BuildCfg, BuildingDef};
use ti_core::domain::economy::Building;
use ti_core::rules::build::{BuildManager, cost_of};

fn cfg() -> BuildCfg {
    BuildCfg {
        max_level: 3,
        buildings: vec![
            BuildingDef {
                id: "Bank".into(),
                name_key: "build.bank".into(),
                base_cost: 100,
                gold_max: 1_000,
                atk_bonus: 0,
                hp_bonus: 0,
            },
            BuildingDef {
                id: "Castle".into(),
                name_key: "build.castle".into(),
                base_cost: 500,
                gold_max: 5_000,
                atk_bonus: 0,
                hp_bonus: 0,
            },
        ],
    }
}

/// `cost_of`（建造费 = 该建筑 `base_cost`；未知建筑 = 0）。
#[test]
fn cost_of_reads_base_cost() {
    let c = cfg();
    assert_eq!(cost_of(&c, Building::Bank), 100, "Bank 造价 = 100");
    assert_eq!(cost_of(&c, Building::Castle), 500, "Castle 造价 = 500");
}

/// `can_afford`：金 ≥ 造价 = 付得起；金 < 造价 = 付不起。
#[test]
fn can_afford_compares_gold_to_cost() {
    let c = cfg();
    let mgr = BuildManager::new(c.clone(), ti_core::rules::build::Bank::new(500), &[]);
    assert!(mgr.can_afford(100), "500 金付得起 100（Bank）");
    assert!(!mgr.can_afford(501), "500 金付不起 501");
    assert!(mgr.can_afford(500), "500 金恰付得起 500（Castle）");
}

/// 建造后银行金扣减 + 等级 +1（`can_afford` 是建前置条件）。
#[test]
fn build_debits_and_raises_level() {
    let mut mgr = BuildManager::new(cfg(), ti_core::rules::build::Bank::new(1_000), &[]);
    let bank = "Bank";
    mgr.build(bank).expect("金足应建成功");
    assert_eq!(mgr.level(bank), 1, "建后等级 = 1");
    assert!(mgr.bank().total() < 1_000, "金已扣减");
}
