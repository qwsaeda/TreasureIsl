//! S15.1 进度域（地图/卡牌）业务场景穷举测试（`exhaust_progression`，第 6 域）。
//!
//! 覆盖 `docs/business/progression.md`（地图逐格结算 + 卡片效果）+ `00_design.md` §1 不变式
//! 在进度域上的体现：
//! - 地图侧：`balance.selection.maps` 全 3 张真实图 × 全格 × 全方向（4）——
//!   自由几何判定 `can_walk`（确定性 + 边界钳制精确值）+ `move_to` 位置/BOSS 不变式
//!   （不越界、`moved`/`boss` 标志精确值、同 seed 逐位一致）；
//! - 卡牌侧：事件 id（`0..=card_event_id_max`）× 种子档位 × 状态档位
//!   —— `CardEngine::apply` 确定性 + 钳制不变式（`hp` 不超过 `hp_max`、`gold` 不超过 `gold_max`）
//!   + `draw_one` 同 seed 可复现。
//!
//! 枚举维度（种子/状态档位/事件 id 上界）由 `assets/data/exhaust_scenarios.json`
//! （`ti_core::data::exhaust()`）驱动；业务几何（`selection.maps`）与卡牌目录（`balance.cards`）
//! 仍读 `balance()`（单一事实源，测试只做"加载 + 逐条断言"薄壳）。

use ti_core::data::{balance, exhaust};
use ti_core::domain::map::{Dir, GridPos, MapTerrain};
use ti_core::rules::cards::{CardEngine, CardResult, CardState};
use ti_core::rules::cards_draw::CardPool;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::rng::GameRng;

/// 目标格是否「在界内」的手算基准（与 `main_map.rs::step_move` 的 `i64` 中间量同口径，
/// 全程 `i64` 不经过 `i32` 截断，防 pedantic `cast_possible_truncation`）。
fn in_bounds(row: u32, col: u32, d: Dir, grid_w: u32, grid_h: u32) -> bool {
    let (dr, dc) = d.delta();
    let nr = i64::from(row) + i64::from(dr);
    let nc = i64::from(col) + i64::from(dc);
    nr >= 0 && nc >= 0 && nr < i64::from(grid_h) && nc < i64::from(grid_w)
}

/// `usize` → `u32` 安全转换（测试枚举空间规模远小于 `u32::MAX`，越界钳到 `u32::MAX`，不 panic）。
#[allow(clippy::cast_possible_truncation)]
fn usize_as_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// 地图侧穷举（1）：全 3 真实图 × 全格 × 全方向，`can_walk` 纯几何判定 = 确定性 + 边界钳制精确值。
///
/// 开地图（无阻挡格）时 `can_walk` 当且仅当「目标格在界内」为真。逐格逐方向对手算边界，
/// 防 `step_move` 的 `i64` 中间量 / `u32` 下溢逻辑被改坏。
#[test]
fn exhaust_progression_map_can_walk_all_cells_all_dirs() {
    let b = balance().expect("balance.json valid");
    let maps = &b.selection.maps;
    assert!(!maps.is_empty(), "balance.selection.maps 非空");
    let dirs = Dir::all();

    let mut combos: u32 = 0;
    for m in maps {
        let terrain = MapTerrain::open(m.grid_w, m.grid_h);
        for row in 0..m.grid_h {
            for col in 0..m.grid_w {
                let pos = GridPos::new(row, col);
                for &d in &dirs {
                    let expected = in_bounds(row, col, d, m.grid_w, m.grid_h);
                    let actual = MapSession::can_walk(&terrain, pos, d);
                    assert_eq!(
                        actual, expected,
                        "can_walk(图{m:?} 格({row},{col}) {d:?}) 须 == 目标在界内 {expected}（手算精确值）"
                    );
                    // 确定性：同输入再查一次必一致（纯几何，无 RNG）
                    assert_eq!(
                        actual,
                        MapSession::can_walk(&terrain, pos, d),
                        "can_walk 纯函数须确定性"
                    );
                    combos += 1;
                }
            }
        }
    }
    let expected: u32 = maps.iter().map(|m| m.grid_w * m.grid_h * usize_as_u32(dirs.len())).sum();
    assert_eq!(combos, expected, "组合数 = Σ(每图 grid_w×grid_h×4)（JSON 驱动档位 × 真实图几何）");
}

/// 地图侧穷举（2）：全格 × 全方向 × 种子档位，`move_to` 位置/BOSS/确定性不变式。
///
/// 非法移动（越界）须保持原位（`moved=false`）；合法移动须精确到目标格；
/// 落到 BOSS 格时 `boss=true`（`progression.md` 落格结算，BOSS 优先于据点）；
/// 同 seed 两次独立跑逐字段一致。
#[test]
fn exhaust_progression_map_move_to_invariants() {
    let e = exhaust().expect("exhaust_scenarios.json should parse");
    let b = balance().expect("balance.json valid");
    let maps = &b.selection.maps;
    let seeds = &e.progression.card_seeds;
    let dirs = Dir::all();

    let mut combos: u32 = 0;
    for m in maps {
        let terrain = MapTerrain::open(m.grid_w, m.grid_h);
        let landmarks: Vec<(u32, u32)> = m.landmarks.iter().map(|&[r, c]| (r, c)).collect();
        let boss: Option<GridPos> = m.boss_tile.map(|t| GridPos::new(t[0], t[1]));
        for row in 0..m.grid_h {
            for col in 0..m.grid_w {
                let start = GridPos::new(row, col);
                for &d in &dirs {
                    for &seed in seeds {
                        let mut s1 = MapSession::with_boss(
                            terrain.clone(),
                            start,
                            &landmarks,
                            CardPool::from_balance(),
                            boss,
                        );
                        let mut rng1 = GameRng::new(seed);
                        let r1 = s1.move_to(d, &mut rng1);
                        let mut s2 = MapSession::with_boss(
                            terrain.clone(),
                            start,
                            &landmarks,
                            CardPool::from_balance(),
                            boss,
                        );
                        let mut rng2 = GameRng::new(seed);
                        let r2 = s2.move_to(d, &mut rng2);
                        // 确定性：同 seed + 同起点 + 同方向 ⇒ 逐字段一致
                        assert_eq!(
                            r1, r2,
                            "move_to 确定性违例（图{m:?} 起点({row},{col}) {d:?} seed={seed}）"
                        );
                        // 位置不变式：目标在界内则精确移动，否则原位不动
                        let moved_expect = in_bounds(row, col, d, m.grid_w, m.grid_h);
                        assert_eq!(
                            r1.moved, moved_expect,
                            "moved 标志须 == 目标在界内（手算精确值）"
                        );
                        // BOSS 标志不变式：仅当「本回合移动且落在 BOSS 格」时为真
                        let expect_boss = moved_expect && boss.is_some_and(|t| t == r1.pos);
                        assert_eq!(
                            r1.boss, expect_boss,
                            "boss 标志须精确（落格结算，BOSS 优先于据点）"
                        );
                        // 据点标志不变式：BOSS 格互斥时据点必 None（战斗优先）
                        if expect_boss {
                            assert!(
                                r1.landmark.is_none(),
                                "BOSS 格与据点互斥：boss=true 时 landmark 必 None"
                            );
                        }
                        combos += 1;
                    }
                }
            }
        }
    }
    let expected: u32 = maps
        .iter()
        .map(|m| m.grid_w * m.grid_h * usize_as_u32(dirs.len()) * usize_as_u32(seeds.len()))
        .sum();
    assert_eq!(combos, expected, "组合数 = Σ(每图 grid_w×grid_h×4×|seeds|)（JSON 驱动）");
}

/// 卡牌侧穷举（1）：全事件 id × 状态档位，`apply` 确定性 + 钳制不变式。
///
/// 两次独立跑同事件 ⇒ 终态逐字段一致（确定性，`apply` 不耗 RNG）；终态恒满足
/// `hp` 不超过 `hp_max`、`gold` 不超过 `gold_max`（0 / 上限边界钳制，
/// `progression.md` 效果结算）；`UnknownEvent` 不得改动状态。
#[test]
fn exhaust_progression_cards_apply_all_events_states() {
    let e = exhaust().expect("exhaust_scenarios.json should parse");
    let engine = CardEngine::from_balance();
    assert!(!engine.catalog().is_empty(), "balance.cards 卡牌目录非空");
    let states = &e.progression.card_state_variants;
    let event_max = e.progression.card_event_id_max;

    let mut combos: u32 = 0;
    for event_id in 0..=event_max {
        for v in states {
            let base = CardState { hp: v.hp, gold: v.gold, hp_max: v.hp_max, gold_max: v.gold_max };
            let mut a = base;
            let ra = engine.apply(event_id, &mut a);
            let mut b = base;
            let rb = engine.apply(event_id, &mut b);
            assert_eq!(a, b, "apply 确定性违例（event={event_id} 状态={v:?}）");
            assert_eq!(ra, rb, "apply 结果码确定性违例");
            // 钳制不变式：终态恒在合法区间（0 下界由 saturating_sub 保证，上限由 min 保证）
            assert!(
                a.hp <= v.hp_max && a.gold <= v.gold_max,
                "apply 后 hp/gold 越上限（event={event_id} 状态={v:?}）"
            );
            // 无处理路径事件：状态须原样不变
            if matches!(ra, CardResult::UnknownEvent) {
                assert_eq!(a, base, "UnknownEvent 不得改动状态");
            }
            combos += 1;
        }
    }
    let expected = (event_max + 1) * usize_as_u32(states.len());
    assert_eq!(combos, expected, "组合数 = (event_max+1)×|states|（JSON 驱动）");
}

/// 卡牌侧穷举（2）：`draw_one` 同 seed 可复现（全种子 × 抽牌数档位）。
///
/// 同 seed 两次独立抽牌 ⇒ 抽到的事件 id 序列逐位一致（`progression.md` 可复现抽牌）；
/// 抽中 id 必落在事件 id 空间（`1..=card_event_id_max`，`all_event_ids` 口径）。
#[test]
fn exhaust_progression_cards_draw_reproducible_all_seeds() {
    let e = exhaust().expect("exhaust_scenarios.json should parse");
    let pool = CardPool::from_balance();
    let seeds = &e.progression.card_seeds;
    let event_max = e.progression.card_event_id_max;
    let draw_counts = [1u32, 3, 5, 10];

    let mut combos: u32 = 0;
    for &seed in seeds {
        for &n in &draw_counts {
            let mut r1 = GameRng::new(seed);
            let hand1 = pool.draw_one(&mut r1, n);
            let mut r2 = GameRng::new(seed);
            let hand2 = pool.draw_one(&mut r2, n);
            assert_eq!(
                hand1, hand2,
                "draw_one 可复现违例（seed={seed} n={n}）：两次独立抽牌序列须逐位一致"
            );
            for &id in &hand1 {
                assert!(
                    id >= 1 && id <= event_max,
                    "draw_one 抽中事件 id={id} 越界（须在 1..={event_max}）"
                );
            }
            combos += 1;
        }
    }
    let expected = usize_as_u32(seeds.len() * draw_counts.len());
    assert_eq!(combos, expected, "组合数 = |seeds|×抽牌数档位（JSON 驱动）");
}
