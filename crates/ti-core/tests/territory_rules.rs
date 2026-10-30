//! 领地系统规则测试（S28.1/S28.2；`docs/business/territory.md` §10 可测规则表 1-13）。
//!
//! 本文件位 `tests/`（源码内联测试被 doccheck2 拦截；EXP-052 口径）。
//! 判据：`cargo test -p ti-core --test territory_rules` 全绿。

use std::collections::BTreeMap;

use ti_core::data::{Balance, BoardMap, balance, board};
use ti_core::rules::saveload::{
    SaveSlotV2, SaveSlotV3, SaveSlotV4, SaveSlotV5, SeatSave, TerritorySave,
};
use ti_core::rules::territory::{
    DungeonOutcome, SeatIdx, SeatPool, SettleAction, SiegeOutcome, TerritoryApply, TerritoryRules,
    TerritoryState, TurnCycle,
};
use ti_core::rules::territory_v2::SeatState;

/// 测试基建：m01 赛道 + 余额表 + 全新领地账本。
fn fixture() -> (BoardMap, &'static Balance, TerritoryState) {
    let bal = balance().expect("balance.json 加载");
    let db = board().expect("board.json 加载");
    let map = db.map("m01").expect("m01 赛道").clone();
    let state = TerritoryState::new(&map, bal).expect("领地账本构建");
    (map, bal, state)
}

/// §1：选 ≥1 玩家，其余补 AI，合计 5。
#[test]
fn seats_split_players_and_ai_to_five() {
    let cases: [&[u32]; 4] = [&[0], &[0, 2, 4], &[1], &[0, 1, 2, 3, 4]];
    for player_indexes in cases {
        let pool = SeatPool::build(player_indexes);
        assert_eq!(pool.len(), 5, "席位总数恒 5");
        assert_eq!(pool.player_count(), player_indexes.len(), "玩家席 = 选中数");
        assert_eq!(pool.ai_count(), 5 - player_indexes.len(), "AI 席补满 5");
        assert!(pool.player_count() >= 1, "至少 1 个玩家席位");
    }
}

/// §1：席位顺序 = 角色 index 升序（panda < tichy < kidd < drake < katharina）。
#[test]
fn seat_order_follows_character_index() {
    let pool = SeatPool::build(&[3, 0, 1]); // 乱序输入
    let order: Vec<&str> = pool.iter().map(|s| s.character).collect();
    assert_eq!(order, ["panda", "tichy", "kidd", "drake", "katharina"]);
    let indexes: Vec<u32> = pool.iter().map(|s| s.index).collect();
    assert_eq!(indexes, [0, 1, 2, 3, 4], "角色 index 升序 = 回合顺序");
    assert_eq!(
        pool.player_seats(),
        vec![0u8, 1, 3],
        "玩家席 = 选中的 0/1/3 角，其余（kidd=2/katharina=4）为 AI"
    );
}

/// §3：无主格副本胜利 ⇒ 易主 + level 1，该类两格同易主 + 首占奖励。
#[test]
fn capture_wins_takes_both_cells_of_isle() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    let isle = "harbour";
    let cells = state.isle_cells(isle).to_vec();
    assert_eq!(cells.len(), 2, "每类岛 2 格");
    let idx = cells[0];
    let seat: SeatIdx = 0;
    let bonus = state.apply_dungeon(&rules, idx, seat, DungeonOutcome::Win);
    assert_eq!(bonus, bal.territory.first_capture_bonus, "无主首占给 first_capture_bonus");
    for cell in &cells {
        let own = state.ownership(*cell).expect("格在账本");
        assert_eq!(own.owner, Some(seat), "两格同易主");
        assert_eq!(own.level, 1, "占领后 level = 1");
    }
}

/// §3：副本失败 ⇒ 归属不变。
#[test]
fn capture_loss_keeps_ownership() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    let isle = "wreck";
    let cell = state.isle_cells(isle)[0];
    let before = state.ownership(cell).copied();
    state.apply_dungeon(&rules, cell, 1, DungeonOutcome::Loss);
    assert_eq!(state.ownership(cell).copied(), before, "失败归属不变");
    assert_eq!(before.expect("无主格在账本").owner, None, "初始无主");
}

/// §4.2：他人格交钱 ⇒ 付 toll 给所有者，本圈产出归所有者（交钱者 Collect 归零）。
#[test]
fn paying_toll_transfers_output_to_owner() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    let isle = "trading_post";
    let cell = state.isle_cells(isle)[0];
    // 先让 AI 席 2 占领该岛（副本胜利）
    state.apply_dungeon(&rules, cell, 2, DungeonOutcome::Win);
    assert_eq!(state.ownership(cell).expect("在账本").owner, Some(2));
    // 玩家席 0 落到他人格：交钱
    let gold = bal.territory.first_capture_bonus + 1000;
    let result = rules.settle(&state, cell, 0, gold, SettleAction::PayToll);
    let owed = state.apply_pay_toll(&rules, cell, 0);
    assert_eq!(result.gold_out, owed, "settle 与 apply 金额一致");
    assert!(owed > 0, "他人格过路费 > 0");
    assert!(result.gold_out <= gold, "过路费 ≤ 行动者金币");
    // 交钱者本圈不再领该格产出（§4.2 防双吃）
    let collect = rules.settle(&state, cell, 0, gold, SettleAction::Collect);
    assert_eq!(collect.gold_in, 0, "非所有者 Collect 归零");
    let owner_collect = rules.settle(&state, cell, 2, gold, SettleAction::Collect);
    assert!(owner_collect.gold_in > 0, "所有者可领产出");
}

/// §4：他人格攻占胜利 ⇒ 夺回且 level 降为 1。
#[test]
fn siege_win_recaptures_with_level_one() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    let isle = "volcano";
    let cell = state.isle_cells(isle)[0];
    // 席 1 占领并加固到 2 级
    state.apply_dungeon(&rules, cell, 1, DungeonOutcome::Win);
    assert!(state.apply_upgrade(&rules, cell, 1), "升级到 2 级");
    assert_eq!(state.ownership(cell).expect("在账本").level, 2);
    // 席 3 攻占胜利 ⇒ 两格夺回且 level 归 1
    state.apply_siege(&rules, cell, 3, SiegeOutcome::Win);
    for c in state.isle_cells(isle).to_vec() {
        let own = state.ownership(c).expect("在账本");
        assert_eq!(own.owner, Some(3), "夺回易主");
        assert_eq!(own.level, 1, "夺回降级 = 1");
    }
    // 攻占失败 ⇒ 归属不变
    state.apply_siege(&rules, cell, 4, SiegeOutcome::Loss);
    assert_eq!(state.ownership(cell).expect("在账本").owner, Some(3));
}

/// §4.1：升级 ⇒ 产出 / 守军 / 过路费 / 加固费同步提高。
#[test]
fn upgrade_raises_output_defense_and_toll() {
    let (_map, bal, _state) = fixture();
    let rules = TerritoryRules::new(bal);
    let isle = "pirate_lair";
    for level in 1..=3u8 {
        let toll = rules.toll_at(isle, level);
        let toll_next = rules.toll_at(isle, level + 1);
        let prod = rules.production_at(isle, level);
        let prod_next = rules.production_at(isle, level + 1);
        let def = rules.defense_at(isle, level);
        let def_next = rules.defense_at(isle, level + 1);
        let cost = rules.upgrade_cost_at(isle, level);
        let cost_next = rules.upgrade_cost_at(isle, level + 1);
        assert!(toll < toll_next, "过路费随级提高（{toll} → {toll_next}）");
        assert!(prod < prod_next, "产出随级提高");
        assert!(def < def_next, "守军随级提高");
        assert!(cost < cost_next, "加固费随级提高");
    }
}

/// §3/§4.1：升级不超上限（`lighthouse`/`shipyard`/`skull_cliff`/`lightship` = 2）。
#[test]
fn upgrade_caps_at_isle_max_level() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    let isle = "lighthouse";
    assert_eq!(rules.level_cap(isle), 2, "灯塔礁上限 = 2（§3 表）");
    let cell = state.isle_cells(isle)[0];
    state.apply_dungeon(&rules, cell, 0, DungeonOutcome::Win);
    assert!(state.apply_upgrade(&rules, cell, 0), "升到 2 级");
    assert_eq!(state.ownership(cell).expect("在账本").level, 2);
    assert!(!state.apply_upgrade(&rules, cell, 0), "上限后不可再升");
    assert_eq!(state.ownership(cell).expect("在账本").level, 2, "等级冻结");
    // 全局上限 3 的岛（harbour）可升到 3
    let harbour = state.isle_cells("harbour")[0];
    state.apply_dungeon(&rules, harbour, 0, DungeonOutcome::Win);
    assert!(state.apply_upgrade(&rules, harbour, 0), "harbour 升 2");
    assert!(state.apply_upgrade(&rules, harbour, 0), "harbour 升 3");
    assert!(!state.apply_upgrade(&rules, harbour, 0), "全局上限 3");
}

/// §2：每图赛道 = 31 格 = 起点 1 + 领地 20 + 功能 10（事件 4 / 卡牌 3 / 宝藏 2 / BOSS 1）。
#[test]
fn v5_track_has_31_cells_composition() {
    let db = board().expect("board.json");
    let bal = balance().expect("balance.json");
    for (map_id, map) in &db.maps {
        assert_eq!(map.spaces.len(), 31, "{map_id} 赛道 = 31 格");
        assert_eq!(map.spaces[0].kind, "start", "{map_id} 起点 = 格 0");
        let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
        for s in &map.spaces {
            *counts.entry(&s.kind).or_default() += 1;
        }
        assert_eq!(counts.get("start"), Some(&1), "{map_id} 起点 1");
        assert_eq!(counts.get("territory"), Some(&20), "{map_id} 领地 20");
        assert_eq!(counts.get("event"), Some(&4), "{map_id} 事件 4");
        assert_eq!(counts.get("card"), Some(&3), "{map_id} 卡牌 3");
        assert_eq!(counts.get("treasure"), Some(&2), "{map_id} 宝藏 2");
        assert_eq!(counts.get("boss"), Some(&1), "{map_id} BOSS 1");
        // 10 类岛各 2 格 + 全部注册于 balance.territory
        let mut isle_counts: BTreeMap<&str, u32> = BTreeMap::new();
        for s in &map.spaces {
            if let Some(isle) = &s.isle {
                *isle_counts.entry(isle.as_str()).or_default() += 1;
            }
        }
        assert_eq!(isle_counts.len(), 10, "{map_id} 岛类 = 10");
        for (isle, n) in &isle_counts {
            assert_eq!(*n, 2, "{map_id} {isle} = 2 格");
            assert!(
                bal.territory.prototypes.iter().any(|p| p.id == *isle),
                "{map_id} {isle} 已注册于 balance.territory"
            );
        }
    }
}

/// §3：每类岛两格同属一个领地（占领其一 ⇒ 另一格同变）。
#[test]
fn isle_cells_share_one_territory() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    for isle in [
        "harbour",
        "lighthouse",
        "wreck",
        "pirate_lair",
        "shipyard",
        "trading_post",
        "skull_cliff",
        "volcano",
        "lightship",
        "boss_fort",
    ] {
        let c0 = state.isle_cells(isle)[0];
        let c1 = state.isle_cells(isle)[1];
        assert_ne!(c0, c1, "{isle} 两格不同");
        state.apply_dungeon(&rules, c0, 4, DungeonOutcome::Win);
        assert_eq!(state.ownership(c0).expect("在账本").owner, Some(4));
        assert_eq!(state.ownership(c1).expect("在账本").owner, Some(4), "{isle} 占一格 ⇒ 两格同归");
    }
}

/// §6：AI 与玩家走同一分支表（策略只选分支；同状态同分支 ⇒ 同结算结果）。
#[test]
fn ai_uses_same_settlement_table() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    let pool = SeatPool::build(&[0]);
    let ai_seat: SeatIdx = u8::try_from(pool.get(1).expect("席 1 = AI").index).unwrap_or(0);
    assert!(!pool.get(ai_seat).expect("席 1").is_player, "席 1 为 AI 席");
    let player_seat = 0u8;

    // 无主格：AI 与玩家策略都选「进副本」（同一分支表第一支）
    let cell = state.isle_cells("wreck")[0];
    let gold = bal.economy.initial_gold;
    let ai_action = rules.ai_policy(&state, cell, ai_seat, gold);
    let player_action = rules.ai_policy(&state, cell, player_seat, gold);
    assert_eq!(ai_action, SettleAction::EnterDungeon, "无主格 = 进副本");
    assert_eq!(player_action, SettleAction::EnterDungeon, "玩家同表");

    // 占领后 AI 策略自领产出（upgrade_priority 0.3 < 0.5 ⇒ 不升级，领产出）
    state.apply_dungeon(&rules, cell, ai_seat, DungeonOutcome::Win);
    let ai_own = rules.ai_policy(&state, cell, ai_seat, gold);
    assert!(
        matches!(ai_own, SettleAction::Collect | SettleAction::Maxed),
        "AI 自己格 = 领产出或已满级"
    );
    // 同状态同分支 ⇒ settle 金额 = 同一公式（所有者领产出；非所有者归零）
    let a = rules.settle(&state, cell, ai_seat, gold, ai_own);
    let b = rules.settle(&state, cell, player_seat, gold, ai_own);
    assert_eq!(a.gold_in, state.apply_collect(&rules, cell, ai_seat), "settle ≡ apply 同公式");
    assert_eq!(b.gold_in, 0, "非所有者 Collect 归零（AI 与玩家同表）");
}

/// §8/§10：存档 v3 → v4 迁移 = 单席位（主角色字段保留）+ 空领地图 + `current_seat = 0`。
#[test]
fn save_migrate_v3_to_v4_defaults() {
    let v2 = SaveSlotV2::new("drake".to_owned(), "m02".to_owned());
    let mut v3 = SaveSlotV3::migrate_v2_to_v3(&v2);
    v3.gold = 500;
    v3.boat_level = 2;
    let v4 = SaveSlotV4::migrate_v3_to_v4(&v3);
    assert_eq!(v4.version, 4, "迁移补版本号 4");
    assert_eq!(v4.character, "drake", "主角色字段保留（v3 兼容）");
    assert_eq!(v4.map, "m02");
    assert_eq!(v4.gold, 500, "金币保留");
    assert_eq!(v4.boat_level, 2, "船等级保留");
    assert!(v4.seats.is_empty(), "v3 迁移 = 空席位（读档按 §8 补 1 玩家 + 4 AI）");
    assert!(v4.territories.is_empty(), "v3 迁移 = 空领地图（全图无主）");
    assert_eq!(v4.current_seat, 0, "current_seat 缺省 0");
    // 幂等：再迁不变
    assert_eq!(v4.clone().migrate(), v4, "v4 自迁移幂等");
}

/// §8/§10：存档往返还原归属 / 金币 / 当前席位（`territories` + `seats` + `current_seat`）。
#[test]
fn save_roundtrip_restores_territories() {
    let (_map, bal, mut state) = fixture();
    let db = board().expect("board.json");
    let m01 = db.map("m01").expect("m01");
    let rules = TerritoryRules::new(bal);
    // 席 2 占领 harbour（L1）+ 席 3 占领 volcano 并加固到 2
    let harbour = state.isle_cells("harbour")[0];
    let volcano = state.isle_cells("volcano")[0];
    state.apply_dungeon(&rules, harbour, 2, DungeonOutcome::Win);
    state.apply_dungeon(&rules, volcano, 3, DungeonOutcome::Win);
    assert!(state.apply_upgrade(&rules, volcano, 3));
    // 快照（§8：owner_seat + level；无主 = 缺省不写）
    let owned: BTreeMap<u32, (u8, u8)> = state
        .snapshot()
        .into_iter()
        .filter(|(_, (owner, _))| owner.is_some())
        .map(|(k, (owner, level))| (k, (owner.unwrap_or(0), level)))
        .collect();
    assert_eq!(owned.len(), 4, "4 格有主（harbour×2 + volcano×2）");
    // 进 v4 存档
    let v2p = SaveSlotV2::new("panda".to_owned(), "m01".to_owned());
    let mut v4 = SaveSlotV4::migrate_v3_to_v4(&SaveSlotV3::migrate_v2_to_v3(&v2p));
    v4.territories.push(TerritorySave {
        map: "m01".to_owned(),
        cells: owned.clone(),
        garrisons: Default::default(),
    });
    v4.current_seat = 3;
    let json = v4.to_json();
    let back = SaveSlotV4::from_json(&json).expect("v4 JSON 往返");
    assert_eq!(back.territories.len(), 1, "领地图 1 张（m01）");
    let cells = &back.territories[0].cells;
    // 还原账本（§8：缺格 = 无主；有主格归属/等级逐格还原）
    let restored = TerritoryState::restore(
        m01,
        bal,
        cells.iter().map(|(k, (owner, level))| (*k, (Some(*owner), *level))).collect(),
    )
    .expect("restore");
    for (k, (owner, level)) in &owned {
        let own = restored.ownership(*k).expect("格在账本");
        assert_eq!(own.owner, Some(*owner), "归属还原");
        assert_eq!(own.level, *level, "等级还原");
    }
    // 无主格（lighthouse）还原后仍无主
    let lighthouse = restored.isle_cells("lighthouse")[0];
    assert_eq!(restored.ownership(lighthouse).expect("在账本").owner, None, "无主格保持无主");
    assert_eq!(back.current_seat, 3, "当前席位还原");
}

/// S28.6/§9：副本守军战 = 同一战斗模型（`BattleSession::new_with_enemy` + `defender_unit`；
/// 确定性 = 同 seed 同轨迹；守军强度随加固等级提高）。
#[test]
fn dungeon_battle_shares_combat_model() {
    let (_map, bal, state) = fixture();
    let rules = TerritoryRules::new(bal);
    let player = ti_core::domain::combat::Unit { hp: 100, atk: 30 };
    let enemy1 = rules.defender_unit("wreck", 1);
    let enemy3 = rules.defender_unit("wreck", 3);
    assert!(enemy1.hp > 0, "守军 HP > 0（defense_base × 系数）");
    assert!(enemy3.hp > enemy1.hp, "加固后守军 HP 提高（§4.1 公式）");
    assert!(enemy1.atk > 0, "守军 ATK = 守军强度 × dungeon_attack_coef");
    // 确定性：同 seed 同参数 ⇒ 同轨迹（`rules::battle` 同模型）
    let mut a = ti_core::rules::battle::BattleSession::new_with_enemy(
        bal.combat.clone(),
        7,
        player,
        enemy1,
    );
    let mut b = ti_core::rules::battle::BattleSession::new_with_enemy(
        bal.combat.clone(),
        7,
        player,
        enemy1,
    );
    let (trace_a, winner_a, rounds_a) = a.finish();
    let (trace_b, winner_b, rounds_b) = b.finish();
    assert_eq!(trace_a, trace_b, "同 seed 同轨迹");
    assert_eq!(winner_a, winner_b);
    assert_eq!(rounds_a, rounds_b);
    // 玩家强于 1 级守军 ⇒ 胜（副本胜利 = 占领，§3）
    assert_eq!(winner_a, ti_core::domain::combat::Side::Player, "强玩家胜 1 级守军");
    let _ = &state;
}
#[test]
fn ai_turn_cycle_visits_all_seats_in_order() {
    let pool = SeatPool::build(&[0, 2]); // 玩家 = panda/kidd；AI = tichy/drake/katharina
    let mut cycle = TurnCycle::new(&pool);
    assert_eq!(cycle.current, 0);
    let mut seen: Vec<SeatIdx> = Vec::new();
    for _ in 0..10 {
        seen.push(cycle.current);
        cycle.advance();
    }
    assert_eq!(seen, [0, 1, 2, 3, 4, 0, 1, 2, 3, 4], "5 席循环升序");
    assert!(cycle.is_player_seat(0) && cycle.is_player_seat(2), "玩家席标记正确");
    assert!(
        !cycle.is_player_seat(1) && !cycle.is_player_seat(3) && !cycle.is_player_seat(4),
        "AI 席标记正确"
    );
    // 热座交接：当前 = AI 席 1 ⇒ 下一个玩家席 = 2
    let mut cycle2 = TurnCycle::new(&pool);
    cycle2.advance(); // current = 1（AI）
    assert_eq!(cycle2.next_player_seat(), 2, "AI 席 1 之后 = 玩家席 2");
    // 当前 = 玩家席 2 ⇒ 下一个玩家席 = 0（循环）
    let mut cycle3 = TurnCycle::new(&pool);
    cycle3.advance();
    cycle3.advance(); // current = 2（玩家）
    assert_eq!(cycle3.next_player_seat(), 0, "玩家席 2 之后循环回 0");
}

/// S28.7/§6：AI 自动回合 = 同表结算（`ai_turn` ≡ `ai_policy` + `settle` 组合，确定性）。
#[test]
fn ai_turn_uses_same_settlement_table() {
    let (_map, bal, mut state) = fixture();
    let rules = TerritoryRules::new(bal);
    let pool = SeatPool::build(&[0]);
    let ai_seat = u8::try_from(pool.get(1).expect("席 1").index).unwrap_or(0);
    assert!(!pool.get(ai_seat).expect("席 1").is_player);
    let cell = state.isle_cells("wreck")[0];
    let gold = bal.economy.initial_gold;
    // 无主格：AI 回合 = 进副本
    let turn = rules.ai_turn(&state, ai_seat, cell, gold);
    assert_eq!(turn.action, SettleAction::EnterDungeon, "无主格 AI 选副本");
    assert_eq!(
        turn.result,
        rules.settle(&state, cell, ai_seat, gold, turn.action),
        "ai_turn ≡ 同表组合"
    );
    // 确定性：同状态同输入 ⇒ 同输出（AI 不读玩家私有状态）
    let again = rules.ai_turn(&state, ai_seat, cell, gold);
    assert_eq!(turn, again, "同状态同输入结果恒同");
    // 占领后：AI 策略改分支（upgrade_priority 0.3 < 0.5 ⇒ 领产出）
    state.apply_dungeon(&rules, cell, ai_seat, DungeonOutcome::Win);
    let own = rules.ai_turn(&state, ai_seat, cell, gold);
    assert!(matches!(own.action, SettleAction::Collect | SettleAction::Maxed), "自己格 = 领产出");
    assert_eq!(own.result.gold_in, state.apply_collect(&rules, cell, ai_seat), "产出金额 = 同公式");
}

/// §8/§10：v4 → v5 迁移 = 席位补默认人手/武器 + 领地驻军空（`save_migrate_v4_to_v5_defaults`）。
#[test]
fn save_migrate_v4_to_v5_defaults() {
    let v2p = SaveSlotV2::new("panda".to_owned(), "m01".to_owned());
    let mut v4 = SaveSlotV4::migrate_v3_to_v4(&SaveSlotV3::migrate_v2_to_v3(&v2p));
    // v4 席位 crew/arms 为 0（serde default 缺省）。
    v4.seats.push(SeatSave {
        character: "panda".to_owned(),
        is_player: true,
        gold: 500,
        position: 0,
        boat_level: 1,
        buildings: Vec::new(),
        crew: 0,
        arms: 0,
    });
    let v5 = SaveSlotV5::migrate_v4_to_v5(&v4);
    assert_eq!(v5.version, 5, "迁移补版本号 5");
    assert_eq!(v5.seats[0].crew, 20, "迁移补默认人手（船 1 级 ≈ 20）");
    assert_eq!(v5.seats[0].arms, 5, "迁移补默认武器");
    assert_eq!(v5.seats[0].gold, 500, "金币保留");
    assert!(v5.territories.is_empty(), "迁移 = 空领地图");
    // 幂等：再迁不变
    assert_eq!(v5.clone().migrate(), v5, "v5 自迁移幂等");
}

/// §8/§10：存档往返还原归属/金币/人手/武器/驻军/当前席位（`save_roundtrip_restores_territories`）。
#[test]
fn save_v5_roundtrip_restores_resources_and_garrisons() {
    let (_map, bal, mut state) = fixture();
    let db = board().expect("board.json");
    let _m01 = db.map("m01").expect("m01");
    let rules = TerritoryRules::new(bal);
    // 席 2 占领 harbour（L1 + 驻军 8）；席 3 占领 volcano（L2 + 驻军 5）。
    let harbour = state.isle_cells("harbour")[0];
    let volcano = state.isle_cells("volcano")[0];
    state.apply_dungeon(&rules, harbour, 2, DungeonOutcome::Win);
    state.apply_dungeon(&rules, volcano, 3, DungeonOutcome::Win);
    assert!(state.apply_upgrade(&rules, volcano, 3));
    // 设驻军（守恒：从舰队分出）
    let mut seat2 = SeatState::new(1000, 30, 20, 0);
    let mut seat3 = SeatState::new(1000, 30, 20, 0);
    let _ = ti_core::rules::territory_v2::capture_cell(&mut seat2, &mut state, harbour, 2, 8);
    let _ = ti_core::rules::territory_v2::set_garrison(&mut seat3, &mut state, volcano, 3, 5);
    // 进 v5 存档
    let v2p = SaveSlotV2::new("panda".to_owned(), "m01".to_owned());
    let v4 = SaveSlotV4::migrate_v3_to_v4(&SaveSlotV3::migrate_v2_to_v3(&v2p));
    let mut v5 = SaveSlotV5::migrate_v4_to_v5(&v4);
    v5.seats.push(SeatSave {
        character: "panda".to_owned(),
        is_player: true,
        gold: 1000,
        position: 0,
        boat_level: 1,
        buildings: Vec::new(),
        crew: seat2.crew,
        arms: seat2.arms,
    });
    v5.seats.push(SeatSave {
        character: "tichy".to_owned(),
        is_player: false,
        gold: 1000,
        position: 0,
        boat_level: 1,
        buildings: Vec::new(),
        crew: seat3.crew,
        arms: seat3.arms,
    });
    let owned: std::collections::BTreeMap<u32, (u8, u8)> = state
        .snapshot()
        .into_iter()
        .filter(|(_, (owner, _))| owner.is_some())
        .map(|(k, (owner, level))| (k, (owner.unwrap_or(0), level)))
        .collect();
    v5.territories.push(TerritorySave {
        map: "m01".to_owned(),
        cells: owned.clone(),
        garrisons: state.garrison_map(),
    });
    v5.current_seat = 3;
    // JSON 往返
    let json = v5.to_json();
    let back = SaveSlotV5::from_json(&json).expect("v5 JSON 往返");
    assert_eq!(back.version, 5);
    assert_eq!(back.seats[0].crew, seat2.crew, "人手往返");
    assert_eq!(back.seats[0].arms, seat2.arms, "武器往返");
    assert_eq!(back.territories[0].garrisons.get(&harbour), Some(&8), "驻军往返");
    assert_eq!(back.current_seat, 3, "当前席位往返");
}
