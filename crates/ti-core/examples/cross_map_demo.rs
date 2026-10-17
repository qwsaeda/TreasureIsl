//! 跨图推进演示（`ti-core`，纯玩法层）：打满 3 图（m01→m02→m03）。
//!
//! 忠实 `docs/business/progression.md`「跨图推进」+ `loop_core.md`：
//! - 每图 = 单图完整循环（随机经据点贸易/建造 → 逐格随机路线到 BOSS → 战斗定胜负）；
//! - **击败当前图 BOSS 才解锁下一图**（`rules::progression::ProgressionState`）；
//! - 失败（BOSS 胜 / 超回合）→ **重试当前图**（换战斗 seed）；
//! - 击败 m03 = **通关**（全图通杀）。
//!
//! 角色随机选一次（跨图不换），图按解锁顺序推进。
//! - 不带参：每次抽新 seed；`--seed N` 固定复现。
//!
//! ```sh
//! cargo run -p ti-core --example cross_map_demo
//! cargo run -p ti-core --example cross_map_demo -- --seed 3
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
use ti_core::rules::progression::ProgressionState;
use ti_core::rules::rng::GameRng;
use ti_core::rules::saveload::SaveSlotV2;
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

    /// 开战单位 = 基础 + 建造加成 − 卡牌暂存扣血。
    fn battle_unit(&mut self, base: Unit, builder: &BuildManager) -> Unit {
        let atk = base.atk.saturating_add(builder.build_atk_bonus());
        let hp =
            base.hp.saturating_add(builder.build_hp_bonus()).saturating_sub(self.pending_damage);
        self.pending_damage = 0;
        Unit { hp, atk }
    }
}

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
                "[{label}] ★ 到达 ({tr},{tc})（共 {steps} 步）",
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

/// 据点经济：随机买/卖 → 用宿主当前金注资建造 → 随机建造 1 级。返回注资后的 `BuildManager`。
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
    let mut builder = BuildManager::new(b.build.clone(), Bank::new(host.gold), &[]);
    let n_build = u32::try_from(b.build.buildings.len()).unwrap_or(4);
    let pick = rng.in_range(n_build.saturating_add(1));
    if pick < n_build {
        let id = &b.build.buildings[usize::try_from(pick).unwrap_or(0)].id;
        let cap0 = builder.gold_cap();
        match builder.build(id) {
            Ok(()) => host.gold = builder.bank().gold,
            Err(e) => assert!(matches!(e, BuildErr::Insufficient), "建造仅可被金不足拒（{e:?}）"),
        }
        println!(
            "    ◆ 据点：随机买 {qty} 件 + 建 {id} → 金上限 {cap0}→{}，宿主金 → {}",
            builder.gold_cap(),
            host.gold
        );
    } else {
        println!("    ◆ 据点：随机买 {qty} 件 + 不建（pick 末位）→ 宿主金 → {}", host.gold);
    }
    builder
}

/// 打一局某图（单图完整循环）：随机经据点(经济) → 逐格随机到 BOSS → 战斗。
/// 返回是否击败 BOSS（Player 胜）。
#[allow(clippy::too_many_lines)]
fn play_map(m: MapSlot, b: &ti_core::data::Balance, rng: &mut GameRng) -> bool {
    let base_player = b.combat.player.expect("combat.player 在表");
    let map_def = &b.selection.maps[usize::try_from(m.zero_index()).unwrap_or(0)];
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

    println!(
        "\n─── 图 {}（{}×{}，BOSS 档 {}，起点 ({sr},{sc}) → BOSS ({br},{bc})）───",
        m.label(),
        map_def.grid_w,
        map_def.grid_h,
        map_def.boss_tier,
        sr = start.row,
        sc = start.col,
        br = boss.row,
        bc = boss.col,
    );

    // 探索：随机经一个据点（经济），再走 BOSS
    let builder = if landmarks.is_empty() {
        println!("    本图无据点，直接往 BOSS 走（无建造加成）");
        BuildManager::new(b.build.clone(), Bank::new(0), &[])
    } else {
        let n_lm = u32::try_from(landmarks.len()).unwrap_or(1);
        let idx = usize::try_from(rng.in_range(n_lm)).unwrap_or(0);
        let (lr, lc) = landmarks[idx];
        let stop = GridPos::new(lr, lc);
        println!("    探索：随机经据点 ({lr},{lc})（贸易/建造）");
        walk_to_target(
            &mut session_map,
            stop,
            &pool,
            &mut host,
            rng,
            b,
            map_def.grid_h,
            map_def.grid_w,
            "探",
        );
        landmark_economy(b, &mut host, rng)
    };
    walk_to_target(
        &mut session_map,
        boss,
        &pool,
        &mut host,
        rng,
        b,
        map_def.grid_h,
        map_def.grid_w,
        "近",
    );

    // 战斗
    let battle_seed = rng.next().unsigned_abs();
    let unit = host.battle_unit(base_player, &builder);
    let mut bs =
        BattleSession::new_with_cfg(b.combat.clone(), battle_seed, unit, map_def.boss_tier);
    let (_trace, winner, rounds) = bs.finish();
    assert!(rounds <= b.combat.max_rounds, "战斗超回合上限（不变式）");
    let won = winner == ti_core::domain::combat::Side::Player;
    println!(
        "    战斗：玩家 hp={} atk={} vs BOSS 档 {}（seed={battle_seed}）→ {rounds} 回合，{}",
        unit.hp,
        unit.atk,
        map_def.boss_tier,
        if won { "★ 玩家胜" } else { "✕ BOSS 胜" }
    );

    // 存档 roundtrip（不变式）
    let mut slot = SaveSlotV2::new("player".to_string(), format!("m{:02}", m.index()));
    slot.position = ti_core::rules::saveload::GridPos { row: boss.row, col: boss.col };
    slot.gold = host.gold;
    slot.hp = base_player.hp;
    let json = slot.to_json();
    let reparsed = SaveSlotV2::from_json(json.as_str()).expect("存档 JSON 可解析");
    assert_eq!(slot, reparsed, "存档 roundtrip 恒等（不变式）");
    won
}

/// 不带参 → 抽新 seed；`--seed N` 固定。
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

#[allow(clippy::too_many_lines)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (seed, pinned) = resolve_seed(&args);
    let b = balance().expect("balance.json 有效");
    let mut rng = GameRng::new(seed);

    // 随机选角一次（跨图不换角色）
    let n_char = u32::try_from(Character::all().len()).unwrap_or(5);
    let ch = Character::from_index(rng.in_range(n_char)).expect("角色索引在表");
    println!("===== 跨图推进（打满 3 图）seed={seed}：角色 {} =====", ch.label());

    let mut progression = ProgressionState::initial();
    let max_retry = 3u32; // 每图失败重试次数上限（避免无限）
    let mut failed = false;

    for m in MapSlot::all() {
        if !progression.can_select(m) {
            println!("\n✕ 图 {m:?} 未解锁（前图 BOSS 未击败）→ 无法推进，游戏失败");
            failed = true;
            break;
        }
        println!("\n当前解锁 = {:?}", progression.unlocked());
        // 打这张图（失败则换 seed 重试当前图，上限 max_retry 次）
        let mut attempt = 0u32;
        let mut won = false;
        while !won && attempt <= max_retry {
            won = play_map(m, b, &mut rng);
            if !won {
                attempt += 1;
                if attempt <= max_retry {
                    println!("    失败，换战斗 seed 重试当前图（第 {attempt} 次）");
                }
            }
        }
        if !won {
            println!("\n✕ 图 {} 重试 {max_retry} 次仍未胜 → 游戏失败", m.label());
            failed = true;
            break;
        }
        // 推进：击败本图 BOSS → 解锁下一图 / 通关（功能式：消费旧状态、返回新状态）
        progression = progression.on_boss_defeated(m);
        if progression.is_cleared() {
            println!("\n★ 击败 {} 的 BOSS → 通关！全 3 图通杀", m.label());
            break;
        }
        println!("\n★ 击败 {} 的 BOSS → 解锁下一图", m.label());
    }

    if !failed && progression.is_cleared() {
        println!("\n========== 通关！击败全部 3 图 BOSS（跨图推进完成）==========");
    }
    if pinned {
        println!("\n（固定 seed={seed}，重跑 --seed {seed} 可逐位复现）");
    } else {
        println!("\n（本局 seed={seed} 自动抽取；再跑一次不带参会抽新 seed）");
    }
}
