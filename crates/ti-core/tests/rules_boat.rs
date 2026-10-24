//! S21.10 船系统规则测试（判据 = `docs/business/economy.md` §船系统可测试规则表）。
//!
//! | 规则 | 单测 |
//! |---|---|
//! | 造船扣金 + 抬 cargo/atk/hp（立即生效） | `boat_upgrade_debits_and_raises` |
//! | 单次贸易数量上限 = `min(qty_max, cargo)`；1 级 ≡ 现状 | `trade_qty_capped_by_boat_cargo` |
//! | 等级单调（不可降级；满级无操作；金不够拒绝） | `boat_level_monotonic` |
//! | 船加成与建筑加成叠加进战斗 | `combat_bonus_includes_boat` |
//!
//! 数值全部由 `balance.json → boat.levels` 注入（测试里只断言**业务常量**
//! ——等级链 99/150/240/400 与加成 0/4/9/15、0/80/180/320 ——锁规格，不复制实现）。

use ti_core::data::balance;
use ti_core::domain::combat::Unit;
use ti_core::rules::battle::{BattleSession, CombatBonus};
use ti_core::rules::boat::{BoatErr, BoatSession};
use ti_core::rules::build::{Bank, BuildManager};
use ti_core::rules::trade::{TradeErr, TradeLedger};

/// 规则 1：造船扣金 + 抬 cargo/atk/hp（立即生效）。
#[test]
fn boat_upgrade_debits_and_raises() {
    let b = balance().expect("balance.json 有效");
    let mut boat = BoatSession::new(b.boat.clone(), 1);
    let mut gold = 5_000u32;

    // 1 级 = 开局小舢板（cargo 99 = 现状有效上限；加成 0）
    assert_eq!(boat.level(), 1, "开局 1 级");
    assert_eq!(boat.cargo(), 99, "1 级货舱 = 现状有效上限 99");
    assert_eq!(boat.atk(), 0, "1 级无战斗 atk 加成");
    assert_eq!(boat.hp(), 0, "1 级无战斗 hp 加成");

    // 升 2 级：扣造价 + 货舱/加成立即生效
    let cost2 = boat.next_cost().expect("2 级造价在表");
    let level = boat.upgrade(&mut gold).expect("金足应升级成功");
    assert_eq!(level, 2, "升级后等级 = 2");
    assert_eq!(gold, 5_000 - cost2, "扣款 = 2 级造价");
    assert_eq!(boat.level(), 2);
    assert_eq!(boat.cargo(), 150, "2 级货舱 150");
    assert_eq!(boat.atk(), 4, "2 级累计 atk 4");
    assert_eq!(boat.hp(), 80, "2 级累计 hp 80");

    // 升 3 级：累加生效（累计值直接加）
    let cost3 = boat.next_cost().expect("3 级造价在表");
    let gold_before = gold;
    boat.upgrade(&mut gold).expect("金足应升级成功");
    assert_eq!(gold, gold_before - cost3, "扣款 = 3 级造价");
    assert_eq!(boat.cargo(), 240, "3 级货舱 240");
    assert_eq!(boat.atk(), 9, "3 级累计 atk 9");
    assert_eq!(boat.hp(), 180, "3 级累计 hp 180");
}

/// 规则 2：单次贸易数量上限 = `min(qty_max, cargo)`；1 级 ≡ 现状。
#[test]
fn trade_qty_capped_by_boat_cargo() {
    let b = balance().expect("balance.json 有效");
    let gid = b.trade.goods[0].id;
    let qty_max = b.trade.goods[0].qty_max;
    assert_eq!(qty_max, 400, "目录上限已翻转为 400（与船舱钳制同一步落地）");

    // 1 级（cargo 99）⇒ 有效上限 = min(400, 99) = 99 = 旧 qty_max（1 级 ≡ 现状）
    let boat1 = BoatSession::new(b.boat.clone(), 1);
    assert_eq!(boat1.effective_qty_max(qty_max), 99, "1 级有效上限 = 99");
    let mut l1 = TradeLedger::new(b.trade.clone(), 1_000_000, &[], b.economy.gold_max);
    boat1.apply_cargo_to(&mut l1);
    assert_eq!(l1.qty_cap(gid), 99, "账本有效上限 = min(qty_max, cargo) = 99");
    assert_eq!(l1.buy(gid, 100), Err(TradeErr::QtyOutOfRange), "1 级买 100 超货舱（现状上限 99）");
    l1.buy(gid, 99).expect("1 级买 99 应成功（= 现状上限）");

    // 2 级（cargo 150）⇒ 有效上限 = 150
    let boat2 = BoatSession::new(b.boat.clone(), 2);
    let mut l2 = TradeLedger::new(b.trade.clone(), 1_000_000, &[], b.economy.gold_max);
    boat2.apply_cargo_to(&mut l2);
    assert_eq!(l2.qty_cap(gid), 150, "2 级有效上限 = 150");
    assert_eq!(l2.buy(gid, 151), Err(TradeErr::QtyOutOfRange));

    // 4 级（cargo 400）⇒ 有效上限 = min(400, 400) = 400（换大船 = 单次可买更多）
    let boat4 = BoatSession::new(b.boat.clone(), 4);
    assert_eq!(boat4.effective_qty_max(qty_max), 400, "4 级有效上限 = 400");
    let mut l4 = TradeLedger::new(b.trade.clone(), 10_000_000, &[], b.economy.gold_max);
    boat4.apply_cargo_to(&mut l4);
    assert_eq!(l4.qty_cap(gid), 400, "4 级有效上限 = 400");
    assert_eq!(l4.buy(gid, 401), Err(TradeErr::QtyOutOfRange));
    l4.buy(gid, 400).expect("4 级买 400 应成功");
}

/// 规则 3：等级单调（不可降级；满级无操作；金不够拒绝）。
#[test]
fn boat_level_monotonic() {
    let b = balance().expect("balance.json 有效");

    // 金不够 ⇒ 拒绝，状态不变
    let mut poor = BoatSession::new(b.boat.clone(), 1);
    let mut poor_gold = 10u32;
    assert_eq!(poor.upgrade(&mut poor_gold), Err(BoatErr::InsufficientGold), "金不够拒绝");
    assert_eq!(poor.level(), 1, "拒绝后等级不变");
    assert_eq!(poor_gold, 10, "拒绝后金币不变");

    // 逐级升级：每次只升 1 级、货舱严格递增
    let mut boat = BoatSession::new(b.boat.clone(), 1);
    let mut gold = 100_000u32;
    let mut prev_cargo = boat.cargo();
    while let Some(cost) = boat.next_cost() {
        assert!(gold >= cost, "金足够支付下一级造价");
        let prev_level = boat.level();
        let level = boat.upgrade(&mut gold).expect("升级应成功");
        assert_eq!(level, prev_level + 1, "单向：每次只升 1 级");
        assert!(boat.cargo() > prev_cargo, "货舱容量严格递增");
        prev_cargo = boat.cargo();
    }
    assert_eq!(boat.level(), 4, "满级 = 4 级（巡防战船）");
    assert_eq!(boat.cargo(), 400, "4 级货舱 400");
    assert!(boat.is_max_level(), "满级判定");

    // 满级 ⇒ 无操作（不扣款、不越界）
    let gold_before = gold;
    assert_eq!(boat.upgrade(&mut gold), Err(BoatErr::MaxLevel), "满级升级无操作");
    assert_eq!(boat.level(), 4, "满级后等级不变");
    assert_eq!(gold, gold_before, "满级后金币不变");

    // 无降级接口（API 面只有 upgrade ⇒ 编译期即不可降级）
}

/// 规则 4：船加成与建筑加成叠加进战斗（总 atk/hp = 建筑 Σ + 船累计值）。
#[test]
fn combat_bonus_includes_boat() {
    let b = balance().expect("balance.json 有效");

    // 建筑侧：建 1 级 Castle ⇒ 取其 Σ 加成
    let mut mgr = BuildManager::new(b.build.clone(), Bank::new(10_000), &[]);
    mgr.build("Castle").expect("金足应建成 Castle");
    let build_atk = mgr.build_atk_bonus();
    let build_hp = mgr.build_hp_bonus();
    assert!(build_atk > 0 && build_hp > 0, "Castle 有战斗加成");

    // 船侧：3 级（三桅大帆船）累计 atk 9 / hp 180
    let boat = BoatSession::new(b.boat.clone(), 3);
    assert_eq!(boat.atk(), 9, "3 级累计 atk 9");
    assert_eq!(boat.hp(), 180, "3 级累计 hp 180");

    // 叠加：总加成 = 建筑 Σ + 船累计值
    let bonus = CombatBonus::combine(build_atk, build_hp, &boat);
    assert_eq!(bonus.atk, build_atk + boat.atk(), "总 atk = 建筑 Σ + 船");
    assert_eq!(bonus.hp, build_hp + boat.hp(), "总 hp = 建筑 Σ + 船");

    let base = Unit { hp: 1_000, atk: 100 };
    let unit = bonus.apply(base);
    assert_eq!(unit.atk, 100 + build_atk + boat.atk(), "玩家 atk = 基础 + 建筑 + 船");
    assert_eq!(unit.hp, 1_000 + build_hp + boat.hp(), "玩家 hp = 基础 + 建筑 + 船");

    // 战斗内生效：同 seed 下带加成 ⇒ BOSS 掉血更多（船加成进伤害公式）
    let mut plain = BattleSession::new_with_cfg(b.combat.clone(), 42, base, 0);
    let mut buffed = BattleSession::new_with_bonus(b.combat.clone(), 42, base, 0, bonus);
    let (_, boss_hp_plain, _) = plain.step();
    let (_, boss_hp_buffed, _) = buffed.step();
    assert!(
        boss_hp_buffed < boss_hp_plain,
        "船加成进伤害公式：BOSS 剩余 HP {boss_hp_buffed} 应 < {boss_hp_plain}"
    );

    // 1 级船加成 = 0 ⇒ 与「只有建筑加成」等价（1 级 ≡ 现状）
    let boat1 = BoatSession::new(b.boat.clone(), 1);
    let bonus1 = CombatBonus::combine(build_atk, build_hp, &boat1);
    assert_eq!(bonus1.atk, build_atk, "1 级船无 atk 加成");
    assert_eq!(bonus1.hp, build_hp, "1 级船无 hp 加成");
}
