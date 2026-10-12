//! S8.3.e / S8.8 / S12.4：`rules::messages` + `rules::menu` + `rules::selection` 业务规则集成测试。
//!
//! 判据（`docs/business/07_ui_flow.md` §6 可测规则表 P4 条目）：
//! - 消息翻页首尾钳位（规则 10 `msg_page_clamped`，`next` 末页保持 / `prev` 首页回末页）；
//! - 消息关闭回打开源场景（规则 11 `msg_close_returns_source`，`FlowSource` 归属）；
//! - 切场景后选中项/页码复位（规则 12 `scene_switch_resets_state`，无残留）；
//! - 菜单项 0..5 上下循环不越界（规则 1 `menu_cursor_wraps`）+ 菜单项目标（规则 2
//!   `menu_item_targets`，路由语义唯一事实源）；
//! - 选角/选图状态机（锁定地图拒绝 / 双选完成）+ 场景切换复位（`SelectionState::reset`）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。
//! 类型路径走 `ti_core::domain::{flow,messages,selection}`（c1 契约 §1 唯一类型家）。

use ti_core::domain::flow::{MenuDir, MenuEvent, MenuScene, MenuTarget, Sfx};
use ti_core::domain::messages::{FlowSource, MsgEvent, MsgKind};
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::menu::{MenuStateMachine, menu_item_target, move_menu_cursor};
use ti_core::rules::messages::{MsgSession, page_text, total_pages};
use ti_core::rules::selection::SelectionFlow;

// ── 消息分页（规则 10 / 11）─────────────────────────────────────────────

#[test]
fn total_pages_help_from_balance() {
    // `balance.messages.pages.help = 3`（与 `domain::messages` 分页基准一致）
    assert_eq!(total_pages(MsgKind::Help), 3);
}

#[test]
fn total_pages_msgs_single_page() {
    // `balance.messages.pages.msgs = 1`
    assert_eq!(total_pages(MsgKind::Msgs), 1);
}

#[test]
fn total_pages_msgl_single_page() {
    // `balance.messages.pages.msgl = 1`（数据注入优先于缺省回退）
    assert_eq!(total_pages(MsgKind::Msgl), 1);
}

#[test]
fn page_text_help_page_zero() {
    let key = page_text(MsgKind::Help, 0);
    assert_eq!(key, "msg.help.1", "第 0 页 → key `msg.help.1`");
}

#[test]
fn page_text_help_last_page() {
    let key = page_text(MsgKind::Help, 2);
    assert_eq!(key, "msg.help.3", "第 2 页（末页）→ key `msg.help.3`");
}

#[test]
fn page_text_out_of_bounds_clamped_to_last() {
    let key = page_text(MsgKind::Help, 99);
    assert_eq!(key, "msg.help.3", "越界页钳到最后一页 key（不 panic）");
}

#[test]
fn page_text_single_page_kind() {
    // `msgs` 只 1 页，任何 page → `msg.msgs.1`
    let key = page_text(MsgKind::Msgs, 5);
    assert_eq!(key, "msg.msgs.1");
}

/// 规则 10（`msg_page_clamped`）：`next` 到末页保持、`prev` 首页回末页，首尾不越界。
#[test]
fn msg_page_clamped() {
    let mut s = MsgSession::new();
    s.open(MsgKind::Help, FlowSource::FromIndex);
    let total = total_pages(MsgKind::Help); // 3

    // next 逐页推进到末页（开在 page=0；第 k 次 next 后 page = min(k+1, total-1)，末页保持）
    for k in 0..total {
        let _ = s.handle(MsgEvent::NextPage);
        let p = s.current().expect("打开会话必有页");
        assert_eq!(
            p.page,
            k.saturating_add(1).min(total - 1),
            "第 {k} 次 next 后页码应=min(k+1, total-1)"
        );
    }
    // 末页保持（不环绕）
    let changed = s.handle(MsgEvent::NextPage);
    assert!(!changed, "末页 next 不产生变化");
    let p = s.current().expect("打开会话必有页");
    assert_eq!(p.page, total - 1, "末页 next 保持末页（首尾钳位）");

    // prev 首页回末页（循环）
    s.open(MsgKind::Help, FlowSource::FromIndex); // 复位到 0 页
    let _ = s.handle(MsgEvent::PrevPage);
    let p = s.current().expect("打开会话必有页");
    assert_eq!(p.page, total - 1, "首页 prev 回末页（循环钳位）");
}

/// 单页界面（`msgs`/`msgl`/`msgm`/`info` = 1 页）：`next` 恒不变、`prev` 恒不变。
#[test]
fn msg_page_single_page_kinds_invariant() {
    for kind in [MsgKind::Msgs, MsgKind::Msgl, MsgKind::Msgm, MsgKind::Info] {
        let mut s = MsgSession::new();
        s.open(kind, FlowSource::FromIndex);
        assert_eq!(total_pages(kind), 1, "{kind:?} 单页（balance.messages.pages）");
        assert!(!s.handle(MsgEvent::NextPage), "{kind:?} 单页 next 不变");
        assert!(!s.handle(MsgEvent::PrevPage), "{kind:?} 单页 prev 不变");
        let p = s.current().expect("打开会话必有页");
        assert_eq!(p.page, 0, "{kind:?} 恒 0 页");
    }
}

/// 未打开会话的事件不产生变化（`None` 页态上 `Next`/`Prev`/`Close` 幂等，不 panic）。
#[test]
fn msg_events_on_closed_session_noop() {
    let mut s = MsgSession::new();
    assert!(!s.handle(MsgEvent::NextPage), "未打开 next 无变化");
    assert!(!s.handle(MsgEvent::PrevPage), "未打开 prev 无变化");
    assert!(s.handle(MsgEvent::Close), "close 恒产生变化（关闭态确认）");
    assert!(!s.is_open());
    assert_eq!(s.close_target(), None, "未打开无界可回");
}

/// 规则 11（`msg_close_returns_source`）：关闭目标由打开源场景决定，不得回固定场景。
#[test]
fn msg_close_returns_source() {
    let mut s = MsgSession::new();

    // 从主地图打开 → 关闭回主地图（状态码 4 = 主地图域）
    s.open(MsgKind::Msgl, FlowSource::FromMainMap);
    assert!(s.is_open());
    assert_eq!(s.close_target(), Some(4), "主地图打开 → 关闭回主地图");

    // 从主菜单打开 → 关闭回主菜单（状态码 0 = 主菜单）
    s.open(MsgKind::Help, FlowSource::FromIndex);
    assert_eq!(s.close_target(), Some(0), "主菜单打开 → 关闭回主菜单");

    // 关闭后无页、无界可回
    let _ = s.handle(MsgEvent::Close);
    assert!(!s.is_open());
    assert_eq!(s.close_target(), None, "关闭后无打开源可回");
}

// ── 场景切换复位（规则 12 `scene_switch_resets_state`）─────────────────

/// 规则 12（`scene_switch_resets_state`）：切场景后选中项/页码复位，禁止残留。
#[test]
fn scene_switch_resets_state() {
    // 消息会话：深翻页 + 切换种类后强制复位，不残留页码/种类
    let mut s = MsgSession::new();
    s.open(MsgKind::Help, FlowSource::FromIndex);
    let _ = s.handle(MsgEvent::NextPage);
    let _ = s.handle(MsgEvent::NextPage);
    let residue_page = s.current().expect("打开会话必有页").page;
    assert_eq!(residue_page, 2, "两次 next 后页码 = 2");

    s.reset();
    assert!(!s.is_open(), "reset 后消息会话关闭");
    assert_eq!(s.source(), FlowSource::FromIndex, "reset 回默认打开源");
    assert_eq!(s.close_target(), None, "reset 后无界可回");

    // 重新打开必须从 0 页起（禁止残留旧页码/旧种类）
    s.open(MsgKind::Info, FlowSource::FromMainMap);
    let p = s.current().expect("打开会话必有页");
    assert_eq!(p.page, 0, "重开强制 page=0（无残留）");
    assert_eq!(p.kind, MsgKind::Info, "重开种类切换生效（无残留旧 kind）");

    // 选择状态：双选完成 → 切场景复位回空
    let mut flow = SelectionFlow::default();
    let _ = flow.select_role(Character::Panda);
    let _ = flow.select_map(MapSlot::M01);
    assert!(flow.is_completed(), "角色 + 地图双选 → completed");
    flow.reset();
    assert!(!flow.state().has_character(), "reset 清角色（无残留）");
    assert!(!flow.state().has_map(), "reset 清地图（无残留）");
    assert!(!flow.is_completed(), "清空后 completed 复位");
    assert_eq!(flow.unlocked().len(), 3, "reset 不清已解锁地图清单（探测事实保留）");
}

// ── 菜单光标 + 项目标（规则 1 / 2）─────────────────────────────────────

/// 规则 1（`menu_cursor_wraps`）：菜单项 0..5 上下循环不越界。
#[test]
fn menu_cursor_wraps() {
    let count = 6;
    let (n, _) = move_menu_cursor(0, MenuDir::Up, count);
    assert_eq!(n, 5, "首项上移回末项（循环）");
    let (n, _) = move_menu_cursor(5, MenuDir::Down, count);
    assert_eq!(n, 0, "末项下移回首项（循环）");
    let (n, changed) = move_menu_cursor(3, MenuDir::Down, count);
    assert_eq!(n, 4, "中间项下移 +1");
    assert!(changed);
}

/// 菜单项六格上下循环不越界（归一化 `MoveUp` 与 `MoveDown` 事件码）。
#[test]
fn menu_cursor_no_out_of_bounds() {
    for start in 0..6 {
        for _ in 0..4 {
            let (n, _) = move_menu_cursor(start, MenuDir::Up, 6);
            assert!((0..6).contains(&n), "start={start} 上移 4 次仍 0..5");
            let (n, _) = move_menu_cursor(start, MenuDir::Down, 6);
            assert!((0..6).contains(&n), "start={start} 下移 4 次仍 0..5");
        }
    }
    // 零项菜单不 panic（返回 (0, false)）
    assert_eq!(move_menu_cursor(0, MenuDir::Up, 0), (0, false));
    assert_eq!(move_menu_cursor(0, MenuDir::Down, 0), (0, false));
}

/// 规则 2（`menu_item_targets`）：菜单项 0/1/5 分别跳 SelectRole/Saveload/退出。
#[test]
fn menu_item_targets() {
    assert_eq!(menu_item_target(0), Some(MenuTarget::SelectRole));
    assert_eq!(menu_item_target(1), Some(MenuTarget::Saveload));
    assert_eq!(menu_item_target(2), Some(MenuTarget::Help));
    assert_eq!(menu_item_target(3), Some(MenuTarget::Help));
    assert_eq!(menu_item_target(4), Some(MenuTarget::Help));
    assert_eq!(menu_item_target(5), Some(MenuTarget::Exit));
    assert_eq!(menu_item_target(6), None, "越界项无目标（不 panic）");
}

/// 场景判别值互不相同（`07_ui_flow.md` §1 硬规则 2 / §6 规则 4 的域层语义）。
#[test]
fn menu_scene_discriminants_unique() {
    let all = MenuScene::all();
    let mut seen = std::collections::BTreeSet::new();
    for s in all {
        assert!(seen.insert(s.discriminant()), "场景 {s:?} 判别值重复");
    }
    assert_eq!(seen.len(), all.len(), "8 场景判别值互不相同");
}

/// 状态机转移表：菜单项 0..5 的 (事件, 音效) 行为（规则 2 状态机侧断言）。
#[test]
fn menu_state_machine_items() {
    let mut sm = MenuStateMachine::new();
    assert_eq!(sm.scene(), MenuScene::Index, "初始场景 = Index");

    let sfx = sm.handle(MenuEvent::Click(0));
    assert_eq!(sm.scene(), MenuScene::SelectRole);
    assert_eq!(sfx, Some(Sfx::Press), "项 0 有效确认 → Press");

    sm.set_scene(MenuScene::Index);
    let sfx = sm.handle(MenuEvent::Click(5));
    assert_eq!(sm.scene(), MenuScene::Exit);
    assert_eq!(sfx, None, "项 5 退出无音效");

    sm.set_scene(MenuScene::SelectRole);
    let sfx = sm.handle(MenuEvent::Click(27));
    assert_eq!(sm.scene(), MenuScene::Index, "子场景返回 → Index");
    assert_eq!(sfx, None);
}

/// 状态机可达集（`07_ui_flow.md` §1 转换表闭包）：从 `Intro`/`Index` 两入口
/// 出发可达 6 场景（`SelectMap`/`Option` 由地图/选项流驱动，不经菜单转移表）。
#[test]
fn menu_reachable_all_scenes() {
    let sm = MenuStateMachine::new();
    let reachable = sm.reachable();
    assert_eq!(
        reachable,
        vec![
            MenuScene::Intro,
            MenuScene::Index,
            MenuScene::SelectRole,
            MenuScene::Saveload,
            MenuScene::Help,
            MenuScene::Exit,
        ],
        "两入口可达闭包 = 6 场景（SelectMap/Option 不经菜单转移表）"
    );
}

/// 选角/选图：锁定地图拒绝（Invalid）+ 双选完成。
#[test]
fn selection_locked_map_rejected() {
    let mut flow = SelectionFlow::new(&[MapSlot::M01, MapSlot::M02]);
    let _ = flow.select_role(Character::Kidd);
    let sfx = flow.select_map(MapSlot::M03);
    assert!(!flow.is_completed(), "锁定地图不完成");
    assert_eq!(sfx, Some(Sfx::Invalid), "锁定地图 → Invalid");
}

/// 选角/选图：先选地图再选角色同样双选完成（顺序无关）。
#[test]
fn selection_map_then_role_completes() {
    let mut flow = SelectionFlow::default();
    let _ = flow.select_map(MapSlot::M02);
    assert!(!flow.is_completed(), "只选地图未完成");
    let sfx = flow.select_role(Character::Tichy);
    assert!(flow.is_completed(), "角色 + 地图双选 → completed");
    assert_eq!(sfx, Some(Sfx::Press));
}

/// 选角/选图：双选完成后切场景复位（规则 12 选择侧）。
#[test]
fn selection_reset_after_scene_switch() {
    let mut flow = SelectionFlow::default();
    let _ = flow.select_role(Character::Panda);
    let _ = flow.select_map(MapSlot::M01);
    assert!(flow.is_completed());
    flow.reset();
    assert!(!flow.state().has_character(), "reset 清角色");
    assert!(!flow.state().has_map(), "reset 清地图");
    assert!(!flow.is_completed(), "reset 清 completed");
}

/// 完整选择流程（`balance.selection` 目录）：全解锁 15 组合 / 部分解锁 10 组合，
/// 全 `completed` 对账通过（库 0 panic：`all_flows` 过滤式长度对账）。
#[test]
fn selection_all_flows_full() {
    let flow = SelectionFlow::default();
    let flows = flow.all_flows();
    assert_eq!(flows.len(), 15, "5 角色 × 3 地图 = 15 组合");
}

#[test]
fn selection_all_flows_partial_unlocked() {
    let flow = SelectionFlow::new(&[MapSlot::M01, MapSlot::M02]);
    let flows = flow.all_flows();
    assert_eq!(flows.len(), 10, "5 角色 × 2 已解锁地图 = 10 组合");
}
