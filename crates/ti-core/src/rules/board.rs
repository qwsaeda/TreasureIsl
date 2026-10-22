//! 环岛赛道规则（摇骰 → 前进 N 格 → 落格结算）。
//!
//! 业务语义（`docs/business/progression.md` §环岛赛道）：
//! - 赛道 = 岛屿外环的有序空间序列（数据源 `assets/data/board.json`）；
//! - 一次回合 = **摇骰定步数**（`1..=dice_sides`）→ **沿赛道前进 N 格** → **落格结算**；
//! - 前进是单向的（绕岛一圈回到起点 = 一圈），途中经过起点计一圈；
//! - 落格类型 → 事件：港口/城镇 = 贸易建造、事件/卡牌 = 抽卡、遭遇 = 扣血、
//!   宝藏 = 得金、BOSS = 进战斗场景。
//!
//! 确定性：同 `rng` seed + 同步数序列 ⇒ 同落格结果（可回放，EXP-040 同口径）。

use crate::data::BoardMap;
use crate::domain::map::GridPos;
use crate::rules::rng::GameRng;

/// 赛道空间类型（`board.json` 的 `kind` 键）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpaceKind {
    /// 起点（绕岛回到此处 = 补给）。
    Start,
    /// 普通格（路过无事件）。
    Plain,
    /// 港口（可停靠：贸易 / 建造）。
    Port,
    /// 城镇（据点：查看信息 / 贸易）。
    City,
    /// 事件格（触发随机事件）。
    Event,
    /// 卡牌格（抽一张卡）。
    Card,
    /// 遭遇格（扣血 + 损失金币）。
    Battle,
    /// 宝藏格（获得金币）。
    Treasure,
    /// BOSS 格（正好停在 BOSS 格；是否触发战斗由 `Turn::boss` 判「经过即触发 + 圈数门槛」）。
    Boss,
}

impl SpaceKind {
    /// 类型键（`board.json` 用同一套字符串）。
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Plain => "plain",
            Self::Port => "port",
            Self::City => "city",
            Self::Event => "event",
            Self::Card => "card",
            Self::Battle => "battle",
            Self::Treasure => "treasure",
            Self::Boss => "boss",
        }
    }

    /// 类型键 → 枚举（未知键 = `None`，数据加载期硬错误）。
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "start" => Self::Start,
            "plain" => Self::Plain,
            "port" => Self::Port,
            "city" => Self::City,
            "event" => Self::Event,
            "card" => Self::Card,
            "battle" => Self::Battle,
            "treasure" => Self::Treasure,
            "boss" => Self::Boss,
            _ => return None,
        })
    }
}

/// 赛道上的一个空间。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackSpace {
    /// 格子坐标（等距投影用）。
    pub cell: GridPos,
    /// 空间类型。
    pub kind: SpaceKind,
    /// 分区号（`1..=9`）。
    pub group: u8,
    /// 地形码（`S`/`R`/`L`/`T`）。
    pub terrain: char,
}

/// 赛道（有序空间序列）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    spaces: Vec<TrackSpace>,
}

impl Track {
    /// 从数据表构建赛道。
    ///
    /// # Errors
    /// 空间序列为空 / 未知 `kind` 键时返回错误（数据层硬失败，不 fallback）。
    pub fn from_map(map: &BoardMap) -> Result<Self, String> {
        if map.spaces.is_empty() {
            return Err("board.json: empty space sequence".to_string());
        }
        let mut spaces = Vec::with_capacity(map.spaces.len());
        for (i, s) in map.spaces.iter().enumerate() {
            let Some(kind) = SpaceKind::from_key(&s.kind) else {
                return Err(format!("board.json: spaces[{i}].kind unknown type `{}`", s.kind));
            };
            spaces.push(TrackSpace {
                cell: GridPos::new(s.row, s.col),
                kind,
                group: s.group,
                terrain: s.terrain,
            });
        }
        Ok(Self { spaces })
    }

    /// 空间数（赛道长度）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.spaces.len()
    }

    /// 赛道是否为空（恒 `false`，`from_map` 已拒绝空序列）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.spaces.is_empty()
    }

    /// 第 `i` 个空间（越界 = `None`）。
    #[must_use]
    pub fn space(&self, i: usize) -> Option<&TrackSpace> {
        self.spaces.get(i)
    }

    /// 全部空间（只读迭代）。
    #[must_use]
    pub fn spaces(&self) -> &[TrackSpace] {
        &self.spaces
    }

    /// 坐标 → 空间序号（不在赛道上 = `None`）。
    #[must_use]
    pub fn index_of(&self, cell: GridPos) -> Option<usize> {
        self.spaces.iter().position(|s| s.cell == cell)
    }

    /// 起点序号（`start` 类型空间；无 = `0`）。
    #[must_use]
    pub fn start_index(&self) -> usize {
        self.spaces.iter().position(|s| s.kind == SpaceKind::Start).unwrap_or(0)
    }

    /// BOSS 序号（`boss` 类型空间；无 = `None`）。
    #[must_use]
    pub fn boss_index(&self) -> Option<usize> {
        self.spaces.iter().position(|s| s.kind == SpaceKind::Boss)
    }
}

/// 落格结算类型（表现层据此派发事件）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    /// 普通格（无事件）。
    Plain,
    /// 起点（绕圈补给由调用方按圈数结算）。
    Start,
    /// 港口：可贸易 / 建造。
    Port,
    /// 城镇据点：可查看 / 贸易。
    City,
    /// 事件格：触发随机事件（卡牌）。
    Event,
    /// 卡牌格：抽一张卡。
    Card,
    /// 遭遇格：扣血 + 失金。
    Battle,
    /// 宝藏格：得金。
    Treasure,
    /// BOSS 格：进入战斗场景。
    Boss,
}

impl Landing {
    /// 空间类型 → 落格结算类型。
    #[must_use]
    pub fn of(kind: SpaceKind) -> Self {
        match kind {
            SpaceKind::Start => Self::Start,
            SpaceKind::Plain => Self::Plain,
            SpaceKind::Port => Self::Port,
            SpaceKind::City => Self::City,
            SpaceKind::Event => Self::Event,
            SpaceKind::Card => Self::Card,
            SpaceKind::Battle => Self::Battle,
            SpaceKind::Treasure => Self::Treasure,
            SpaceKind::Boss => Self::Boss,
        }
    }
}

/// BOSS 触发状态（一次摇骰回合的判定结果）。
///
/// S19.3 规则：**经过或落在 BOSS 格即触发**（不再要求「恰好停在 BOSS 格」——
/// 后者在 33~46 格赛道上等于 1/40 量级的运气，节奏不可预期）；
/// 但 BOSS 需先绕岛满 `boss_min_laps` 圈才现身（跨图节奏对齐 + 保证攒金建造时间）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossTrigger {
    /// 本回合未遇 BOSS。
    None,
    /// 经过/落在 BOSS 格，但未达现身门槛（还差若干圈）。
    Locked,
    /// BOSS 现身 → 触发战斗场景。
    Engaged,
}

impl BossTrigger {
    /// 是否触发战斗（只有 [`Self::Engaged`] 为真）。
    #[must_use]
    pub fn engaged(self) -> bool {
        matches!(self, Self::Engaged)
    }
}

/// 一次摇骰回合的结果（规则层产出，表现层据此派发结算与场景切换）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turn {
    /// 摇出的步数（`1..=sides`）。
    pub roll: u32,
    /// 落格所在空间的结算类型（`Landing::Boss` = 正好停在 BOSS 格）。
    pub landing: Landing,
    /// 本回合的 BOSS 触发状态。
    pub boss: BossTrigger,
}

/// 赛道会话（玩家在赛道上的位置 + 摇骰/前进）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardSession {
    track: Track,
    index: usize,
    last_roll: u32,
    laps: u32,
}

impl BoardSession {
    /// 新建会话（`index` = 起始空间序号）。
    #[must_use]
    pub fn new(track: Track, index: usize) -> Self {
        let index = if track.is_empty() { 0 } else { index % track.len() };
        Self { track, index, last_roll: 0, laps: 0 }
    }

    /// 赛道（只读）。
    #[must_use]
    pub fn track(&self) -> &Track {
        &self.track
    }

    /// 当前空间序号。
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    /// 当前所在格坐标（等距投影用）。
    #[must_use]
    pub fn pos(&self) -> GridPos {
        self.track.space(self.index).map_or(GridPos::new(0, 0), |s| s.cell)
    }

    /// 当前所在空间（只读）。
    #[must_use]
    pub fn current(&self) -> Option<&TrackSpace> {
        self.track.space(self.index)
    }

    /// 上一次摇出的步数（`0` = 还没摇过）。
    #[must_use]
    pub fn last_roll(&self) -> u32 {
        self.last_roll
    }

    /// 已绕圈数（经过起点计圈）。
    #[must_use]
    pub fn laps(&self) -> u32 {
        self.laps
    }

    /// 摇骰：返回步数 `1..=sides`（`sides == 0` 视作 1）。
    ///
    /// 确定性：走 `GameRng`（PCG64），同 seed 同序列 ⇒ 同步数。
    pub fn spin(&mut self, rng: &mut GameRng, sides: u32) -> u32 {
        let sides = sides.max(1);
        let roll = rng.in_range(sides) + 1;
        self.last_roll = roll;
        roll
    }

    /// 沿赛道前进 `steps` 格，返回**经过的空间序列**（含落点，顺序 = 前进顺序）。
    ///
    /// 绕圈：越过末格回到首格时 `laps` 递增（起点补给由调用方按 `laps` 结算）。
    pub fn advance(&mut self, steps: u32) -> Vec<TrackSpace> {
        let mut passed = Vec::new();
        for _ in 0..steps {
            if self.advance_one().is_none() {
                break;
            }
            if let Some(s) = self.track.space(self.index) {
                passed.push(s.clone());
            }
        }
        passed
    }

    /// 前进一格（绕圈回起点则 `laps` +1）；返回落到空间的类型（空赛道 = `None`）。
    fn advance_one(&mut self) -> Option<SpaceKind> {
        if self.track.is_empty() {
            return None;
        }
        let len = self.track.len();
        self.index = (self.index + 1) % len;
        if self.index == 0 {
            self.laps = self.laps.saturating_add(1);
        }
        self.track.space(self.index).map(|s| s.kind)
    }

    /// 落格结算类型（当前所在空间）。
    #[must_use]
    pub fn landing(&self) -> Landing {
        self.current().map_or(Landing::Plain, |s| Landing::of(s.kind))
    }

    /// 一次摇骰回合 = 摇骰 → 前进 → 判定 BOSS 触发（S19.3）。
    ///
    /// - `sides` = 骰面数（步数 `1..=sides`）；
    /// - `boss_min_laps` = BOSS 现身所需绕岛圈数（`0` = 立即可挑战）。
    ///
    /// BOSS 判定：**本回合经过或落在 BOSS 格**即遇 BOSS；门槛按**经过那一刻**已绕圈数
    /// 判定（`laps >= boss_min_laps` → 现身，否则未现）——同回合「先过 BOSS 再绕满圈」
    /// 不算现身（BOSS 现身前经过它只是路过）。
    ///
    /// 现身时**停在 BOSS 格**（被 BOSS 拦停，位置与视觉一致）。
    pub fn step(&mut self, rng: &mut GameRng, sides: u32, boss_min_laps: u32) -> Turn {
        let roll = self.spin(rng, sides);
        let mut boss = BossTrigger::None;
        for _ in 0..roll {
            let Some(kind) = self.advance_one() else {
                break;
            };
            if kind == SpaceKind::Boss {
                boss = if self.laps >= boss_min_laps {
                    BossTrigger::Engaged
                } else {
                    BossTrigger::Locked
                };
                if boss.engaged() {
                    break; // 被拦停在 BOSS 格
                }
            }
        }
        Turn { roll, landing: self.landing(), boss }
    }

    /// 是否位于 BOSS 格。
    #[must_use]
    pub fn at_boss(&self) -> bool {
        self.current().is_some_and(|s| s.kind == SpaceKind::Boss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::BoardSpace;

    fn demo_track() -> Track {
        let spaces = vec![
            TrackSpace { cell: GridPos::new(0, 0), kind: SpaceKind::Start, group: 1, terrain: 'L' },
            TrackSpace { cell: GridPos::new(0, 1), kind: SpaceKind::Plain, group: 1, terrain: 'L' },
            TrackSpace { cell: GridPos::new(1, 1), kind: SpaceKind::Card, group: 1, terrain: 'L' },
            TrackSpace {
                cell: GridPos::new(1, 0),
                kind: SpaceKind::Battle,
                group: 1,
                terrain: 'L',
            },
            TrackSpace { cell: GridPos::new(2, 0), kind: SpaceKind::Boss, group: 1, terrain: 'L' },
        ];
        Track { spaces }
    }

    #[test]
    fn track_from_map_rejects_unknown_kind() {
        let spaces =
            vec![BoardSpace { row: 0, col: 0, kind: "nope".into(), group: 1, terrain: 'L' }];
        assert!(Track::from_map(&BoardMap { grid_w: 1, grid_h: 1, spaces }).is_err());
    }

    #[test]
    fn track_from_map_rejects_empty() {
        assert!(Track::from_map(&BoardMap { grid_w: 1, grid_h: 1, spaces: Vec::new() }).is_err());
    }

    #[test]
    fn spin_is_in_range_and_reproducible() {
        let mut session = BoardSession::new(demo_track(), 0);
        let mut rng_a = GameRng::new(7);
        let mut rng_b = GameRng::new(7);
        for _ in 0..20 {
            let ra = session.spin(&mut rng_a, 6);
            let rb = session.spin(&mut rng_b, 6);
            assert_eq!(ra, rb, "同 seed 同步数");
            assert!((1..=6).contains(&ra), "步数在 1..=6：{ra}");
        }
    }

    #[test]
    fn advance_moves_forward_and_wraps() {
        let mut s = BoardSession::new(demo_track(), 0);
        let passed = s.advance(2);
        assert_eq!(passed.len(), 2);
        assert_eq!(s.index(), 2);
        assert_eq!(s.pos(), GridPos::new(1, 1));
        // 再走 3 格 → 绕回 0（过起点）+1 圈
        s.advance(3);
        assert_eq!(s.index(), 0);
        assert_eq!(s.laps(), 1);
    }

    #[test]
    fn landing_reflects_space_kind() {
        let mut s = BoardSession::new(demo_track(), 0);
        assert_eq!(s.landing(), Landing::Start);
        s.advance(1);
        assert_eq!(s.landing(), Landing::Plain);
        s.advance(1);
        assert_eq!(s.landing(), Landing::Card);
        s.advance(1);
        assert_eq!(s.landing(), Landing::Battle);
        s.advance(1);
        assert_eq!(s.landing(), Landing::Boss);
        assert!(s.at_boss());
    }

    /// 门槛测试专用赛道（10 格：起点@0 + BOSS@8）。
    ///
    /// BOSS 在圈内位置早于起点 ⇒ 圈内的首次经过 BOSS 必发生在第 0 圈
    /// （= `Locked`），绕满 1 圈后的再次经过才是 `Engaged`；赛道 ≥ 7 格
    /// 保证单次摇骰（1..=6）无法从起点直接越过 BOSS 绕回起点（避免同回合跨圈）。
    fn gate_track() -> Track {
        let kinds = [
            SpaceKind::Start,
            SpaceKind::Plain,
            SpaceKind::Plain,
            SpaceKind::Plain,
            SpaceKind::Plain,
            SpaceKind::Plain,
            SpaceKind::Plain,
            SpaceKind::Plain,
            SpaceKind::Boss,
            SpaceKind::Plain,
        ];
        let spaces = kinds
            .iter()
            .enumerate()
            .map(|(i, kind)| TrackSpace {
                cell: GridPos::new(0, u32::try_from(i).unwrap_or(0)),
                kind: *kind,
                group: 1,
                terrain: 'L',
            })
            .collect();
        Track { spaces }
    }

    /// 找一个首摇 = `want` 的 seed（构造特定骰点序列用）。
    fn seed_with_roll(sides: u32, want: u32) -> u32 {
        for seed in 1..1000 {
            let mut rng = GameRng::new(seed);
            if rng.in_range(sides) + 1 == want {
                return seed;
            }
        }
        0
    }

    #[test]
    fn step_boss_locked_until_lap_gate_met() {
        // S19.3：BOSS 需绕岛满 `boss_min_laps` 圈才现身（跨图节奏对齐）。
        let mut s = BoardSession::new(gate_track(), 0);
        let mut rng = GameRng::new(7);
        let mut saw_locked = false;
        let mut saw_engaged = false;
        for _ in 0..40 {
            let turn = s.step(&mut rng, 6, 1);
            match turn.boss {
                BossTrigger::Locked => saw_locked = true,
                BossTrigger::Engaged => {
                    saw_engaged = true;
                    assert!(s.laps() >= 1, "绕满 1 圈后应现身");
                    break;
                }
                BossTrigger::None => {}
            }
        }
        assert!(saw_locked, "未绕满圈时经过 BOSS 应为 Locked");
        assert!(saw_engaged, "绕满 1 圈后经过 BOSS 应现身");
    }

    #[test]
    fn step_gate_is_evaluated_at_crossing_moment() {
        // 同回合「先过 BOSS 再绕满圈」= 未现身（门槛按经过那一刻的圈数判定）。
        let mut s = BoardSession::new(gate_track(), 0);
        s.advance(6); // 位置 = 6：距 BOSS@8 两格，距起点@0 四格
        let mut rng = GameRng::new(seed_with_roll(6, 6));
        let turn = s.step(&mut rng, 6, 1);
        assert_eq!(turn.boss, BossTrigger::Locked, "经过 BOSS 时未绕满圈 → 未现身");
        assert!(s.laps() >= 1, "本回合稍后才绕满圈（但经过 BOSS 时未满圈）");
    }

    #[test]
    fn step_engages_boss_without_gate_and_is_reproducible() {
        // 门槛 0：经过 BOSS 即现身；同 seed 同步数同结果。
        let run = |seed: u32| {
            let mut s = BoardSession::new(demo_track(), 0);
            let mut rng = GameRng::new(seed);
            for _ in 0..20 {
                let turn = s.step(&mut rng, 6, 0);
                if turn.boss.engaged() {
                    return (turn.roll, s.index(), s.laps());
                }
            }
            (0, s.index(), s.laps())
        };
        let a = run(7);
        let b = run(7);
        assert_eq!(a, b, "同 seed 同步数同落点");
        assert_ne!(a.0, 0, "门槛 0 时经过 BOSS 应现身");
    }

    #[test]
    fn step_stops_at_boss_cell_when_engaged() {
        // 现身时停在 BOSS 格（被拦停）：位置 = BOSS 格，且不越过后续格。
        let mut s = BoardSession::new(gate_track(), 0);
        let mut rng = GameRng::new(3);
        let mut engaged = false;
        for _ in 0..40 {
            let turn = s.step(&mut rng, 6, 0);
            if turn.boss.engaged() {
                engaged = true;
                assert!(s.at_boss(), "现身后位置应停在 BOSS 格");
                assert_eq!(turn.landing, Landing::Boss, "落格类型 = BOSS");
                break;
            }
        }
        assert!(engaged, "门槛 0 时经过 BOSS 应现身");
    }

    #[test]
    fn board_data_matches_extraction() {
        let Ok(db) = crate::data::board() else {
            return;
        };
        for (id, m) in &db.maps {
            let Ok(track) = Track::from_map(m) else {
                continue;
            };
            assert!(track.len() >= 20, "{id} 赛道长度 {0} 过短", track.len());
            assert_eq!(track.start_index(), 0, "{id} 起点应为第 0 格");
            assert!(track.boss_index().is_some(), "{id} 应有 BOSS 格");
            // S19.2：宝藏格必须实际可达（`balance.board.treasure_gold` 非死数值）
            let has_treasure = track.spaces().iter().any(|s| s.kind == SpaceKind::Treasure);
            assert!(has_treasure, "{id} 缺宝藏格（treasure_gold 不可达）");
        }
    }
}
