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

/// 战斗道具效果（S27.7：`docs/business/combat.md` §道具表）。
///
/// **规则只消费效果，不查表**：哪个道具带哪个效果 = `balance.combat.items[]`（数据），
/// 效果语义 = 本枚举（规则），两者不耦合 ⇒ 调数值/加道具不改 `rules::battle`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEffect {
    /// 对敌固定伤害（不吃 atk、不吃随机）。
    Damage(u32),
    /// 按上限百分比回复自身 HP（结算时钳到上限，`pct > 100` 按 100 计）。
    HealPct(u32),
    /// 若干回合内 atk 按百分比提升（`rounds` = 生效的攻击次数）。
    AtkBuff {
        /// 提升百分比（`50` = ×1.5）。
        pct: u32,
        /// 生效回合数（使用当回合不计）。
        rounds: u32,
    },
    /// 敌方下一次反击落空（消耗 1 次）。
    Stagger,
}

/// 玩家回合动作（S27.7：`docs/business/combat.md` §战斗动作）。
///
/// `Attack` 是默认动作 = 旧行为（`BattleSession::step()` ≡ `step_with(Attack)`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CombatAction {
    /// 普通攻击（走 §伤害模型）。
    #[default]
    Attack,
    /// 防御（当回合敌方反击减伤，`balance.combat.defend_reduction_pct`）。
    Defend,
    /// 使用道具（结算该道具效果）。
    Item(ItemEffect),
}

/// S15.6 QA141：HP 血条填充宽度百分比（0..=100，整数）。
///
/// `displayed` = 显示 HP（EXP-085 平滑插值），`max` = 满血值。
/// 显示值若越出 `[0, max]`（插值过冲/脏数据）先**钳制**再折算，
/// 保证血条宽度永不超过槽宽（越界 = 渲染层溢出到邻槽，QA140 判据域）。
/// 纯函数、确定性（EXP-080）：同输入恒同输出。
#[must_use]
pub fn hp_fill_pct(displayed: u32, max: u32) -> u32 {
    if max == 0 {
        return 0;
    }
    displayed.min(max) * 100 / max
}

#[cfg(test)]
mod tests {
    use super::hp_fill_pct;

    #[test]
    fn hp_fill_bounds() {
        assert_eq!(hp_fill_pct(0, 100), 0);
        assert_eq!(hp_fill_pct(50, 100), 50);
        assert_eq!(hp_fill_pct(100, 100), 100);
        // 满血 = max
        assert_eq!(hp_fill_pct(0, 0), 0, "max=0 不除零");
        // 显示值过冲钳制（QA141：血条宽度永不超过槽宽）
        assert_eq!(hp_fill_pct(150, 100), 100);
        // 整型截断（101/100 → 100）
        assert_eq!(hp_fill_pct(101, 100), 100);
        assert_eq!(hp_fill_pct(99, 100), 99);
    }
}
