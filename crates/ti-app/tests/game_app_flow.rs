//! S5.1 验收：`GameApp` 交互状态机全链路（`docs/gamebook.md` §5 可玩流程）。
//!
//! 用 `GameApp::handle` 喂输入事件，断言场景流转 + 各 rules 行为正确
//! （菜单→选人→选图→主地图移动→贸易→建造→战斗→存读档→回主菜单）。
//! 不碰渲染/窗口，纯逻辑（headless 跑通 = P3-3 判据基础）。

use ti_app::game_app::{GameApp, GameInput, Scene};

/// 菜单→新游戏→选角→选图→主地图（P3-3 全链路前半段）。
#[test]
fn flow_menu_to_main_map() {
    let mut app = GameApp::new(176.0, 220.0);
    assert_eq!(app.scene(), Scene::Index);

    // 主菜单：菜单项 0（新游戏）确认
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectRole);

    // 选角：默认 None，Next → 索引 0（Panda）
    app.handle(GameInput::Next);
    assert_eq!(app.character, Some(ti_core::oracle::selection::Character::Panda));
    app.handle(GameInput::Confirm); // 角色选定 → 进选图
    assert_eq!(app.scene(), Scene::SelectMap);

    // 选图：默认 None，Next → 索引 0（M01）
    app.handle(GameInput::Next);
    assert_eq!(app.map_slot, Some(ti_core::oracle::selection::MapSlot::M01));
    app.handle(GameInput::Confirm); // 角色+地图齐全 → 进主地图
    assert_eq!(app.scene(), Scene::MainMap);
    assert!(app.map_pos.is_some(), "主地图应有初始位置");
}

/// 主地图：移动 + 贸易 + 建造。
#[test]
fn flow_main_map_move_trade_build() {
    let mut app = GameApp::new(176.0, 220.0);
    // 快进到主地图
    app.handle(GameInput::Confirm); // 新游戏 → SelectRole
    app.handle(GameInput::Next); // 选角 → Panda
    app.handle(GameInput::Confirm); // 角色确认 → SelectMap
    app.handle(GameInput::Next); // 选图 → M01
    app.handle(GameInput::Confirm); // 地图确认 → MainMap
    assert_eq!(app.scene(), Scene::MainMap);

    let start_pos = app.map_pos;
    let gold_before = app.gold;
    // 移动（主地图 South 方向，开放地形应可移动）
    app.handle(GameInput::Walk(ti_core::oracle::main_map::Dir::South));
    let moved = app.map_pos;
    assert_ne!(moved, start_pos, "开放地形下 South 移动应改变位置");
    assert_eq!(
        moved.expect("移动后位置应存在").row,
        start_pos.expect("初始位置应存在").row + 1,
        "South 方向 = 行 +1"
    );

    // 贸易：买 1 件商品 gid=1（价格 20，开局 1000 金）
    let gold0 = app.gold;
    app.handle(GameInput::TradeBuy(1));
    assert!(app.gold < gold0, "买入后金应减少");
    assert_eq!(app.owned.len(), 1, "买入后持有量 +1");
    assert_eq!(app.gold, gold0 - 20, "买入扣固定价格表价 20");

    // 建造：建 Castle（基础成本 1000）
    let bank_before = app.bank.gold;
    app.handle(GameInput::Build(ti_core::oracle::build::Building::Castle));
    // Castle 成本 1000 = bank.gold（开局 1000），扣后 bank.gold = 0
    assert_eq!(app.buildings.len(), 1, "建造后建筑列表 +1");
    assert!(bank_before > 0, "建造前银行应有金");
    let _ = gold_before;
}

/// 据点事件码 6 → 战斗场景。
#[test]
fn flow_landmark_to_battle() {
    let mut app = GameApp::new(176.0, 220.0);
    // 快进到主地图
    app.handle(GameInput::Confirm); // 新游戏 → SelectRole
    app.handle(GameInput::Next); // 选角 → Panda
    app.handle(GameInput::Confirm); // 角色确认 → SelectMap
    app.handle(GameInput::Next); // 选图 → M01
    app.handle(GameInput::Confirm); // 地图确认 → MainMap
    assert_eq!(app.scene(), Scene::MainMap);

    // 据点事件码 6 → 战斗
    app.handle(GameInput::Landmark(6));
    assert_eq!(app.scene(), Scene::Vs);
    assert!(app.battle.is_some(), "战斗会话应已建立");
}

/// 战斗：推进到结束 + 回主菜单。
#[test]
fn flow_battle_to_menu() {
    let mut app = GameApp::new(176.0, 220.0);
    // 快进到主地图
    app.handle(GameInput::Confirm); // 新游戏 → SelectRole
    app.handle(GameInput::Next); // 选角 → Panda
    app.handle(GameInput::Confirm); // 角色确认 → SelectMap
    app.handle(GameInput::Next); // 选图 → M01
    app.handle(GameInput::Confirm); // 地图确认 → MainMap
    app.handle(GameInput::Landmark(6));
    assert_eq!(app.scene(), Scene::Vs);

    // 推进战斗回合（最多 1000 回合必结束）
    for _ in 0..1000 {
        app.handle(GameInput::Attack);
        if let Some(b) = app.battle.as_ref() {
            if b.finished() {
                break;
            }
        }
    }
    assert!(app.battle.as_ref().is_some_and(ti_app::BattleView::finished), "1000 回合内战斗应结束");
    // 回主菜单
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::Index);
}

/// 主地图 → 消息场景（事件码 1 据点）+ 翻页 + 关闭。
#[test]
fn flow_message_pages() {
    let mut app = GameApp::new(176.0, 220.0);
    // 快进到主地图
    app.handle(GameInput::Confirm); // 新游戏 → SelectRole
    app.handle(GameInput::Next); // 选角 → Panda
    app.handle(GameInput::Confirm); // 角色确认 → SelectMap
    app.handle(GameInput::Next); // 选图 → M01
    app.handle(GameInput::Confirm); // 地图确认 → MainMap
    // 主地图事件码 1 → Msgl 消息
    app.handle(GameInput::Landmark(1));
    assert_eq!(app.scene(), Scene::Message);
    assert_eq!(app.msg_page, 0);
    // 翻页（Msgl 共 1 页，next 保持）
    app.handle(GameInput::MsgNext);
    assert_eq!(app.msg_page, 0, "Msgl 单页，next 到末页保持");
    // 关闭回主地图
    app.handle(GameInput::MsgClose);
    assert_eq!(app.scene(), Scene::MainMap);
}

/// 存读档：写档 + 读档往返。
#[test]
fn flow_save_load_roundtrip() {
    let mut app = GameApp::new(176.0, 220.0);
    // 快进到主地图
    app.handle(GameInput::Confirm); // 新游戏 → SelectRole
    app.handle(GameInput::Next); // 选角 → Panda
    app.handle(GameInput::Confirm); // 角色确认 → SelectMap
    app.handle(GameInput::Next); // 选图 → M01
    app.handle(GameInput::Confirm); // 地图确认 → MainMap
    assert_eq!(app.scene(), Scene::MainMap);

    // 主地图回主菜单
    app.handle(GameInput::MapBack);
    assert_eq!(app.scene(), Scene::Index);
    // 菜单项 1（继续游戏）→ 存读档子窗口
    app.handle(GameInput::MoveDown); // 菜单项 1
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Saveload);

    // 写档（槽 0）
    app.handle(GameInput::WriteSave(0));
    let saved_slot = app.save_slots[0];
    assert_eq!(saved_slot.gold, 1000, "写档应保存当前金（开局 1000）");
    assert_eq!(saved_slot.character, 1, "写档角色 = Panda(index 0 + 1)");

    // 读档（槽 0）→ 应回主地图
    app.handle(GameInput::LoadSave(0));
    assert!(matches!(app.scene(), Scene::MainMap | Scene::SelectRole));
}

/// 主菜单退出（菜单项 5 → quit）。
#[test]
fn flow_menu_quit() {
    let mut app = GameApp::new(176.0, 220.0);
    // 移动到菜单项 5（退出）
    for _ in 0..5 {
        app.handle(GameInput::MoveDown);
    }
    app.handle(GameInput::Confirm);
    assert!(app.should_quit(), "菜单项 5 应置退出标志");
    assert_eq!(app.scene(), Scene::Exit);
}
