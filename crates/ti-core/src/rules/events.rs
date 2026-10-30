//! 海事随机事件（v2，2026-10-30 用户定案）。
//!
//! 单一事实源 = [`docs/business/territory.md`](../../../../docs/business/territory.md) §6。
//! 事件落在**不能占领的功能格**（概率触发，非必发生）；常用集随游戏自带，每图可配特有事件
//! （`map.json`/`manifest → events[]` 与 `balance.events` 合并）。
//!
//! 本模块只算**效果**（确定性），应用（改席位账本）由调用方做——保持可测、可复用。

use crate::data::EventCfg;

/// 海事事件（效果式枚举）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaritimeEvent {
    /// 风神保佑：加速。`jump_cells` 非 0 = 随机前进/后退 N 格；0 = 多扔骰子一次。
    WindGod {
        /// 前进（正）/后退（负）格数；0 = 改用 `extra_roll`（多扔骰子一次）。
        jump_cells: i32,
    },
    /// 翻车：沉船，损舰队资产（人手/武器/金币 %）。
    Shipwreck,
    /// 遇宝藏沉船：+ 金币 / 武器。
    TreasureWreck,
    /// 风浪：损舰队资产。
    Storm,
    /// 铁匠：购买武器（走 `rules::merchant`，事件只标记）。
    Blacksmith,
    /// 流浪儿：加入队伍（+ 人手）。
    Urchin,
    /// 孤岛救援：救起水手（+ 人手）。
    IslandRescue,
}

/// 事件效果（确定性；应用由调用方做）。
///
/// 损耗 = 百分比字段（`*_loss_pct`，调用方按**实际**资源算）；奖励 = 绝对增量字段
/// （`gold_delta`/`crew_delta`/`arms_delta`）。两类事件二选一（损耗事件用 pct，奖励用绝对）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventEffect {
    /// 金币绝对增量（+ 奖励）。
    pub gold_delta: i64,
    /// 人手绝对增量（+ 加入）。
    pub crew_delta: i32,
    /// 武器绝对增量（+ 奖励）。
    pub arms_delta: i32,
    /// 金币损耗（万分比；调用方按实际金算）。
    pub gold_loss_pct: u32,
    /// 人手损耗（万分比；调用方按实际人手算）。
    pub crew_loss_pct: u32,
    /// 武器损耗（万分比；调用方按实际武器算）。
    pub arms_loss_pct: u32,
    /// 是否多扔骰子一次（风神保佑加速，`jump_cells == 0` 时）。
    pub extra_roll: bool,
    /// 随机前进/后退格数（风神保佑；0 = 不跳，用 `extra_roll`）。
    pub jump_cells: i32,
    /// 是否触发商人购买（铁匠）。
    pub merchant_buy: bool,
}

impl MaritimeEvent {
    /// 计算效果（确定性；损耗 = pct 字段，奖励 = 绝对字段）。
    #[must_use]
    pub fn effect(&self, cfg: &EventCfg) -> EventEffect {
        match self {
            Self::WindGod { jump_cells } => EventEffect {
                gold_delta: 0,
                crew_delta: 0,
                arms_delta: 0,
                gold_loss_pct: 0,
                crew_loss_pct: 0,
                arms_loss_pct: 0,
                extra_roll: *jump_cells == 0,
                jump_cells: *jump_cells,
                merchant_buy: false,
            },
            Self::Shipwreck | Self::Storm => EventEffect {
                gold_delta: 0,
                crew_delta: 0,
                arms_delta: 0,
                gold_loss_pct: cfg.gold_loss_pct,
                crew_loss_pct: cfg.crew_loss_pct,
                arms_loss_pct: cfg.arms_loss_pct,
                extra_roll: false,
                jump_cells: 0,
                merchant_buy: false,
            },
            Self::TreasureWreck => EventEffect {
                gold_delta: i64::from(cfg.treasure_gold),
                crew_delta: 0,
                arms_delta: i32::try_from(cfg.treasure_arms).unwrap_or(0),
                gold_loss_pct: 0,
                crew_loss_pct: 0,
                arms_loss_pct: 0,
                extra_roll: false,
                jump_cells: 0,
                merchant_buy: false,
            },
            Self::Blacksmith => EventEffect {
                gold_delta: 0,
                crew_delta: 0,
                arms_delta: 0,
                gold_loss_pct: 0,
                crew_loss_pct: 0,
                arms_loss_pct: 0,
                extra_roll: false,
                jump_cells: 0,
                merchant_buy: true,
            },
            Self::Urchin | Self::IslandRescue => EventEffect {
                gold_delta: 0,
                crew_delta: i32::try_from(cfg.rescue_crew).unwrap_or(0),
                arms_delta: 0,
                gold_loss_pct: 0,
                crew_loss_pct: 0,
                arms_loss_pct: 0,
                extra_roll: false,
                jump_cells: 0,
                merchant_buy: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> EventCfg {
        EventCfg::default()
    }

    #[test]
    fn wind_god_extra_roll_when_jump_zero() {
        let e = MaritimeEvent::WindGod { jump_cells: 0 };
        let fx = e.effect(&cfg());
        assert!(fx.extra_roll);
        assert_eq!(fx.jump_cells, 0);
        assert_eq!(fx.crew_loss_pct, 0);
    }

    #[test]
    fn wind_god_jump_when_nonzero() {
        let e = MaritimeEvent::WindGod { jump_cells: -3 };
        let fx = e.effect(&cfg());
        assert!(!fx.extra_roll);
        assert_eq!(fx.jump_cells, -3);
    }

    #[test]
    fn treasure_wreck_grants_gold_and_arms() {
        let fx = MaritimeEvent::TreasureWreck.effect(&cfg());
        assert_eq!(fx.gold_delta, 200);
        assert_eq!(fx.arms_delta, 3);
        assert_eq!(fx.gold_loss_pct, 0);
    }

    #[test]
    fn rescue_adds_crew() {
        let fx = MaritimeEvent::IslandRescue.effect(&cfg());
        assert_eq!(fx.crew_delta, 4);
        let fx2 = MaritimeEvent::Urchin.effect(&cfg());
        assert_eq!(fx2.crew_delta, 4);
    }

    #[test]
    fn blacksmith_marks_merchant_buy() {
        let fx = MaritimeEvent::Blacksmith.effect(&cfg());
        assert!(fx.merchant_buy);
    }

    #[test]
    fn shipwreck_losses_are_pct_fields() {
        let fx = MaritimeEvent::Shipwreck.effect(&cfg());
        assert_eq!(fx.crew_loss_pct, 1000);
        assert_eq!(fx.arms_loss_pct, 1000);
        assert_eq!(fx.gold_loss_pct, 500);
        assert_eq!(fx.gold_delta, 0);
    }

    #[test]
    fn deterministic_same_event_same_effect() {
        assert_eq!(
            MaritimeEvent::WindGod { jump_cells: 2 }.effect(&cfg()),
            MaritimeEvent::WindGod { jump_cells: 2 }.effect(&cfg())
        );
    }
}
