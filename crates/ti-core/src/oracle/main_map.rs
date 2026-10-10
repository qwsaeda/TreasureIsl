//! oracle：S3.5 主地图（glmapm0x/移动/据点/事件分派）差分基准，测试专用。
//!
//! 直译 `docs/spec/main_map.md` 的移动/据点/事件规则：
//! - 网格坐标 [`GridPos`]（`row × grid_w + col`，与 S1.8b f8 公式一致）；
//! - 方向枚举 [`Dir`]（0=北 / 1=西 / 2=南 / 3=东，`探索格` 写 `a1+124` 值域）；
//! - 地形 [`MapTerrain`]（`grid_w`/`grid_h` + 可通行表 `blocked`，越界=不可通行，`二维网格取值` 等价）；
//! - [`oracle_move`]：单步移动（`cards` 每步语义，越界/阻挡 → `None`）；
//! - [`oracle_pick_dir`]：`探索格` 方向可达性（优先级 南→东→北→西，仅 `+22&3==1` 格）；
//! - [`oracle_walk`]：走 N 步（给定方向位，每步同方向，`cards` 确定性分支）；
//! - [`oracle_dispatch_event`]：事件码 → 场景目标（spec §d，1..8 有主分派 case，其余 `None`）；
//! - [`oracle_at_landmark`]：据点/地标判定（坐标在列表内 → `true`）。
//!
//! 随机分支（`rand() % 4`，`cards` 行 77-94）不在此 oracle 建模，
//! diff 测试以固定输入做 1 步语义断言（随机序列对齐归 S3.12）。

/// 网格坐标（行/列，`u32`，越界检查用）。
///
/// 与 S1.8b f8 公式 `cell = row × grid_w + col` 一致；
/// `row` 增大 = 南（iso 屏幕下移），`col` 增大 = 东（iso 屏幕右移，spec §b 推测）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    /// 行坐标（`0..grid_h`，增大 = 南）。
    pub row: u32,
    /// 列坐标（`0..grid_w`，增大 = 东）。
    pub col: u32,
}

impl GridPos {
    /// 新建坐标。
    #[must_use]
    pub const fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }

    /// 单元格线性索引（`row × grid_w + col`，S1.8b f8 低 16 位公式）。
    #[must_use]
    pub const fn index(self, grid_w: u32) -> u32 {
        self.row * grid_w + self.col
    }
}

/// 方向枚举（`探索格` 写 `a1+124` 的值域 0..=3，spec §b 方向枚举表）。
///
/// 命名依据等距投影（**推测**，渲染层校准）：
/// - `North`(0)：`row-1`（北，行 30-33 优先级 4/兜底）；
/// - `West`(1)：`col-1`（西，行 39-42 优先级 3）；
/// - `South`(2)：`row+1`（南，行 55-58 优先级 1）；
/// - `East`(3)：`col+1`（东，行 47-50 优先级 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// 北（`row-1`，方向码 0）。
    North,
    /// 西（`col-1`，方向码 1）。
    West,
    /// 南（`row+1`，方向码 2）。
    South,
    /// 东（`col+1`，方向码 3）。
    East,
}

impl Dir {
    /// 全部方向（按枚举声明顺序，diff 测试遍历用）。
    #[must_use]
    pub const fn all() -> [Dir; 4] {
        [Self::North, Self::West, Self::South, Self::East]
    }

    /// 方向码（`a1+124` 值，0..=3）。
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Self::North => 0,
            Self::West => 1,
            Self::South => 2,
            Self::East => 3,
        }
    }

    /// 方向码 → 枚举（值域外返回 `None`）。
    #[must_use]
    pub fn from_code(code: u32) -> Option<Dir> {
        match code {
            0 => Some(Self::North),
            1 => Some(Self::West),
            2 => Some(Self::South),
            3 => Some(Self::East),
            _ => None,
        }
    }

    /// 行/列增量（`cards` switch 语义：南=row+1/东=col+1/北=row-1/西=col-1）。
    #[must_use]
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (-1, 0),
            Self::West => (0, -1),
            Self::South => (1, 0),
            Self::East => (0, 1),
        }
    }
}

/// 地形（`grid_w`/`grid_h` + 可通行表，`二维网格取值` 网格取值 oracle 化）。
///
/// - `grid_w`/`grid_h`：m01/m02=20×20，m03=15×15（S1.8b）；
/// - `blocked`：长度 `grid_w × grid_h` 的二维可通行表（`blocked[i]` = 该格不可通行）；
///   越界坐标一律视为不可通行（`二维网格取值` 越界返回 0 → `0+22` 越界读被 `&3==1` 挡住，行为等价）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTerrain {
    /// 网格宽（`*(v6+692)`，m01/m02=20，m03=15）。
    pub grid_w: u32,
    /// 网格高（m01/m02=20，m03=15）。
    pub grid_h: u32,
    /// 不可通行表（`grid_w × grid_h`，`true` = 不可通行）。
    pub blocked: Vec<bool>,
}

impl MapTerrain {
    /// 新建地形（全可通行，`blocked` 全 `false`）。
    #[must_use]
    pub fn open(grid_w: u32, grid_h: u32) -> Self {
        Self { grid_w, grid_h, blocked: vec![false; grid_w.saturating_mul(grid_h) as usize] }
    }

    /// 该格是否不可通行（越界 = 不可通行，`二维网格取值` 等价）。
    #[must_use]
    pub fn is_blocked(&self, pos: GridPos) -> bool {
        if pos.row >= self.grid_h || pos.col >= self.grid_w {
            return true;
        }
        self.blocked[pos.index(self.grid_w) as usize]
    }
}

/// 事件分派的目标场景（spec §d，`ui.pak` 场景名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneTarget {
    /// `msgl.Map`（据点/对话框，事件码 1/4）。
    Msgl,
    /// `msgm.Map`（面板/卡片/消息，事件码 2/5/7/8）。
    Msgm,
    /// `msgs.Map`（事件码 8 虚调用 `+80`==0 分支，spec §d 行 2213）。
    Msgs,
    /// `card` 场景（卡牌准备，事件码 3 的 `cards` 目标，**推测**场景标识）。
    Card,
    /// `bigcard` 场景（大卡，**推测**）。
    Bigcard,
    /// `vs.Map`（战斗，事件码 6，`a1+716=25` 战斗态）。
    Vs,
}

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
