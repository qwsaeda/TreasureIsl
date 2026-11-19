//! 战斗会话视图（`BattleView`）：回合推进与快照读取。
//!
//! 拆分自 `game_app.rs`（S20.0b）。

use ti_core::domain::combat::Side;

use super::{GameApp, Scene};

/// 战斗会话视图（兵力模型，S34.6 D5）：壳层只编排，结算走 `host.rs::battle_step`
/// （`force::resolve` 一次性出结果 + 双方必损）。旧 HP 回合制 `BattleSession`/血条字段已删。
#[derive(Debug, Clone)]
pub struct BattleView {
    /// BOSS 档 tier（`balance.combat.boss_tiers` 难度阶梯；`defender_force` 据此查守方兵力）。
    pub boss_tier: u32,
    /// S28.6 副本战斗标记（`Some((格序号, 岛原型, 等级))` = 领地主副本战斗；`None` = BOSS 战。
    /// 战斗结束钩据此分结算）。
    pub dungeon: Option<(u32, String, u8)>,
    /// S28.6 副本战持久标记（终局后 `dungeon` 被 `battle_step` 消费置 `None`，
    /// 本标记保「本局 = 副本战」语义，`vs` 场景 Back 路由据此区分 副本→主地图 / BOSS→主菜单）。
    pub dungeon_battle: bool,
    /// 胜方（`battle_step` 结算后写，`None` = 未结束）。
    pub outcome: Option<Side>,
    /// 攻方战后剩余人手（`force::resolve.attacker_after.crew`；`None` = 未结算）。
    pub after_crew: Option<u32>,
    /// 攻方战后剩余武器（`force::resolve.attacker_after.arms`）。
    pub after_arms: Option<u32>,
}
impl BattleView {
    /// 新建 BOSS 战视图（兵力模型：只记 `boss_tier`；攻/守兵力由 `battle_step` 经
    /// `defender_force`（守方）+ 席位 `seat_crew/arms`（攻方）结算，构造时全占位 `None`）。
    #[must_use]
    pub const fn new(boss_tier: u32) -> Self {
        Self {
            boss_tier,
            dungeon: None,
            dungeon_battle: false,
            outcome: None,
            after_crew: None,
            after_arms: None,
        }
    }

    /// 副本战视图（S28.6 / territory.md §9：`dungeon = 领地主副本格`，敌方由
    /// `defender_force` 经 `rules.defense_at(isle, level)` 重算；`dungeon_battle` 标记）。
    #[must_use]
    pub fn new_dungeon(dungeon: (u32, String, u8)) -> Self {
        Self {
            boss_tier: 0,
            dungeon: Some(dungeon),
            dungeon_battle: true,
            outcome: None,
            after_crew: None,
            after_arms: None,
        }
    }

    /// 是否结束。
    #[must_use]
    pub fn finished(&self) -> bool {
        self.outcome.is_some()
    }

    /// 胜方（`None` = 未结束，`outcome` 字段投影）。
    #[must_use]
    pub fn outcome(&self) -> Option<Side> {
        self.outcome
    }
}
impl GameApp {
    /// 战斗会话（只读）。
    #[must_use]
    pub fn battle(&self) -> Option<&BattleView> {
        self.battle.as_ref()
    }

    /// S28.6 开一场领地主副本战斗（兵力模型，S34.6 D5：敌方由 `defender_force` 经
    /// `rules.defense_at(isle, level)` 重算，攻方 = 席位 `seat_crew/arms`；确定性 = 无 HP 随机）。
    pub fn start_dungeon_battle(&mut self, cell: u32, isle: &str, level: u8) {
        let view = BattleView::new_dungeon((cell, String::from(isle), level));
        self.battle = Some(view);
        self.battle_won = false;
        self.scene = Scene::Vs;
    }
}
