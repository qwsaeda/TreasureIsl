//! 地图域类型（业务文档：`docs/business/progression.md`）。
//!
//! 网格坐标 / 方向 / 地形可通行表 / 据点事件目标。
//! **等距投影公式**（渲染与触摸命中共用，`docs/business/progression.md` §地图结构 + `docs/parity.md` P0-5）：
//! ```text
//! iso_x = 32 × (row + col)
//! iso_y = 16 × (row + grid_w - 1 - col)
//! ```
//! 玩法层只经 [`GridPos::iso_pos`] / [`MapTerrain::iso_pos`] 取投影坐标（S12.3 收口：
//! 公式单一事实源在域模型，渲染/触摸层不再各自硬编码系数）。

/// 等距投影横向系数（`progression.md` §地图结构 内联公式，`iso_x = 32 × (row + col)`）。
pub const ISO_X_STEP: i32 = 32;
/// 等距投影纵向系数（`iso_y = 16 × (row + grid_w - 1 - col)`）。
pub const ISO_Y_STEP: i32 = 16;

/// 网格坐标（行/列）。
///
/// 业务语义（`docs/business/progression.md` §网格）：
/// - `row` 增大 = 向南（iso 屏幕下移）；
/// - `col` 增大 = 向东（iso 屏幕右移）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    /// 行坐标（`0..grid_h`）。
    pub row: u32,
    /// 列坐标（`0..grid_w`）。
    pub col: u32,
}

impl GridPos {
    /// 新建坐标。
    #[must_use]
    pub const fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }

    /// 单元格线性索引（`row × grid_w + col`）。
    #[must_use]
    pub const fn index(self, grid_w: u32) -> u32 {
        self.row * grid_w + self.col
    }

    /// 等距投影坐标（`iso_x = 32 × (row + col)`，`iso_y = 16 × (row + grid_w - 1 - col)`，
    /// `progression.md` §地图结构；渲染/触摸命中/摄像机共用的单一事实源）。
    ///
    /// `grid_w` 为所在地图网格宽（`iso_y` 依赖）；`u32` 中间量转 `i64` 防溢出，
    /// 结果按 `i32` 饱和截断（正常地图尺寸 `i32` 远够用，越界尺寸输入不报错）。
    #[must_use]
    pub fn iso_pos(self, grid_w: u32) -> (i32, i32) {
        let iso_x = i64::from(ISO_X_STEP) * i64::from(self.row.saturating_add(self.col));
        let inner = i64::from(self.row)
            .saturating_add(i64::from(grid_w).saturating_sub(1))
            .saturating_sub(i64::from(self.col));
        let iso_y = i64::from(ISO_Y_STEP) * inner;
        (i32::try_from(iso_x).unwrap_or(i32::MAX), i32::try_from(iso_y).unwrap_or(i32::MAX))
    }
}

/// 移动方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// 北（`row-1`）。
    North,
    /// 西（`col-1`）。
    West,
    /// 南（`row+1`）。
    South,
    /// 东（`col+1`）。
    East,
}

impl Dir {
    /// 全部方向（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [Dir; 4] {
        [Self::North, Self::West, Self::South, Self::East]
    }

    /// 方向码（`0..=3`，存档/事件用）。
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Self::North => 0,
            Self::West => 1,
            Self::South => 2,
            Self::East => 3,
        }
    }

    /// 方向码 → 枚举（值域外 `None`）。
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

    /// 行/列增量（`North=(-1,0)` / `West=(0,-1)` / `South=(1,0)` / `East=(0,1)`）。
    #[must_use]
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (-1, 0),
            Self::West => (0, -1),
            Self::South => (1, 0),
            Self::East => (0, 1),
        }
    }

    /// 屏幕方位（罗盘，用于 HUD 箭头）。
    #[must_use]
    pub const fn compass(self) -> Compass {
        match self {
            Self::North => Compass::Up,
            Self::West => Compass::Left,
            Self::South => Compass::Down,
            Self::East => Compass::Right,
        }
    }
}

/// 屏幕罗盘方位（HUD 箭头用；与 [`Dir`] 一一对应）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compass {
    /// 上。
    Up,
    /// 左。
    Left,
    /// 下。
    Down,
    /// 右。
    Right,
}

/// 地图地形（网格尺寸 + 可通行表）。
///
/// 业务语义（`docs/business/progression.md` §移动规则）：
/// - 越界坐标一律视为**不可通行**（`is_blocked` 返回 `true`）；
/// - 网格尺寸来自 `balance.selection.maps[i].grid_w/grid_h`（S10.3 改动 2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTerrain {
    /// 网格宽。
    pub grid_w: u32,
    /// 网格高。
    pub grid_h: u32,
    /// 不可通行表（`grid_w × grid_h`，`true` = 不可通行）。
    pub blocked: Vec<bool>,
}

impl MapTerrain {
    /// 新建地形（全可通行）。
    #[must_use]
    pub fn open(grid_w: u32, grid_h: u32) -> Self {
        Self { grid_w, grid_h, blocked: vec![false; grid_w.saturating_mul(grid_h) as usize] }
    }

    /// 该格是否不可通行（越界 = 不可通行）。
    #[must_use]
    pub fn is_blocked(&self, pos: GridPos) -> bool {
        if pos.row >= self.grid_h || pos.col >= self.grid_w {
            return true;
        }
        self.blocked[pos.index(self.grid_w) as usize]
    }

    /// 该格的等距投影坐标（委托 [`GridPos::iso_pos`]，`grid_w` 用本地形网格宽）。
    #[must_use]
    pub fn iso_pos(&self, pos: GridPos) -> (i32, i32) {
        pos.iso_pos(self.grid_w)
    }
}

/// 据点事件触发后的目标场景。
///
/// 业务语义（`docs/business/progression.md` §据点事件）：玩家踏入据点后
/// 按事件码分派到战斗 / 对话框 / 面板 / 卡片等场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneTarget {
    /// 据点对话框 / 资源结算面板。
    Msgl,
    /// 面板 / 卡片 / 贸易结果。
    Msgm,
    /// 通用消息提示 / 退出对话框。
    Msgs,
    /// 卡片事件（卡牌准备，复用 `msgm` 会话）。
    Card,
    /// 大卡片事件（复用 `msgm` 会话）。
    Bigcard,
    /// 战斗。
    Vs,
}
