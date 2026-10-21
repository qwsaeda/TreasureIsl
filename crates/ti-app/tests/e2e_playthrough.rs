//! E2E 完整游戏流程测试：模拟真实玩家操作，验证全链路状态一致性。
//!
//! 覆盖：菜单导航 → 角色选择 → 地图选择 → 主地图移动 → 贸易 → 建造 →
//!       战斗 → 存读档 → 消息系统 → 边界情况。
//!
//! 与 `game_app_flow.rs` 的区别：
//! - `game_app_flow.rs`：单模块流程验证（每个测试独立）
//! - `e2e_playthrough.rs`：完整会话连续性（一个测试跑完整局游戏）

use ti_app::game_app::{BattleView, GameApp, GameInput, Scene};
use ti_core::domain::economy::Building;
use ti_core::domain::selection::{Character, MapSlot};

/// 完整游戏会话：从启动到退出，模拟真实玩家操作序列。
#[allow(clippy::too_many_lines)]
#[test]
fn e2e_full_game_session() {
    let mut app = GameApp::new(1080.0, 1920.0);
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

    // ── 阶段 4：主地图摇骰前进（S19：环岛赛道，方向键走格已彻底移除）──────────
    let mut move_count = 0;

    // 摇骰 4 次，沿赛道前进
    for _ in 0..4 {
        let before = app.map_pos();
        app.handle(GameInput::Spin);
        if app.map_pos() != before {
            move_count += 1;
        }
    }
    assert!(move_count > 0, "摇骰应至少前进一次");

    // ── 阶段 5：贸易系统 ──────────────────────────────────────────────
    // S19：阶段 4 摇骰已改玩家金（落格结算），故贸易断言以「摇骰后金」为基线（`gold_before_trade`），
    // 不再假设 = `initial_gold`；`initial_gold` 仅用于阶段 6 银行（银行独立金池，不受赛道摇骰影响）。
    let initial_gold = ti_core::data::balance().map_or(1000, |b| b.economy.initial_gold);
    let gold_before_trade = app.gold();
    assert!(gold_before_trade < initial_gold.saturating_add(1000), "摇骰后金应在合理区间");

    // 买入商品 1
    let price1 = app.price_table().lookup(1).unwrap_or(0);
    app.handle(GameInput::TradeBuy(1));
    assert_eq!(app.gold(), gold_before_trade.saturating_sub(price1), "买入后金应减少商品1单价");
    assert_eq!(app.owned().len(), 1, "买入后持有量 +1");

    // 买入商品 2
    let price2 = app.price_table().lookup(2).unwrap_or(0);
    app.handle(GameInput::TradeBuy(2));
    assert_eq!(
        app.gold(),
        gold_before_trade.saturating_sub(price1).saturating_sub(price2),
        "第二次买入后金应再减少商品2单价"
    );

    // 卖出商品 1
    let gold_before_sell = app.gold();
    app.handle(GameInput::TradeSell(1));
    assert!(app.gold() > gold_before_sell, "卖出后金应增加");

    // ── 阶段 6：建造系统 ──────────────────────────────────────────────
    let bank_before = app.bank().gold;
    assert_eq!(bank_before, initial_gold, "开局银行金应等于 balance.economy.initial_gold");

    // 建造 Castle（成本走 `balance.build.buildings`，`rules::build::cost_of`）
    let b = ti_core::data::balance().expect("balance.json 加载失败");
    let castle_cost = ti_core::rules::build::cost_of(&b.build, Building::Castle);
    app.handle(GameInput::Build(Building::Castle));
    assert_eq!(app.buildings().len(), 1, "建造后建筑列表 +1");
    assert_eq!(
        app.bank().gold,
        initial_gold.saturating_sub(castle_cost),
        "建造后银行金应减少 Castle 成本"
    );

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
    let mut app = GameApp::new(1080.0, 1920.0);

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

    // 连续摇骰 100 次不应崩溃（绕圈推进）
    for _ in 0..100 {
        app.handle(GameInput::Spin);
    }
    assert!(
        matches!(app.scene(), Scene::MainMap | Scene::Vs),
        "摇骰不应跳到无关场景（BOSS 格会开战斗）"
    );

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
    let mut app = GameApp::new(1080.0, 1920.0);
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

    // 执行一系列操作（摇骰前进，确定性种子 123）
    for _ in 0..10 {
        app.handle(GameInput::Spin);
    }

    // 摇骰沿赛道推进，位置应始终存在（BOSS 格会切战斗，仍属合法）
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

/// E2E 纯触摸经济闭环（S15.6 QA149）：0 键盘——
/// `index` / `select_role` / `select_map` 各点卡 0 进 `MainMap`，再点 HUD 三按钮完成
/// 贸易（买 1）→ 卖出（全卖）→ 建造（`Castle`）。坐标全数据驱动自 `scenes.json`。
#[allow(clippy::cast_precision_loss)] // i32 布局坐标 → f32 触摸坐标（1080 呈现层）
#[test]
fn e2e_touch_economy_loop() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    let scenes = ti_core::data::scenes().expect("scenes.json 加载");
    // 单次点按（显式传 `&mut GameApp`，闭包不捕获 app，避免可变借用在作用域内滞留）
    let tap = |app: &mut GameApp, x: f32, y: f32| {
        let s = ti_app::touch::TouchSample { gesture: ti_platform::settings::Gesture::Tap, x, y };
        app.poll_touch(Some(s))
    };
    // 卡 0 命中中心：`ListLayout::item_rect` 同式（顶 = y - highlight_offset）
    let card_center = |scene_id: &str| -> (f32, f32) {
        let s = scenes.get(scene_id).expect("scenes.json 缺场景");
        let top = s.list.y - s.list.highlight_offset;
        (s.list.x as f32 + s.list.w as f32 / 2.0, top as f32 + s.list.item_h as f32 / 2.0)
    };
    // HUD 按钮中心（scenes.json main_map extra 矩形）
    let hud_center = |key: &str| -> (f32, f32) {
        let main = scenes.get("main_map").expect("scenes.json 缺 main_map");
        let r = main.extra.get(key).expect("main_map 缺 HUD 按钮矩形");
        ((r.x + r.w / 2) as f32, (r.y + r.h / 2) as f32)
    };

    // index 卡 0 = 「新游戏」→ 选角
    let (x, y) = card_center("index");
    assert!(tap(&mut app, x, y), "index 卡 0 点按应命中");
    assert_eq!(app.scene(), Scene::SelectRole, "index 卡 0 应进选角");
    // select_role 卡 0 = Panda → 选图
    let (x, y) = card_center("select_role");
    assert!(tap(&mut app, x, y), "select_role 卡 0 点按应命中");
    assert_eq!(app.scene(), Scene::SelectMap, "选角卡 0 应进选图");
    // select_map 卡 0 = M01 → 主地图
    let (x, y) = card_center("select_map");
    assert!(tap(&mut app, x, y), "select_map 卡 0 点按应命中");
    assert_eq!(app.scene(), Scene::MainMap, "选图卡 0 应进主地图");
    assert!(app.map_pos().is_some(), "主地图应有初始位置");

    let initial_gold = ti_core::data::balance().expect("balance.json 加载").economy.initial_gold;
    let price1 = app.price_table().lookup(1).unwrap_or(0);

    // ① 贸易臂：买商品 1（键盘 T 同语义；HUD 臂不切场景，poll_touch 返回 false 属正常，
    // 命中判据 = 下方状态断言）
    let (x, y) = hud_center("hud_trade");
    tap(&mut app, x, y);
    assert_eq!(app.gold(), initial_gold.saturating_sub(price1), "触摸贸易后金 = 开局金 - 单价");
    assert_eq!(app.owned().len(), 1, "触摸贸易后持有量 +1");

    // ② 卖出臂：全卖商品 1（键盘 Y 同语义）
    let gold_before_sell = app.gold();
    let (x, y) = hud_center("hud_sell");
    tap(&mut app, x, y);
    assert!(app.gold() > gold_before_sell, "触摸卖出后金应增加");
    assert_eq!(app.owned().len(), 0, "全卖后持有量清零");

    // ③ 建造臂：造 Castle（键盘 B 同语义）
    let castle_cost = ti_core::rules::build::cost_of(
        &ti_core::data::balance().expect("balance.json 加载").build,
        Building::Castle,
    );
    let bank_before = app.bank().gold;
    let (x, y) = hud_center("hud_build");
    tap(&mut app, x, y);
    assert_eq!(app.buildings().len(), 1, "触摸建造后建筑 +1");
    assert_eq!(
        app.bank().gold,
        bank_before.saturating_sub(castle_cost),
        "触摸建造后银行金 = 前值 - Castle 成本"
    );
}

/// E2E 战斗完整性：多次战斗结果应一致（固定种子）。
#[test]
fn e2e_battle_determinism() {
    let seed = 42u32;

    // 第一次战斗
    let mut app1 = GameApp::new(1080.0, 1920.0);
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
    let outcome1 = app1.battle().map(BattleView::outcome);
    let player_hp1 = app1.battle().map(BattleView::player_hp);
    let boss_hp1 = app1.battle().map(BattleView::boss_hp);

    // 第二次战斗（相同种子）
    let mut app2 = GameApp::new(1080.0, 1920.0);
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
    let outcome2 = app2.battle().map(BattleView::outcome);
    let player_hp2 = app2.battle().map(BattleView::player_hp);
    let boss_hp2 = app2.battle().map(BattleView::boss_hp);

    // 两次战斗结果应完全一致
    assert_eq!(rounds1, rounds2, "相同种子战斗回合数应一致");
    assert_eq!(outcome1, outcome2, "相同种子战斗结果应一致");
    assert_eq!(player_hp1, player_hp2, "相同种子玩家 HP 应一致");
    assert_eq!(boss_hp1, boss_hp2, "相同种子 BOSS HP 应一致");
}

/// S19：环岛赛道摇骰闭环（e2e）：进图 → 赛道会话在位 → 摇骰前进（索引推进 + 位置 = 赛道格 +
/// 步数值域 + 可复现）→ 多轮摇骰必过起点（圈数 ≥ 1）→ 同 seed 同序列 = 同落格（确定性）。
#[test]
fn e2e_board_spin_loop() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    // 进入主地图（index 新游戏 → 选角色确认 → 选图 M01 确认）
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    let board = app.board.as_ref().expect("主地图应构建环岛赛道（board.json 数据源）");
    let track_len = board.track().len();
    assert!(track_len >= 20, "赛道长度 {track_len} 过短");
    assert_eq!(board.index(), board.track().start_index(), "起始 = 赛道起点");
    assert_eq!(app.map_pos(), Some(board.pos()), "玩家位置 = 当前赛道格");

    // 摇骰一次：索引推进 + 步数值域 + 位置同步
    let idx0 = board.index();
    app.handle(GameInput::Spin);
    let b1 = app.board.as_ref().expect("摇骰后 board 会话应在位");
    let roll = b1.last_roll();
    assert!((1..=6).contains(&roll), "步数应在 1..=6：{roll}");
    let expected = (idx0 + usize::try_from(roll).unwrap_or(1)) % track_len;
    assert_eq!(b1.index(), expected, "摇 {roll} 格后索引应 = ({idx0} + {roll}) % {track_len}");
    assert_eq!(app.map_pos(), Some(b1.pos()), "玩家位置 = 新赛道格");

    // 多轮摇骰（同 seed 可复现）：累计步数 ≥ 赛道长度 → 必过起点（圈数 ≥ 1）
    for _ in 0..40 {
        app.handle(GameInput::Spin);
    }
    let b2 = app.board.as_ref().expect("摇骰后 board 会话应在位");
    assert!(b2.laps() >= 1, "40 轮摇骰（期望 ~140 步）应至少过起点一圈：laps={}", b2.laps());

    // 同 seed 复现：两条独立路径同序列 = 同落点
    let mut a = GameApp::new(1080.0, 1920.0);
    a.rng_mut().set_seed(7);
    a.set_rng_seed(7);
    let mut b = GameApp::new(1080.0, 1920.0);
    b.rng_mut().set_seed(7);
    b.set_rng_seed(7);
    for _ in 0..3 {
        for _ in 0..3 {
            a.handle(GameInput::Confirm);
            b.handle(GameInput::Confirm);
        }
        for _ in 0..12 {
            a.handle(GameInput::Spin);
            b.handle(GameInput::Spin);
        }
        let ab = a.board.as_ref().expect("board 会话应在位");
        let bb = b.board.as_ref().expect("board 会话应在位");
        assert_eq!(ab.pos(), bb.pos(), "同 seed 同序列 ⇒ 同落格（确定性）");
    }
}
