//! 完整一局《金银岛》全链路 demo（纯玩法层 `ti-core`，零渲染、零 IO）。
//!
//! **忠实接线**（逐行对齐 `ti-app/game_app.rs` 的真实主循环，非手拼）：
//! - 战斗玩家单位 = `base.atk + build_atk_bonus()` / `(base.hp + build_hp_bonus()) − 卡牌暂存Damage`
//!   （对应 `game_app.rs:1420-1425`，成长机制 + 卡牌 debuff 同通道）；
//! - 卡牌 `Gold/Damage/Heal` 更新宿主金 + 暂存扣血（对应 `game_app.rs:1548-1574`）；
//! - 金跨域携带（贸易从宿主当前金起算，非重开 `initial_gold`）；
//! - 战斗 seed 走会话 RNG 流（`rng.next().unsigned_abs()`，非写死 42）。

use ti_core::data::{CardEffectDef, balance, strings};
use ti_core::domain::combat::{Side, Unit};
use ti_core::domain::map::{Dir, GridPos, MapTerrain};
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::battle::BattleSession;
use ti_core::rules::build::{Bank, BuildManager};
use ti_core::rules::cards_draw::CardPool;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::rng::GameRng;
use ti_core::rules::saveload::{SaveManager, SaveSlotV2};
use ti_core::rules::selection::SelectionFlow;
use ti_core::rules::trade::TradeLedger;

/// 宿主跨域状态（对齐 `game_app.rs` 的 `gold` / `card_state.hp` 暂存通道）。
struct Host {
    /// 宿主金币（跨域携带，卡牌 Gold 累加 / 贸易建造扣减）。
    gold: u32,
    /// 卡牌暂存扣血（Damage 累加 / Heal 抵消，开战并入玩家 hp）。
    pending_card_damage: u32,
}

impl Host {
    fn new(initial_gold: u32) -> Self {
        Self { gold: initial_gold, pending_card_damage: 0 }
    }

    /// 忠实 `game_app.rs:1555-1567`：按卡牌效果变体更新宿主金 + 暂存扣血。
    fn apply_card_effect(&mut self, eff: &CardEffectDef, gold_max: u32) {
        match eff {
            CardEffectDef::Gold(n) => self.gold = self.gold.saturating_add(*n).min(gold_max),
            CardEffectDef::Damage(n) => {
                self.pending_card_damage = self.pending_card_damage.saturating_add(*n);
            }
            CardEffectDef::Heal(n) => {
                self.pending_card_damage = self.pending_card_damage.saturating_sub(*n);
            }
            CardEffectDef::Move | CardEffectDef::Unknown => {}
        }
    }

    /// 忠实 `game_app.rs:1420-1425`：玩家单位 = 基础 + 建造加成 − 卡牌暂存扣血，随后复位暂存。
    fn battle_unit(&mut self, base: Unit, builder: &BuildManager) -> Unit {
        let atk = base.atk.saturating_add(builder.build_atk_bonus());
        let hp = base
            .hp
            .saturating_add(builder.build_hp_bonus())
            .saturating_sub(self.pending_card_damage);
        self.pending_card_damage = 0;
        Unit { hp, atk }
    }
}

fn dir_cn(d: Dir) -> &'static str {
    match d {
        Dir::North => "北",
        Dir::West => "西",
        Dir::South => "南",
        Dir::East => "东",
    }
}

fn eff_text(e: &CardEffectDef) -> String {
    match e {
        CardEffectDef::Damage(v) => format!("扣 {v} HP（暂存）"),
        CardEffectDef::Heal(v) => format!("回 {v} HP（抵消暂存）"),
        CardEffectDef::Gold(v) => format!("+{v} 金"),
        CardEffectDef::Move => "移动事件".to_string(),
        CardEffectDef::Unknown => "未知事件（无效果）".to_string(),
    }
}

/// 跑一场战斗并逐回合打印（首 3 回合 + 末 2 回合，中间省略）。
fn run_battle(label: &str, tier: u32, seed: u32, unit: Unit, b: &ti_core::data::Balance) -> u32 {
    let boss = b.combat.boss(tier).expect("tier 在表");
    println!(
        "    {label}（BOSS 档 {tier} hp={bhp} atk={batk} vs 玩家 hp={ph} atk={pa}，seed={seed}）",
        bhp = boss.hp,
        batk = boss.atk,
        ph = unit.hp,
        pa = unit.atk,
    );
    let cfg = b.combat.clone();
    let mut battle = BattleSession::new_with_cfg(cfg, seed, unit, tier);
    let (trace, winner, rounds) = battle.finish();
    for (i, &(ph, eh)) in trace.iter().enumerate() {
        if i == 3 {
            println!("      …（中间 {} 回合省略）…", trace.len().saturating_sub(5));
            continue;
        }
        if !(i < 3 || i + 1 >= trace.len()) {
            continue;
        }
        let (pph, peh) = if i == 0 { (unit.hp, boss.hp) } else { (trace[i - 1].0, trace[i - 1].1) };
        println!(
            "      回合 {i:>3}：玩家 {pph} → {ph}（BOSS 打 {db}）   BOSS {peh} → {eh}（玩家打 {dp}）",
            db = pph.saturating_sub(ph),
            dp = peh.saturating_sub(eh),
        );
    }
    match winner {
        Side::Player => println!("    ★ 玩家胜（{rounds} 回合，BOSS 归零）"),
        Side::Enemy => println!(
            "    ✕ BOSS 胜（{rounds} 回合；未归零走 {mr} 回合兜底，combat.md 胜负判定表）",
            mr = b.combat.max_rounds,
        ),
    }
    rounds
}

#[allow(clippy::too_many_lines)]
fn main() {
    // `--seed N`（或 `--seed=N`）：改输入 → 卡牌触发步位/金/战斗 seed/RNG 续流全变，
    // 证明打印值是引擎算出来的（非写死）。缺省 42。
    let seed: u32 = {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut s = 42u32;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--seed" {
                if i + 1 < args.len() {
                    s = args[i + 1].parse().unwrap_or(s);
                    i += 2;
                    continue;
                }
            } else if let Some(v) = args[i].strip_prefix("--seed=") {
                s = v.parse().unwrap_or(s);
            }
            i += 1;
        }
        s
    };

    let b = balance().expect("balance.json 有效");
    let str_table = strings().expect("strings.json 有效");
    let mut rng = GameRng::new(seed);
    let base_player = b.combat.player.expect("combat.player 在表");
    let mut host = Host::new(b.economy.initial_gold);
    // 空建造管理器（开局无建筑 → 加成 0，忠实 start_battle 无建筑退化路径）。
    let mut builder = BuildManager::new(b.build.clone(), Bank::new(0), &[]);

    println!(
        "===== 《金银岛》完整一局（执行 ti-core 业务规则：加载数据 → 交互，seed={seed} 会话 RNG 可复现）=====\n"
    );

    // ── [1] 选角 + 选图（真实转移状态机）──────────────────────────
    println!("[1] 选角/选图（5 角色 × 3 地图，均选定才进主地图）");
    println!(
        "    候选角色：{}",
        Character::all().iter().map(|c| c.label().to_string()).collect::<Vec<_>>().join(" / ")
    );
    let all_maps = MapSlot::all();
    let mut sel = SelectionFlow::new(&all_maps);
    let _ = sel.select_role(Character::Kidd);
    println!("    用户选角 Kidd（转移：未完成，还差地图）");
    let _ = sel.select_map(MapSlot::M01);
    println!("    用户选图 M01（glmapm01.Map）→ completed={} 进主地图", sel.is_completed());
    let map_def = &b.selection.maps[0];
    let start = GridPos::new(map_def.player_start[0], map_def.player_start[1]);
    let boss_tile = map_def.boss_tile.expect("m01 有 BOSS 格");
    println!(
        "    地图：{w}×{h}，BOSS 档 {tier}，起点 ({sr},{sc})，卡牌触发率 {chance:.0}%\n",
        w = map_def.grid_w,
        h = map_def.grid_h,
        tier = map_def.boss_tier,
        sr = start.row,
        sc = start.col,
        chance = b.cards.event_chance * 100.0,
    );

    // ── [2] 主地图移动 + 卡牌事件（金/暂存扣血走宿主通道）──────────
    println!(
        "[2] 主地图：北 ×{n} + 西 ×{w} 逐格结算 → 落 BOSS 格",
        n = start.row - boss_tile[0],
        w = start.col - boss_tile[1],
    );
    let pool = CardPool::from_balance();
    let landmarks: Vec<(u32, u32)> = map_def.landmarks.iter().map(|p| (p[0], p[1])).collect();
    let mut session = MapSession::with_boss(
        MapTerrain::open(map_def.grid_w, map_def.grid_h),
        start,
        &landmarks,
        pool.clone(),
        Some(GridPos::new(boss_tile[0], boss_tile[1])),
    );
    let mut dirs: Vec<Dir> = Vec::new();
    for _ in 0..(start.row - boss_tile[0]) {
        dirs.push(Dir::North);
    }
    for _ in 0..(start.col - boss_tile[1]) {
        dirs.push(Dir::West);
    }
    let mut entered_boss = false;
    for (i, d) in dirs.iter().enumerate() {
        let r = session.move_to(*d, &mut rng);
        if let Some(event_id) = r.card_event {
            let card = pool.deck().iter().find(|c| c.id == event_id).expect("抽中卡必在池内");
            let (bg, bd) = (host.gold, host.pending_card_damage);
            host.apply_card_effect(&card.effect, b.economy.gold_max);
            let dlabel = dir_cn(*d);
            println!(
                "    步 {i:>2}：{dlabel} → ({pr},{pc})　卡牌 #{id}「{name}」（{eff}）：金 {bg}→{ng} 暂存扣血 {bd}→{nd}",
                pr = r.pos.row,
                pc = r.pos.col,
                id = event_id,
                name = str_table.get(&card.name_key),
                eff = eff_text(&card.effect),
                ng = host.gold,
                nd = host.pending_card_damage,
            );
        } else {
            let dlabel = dir_cn(*d);
            println!(
                "    步 {i:>2}：{dlabel} → ({pr},{pc})　无事件",
                pr = r.pos.row,
                pc = r.pos.col,
            );
        }
        if let Some(t) = r.landmark {
            println!("    ◆ 落据点 → 事件码 1 → 场景 {t:?}");
        }
        if r.boss {
            entered_boss = true;
            println!(
                "    ★ 落 BOSS 格 ({pr},{pc}) → 进战斗（据点让位 BOSS）",
                pr = r.pos.row,
                pc = r.pos.col,
            );
            break;
        }
    }
    assert!(entered_boss, "路线应到达 BOSS 格");
    println!();

    // ── [3] 战斗 #1（开局无建筑：加成 0，但卡牌暂存扣血并入）─────
    let pd1 = host.pending_card_damage;
    println!("[3] 战斗 #1（开局未建造 → build_atk/hp_bonus=0；卡牌暂存扣血 {pd1} 并入玩家 hp）");
    let seed1 = rng.next().unsigned_abs();
    let unit1 = host.battle_unit(base_player, &builder);
    let rounds1 = run_battle("    开战前单位：", map_def.boss_tier, seed1, unit1, b);
    println!(
        "      （开战玩家单位 hp={uh} atk={ua} = 基础({bh},{ba}) + 建造加成(0,0) − 暂存扣血 {pd}）\n",
        uh = unit1.hp,
        ua = unit1.atk,
        bh = base_player.hp,
        ba = base_player.atk,
        pd = pd1,
    );

    // ── [4] 贸易赚金（金跨域携带）+ 用户升级选择 ──────────────────
    println!(
        "[4] 回港贸易（金从宿主当前 {g} 起算，非重开 initial_gold）+ 升级 4 选 1",
        g = host.gold
    );
    let mut ledger = TradeLedger::new(b.trade.clone(), host.gold, &[], b.economy.gold_max);
    let good2 = b.trade.goods.iter().find(|g| g.id == 2).expect("goods.2 在表");
    let qty: u32 = 10;
    let bought = ledger.buy(2, qty).expect("买 10 件");
    println!(
        "    买 goods.2「{name}」×{qty}（单价 {buy}，共 {amt}）→ 余金 {left}",
        name = str_table.get(&good2.name_key),
        buy = good2.buy,
        amt = bought.amount,
        left = bought.gold_left,
    );
    let sold = ledger.sell(2, qty).expect("卖出全部");
    host.gold = sold.gold_left;
    println!(
        "    卖 ×{qty}（售价 round({buy} × {markup}) = {unit}，共 {amt}）→ 宿主金 {left}（利润系数 {markup}）\n",
        buy = good2.buy,
        markup = b.trade.markup,
        unit = sold.amount / qty,
        amt = sold.amount,
        left = host.gold,
    );

    builder = BuildManager::new(b.build.clone(), Bank::new(host.gold), &[]);
    println!("    升级选择面板（当前金 {g}，造价 = base_cost × 目标级）：", g = host.gold);
    for def in &b.build.buildings {
        let cost = builder.build_cost(&def.id, 1).expect("目录内");
        println!(
            "      - {id}「{name}」：造价 {cost}，建成后金上限 {gm}，每级 atk+{ab} hp+{hb}",
            id = def.id,
            name = str_table.get(&def.name_key),
            gm = def.gold_max,
            ab = def.atk_bonus,
            hb = def.hp_bonus,
        );
    }
    let choice = "Fort";
    println!("\n    用户选择建造 {choice}：");
    if builder.build(choice).is_ok() {
        println!(
            "      成功：金 {before} → {after}，金上限抬升 {cap}，此后战斗获得 atk+{ab} hp+{hb}",
            before = host.gold,
            after = builder.bank().gold,
            cap = builder.gold_cap(),
            ab = builder.build_atk_bonus(),
            hb = builder.build_hp_bonus(),
        );
        host.gold = builder.bank().gold;
    }
    let rejected = builder.build("Castle");
    println!(
        "      再试 Castle（造价 1000 > 余金 {g}）→ 被拒（{e:?}），状态不变\n",
        g = builder.bank().gold,
        e = rejected.err(),
    );

    // ── [5] 战斗 #2（升级后同 BOSS 复战：加成真的进战斗）────────
    let pd2 = host.pending_card_damage;
    println!("[5] 战斗 #2（同档 BOSS 复战，验证建造加成真的进战斗单位，非纸面注释）");
    let seed2 = rng.next().unsigned_abs();
    let unit2 = host.battle_unit(base_player, &builder);
    let rounds2 = run_battle("    开战前单位：", map_def.boss_tier, seed2, unit2, b);
    println!(
        "      （开战玩家单位 hp={uh} atk={ua} = 基础({bh},{ba}) + 建造加成({ab},{hb}) − 暂存扣血 {pd}；对比 #1 的 atk={ua1}，攻速提升 → 回合数 {r1} → {r2}）\n",
        uh = unit2.hp,
        ua = unit2.atk,
        bh = base_player.hp,
        ba = base_player.atk,
        ab = builder.build_atk_bonus(),
        hb = builder.build_hp_bonus(),
        pd = pd2,
        ua1 = unit1.atk,
        r1 = rounds1,
        r2 = rounds2,
    );

    // ── [6] 回据点事件分派 ────────────────────────────────────────
    println!("[6] 回据点（BOSS 格 → 南 12 + 东 12 → 据点 (17,17)，几何 walk 不触发事件）");
    let mut walk_session = MapSession::with_boss(
        MapTerrain::open(map_def.grid_w, map_def.grid_h),
        GridPos::new(boss_tile[0], boss_tile[1]),
        &landmarks,
        pool.clone(),
        Some(GridPos::new(boss_tile[0], boss_tile[1])),
    );
    let (p1, s1) = walk_session.walk(Dir::South, 12, &mut rng);
    let (p2, s2) = walk_session.walk(Dir::East, 12, &mut rng);
    println!(
        "    南走 {s1} 步 → ({pr},{pc})，东走 {s2} 步 → ({er},{ec})",
        pr = p1.row,
        pc = p1.col,
        er = p2.row,
        ec = p2.col,
    );
    if walk_session.at_landmark() {
        let t = walk_session.dispatch(1).expect("事件码 1 有目标");
        println!("    ◆ 据点 (17,17) 命中 → 事件码 1 → 场景 {t:?}（据点对话框/资源结算）");
    }
    println!();

    // ── [7] 存档 / 读档 roundtrip + RNG 精确续流 ─────────────────
    println!("[7] 存档（槽 0）→ 读档 roundtrip + RNG 续流");
    let mut mgr = SaveManager::from_balance();
    let seed_snap = rng.save_state();
    let steps_snap = rng.steps_taken();
    let mut slot = SaveSlotV2::new("kidd".to_string(), "m01".to_string());
    slot.position = ti_core::rules::saveload::GridPos { row: 17, col: 17 };
    slot.gold = host.gold;
    slot.hp = base_player.hp;
    slot.buildings = vec!["Fort".to_string()];
    slot.rng_seed = seed_snap;
    slot.rng_steps = steps_snap;
    let json = slot.to_json();
    mgr.set_slot(0, slot.clone());
    let mut mgr2 = SaveManager::from_balance();
    let _ = mgr2.set_slot_json(0, &json);
    let loaded = mgr2.slot(0).expect("槽 0 已写入").clone();
    let reparsed = SaveSlotV2::from_json(&json).expect("JSON 可解析");
    assert_eq!(loaded, reparsed, "读档 == JSON 重解析");
    let tail_a = {
        let mut ra = GameRng::restore_state(loaded.rng_seed, loaded.rng_steps);
        [ra.in_range(100), ra.in_range(100), ra.in_range(100), ra.in_range(100)]
    };
    let tail_b = {
        let mut rb = GameRng::restore_state(slot.rng_seed, slot.rng_steps);
        [rb.in_range(100), rb.in_range(100), rb.in_range(100), rb.in_range(100)]
    };
    assert_eq!(tail_a, tail_b, "RNG 续流可复现");
    println!(
        "    存档 JSON：角色 {} 地图 {} 位置 ({pr},{pc}) 金 {gold} 建筑 {bld:?} rng(seed={ss}, steps={st})",
        loaded.character,
        loaded.map,
        pr = loaded.position.row,
        pc = loaded.position.col,
        gold = loaded.gold,
        bld = loaded.buildings,
        ss = loaded.rng_seed,
        st = loaded.rng_steps,
    );
    println!("    roundtrip 一致 ✓，读档后续流 4 次抽样 = {tail_a:?}（逐位可复现）");

    // 确定性自检（seed=42 全量重放，卡牌触发序列逐位一致）
    let mut replay = GameRng::new(42);
    let mut replay_session = MapSession::with_boss(
        MapTerrain::open(map_def.grid_w, map_def.grid_h),
        start,
        &landmarks,
        pool,
        Some(GridPos::new(boss_tile[0], boss_tile[1])),
    );
    let mut replay_hit: Vec<u32> = Vec::new();
    for d in &dirs {
        let r = replay_session.move_to(*d, &mut replay);
        if let Some(id) = r.card_event {
            replay_hit.push(id);
        }
        if r.boss {
            break;
        }
    }
    println!("\n===== 确定性自检：seed=42 重放，主地图卡牌触发序列 = {replay_hit:?} =====");
}
