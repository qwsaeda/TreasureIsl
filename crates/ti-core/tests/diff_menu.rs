//! S3.3 主菜单事件状态机差分测试。
//!
//! 遍历所有 (`MenuScene` × `MenuEvent`) 组合，oracle 与 impl 逐位一致；
//! 另加 2 个语义测试。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::menu::{
    MenuEvent, MenuScene, Sfx, oracle_menu_transition, oracle_reachable_menu,
};
use ti_core::rules::menu::MenuStateMachine;

/// 将 oracle 场景与 impl 状态机的 transition 结果逐位对比。
fn assert_transition_equal(
    oracle: (MenuScene, Option<Sfx>),
    impl_scene: MenuScene,
    impl_sfx: Option<Sfx>,
) {
    assert_eq!(oracle.0, impl_scene, "场景不一致");
    assert_eq!(oracle.1, impl_sfx, "音效不一致");
}

/// 遍历所有 (`MenuScene` × `MenuEvent`) 组合，oracle vs impl 逐位一致。
#[test]
fn diff_all_combinations() {
    for scene in MenuScene::all() {
        for ev in MenuEvent::all() {
            let oracle = oracle_menu_transition(scene, ev);
            let mut sm = MenuStateMachine::new();
            sm.set_scene(scene);
            let impl_sfx = sm.handle(ev);
            assert_transition_equal(oracle, sm.scene(), impl_sfx);
        }
    }
}

/// 可达性：`oracle_reachable_menu` 与 impl `reachable` 逐位一致。
#[test]
fn diff_reachable() {
    let o_reachable = oracle_reachable_menu();
    let mut sm = MenuStateMachine::new();
    sm.set_scene(MenuScene::Index);
    let i_reachable = sm.reachable();
    assert_eq!(o_reachable, i_reachable, "可达场景集不一致");
}

/// 语义测试 (a)：从 Index 按新游戏事件（`Click(0)`）→ `SelectRole` + `Sfx::Press`。
#[test]
fn semantic_index_new_game() {
    let mut sm = MenuStateMachine::new();
    sm.set_scene(MenuScene::Index);

    let sfx = sm.handle(MenuEvent::Click(0));
    assert_eq!(sm.scene(), MenuScene::SelectRole, "Click(0) 应进入 SelectRole");
    assert_eq!(sfx, Some(Sfx::Press), "新游戏应触发 Press 音效");

    let oracle = oracle_menu_transition(MenuScene::Index, MenuEvent::Click(0));
    assert_eq!(oracle, (MenuScene::SelectRole, Some(Sfx::Press)));
}

/// 语义测试 (b)：Exit 可达性 — 从 Index 发 Quit → `scene()` 返回 Exit 且 `reachable()` 含 Exit。
#[test]
fn semantic_exit_reachable() {
    let mut sm = MenuStateMachine::new();
    sm.set_scene(MenuScene::Index);

    let _ = sm.handle(MenuEvent::Quit);
    assert_eq!(sm.scene(), MenuScene::Exit, "Quit 应进入 Exit");

    let mut sm2 = MenuStateMachine::new();
    sm2.set_scene(MenuScene::Index);
    let reachable = sm2.reachable();
    assert!(reachable.contains(&MenuScene::Exit), "reachable() 应包含 Exit，实际: {reachable:?}");

    let o_reachable = oracle_reachable_menu();
    assert!(o_reachable.contains(&MenuScene::Exit), "oracle 可达集应包含 Exit");
}
