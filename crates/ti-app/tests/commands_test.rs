//! S9.5 Command 派发验收：`GameCommand` 各变体经 `dispatch` 路由后状态变化正确，
//! 且 `dispatch` 返回值（true = 消费 / 发生了场景切换）与 `route_input` 一致。

use ti_app::commands::{Command as Cmd, GameCommand};
use ti_app::game_app::{GameApp, GameInput, Scene};
use ti_app::scenes::{MenuHost, SessionHost};
use ti_core::domain::economy::Building;

/// `From<GameCommand> for GameInput` 与 `GameInput` 的 1:1 对齐（穷举校验，§5.6.6）。
#[test]
fn command_to_input_is_exhaustive() {
    let pairs: Vec<(GameCommand, GameInput)> = vec![
        (GameCommand::MoveUp, GameInput::MoveUp),
        (GameCommand::MoveDown, GameInput::MoveDown),
        (GameCommand::Confirm, GameInput::Confirm),
        (GameCommand::Back, GameInput::Back),
        (GameCommand::Prev, GameInput::Prev),
        (GameCommand::Next, GameInput::Next),
        (GameCommand::Spin, GameInput::Spin),
        (GameCommand::Landmark(6), GameInput::Landmark(6)),
        (GameCommand::TradeBuy(1), GameInput::TradeBuy(1)),
        (GameCommand::TradeSell(2), GameInput::TradeSell(2)),
        (GameCommand::Build(Building::Castle), GameInput::Build(Building::Castle)),
        (GameCommand::Attack, GameInput::Attack),
        (GameCommand::LoadSave(0), GameInput::LoadSave(0)),
        (GameCommand::WriteSave(1), GameInput::WriteSave(1)),
        (GameCommand::MsgNext, GameInput::MsgNext),
        (GameCommand::MsgPrev, GameInput::MsgPrev),
        (GameCommand::MsgClose, GameInput::MsgClose),
        (GameCommand::SaveloadClose, GameInput::SaveloadClose),
        (GameCommand::MapBack, GameInput::MapBack),
    ];
    for (cmd, expected) in pairs {
        assert_eq!(GameInput::from(cmd), expected, "From<GameCommand> 对齐失败：{cmd:?}");
    }
}

/// 无参命令的 `has_param` 返回 false。
#[test]
fn command_has_param_false_for_paramless() {
    assert!(!GameCommand::MoveUp.has_param());
    assert!(!GameCommand::Confirm.has_param());
    assert!(!GameCommand::Attack.has_param());
    assert!(!GameCommand::MsgClose.has_param());
    assert!(!GameCommand::MapBack.has_param());
}

/// 带参命令的 `has_param` 返回 true。
#[test]
fn command_has_param_true_for_parameterized() {
    assert!(!GameCommand::Spin.has_param());
    assert!(GameCommand::Landmark(1).has_param());
    assert!(GameCommand::TradeBuy(3).has_param());
    assert!(GameCommand::Build(Building::Fort).has_param());
    assert!(GameCommand::LoadSave(2).has_param());
    assert!(GameCommand::WriteSave(0).has_param());
}

/// Index 场景：`MoveDown` 不消费（场景不变，`dispatch` 返回 false），
/// `Confirm`（菜单项 0）消费（切到 `SelectRole`，`dispatch` 返回 true）。
#[test]
fn dispatch_index_move_and_confirm() {
    let mut app = GameApp::new(1080.0, 1920.0);
    assert_eq!(app.scene(), Scene::Index);

    // MoveDown：菜单下移，无场景切换 → false
    let consumed_down = app.dispatch(GameCommand::MoveDown);
    assert!(!consumed_down, "MoveDown 不应发生场景切换");
    assert_eq!(app.menu_selected(), 1);
    assert_eq!(app.scene(), Scene::Index);

    // MoveUp：回到 0
    app.dispatch(GameCommand::MoveUp);
    assert_eq!(app.menu_selected(), 0);

    // Confirm（菜单项 0 = 新游戏）→ 切 SelectRole → true
    let consumed = app.dispatch(GameCommand::Confirm);
    assert!(consumed, "Confirm 菜单项 0 应发生场景切换");
    assert_eq!(app.scene(), Scene::SelectRole);
}

/// 主地图：`Spin`（摇骰）沿环岛赛道前进玩家位置，不切场景 → false。
/// `Landmark(6)` 触发战斗 → 切 Vs → true。
#[test]
fn dispatch_main_map_spin_and_battle() {
    let mut app = GameApp::new(1080.0, 1920.0);
    // 快进到主地图
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap);

    let before = app.map_pos();
    let consumed_spin = app.dispatch(GameCommand::Spin);
    let dungeon_active = app.battle().is_some_and(|v| v.dungeon.is_some());
    if dungeon_active {
        // S28.6：落格 = 无主领地格 ⇒ 自动进副本守军战（`start_dungeon_battle` 直接切 Vs；
        // dispatch 消费标记 = 场景自身消费，副本切场景不经 dispatch 返回值）
        assert_eq!(app.scene(), Scene::Vs, "副本战在 vs 场景");
        assert_ne!(app.map_pos(), before, "摇骰应沿赛道前进");
    } else {
        assert!(!consumed_spin, "Spin 落点非 BOSS 格 / 无主领地格，不切场景");
        assert_ne!(app.map_pos(), before, "摇骰应沿赛道前进");
        // 据点码 6 = 战斗 → 切 Vs
        let consumed_landmark = app.dispatch(GameCommand::Landmark(6));
        assert!(consumed_landmark, "Landmark(6) 应切到 Vs");
        assert_eq!(app.scene(), Scene::Vs);
    }
}

/// 战斗场景：`Attack` 推进回合，不切场景 → false（战斗未结束时）。
#[test]
fn dispatch_vs_attack_does_not_consume() {
    let mut app = GameApp::new(1080.0, 1920.0);
    // 快进到战斗
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap);
    app.handle(GameInput::Landmark(6));
    assert_eq!(app.scene(), Scene::Vs);

    let rounds_before = app.battle().expect("战斗应已开启").rounds;
    // 第一步 Attack 不一定切场景（战斗进行中）
    let consumed = app.dispatch(GameCommand::Attack);
    let rounds_after = app.battle().expect("战斗应仍在").rounds;
    assert!(rounds_after > rounds_before, "Attack 应推进回合");
    // 若战斗未结束，dispatch 返回 false
    let battle_finished = app.battle().expect("战斗应仍在").outcome().is_some();
    if !battle_finished {
        assert!(!consumed, "战斗未结束时 Attack 不应切场景");
    }
}

/// 消息场景：`MsgNext` 翻页不切场景 → false（页数受限时保持）；`MsgClose` 关闭 → true。
#[test]
fn dispatch_message_next_and_close() {
    let mut app = GameApp::new(1080.0, 1920.0);
    // 快进到主地图，再打开消息（事件码 1 → MsgKind::Msgl）
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap);
    app.handle(GameInput::Landmark(1));
    assert_eq!(app.scene(), Scene::Message);

    let page_before = app.msg_page();
    let total = app.msg_total();
    let consumed_next = app.dispatch(GameCommand::MsgNext);
    assert!(!consumed_next, "MsgNext 不切场景");
    // 如果还有下一页，page 应递增；否则保持末页
    if page_before + 1 < total {
        assert_eq!(app.msg_page(), page_before + 1, "MsgNext 应翻页");
    } else {
        assert_eq!(app.msg_page(), page_before, "末页 MsgNext 应保持");
    }

    let consumed_close = app.dispatch(GameCommand::MsgClose);
    assert!(consumed_close, "MsgClose 应切回场景源");
    assert_eq!(app.scene(), Scene::MainMap, "主地图打开的消息关闭应回主地图");
}

/// 存读档：`WriteSave(0)` 写档不切场景 → false；`SaveloadClose` → true。
#[test]
fn dispatch_saveload_write_and_close() {
    let mut app = GameApp::new(1080.0, 1920.0);
    // 主菜单 → 存读档
    app.set_menu_selected(1);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Saveload);

    let consumed_write = app.dispatch(GameCommand::WriteSave(0));
    assert!(!consumed_write, "WriteSave 不切场景");

    let consumed_close = app.dispatch(GameCommand::SaveloadClose);
    assert!(consumed_close, "SaveloadClose 应切回 Index");
    assert_eq!(app.scene(), Scene::Index);
}

/// `Command::execute` trait 方法与 `dispatch` 等价性（同一命令走 trait 入口结果一致）。
#[test]
fn command_trait_execute_matches_dispatch() {
    // 两个独立 app，同一命令序列，经不同入口执行，结果应一致。
    let mut app_a = GameApp::new(1080.0, 1920.0);
    let mut app_b = GameApp::new(1080.0, 1920.0);

    app_a.set_menu_selected(0);
    app_b.set_menu_selected(0);

    let cmd_a = GameCommand::MoveDown;
    let cmd_b = GameCommand::MoveDown;
    let via_dispatch = app_a.dispatch(cmd_a);
    let via_trait = cmd_b.execute(&mut app_b);
    assert_eq!(via_dispatch, via_trait, "dispatch 与 trait execute 结果应一致");
    assert_eq!(app_a.menu_selected(), app_b.menu_selected());
}
