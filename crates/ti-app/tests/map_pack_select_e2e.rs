//! S31.2b 地图包选择面 e2e（`docs/process/21_map_package_spec.md` §7 选 B 契约·UI 面）：
//! `select_map` 包列表标记（已装包 ∩ 内嵌槽位 = 金边条）+ 注入源可观测 + 0 回归。
//!
//! 判据（台账 S31.2b dod）：未注入 = 0 行为变化（`map_pack_ids` = `None`）；
//! 注入清单可经 `SelectionHost::map_pack_ids` / `active_map_pack` 读取（壳层采集面）。

use ti_app::{GameApp, GameInput, MapHost, Scene, SelectionHost};

fn enter_select_map(app: &mut GameApp) {
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectMap, "应到 `select_map`");
}

/// 0 回归判据：未注入包时，包清单面 = `None`（`select_map` 走内嵌 3 槽旧流）。
#[test]
fn map_pack_ids_none_without_injection() {
    let mut app = GameApp::new(1080.0, 1920.0);
    enter_select_map(&mut app);
    assert_eq!(app.map_pack_ids(), None, "未注入包 = 清单面 None（0 行为变化）");
    assert_eq!(app.active_map_pack(), None, "未注入包 = 注入源 None");
}

/// 切图重进主地图 = 会话重建路径（board session 按当前槽重初始化；S31.2 注入后
/// `build_board_session` 读的是注入合并表 ⇒ 会话跟包走）。
#[test]
fn re_enter_main_map_rebuilds_board_session() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(7);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectMap);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "Confirm 进主地图");
    let first = app.board_index();
    // 切图 = `MapBack` 回主菜单 → 再进选图 → 重进主地图（`enter_main_map_at(None)`
    // 全量重建 board/territory 会话；主地图 `Back` = 关建造面板，非导航）。
    app.handle(GameInput::MapBack);
    assert_eq!(app.scene(), Scene::Index, "MapBack 回主菜单");
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectMap, "双 Confirm 到选图");
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "再 Confirm 重进主地图");
    assert_eq!(app.board_index(), first, "同槽重进 = 会话重建后起点一致");
    assert!(app.board_spaces().len() >= 8, "赛道在位（重建成功，非半初始化）");
}

/// 输入面：非导航输入经 `select_map` 的 `on_input` 不 panic、不改场景（0 行为变化口径）。
#[test]
fn select_map_input_surface_no_panic() {
    let mut app = GameApp::new(1080.0, 1920.0);
    enter_select_map(&mut app);
    for input in [
        GameInput::MoveUp,
        GameInput::MoveDown,
        GameInput::Prev,
        GameInput::Next,
        GameInput::Spin,
        GameInput::Landmark(0),
        GameInput::Landmark(9),
        GameInput::TradeBuy(0),
        GameInput::TradeSell(0),
        GameInput::MsgNext,
        GameInput::MsgPrev,
    ] {
        app.handle(input);
    }
    assert_eq!(app.scene(), Scene::SelectMap, "非导航输入不改场景（`select_map` 0 行为变化）");
}
