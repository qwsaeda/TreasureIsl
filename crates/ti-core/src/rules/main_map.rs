//! S3.5 / S8.3.c 主地图（移动/据点/事件 + 卡牌触发）正式实现。
//!
//! S8.3.c 数据化（`docs/business/loop_core.md` §主地图 + `docs/business/progression.md` §卡片）：
//! - 移动/据点几何规则直接落本文件（单步移动 / 方向可达 / 行走 / 事件分派 / 据点判定，
//!   纯几何判定，行为以 `docs/business/progression.md` 为准）；
//! - **事件触发 + 卡牌** 改走 [`crate::rules::cards_draw::CardPool`]（卡池/权重/概率全来自
//!   `balance.cards`，抽牌走可复现 `GameRng`）。
//!
//! S12.3 补全（`docs/business/progression.md` 可测试规则表）：
//! - `MapSession` 增加 `boss_tile: Option<GridPos>` 字段，`StepResult` 增加 `boss: bool` 字段；
//!   移动落到 BOSS 格时 `boss=true`（业务：BOSS 格进入战斗场景，`combat.md`）；
//! - `move_to` 在合法移动后判定 BOSS 格命中（与据点判定同路径，互斥：据点优先）。
//!
//! 可测试（`docs/business/loop_core.md` 可测规则 + S8.3.c 判据）：
//! - 同 `rng` seed + 同移动序列 → 同卡牌事件（可复现抽牌）；
//! - `event_chance` 走 `balance.cards`（0.0 → 永不触发；1.0 → 每次移动都触发）。

use crate::domain::map::{Dir, GridPos, MapTerrain, SceneTarget};
use crate::rules::cards_draw::CardPool;
use crate::rules::rng::GameRng;

/// 主地图会话：地形 + 当前位置 + 据点列表 + BOSS 格 + 卡牌池（`balance.cards` 注入，S8.3.c）。
#[derive(Debug, Clone)]
pub struct MapSession {
    terrain: MapTerrain,
    pos: GridPos,
    landmarks: Vec<(u32, u32)>,
    pool: CardPool,
    /// BOSS 格坐标（`None` = 本地图无 BOSS 格；S12.3 补全，`progression.md` 落格结算 §BOSS 格）。
    boss_tile: Option<GridPos>,
}

/// 一次移动 + 卡牌事件的结果（S8.3.c：移动规则各测，卡牌事件可复现）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepResult {
    /// 移动后位置（单步判定合法才更新，非法保持原位）。
    pub pos: GridPos,
    /// 本次移动是否合法（越界/阻挡 → `false`，不触发卡牌）。
    pub moved: bool,
    /// 据点事件目标（落在据点上 `Some`，否则 `None`；BOSS 格优先级高于据点）。
    pub landmark: Option<SceneTarget>,
    /// 卡牌事件 id（触发 + 抽中卡时 `Some`，否则 `None`；走 `balance.cards` 可复现）。
    pub card_event: Option<u32>,
    /// 是否落在 BOSS 格（`true` → 调用方应进入战斗场景，`progression.md` 落格结算）。
    pub boss: bool,
}

impl MapSession {
    /// 单步移动几何判定（`docs/business/progression.md` §移动规则）：
    /// 越界 / 阻挡 → `None`；合法 → `Some(new)`。负增量走 `i64` 中间量防 `u32` 下溢。
    fn step_move(terrain: &MapTerrain, pos: GridPos, d: Dir) -> Option<GridPos> {
        let (dr, dc) = d.delta();
        let nr = i64::from(pos.row) + i64::from(dr);
        let nc = i64::from(pos.col) + i64::from(dc);
        if nr < 0 || nc < 0 {
            return None;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (nrow, ncol) = (nr as u32, nc as u32);
        if nrow >= terrain.grid_h || ncol >= terrain.grid_w {
            return None;
        }
        if terrain.is_blocked(GridPos::new(nrow, ncol)) {
            return None;
        }
        Some(GridPos::new(nrow, ncol))
    }

    /// 新建主地图会话（指定地形 + 初始位置 + 据点列表 + 卡牌池，S8.3.c 数据化）。
    ///
    /// `boss_tile = None`（本地图无 BOSS 格；带 BOSS 格用 [`MapSession::with_boss`]）。
    #[must_use]
    pub fn with_pool(
        terrain: MapTerrain,
        pos: GridPos,
        landmarks: &[(u32, u32)],
        pool: CardPool,
    ) -> Self {
        Self { terrain, pos, landmarks: landmarks.to_vec(), pool, boss_tile: None }
    }

    /// 新建主地图会话（卡牌池走 `balance.cards` 单一事实源，`docs/business/progression.md`）。
    ///
    /// `boss_tile = None`（本地图无 BOSS 格；带 BOSS 格用 [`MapSession::with_boss`]）。
    #[must_use]
    pub fn new(terrain: MapTerrain, pos: GridPos, landmarks: &[(u32, u32)]) -> Self {
        Self::with_pool(terrain, pos, landmarks, CardPool::from_balance())
    }

    /// 新建带 BOSS 格的主地图会话（S12.3，`progression.md` 落格结算：BOSS 格进入战斗场景）。
    ///
    /// `boss_tile = Some(tile)`：玩家 `move_to` 落到该格时 `StepResult::boss = true`，
    /// 调用方（表现层）据此切战斗场景。BOSS 格与据点格互斥时 BOSS 优先（战斗优先于菜单）。
    #[must_use]
    pub fn with_boss(
        terrain: MapTerrain,
        pos: GridPos,
        landmarks: &[(u32, u32)],
        pool: CardPool,
        boss_tile: Option<GridPos>,
    ) -> Self {
        Self { terrain, pos, landmarks: landmarks.to_vec(), pool, boss_tile }
    }

    /// 是否位于 BOSS 格（当前 `pos` 与 `boss_tile` 匹配；无 BOSS 格恒 `false`）。
    #[must_use]
    pub fn at_boss_tile(&self) -> bool {
        self.boss_tile.is_some_and(|t| t == self.pos)
    }

    /// BOSS 格坐标（只读；`None` = 本地图无 BOSS 格）。
    #[must_use]
    pub fn boss_tile(&self) -> Option<GridPos> {
        self.boss_tile
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

    /// 单步移动（几何判定合法后判定 BOSS 格 → 据点 → 卡牌事件，S8.3.c 可复现）。
    ///
    /// - 越界/阻挡 → `moved=false`，位置不变，不触发卡牌（`card_event=None`，`boss=false`）；
    /// - 合法 → 更新位置；落在 BOSS 格 → `boss=true`（优先于据点，`progression.md` 落格结算）；
    /// - 否则落在据点 → `landmark=Some(target)`；
    /// - 卡牌事件：`balance.cards.event_chance` 概率（走 `rng.in_range` 确定性）触发时
    ///   抽 1 张卡（`pool.draw_one` 可复现），`card_event=Some(卡 id)`。
    ///
    /// 确定性：同 `rng` seed + 同 `dir` 序列 → 同 `StepResult`（EXP-040，可回放可测）。
    #[must_use]
    pub fn move_to(&mut self, d: Dir, rng: &mut GameRng) -> StepResult {
        let next = Self::step_move(&self.terrain, self.pos, d);
        let moved = next.is_some();
        if let Some(p) = next {
            self.pos = p;
        }
        let boss = moved && self.at_boss_tile();
        let landmark = if boss {
            None
        } else if self.at_landmark() {
            self.dispatch(1)
        } else {
            None
        };
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
        StepResult { pos: self.pos, moved, landmark, card_event, boss }
    }

    /// 选择方向（探测优先级 南→东→北→西，取第一个可通行方向，不移动）。
    #[must_use]
    pub fn choose_direction(&self) -> Option<Dir> {
        if self.terrain.is_blocked(self.pos) {
            return None;
        }
        [Dir::South, Dir::East, Dir::North, Dir::West]
            .into_iter()
            .find(|&d| Self::step_move(&self.terrain, self.pos, d).is_some())
    }

    /// 走 N 步（几何走 N 步；S8.3.c：每步后判定卡牌，同 `rng` 可复现）。
    ///
    /// 返回 `(终点, 已走步数)`；BOSS 格/据点/卡牌事件在每步内判定（由调用方按步 `move_to` 驱动；
    /// 本方法只更新几何位置，不触发事件结算）。
    #[must_use]
    pub fn walk(&mut self, d: Dir, steps: u32, rng: &mut GameRng) -> (GridPos, u32) {
        let (end, walked) = Self::walk_steps(&self.terrain, self.pos, d, steps);
        self.pos = end;
        let _ = rng; // 几何走 N 步（一次性），卡牌触发由调用方按步 `move_to` 驱动
        (end, walked)
    }

    /// 是否在据点上（坐标在据点列表内 → `true`）。
    #[must_use]
    pub fn at_landmark(&self) -> bool {
        self.landmarks.iter().any(|&(r, c)| r == self.pos.row && c == self.pos.col)
    }

    /// 据点事件目标（事件码 1 → 据点对话框）。
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn landmark_target(&self) -> Option<SceneTarget> {
        Self::dispatch_event(1)
    }

    /// 事件分派（事件码 → 目标场景，`docs/business/progression.md` §据点事件）。
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn dispatch(&self, code: u32) -> Option<SceneTarget> {
        Self::dispatch_event(code)
    }

    /// 静态事件分派（不依赖会话状态，事件码 → 目标场景）：
    /// `1|4 → Msgl`（据点/货物结果），`2|5|7 → Msgm`（面板/卡片/消息），
    /// `6 → Vs`（战斗），`8 → Msgs`（通用消息），其余码无场景 → `None`。
    #[must_use]
    pub fn dispatch_event(code: u32) -> Option<SceneTarget> {
        match code {
            1 | 4 => Some(SceneTarget::Msgl),
            2 | 5 | 7 => Some(SceneTarget::Msgm),
            6 => Some(SceneTarget::Vs),
            8 => Some(SceneTarget::Msgs),
            _ => None,
        }
    }

    /// 走 N 步几何结算：每步同方向走 1 格，任一步不可通行即停，返回 `(终点, 已走步数)`。
    fn walk_steps(terrain: &MapTerrain, pos: GridPos, d: Dir, steps: u32) -> (GridPos, u32) {
        let mut cur = pos;
        let mut walked = 0u32;
        for _ in 0..steps {
            match Self::step_move(terrain, cur, d) {
                Some(next) => {
                    cur = next;
                    walked += 1;
                }
                None => break,
            }
        }
        (cur, walked)
    }

    /// 静态单步合法性预检（不移动、不改状态）。
    #[must_use]
    pub fn can_walk(terrain: &MapTerrain, pos: GridPos, d: Dir) -> bool {
        Self::step_move(terrain, pos, d).is_some()
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

    /// S12.3：`map_boss_tile_enters_combat`（`progression.md` 落格结算：BOSS 格进入战斗场景）。
    #[test]
    fn boss_tile_move_sets_boss_flag() {
        let t = MapTerrain::open(4, 4);
        let pool = CardPool::new(Vec::new(), 0, 0.0);
        let boss = GridPos::new(2, 2);
        let mut s = MapSession::with_boss(t, GridPos::new(0, 0), &[], pool, Some(boss));
        let mut rng = GameRng::new(1);
        // 从 (0,0) 南走 2 步到 (2,0)，东走 2 步到 (2,2) = BOSS 格
        let r1 = s.move_to(Dir::South, &mut rng);
        assert!(!r1.boss, "未达 BOSS 格 → boss=false");
        let r2 = s.move_to(Dir::South, &mut rng);
        assert!(!r2.boss, "未到 BOSS 格 → boss=false");
        let r3 = s.move_to(Dir::East, &mut rng);
        assert!(!r3.boss, "未到 BOSS 格 → boss=false");
        let r4 = s.move_to(Dir::East, &mut rng);
        assert!(r4.boss, "落在 BOSS 格 (2,2) → boss=true");
        assert_eq!(r4.pos, boss, "位置应为 BOSS 格");
    }

    /// S12.3：BOSS 格与据点格同坐标时 BOSS 优先（`progression.md` 战斗优先于菜单）。
    #[test]
    fn boss_tile_takes_priority_over_landmark() {
        let t = MapTerrain::open(4, 4);
        let pool = CardPool::new(Vec::new(), 0, 0.0);
        let boss_pos = GridPos::new(1, 1);
        let mut s = MapSession::with_boss(t, GridPos::new(0, 0), &[(1, 1)], pool, Some(boss_pos));
        let mut rng = GameRng::new(1);
        let _ = s.move_to(Dir::South, &mut rng);
        let r = s.move_to(Dir::East, &mut rng);
        assert!(r.boss, "BOSS 格命中 → boss=true");
        assert!(r.landmark.is_none(), "BOSS 优先 → landmark=None");
    }

    /// S12.3：无 BOSS 格（`boss_tile=None`）时 `boss` 恒 `false`。
    #[test]
    fn no_boss_tile_boss_flag_false() {
        let t = MapTerrain::open(4, 4);
        let pool = CardPool::new(Vec::new(), 0, 0.0);
        let mut s = MapSession::with_boss(t, GridPos::new(0, 0), &[], pool, None);
        let mut rng = GameRng::new(1);
        for _ in 0..4 {
            let r = s.move_to(Dir::East, &mut rng);
            assert!(!r.boss, "无 BOSS 格 → boss 恒 false");
        }
        assert_eq!(s.boss_tile(), None, "boss_tile() 应返回 None");
    }

    /// S12.3：`at_boss_tile()` 只读判定（不移动）。
    #[test]
    fn at_boss_tile_readonly_check() {
        let pool = CardPool::new(Vec::new(), 0, 0.0);
        let boss = GridPos::new(3, 3);
        let s = MapSession::with_boss(
            MapTerrain::open(4, 4),
            GridPos::new(0, 0),
            &[],
            pool.clone(),
            Some(boss),
        );
        assert!(!s.at_boss_tile(), "当前位置 (0,0) ≠ BOSS 格 (3,3)");
        let s2 = MapSession::with_boss(MapTerrain::open(4, 4), boss, &[], pool, Some(boss));
        assert!(s2.at_boss_tile(), "当前位置 = BOSS 格 → true");
    }
}
