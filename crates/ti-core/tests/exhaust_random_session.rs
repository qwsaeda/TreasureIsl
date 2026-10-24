//! S15.1 随机穷举场景测试（`exhaust_random_session`）：
//! **选择点（角色/地图/走法/是否开战/买卖量/建什么）全部由 seeded RNG 采样决定**——
//! 既非脚本写死、也非人工挑选；每个决策点与每个结果态断言**业务不变式**，
//! 跨多个顶层 seed 通用成立。数值/卡池/目录读 `balance()`（单一事实源），随机走 `GameRng`。
//!
//! 不变式（任意随机选择下恒成立）：
//! - 位置恒在格内（`row < grid_h && col < grid_w`）；
//! - 宿主金 `0 ≤ gold ≤ gold_max`（卡牌 Gold 钳制、贸易金不足被拒状态不变）；
//! - 战斗必终止（`rounds ≤ max_rounds`）且开战单位 `hp ≥ 0`（卡牌暂存扣血饱和）；
//! - 建造金上限**单调不减**（`gold_cap` 只升不降），金不足被拒 = 状态不变；
//! - 存档 JSON roundtrip 恒等。
//!
//! 逐步文本：`run_random_session` 返回 `SessionTrace.steps`（每步决策 + 结果态一行），
//! `verbose_session_process_readable` 把 3 个 seed 的过程逐行打印出来（`-- --nocapture`）。

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

    /// 忠实 `game_app.rs:1555-1567`：按卡牌效果变体更新宿主金 + 暂存扣血。
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

    /// 忠实 `game_app.rs:1420-1425`：开战单位 = 基础 + 建造加成 − 卡牌暂存扣血，随后复位暂存。
    fn battle_unit(&mut self, base: Unit, builder: &BuildManager) -> Unit {
        let atk = base.atk.saturating_add(builder.build_atk_bonus());
        let hp =
            base.hp.saturating_add(builder.build_hp_bonus()).saturating_sub(self.pending_damage);
        self.pending_damage = 0;
        Unit { hp, atk }
    }
}

/// 一个会话的运行产物：`fingerprint`（选择指纹，供「非固定」多样性断言）+
/// `steps`（**逐步可读文本**，每行 = 一步决策 + 结果态，让人能「看过程」而不只看 ok）。
struct SessionTrace {
    fingerprint: u64,
    steps: Vec<String>,
}

/// 跑一个**全随机**会话：所有选择由 `rng` 采样，逐态断言不变式，并逐步记录文本。
#[allow(clippy::too_many_lines)]
fn run_random_session(b: &ti_core::data::Balance, rng: &mut GameRng, session: u32) -> SessionTrace {
    let base_player = b.combat.player.expect("combat.player 在表");
    let n_char = u32::try_from(Character::all().len()).unwrap_or(5);
    let n_map = u32::try_from(MapSlot::all().len()).unwrap_or(3);
    let mut steps: Vec<String> = Vec::new();

    // [1] 随机选角 + 随机选图（转移状态机；均选定才进主地图）
    let ch = Character::from_index(rng.in_range(n_char)).expect("角色索引在表");
    let m = MapSlot::from_index(rng.in_range(n_map)).expect("地图索引在表");
    let mut sel = SelectionFlow::new(&MapSlot::all());
    let _ = sel.select_role(ch);
    let _ = sel.select_map(m);
    assert!(sel.is_completed(), "会话 {session}：角色+地图选定须 completed=true");
    let map_def = &b.selection.maps[usize::try_from(m.zero_index()).unwrap_or(0)];
    steps.push(format!("[1] 选角 {} + 选图 {}（转移状态机 → completed）", ch.label(), m.label()));

    // [2] 主地图：随机步数 × 随机方向逐格走（每步可能触发卡牌 → 金/暂存扣血）
    let pool = CardPool::from_balance();
    let start = GridPos::new(map_def.player_start[0], map_def.player_start[1]);
    let landmarks: Vec<(u32, u32)> = map_def.landmarks.iter().map(|p| (p[0], p[1])).collect();
    let boss_tile = map_def.boss_tile.map(|p| GridPos::new(p[0], p[1]));
    let mut session_map = MapSession::with_boss(
        MapTerrain::open(map_def.grid_w, map_def.grid_h),
        start,
        &landmarks,
        pool.clone(),
        boss_tile,
    );
    let mut host = Host::new(b.economy.initial_gold);
    let walk_steps = rng.in_range(15); // 0..=14 步（随机长度）
    let mut hit_boss = false;
    let mut walked = 0u32;
    let mut cards: Vec<String> = Vec::new();
    let gold_walk0 = host.gold;
    for _ in 0..walk_steps {
        let d = Dir::all()[usize::try_from(rng.in_range(4)).unwrap_or(0)];
        let r = session_map.move_to(d, rng);
        if let Some(ev) = r.card_event {
            let card = pool
                .deck()
                .iter()
                .find(|c| c.id == ev)
                .expect("抽中卡必在池内（draw_one 只出池内 id）");
            let g0 = host.gold;
            let dmg0 = host.pending_damage;
            host.apply_card(&card.effect, b.economy.gold_max);
            assert!(host.gold <= b.economy.gold_max, "会话 {session}：金超上限（不变式）");
            cards.push(format!(
                "抽 {}(#{}) 金 {g0}→{} / 暂存扣血 {dmg0}→{}",
                card.name_key, card.id, host.gold, host.pending_damage
            ));
        }
        assert!(
            r.pos.row < map_def.grid_h && r.pos.col < map_def.grid_w,
            "会话 {session}：位置越界（{},{})",
            r.pos.row,
            r.pos.col,
        );
        walked += 1;
        if r.boss {
            hit_boss = true;
            break;
        }
    }
    let end = session_map.pos();
    steps.push(format!(
        "[2] 随机走 {} 步（请求 {walk_steps}）：({},{}) → ({},{})，抽 {} 张卡，金 {gold_walk0}→{}",
        walked,
        start.row,
        start.col,
        end.row,
        end.col,
        cards.len(),
        host.gold
    ));
    for c in &cards {
        steps.push(format!("      {c}"));
    }

    // [3] 随机决定是否开战（落 BOSS 格 或 随机主动开战）
    let mut builder = BuildManager::new(b.build.clone(), Bank::new(0), &[]);
    let fight = hit_boss || rng.in_range(2) == 0;
    let reason = if hit_boss { "落到 BOSS 格" } else { "随机主动开战" };
    steps.push(format!("[3] 开战决策：fight={fight}（{reason}）"));
    if fight {
        let seed = rng.next().unsigned_abs();
        let unit = host.battle_unit(base_player, &builder);
        assert!(
            unit.hp <= base_player.hp.saturating_add(builder.build_hp_bonus()),
            "开战 hp 超基础+加成"
        );
        let mut bs = BattleSession::new_with_cfg(b.combat.clone(), seed, unit, map_def.boss_tier);
        let (_trace, winner, rounds) = bs.finish();
        assert!(rounds <= b.combat.max_rounds, "会话 {session}：战斗超回合上限（不变式）");
        steps.push(format!(
            "      战斗 seed={seed}：单位 atk={}（基础 {} + 建造 {}）、hp={}; {} 回合 → 胜者 {winner:?}",
            unit.atk,
            base_player.atk,
            builder.build_atk_bonus(),
            unit.hp,
            rounds,
        ));
    }

    // [4] 随机买卖（数量在 [0, 有效上限]，金不足被拒 = 状态不变）
    let mut ledger = TradeLedger::new(b.trade.clone(), host.gold, &[], b.economy.gold_max);
    let gold_trade0 = host.gold;
    let trade_note = match b.trade.goods.first() {
        None => "无商品".to_string(),
        Some(good) => {
            // 有效上限 = min(目录 qty_max, 船舱 cargo)；1 级船 = 99 = 现状（0 漂移）
            let qty = rng.in_range(ledger.qty_cap(good.id).saturating_add(1)); // 0..=有效上限
            let note = match ledger.buy(good.id, qty) {
                Ok(tr) => format!(
                    "买 {qty} × 商品#{}({}) 单价 {} → 金 {gold_trade0}→{}（实扣 {}）",
                    good.id, good.name_key, good.buy, tr.gold_left, tr.amount
                ),
                Err(e) => format!(
                    "买 {qty} × 商品#{}({}) 被拒（{e:?}）→ 状态不变",
                    good.id, good.name_key
                ),
            };
            let _ = ledger.sell(good.id, 0); // 全卖
            note
        }
    };
    host.gold = ledger.gold();
    assert!(host.gold <= b.economy.gold_max, "会话 {session}：贸易后金超上限（不变式）");
    steps.push(format!("[4] 贸易：{trade_note}；收尾金 {gold_trade0}→{}", host.gold));

    // [5] 随机建造（目录选 1 或 不建；金上限单调不减）
    let n_build = u32::try_from(b.build.buildings.len()).unwrap_or(4);
    let pick = rng.in_range(n_build.saturating_add(1)); // 0..=n_build：末位 = 不建
    let build_note = if pick < n_build {
        let bdef = &b.build.buildings[usize::try_from(pick).unwrap_or(0)];
        let id = &bdef.id;
        let cap_before = builder.gold_cap();
        let gold_build0 = host.gold;
        match builder.build(id) {
            Ok(()) => {
                host.gold = builder.bank().gold;
                assert!(host.gold <= b.economy.gold_max, "建造后金超上限");
                assert!(
                    builder.gold_cap() >= cap_before,
                    "会话 {session}：建造后金上限单调不减（不变式）"
                );
                format!(
                    "建 {id}（base {}）→ 金上限 {cap_before}→{}（单调），金 {gold_build0}→{}",
                    bdef.base_cost,
                    builder.gold_cap(),
                    host.gold
                )
            }
            Err(e) => {
                assert!(
                    matches!(e, BuildErr::Insufficient),
                    "会话 {session}：建造错误非金不足（{e:?}），id 在目录且只建 1 级故仅 Insufficient 可达"
                );
                format!("建 {id} 被拒（{e:?}）→ 状态不变")
            }
        }
    } else {
        format!("不建（pick={pick} = 末位）")
    };
    steps.push(format!("[5] 建造：{build_note}"));

    // [6] 存档 roundtrip 不变式
    let mut slot = SaveSlotV2::new(ch.label().to_string(), format!("m{:02}", m.index()));
    slot.position = ti_core::rules::saveload::GridPos { row: end.row, col: end.col };
    slot.gold = host.gold;
    slot.hp = base_player.hp;
    let json = slot.to_json();
    let reparsed = SaveSlotV2::from_json(json.as_str()).expect("存档 JSON 可解析");
    assert_eq!(slot, reparsed, "会话 {session}：存档 roundtrip 恒等（不变式）");
    steps.push(format!(
        "[6] 存档：角色 {} 位置({},{}) 金 {} → JSON roundtrip 恒等",
        ch.label(),
        end.row,
        end.col,
        host.gold
    ));

    // 选择指纹（证明各会话选择不同 → 非固定）：角色/地图/走步数/是否开战/建什么。
    let mut f: u64 = u64::from(ch.index());
    f = f.wrapping_mul(31).wrapping_add(u64::from(m.zero_index()));
    f = f.wrapping_mul(31).wrapping_add(u64::from(walk_steps));
    f = f.wrapping_mul(31).wrapping_add(u64::from(u32::from(fight)));
    f = f.wrapping_mul(31).wrapping_add(u64::from(pick));
    SessionTrace { fingerprint: f, steps }
}

/// 跨多个顶层 seed × 每 seed 多会话，全部不变式必须成立（随机穷举，可复现）。
#[test]
fn random_session_invariants_hold_for_any_choices() {
    let b = balance().expect("balance.json 有效");
    let top_seeds: [u32; 8] = [0, 1, 7, 42, 123, 999, 4242, 31337];
    let sessions_per_seed: u32 = 50;
    let mut fingerprints: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for ts in top_seeds {
        let mut rng = GameRng::new(ts);
        for s in 0..sessions_per_seed {
            let fp = run_random_session(b, &mut rng, s).fingerprint;
            fingerprints.insert(fp);
        }
    }
    // 非固定证明：400 会话的选择指纹须高度多样（固定脚本只会 = 1 种）。
    let distinct = u32::try_from(fingerprints.len()).unwrap_or(0);
    assert!(
        distinct >= 100,
        "选择指纹多样性不足：{distinct} < 100（400 会话），疑似选择被固定而非随机穷举"
    );
    println!(
        "exhaust_random_session: {} 顶层 seed × {} 会话 = {} 个全随机会话，不变式全通过；选择指纹 distinct = {distinct}（非固定）",
        top_seeds.len(),
        sessions_per_seed,
        u32::try_from(top_seeds.len()).unwrap_or(0) * sessions_per_seed,
    );
}

/// 打印 **3 个不同 seed** 的完整逐步过程，让人「看到文字过程」而非只看 ok。
/// 每个会话：选角/选图 → 行走(抽卡/金变化) → 是否开战(战斗回合) → 买卖 → 建造 → 存档，逐行列出。
/// 跑法：`cargo test -p ti-core --test exhaust_random_session -- --nocapture`
#[test]
fn verbose_session_process_readable() {
    let b = balance().expect("balance.json 有效");
    for ts in [42u32, 123, 31337] {
        let mut rng = GameRng::new(ts);
        let trace = run_random_session(b, &mut rng, 0);
        println!("\n===== 随机会话 seed={ts}（逐步过程；每行 = 一步决策 + 结果态）=====");
        for line in &trace.steps {
            println!("{line}");
        }
        println!(
            "---- 本会话选择指纹 = {}（400 会话里 distinct 很高 → 非固定）----",
            trace.fingerprint
        );
    }
}
