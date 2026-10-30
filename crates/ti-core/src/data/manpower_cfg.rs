//! 领地 v2 资源/事件/商人配置（S33.2）：`balance.manpower` / `balance.events` / `balance.merchants`
//! 的公开配置类型 + Raw 转换。
//!
//! 单一事实源 = `docs/business/combat.md` §兵力模型 + `docs/business/territory.md` §6-7 +
//! `docs/business/economy.md` 资源节。rules 层消费本模块类型（数据为唯一事实源）。

use crate::data::balance::RawEvents;
use crate::data::balance::RawManpower;
use crate::data::balance::RawMerchants;

/// 0 = 未提供（serde default 缺省），取默认值；非 0 = 用户值（越界钳制由调用方做）。
fn pick(v: u32, default: u32) -> u32 {
    if v == 0 { default } else { v }
}

/// 战力系数（`balance.manpower`；`rules::force` 消费）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManpowerCfg {
    /// 武装人手战力系数（每人）。
    pub weapon_coef: u32,
    /// 徒手人手战力系数（每人）。
    pub barehand_coef: u32,
    /// 每战人手损耗（万分比，0..=10000）。
    pub loss_pct: u32,
    /// 每战武器损耗（万分比，0..=10000）。
    pub weapon_loss_pct: u32,
    /// 初始人手（v5 迁移默认源；`territory.md` §11 U4：船 1 级 ≈ 20 水手）。
    pub initial_crew: u32,
    /// 初始武器（v5 迁移默认源）。
    pub initial_arms: u32,
}

impl Default for ManpowerCfg {
    fn default() -> Self {
        Self {
            weapon_coef: 2,
            barehand_coef: 1,
            loss_pct: 1500,        // 15%
            weapon_loss_pct: 2000, // 20%
            initial_crew: 20,
            initial_arms: 5,
        }
    }
}

impl From<RawManpower> for ManpowerCfg {
    fn from(r: RawManpower) -> Self {
        let d = Self::default();
        Self {
            weapon_coef: pick(r.weapon_coef, d.weapon_coef),
            barehand_coef: pick(r.barehand_coef, d.barehand_coef),
            loss_pct: pick(r.loss_pct, d.loss_pct).min(10000),
            weapon_loss_pct: pick(r.weapon_loss_pct, d.weapon_loss_pct).min(10000),
            initial_crew: pick(r.initial_crew, d.initial_crew),
            initial_arms: pick(r.initial_arms, d.initial_arms),
        }
    }
}

/// 海事事件参数（`balance.events`；`rules::events` 消费）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventCfg {
    /// 翻车/风浪：人手损耗（万分比）。
    pub crew_loss_pct: u32,
    /// 翻车/风浪：武器损耗（万分比）。
    pub arms_loss_pct: u32,
    /// 翻车/风浪：金币损耗（万分比）。
    pub gold_loss_pct: u32,
    /// 遇宝藏沉船：金币奖励。
    pub treasure_gold: u32,
    /// 遇宝藏沉船：武器奖励。
    pub treasure_arms: u32,
    /// 流浪儿/孤岛救援：加入人手。
    pub rescue_crew: u32,
}

impl Default for EventCfg {
    fn default() -> Self {
        Self {
            crew_loss_pct: 1000, // 10%
            arms_loss_pct: 1000, // 10%
            gold_loss_pct: 500,  // 5%
            treasure_gold: 200,
            treasure_arms: 3,
            rescue_crew: 4,
        }
    }
}

impl From<RawEvents> for EventCfg {
    fn from(r: RawEvents) -> Self {
        let d = Self::default();
        Self {
            crew_loss_pct: pick(r.crew_loss_pct, d.crew_loss_pct).min(10000),
            arms_loss_pct: pick(r.arms_loss_pct, d.arms_loss_pct).min(10000),
            gold_loss_pct: pick(r.gold_loss_pct, d.gold_loss_pct).min(10000),
            treasure_gold: pick(r.treasure_gold, d.treasure_gold),
            treasure_arms: pick(r.treasure_arms, d.treasure_arms),
            rescue_crew: pick(r.rescue_crew, d.rescue_crew),
        }
    }
}

/// 商人价格（`balance.merchants`；`rules::merchant` 消费）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MerchantCfg {
    /// 武器单价（金）。
    pub weapon_price: u32,
    /// 人手（水手）单价（金）。
    pub slave_price: u32,
}

impl Default for MerchantCfg {
    fn default() -> Self {
        Self { weapon_price: 50, slave_price: 30 }
    }
}

impl From<RawMerchants> for MerchantCfg {
    fn from(r: RawMerchants) -> Self {
        let d = Self::default();
        Self {
            weapon_price: pick(r.weapon_price, d.weapon_price),
            slave_price: pick(r.slave_price, d.slave_price),
        }
    }
}
