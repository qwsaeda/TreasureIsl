//! 领地系统类型（S28.3）：`balance.territory` 公开配置类型 + `RawTerritory` 转换。
//!
//! 拆分自 `data/mod.rs`（`audit_code` 文件规模 >900 行超限；一个文件 = 一个职责，`02_conventions` §9）。

use crate::data::balance::RawTerritory;

// ------------------------------------------------------- 领地系统（S28.3）

/// 领地系统参数（`balance.territory`）。
#[derive(Debug, Clone, PartialEq)]
pub struct TerritoryCfg {
    /// 加固等级上限。
    pub max_level: u32,
    /// 过路费每级系数。
    pub toll_per_level: f32,
    /// 加固成本每级系数。
    pub upgrade_cost_per_level: f32,
    /// 产出每级系数。
    pub production_per_level: f32,
    /// 守军每级系数。
    pub defense_per_level: f32,
    /// 首占奖励（金）。
    pub first_capture_bonus: u32,
    /// 副本守军 atk 系数（`atk = defense × 系数`；S28.6）。
    pub dungeon_attack_coef: f32,
    /// 据点岛升级参数。
    pub home_upgrade: HomeUpgradeCfg,
    /// 10 类岛 → 主题色令牌。
    pub isle_tokens: Vec<String>,
    /// AI 席位策略参数。
    pub ai: TerritoryAiCfg,
    /// 10 类岛原型。
    pub prototypes: Vec<TerritoryPrototypeCfg>,
}

/// 据点岛升级参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HomeUpgradeCfg {
    /// 基础成本（金）。
    pub base_cost: u32,
    /// 每级成本系数。
    pub per_level: f32,
    /// 等级上限。
    pub max_level: u32,
}

/// AI 席位策略参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerritoryAiCfg {
    /// 攻占阈值（防守强度 < 此值 ⇒ 攻占；否则交钱）。
    pub attack_threshold: f32,
    /// 交钱偏好（0..1，越高越倾向交钱）。
    pub toll_aversion: f32,
    /// 升级优先阈值。
    pub upgrade_priority: f32,
}

/// 单个岛原型配置。
#[derive(Debug, Clone, PartialEq)]
pub struct TerritoryPrototypeCfg {
    /// 原型 id。
    pub id: String,
    /// 岛名 i18n 键。
    pub name_key: String,
    /// 专属副本场景 id。
    pub dungeon_id: String,
    /// 副本名 i18n 键。
    pub dungeon_name_key: String,
    /// 守军名（显示用）。
    pub guard: String,
    /// 占领后产出语义（显示用）。
    pub benefit: String,
    /// 加固等级上限。
    pub max_level: u32,
    /// 过路费基值。
    pub toll_base: u32,
    /// 加固成本基值。
    pub upgrade_cost_base: u32,
    /// 每圈产出基值。
    pub production_base: u32,
    /// 守军基值（战斗 HP 基线）。
    pub defense_base: u32,
}

/// `RawTerritory` → [`TerritoryCfg`]（`data/mod.rs` `convert()` 调用；单一事实源 `balance.territory`）。
pub(crate) fn convert(raw: RawTerritory) -> TerritoryCfg {
    TerritoryCfg {
        max_level: raw.max_level,
        toll_per_level: raw.toll_per_level,
        upgrade_cost_per_level: raw.upgrade_cost_per_level,
        production_per_level: raw.production_per_level,
        defense_per_level: raw.defense_per_level,
        first_capture_bonus: raw.first_capture_bonus,
        dungeon_attack_coef: raw.dungeon_attack_coef,
        home_upgrade: HomeUpgradeCfg {
            base_cost: raw.home_upgrade.base_cost,
            per_level: raw.home_upgrade.per_level,
            max_level: raw.home_upgrade.max_level,
        },
        isle_tokens: raw.isle_tokens,
        ai: TerritoryAiCfg {
            attack_threshold: raw.ai.attack_threshold,
            toll_aversion: raw.ai.toll_aversion,
            upgrade_priority: raw.ai.upgrade_priority,
        },
        prototypes: raw
            .prototypes
            .into_iter()
            .map(|pr| TerritoryPrototypeCfg {
                id: pr.id,
                name_key: pr.name_key,
                dungeon_id: pr.dungeon_id,
                dungeon_name_key: pr.dungeon_name_key,
                guard: pr.guard,
                benefit: pr.benefit,
                max_level: pr.max_level,
                toll_base: pr.toll_base,
                upgrade_cost_base: pr.upgrade_cost_base,
                production_base: pr.production_base,
                defense_base: pr.defense_base,
            })
            .collect(),
    }
}
