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
    /// 每圈领地维护费（金，S61 飞轮 R21）：每圈过起点按「拥有领地格数 × 此值」扣金，
    /// 让金随圈数可耗尽 ⇒ 破产可触发（消除旧「金单调累积无衰减」对局不收敛根因）。
    /// 缺省 0 = 维持旧行为（无维护费，向后兼容老档/旧 balance）。
    pub maintenance_per_lap: u32,
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

/// AI 攻占决策公式形态（易变平衡项，`balance.territory.ai.siege_mode`；数据驱动，改 AI 激进/保守不改代码）。
///
/// - `WinOnly`：预测胜即攻占（旧 `attack_threshold > 0.0` 开关语义，等价 `>= 1.0` 战力比门控全开）。
/// - `Ratio`：攻方战力 ≥ `attack_threshold` × 守方战力才攻占（`attack_threshold` 为战力比，体现「强舰队才攻占」梯度）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiegeMode {
    /// 预测胜即攻占。
    WinOnly,
    /// 战力比门控（`attack_threshold` = 战力比阈值）。
    Ratio,
}

/// AI 自己格升级决策公式形态（易变平衡项，`balance.territory.ai.upgrade_mode`）。
///
/// - `PriorityGate`：`upgrade_priority >= 0.5` 硬阈值才升级（旧实现；0.3 的 AI 永不升级）。
/// - `GoldBuffer`：金 ≥ 升级费 × (1.0 + `upgrade_priority`) 才升级（金富余门控；`upgrade_priority` 越高留缓冲越大、越克制）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeMode {
    /// 优先度硬阈值门控。
    PriorityGate,
    /// 金富余门控。
    GoldBuffer,
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
    /// 攻占决策公式形态（`SiegeMode`，数据驱动）。
    pub siege_mode: SiegeMode,
    /// 升级决策公式形态（`UpgradeMode`，数据驱动）。
    pub upgrade_mode: UpgradeMode,
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
        maintenance_per_lap: raw.maintenance_per_lap,
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
            // 缺省 = 现行形态（战力比门控 + 金富余门控，A-02/B-02 修复行为）；数据可换 win_only/priority_gate。
            siege_mode: match raw.ai.siege_mode.as_str() {
                "win_only" => SiegeMode::WinOnly,
                _ => SiegeMode::Ratio,
            },
            upgrade_mode: match raw.ai.upgrade_mode.as_str() {
                "priority_gate" => UpgradeMode::PriorityGate,
                _ => UpgradeMode::GoldBuffer,
            },
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
