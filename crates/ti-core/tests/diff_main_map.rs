//! S3.5 主地图（glmapm0x/移动/据点/事件分派）差分测试。
//!
//! 遍历小地形 × 4 方向 × 边界/阻挡组合 × 据点 × 事件码 1..31，
//! oracle 与 impl 逐位一致（移动结果/方向选择/走 N 步/据点判定/事件分派全等）；
//! 另加 2 个语义测试（随机 walk 1 步语义 + 据点命中）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::oracle::main_map::{
    Dir, GridPos, MapTerrain, oracle_at_landmark, oracle_dispatch_event, oracle_move,
    oracle_pick_dir, oracle_walk,
};
use ti_core::rules::main_map::MapSession;

/// 断言 oracle 与 impl 的 move 结果逐位一致。
fn assert_move_equal(terrain: &MapTerrain, pos: GridPos, d: Dir) {
    let o = oracle_move(terrain, pos, d);
    let mut s = MapSession::new(terrain.clone(), pos, &[]);
    let i = s.move_to(d);
    assert_eq!(o, i, "oracle_move vs MapSession::move_to ({pos:?} {d:?}) 不一致");
}

/// 断言 oracle 与 impl 的 `pick_dir` 结果逐位一致。
fn assert_pick_dir_equal(terrain: &MapTerrain, pos: GridPos) {
    let o = oracle_pick_dir(terrain, pos);
    let s = MapSession::new(terrain.clone(), pos, &[]);
    let i = s.choose_direction();
    assert_eq!(o, i, "oracle_pick_dir vs choose_direction ({pos:?}) 不一致");
}

/// 断言 oracle 与 impl 的 walk 结果逐位一致。
fn assert_walk_equal(terrain: &MapTerrain, pos: GridPos, d: Dir, steps: u32) {
    let o = oracle_walk(terrain, pos, d, steps);
    let mut s = MapSession::new(terrain.clone(), pos, &[]);
    let i = s.walk(d, steps);
    assert_eq!(o, i, "oracle_walk vs walk ({pos:?} {d:?} x{steps}) 不一致");
}

/// 断言 oracle 与 impl 的 `at_landmark` 结果逐位一致。
fn assert_landmark_equal(landmarks: &[(u32, u32)], pos: GridPos) {
    let o = oracle_at_landmark(landmarks, pos);
    let s = MapSession::new(MapTerrain::open(4, 4), pos, landmarks);
    let i = s.at_landmark();
    assert_eq!(o, i, "oracle_at_landmark vs at_landmark ({pos:?}) 不一致");
}

/// 断言 oracle 与 impl 的 dispatch 结果逐位一致。
fn assert_dispatch_equal(code: u32) {
    let o = oracle_dispatch_event(code);
    let s = MapSession::new(MapTerrain::open(4, 4), GridPos::new(0, 0), &[]);
    let i = s.dispatch(code);
    assert_eq!(o, i, "oracle_dispatch_event vs dispatch (码 {code}) 不一致");
}

/// 遍历 4×4 全地形 × 每格 × 4 方向：`oracle_move` vs `MapSession::move_to`。
#[test]
fn diff_move_full_grid_4x4() {
    let t = MapTerrain::open(4, 4);
    for row in 0..4u32 {
        for col in 0..4u32 {
            let pos = GridPos::new(row, col);
            for d in Dir::all() {
                assert_move_equal(&t, pos, d);
            }
        }
    }
}

/// 遍历 4×4 全地形 × 每格：`oracle_pick_dir` vs `choose_direction`。
#[test]
fn diff_pick_dir_full_grid_4x4() {
    let t = MapTerrain::open(4, 4);
    for row in 0..4u32 {
        for col in 0..4u32 {
            assert_pick_dir_equal(&t, GridPos::new(row, col));
        }
    }
}

/// 遍历 4×4 全地形 × 每格 × 4 方向 × steps `0..4`：`oracle_walk` vs `walk`。
#[test]
fn diff_walk_full_grid_4x4() {
    let t = MapTerrain::open(4, 4);
    for row in 0..4u32 {
        for col in 0..4u32 {
            for d in Dir::all() {
                for steps in 0..4u32 {
                    assert_walk_equal(&t, GridPos::new(row, col), d, steps);
                }
            }
        }
    }
}

/// 遍历阻挡格组合 × 4 方向 × 边界：`oracle_move` vs `move_to`。
#[test]
fn diff_move_blocked_and_bounds() {
    // 构造含阻挡格的地形：阻挡 (1,1)(1,2)(2,1)
    let mut t = MapTerrain::open(4, 4);
    t.blocked[GridPos::new(1, 1).index(4) as usize] = true;
    t.blocked[GridPos::new(1, 2).index(4) as usize] = true;
    t.blocked[GridPos::new(2, 1).index(4) as usize] = true;

    for row in 0..4u32 {
        for col in 0..4u32 {
            let pos = GridPos::new(row, col);
            for d in Dir::all() {
                assert_move_equal(&t, pos, d);
                assert_pick_dir_equal(&t, pos);
            }
            for steps in 0..4u32 {
                for d in Dir::all() {
                    assert_walk_equal(&t, pos, d, steps);
                }
            }
        }
    }
}

/// 全阻挡格：所有移动/方向选择应返回 None/不变。
#[test]
fn diff_move_all_blocked() {
    let mut t = MapTerrain::open(4, 4);
    for i in 0..16 {
        t.blocked[i] = true;
    }
    let pos = GridPos::new(2, 2);
    assert_eq!(oracle_move(&t, pos, Dir::North), None);
    assert_eq!(oracle_pick_dir(&t, pos), None);
    let mut s = MapSession::new(t.clone(), pos, &[]);
    assert_eq!(s.move_to(Dir::North), None);
    assert_eq!(s.choose_direction(), None);
    assert_eq!(s.pos(), pos, "全阻挡移动后位置应不变");
}

/// 遍历事件码 `1..31` + 边界码 `0`：`oracle_dispatch` vs `dispatch`。
#[test]
fn diff_dispatch_all_event_codes() {
    for code in 0..=31u32 {
        assert_dispatch_equal(code);
    }
}

/// 遍历据点坐标命中/未命中：`oracle_at_landmark` vs `at_landmark`。
#[test]
fn diff_landmark_hit_and_miss() {
    let landmarks = [(0u32, 0u32), (1u32, 3u32), (3u32, 2u32)];
    for row in 0..4u32 {
        for col in 0..4u32 {
            assert_landmark_equal(&landmarks, GridPos::new(row, col));
        }
    }
}

/// 语义测试 (a)：4×4 全开放，中心格 (2,2) 南→东→北→西 优先级。
#[test]
fn semantic_pick_dir_priority_order() {
    let t = MapTerrain::open(4, 4);
    let center = GridPos::new(2, 2);
    // 全开放 → 优先 South（row+1）
    assert_eq!(oracle_pick_dir(&t, center), Some(Dir::South));

    // 阻挡 South → East
    let mut t2 = t.clone();
    t2.blocked[GridPos::new(3, 2).index(4) as usize] = true;
    assert_eq!(oracle_pick_dir(&t2, center), Some(Dir::East));

    // 阻挡 South+East → North
    t2.blocked[GridPos::new(2, 3).index(4) as usize] = true;
    assert_eq!(oracle_pick_dir(&t2, center), Some(Dir::North));

    // 阻挡 South+East+North → West
    t2.blocked[GridPos::new(1, 2).index(4) as usize] = true;
    assert_eq!(oracle_pick_dir(&t2, center), Some(Dir::West));

    // 阻挡全部 4 方向 → None
    t2.blocked[GridPos::new(2, 1).index(4) as usize] = true;
    assert_eq!(oracle_pick_dir(&t2, center), None);
}

/// 语义测试 (b)：随机 walk 1 步语义（确定性分支，`steps=1` 走 1 格）。
#[test]
fn semantic_walk_one_step() {
    let t = MapTerrain::open(4, 4);
    let start = GridPos::new(1, 1);
    // 走 1 步 South → (2,1)
    let (end, walked) = oracle_walk(&t, start, Dir::South, 1);
    assert_eq!(end, GridPos::new(2, 1));
    assert_eq!(walked, 1);

    // impl 一致
    let mut s = MapSession::new(t, start, &[]);
    let (i_end, i_walked) = s.walk(Dir::South, 1);
    assert_eq!((end, walked), (i_end, i_walked));
    assert_eq!(s.pos(), GridPos::new(2, 1));
}
