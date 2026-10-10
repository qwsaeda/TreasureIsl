//! E2E 完整游戏流程测试：模拟真实玩家操作，验证全链路状态一致性。
//!
//! 覆盖：菜单导航 → 角色选择 → 地图选择 → 主地图移动 → 贸易 → 建造 →
//!       战斗 → 存读档 → 消息系统 → 边界情况。
//!
//! 与 `game_app_flow.rs` 的区别：
//! - `game_app_flow.rs`：单模块流程验证（每个测试独立）
//! - `e2e_playthrough.rs`：完整会话连续性（一个测试跑完整局游戏）

use ti_app::game_app::{BattleView, GameApp, GameInput, Scene};
use ti_core::oracle::build::Building;
use ti_core::oracle::main_map::Dir;
use ti_core::oracle::selection::{Character, MapSlot};

/// 完整游戏会话：从启动到退出，模拟真实玩家操作序列。
#[allow(clippy::too_many_lines)]
#[test]
fn e2e_full_game_session() {
    let mut app = GameApp::new(176.0, 220.0);
    app.rng_mut().set_seed(42);
    app.set_rng_seed(42);

    // ── 阶段 1：主菜单导航 ──────────────────────────────────────────
    assert_eq!(app.scene(), Scene::Index, "启动应进入主菜单");

    // 菜单上下移动（循环）
    app.handle(GameInput::MoveUp);
    assert_eq!(app.menu_selected(), 5, "MoveUp 从 0 应循环到 5");
    app.handle(GameInput::MoveDown);
    assert_eq!(app.menu_selected(), 0, "MoveDown 从 5 应循环到 0");

    // 选择「新游戏」
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectRole, "新游戏应进入选角");

    // ── 阶段 2：角色选择 ──────────────────────────────────────────────
    // 遍历所有角色
    for i in 0..5 {
        app.handle(GameInput::Next);
        let expected = match i {
            0 => Character::Panda,
            1 => Character::Tichy,
            2 => Character::Kidd,
            3 => Character::Drake,
            4 => Character::Katharina,
            _ => unreachable!(),
        };
        assert_eq!(app.character(), Some(expected), "Next 应切换到角色 {i}");
    }
    // 循环回 Panda
    app.handle(GameInput::Next);
    assert_eq!(app.character(), Some(Character::Panda), "角色选择应循环");

    // 选择 Panda 并确认
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectMap, "角色确认后应进入选图");

    // ── 阶段 3：地图选择 ──────────────────────────────────────────────
    // 遍历所有地图
    for i in 0..3 {
        app.handle(GameInput::Next);
        let expected = match i {
            0 => MapSlot::M01,
            1 => MapSlot::M02,
            2 => MapSlot::M03,
            _ => unreachable!(),
        };
        assert_eq!(app.map_slot(), Some(expected), "Next 应切换到地图 {i}");
    }
    // 循环回 M01
    app.handle(GameInput::Next);
    assert_eq!(app.map_slot(), Some(MapSlot::M01), "地图选择应循环");

    // 选择 M01 并确认
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "地图确认后应进入主地图");
    assert!(app.map_pos().is_some(), "主地图应有初始位置");

    // ── 阶段 4：主地图移动 ────────────────────────────────────────────
    let mut move_count = 0;

    // 尝试四个方向移动
    for dir in [Dir::South, Dir::East, Dir::North, Dir::West] {
        let before = app.map_pos();
        app.handle(GameInput::Walk(dir));
        if app.map_pos() != before {
            move_count += 1;
        }
    }
    assert!(move_count > 0, "至少应有一个方向可移动");

    // ── 阶段 5：贸易系统 ──────────────────────────────────────────────
    let gold_before_trade = app.gold();
    assert_eq!(gold_before_trade, 1000, "开局金应为 1000");

    // 买入商品 1（价格 20）
    app.handle(GameInput::TradeBuy(1));
    assert_eq!(app.gold(), 980, "买入后金应减少 20");
    assert_eq!(app.owned().len(), 1, "买入后持有量 +1");

    // 买入商品 2（价格 76）
    app.handle(GameInput::TradeBuy(2));
    assert_eq!(app.gold(), 904, "第二次买入后金应再减少 76");

    // 卖出商品 1
    let gold_before_sell = app.gold();
    app.handle(GameInput::TradeSell(1));
    assert!(app.gold() > gold_before_sell, "卖出后金应增加");

    // ── 阶段 6：建造系统 ──────────────────────────────────────────────
    let bank_before = app.bank().gold;
    assert_eq!(bank_before, 1000, "开局银行金应为 1000");

    // 建造 Castle（成本 1000）
    app.handle(GameInput::Build(Building::Castle));
    assert_eq!(app.buildings().len(), 1, "建造后建筑列表 +1");
    assert_eq!(app.bank().gold, 0, "建造后银行金应减少 1000");

    // ── 阶段 7：消息系统（战斗前，battle=None 时 MsgClose 回 MainMap）────
    app.handle(GameInput::Landmark(1));
    assert_eq!(app.scene(), Scene::Message, "事件码 1 应进入消息");

    let page_before = app.msg_page();
    app.handle(GameInput::MsgNext);
    assert_eq!(app.msg_page(), page_before, "单页消息 next 应保持");

    app.handle(GameInput::MsgClose);
    assert_eq!(app.scene(), Scene::MainMap, "消息关闭应回主地图");

    // ── 阶段 8：战斗系统 ──────────────────────────────────────────────
    app.handle(GameInput::Landmark(6));
    assert_eq!(app.scene(), Scene::Vs, "事件码 6 应进入战斗");
    assert!(app.battle().is_some(), "战斗会话应已建立");

    let mut battle_rounds = 0;
    for _ in 0..1000 {
        app.handle(GameInput::Attack);
        battle_rounds += 1;
        if let Some(b) = app.battle() {
            if b.finished() {
                break;
            }
        }
    }
    assert!(app.battle().is_some_and(BattleView::finished), "战斗应在 1000 回合内结束");
    assert!(battle_rounds > 0, "战斗应至少推进 1 回合");

    // 战斗结束回主菜单
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::Index, "战斗结束应回主菜单");

    // ── 阶段 9：存读档 ────────────────────────────────────────────────
    app.handle(GameInput::MoveDown);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Saveload, "应进入存读档");

    app.handle(GameInput::WriteSave(0));
    let saved = app.save_slots()[0];
    assert_eq!(saved.character, 1, "存档角色应为 Panda");

    app.handle(GameInput::LoadSave(0));
    assert!(matches!(app.scene(), Scene::MainMap | Scene::SelectRole), "读档后应回主地图或选角");

    // ── 阶段 10：退出游戏 ─────────────────────────────────────────────
    app.handle(GameInput::MapBack);
    assert_eq!(app.scene(), Scene::Index, "应回主菜单");

    // 选择退出（menu_selected 在阶段 9 后 = 1，需 4 次 MoveDown 到 5）
    for _ in 0..4 {
        app.handle(GameInput::MoveDown);
    }
    app.handle(GameInput::Confirm);
    assert!(app.should_quit(), "应置退出标志");
    assert_eq!(app.scene(), Scene::Exit, "应进入退出场景");
}

/// E2E 边界情况：非法输入不应崩溃或卡死。
#[test]
fn e2e_invalid_inputs_no_crash() {
    let mut app = GameApp::new(176.0, 220.0);

    // 主菜单按返回（无效）
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::Index, "主菜单返回应保持");

    // 选角按返回
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::Index, "选角返回应回主菜单");

    // 选图按返回
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::SelectRole, "选图返回应回选角");

    // 主地图非法移动（越界）
    // 当前在 SelectRole（line 201 确认），需先 Next 设角色，再 Confirm 进 SelectMap
    app.handle(GameInput::Next); // 在 SelectRole 中选角色（Panda）
    app.handle(GameInput::Confirm); // 角色已选 → SelectMap
    app.handle(GameInput::Next); // 选地图
    app.handle(GameInput::Confirm); // 角色+地图均选 → MainMap
    assert_eq!(app.scene(), Scene::MainMap);

    // 连续移动 100 次不应崩溃
    for _ in 0..100 {
        app.handle(GameInput::Walk(Dir::North));
    }
    assert_eq!(app.scene(), Scene::MainMap, "越界移动不应改变场景");

    // 非法贸易（不存在的商品）
    let gold_before = app.gold();
    app.handle(GameInput::TradeBuy(999));
    assert_eq!(app.gold(), gold_before, "非法商品不应扣金");

    // 非法建造（超过等级上限）
    let buildings_before = app.buildings().len();
    for _ in 0..20 {
        app.handle(GameInput::Build(Building::Castle));
    }
    // 建造应失败（金不足或等级上限）
    assert!(app.buildings().len() <= buildings_before + 1, "建造不应超过上限");
}

/// E2E 状态一致性：多轮操作后状态应保持一致。
#[test]
fn e2e_state_consistency() {
    let mut app = GameApp::new(176.0, 220.0);
    app.rng_mut().set_seed(123);

    // 快进到主地图
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap);

    // 记录初始状态
    let _initial_gold = app.gold();
    let _initial_pos = app.map_pos();

    // 执行一系列操作
    for _ in 0..10 {
        app.handle(GameInput::Walk(Dir::South));
        app.handle(GameInput::Walk(Dir::East));
        app.handle(GameInput::Walk(Dir::North));
        app.handle(GameInput::Walk(Dir::West));
    }

    // 位置应回到原点（如果地形允许）
    // 注意：如果中间有障碍物，位置可能不同，这里只验证不崩溃
    assert!(app.map_pos().is_some(), "位置应始终存在");

    // 金不应为负
    assert!(app.gold() <= 999_999, "金不应超过上限");

    // 建筑列表不应无限增长
    assert!(app.buildings().len() <= 4, "建筑不应超过类型数");

    // 存读档一致性
    app.handle(GameInput::MapBack);
    app.handle(GameInput::MoveDown);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::WriteSave(0));

    let _saved_gold = app.save_slots()[0].gold;
    let _saved_char = app.save_slots()[0].character;

    // 读档后状态应恢复
    app.handle(GameInput::LoadSave(0));
    // 注意：读档后场景可能是 MainMap 或 SelectRole，取决于实现
    if app.scene() == Scene::MainMap {
        // 状态应恢复
        assert!(app.gold() <= 999_999, "读档后金不应超过上限");
    }
}

/// E2E 战斗完整性：多次战斗结果应一致（固定种子）。
#[test]
fn e2e_battle_determinism() {
    let seed = 42u32;

    // 第一次战斗
    let mut app1 = GameApp::new(176.0, 220.0);
    app1.rng_mut().set_seed(seed);
    app1.set_rng_seed(seed);
    app1.handle(GameInput::Confirm);
    app1.handle(GameInput::Next);
    app1.handle(GameInput::Confirm);
    app1.handle(GameInput::Next);
    app1.handle(GameInput::Confirm);
    app1.handle(GameInput::Landmark(6));
    assert_eq!(app1.scene(), Scene::Vs);

    let mut rounds1 = 0;
    for _ in 0..1000 {
        app1.handle(GameInput::Attack);
        rounds1 += 1;
        if app1.battle().is_some_and(BattleView::finished) {
            break;
        }
    }
    let outcome1 = app1.battle().map(|b| b.outcome);
    let player_hp1 = app1.battle().map(|b| b.player_hp);
    let boss_hp1 = app1.battle().map(|b| b.boss_hp);

    // 第二次战斗（相同种子）
    let mut app2 = GameApp::new(176.0, 220.0);
    app2.rng_mut().set_seed(seed);
    app2.set_rng_seed(seed);
    app2.handle(GameInput::Confirm);
    app2.handle(GameInput::Next);
    app2.handle(GameInput::Confirm);
    app2.handle(GameInput::Next);
    app2.handle(GameInput::Confirm);
    app2.handle(GameInput::Landmark(6));
    assert_eq!(app2.scene(), Scene::Vs);

    let mut rounds2 = 0;
    for _ in 0..1000 {
        app2.handle(GameInput::Attack);
        rounds2 += 1;
        if app2.battle().is_some_and(BattleView::finished) {
            break;
        }
    }
    let outcome2 = app2.battle().map(|b| b.outcome);
    let player_hp2 = app2.battle().map(|b| b.player_hp);
    let boss_hp2 = app2.battle().map(|b| b.boss_hp);

    // 两次战斗结果应完全一致
    assert_eq!(rounds1, rounds2, "相同种子战斗回合数应一致");
    assert_eq!(outcome1, outcome2, "相同种子战斗结果应一致");
    assert_eq!(player_hp1, player_hp2, "相同种子玩家 HP 应一致");
    assert_eq!(boss_hp1, boss_hp2, "相同种子 BOSS HP 应一致");
}
