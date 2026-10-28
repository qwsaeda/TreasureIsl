//! 战斗会话视图（`BattleView`）：回合推进与快照读取。
//!
//! 拆分自 `game_app.rs`（S20.0b）。

use ti_core::domain::combat::{Side, Unit};
use ti_core::rules::battle::BattleSession;

use ti_core::data::CombatCfg;
use ti_core::data::balance;
use ti_core::rules::cards::CardState;
use ti_core::rules::territory::TerritoryRules;

use super::{GameApp, Scene};

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
    /// S28.6 副本战斗标记（`Some((格序号, 岛原型, 等级))` = 领地主副本战斗，敌方 =
    /// `rules::territory::defender_unit`；`None` = BOSS 战斗。战斗结束钩据此分结算）。
    pub dungeon: Option<(u32, String, u8)>,
    /// S28.6 副本战持久标记（终局后 `dungeon` 被 `battle_step` 消费置 `None`，
    /// 本标记保「本局 = 副本战」语义，`vs` 场景 Back 路由据此区分 副本→主地图 / BOSS→主菜单）。
    pub dungeon_battle: bool,
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
            dungeon: None,
            dungeon_battle: false,
        })
    }

    /// 副本守军战会话（S28.6 / territory.md §9：敌方 = 调用方注入的守军 [`Unit`]，
    /// 数值 = `rules::territory::defender_unit`；伤害模型同 BOSS 战（`CombatCfg` + seed 确定性）。
    #[must_use]
    pub fn new_dungeon(cfg: &CombatCfg, seed: u32, player: Unit, enemy: Unit) -> Self {
        let session = BattleSession::new_with_enemy(cfg.clone(), seed, player, enemy);
        let outcome = session.outcome();
        let boss_hp = session.boss_hp();
        Self {
            player_hp_max: player.hp,
            boss_hp_max: boss_hp,
            player,
            boss_tier: 0,
            session,
            rounds: 0,
            player_hp: player.hp,
            boss_hp,
            outcome,
            dungeon: None,
            dungeon_battle: true,
        }
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

    /// S28.6 开一场领地主副本战斗（敌方 = `rules::territory::defender_unit(isle, level)`；
    /// 数值全走 `balance`：玩家单位同 `start_battle` 成长口径（建筑加成 + 卡牌减益），
    /// 伤害模型同 BOSS 战（`rules::battle`，确定性 = 同 seed 同轨迹，territory.md §9）。
    /// 数据缺失 = 硬错误 eprintln + return（`02_conventions.md` §4.4，不 fallback）。
    pub fn start_dungeon_battle(&mut self, cell: u32, isle: &str, level: u8) {
        let seed = self.rng.next().unsigned_abs();
        let b = match balance() {
            Ok(b) => b,
            Err(_e) => {
                eprintln!("start_dungeon_battle: balance.json 加载失败（硬错误，不 fallback）");
                return;
            }
        };
        let Some(player) = b.combat.player else {
            eprintln!("start_dungeon_battle: balance.combat.player 缺失（硬错误，不 fallback）");
            return;
        };
        let player_atk = player.atk.saturating_add(self.build_atk_bonus());
        let player_hp =
            player.hp.saturating_add(self.build_hp_bonus()).saturating_sub(self.card_state.hp);
        let player_unit = Unit { hp: player_hp, atk: player_atk };
        self.card_state = CardState::default();
        let enemy = TerritoryRules::new(b).defender_unit(isle, level);
        let mut view = BattleView::new_dungeon(&b.combat, seed, player_unit, enemy);
        view.dungeon = Some((cell, String::from(isle), level));
        self.battle = Some(view);
        self.battle_won = false;
        self.scene = Scene::Vs;
    }
}
