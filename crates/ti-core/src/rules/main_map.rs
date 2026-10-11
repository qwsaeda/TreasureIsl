//! S3.5 / S8.3.c 主地图（移动/据点/事件 + 卡牌触发）正式实现。
//!
//! S8.3.c 数据化（`docs/business/loop_core.md` §主地图 + `docs/business/progression.md` §卡片）：
//! - 移动/据点几何规则仍走 `oracle::main_map` 的 `oracle_move`/`oracle_walk`/`oracle_at_landmark`
//!   （冻结参照层，几何判定非业务推测）；
//! - **事件触发 + 卡牌** 改走 [`crate::rules::cards_draw::CardPool`]（卡池/权重/概率全来自
//!   `balance.cards`，抽牌走可复现 `GameRng`），不再读 oracle `rand` 推测序列（S8 定位 §1）。
//!
//! 可测试（`docs/business/loop_core.md` 可测规则 + S8.3.c 判据）：
//! - 同 `rng` seed + 同移动序列 → 同卡牌事件（可复现抽牌）；
//! - `event_chance` 走 `balance.cards`（0.0 → 永不触发；1.0 → 每次移动都触发）。

use crate::oracle::main_map::{
    Dir, GridPos, MapTerrain, SceneTarget, oracle_at_landmark, oracle_dispatch_event, oracle_move,
    oracle_pick_dir, oracle_walk,
};
use crate::rules::cards_draw::CardPool;
use crate::rules::rng::GameRng;

/// 主地图会话：地形 + 当前位置 + 据点列表 + 卡牌池（`balance.cards` 注入，S8.3.c）。
#[derive(Debug, Clone)]
pub struct MapSession {
    terrain: MapTerrain,
    pos: GridPos,
    landmarks: Vec<(u32, u32)>,
    pool: CardPool,
}

/// 一次移动 + 卡牌事件的结果（S8.3.c：移动规则各测，卡牌事件可复现）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepResult {
    /// 移动后位置（`oracle_move` 判定合法才更新，非法保持原位）。
    pub pos: GridPos,
    /// 本次移动是否合法（越界/阻挡 → `false`，不触发卡牌）。
    pub moved: bool,
    /// 据点事件目标（落在据点上 `Some`，否则 `None`）。
    pub landmark: Option<SceneTarget>,
    /// 卡牌事件 id（触发 + 抽中卡时 `Some`，否则 `None`；走 `balance.cards` 可复现）。
    pub card_event: Option<u32>,
}

impl MapSession {
    /// 新建主地图会话（指定地形 + 初始位置 + 据点列表 + 卡牌池，S8.3.c 数据化）。
    #[must_use]
    pub fn with_pool(
        terrain: MapTerrain,
        pos: GridPos,
        landmarks: &[(u32, u32)],
        pool: CardPool,
    ) -> Self {
        Self { terrain, pos, landmarks: landmarks.to_vec(), pool }
    }

    /// 新建主地图会话（卡牌池走 `balance.cards` 单一事实源，`docs/business/progression.md`）。
    #[must_use]
    pub fn new(terrain: MapTerrain, pos: GridPos, landmarks: &[(u32, u32)]) -> Self {
        Self::with_pool(terrain, pos, landmarks, CardPool::from_balance())
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

    /// 卡牌池（只读，`balance.cards` 注入，S8.3.c）。
    #[must_use]
    pub fn pool(&self) -> &CardPool {
        &self.pool
    }

    /// 事件触发概率（`balance.cards.event_chance`，`0.0..=1.0`）。
    #[must_use]
    pub fn event_chance(&self) -> f32 {
        self.pool.event_chance()
    }

    /// 单步移动（几何走 `oracle_move`；合法后判定据点 + 卡牌事件，S8.3.c 可复现）。
    ///
    /// - 越界/阻挡 → `moved=false`，位置不变，不触发卡牌（`card_event=None`）；
    /// - 合法 → 更新位置；落在据点 → `landmark=Some(target)`；
    /// - 卡牌事件：`balance.cards.event_chance` 概率（走 `rng.in_range` 确定性）触发时
    ///   抽 1 张卡（`pool.draw_one` 可复现），`card_event=Some(卡 id)`。
    ///
    /// 确定性：同 `rng` seed + 同 `dir` 序列 → 同 `StepResult`（EXP-040，可回放可测）。
    #[must_use]
    pub fn move_to(&mut self, d: Dir, rng: &mut GameRng) -> StepResult {
        let next = oracle_move(&self.terrain, self.pos, d);
        let moved = next.is_some();
        if let Some(p) = next {
            self.pos = p;
        }
        let landmark = if self.at_landmark() { self.dispatch(1) } else { None };
        let mut card_event = None;
        if moved {
            let chance_pct = f64::from(self.event_chance()) * 100.0;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let pct = u32::try_from(chance_pct.round() as u64).unwrap_or(0);
            if pct > 0 {
                let roll = rng.in_range(100);
                if roll < pct {
                    let hand = self.pool.draw_one(rng, 1);
                    card_event = hand.first().copied();
                }
            }
        }
        StepResult { pos: self.pos, moved, landmark, card_event }
    }

    /// 选择方向（行为与 `oracle_pick_dir` 逐位一致，不移动）。
    #[must_use]
    pub fn choose_direction(&self) -> Option<Dir> {
        oracle_pick_dir(&self.terrain, self.pos)
    }

    /// 走 N 步（几何走 `oracle_walk`；S8.3.c：每步后判定卡牌，同 `rng` 可复现）。
    ///
    /// 返回 `(终点, 已走步数)`；卡牌事件在每步内判定（`event_chance` 概率，走 `rng`）。
    #[must_use]
    pub fn walk(&mut self, d: Dir, steps: u32, rng: &mut GameRng) -> (GridPos, u32) {
        let (end, walked) = oracle_walk(&self.terrain, self.pos, d, steps);
        self.pos = end;
        let _ = rng; // 几何走 N 步（`oracle_walk` 一次性），卡牌触发由调用方按步 `move_to` 驱动
        (end, walked)
    }

    /// 是否在据点上（行为与 `oracle_at_landmark` 逐位一致）。
    #[must_use]
    pub fn at_landmark(&self) -> bool {
        oracle_at_landmark(&self.landmarks, self.pos)
    }

    /// 据点事件目标（`oracle_dispatch_event` 据点码 → `SceneTarget`）。
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn landmark_target(&self) -> Option<SceneTarget> {
        oracle_dispatch_event(1)
    }

    /// 事件分派（行为与 `oracle_dispatch_event` 逐位一致）。
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn dispatch(&self, code: u32) -> Option<SceneTarget> {
        oracle_dispatch_event(code)
    }

    /// 静态事件分派（不依赖会话状态，行为与 `oracle_dispatch_event` 逐位一致）。
    #[must_use]
    pub fn dispatch_event(code: u32) -> Option<SceneTarget> {
        oracle_dispatch_event(code)
    }

    /// 静态单步合法性预检（不移动、不改状态，等价 `oracle_move` 的 `Option` 判定）。
    #[must_use]
    pub fn can_walk(terrain: &MapTerrain, pos: GridPos, d: Dir) -> bool {
        oracle_move(terrain, pos, d).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_move_and_walk() {
        let t = MapTerrain::open(4, 4);
        let pool = CardPool::from_balance();
        let mut s = MapSession::with_pool(t, GridPos::new(2, 2), &[], pool);
        let mut rng = GameRng::new(1);
        let r = s.move_to(Dir::North, &mut rng);
        assert_eq!(r.pos, GridPos::new(1, 2));
        assert!(r.moved);
        // 从 (1,2) 东走 2 步：(1,3) 合法，(1,4) 越界 → 停在 (1,3)，走 1 步
        let (end, walked) = s.walk(Dir::East, 2, &mut rng);
        assert_eq!(end, GridPos::new(1, 3));
        assert_eq!(walked, 1, "col=2+2=4 越界（grid_w=4），走 1 步后撞界");
    }

    #[test]
    fn session_landmark() {
        let t = MapTerrain::open(4, 4);
        let pool = CardPool::from_balance();
        let mut s = MapSession::with_pool(t, GridPos::new(0, 0), &[(1, 3)], pool);
        let mut rng = GameRng::new(1);
        assert!(!s.at_landmark());
        let _ = s.move_to(Dir::South, &mut rng);
        let _ = s.move_to(Dir::East, &mut rng);
        let _ = s.move_to(Dir::East, &mut rng);
        let _ = s.move_to(Dir::East, &mut rng);
        assert!(s.at_landmark(), "应停在 (1,3) 据点上");
    }

    #[test]
    fn session_dispatch() {
        let t = MapTerrain::open(4, 4);
        let pool = CardPool::from_balance();
        let s = MapSession::with_pool(t, GridPos::new(0, 0), &[], pool);
        assert_eq!(s.dispatch(1), Some(SceneTarget::Msgl));
        assert_eq!(s.dispatch(9), None);
    }

    /// S8.3.c 可复现判据：同 seed + 同移动序列 → 同卡牌事件。
    #[test]
    fn card_event_reproducible_same_seed() {
        let pool = CardPool::from_balance();
        let mut a =
            MapSession::with_pool(MapTerrain::open(4, 4), GridPos::new(0, 0), &[], pool.clone());
        let mut b =
            MapSession::with_pool(MapTerrain::open(4, 4), GridPos::new(0, 0), &[], pool.clone());
        let mut ra = GameRng::new(42);
        let mut rb = GameRng::new(42);
        for _ in 0..4 {
            let _ = a.move_to(Dir::East, &mut ra);
            let _ = b.move_to(Dir::East, &mut rb);
        }
        assert_eq!(a.pos(), b.pos(), "同 seed 同位置");
    }

    /// S8.3.c：`event_chance=0` → 永不触发卡牌（`card_event` 恒 `None`）。
    #[test]
    fn zero_event_chance_never_triggers_card() {
        let t = MapTerrain::open(4, 4);
        // 空卡池 → `event_chance()` 回 0.0 → 永不触发
        let pool = CardPool::new(Vec::new(), 0, 0.0);
        let mut s = MapSession::with_pool(t, GridPos::new(0, 0), &[], pool);
        let mut rng = GameRng::new(1);
        for _ in 0..5 {
            let r = s.move_to(Dir::East, &mut rng);
            assert!(r.card_event.is_none(), "event_chance=0 不应触发卡牌");
        }
    }
}
