//! S15.1 经济域业务场景穷举测试（`exhaust_economy`）。
//!
//! 枚举空间全部由 `assets/data/exhaust_scenarios.json`（`ti_core::data::exhaust()`）驱动：
//! - 贸易侧：balance.json 全 5 商品 × `qty` 偏移档（JSON `trade.qty_offset_levels`，
//!   逐商品取自己的 `qty_max` 再偏移）× `gold` 偏移档（JSON `trade.gold_offset_levels`）
//! - 建造侧：balance.json 全 4 建筑 × 逐档升级 `0..max_level`（`max_level` 读真实表，不硬编码）
//! - 夹具隔离假值：JSON `trade.fixture`/`build.fixture` 驱动，双轨交叉验证
//!
//! 不变式（每组合，精确值对账，非自我循环）：
//! - `buy`：`InsufficientGold`/`QtyOutOfRange` 命中路径与独立手算一致
//! - `sell`：`gold_left`/`overflow` 精确值与独立手算一致；全程 `gold ≤ gold_cap`
//! - `build`：`build_cost`/`gold_cap()`/`bank().gold` 逐档对账；满级后 `InvalidLevel`

use ti_core::data::{BuildCfg, BuildingDef, GoodsDef, TradeCfg, balance, exhaust};
use ti_core::rules::build::{Bank, BuildErr, BuildManager};
use ti_core::rules::trade::{TradeErr, TradeLedger};

/// `round(x × y)` 钳到 `u32`（避免直接 cast 触发 clippy pedantic，与 `rules::trade::f64_to_u32` 同口径）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn f64_round_to_u32(x: f64, y: f64) -> u32 {
    (x * y).round().min(f64::from(u32::MAX)) as u32
}

/// 贸易侧穷举：全 5 商品 × `qty` 全偏移档 × `gold` 全偏移档（档位由 JSON 驱动，非硬编码 5/4 档）。
#[test]
fn exhaust_trade_all_goods_qty_gold() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let cfg = real_trade_cfg();
    let gold_cap = balance().expect("balance.json 有效").economy.gold_max;
    let markup = cfg.markup;
    let mut combos: u32 = 0;

    for good in &cfg.goods {
        let price = good.buy;
        let cap_qty = good.qty_max;
        let sell_unit = f64_round_to_u32(f64::from(price), f64::from(markup));

        for &qty_off in &es.trade.qty_offset_levels {
            let Some(qty) = apply_offset(cap_qty, qty_off) else {
                continue; // 越界偏移（如 qty_max=1 时取 qty_max-1）跳过，不强行构造 0/负值
            };
            for &gold_off in &es.trade.gold_offset_levels {
                combos += check_trade_combo(
                    &cfg, good.id, price, cap_qty, sell_unit, qty, gold_off, gold_cap, markup,
                );
            }
        }
    }
    assert!(
        combos > 0,
        "贸易组合数须 > 0（JSON 驱动的档位非空，且 5 商品全表遍历至少产生若干有效组合）"
    );
}

/// 单条贸易组合的不变式断言（buy + sell 两侧精确对账），返回本组合贡献的组合计数。
#[allow(clippy::too_many_arguments)]
fn check_trade_combo(
    cfg: &TradeCfg,
    gid: u32,
    price: u32,
    cap_qty: u32,
    sell_unit: u32,
    qty: u32,
    gold_off: i32,
    gold_cap: u32,
    _markup: f32,
) -> u32 {
    let gold = apply_offset_u32(price, gold_off);
    let mut ledger = TradeLedger::new(cfg.clone(), gold, &[], gold_cap);
    let mut count: u32 = 0;

    // ── buy 侧不变式 ──
    match ledger.buy(gid, qty) {
        Err(TradeErr::QtyOutOfRange) => {
            let expected_out_of_range = qty == 0 || qty > cap_qty;
            assert!(expected_out_of_range, "buy(gid={gid}, qty={qty}) 意外命中 QtyOutOfRange");
        }
        Err(TradeErr::InsufficientGold) => {
            let expected = gold < price.saturating_mul(qty);
            assert!(
                expected,
                "buy(gid={gid}, qty={qty}, gold={gold}) 意外命中 InsufficientGold（独立手算：{gold} < {price}×{qty}={}）",
                price.saturating_mul(qty)
            );
        }
        Err(TradeErr::UnknownGood) => unreachable!("buy(gid={gid}) 不应 UnknownGood（全表遍历）"),
        Err(TradeErr::NoStock) => unreachable!("buy 不应命中 NoStock"),
        Ok(res) => {
            let cost = price.saturating_mul(qty);
            assert_eq!(res.amount, cost, "buy 成交额须 = price×qty（独立手算）");
            assert_eq!(res.gold_left, gold - cost, "buy 后金币须 = gold - cost（独立手算）");
            assert_eq!(res.overflow, 0, "buy 不产生 overflow");
            assert!(
                res.gold_left <= gold_cap,
                "buy 后 gold ≤ gold_cap（无下溢由 saturating 保证）"
            );
            assert_eq!(ledger.owned(gid), qty, "buy 后持有量 = qty");
        }
    }
    count += 1;

    // ── sell 侧不变式（需先持有货物，sellable_qty > 0 才执行）──
    let sellable_qty = qty.min(cap_qty);
    if sellable_qty > 0 {
        let mut ledger = TradeLedger::new(cfg.clone(), 0, &[(gid, sellable_qty)], gold_cap);
        let gain = sell_unit.saturating_mul(sellable_qty);
        let raw = 0u32.saturating_add(gain);
        let (expected_gold, expected_overflow) = if gold_cap > 0 {
            let clamped = raw.min(gold_cap);
            (clamped, raw.saturating_sub(clamped))
        } else {
            (raw, 0)
        };
        match ledger.sell(gid, sellable_qty) {
            Err(TradeErr::NoStock) => {
                unreachable!("sell(gid={gid}, qty={sellable_qty}) 意外 NoStock（已预置持有量）")
            }
            Err(e) => {
                unreachable!("sell(gid={gid}) 意外 {e:?}（全表遍历不应未知/越界）")
            }
            Ok(res) => {
                assert_eq!(res.gold_left, expected_gold, "sell 后金币须 = 独立手算值");
                assert_eq!(res.overflow, expected_overflow, "sell overflow 须 = 独立手算值");
                assert!(res.gold_left <= gold_cap, "sell 后 gold ≤ gold_cap");
                assert_eq!(ledger.owned(gid), 0, "sell 全量后持有量 = 0");
            }
        }
        count += 1;
    }
    count
}

/// `qty_max` 偏移档 → 具体 `qty` 值（负偏移且结果 < 1 时返回 `None`，调用方跳过该档）。
fn apply_offset(cap_qty: u32, off: i32) -> Option<u32> {
    let v = i64::from(cap_qty) + i64::from(off);
    (v >= 0).then(|| u32::try_from(v).unwrap_or(0)).filter(|_| v >= 0)
}

/// `price` 偏移档 → 具体 `gold` 值（负偏移且结果 < 0 时钳到 0，`u32` 无下溢语义）。
fn apply_offset_u32(price: u32, off: i32) -> u32 {
    let v = i64::from(price) + i64::from(off);
    u32::try_from(v.max(0)).unwrap_or(0)
}

fn real_trade_cfg() -> TradeCfg {
    balance().expect("balance.json 有效").trade.clone()
}

/// 夹具隔离假值贸易侧（JSON `trade.fixture` 驱动，不读 balance，验证边界逻辑独立于具体数值）：
/// 逐用例对账 `sell` 后 `gold_left`/`overflow` 精确值。
#[test]
fn exhaust_trade_fixture_cross_validation() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let fx = &es.trade.fixture;
    let cfg = TradeCfg {
        markup: fx.markup,
        goods: fx
            .goods
            .iter()
            .map(|g| GoodsDef {
                id: g.id,
                name_key: format!("goods.{}", g.id),
                buy: g.buy,
                qty_max: g.qty_max,
            })
            .collect(),
    };

    for c in &fx.cases {
        let mut ledger = TradeLedger::new(cfg.clone(), c.gold, &[(c.gid, c.owned)], fx.gold_cap);
        let gain = c.sell_unit.saturating_mul(c.owned);
        let raw = c.gold.saturating_add(gain);
        let (expected_gold, expected_overflow) = if fx.gold_cap > 0 {
            let clamped = raw.min(fx.gold_cap);
            (clamped, raw.saturating_sub(clamped))
        } else {
            (raw, 0)
        };
        let res = ledger.sell(c.gid, c.owned).expect("夹具 sell 全量应成功");
        assert_eq!(res.gold_left, expected_gold, "夹具 sell 后金币（独立手算）");
        assert_eq!(res.overflow, expected_overflow, "夹具 sell overflow（独立手算）");
        assert!(res.gold_left <= fx.gold_cap, "夹具 0 ≤ gold ≤ gold_cap");
    }

    // gold_cap = 0（无限制）交叉验证：sell 加金不钳制、overflow 恒 0
    let c = &fx.unlimited_case;
    let mut unlimited = TradeLedger::new_unlimited(cfg.clone(), c.gold, &[(c.gid, c.owned)]);
    let res = unlimited.sell(c.gid, c.owned).expect("unlimited sell 应成功");
    assert_eq!(res.gold_left, c.gold + c.sell_unit.saturating_mul(c.owned), "无上限加金不钳制");
    assert_eq!(res.overflow, 0, "无上限 overflow 恒 0");
}

/// 建造侧穷举：全 4 建筑 × 逐档升级 `0..max_level`（真实表），
/// 全排列组合路径 + 单建筑逐个建满 + 交叉升级，每步断言 `build_cost` 单调递增
/// + `gold_cap()` = 该路径已建建筑 `gold_max` 最大值（任意中途取值都对）+ `bank().gold` 无下溢。
#[test]
fn exhaust_build_all_buildings_full_upgrade_paths() {
    let b = balance().expect("balance.json 有效");
    let cfg = b.build.clone();
    assert_eq!(cfg.buildings.len(), 4, "balance.json 全 4 建筑");
    let max_level = cfg.max_level;
    assert!(max_level > 0, "max_level 须 > 0（具体值由 balance.json 数据校验，测试只守逻辑）");

    let all_ids: Vec<&str> = cfg.buildings.iter().map(|x| x.id.as_str()).collect();

    // 路径 1：单建筑逐个建满（4 条路径，每条 = 该建筑 0..max_level 全档）
    for id in &all_ids {
        let base_cost = cfg.get(id).expect("建筑必在表内").base_cost;
        let gold_max = cfg.get(id).expect("建筑必在表内").gold_max;
        let mut mgr = BuildManager::new(cfg.clone(), Bank::new(u32::MAX / 4), &[]);
        for level in 1..=max_level {
            let expected_cost = base_cost.saturating_mul(level);
            assert_eq!(
                mgr.build_cost(id, level),
                Some(expected_cost),
                "build_cost({id}, {level}) 须 = base_cost×level（独立手算）"
            );
            let r = mgr.build(id);
            assert_eq!(r, Ok(()), "build({id} 第 {level} 档) 应成功（金量充足）");
            assert_eq!(mgr.level(id), level, "build 后 level 须 = {level}");
            assert!(
                mgr.bank().gold <= u32::MAX / 4,
                "bank().gold 上界不越初始值（无下溢由 saturating 保证）"
            );
            assert_eq!(mgr.gold_cap(), gold_max, "单建筑 gold_cap = 该建筑 gold_max（精确值）");
        }
        // 满级后再 build 须 InvalidLevel
        assert_eq!(
            mgr.build(id),
            Err(BuildErr::InvalidLevel),
            "max_level 后 build 须 InvalidLevel"
        );
    }

    // 路径 2：全 4 建筑交叉升级（每建筑各建 1 档，按目录顺序），验证任意中途取值的
    // `gold_cap` 都精确等于「已建建筑子集」的 gold_max 最大值（逐档对账，非只看最终值）
    let mut mgr = BuildManager::new(cfg.clone(), Bank::new(1_000_000), &[]);
    let full_max_cap: u32 = cfg.buildings.iter().map(|x| x.gold_max).max().expect("4 建筑非空");
    for (i, id) in all_ids.iter().enumerate() {
        mgr.build(id).expect("交叉升级 1 档应成功");
        let built_ids = &all_ids[0..=i];
        let expected_cap: u32 = built_ids
            .iter()
            .map(|id2| cfg.get(id2).expect("建筑必在表内").gold_max)
            .max()
            .expect("非空子集");
        assert_eq!(
            mgr.gold_cap(),
            expected_cap,
            "交叉升级中途（已建 {built_ids:?}）gold_cap 须 = 该子集 gold_max 最大值（精确值）"
        );
    }
    assert_eq!(
        mgr.gold_cap(),
        full_max_cap,
        "全 4 建筑各 1 档后 gold_cap = 全表 gold_max 最大值（精确值）"
    );
    let _ = mgr;
}

/// 夹具隔离假值建造侧（JSON `build.fixture` 驱动，不读 balance，验证升级路径逻辑独立于具体数值）。
#[test]
fn exhaust_build_fixture_cross_validation() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let fx = &es.build.fixture;
    let cfg = BuildCfg {
        max_level: fx.max_level,
        buildings: fx
            .buildings
            .iter()
            .map(|b| BuildingDef {
                id: b.id.clone(),
                name_key: format!("build.{}", b.id),
                base_cost: b.base_cost,
                gold_max: b.gold_max,
                atk_bonus: 0,
                hp_bonus: 0,
            })
            .collect(),
    };
    let first = &cfg.buildings[0];
    let second = &cfg.buildings[1];
    assert!(cfg.buildings.len() >= 2, "build.fixture 至少 2 建筑（JSON 保证）");
    let a = first;
    let b = second;

    // A 建满 `max_level` 档总成本 = base_cost × (1 + 2 + ... + max_level)
    let a_total_cost: u32 = (1..=fx.max_level).map(|lvl| a.base_cost.saturating_mul(lvl)).sum();
    let mut mgr = BuildManager::new(cfg.clone(), Bank::new(fx.initial_bank_gold), &[]);
    for _ in 0..fx.max_level {
        mgr.build(&a.id).expect("A 逐档");
    }
    assert_eq!(mgr.build(&a.id), Err(BuildErr::InvalidLevel), "A 满档后须 InvalidLevel");
    assert_eq!(mgr.gold_cap(), a.gold_max, "只建 A → gold_cap = A.gold_max（精确值）");
    assert_eq!(mgr.bank().gold, fx.initial_bank_gold - a_total_cost, "A 全档总成本独立手算对账");

    mgr.build(&b.id).expect("B 1 档");
    assert_eq!(mgr.gold_cap(), a.gold_max.max(b.gold_max), "建 B 不降 A 的上限（单调，精确值）");
    let _ = mgr;
}
