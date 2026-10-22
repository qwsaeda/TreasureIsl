//! 战斗会话视图（`BattleView`）：回合推进与快照读取。
//!
//! 拆分自 `game_app.rs`（S20.0b）。

use ti_core::domain::combat::{Side, Unit};
use ti_core::rules::battle::BattleSession;

use super::GameApp;

/// 战斗会话封装（持 `rules::battle::BattleSession`，壳层只编排）。
#[derive(Debug, Clone)]
pub struct BattleView {
    /// 玩家单位（HP/atk，来自角色）。
    pub player: Unit,
    /// BOSS 档 tier（0..=4，`balance.combat.boss_tiers` 表）。
    pub boss_tier: u32,
    /// 战斗会话（`rules::battle::BattleSession`，数值全走 `balance().combat`）。
    pub session: BattleSession,
    /// 已战回合数（`step` 后同步，`BattleSession::round` 镜像）。
    pub rounds: u32,
    /// 玩家当前 HP（`step` 后同步，`BattleSession` 无只读访问器）。
    pub player_hp: u32,
    /// BOSS 当前 HP（`step` 后同步，`BattleSession::boss_hp` 镜像）。
    pub boss_hp: u32,
    /// 胜方（`step` 后同步，`None` = 未结束）。
    pub outcome: Option<Side>,
    /// 玩家初始 HP（血条分母，`Unit.hp`）。
    pub player_hp_max: u32,
    /// BOSS 初始 HP（血条分母，`BattleSession::boss_hp()` 构造时快照）。
    pub boss_hp_max: u32,
}
impl BattleView {
    /// 新建战斗视图（`BattleSession::from_balance`，数值走 `balance().combat`）。
    ///
    /// `balance()` 加载失败返回 `None`（硬错误，不 fallback，`02_conventions.md` §4.4）。
    #[must_use]
    pub fn new(seed: u32, player: Unit, boss_tier: u32) -> Option<Self> {
        let session = BattleSession::from_balance(seed, player, boss_tier).ok()?;
        let outcome = session.outcome();
        let boss_hp = session.boss_hp();
        Some(Self {
            player_hp_max: player.hp,
            boss_hp_max: boss_hp,
            player,
            boss_tier,
            session,
            rounds: 0,
            player_hp: player.hp,
            boss_hp,
            outcome,
        })
    }

    /// 是否结束。
    #[must_use]
    pub fn finished(&self) -> bool {
        self.outcome.is_some()
    }

    /// 胜方（`None` = 未结束，`outcome` 字段投影，测试用）。
    #[must_use]
    pub fn outcome(&self) -> Option<Side> {
        self.outcome
    }

    /// 玩家当前 HP（`player_hp` 字段投影，测试用）。
    #[must_use]
    pub fn player_hp(&self) -> u32 {
        self.player_hp
    }

    /// BOSS 当前 HP（`boss_hp` 字段投影，测试用）。
    #[must_use]
    pub fn boss_hp(&self) -> u32 {
        self.boss_hp
    }

    /// 单回合推进（`BattleSession::step`，确定性走 `balance().combat` + `GameRng`）。
    ///
    /// 已结束的会话不再推进（与 `BattleSession::step` 一致）。
    pub fn step(&mut self) {
        let (ph, eh, winner) = self.session.step();
        self.player_hp = ph;
        self.boss_hp = eh;
        self.rounds = self.session.round();
        self.outcome = winner;
    }
}
impl GameApp {
    /// 战斗会话（只读）。
    #[must_use]
    pub fn battle(&self) -> Option<&BattleView> {
        self.battle.as_ref()
    }
}
