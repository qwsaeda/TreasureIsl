//! oracle 主地图差分基准（S11 降级为纯基准函数；类型在 domain::map）。
//!
//! 差分基准 = 从 `legacy/docs/spec/*.md` 伪代码直译的纯函数，供 `tests/diff_*.rs` 对账
//! 正向实现（`rules::`）；本模块**不参与运行时**，S11 起物理归档 `legacy/` 前保留为对照基准。


pub use crate::domain::map::{Dir, GridPos, MapTerrain, SceneTarget};

/// oracle：单步移动（`cards` 每步语义）。
///
/// 规则（spec §b 单步移动）：
/// ```text
/// Δ(row,col) = d 的 delta（North=(-1,0)/West=(0,-1)/South=(1,0)/East=(0,1)）
/// new = (row+Δr, col+Δc)
/// 合法 = 0<=new_row<grid_h && 0<=new_col<grid_w && !blocked[new]
/// 合法 → Some(new)，否则 None
/// ```
/// 负增量用 `i64` 中间计算避免 `u32` 下溢。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn oracle_move(terrain: &MapTerrain, pos: GridPos, d: Dir) -> Option<GridPos> {
    let (dr, dc) = d.delta();
    let nr = i64::from(pos.row) + i64::from(dr);
    let nc = i64::from(pos.col) + i64::from(dc);
    if nr < 0 || nc < 0 {
        return None;
    }
    let nrow = nr as u32;
    let ncol = nc as u32;
    if nrow >= terrain.grid_h || ncol >= terrain.grid_w {
        return None;
    }
    if terrain.blocked[(nrow * terrain.grid_w + ncol) as usize] {
        return None;
    }
    Some(GridPos::new(nrow, ncol))
}

/// oracle：`探索格` 方向可达性（仅可通行格 `+22&3==1` 才有方向，否则 `None`）。
///
/// 规则（spec §b 方向枚举表，探测优先级 南→东→北→西）：
/// - 当前格不可通行 → `None`（入口 `+22&3==1` 不满足）；
/// - 优先尝试 `South`(`row+1`)、`East`(`col+1`)、`North`(`row-1`)、`West`(`col-1`)；
/// - 取第一个可通行的方向；全不可通行 → `None`。
///
/// 方向位掩码（`a1+129/130`，8/4/2/1）不在此 oracle 建模（位运算归 `oracle_walk` 语义）。
#[must_use]
pub fn oracle_pick_dir(terrain: &MapTerrain, pos: GridPos) -> Option<Dir> {
    if terrain.is_blocked(pos) {
        return None;
    }
    [Dir::South, Dir::East, Dir::North, Dir::West]
        .into_iter()
        .find(|&d| oracle_move(terrain, pos, d).is_some())
}

/// oracle：走 N 步（`cards` 确定性分支，给定方向每步同方向）。
///
/// 规则：
/// - 每步按 `Dir` 的 delta 走 1 格（`oracle_move` 语义）；
/// - 任一步不可通行 → 停在原地，返回 `(当前位置, 已走步数)`；
/// - `steps` 步全走成功 → 返回 `(终点, steps)`。
///
/// 随机分支（方向位全 0 → `rand()%4`，spec §b 行 77-94）不在此建模。
#[must_use]
pub fn oracle_walk(terrain: &MapTerrain, pos: GridPos, d: Dir, steps: u32) -> (GridPos, u32) {
    let mut cur = pos;
    let mut walked = 0u32;
    for _ in 0..steps {
        match oracle_move(terrain, cur, d) {
            Some(next) => {
                cur = next;
                walked += 1;
            }
            None => break,
        }
    }
    (cur, walked)
}

/// oracle：事件码分派（spec §d，`事件码生成器` 高 6 位+1，值域 1..31）。
///
/// 主分派 `主地图帧更新` 行 1709 `switch` 只有 case 1..8（行 1711-2281 实测）：
///
/// | 事件码 | 目标场景 | 证据行号 |
/// |---|---|---|
/// | `1` | `Msgl`（据点/地标） | 1711-1796 |
/// | `2` | `Msgm`（面板） | 1891-1961 |
/// | `3` | `None`（卡牌随机数，无场景） | 2044-2050 |
/// | `4` | `Msgl`（货物结果提示） | 2051-2122 |
/// | `5` | `Msgm`（卡片翻转，行 2135） | 2123-2136 |
/// | `6` | `Vs`（战斗） | 2137-2156 |
/// | `7` | `Msgm`（消息） | 2157-2196 |
/// | `8` | `Msgs`（虚调用 `+80`==0，行 2213） | 2197-2281 |
///
/// 事件码 9..31：主分派无 case（由 `场景管理器帧更新总控` 排队事件消费路径处理，S3.1 域），返回 `None`。
#[must_use]
pub fn oracle_dispatch_event(code: u32) -> Option<SceneTarget> {
    match code {
        1 | 4 => Some(SceneTarget::Msgl),
        2 | 5 | 7 => Some(SceneTarget::Msgm),
        6 => Some(SceneTarget::Vs),
        8 => Some(SceneTarget::Msgs),
        _ => None,
    }
}

/// oracle：据点/地标判定（坐标在据点列表内 → `true`）。
///
/// 直译 `trade` 的据点判定核心（行 145-160 遍历据点定义对象命中判定，
/// 概率/名称表 `word_7FB48` 语义在 oracle 层简化为纯坐标命中）。
#[must_use]
pub fn oracle_at_landmark(landmarks: &[(u32, u32)], pos: GridPos) -> bool {
    landmarks.iter().any(|&(r, c)| r == pos.row && c == pos.col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_within_bounds() {
        let t = MapTerrain::open(4, 4);
        let pos = GridPos::new(2, 2);
        assert_eq!(oracle_move(&t, pos, Dir::North), Some(GridPos::new(1, 2)));
        assert_eq!(oracle_move(&t, pos, Dir::South), Some(GridPos::new(3, 2)));
        assert_eq!(oracle_move(&t, pos, Dir::West), Some(GridPos::new(2, 1)));
        assert_eq!(oracle_move(&t, pos, Dir::East), Some(GridPos::new(2, 3)));
    }

    #[test]
    fn move_out_of_bounds_none() {
        let t = MapTerrain::open(4, 4);
        let corner = GridPos::new(0, 0);
        assert_eq!(oracle_move(&t, corner, Dir::North), None);
        assert_eq!(oracle_move(&t, corner, Dir::West), None);
        let far = GridPos::new(3, 3);
        assert_eq!(oracle_move(&t, far, Dir::South), None);
        assert_eq!(oracle_move(&t, far, Dir::East), None);
    }

    #[test]
    fn move_blocked_none() {
        let mut t = MapTerrain::open(4, 4);
        t.blocked[GridPos::new(2, 2).index(4) as usize] = true;
        assert_eq!(oracle_move(&t, GridPos::new(2, 1), Dir::East), None);
        // 当前格本身被阻挡不影响相邻格移动
        assert_eq!(oracle_move(&t, GridPos::new(2, 2), Dir::North), Some(GridPos::new(1, 2)));
    }

    #[test]
    fn pick_dir_priority() {
        let mut t = MapTerrain::open(4, 4);
        // 中心格 (2,2)，全开放 → 优先 South
        assert_eq!(oracle_pick_dir(&t, GridPos::new(2, 2)), Some(Dir::South));
        // 阻挡 South 后 → East
        t.blocked[GridPos::new(3, 2).index(4) as usize] = true;
        assert_eq!(oracle_pick_dir(&t, GridPos::new(2, 2)), Some(Dir::East));
        // 阻挡 South+East → North
        t.blocked[GridPos::new(2, 3).index(4) as usize] = true;
        assert_eq!(oracle_pick_dir(&t, GridPos::new(2, 2)), Some(Dir::North));
    }

    #[test]
    fn pick_dir_all_blocked_none() {
        let t = MapTerrain::open(4, 4);
        let corner = GridPos::new(0, 0);
        // 角格只有 East/South 可走，North/West 越界 → South
        assert_eq!(oracle_pick_dir(&t, corner), Some(Dir::South));
        // 全阻挡格 → None
        let mut t2 = MapTerrain::open(4, 4);
        for i in 0..16 {
            t2.blocked[i] = true;
        }
        assert_eq!(oracle_pick_dir(&t2, GridPos::new(2, 2)), None);
    }

    #[test]
    fn walk_multiple_steps() {
        let t = MapTerrain::open(4, 4);
        let start = GridPos::new(1, 1);
        assert_eq!(oracle_walk(&t, start, Dir::East, 2), (GridPos::new(1, 3), 2));
        // 撞墙（East 走 2 步到 (1,3)，再 East 越界）
        assert_eq!(oracle_walk(&t, start, Dir::East, 4), (GridPos::new(1, 3), 2));
    }

    #[test]
    fn walk_zero_steps() {
        let t = MapTerrain::open(4, 4);
        let start = GridPos::new(1, 1);
        assert_eq!(oracle_walk(&t, start, Dir::North, 0), (start, 0));
    }

    #[test]
    fn dispatch_event_codes() {
        assert_eq!(oracle_dispatch_event(1), Some(SceneTarget::Msgl));
        assert_eq!(oracle_dispatch_event(2), Some(SceneTarget::Msgm));
        assert_eq!(oracle_dispatch_event(3), None);
        assert_eq!(oracle_dispatch_event(4), Some(SceneTarget::Msgl));
        assert_eq!(oracle_dispatch_event(5), Some(SceneTarget::Msgm));
        assert_eq!(oracle_dispatch_event(6), Some(SceneTarget::Vs));
        assert_eq!(oracle_dispatch_event(7), Some(SceneTarget::Msgm));
        assert_eq!(oracle_dispatch_event(8), Some(SceneTarget::Msgs));
        // 9..31 无主分派 case
        assert_eq!(oracle_dispatch_event(9), None);
        assert_eq!(oracle_dispatch_event(31), None);
        assert_eq!(oracle_dispatch_event(0), None);
    }

    #[test]
    fn at_landmark() {
        let landmarks = [(1u32, 2u32), (3u32, 0u32)];
        assert!(oracle_at_landmark(&landmarks, GridPos::new(1, 2)));
        assert!(oracle_at_landmark(&landmarks, GridPos::new(3, 0)));
        assert!(!oracle_at_landmark(&landmarks, GridPos::new(0, 0)));
        assert!(!oracle_at_landmark(&[], GridPos::new(0, 0)));
    }

    #[test]
    fn dir_roundtrip() {
        for d in Dir::all() {
            assert_eq!(Dir::from_code(d.code()), Some(d));
        }
        assert_eq!(Dir::from_code(4), None);
    }
}
