//! 随机穷举「一局」演示（`ti-core`，纯玩法层）。
//!
//! **忠实核心循环**（`docs/business/loop_core.md`）：胜利=击败 BOSS、失败=HP 归零。
//! 一局 = 随机选角+选图 → **逐格随机路线走到 BOSS 格**（m01/m02 直线 20 步，不截断）→
//! 途中落据点则贸易/建造 → 落 BOSS 格进战斗 → 胜/负。每个决策点由 seeded RNG 采样。
//!
//! - 不带参：每次跑**抽一个全新 seed**（当前时刻纳秒）→ 每局路线/抽卡/买卖/建造都不同；
//! - `--seed N`（或 `--seed=N`）：固定该 seed，重跑可逐位复现同一局。
//!
//! ```sh
//! cargo run -p ti-core --example random_session_demo              # 随机新 seed
//! cargo run -p ti-core --example random_session_demo -- --seed 7  # 固定复现
//! ```

use std::time::{SystemTime, UNIX_EPOCH};
use ti_core::data::{CardEffectDef, balance};
use ti_core::domain::combat::Unit;
use ti_core::domain::map::{Dir, GridPos, MapTerrain};
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::battle::BattleSession;
use ti_core::rules::build::{Bank, BuildErr, BuildManager};
use ti_core::rules::cards_draw::CardPool;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::rng::GameRng;
use ti_core::rules::saveload::SaveSlotV2;
use ti_core::rules::selection::SelectionFlow;
use ti_core::rules::trade::TradeLedger;

/// 宿主跨域状态（对齐 `game_app.rs` 的金 + 卡牌暂存扣血通道）。
struct Host {
    gold: u32,
    pending_damage: u32,
}

impl Host {
    fn new(gold: u32) -> Self {
        Self { gold, pending_damage: 0 }
    }

    /// 忠实 `game_app.rs:1555-1567`。
    fn apply_card(&mut self, eff: &CardEffectDef, gold_max: u32) {
        match eff {
            CardEffectDef::Gold(n) => self.gold = self.gold.saturating_add(*n).min(gold_max),
            CardEffectDef::Damage(n) => {
                self.pending_damage = self.pending_damage.saturating_add(*n);
            }
            CardEffectDef::Heal(n) => self.pending_damage = self.pending_damage.saturating_sub(*n),
            CardEffectDef::Move | CardEffectDef::Unknown => {}
        }
    }

    /// 忠实 `game_app.rs:1420-1425`：开战单位 = 基础 + 建造加成 − 卡牌暂存扣血。
    fn battle_unit(&mut self, base: Unit, builder: &BuildManager) -> Unit {
        let atk = base.atk.saturating_add(builder.build_atk_bonus());
        let hp =
            base.hp.saturating_add(builder.build_hp_bonus()).saturating_sub(self.pending_damage);
        self.pending_damage = 0;
        Unit { hp, atk }
    }
}

/// 不带参 → 每次抽新 seed（SystemTime 纳秒低 32 位）；带 `--seed N` → 固定 N（可复现）。
fn resolve_seed(args: &[String]) -> (u32, bool) {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--seed" && i + 1 < args.len() {
            if let Ok(v) = args[i + 1].parse::<u32>() {
                return (v, true);
            }
        } else if let Some(v) = args[i].strip_prefix("--seed=") {
            if let Ok(val) = v.parse::<u32>() {
                return (val, true);
            }
        }
        i += 1;
    }
    let fresh = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u32::try_from(d.as_nanos() & u128::from(u32::MAX)).unwrap_or(0));
    (fresh, false)
}

/// 朝 BOSS 缩短曼哈顿距离的方向集合（`North=row-1 / West=col-1 / South=row+1 / East=col+1`）。
/// 从中随机挑一个 → 每局路线不同，但**保证有限步内到达目标格**（无障碍地图 = 恰为曼哈顿距离步数）。
fn reducing_dirs(r: u32, c: u32, tr: u32, tc: u32) -> Vec<Dir> {
    let mut v: Vec<Dir> = Vec::new();
    if r > tr {
        v.push(Dir::North);
    } else if r < tr {
        v.push(Dir::South);
    }
    if c > tc {
        v.push(Dir::West);
    } else if c < tc {
        v.push(Dir::East);
    }
    v
}

/// 逐格「目标导向随机」走到目标格：每步从缩短曼哈顿距离的方向里随机挑一个（路线随机、必达）；
/// 每格可能抽卡（金/暂存扣血走宿主通道）。返回实际步数。
#[allow(clippy::too_many_arguments)]
fn walk_to_target(
    session_map: &mut MapSession,
    target: GridPos,
    pool: &CardPool,
    host: &mut Host,
    rng: &mut GameRng,
    b: &ti_core::data::Balance,
    grid_h: u32,
    grid_w: u32,
    label: &str,
) -> u32 {
    let mut steps = 0u32;
    loop {
        if session_map.pos() == target {
            println!(
                "[{label}] ★ 到达目标 ({tr},{tc})（共 {steps} 步）",
                tr = target.row,
                tc = target.col
            );
            return steps;
        }
        let pos = session_map.pos();
        let dirs = reducing_dirs(pos.row, pos.col, target.row, target.col);
        let d = dirs
            [usize::try_from(rng.in_range(u32::try_from(dirs.len()).unwrap_or(1))).unwrap_or(0)];
        let r = session_map.move_to(d, rng);
        steps += 1;
        if let Some(ev) = r.card_event {
            let card = pool.deck().iter().find(|c| c.id == ev).expect("抽中卡必在池内");
            let (g0, dmg0) = (host.gold, host.pending_damage);
            host.apply_card(&card.effect, b.economy.gold_max);
            assert!(host.gold <= b.economy.gold_max, "金超上限（不变式）");
            println!(
                "    步 {steps:>2} 走 {:?} → ({},{})　抽 {}(#{})：金 {g0}→{} / 暂存扣血 {dmg0}→{}",
                d, r.pos.row, r.pos.col, card.name_key, card.id, host.gold, host.pending_damage
            );
        } else {
            println!("    步 {steps:>2} 走 {:?} → ({},{})　无事件", d, r.pos.row, r.pos.col);
        }
        assert!(r.pos.row < grid_h && r.pos.col < grid_w, "位置越界（不变式）");
    }
}

/// 据点经济阶段（忠实 `loop_core`「据点交互」）：随机买/卖 → 用宿主当前金**注资建造管理器** →
/// 随机建造 1 级（金不足被拒=状态不变）。返回注资后的 `BuildManager`（其建筑加成会进后续战斗）。
fn landmark_economy(
    b: &ti_core::data::Balance,
    host: &mut Host,
    rng: &mut GameRng,
) -> BuildManager {
    let mut ledger = TradeLedger::new(b.trade.clone(), host.gold, &[], b.economy.gold_max);
    let qty = b.trade.goods.first().map_or(0, |g| rng.in_range(g.qty_max.saturating_add(1)));
    if qty > 0 {
        let _ = ledger.buy(b.trade.goods[0].id, qty);
    }
    let _ = ledger.sell(b.trade.goods[0].id, 0);
    host.gold = ledger.gold();
    // 用宿主当前金注资建造管理器（否则 Bank=0 → 任何建造都被金不足拒，加成恒 0）
    let mut builder = BuildManager::new(b.build.clone(), Bank::new(host.gold), &[]);
    let n_build = u32::try_from(b.build.buildings.len()).unwrap_or(4);
    let pick = rng.in_range(n_build.saturating_add(1));
    if pick < n_build {
        let id = &b.build.buildings[usize::try_from(pick).unwrap_or(0)].id;
        let cap0 = builder.gold_cap();
        match builder.build(id) {
            Ok(()) => host.gold = builder.bank().gold,
            Err(e) => {
                assert!(matches!(e, BuildErr::Insufficient), "建造仅可被金不足拒（{e:?}）");
            }
        }
        println!(
            "    ◆ 据点：随机买 {qty} 件 + 建 {id}（金不足则被拒）→ 金上限 {cap0}→{}，宿主金 → {}",
            builder.gold_cap(),
            host.gold
        );
    } else {
        println!("    ◆ 据点：随机买 {qty} 件 + 不建（pick 末位）→ 宿主金 → {}", host.gold);
    }
    builder
}

#[allow(clippy::too_many_lines, clippy::many_single_char_names)]
fn run_one_session(seed: u32) {
    let b = balance().expect("balance.json 有效");
    let base_player = b.combat.player.expect("combat.player 在表");
    let mut rng = GameRng::new(seed);
    let n_char = u32::try_from(Character::all().len()).unwrap_or(5);
    let n_map = u32::try_from(MapSlot::all().len()).unwrap_or(3);

    println!("===== 随机一局 seed={seed}（胜利=击败 BOSS；不指定 --seed 则每次抽新 seed）=====");

    // [1] 随机选角 + 随机选图
    let ch = Character::from_index(rng.in_range(n_char)).expect("角色索引在表");
    let m = MapSlot::from_index(rng.in_range(n_map)).expect("地图索引在表");
    let mut sel = SelectionFlow::new(&MapSlot::all());
    let _ = sel.select_role(ch);
    let _ = sel.select_map(m);
    assert!(sel.is_completed(), "角色+地图选定须 completed=true");
    let map_def = &b.selection.maps[usize::try_from(m.zero_index()).unwrap_or(0)];
    println!(
        "[1] 随机选 {} + {}（{}×{}，BOSS 档 {}，起点 ({sr},{sc})，BOSS 格 ({br},{bc})）",
        ch.label(),
        m.label(),
        map_def.grid_w,
        map_def.grid_h,
        map_def.boss_tier,
        sr = map_def.player_start[0],
        sc = map_def.player_start[1],
        br = map_def.boss_tile.map_or(0, |p| p[0]),
        bc = map_def.boss_tile.map_or(0, |p| p[1]),
    );

    // [2] 探索 + 经济 + 战斗：随机经一个据点（贸易/建造）→ 再逐格随机路线走到 BOSS 格
    let pool = CardPool::from_balance();
    let start = GridPos::new(map_def.player_start[0], map_def.player_start[1]);
    let landmarks: Vec<(u32, u32)> = map_def.landmarks.iter().map(|p| (p[0], p[1])).collect();
    let boss = map_def.boss_tile.map_or(start, |p| GridPos::new(p[0], p[1]));
    let mut session_map = MapSession::with_boss(
        MapTerrain::open(map_def.grid_w, map_def.grid_h),
        start,
        &landmarks,
        pool.clone(),
        map_def.boss_tile.map(|p| GridPos::new(p[0], p[1])),
    );
    let mut host = Host::new(b.economy.initial_gold);

    // 2a. 随机经一个据点（探索→贸易/建造）；本图无据点则直接去 BOSS。
    // builder：据点阶段用宿主当前金注资（返回真能建造的 BuildManager）；无据点 = 空管理器（无加成）。
    let builder = if landmarks.is_empty() {
        println!("[2] 探索：本图无据点，直接往 BOSS ({},{}) 走（无建造加成）", boss.row, boss.col);
        BuildManager::new(b.build.clone(), Bank::new(0), &[])
    } else {
        let n_lm = u32::try_from(landmarks.len()).unwrap_or(1);
        let idx = usize::try_from(rng.in_range(n_lm)).unwrap_or(0);
        let (lr, lc) = landmarks[idx];
        let stop = GridPos::new(lr, lc);
        println!(
            "[2] 探索：先随机经据点 ({lr},{lc})（贸易/建造），再往 BOSS ({},{}) 走",
            boss.row, boss.col
        );
        walk_to_target(
            &mut session_map,
            stop,
            &pool,
            &mut host,
            &mut rng,
            b,
            map_def.grid_h,
            map_def.grid_w,
            "2a",
        );
        landmark_economy(b, &mut host, &mut rng)
    };

    // 2b. 逐格随机路线走到 BOSS 格（每格可能抽卡）
    walk_to_target(
        &mut session_map,
        boss,
        &pool,
        &mut host,
        &mut rng,
        b,
        map_def.grid_h,
        map_def.grid_w,
        "2b",
    );

    // [3] 战斗（玩家单位 = 基础 + 建造加成 − 卡牌暂存扣血；忠实 game_app.rs:1420-1425）
    let battle_seed = rng.next().unsigned_abs();
    let unit = host.battle_unit(base_player, &builder);
    let mut bs =
        BattleSession::new_with_cfg(b.combat.clone(), battle_seed, unit, map_def.boss_tier);
    let (_trace, winner, rounds) = bs.finish();
    assert!(rounds <= b.combat.max_rounds, "战斗超回合上限（不变式）");
    println!(
        "[3] 战斗：玩家 hp={} atk={}（基础({},{}) + 建造(+{},+{}) − 暂存扣血）vs BOSS 档 {tier}；seed={battle_seed} → {rounds} 回合",
        unit.hp,
        unit.atk,
        base_player.hp,
        base_player.atk,
        builder.build_hp_bonus(),
        builder.build_atk_bonus(),
        tier = map_def.boss_tier,
    );

    // [4] 结局（loop_core：胜利=击败 BOSS；失败=HP 归零 / 超回合兜底）
    let outcome = match winner {
        ti_core::domain::combat::Side::Player => "★ 胜利（击败 BOSS，本图通关）",
        ti_core::domain::combat::Side::Enemy => "✕ 失败（BOSS 胜；HP 归零或超 max_rounds 兜底）",
    };
    println!("[4] 结局：{outcome}");

    // [5] 存档 roundtrip（不变式）
    let mut slot = SaveSlotV2::new(ch.label().to_string(), format!("m{:02}", m.index()));
    slot.position = ti_core::rules::saveload::GridPos {
        row: session_map.pos().row,
        col: session_map.pos().col,
    };
    slot.gold = host.gold;
    slot.hp = base_player.hp;
    let json = slot.to_json();
    let reparsed = SaveSlotV2::from_json(json.as_str()).expect("存档 JSON 可解析");
    assert_eq!(slot, reparsed, "存档 roundtrip 恒等（不变式）");
    println!(
        "[5] 存档：角色 {} 位置({},{}) 金 {} → JSON roundtrip 恒等 ✓",
        ch.label(),
        slot.position.row,
        slot.position.col,
        host.gold
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (seed, pinned) = resolve_seed(&args);
    run_one_session(seed);
    if pinned {
        println!("\n（固定 seed={seed}，重跑 --seed {seed} 可逐位复现本局）");
    } else {
        println!("\n（本局 seed={seed} 自动抽取；再跑一次不带参会抽新 seed，得到不同的一局）");
    }
}
