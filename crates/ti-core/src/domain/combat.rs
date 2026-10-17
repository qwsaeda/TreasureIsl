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
