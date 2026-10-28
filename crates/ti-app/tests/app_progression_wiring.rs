//! 跨图推进 app 层接线验收（`docs/business/progression.md` §跨图推进 定稿 → 接线）。
//!
//! 覆盖三条接线（`crates/ti-app/src/game_app.rs`）：
//! 1. 新档 `unlocked_maps` = 规则层初始值（仅 m01，`ProgressionState::initial_unlocked`）；
//! 2. 击败当前图 BOSS → 解锁下一图（`battle_step` 结算钩，走规则层、幂等）；
//! 3. 读档按存档 `unlocked_mask` 忠实还原解锁链（`restore_save`）：非零掩码逐位还原
//!    （含 CLEARED 位 bit 3 = 通关门禁），`mask=0` 旧档退「最小前置链」可回读；
//! 4. 通关门禁（`is_cleared` 逐图）：新档 false；m03 胜（`battle_step`）置 true；
//!    读档按 CLEARED 位还原。
//!
//! 确定性约定（EXP-040）：战斗模型 `atk_plus_flat`（伤害 = `atk` + 0..=3）+ `balance.combat`
//! （player hp 2000/atk 60；`boss_tiers` 依次为 2400/10、4800/12、7200/14）
//! ⇒ **档 0/1 固定胜**（最坏承伤：档 0 = 39×13=507、档 1 = 79×15=1185，均 < 2000；
//! 本测试仅打这两档）；档 2 最坏 119×17=2023 > 2000 非固定胜且 seed 敏感，不在本测试断言。

use ti_app::game_app::{BattleView, GameApp, GameInput, Scene};
use ti_core::domain::combat::Side;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::MapSlot;

/// 新游戏 → 选角（Panda）→ 选图 M01 → 主地图。
fn enter_m01(app: &mut GameApp) {
    app.handle(GameInput::Confirm); // 主菜单 0 = 新游戏 → SelectRole
    app.handle(GameInput::Next); // Panda
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Next); // 首次 Next 置 M01
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap, "M01 应可进入主地图");
}

/// 据点事件 6 = 进 BOSS 战斗，打到结果落定（≤1000 回合）。
fn run_boss_battle_to_end(app: &mut GameApp) {
    app.handle(GameInput::Landmark(6));
    assert_eq!(app.scene(), Scene::Vs, "事件 6 应进战斗");
    for _ in 0..1000 {
        app.handle(GameInput::Attack);
        if app.battle().is_some_and(BattleView::finished) {
            break;
        }
    }
    assert!(app.battle().is_some_and(BattleView::finished), "战斗应在 1000 回合内结束");
    assert!(
        app.battle().is_some_and(|b| b.outcome() == Some(Side::Player)),
        "无建筑加成时 BOSS 档 0/1 固定胜（承伤 < 2000 hp）"
    );
}

/// 接线 1：新档仅 m01 解锁；锁定图 Confirm 被拒（停留选图、选择态不变）。
#[test]
fn app_new_game_starts_with_only_m01_unlocked() {
    let mut app = GameApp::new(1080.0, 1920.0);
    assert_eq!(
        app.unlocked_maps,
        vec![MapSlot::M01],
        "新档解锁集合 = 规则层初始值（仅 m01，非 MapSlot::all() 全开）"
    );

    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm); // → SelectMap
    // 锁定 m02 / m03：Confirm 应被拒（Sfx::Invalid，场景停留、选择态不变）
    app.handle(GameInput::Next); // M01
    app.handle(GameInput::Next); // M02
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectMap, "未解锁 m02 的 Confirm 应被拒（停留选图）");
    assert_eq!(app.selection.map_slot, None, "被拒后选择态不应提交 m02");
    app.handle(GameInput::Next); // M03
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectMap, "未解锁 m03 的 Confirm 应被拒（停留选图）");
    // 解锁 m01：正常进主地图
    app.handle(GameInput::Next); // 循环回 M01
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "m01 已解锁应可进主地图");
}

/// 接线 2：击败 m01 BOSS → 解锁 m02；战斗结束后重复结算不改变集合（幂等）。
#[test]
fn app_boss_victory_unlocks_next_map_and_is_idempotent() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(42);
    app.set_rng_seed(42);
    enter_m01(&mut app);
    assert_eq!(app.unlocked_maps, vec![MapSlot::M01]);

    run_boss_battle_to_end(&mut app);
    assert_eq!(app.unlocked_maps, vec![MapSlot::M01, MapSlot::M02], "击败 m01 BOSS 后应解锁 m02");

    // 战斗已结束：继续攻击不改变解锁集合（结算钩幂等）
    for _ in 0..5 {
        app.handle(GameInput::Attack);
    }
    assert_eq!(app.unlocked_maps, vec![MapSlot::M01, MapSlot::M02], "重复结算不应改变解锁集合");
}

/// 接线 2 全链路：m01 胜 → 选 m02 → 胜 → 全解锁（实机可玩路径的跨图闭环）。
#[test]
fn app_full_chain_m01_then_m02_unlocks_all_maps() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    enter_m01(&mut app);
    run_boss_battle_to_end(&mut app);
    assert_eq!(app.unlocked_maps, vec![MapSlot::M01, MapSlot::M02]);

    // 战斗结束 → 主菜单（菜单选中位保持 0 = 新游戏）→ 重进选图（m02 此时应可选）
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::Index, "战斗结束按返回应回主菜单");
    assert_eq!(app.menu_selected(), 0, "菜单选中位应保持 0（新游戏）");
    app.handle(GameInput::Confirm); // 新游戏 → SelectRole（reset_selection）
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm); // → SelectMap

    // 光标态不确定（reset_selection 后可能 None 或残留 M01）：循环 Next 到 M02
    for _ in 0..3 {
        if app.map_slot() == Some(MapSlot::M02) {
            break;
        }
        app.handle(GameInput::Next);
    }
    assert_eq!(app.map_slot(), Some(MapSlot::M02));
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "击败 m01 后 m02 应可选并进主地图");

    run_boss_battle_to_end(&mut app);
    assert_eq!(
        app.unlocked_maps,
        vec![MapSlot::M01, MapSlot::M02, MapSlot::M03],
        "击败 m02 BOSS 后应解锁 m03（全图开放）"
    );
}

/// 接线 3：读档按存档 `map_slot` 的最小前置链恢复解锁（schema 不变，旧档可回读）；
/// 读档恢复 `gold`/位置（Q007：`enter_main_map` 初始值铺底后须再覆写存档值）。
#[test]
fn app_save_load_restores_unlock_chain() {
    let b = ti_core::data::balance().expect("balance.json 加载失败");
    // 存档 map_slot=1（M02）→ 恢复 [m01, m02]；position = m02 赛道起点格行主序索引 + gold=500
    // （S28.3 31 格环：位置按「在赛道上」取档，读档精确吸附，无歧义）
    let gw = b.selection.maps.iter().find(|m| m.index == 1).map_or(20, |d| d.grid_w);
    let start_cell = ti_core::data::board()
        .expect("board.json")
        .map("m02")
        .expect("m02")
        .spaces
        .first()
        .expect("起点格");
    let mut app = GameApp::new(1080.0, 1920.0);
    let save = SaveSlot {
        version: 2,
        character: 1,
        gold: 500,
        map_slot: 1,
        position: start_cell.row * gw + start_cell.col,
        rng_steps: 0,
        unlocked_mask: 0,
        boat_level: 1,
    };
    app.write_save_slot(0, save);
    app.restore_save(app.save_slots()[0]);
    assert_eq!(app.scene(), Scene::MainMap, "读档应进主地图");
    assert_eq!(app.map_slot(), Some(MapSlot::M02));
    assert_eq!(
        app.unlocked_maps,
        vec![MapSlot::M01, MapSlot::M02],
        "选中 M02 的存档 ⇒ 解锁链 = [m01, m02]（最小前置）"
    );
    // Q007：读档恢复存档值（不被 enter_main_map 的 initial_gold 覆写）
    assert_eq!(app.gold(), 500, "读档后金应恢复存档值（Q007）");
    assert_eq!(app.bank().gold, 500, "读档后银行金应恢复存档值（Q007）");
    assert_eq!(
        app.map_pos(),
        Some(ti_core::domain::map::GridPos::new(start_cell.row, start_cell.col)),
        "读档后位置应精确吸附到存档赛道格（31 格环）"
    );

    // 旧档回归：存档 map_slot=2（M03）→ 全解锁（多解锁值可回读）
    let mut app2 = GameApp::new(1080.0, 1920.0);
    let save2 = SaveSlot {
        version: 2,
        character: 1,
        gold: 800,
        map_slot: 2,
        position: 0,
        rng_steps: 0,
        unlocked_mask: 0,
        boat_level: 1,
    };
    app2.write_save_slot(0, save2);
    app2.restore_save(app2.save_slots()[0]);
    assert_eq!(app2.map_slot(), Some(MapSlot::M03));
    assert_eq!(
        app2.unlocked_maps,
        vec![MapSlot::M01, MapSlot::M02, MapSlot::M03],
        "旧档（高图直选）按链回读 = 全解锁，不迁移 schema"
    );
    assert_eq!(app2.gold(), 800, "读档后金应恢复存档值 800（Q007）");
}

/// S15.2（F23）：掩码忠实读档 —— 选中 m01 但 m02 已解锁（`mask = 0b0011`）应忠实保留
/// 多解锁进度，而非退「最小前置链」（`mask=0` 旧档才退链，见接线 3）。
#[test]
fn app_save_load_restores_mask_faithfully() {
    let mut app = GameApp::new(1080.0, 1920.0);
    let save = SaveSlot {
        version: 2,
        character: 1,
        gold: 300,
        map_slot: 0, // 停在 M01（重试），但 m02 已解锁
        position: 7,
        rng_steps: 0,
        unlocked_mask: 0b0011,
        boat_level: 1,
    };
    app.write_save_slot(0, save);
    app.restore_save(app.save_slots()[0]);
    assert_eq!(app.map_slot(), Some(MapSlot::M01));
    assert_eq!(
        app.unlocked_maps,
        vec![MapSlot::M01, MapSlot::M02],
        "非零掩码忠实还原（选中 m01 但 m02 已解锁的进度不丢失）"
    );
    assert!(!app.is_cleared(), "非通关掩码读档后 cleared 应为 false");
}

/// S15.2 逐图通关门禁：通关位（掩码 bit 3）读档还原 + 新档未通关。
#[test]
fn app_save_load_restores_cleared_gate() {
    let mut app = GameApp::new(1080.0, 1920.0);
    assert!(!app.is_cleared(), "新档未通关");
    assert!(!app.is_map_cleared(MapSlot::M01));
    assert!(!app.is_map_cleared(MapSlot::M03));

    // 通关档：全解锁（0b0111）+ CLEARED 位（0b1000）= 0b1111
    let save = SaveSlot {
        version: 2,
        character: 1,
        gold: 100,
        map_slot: 2, // M03
        position: 0,
        rng_steps: 0,
        unlocked_mask: 0b1111,
        boat_level: 1,
    };
    app.write_save_slot(0, save);
    app.restore_save(app.save_slots()[0]);
    assert_eq!(
        app.unlocked_maps,
        vec![MapSlot::M01, MapSlot::M02, MapSlot::M03],
        "通关档读档后全图解锁"
    );
    assert!(app.is_cleared(), "CLEARED 位读档后应还原通关位");
    assert!(app.is_map_cleared(MapSlot::M03), "m03 击败位 = 通关位");
    assert!(app.is_map_cleared(MapSlot::M01), "通关档 m01/m02 击败位应为 true");
}
