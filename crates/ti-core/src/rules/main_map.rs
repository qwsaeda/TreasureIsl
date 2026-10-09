//! S3.5 主地图（glmapm0x/移动/据点/事件分派）正式实现。
//!
//! 与 `oracle::main_map` 行为逐位一致（同一输入产生逐位相同的
//! 移动结果/方向选择/事件分派/据点判定），但结构不同：用 `MapSession`
//! 结构体 + 方法承载状态，而非自由函数。
//!
//! 依据 `docs/spec/main_map.md`：
//! - 网格坐标 [`GridPos`]、方向 [`Dir`]、地形 [`MapTerrain`]、场景目标 [`SceneTarget`]
//!   （从 oracle 重导出）；
//! - [`MapSession`]：主地图会话状态（地形 + 当前位置 + 据点列表）；
//! - `move_to` / `choose_direction` / `walk` / `at_landmark` / `dispatch` 方法。

use crate::oracle::main_map::{
    Dir, GridPos, MapTerrain, SceneTarget, oracle_at_landmark, oracle_dispatch_event, oracle_move,
    oracle_pick_dir, oracle_walk,
};

/// 主地图会话：地形 + 当前位置 + 据点列表。
#[derive(Debug, Clone)]
pub struct MapSession {
    terrain: MapTerrain,
    pos: GridPos,
    landmarks: Vec<(u32, u32)>,
}

impl MapSession {
    /// 新建主地图会话（指定地形、初始位置、据点列表）。
    #[must_use]
    pub fn new(terrain: MapTerrain, pos: GridPos, landmarks: &[(u32, u32)]) -> Self {
        Self { terrain, pos, landmarks: landmarks.to_vec() }
    }

    /// 当前地形（只读）。
    #[must_use]
    pub fn terrain(&self) -> &MapTerrain {
        &self.terrain
    }

    /// 当前位置（只读）。
    #[must_use]
    pub fn pos(&self) -> GridPos {
        self.pos
    }

    /// 据点列表（只读）。
    #[must_use]
    pub fn landmarks(&self) -> &[(u32, u32)] {
        &self.landmarks
    }

    /// 单步移动（行为与 `oracle_move` 逐位一致）。
    ///
    /// 可通行 → 更新位置 + `Some(新位置)`；越界/阻挡 → 位置不变 + `None`。
    #[must_use]
    pub fn move_to(&mut self, d: Dir) -> Option<GridPos> {
        let next = oracle_move(&self.terrain, self.pos, d);
        if let Some(p) = next {
            self.pos = p;
        }
        next
    }

    /// 选择方向（行为与 `oracle_pick_dir` 逐位一致，不移动）。
    #[must_use]
    pub fn choose_direction(&self) -> Option<Dir> {
        oracle_pick_dir(&self.terrain, self.pos)
    }

    /// 走 N 步（行为与 `oracle_walk` 逐位一致），更新位置。
    ///
    /// 返回 `(终点, 已走步数)`。
    #[must_use]
    pub fn walk(&mut self, d: Dir, steps: u32) -> (GridPos, u32) {
        let (end, walked) = oracle_walk(&self.terrain, self.pos, d, steps);
        self.pos = end;
        (end, walked)
    }

    /// 是否在据点上（行为与 `oracle_at_landmark` 逐位一致）。
    #[must_use]
    pub fn at_landmark(&self) -> bool {
        oracle_at_landmark(&self.landmarks, self.pos)
    }

    /// 事件分派（行为与 `oracle_dispatch_event` 逐位一致）。
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn dispatch(&self, code: u32) -> Option<SceneTarget> {
        oracle_dispatch_event(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_move_and_walk() {
        let t = MapTerrain::open(4, 4);
        let mut s = MapSession::new(t, GridPos::new(2, 2), &[]);
        assert_eq!(s.move_to(Dir::North), Some(GridPos::new(1, 2)));
        assert_eq!(s.pos(), GridPos::new(1, 2));
        // 从 (1,2) 东走 2 步：(1,3) 合法，(1,4) 越界 → 停在 (1,3)，走 1 步
        let (end, walked) = s.walk(Dir::East, 2);
        assert_eq!(end, GridPos::new(1, 3));
        assert_eq!(walked, 1, "col=2+2=4 越界（grid_w=4），走 1 步后撞界");
    }

    #[test]
    fn session_landmark() {
        let t = MapTerrain::open(4, 4);
        let mut s = MapSession::new(t, GridPos::new(0, 0), &[(1, 3)]);
        assert!(!s.at_landmark());
        let _ = s.move_to(Dir::South);
        let _ = s.move_to(Dir::East);
        let _ = s.move_to(Dir::East);
        let _ = s.move_to(Dir::East);
        assert!(s.at_landmark(), "应停在 (1,3) 据点上");
    }

    #[test]
    fn session_dispatch() {
        let t = MapTerrain::open(4, 4);
        let s = MapSession::new(t, GridPos::new(0, 0), &[]);
        assert_eq!(s.dispatch(1), Some(SceneTarget::Msgl));
        assert_eq!(s.dispatch(9), None);
    }
}
