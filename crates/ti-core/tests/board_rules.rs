//! 环岛赛道规则集成测试（S52.1；`docs/business/progression.md` §环岛赛道）。
//!
//! S52.1 补测：`board.rs` `Track` / `BoardSession` 查询 fn 此前 0 集成测试
//! （飞轮 R2 找问题 P1-8：index_of/start_index/boss_index/position_index/last_roll/landing）。
//! oracle = `progression.md` §2（31 格赛道：start 在格 0，boss 在 `31//2` = 格 15）。
//! 判据：`cargo test -p ti-core --test board_rules` 全绿。

use ti_core::data::board;
use ti_core::rules::board::{BoardSession, Landing, Track};

fn track() -> Track {
    let db = board().expect("board.json");
    let map = db.map("m01").expect("m01");
    Track::from_map(map).expect("m01 赛道构建")
}

/// `start_index`：起点在格 0（`progression.md` §2 不变量）。
#[test]
fn start_index_is_zero() {
    assert_eq!(track().start_index(), 0, "m01 起点 = 格 0");
}

/// `boss_index`：BOSS 在环半周 `31 // 2` = 格 15（非末格；`territory.md` §2）。
#[test]
fn boss_index_is_midway_not_last() {
    let t = track();
    let boss = t.boss_index().expect("m01 有 BOSS 格");
    assert_eq!(boss, t.len() / 2, "BOSS = 环半周（len/2）");
    assert_ne!(boss, t.len() - 1, "BOSS 非末格");
}

/// `index_of`：赛道上坐标 = 命中；赛道外坐标 = `None`。
#[test]
fn index_of_hits_on_track_and_none_off() {
    let t = track();
    let sp = t.space(3).expect("格 3 存在");
    assert_eq!(t.index_of(sp.cell), Some(3), "赛道坐标 = 格 3");
    // 构造赛道外的坐标（越界行）= None。
    let off = ti_core::domain::map::GridPos { row: sp.cell.row + 99, col: sp.cell.col };
    assert_eq!(t.index_of(off), None, "赛道外坐标 = None");
}

/// `BoardSession` 位置不变量：`position_index` 与 `index` 一致；初始 = `start_index`。
#[test]
fn board_session_position_tracks_index() {
    let t = track();
    let mut s = BoardSession::new(t.clone(), t.start_index());
    assert_eq!(s.position_index(), 0, "初始位置 = 起点格 0");
    // 前进 3 格 ⇒ position_index = 3。
    let passed = s.advance(3);
    assert_eq!(passed.len(), 3, "前进 3 格 = 3 个经过空间");
    assert_eq!(s.position_index(), 3, "前进后位置 = 3");
}

/// `last_roll` / `spin`：摇骰步数 ∈ `1..=sides`；`sides = 0` 视作 1（不 panic）。
#[test]
fn spin_roll_in_range_and_zero_sides_safe() {
    let t = track();
    let mut s = BoardSession::new(t.clone(), 0);
    let mut rng = ti_core::rules::rng::GameRng::new(42);
    for _ in 0..20 {
        let roll = s.spin(&mut rng, 6);
        assert!((1..=6).contains(&roll), "摇骰步数 ∈ 1..=6");
        assert_eq!(s.last_roll(), roll, "last_roll = 本回合步数");
    }
    // sides = 0 视作 1（`sides.max(1)`）：不 panic，步数 = 1。
    let r0 = s.spin(&mut rng, 0);
    assert_eq!(r0, 1, "sides = 0 视作 1（步数 = 1）");
}

/// `landing`：当前格类型 → 落格结算类型（`Landing::of`）。
#[test]
fn landing_reflects_current_space_kind() {
    let t = track();
    let mut s = BoardSession::new(t.clone(), 0);
    assert_eq!(s.landing(), Landing::Start, "起点格落格 = Start");
    // 前进到第 1 格（m01 格 1 = territory 领地格）。
    s.advance(1);
    assert_eq!(s.landing(), Landing::Territory, "m01 格 1 = territory（落格结算类型）");
}
