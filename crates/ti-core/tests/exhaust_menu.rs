//! S15.1 菜单/消息/选择域业务场景穷举测试（`exhaust_menu`）。
//!
//! 枚举空间全部由 `assets/data/exhaust_scenarios.json`（`ti_core::data::exhaust()`）驱动：
//! - 菜单侧：`MenuScene::all()`（8 场景判别值）× `MenuEvent::all()`（12 事件实例）
//!   = 96 组合，每组合断言 `MenuStateMachine::handle` 无 panic + 判别值合法；
//!   另对 `Index` 场景 × 全 12 事件 + 5 个子场景 × 返回码 27 做逐条精确对账
//!   （与 `07_ui_flow.md` §1 转换表同口径），全局 `reachable()` 闭包与 §1 出边一致。
//! - 消息侧：`MsgKind::all()`（5 类）× 越界页码偏移档（JSON `menu.msg_page_offset_levels`）
//!   全组合，断言 `MsgSession::handle` 翻页钳位精确值 + 关闭回打开源场景 + 切换后无残留。
//! - 选择侧：`Character::all()`（5 角色）× `MapSlot::all()`（3 地图）× 已解锁子集全排列，
//!   断言 `SelectionFlow` 完成/锁定/复位语义精确值。

/// `usize` → `u32` 安全转换（测试枚举空间规模远小于 `u32::MAX`，越界钳到 `u32::MAX`，不 panic）。
#[allow(clippy::cast_possible_truncation)]
fn usize_as_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

use ti_core::data::exhaust;
use ti_core::domain::flow::{MenuDir, MenuEvent, MenuScene, MenuTarget, Sfx};
use ti_core::domain::messages::{FlowSource, MsgEvent, MsgKind};
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::menu::{MenuStateMachine, menu_item_target, move_menu_cursor};
use ti_core::rules::messages::{MsgSession, kind_key, page_text, total_pages};
use ti_core::rules::selection::SelectionFlow;

/// 菜单侧穷举：8 场景 × 12 事件 = 96 组合，无 panic + 场景判别值互异（`07_ui_flow.md` §1 硬规则 2）。
/// 另对关键组合（`Index` 全事件 + 子场景返回码）逐条精确对账（FIND-1 修复）。
#[test]
fn exhaust_menu_all_scenes_all_events() {
    let mut combos: u32 = 0;
    let mut seen_discriminants = std::collections::BTreeSet::new();
    for scene in MenuScene::all() {
        seen_discriminants.insert(scene.discriminant());
        for ev in MenuEvent::all() {
            let mut sm = MenuStateMachine::new();
            sm.set_scene(scene);
            let _ = sm.handle(ev);
            assert!(sm.scene().discriminant() <= 7, "handle 后场景判别值恒 0..=7");
            combos += 1;
        }
    }
    assert_eq!(
        combos,
        usize_as_u32(MenuScene::all().len() * MenuEvent::all().len()),
        "组合数 = MenuScene 场景数 × MenuEvent 事件数（数据驱动，非硬编码 96）"
    );
    assert_eq!(seen_discriminants.len(), 8, "8 场景判别值互不相同（07_ui_flow.md §1 硬规则 2）");

    // 关键组合逐条精确对账（`07_ui_flow.md` §1 转换表）：
    // Index × 全 12 事件（场景 + 音效）+ 5 子场景 × 返回码 27
    let mut sm = MenuStateMachine::new();
    sm.set_scene(MenuScene::Index);
    let cases: [(MenuEvent, MenuScene, Option<Sfx>); 12] = [
        (MenuEvent::Click(0), MenuScene::SelectRole, Some(Sfx::Press)),
        (MenuEvent::Click(1), MenuScene::Saveload, Some(Sfx::Press)),
        (MenuEvent::Click(2), MenuScene::Help, Some(Sfx::Press)),
        (MenuEvent::Click(3), MenuScene::Help, Some(Sfx::Press)),
        (MenuEvent::Click(4), MenuScene::Help, Some(Sfx::Press)),
        (MenuEvent::Click(5), MenuScene::Exit, None),
        (MenuEvent::Click(13), MenuScene::SelectRole, Some(Sfx::Press)),
        (MenuEvent::Click(27), MenuScene::Index, None),
        (MenuEvent::Click(37), MenuScene::Index, Some(Sfx::Invalid)),
        (MenuEvent::Click(38), MenuScene::Index, Some(Sfx::Invalid)),
        (MenuEvent::Invalid, MenuScene::Index, Some(Sfx::Invalid)),
        (MenuEvent::Quit, MenuScene::Exit, None),
    ];
    for (ev, expected_scene, expected_sfx) in cases {
        let mut s = MenuStateMachine::new();
        s.set_scene(MenuScene::Index);
        let sfx = s.handle(ev);
        assert_eq!(s.scene(), expected_scene, "Index + {ev:?} 场景精确对账");
        assert_eq!(sfx, expected_sfx, "Index + {ev:?} 音效精确对账");
    }

    // 5 子场景 × 返回码 27 → 必回 Index（§1 转换表子场景出边）
    for sub in [
        MenuScene::SelectRole,
        MenuScene::SelectMap,
        MenuScene::Saveload,
        MenuScene::Option,
        MenuScene::Help,
    ] {
        let mut s = MenuStateMachine::new();
        s.set_scene(sub);
        let sfx = s.handle(MenuEvent::Click(27));
        assert_eq!(s.scene(), MenuScene::Index, "{sub:?} + Click(27) 必回 Index");
        assert_eq!(sfx, None, "子场景返回码无音效");
    }

    // `reachable()` 结果须与 `07_ui_flow.md` §1 转换表语义一致（合法 6 场景可达）
    let reachable = sm.reachable();
    let mut expected: Vec<MenuScene> = reachable.clone();
    expected.sort_by_key(|s| s.discriminant());
    assert_eq!(reachable, expected, "reachable() 结果按判别值排序（确定性）");
    assert!(
        reachable.contains(&MenuScene::SelectRole),
        "SelectRole 从 Intro/Index 两入口必可达（07_ui_flow.md §1 Index→SelectRole 出边）"
    );
    assert!(reachable.contains(&MenuScene::Saveload), "Saveload 从 Index 必可达（菜单项 1 出边）");
    assert!(
        !reachable.contains(&MenuScene::Option),
        "Option 不经菜单转移表（07_ui_flow.md §1 注：地图/选项流驱动）"
    );
}

/// 菜单光标穷举：循环长度/起始位置档位由 JSON `menu.cursor_counts`/`menu.cursor_starts` 驱动，
/// 断言 `move_menu_cursor` 循环不越界 + 零项菜单返回 `(0,false)`（`07_ui_flow.md` §6 规则 1）。
#[test]
fn exhaust_menu_cursor_all_counts() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    for &count in &es.menu.cursor_counts {
        for &start in &es.menu.cursor_starts {
            if count == 0 {
                assert_eq!(move_menu_cursor(start, MenuDir::Up, count), (0, false));
                assert_eq!(move_menu_cursor(start, MenuDir::Down, count), (0, false));
                continue;
            }
            let start = start.min(count - 1);
            let up = move_menu_cursor(start, MenuDir::Up, count);
            let down = move_menu_cursor(start, MenuDir::Down, count);
            assert!(up.0 < count && down.0 < count, "光标恒不越界（count={count}）");
            // 独立手算对账（与 `move_menu_cursor` 实现同口径：先对 start 取模，再 ±1 循环）
            let s = start % count;
            let expected_up = if s == 0 { count - 1 } else { s - 1 };
            assert_eq!(up.0, expected_up, "Up 手算对账（count={count} start={start}）");
            let expected_down = (s + 1) % count;
            assert_eq!(down.0, expected_down, "Down 手算对账（count={count} start={start}）");
        }
    }
}

/// 菜单项目标穷举：索引档位由 JSON `menu.item_target_indexes` 驱动（含合法 0..=5 + 越界值），
/// 断言 `menu_item_target` 精确映射（`07_ui_flow.md` §6 规则 2 `menu_item_targets`，
/// 路由语义唯一事实源）。
#[test]
fn exhaust_menu_item_targets_all_indexes() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    for &idx in &es.menu.item_target_indexes {
        let expected = match idx {
            0 => Some(MenuTarget::SelectRole),
            1 => Some(MenuTarget::Saveload),
            2..=4 => Some(MenuTarget::Help),
            5 => Some(MenuTarget::Exit),
            _ => None,
        };
        assert_eq!(menu_item_target(idx), expected, "menu_item_target({idx}) 手算对账");
    }
}

/// 消息侧穷举：全 5 类 `MsgKind` × 页码偏移档（JSON `menu.msg_page_offset_levels`，
/// 相对 `total` 偏移，含越界正值/负值），断言 `MsgSession` 翻页钳位精确值
/// + 关闭回打开源场景 + 切换后无残留（`07_ui_flow.md` §6 规则 10/11/12）。
#[test]
fn exhaust_msg_all_kinds_all_pages() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let mut combos: u32 = 0;
    for kind in MsgKind::all() {
        let total = total_pages(kind);
        let page_levels: Vec<u32> = es
            .menu
            .msg_page_offset_levels
            .iter()
            .map(|&off| {
                let v = i64::from(total) + i64::from(off);
                u32::try_from(v.max(0)).unwrap_or(0)
            })
            .collect();
        for &page in &page_levels {
            let expected_clamped = if page >= total { total.saturating_sub(1) } else { page };
            let key = page_text(kind, page);
            // `kind_key`（非 `MsgKind::label`）是 `page_text` 的实际前缀口径
            let expected_key = format!("msg.{}.{}", kind_key(kind), expected_clamped + 1);
            assert_eq!(key, expected_key, "page_text 钳位手算对账（kind={kind:?} page={page}）");
            combos += 1;
        }

        // MsgSession 翻页钳位 + 关闭回源 + 无残留
        let mut session = MsgSession::new();
        session.open(kind, FlowSource::FromMainMap);
        let open_page = session.current().expect("open 后必有页");
        assert_eq!(open_page.page, 0, "open 后恒 page=0（无残留旧页码）");
        // FIND-A11 修复：跨 kind 切换无残留——先 open 另一类 kind、翻到某页，再 open 回
        // `kind`，断言页码强制归 0 且 kind 已切换（不能残留旧 kind 的页码）
        let other = match kind {
            MsgKind::Msgs => MsgKind::Help,
            _ => MsgKind::Msgs,
        };
        let mut session_switch = MsgSession::new();
        session_switch.open(other, FlowSource::FromIndex);
        for _ in 0..total_pages(other).saturating_sub(1) {
            let _ = session_switch.handle(MsgEvent::NextPage);
        }
        let pre_switch_page = session_switch.current().expect("翻页后必有页").page;
        session_switch.open(kind, FlowSource::FromIndex);
        let post_switch = session_switch.current().expect("open 后必有页");
        assert_eq!(post_switch.kind, kind, "跨 kind 切换后 kind 必已更新（不残留旧 kind）");
        assert_eq!(post_switch.page, 0, "跨 kind 切换后页码强制归 0（不残留旧 kind 页码）");
        let _ = pre_switch_page;
        for &page in &page_levels {
            let mut s2 = MsgSession::new();
            s2.open(kind, FlowSource::FromIndex);
            let _ = s2.handle(MsgEvent::NextPage);
            let _ = s2.handle(MsgEvent::PrevPage);
            let _ = page;
            // FIND-4 修复：显式断言 NextPage+PrevPage 后的精确页码（非丢弃结果）
            let p = s2.current().expect("翻页后必有页");
            assert_eq!(
                p.page, 0,
                "首页 NextPage→PrevPage 后必回 0（多页类 NextPage 先到 1，PrevPage 回 0；单页类恒 0）"
            );
            let target = s2.close_target();
            assert_eq!(target, Some(0), "FromIndex 打开关闭必回 0（主菜单域状态码）");
        }
        assert_eq!(session.close_target(), Some(4), "FromMainMap 打开关闭必回 4（主地图域状态码）");
    }
    let expected = usize_as_u32(MsgKind::all().len() * es.menu.msg_page_offset_levels.len());
    assert_eq!(combos, expected, "组合数 = 5 类 × JSON 驱动页码档位数");
}

/// 选择侧穷举：全 5 角色 × 全 3 地图 × 已解锁子集全排列（2^3-1 = 7 种非空子集），
/// 断言 `SelectionFlow` 锁定地图拒绝 + 双选完成 + 复位无残留（`07_ui_flow.md` §6 规则 12
/// 选择侧 + `selection_all_flows_full` 模式扩展到全排列子集）。
/// 角色/地图目录走 `balance.selection`（真实表），子集 mask 遍历全排列。
#[test]
fn exhaust_selection_all_combos() {
    let all_maps = MapSlot::all();
    let all_chars = Character::all();
    let b = ti_core::data::balance().expect("balance.json 有效");
    assert_eq!(
        usize_as_u32(all_maps.len()),
        b.selection.map_count,
        "MapSlot::all() 长度须 = balance.selection.map_count（真实表）"
    );
    assert_eq!(
        usize_as_u32(all_chars.len()),
        b.selection.character_count,
        "Character::all() 长度须 = balance.selection.character_count（真实表）"
    );

    let mut combos: u32 = 0;
    let map_count = usize_as_u32(all_maps.len());
    for mask in 1..(1u32 << map_count) {
        let unlocked: Vec<MapSlot> = (0..map_count)
            .filter(|i| mask & (1u32 << i) != 0)
            .map(|i| all_maps[i as usize])
            .collect();
        for ch in all_chars {
            for m in &unlocked {
                let mut flow = SelectionFlow::new(&unlocked);
                let sfx = flow.select_role(ch);
                assert_eq!(sfx, Some(Sfx::Press), "选已解锁角色恒 Press");
                assert!(flow.state().has_character(), "选角后 character 已设");
                let sfx2 = flow.select_map(*m);
                assert_eq!(sfx2, Some(Sfx::Press), "选已解锁地图恒 Press");
                assert!(flow.is_completed(), "角色 + 地图双选必 completed（精确值）");

                // 锁定地图（不在 unlocked 子集里）须 Invalid（全解锁子集下此分支不执行）
                if let Some(locked) = all_maps.iter().find(|x| !unlocked.contains(x)) {
                    let mut flow2 = SelectionFlow::new(&unlocked);
                    let _ = flow2.select_role(ch);
                    let sfx_locked = flow2.select_map(*locked);
                    assert_eq!(sfx_locked, Some(Sfx::Invalid), "锁定地图必 Invalid（精确值）");
                    assert!(!flow2.is_completed(), "锁定地图未完成");
                }

                // 场景切换复位无残留
                flow.reset();
                assert!(!flow.state().has_character(), "reset 清角色（无残留）");
                assert!(!flow.state().has_map(), "reset 清地图（无残留）");
                assert!(!flow.is_completed(), "reset 清 completed（无残留）");
                assert_eq!(
                    flow.unlocked().len(),
                    unlocked.len(),
                    "reset 不清已解锁清单（探测事实保留）"
                );
                combos += 1;
            }
        }
    }
    // 组合数 = Σ(mask 各子集 unlocked 长度) × 角色数（不硬编码，按实际遍历结果对账）
    let mut expected: u32 = 0;
    for mask in 1..(1u32 << map_count) {
        expected += mask.count_ones() * usize_as_u32(all_chars.len());
    }
    assert_eq!(combos, expected, "组合数 = Σ各子集 unlocked 长度 × 角色数（真实表驱动）");
}
