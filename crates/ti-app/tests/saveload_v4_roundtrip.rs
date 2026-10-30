//! S28.2 存档 v4 往返（`territory.md` §8）：e2e 存读档还原归属/各席位金/当前席；
//! 旧档（v3，无 v4 扩展）读回 0 报错。
//!
//! 判据：写档捕获 `SaveSlotV4Ext`（5 席 + 领地图 + 当前席）→ 新会话 `restore_save`
//! 后：领地归属/席位金/当前席逐字段还原；`v4 = None`（旧档）= 领地空账本 + 无 panic。

use ti_app::{GameApp, GameInput, MapHost, Scene};
use ti_core::domain::persistence::SaveSlot;
use ti_core::rules::territory::{Ownership, TerritoryState};

/// 进主地图并构建带占领的领地账本（确定性入口 = e2e 同款 3×Confirm）。
fn app_with_territory() -> GameApp {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(7);
    app.set_rng_seed(7);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    app.handle(GameInput::Confirm);
    assert_eq!(app.scene(), Scene::MainMap, "应进入主地图");
    // 主玩家摇骰落格（无主格 ⇒ 无主分支；有主格 ⇒ 现状不变；两种都安全走账本）
    app.handle(GameInput::Spin);
    app
}

#[test]
fn v4_roundtrip_restores_ownership_seat_gold_and_turn() {
    let mut app = app_with_territory();
    // 造一个有主的领地快照写入（不依赖摇骰落格恰好无主）
    let bal = ti_core::data::balance().expect("balance.json 缺位");
    let db = ti_core::data::board().expect("board.json 缺位");
    let map_id = app.map_slot().map_or("m01".to_string(), |ms| ms.id().to_string());
    let map_def = db.map(&map_id).expect("地图定义缺位");
    let mut terr = TerritoryState::new(map_def, bal).expect("领地账本构建");
    // 取该图一个领地格塞归属（测试面合法输入 = `mutate`）
    let first_cell = terr.snapshot().keys().next().copied().expect("领地图至少一格");
    terr.mutate(first_cell, Ownership { owner: Some(2), level: 3, garrison: 0 });
    app.territory = Some(terr);
    app.seat_gold[1] = 500;
    app.turn_cycle.current = 2;

    // 写档（trait 捕获）
    let ext = app.save_v4_ext().expect("v4 扩展应捕获");
    assert_eq!(ext.seats.len(), 5, "5 席位全量");
    assert_eq!(ext.seats[1].gold, 500, "席位 1 金");
    assert_eq!(ext.current_seat, 2, "当前席");
    let has_owner = ext
        .territories
        .iter()
        .any(|t| t.cells.get(&first_cell).is_some_and(|(o, l)| *o == 2 && *l == 3));
    assert!(has_owner, "领地图应含 seat2 占据的格（level 3）");

    let slot = SaveSlot { version: 4, v4: Some(Box::new(ext)), ..SaveSlot::default() };

    // 新会话读档还原
    let mut fresh = app_with_territory();
    fresh.territory = None;
    fresh.seat_gold = [0; 5];
    fresh.turn_cycle.current = 0;
    fresh.write_save_slot(0, slot);
    let data = fresh.save_slots()[0].clone().clone();
    fresh.restore_save(data);

    let restored = fresh.territory.as_ref().expect("读档后领地账本应在位");
    assert_eq!(
        restored.ownership(first_cell).copied(),
        Some(Ownership { owner: Some(2), level: 3, garrison: 0 }),
        "归属还原"
    );
    assert_eq!(fresh.seat_gold[1], 500, "席位金还原");
    assert_eq!(fresh.turn_cycle.current, 2, "当前席还原");
}

#[test]
fn v3_slot_reads_back_without_error_and_empty_territory() {
    // 旧档（v3）= v4 扩展缺省 None：读回 0 报错 + 领地 = 空账本
    let mut app = app_with_territory();
    let slot = SaveSlot { version: 3, ..SaveSlot::default() };
    app.write_save_slot(0, slot);
    let data = app.save_slots()[0].clone().clone();
    assert!(data.v4.is_none(), "旧档无 v4 扩展");
    app.restore_save(data); // 0 报错（不 panic 即过）
    // 旧档读回后领地 = 空账本（全图无主）
    let terr = app.territory.as_ref().expect("读档后应重建领地账本");
    assert!(terr.snapshot().iter().all(|(_, (owner, _))| owner.is_none()), "旧档读回 = 空领地图");
}
