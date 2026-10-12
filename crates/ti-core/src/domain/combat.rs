//! 战斗域类型（业务文档：`docs/business/combat.md`）。
//!
//! 只放**类型定义与纯查询**，不含回合推进（战斗推进在 `rules::battle::BattleSession`，
//! 数值全部从 `balance.combat` 注入）。

/// 战斗单位（玩家棋子或 BOSS）。
///
/// 业务语义（`docs/business/combat.md`）：
/// - `hp`：当前生命值，归零即败；
/// - `atk`：攻击力，伤害公式的输入之一（`docs/business/combat.md` §伤害模型）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    /// 当前 HP。
    pub hp: u32,
    /// 攻击力。
    pub atk: u32,
}

/// 阵营。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// 玩家。
    Player,
    /// 敌方（BOSS）。
    Enemy,
}
