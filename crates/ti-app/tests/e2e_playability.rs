#![allow(clippy::unwrap_used, clippy::expect_used)]
//! S74.3 玩家视角完整链路 E2E（诊断：门禁重文档轻玩法，玩家流程测不到）。
//!
//! 覆盖（真实可失败的玩家可观测行为，非摆设断言）：
//! 1. `playability_full_match_to_victory_and_restart`：从菜单进 2 人热座局 →
//!    摇骰/交接 → 破产终局 Victory → 终局面板 Confirm 回主菜单 → 新局全量复位。
//! 2. `playability_defeat_when_all_human_seats_bankrupt`：全人类席破产 ⇒ Defeat
//!    + 玩家可见出口（终局 Confirm 回菜单）。
//! 3. `playability_siege_and_capture`：落他人领地格 → Confirm 攻占 → Vs 战报
//!    Attack 强攻胜 → 该岛两格易主 + level 1 + 驻军重定 → Back 回主地图续玩。
//! 4. `playability_multi_round_economy_loop`：4 轮买→卖→建造闭环，每轮精确核账
//!    （金 ±单价 / 银行金 -=cost / 持有量增删 / 金 ≤ 上限），闭环后仍可摇骰。
//! 5. `playability_ai_turns_visible_and_deterministic`：2 人类 + 3 AI 局多回合
//!    场景恒主地图 + AI toast 可见 + 同 seed 同序列结果一致。
//! 6. `playability_save_and_resume`：真实 UI 写档 → 新会话读档续玩（金/船级/位置/
//!    地图位还原 + 读档后首步摇骰推进），对应 persistence/territory 存档规则。
//! 7. `playability_layout_key_controls_in_viewport`：数据驱动元素级版面断言
//!    （同 `element-dump` 思路，判据落在现行 1080 scenes.json）：文本原语在视口内、
//!    字号代理高度 ≥ 阈值、标题条与首列表项不重叠。
//!
//! 复用 `e2e_playthrough.rs` / `app_seats_e2e.rs` / `app_siege_e2e.rs` /
//! `app_adversarial_e2e.rs` 的 `GameApp` 公开 API 与 `handle(GameInput)` 模式，
//! 不重复造轮子；本文件只新增玩家视角的**完整链路**断言。

use ti_app::game_app::MatchOutcome;
use ti_app::{GameApp, GameInput, MapHost, MenuHost, Scene};
use ti_core::domain::economy::Building;
use ti_core::rules::territory_v2::SeatState;

// ── helpers ────────────────────────────────────────────────────────────────────

/// 走真实 UI 路径进单玩家局主地图（旧单选流回退，0 `MarkSeat`）。
fn enter_main_map(seed: u32) -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(seed);
    app.set_rng_seed(seed);
    app.handle(GameInput::Confirm); // Index → SelectRole
    app.handle(GameInput::Confirm); // 空 seat_marks = 旧流豁免 → SelectMap
    app.handle(GameInput::Next); // M01
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap, "应进主地图");
    app
}

/// 走真实 UI 路径进 2 人热座 + `ai` 个 AI 席主地图（`app_seats_e2e` 同模式）。
fn enter_hot_seat_with_ai(seed: u32, ai: u32) -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(seed);
    app.set_rng_seed(seed);
    app.handle(GameInput::Confirm); // Index → SelectRole
    app.handle(GameInput::Next);
    app.handle(GameInput::MarkSeat); // 席 0
    app.handle(GameInput::Next);
    app.handle(GameInput::MarkSeat); // 席 1
    app.seat_ai_count = Some(ai);
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Next); // M01
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap, "真实 UI 路径进主地图");
    app
}

/// 主地图 Confirm 确认热座交接（pending 在位才调用）。
fn confirm_handoff(app: &mut GameApp) {
    assert!(app.handoff_pending.is_some(), "交接应在位");
    app.handle(GameInput::Confirm);
    assert!(app.handoff_pending.is_none(), "确认后 pending 清空");
}

// ── 1. 一局到胜利 + 重开 ────────────────────────────────────────────────────────

/// 完整对局闭环：2 人热座 → 摇骰/交接 → 破产终局 Victory(seat 0) → 终局面板
/// Confirm 回主菜单 → 新局全量复位（终局清 / 席位复活 / 交接清）。
#[test]
fn playability_full_match_to_victory_and_restart() {
    let mut app = enter_hot_seat_with_ai(7, 0);
    assert!(app.match_outcome.is_none(), "开局无终局");
    assert_eq!(app.turn_cycle.current, 0, "开局当前席 = 玩家 0");

    // 玩家 0 摇骰 → run_ai_turns 轮转 → 交接给玩家 1
    app.handle(GameInput::Spin);
    assert!(app.handoff_pending.is_some(), "玩家 1 热座交接应在位");
    confirm_handoff(&mut app);
    assert_eq!(app.turn_cycle.current, 1, "确认交接后当前席 = 玩家 1");

    // 玩家 1 账本清零（模拟破产）→ 玩家 1 摇骰触发 check_match_end
    app.seat_gold[1] = 0;
    app.seat_crew[1] = 0;
    app.handle(GameInput::Spin);
    assert!(!app.turn_cycle.is_alive(1), "玩家 1 破产出局（is_alive=false）");
    assert!(
        matches!(app.match_outcome, Some(MatchOutcome::Victory { seat: 0 })),
        "最后存活席 0 ⇒ Victory{{seat:0}}，实际 = {:?}",
        app.match_outcome
    );

    // 玩家可见出口：终局面板 Confirm 清终局 + 回主菜单
    assert!(app.handle(GameInput::Confirm), "终局 Confirm 应消费（切场景）");
    assert_eq!(app.scene(), Scene::Index, "终局 Confirm 应回主菜单");
    assert!(app.match_outcome.is_none(), "终局已清（零残留）");

    // 重开一局：新对局全量复位，玩家可继续玩
    app.handle(GameInput::Confirm); // → SelectRole
    app.handle(GameInput::Confirm); // → SelectMap
    app.handle(GameInput::Confirm); // → MainMap
    assert_eq!(app.scene(), Scene::MainMap, "新局应可进主地图");
    assert!(app.match_outcome.is_none(), "新局无终局残留");
    assert!(app.turn_cycle.is_alive(0) && app.turn_cycle.is_alive(1), "新局各席复活");
    assert!(app.handoff_pending.is_none(), "新局无交接残留");
}

// ── 2. 一局到失败（全人类席破产）──────────────────────────────────────────────

/// 2 人局全人类席破产 ⇒ `MatchOutcome::Defeat` + 终局 Confirm 回菜单（玩家可见出口）。
#[test]
fn playability_defeat_when_all_human_seats_bankrupt() {
    let mut app = enter_hot_seat_with_ai(7, 0);
    app.handle(GameInput::Spin);
    confirm_handoff(&mut app);
    app.handle(GameInput::Spin);
    confirm_handoff(&mut app);
    assert_eq!(app.turn_cycle.current, 0, "两轮后当前席回到玩家 0");

    app.seat_gold[0] = 0;
    app.seat_crew[0] = 0;
    app.seat_gold[1] = 0;
    app.seat_crew[1] = 0;
    app.handle(GameInput::Spin); // 玩家 0 摇骰 ⇒ check_match_end 双席破产
    assert!(!app.turn_cycle.is_alive(0), "玩家 0 出局");
    assert!(!app.turn_cycle.is_alive(1), "玩家 1 出局");
    assert!(
        matches!(app.match_outcome, Some(MatchOutcome::Defeat)),
        "全人类席破产 = Defeat，实际 = {:?}",
        app.match_outcome
    );

    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Index, "失败终局面板 Confirm 应回主菜单");
}

// ── 3. 领地攻占 ─────────────────────────────────────────────────────────────────

/// 让 AI 席 1 占领 m01 的 harbour 格（规则层 `capture_cell` 同源夹具）。
fn occupy_harbour_for_ai(app: &mut GameApp) -> u32 {
    let cell = app
        .territory
        .as_ref()
        .expect("领地账本应在位")
        .isle_cells("harbour")
        .first()
        .copied()
        .expect("m01 应有 harbour 岛");
    let mut seat = SeatState::new(1000, 30, 20, 0);
    if let Some(terr) = app.territory.as_mut() {
        let _ = ti_core::rules::territory_v2::capture_cell(&mut seat, terr, cell, 1, 5);
    }
    cell
}

/// 玩家视角攻占闭环：落他人格 → Confirm 进 Vs 战报 → 强攻胜 ⇒ 两格易主 +
/// level 1 + 驻军重定 → Back 回主地图可继续。
#[test]
fn playability_siege_and_capture() {
    let mut app = enter_main_map(11);
    let target = occupy_harbour_for_ai(&mut app);

    let mut saw_pending = false;
    for _ in 0..80 {
        app.handle(GameInput::Spin);
        if let Some((cell, owner)) = app.pending_siege {
            if cell == target && owner == 1 {
                saw_pending = true;
                break;
            }
        }
    }
    assert!(saw_pending, "绕环 80 次摇骰应落一次 AI 占的 harbour 格");

    // 玩家二选一：Confirm = 攻占（Back = 交过路费是另一分支，见 app_siege_e2e）
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Vs, "Confirm 攻占应切 Vs 战报");
    assert!(app.siege_snapshot().is_some(), "Vs 战报应在位");

    let my_seat = app.turn_cycle.current;
    // 强攻夹具（来源：force 战力 = armed×2 + unarmed×1；守军 5 + defence L1≈28 ⇒ 400 > 61）
    app.seat_crew[my_seat as usize] = 200;
    app.seat_arms[my_seat as usize] = 200;
    app.handle(GameInput::Attack);

    let siege = app.siege_snapshot().expect("结算后战报仍在");
    assert_eq!(siege.won, Some(true), "强攻方必胜（兵力模型确定性）");
    let cell = siege.cell;
    let terr = app.territory.as_ref().expect("账本");
    let isle = terr.isle_cells_by_index(cell).expect("该格有岛").to_string();
    for idx in terr.isle_cells(&isle) {
        let own = terr.ownership(*idx).copied().expect("格在账本");
        assert_eq!(own.owner, Some(my_seat), "攻占胜：格 {idx} 归攻方");
        assert_eq!(own.level, 1, "攻占胜：格 {idx} 守军降级 1");
    }
    assert!(terr.garrison_total(my_seat) > 0, "攻占胜：重定驻军 > 0（守恒）");
    assert!(
        app.seat_crew[my_seat as usize] + terr.garrison_total(my_seat) <= 200,
        "人手守恒：战后舰队 + 驻军 ≤ 初始 200"
    );

    // 玩家可见出口：战报 Back 回主地图，攻占结果已写账本
    app.handle(GameInput::Back);
    assert_eq!(app.scene(), Scene::MainMap, "战报 Back 应回主地图续玩");
}

// ── 4. 多轮经济闭环 ─────────────────────────────────────────────────────────────

/// 4 轮「买→卖→建造」闭环，每轮精确核账；闭环后世界仍响应（摇骰推进）。
#[test]
fn playability_multi_round_economy_loop() {
    let mut app = enter_main_map(7);
    let b = ti_core::data::balance().expect("balance.json 加载");
    // 夹具注资：balance.build.buildings 4 类 base_cost 合计 2500 < 5000（来源注明，
    // 非硬编码业务数值——造价仍经 `rules::build::cost_of` 实算）。
    app.bank.gold += 5000;

    for (round, building) in Building::all().into_iter().enumerate() {
        // 买：金 -= 单价，持有 +1
        let gold_before_buy = app.gold();
        let price = app.price_table().lookup(1).unwrap_or(0);
        assert!(price > 0, "价格表应有商品 1 单价");
        app.handle(GameInput::TradeBuy(1));
        assert_eq!(
            app.gold(),
            gold_before_buy.saturating_sub(price),
            "第 {round} 轮买入金 -= 单价"
        );
        assert!(
            app.owned().iter().any(|(id, qty)| *id == 1 && *qty > 0),
            "第 {round} 轮买入后持有商品 1"
        );

        // 卖：金增加（markup>0），持有清零
        let gold_before_sell = app.gold();
        app.handle(GameInput::TradeSell(1));
        assert!(app.gold() > gold_before_sell, "第 {round} 轮卖出金应增加");
        assert!(app.owned().iter().all(|(id, _)| *id != 1), "第 {round} 轮卖出后商品 1 清零");

        // 建造：银行金 -= 成本，建筑列表 +该类
        let bank_before_build = app.bank().gold;
        let cost = ti_core::rules::build::cost_of(&b.build, building);
        app.handle(GameInput::Build(building));
        assert_eq!(
            app.bank().gold,
            bank_before_build.saturating_sub(cost),
            "第 {round} 轮建造银行金 -= 成本（{building:?}）"
        );
        assert!(
            app.buildings().iter().any(|(kind, _)| *kind == building),
            "第 {round} 轮建造后列表含 {building:?}"
        );
        assert!(app.gold() <= b.economy.gold_max, "金恒 ≤ 上限");
    }
    assert_eq!(app.buildings().len(), 4, "4 轮各建 1 类 ⇒ 4 建筑在列");

    // 闭环后仍可玩：摇骰推进赛道（非空转）
    let idx_before = app.board_index();
    app.handle(GameInput::Spin);
    assert_ne!(app.board_index(), idx_before, "经济闭环后摇骰应推进赛道");
}

// ── 5. AI 回合行为 ──────────────────────────────────────────────────────────────

fn alive_mask(app: &GameApp) -> Vec<bool> {
    (0..5u8).map(|seat| app.turn_cycle.is_alive(seat)).collect()
}

/// 2 人类 + 3 AI 多回合：场景恒主地图 + AI toast 可见 + 同 seed 同序列结果一致。
#[test]
fn playability_ai_turns_visible_and_deterministic() {
    let mut app = enter_hot_seat_with_ai(2026, 3);
    let mut saw_ai_toast = false;
    for _ in 0..8 {
        if app.handoff_pending.is_some() {
            confirm_handoff(&mut app);
        }
        app.handle(GameInput::Spin);
        // BOSS 现身切 Vs 也是合法玩家可见分支（同 seed 2026 绕满圈后会发生）；
        // 断言场景不跳到无关面（Index/Saveload/崩溃），非空转。
        assert!(
            matches!(app.scene(), Scene::MainMap | Scene::Vs),
            "AI 自动结算后场景应为主地图或 BOSS 战，实际 = {:?}",
            app.scene()
        );
        if app.last_ai_toast.is_some() {
            saw_ai_toast = true;
        }
    }
    assert!(saw_ai_toast, "8 回合内应有 AI 落格反馈（可见行为，非空转）");

    let run = |seed: u32| {
        let mut a = enter_hot_seat_with_ai(seed, 3);
        for _ in 0..8 {
            if a.handoff_pending.is_some() {
                a.handle(GameInput::Confirm);
            }
            a.handle(GameInput::Spin);
        }
        (alive_mask(&a), a.last_ai_toast.clone(), a.seat_gold)
    };
    assert_eq!(run(2026), run(2026), "同 seed 同 8 回合 ⇒ 存活位/末次 AI toast/金币一致");
}

// ── 6. 存档续玩 ─────────────────────────────────────────────────────────────────

/// 真实 UI 写档 → 新会话读档 → 状态还原 → 读档后首步仍可玩（摇骰推进）。
#[test]
fn playability_save_and_resume() {
    let mut app = enter_main_map(11);
    app.handle(GameInput::Spin);
    app.handle(GameInput::TradeBuy(1));
    app.set_boat_level(2);
    let gold_at_save = app.gold();
    let pos_at_save = app.map_pos();
    let slot_at_save = app.map_slot();

    // 经真实 UI 进存读档场景写槽 0（MapBack → 菜单项 1 → Confirm）
    app.handle(GameInput::MapBack);
    assert_eq!(app.scene(), Scene::Index, "MapBack 应回主菜单");
    app.set_menu_selected(1);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::Saveload, "菜单项 1 应进存读档");
    app.handle(GameInput::WriteSave(0));
    let saved = app.save_slots()[0].clone();
    assert_eq!(saved.gold, gold_at_save, "写档应捕获当前金");
    assert_eq!(saved.boat_level, 2, "写档应捕获船级 2");

    // 续玩：新会话读档（restore_save = Saveload 场景 LoadSave 的同源调用）
    let mut resumed = GameApp::new(1080.0, 1920.0);
    resumed.restore_save(saved);
    assert_eq!(resumed.scene(), Scene::MainMap, "读档应进主地图续玩");
    assert_eq!(resumed.gold(), gold_at_save, "读档还原金");
    assert_eq!(resumed.boat_level(), 2, "读档还原船级");
    assert_eq!(resumed.map_slot(), slot_at_save, "读档还原地图位");
    assert_eq!(resumed.map_pos(), pos_at_save, "读档还原当前位置");

    // 续玩首步：读档后摇骰仍推进（真实可失败的世界响应断言）
    let idx_before = resumed.board_index();
    resumed.handle(GameInput::Spin);
    assert_ne!(resumed.board_index(), idx_before, "续玩首步摇骰应推进赛道");
}

// ── 7. 元素级版面断言（现行 1080 scenes.json，element-dump 思路）─────────────────

/// 数据驱动元素级版面：关键文本原语在 1080×1920 视口内、字号代理高度 ≥ 阈值、
/// 标题条与首列表项不重叠。判据取 `assets/data/scenes.json`（单一事实源），
/// 与 `element-dump` 同思路但落现行 1080 数据（legacy .Map 已退役为参照物）。
#[test]
fn playability_layout_key_controls_in_viewport() {
    const VIEW_W: i32 = 1080;
    const VIEW_H: i32 = 1920;
    const MIN_TEXT_H: i32 = 32; // 文本矩形高度 = 字号代理（实测最小 40）

    let scenes = ti_core::data::scenes().expect("scenes.json 加载");
    for key in ["index", "select_role", "select_map", "saveload", "message", "main_map", "vs"] {
        let layout = scenes.get(key).expect("scenes.json 缺关键场景");
        let texts: Vec<_> = layout.primitives.iter().filter(|p| p.kind == "text").collect();
        assert!(!texts.is_empty(), "{key} 应至少有 1 个文本原语");
        for prim in texts {
            let rect = &prim.rect;
            assert!(
                rect.x >= 0
                    && rect.y >= 0
                    && rect.x + rect.w <= VIEW_W
                    && rect.y + rect.h <= VIEW_H,
                "{key} 文本原语越出 1080×1920 视口: {rect:?}"
            );
            assert!(
                rect.h >= MIN_TEXT_H,
                "{key} 文本原语高度 {} < {MIN_TEXT_H}（字号阈值代理）",
                rect.h
            );
        }
    }

    // 关键控件不重叠：标题条底 < 首列表项顶（列表场景）
    for key in ["select_role", "select_map", "saveload"] {
        let layout = scenes.get(key).expect("scenes.json 缺列表场景");
        let title_bottom = layout.title.y + layout.title.h;
        let first_item_top = layout.list.item_rect(0).y;
        assert!(
            title_bottom < first_item_top,
            "{key} 标题条底 {title_bottom} 与首列表项顶 {first_item_top} 重叠"
        );
    }
}
