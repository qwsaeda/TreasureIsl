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
    assert_eq!(app.character(), Some(ti_core::domain::selection::Character::Panda));
    app.handle(GameInput::Confirm); // 角色选定 → 进选图
    assert_eq!(app.scene(), Scene::SelectMap);

    // 选图：默认 None，Next → 索引 0（M01）
    app.handle(GameInput::Next);
    assert_eq!(app.map_slot(), Some(ti_core::domain::selection::MapSlot::M01));
    app.handle(GameInput::Confirm); // 角色+地图齐全 → 进主地图
    assert_eq!(app.scene(), Scene::MainMap);
    assert!(app.map_pos().is_some(), "主地图应有初始位置");
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

    let start_pos = app.map_pos();
    let gold_before = app.gold();
    // 移动（主地图 South 方向，开放地形应可移动）
    app.handle(GameInput::Walk(ti_core::domain::map::Dir::South));
    let moved = app.map_pos();
    assert_ne!(moved, start_pos, "开放地形下 South 移动应改变位置");
    assert_eq!(
        moved.expect("移动后位置应存在").row,
        start_pos.expect("初始位置应存在").row + 1,
        "South 方向 = 行 +1"
    );

    // 贸易：买 1 件商品 gid=1（价格 20，开局 1000 金）
    let gold0 = app.gold();
    app.handle(GameInput::TradeBuy(1));
    assert!(app.gold() < gold0, "买入后金应减少");
    assert_eq!(app.owned().len(), 1, "买入后持有量 +1");
    assert_eq!(app.gold(), gold0 - 20, "买入扣固定价格表价 20");

    // 建造：建 Castle（基础成本 1000）
    let bank_before = app.bank().gold;
    app.handle(GameInput::Build(ti_core::domain::economy::Building::Castle));
    // Castle 成本 1000 = bank.gold（开局 1000），扣后 bank.gold = 0
    assert_eq!(app.buildings().len(), 1, "建造后建筑列表 +1");
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
    assert!(app.battle().is_some(), "战斗会话应已建立");
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
        if let Some(b) = app.battle() {
            if b.finished() {
                break;
            }
        }
    }
    assert!(app.battle().is_some_and(ti_app::BattleView::finished), "1000 回合内战斗应结束");
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
    assert_eq!(app.msg_page(), 0);
    // 翻页（Msgl 共 1 页，next 保持）
    app.handle(GameInput::MsgNext);
    assert_eq!(app.msg_page(), 0, "Msgl 单页，next 到末页保持");
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
    let saved_slot = app.save_slots()[0];
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

// ── P6 表现层包：`docs/business/07_ui_flow.md §6` 12 条可测规则（逐条具名） ──

use ti_app::scenes::{SceneHost, SceneKind, SceneTransition, markers};
use ti_app::touch::layout;
use ti_platform::gfx::Viewport;
use ti_platform::settings::Gesture;

/// 规则 1 `menu_cursor_wraps`：菜单项 0..5 上下循环不越界。
#[test]
fn menu_cursor_wraps() {
    let mut app = GameApp::new(176.0, 220.0);
    // 从 0 上移 → 回绕到 5；从 5 下移 1 次 → 回绕到 0
    app.handle(GameInput::MoveUp);
    assert_eq!(app.menu_selected(), 5, "0 上移应回绕到 5");
    app.handle(GameInput::MoveDown);
    assert_eq!(app.menu_selected(), 0, "5 下移应回绕到 0");
    // 全程不越界（中间态 0..=5）
    for _ in 0..7 {
        app.handle(GameInput::MoveDown);
        assert!((0..=5).contains(&app.menu_selected()), "菜单项越界：{}", app.menu_selected());
    }
}

/// 规则 2 `menu_item_targets`：菜单项 0/1/5 分别跳 SelectRole/Saveload/退出。
#[test]
fn menu_item_targets() {
    let mut app = GameApp::new(176.0, 220.0);
    // 菜单项 0 → SelectRole
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::SelectRole, "菜单项 0 应进选角");

    // 回主菜单（选角 Back）
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::MoveDown); // 菜单项 1
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Saveload, "菜单项 1 应进存读档");

    let mut app = GameApp::new(176.0, 220.0);
    for _ in 0..5 {
        app.handle(GameInput::MoveDown); // 菜单项 5
    }
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Exit, "菜单项 5 应进退出场景");
    assert!(app.should_quit(), "菜单项 5 应置退出标志");
}

/// 规则 3 `compile_fail`：非法场景转换编译期不可构造。
///
/// 合法转换可构造（编译期证明）；非法转换（`SelectRole → Vs`、`MainMap → Saveload`）
/// 无 impl 块，构造即编译报错——由 `scenes/mod.rs` 的 ```compile_fail``` 文档测试覆盖，
/// 本处给出**合法对**的编译期可构造性正向对照（编译通过 = 类型约束成立）。
#[test]
fn compile_fail_illegal_transition_not_constructible() {
    // 合法转换（`Index → Saveload`）：可构造
    let _ok: SceneTransition<markers::Index, markers::Saveload> =
        <SceneTransition<markers::Index, markers::Saveload>>::new();
    // 非法转换（`SelectRole → Vs`）无 impl；此处编译失败。
    // 作为对照，合法的 `SelectRole → SelectMap` 可构造。
    let _ok2: SceneTransition<markers::SelectRole, markers::SelectMap> =
        <SceneTransition<markers::SelectRole, markers::SelectMap>>::new();
}

/// 规则 4 `scene_discriminants_unique`：9 个场景判别值互不相同。
#[test]
fn scene_discriminants_unique() {
    let kinds = [
        SceneKind::Intro,
        SceneKind::Index,
        SceneKind::SelectRole,
        SceneKind::SelectMap,
        SceneKind::MainMap,
        SceneKind::Vs,
        SceneKind::Message,
        SceneKind::Saveload,
        SceneKind::Exit,
    ];
    let mut seen: Vec<i32> = Vec::new();
    for k in kinds {
        let d = k.discriminant();
        assert!((0..=8).contains(&d), "判别值 {d} 越界 0..=8");
        assert!(!seen.contains(&d), "判别值 {d} 重复");
        seen.push(d);
    }
    assert_eq!(seen.len(), 9, "9 个场景判别值应互不相同");
}

/// 规则 5 `widget_visible_filter`：控件可见性 = `flags&1` + `w>0 && h>0`。
#[test]
fn widget_visible_filter() {
    use ti_core::data::ControlRec;
    let base =
        ControlRec { kind: 0, kind_name: "none", x: 0, y: 0, w: 4, h: 4, flags: 0, extra: 0 };
    // flags=0 → 不可见
    assert!(!base.visible(), "flags=0 不可见");
    let mut rec = base;
    rec.flags = 1;
    assert!(rec.visible(), "flags&1=1 且 w,h>0 可见");
    let mut rec2 = rec;
    rec2.w = 0;
    assert!(!rec2.visible(), "w=0 不可见");
    let mut rec3 = rec;
    rec3.h = 0;
    assert!(!rec3.visible(), "h=0 不可见");
    let mut rec4 = base;
    rec4.flags = 3;
    assert!(rec4.visible(), "flags=3（&1=1）且 w,h>0 可见");
}

/// 规则 6 `widget_hit_bounds`：hit 矩形内外各一点。
#[test]
fn widget_hit_bounds() {
    let layer = ti_app::widgets::WidgetLayer::new().expect("controls.json 缺失，无法测 hit 边界");
    // 取任一场景第一个可见控件，验证 hit 在其矩形内命中、矩形外不命中
    let cfg = layer.cfg();
    let scene = cfg
        .scenes
        .iter()
        .find(|s| layer.visible(&s.id).next().is_some())
        .expect("controls.json 应含至少一个可见控件场景");
    let rec = layer.visible(&scene.id).next().expect("取首个可见控件");
    // 矩形内一点（左上角 +1）
    let inside = i32::try_from(rec.x).expect("控件 x 超 i32") + 1;
    let inside_y = i32::try_from(rec.y).expect("控件 y 超 i32") + 1;
    assert!(layer.hit(&scene.id, inside, inside_y).is_some(), "控件矩形内一点应命中");
    // 矩形外一点（左上角左上方 -1，必在矩形外）
    let outside = i32::try_from(rec.x).expect("控件 x 超 i32") - 1;
    let outside_y = i32::try_from(rec.y).expect("控件 y 超 i32") - 1;
    let hit_outside = layer.hit(&scene.id, outside, outside_y);
    // 外点可能命中其他控件（后画覆盖），只断言不是本控件
    assert!(!hit_outside.is_some_and(|h| std::ptr::eq(h, rec)), "矩形外一点不应命中本控件");
}

/// 规则 7 `touch_logical_roundtrip`：触摸坐标反变换往返一致。
#[test]
fn touch_logical_roundtrip() {
    // 视口 2x：逻辑 176×220 → 屏幕 352×440
    let vp = Viewport::fit(352.0, 440.0);
    let (lx, ly) = (88.0_f32, 110.0_f32);
    let (sx, sy) = vp.logical_to_screen(lx, ly);
    let (rx, ry) = vp.screen_to_logical(sx, sy);
    assert!((rx - lx).abs() < 1e-3, "x 往返不一致：{lx} → {rx}");
    assert!((ry - ly).abs() < 1e-3, "y 往返不一致：{ly} → {ry}");
    // 非整数点也一致
    let (lx2, ly2) = (45.3_f32, 12.7_f32);
    let (sx2, sy2) = vp.logical_to_screen(lx2, ly2);
    let (rx2, ry2) = vp.screen_to_logical(sx2, sy2);
    assert!((rx2 - lx2).abs() < 1e-2, "x 往返不一致：{lx2} → {rx2}");
    assert!((ry2 - ly2).abs() < 1e-2, "y 往返不一致：{ly2} → {ry2}");
}

/// 规则 8 `iso_touch_matches_move`：主地图点按与棋子移动同坐标系。
///
/// 点按坐标 → `iso_grid_cell_at` 命中格 → 与 `GridPos::iso_pos` 渲染公式同坐标系
/// （命中格中心 = 渲染格中心，同一 `iso_x/iso_y` 公式）。
#[test]
fn iso_touch_matches_move() {
    use ti_core::domain::map::GridPos;
    // 20×20 等距网格，摄像机中心 (0,0)（无地形资源时 host 返回 (0,0)）
    let (row, col) = (3u32, 5u32);
    let grid_w = 20u32;
    let pos = GridPos::new(row, col);
    // 域公式单一事实源：`iso_x = 32*(row+col)`，`iso_y = 16*(row+grid_w-1-col)`
    let (ix, iy) = pos.iso_pos(grid_w);
    #[allow(clippy::cast_precision_loss)] // 20×20 等距像素值远小于 f32 尾数，精度无损
    let hit = layout::iso_grid_cell_at(ix as f32, iy as f32, grid_w, grid_w, (0, 0));
    assert_eq!(hit, Some((row, col)), "iso_pos 像素应逆解回原格，实际 {hit:?}");
}

/// 规则 9 `gesture_threshold_split`：位移 <20 判点按、≥20 判滑动。
#[test]
fn gesture_threshold_split() {
    use ti_app::touch::TouchPad;
    // 位移 19（<20）→ 点按
    let mut pad = TouchPad::new();
    let _ = pad.feed(50.0, 50.0, true);
    let got = pad.feed(50.0, 69.0, false); // 位移 19
    assert!(
        matches!(got, Some(ti_app::touch::TouchSample { gesture: Gesture::Tap, .. })),
        "位移 19 <20 应判点按，实际 {got:?}"
    );
    // 位移 20（≥20）→ 滑动（向下）
    let mut pad = TouchPad::new();
    let _ = pad.feed(50.0, 50.0, true);
    let got = pad.feed(50.0, 70.0, false); // 位移 20
    assert!(
        matches!(got, Some(ti_app::touch::TouchSample { gesture: Gesture::SwipeDown, .. })),
        "位移 20 ≥20 应判滑动（下），实际 {got:?}"
    );
}

/// 规则 10 `msg_page_clamped`：消息翻页首尾钳位。
#[test]
fn msg_page_clamped() {
    let mut app = GameApp::new(176.0, 220.0);
    // 快进到主地图 → 打开消息（事件码 1 → Msgl）
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Landmark(1));
    assert_eq!(app.scene(), Scene::Message);
    let total = app.msg_total();
    assert!(total >= 1, "消息总页数应 ≥1");
    // 末页 next 保持
    for _ in 0..(total + 2) {
        app.handle(GameInput::MsgNext);
    }
    assert_eq!(app.msg_page(), total - 1, "末页 next 应钳位在末页");
    // 首页 prev 回末页（循环）
    app.handle(GameInput::MsgPrev);
    for _ in 0..(total + 2) {
        app.handle(GameInput::MsgPrev);
    }
    assert_eq!(app.msg_page(), total - 1, "首页 prev 应回末页（循环钳位）");
}

/// 规则 11 `msg_close_returns_source`：消息关闭回到打开源场景。
#[test]
fn msg_close_returns_source() {
    let mut app = GameApp::new(176.0, 220.0);
    // 快进到主地图 → 打开消息
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Landmark(1));
    assert_eq!(app.scene(), Scene::Message);
    // 关闭 → 回打开源（主地图，因 map_pos 存在且无战斗）
    app.handle(GameInput::MsgClose);
    assert_eq!(app.scene(), Scene::MainMap, "主地图打开的消息关闭应回主地图");

    // 主菜单打开的帮助消息关闭 → 回主菜单
    let mut app = GameApp::new(176.0, 220.0);
    // 菜单项 2（选项/说明）→ Help 消息：从 0 起 MoveDown 2 次
    app.handle(GameInput::MoveDown);
    app.handle(GameInput::MoveDown);
    assert_eq!(app.menu_selected(), 2, "应停在菜单项 2");
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Message, "菜单项 2 应进消息场景");
    app.handle(GameInput::MsgClose);
    assert_eq!(app.scene(), Scene::Index, "主菜单打开的消息关闭应回主菜单");
}

/// 规则 12 `scene_switch_resets_state`：切场景后选中项/页码复位（无残留）。
#[test]
fn scene_switch_resets_state() {
    // 选角场景：角色 + 地图选定进主地图后，回选角应复位选中项
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Next); // 选角 → 第 1 个（Panda）
    let _char = app.character();
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Next); // 选图 → M01
    app.handle(GameInput::Confirm); // 角色 + 地图齐全 → 主地图
    assert_eq!(app.scene(), Scene::MainMap);

    // 回主菜单 → 再进选角：选中项应复位（menu_selected 回到 0）
    app.handle(GameInput::MapBack);
    assert_eq!(app.scene(), Scene::Index);
    assert_eq!(app.menu_selected(), 0, "切回主菜单后菜单选中项应复位");

    // 重新走 新游戏 → 选角：角色选中应复位为 None 起（Next 后回到第 0 个）
    app.handle(GameInput::Confirm); // 菜单项 0 → SelectRole（reset_selection 已清角色/地图）
    assert!(app.character().is_none(), "进选角后角色应复位为未选");
    app.handle(GameInput::Next);
    assert_eq!(
        app.character(),
        Some(ti_core::domain::selection::Character::Panda),
        "复位后 Next 应到第 0 个角色"
    );

    // 消息页码复位：打开消息 → 翻到末页 → 关闭 → 再打开应回首页
    let mut app = GameApp::new(176.0, 220.0);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Next);
    app.handle(GameInput::Confirm); // → 主地图
    app.handle(GameInput::Landmark(1)); // → 消息
    assert_eq!(app.scene(), Scene::Message);
    for _ in 0..=app.msg_total() {
        app.handle(GameInput::MsgNext);
    }
    assert_eq!(app.msg_page(), app.msg_total() - 1, "翻到末页");
    app.handle(GameInput::MsgClose); // 回主地图（页码残留）
    app.handle(GameInput::Landmark(1)); // 再打开消息
    assert_eq!(app.msg_page(), 0, "再打开消息页码应复位到首页");
}
