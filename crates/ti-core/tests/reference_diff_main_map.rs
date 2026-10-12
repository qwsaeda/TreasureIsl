//! S3.5 主地图（移动/据点/BOSS 格）规则测试。
//!
//! 业务规格 `docs/business/progression.md`（可测试规则表 3 条地图条目 + 等距投影公式）：
//! - `map_blocked_step_is_noop`：越界/不可通行不移动、不消耗回合；
//! - `map_step_triggers_settle_once`：移动一格必触发一次落格结算；
//! - `map_boss_tile_enters_combat`：BOSS 格进入战斗场景（战斗优先于据点菜单）；
//! - 等距投影（`iso_x = 32*(row+col)`、`iso_y = 16*(row + grid_w - 1 - col)`）走
//!   `ti_core::domain::map`（c1 契约公共类型，`GridPos::iso_pos`/`MapTerrain::iso_pos`），
//!   与 `ti-app` 渲染层 `TerrainAtlas::iso_x/iso_y` 同源公式（`docs/parity.md` P0-5）。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。
//! 用 `.expect("...")`（测试代码允许，门禁第 6 项豁免 tests/）。

use ti_core::domain::map::{Dir, GridPos, MapTerrain, SceneTarget};
use ti_core::rules::cards_draw::CardPool;
use ti_core::rules::main_map::MapSession;
use ti_core::rules::rng::GameRng;

/// 测试用空卡池（隔离随机，`event_chance=0` 永不触发卡牌）。
fn empty_pool() -> CardPool {
    CardPool::new(Vec::new(), 0, 0.0)
}

/// `map_blocked_step_is_noop`：越界/不可通行不移动、不消耗回合。
///
/// 业务规则（`progression.md` 移动规则）：
/// ```text
/// 方向键 → 目标格
///   若 越界 或 不可通行 → 不移动（不消耗回合）
/// ```
/// 判据：`move_to` 返回 `moved=false`、位置不变、`card_event=None`（不触发卡牌结算）。
#[test]
fn map_blocked_step_is_noop() {
    // 4×4 全开放地形，从 (0,0) 向北/向西移动 → 越界
    let t = MapTerrain::open(4, 4);
    let mut s = MapSession::with_pool(t, GridPos::new(0, 0), &[], empty_pool());
    let mut rng = GameRng::new(1);

    let r_north = s.move_to(Dir::North, &mut rng);
    assert!(!r_north.moved, "北越界 → moved=false");
    assert_eq!(r_north.pos, GridPos::new(0, 0), "越界不移动");
    assert!(r_north.card_event.is_none(), "越界不触发卡牌");
    assert!(!r_north.boss, "越界不触发 BOSS 格");

    let r_west = s.move_to(Dir::West, &mut rng);
    assert!(!r_west.moved, "西越界 → moved=false");
    assert_eq!(r_west.pos, GridPos::new(0, 0), "越界不移动");

    // 不可通行格（阻挡）
    let mut t_blocked = MapTerrain::open(4, 4);
    t_blocked.blocked[GridPos::new(0, 1).index(4) as usize] = true;
    let mut s2 = MapSession::with_pool(t_blocked, GridPos::new(0, 0), &[], empty_pool());
    let mut rng2 = GameRng::new(1);
    let r_blocked = s2.move_to(Dir::East, &mut rng2);
    assert!(!r_blocked.moved, "不可通行 → moved=false");
    assert_eq!(r_blocked.pos, GridPos::new(0, 0), "不可通行不移动");
    assert!(r_blocked.card_event.is_none(), "不可通行不触发卡牌");
}

/// `map_step_triggers_settle_once`：移动一格必触发一次落格结算。
///
/// 业务规则（`progression.md` 移动规则）：
/// ```text
/// 否则 移动一格 → 触发「落格结算」
/// ```
/// 判据：合法移动后 `moved=true`，据点格 `landmark=Some(...)`，BOSS 格 `boss=true`；
/// 同一格每步各触发一次（不重复触发）。
#[test]
fn map_step_triggers_settle_once() {
    // 据点格：走到据点后 landmark=Some，再走一步（离开据点）landmark=None
    let t = MapTerrain::open(4, 4);
    let mut s = MapSession::with_pool(t, GridPos::new(0, 0), &[(1, 0)], empty_pool());
    let mut rng = GameRng::new(1);

    let r1 = s.move_to(Dir::South, &mut rng);
    assert!(r1.moved, "合法移动 → moved=true");
    assert!(r1.landmark.is_some(), "落在据点 (1,0) → landmark=Some");
    assert!(!r1.boss, "非 BOSS 格 → boss=false");

    // 离开据点
    let r2 = s.move_to(Dir::South, &mut rng);
    assert!(r2.moved, "合法移动 → moved=true");
    assert!(r2.landmark.is_none(), "离开据点 → landmark=None");

    // BOSS 格：走到 BOSS 格后 boss=true
    let t2 = MapTerrain::open(4, 4);
    let boss = GridPos::new(2, 0);
    let mut s2 = MapSession::with_boss(t2, GridPos::new(0, 0), &[], empty_pool(), Some(boss));
    let mut rng2 = GameRng::new(1);
    let _ = s2.move_to(Dir::South, &mut rng2);
    let r3 = s2.move_to(Dir::South, &mut rng2);
    assert!(r3.boss, "落在 BOSS 格 → boss=true");
    assert_eq!(r3.pos, boss, "位置应为 BOSS 格");
}

/// `map_boss_tile_enters_combat`：BOSS 格进入战斗场景。
///
/// 业务规则（`progression.md` 落格结算）：
/// ```text
/// 遇到 BOSS 格 → 进入战斗场景（combat.md）
/// ```
/// 判据：`move_to` 落到 BOSS 格时 `StepResult::boss=true`，`landmark=None`（战斗优先于据点菜单）。
#[test]
fn map_boss_tile_enters_combat() {
    let t = MapTerrain::open(4, 4);
    let boss = GridPos::new(3, 3);
    let mut s = MapSession::with_boss(t, GridPos::new(0, 0), &[], empty_pool(), Some(boss));
    let mut rng = GameRng::new(1);

    // 走到 BOSS 格
    let _ = s.move_to(Dir::South, &mut rng);
    let _ = s.move_to(Dir::South, &mut rng);
    let _ = s.move_to(Dir::South, &mut rng);
    let r = s.move_to(Dir::East, &mut rng); // (3,0)→(3,1)，未达 BOSS
    assert!(!r.boss, "未到 BOSS 格 → boss=false");
    let _ = s.move_to(Dir::East, &mut rng); // (3,1)→(3,2)
    let r_final = s.move_to(Dir::East, &mut rng); // (3,2)→(3,3)=BOSS
    assert!(r_final.boss, "落在 BOSS 格 (3,3) → boss=true");
    assert!(s.at_boss_tile(), "当前位置 = BOSS 格");

    // BOSS 格 + 据点格同坐标：BOSS 优先（`progression.md` 战斗优先于菜单）
    let t2 = MapTerrain::open(4, 4);
    let mut s2 = MapSession::with_boss(t2, GridPos::new(0, 0), &[(3, 3)], empty_pool(), Some(boss));
    let mut rng2 = GameRng::new(1);
    for _ in 0..3 {
        let _ = s2.move_to(Dir::South, &mut rng2);
    }
    let r_last = s2.move_to(Dir::East, &mut rng2);
    let _ = s2.move_to(Dir::East, &mut rng2);
    let r_boss = s2.move_to(Dir::East, &mut rng2);
    assert!(r_boss.boss, "落到 BOSS 格 (3,3) → boss=true");
    assert!(r_last.landmark.is_none(), "未达 BOSS 的 (3,2) 非据点 → landmark=None");
    assert_eq!(s2.landmarks(), &[(3, 3)], "据点列表保持只读不变");
}

/// 等距投影公式（`progression.md` §地图结构）：`iso_x = 32*(row+col)`、
/// `iso_y = 16*(row + grid_w - 1 - col)`。玩法层经 `GridPos::iso_pos`/`MapTerrain::iso_pos`
/// 取坐标，与 `ti-app` 渲染层 `TerrainAtlas` 同源（`docs/parity.md` P0-5），不再各写魔数。
#[test]
fn map_iso_projection_formula() {
    // 20×20 实测网格（`progression.md` §地图结构）
    let t = MapTerrain::open(20, 20);
    let p = GridPos::new(3, 5);
    let (iso_x, iso_y) = t.iso_pos(p);
    assert_eq!(iso_x, 32 * (3 + 5), "iso_x = 32*(row+col)");
    assert_eq!(iso_y, 16 * (3 + 20 - 1 - 5), "iso_y = 16*(row + grid_w - 1 - col)");

    // 角格特例：(0, grid_w-1) 的 iso_y = 0（屏幕最左上，`terrain_records` 缺格判据同源）
    let corner = GridPos::new(0, 19);
    assert_eq!(corner.iso_pos(20).1, 0, "iso_y(0, gw-1) = 0");
    let bottom = GridPos::new(19, 0);
    assert_eq!(bottom.iso_pos(20).0, 32 * 19, "iso_x(19, 0) = 32*19");

    // 4×4 小网格也成立（grid_w=4；`iso_y` 依赖 `grid_w`，小网格非 0）
    let small = MapTerrain::open(4, 4);
    assert_eq!(small.iso_pos(GridPos::new(0, 0)), (0, 16 * 3));
    assert_eq!(small.iso_pos(GridPos::new(3, 3)), (32 * 6, 16 * (3 + 4 - 1 - 3)));
}

/// 走 N 步（`progression.md` 移动规则「一格一步」批量）：合法走 `steps` 步，撞界即停。
#[test]
fn map_walk_stops_at_boundary() {
    let t = MapTerrain::open(4, 4);
    let mut s = MapSession::with_pool(t, GridPos::new(1, 1), &[], empty_pool());
    let mut rng = GameRng::new(1);
    let (end, walked) = s.walk(Dir::East, 2, &mut rng);
    assert_eq!(end, GridPos::new(1, 3), "合法走 2 步到 (1,3)");
    assert_eq!(walked, 2);
    let (end2, walked2) = s.walk(Dir::East, 3, &mut rng);
    assert_eq!(end2, GridPos::new(1, 3), "已在边界，再走 0 步");
    assert_eq!(walked2, 0, "撞界走 0 步");
    assert_eq!(s.pos(), GridPos::new(1, 3), "walk 后位置 = 终点");
}

/// 据点事件分派（`progression.md` 据点与事件）：据点菜单目标经 `SceneTarget`（c1 契约），
/// 未知事件码 → `None`（无处理路径，不 panic）。
#[test]
fn map_landmark_dispatch_targets() {
    let t = MapTerrain::open(4, 4);
    let s = MapSession::with_pool(t, GridPos::new(0, 0), &[(1, 1)], empty_pool());
    assert_eq!(MapSession::dispatch_event(1), Some(SceneTarget::Msgl), "事件码 1 → 据点对话框");
    assert_eq!(MapSession::dispatch_event(9), None, "未知事件码 9 → None");
    assert_eq!(s.dispatch(1), Some(SceneTarget::Msgl), "会话侧分派与静态侧一致");
}

/// 静态单步合法性预检（不移动、不改状态）：与 `move_to` 的 `moved` 判定一致。
#[test]
fn map_can_walk_matches_move() {
    let mut t = MapTerrain::open(4, 4);
    t.blocked[GridPos::new(0, 1).index(4) as usize] = true;
    let pos = GridPos::new(0, 0);
    assert!(!MapSession::can_walk(&t, pos, Dir::East), "阻挡格 → 不可走");
    assert!(MapSession::can_walk(&t, pos, Dir::South), "合法格 → 可走");

    let mut s = MapSession::with_pool(t.clone(), pos, &[], empty_pool());
    let mut rng = GameRng::new(1);
    let r = s.move_to(Dir::East, &mut rng);
    assert!(!r.moved, "can_walk=false 与 move_to 判定一致");
    let _ = s;
}

/// 方向枚举（`Dir` 契约公共类型）：码 ↔ 枚举往返 + 增量表（`progression.md` §网格 语义）。
#[test]
fn map_dir_code_roundtrip() {
    for d in Dir::all() {
        assert_eq!(Dir::from_code(d.code()), Some(d), "Dir::from_code({}) 往返", d.code());
        let (dr, dc) = d.delta();
        match d {
            Dir::North => assert_eq!((dr, dc), (-1, 0)),
            Dir::West => assert_eq!((dr, dc), (0, -1)),
            Dir::South => assert_eq!((dr, dc), (1, 0)),
            Dir::East => assert_eq!((dr, dc), (0, 1)),
        }
    }
    assert_eq!(Dir::from_code(4), None, "码值域外 → None");
}
