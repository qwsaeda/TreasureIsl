//! S15.1 穷举场景数据表：`assets/data/exhaust_scenarios.json` → 类型化结构（`OnceLock` 缓存 + 解析失败硬错误）。
//!
//! 用途：业务场景穷举测试（`crates/ti-core/tests/exhaust_*.rs`）的**枚举空间数据源**——
//! 组合维度（seed/qty/level/字段档位）全部由本表驱动，测试代码只做"加载 + 逐条断言"薄壳，
//! 加/改枚举维度 = 改 JSON 一行，不改测试代码（`docs/process/02_conventions.md` §1 数值入 JSON 纪律）。
//!
//! 本表与 `balance.json` 正交：`balance.json` 是**玩法数值**单一事实源，本表是**测试枚举空间**
//! 单一事实源，两者都不进 `check_hardcode` 扫描范围（gate 第 10c 项按路径白名单豁免，见下方说明）。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{LoadError, parse};

/// 穷举场景表（编译期内联，保证 wasm 不依赖运行时文件系统）。
pub(crate) const EXHAUST_JSON: &str =
    include_str!("../../../../assets/data/exhaust_scenarios.json");

/// 战斗域穷举空间。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustCombat {
    /// 种子档位。
    pub seeds: Vec<u32>,
    /// 回合上限档位。
    pub max_rounds_levels: Vec<u32>,
    /// 玩家属性档（hp/atk 组合）。
    pub player_variants: Vec<ExhaustUnit>,
    /// 越界档检查用的种子子集。
    pub oob_seed_check: Vec<u32>,
    /// `max_rounds=1` 边界用例（高 atk 必首回合打死 BOSS）。
    pub max_rounds_one_case: ExhaustCase,
    /// `max_rounds=1` 边界用例（atk 恰好等于档 0 BOSS hp，验证"首回合归零"精确值）。
    pub max_rounds_one_case_exact: ExhaustCase,
    /// 夹具隔离假值（不读 `balance` 的独立配置，双轨交叉验证用）。
    pub fixture: ExhaustCombatFixture,
}

/// 玩家属性档。
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExhaustUnit {
    /// HP。
    pub hp: u32,
    /// 攻击力。
    pub atk: u32,
}

/// 单点用例（seed + 玩家属性 + 目标 BOSS 档）。
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExhaustCase {
    /// RNG 种子。
    pub seed: u32,
    /// 玩家初始 HP。
    pub player_hp: u32,
    /// 玩家攻击力。
    pub player_atk: u32,
    /// 目标 BOSS 档。
    pub boss_tier: u32,
}

/// 战斗夹具隔离配置。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustCombatFixture {
    /// 夹具 BOSS 档（假值，与真实表数值无关）。
    pub boss_tiers: Vec<ExhaustBossTier>,
    /// 夹具种子档位。
    pub seeds: Vec<u32>,
    /// 夹具回合上限档位。
    pub max_rounds_levels: Vec<u32>,
    /// 夹具玩家属性（单档，跨 tier 复用）。
    pub player_hp: u32,
    pub player_atk: u32,
}

/// 夹具 BOSS 档（假值）。
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExhaustBossTier {
    /// 档位号。
    pub tier: u32,
    /// 初始 HP。
    pub hp: u32,
    /// 攻击力。
    pub atk: u32,
}

/// 经济域穷举空间。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustTrade {
    /// `qty` 相对 `qty_max` 的偏移档位（对全 5 商品统一套用，逐商品取自己的 `qty_max` 再偏移）。
    pub qty_offset_levels: Vec<i32>,
    /// 偏移档位说明（与 `qty_offset_levels` 等长，仅用于测试输出报告，不参与计算）。
    pub qty_offset_labels: Vec<String>,
    /// `gold` 相对 `price` 的偏移档位。
    pub gold_offset_levels: Vec<i32>,
    /// 偏移档位说明（与 `gold_offset_levels` 等长，仅用于测试输出报告，不参与计算）。
    pub gold_offset_labels: Vec<String>,
    /// 夹具隔离假值贸易配置。
    pub fixture: ExhaustTradeFixture,
}

/// 贸易夹具隔离配置。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustTradeFixture {
    /// 售价系数（假值）。
    pub markup: f32,
    /// 商品目录（假值）。
    pub goods: Vec<ExhaustGoods>,
    /// 金量上限（假值）。
    pub gold_cap: u32,
    /// 逐用例对账。
    pub cases: Vec<ExhaustTradeCase>,
    /// 无上限交叉验证用例。
    pub unlimited_case: ExhaustTradeCase,
}

/// 贸易夹具商品（假值）。
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExhaustGoods {
    /// 商品 id。
    pub id: u32,
    /// 买入单价。
    pub buy: u32,
    /// 持有上限。
    pub qty_max: u32,
}

/// 贸易夹具单点用例。
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExhaustTradeCase {
    /// 商品 id。
    pub gid: u32,
    /// 持有量。
    pub owned: u32,
    /// 初始金币。
    pub gold: u32,
    /// 卖出单价（独立手算值）。
    pub sell_unit: u32,
}

/// 建造域穷举空间。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustBuild {
    /// 夹具隔离假值建造配置。
    pub fixture: ExhaustBuildFixture,
}

/// 建造夹具隔离配置。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustBuildFixture {
    /// 建筑等级上限（假值）。
    pub max_level: u32,
    /// 建筑目录（假值）。
    pub buildings: Vec<ExhaustBuilding>,
    /// 初始银行金量（假值）。
    pub initial_bank_gold: u32,
}

/// 建造夹具建筑（假值）。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustBuilding {
    /// 建筑 id（假值字符串）。
    pub id: String,
    /// 基础造价。
    pub base_cost: u32,
    /// 抬升的金量上限。
    pub gold_max: u32,
}

/// 存读档域穷举空间。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustSaveload {
    /// RNG 种子档位。
    pub seeds: Vec<u32>,
    /// 金币档位。
    pub gold_levels: Vec<u32>,
    /// HP 档位。
    pub hp_levels: Vec<u32>,
    /// 银行档位。
    pub bank_levels: Vec<u32>,
    /// `rng_steps` 档位（精确续流测试）。
    pub rng_step_levels: Vec<u128>,
    /// 旧 26 字节二进制 `SaveSlot` 单点用例。
    pub legacy_slot_cases: Vec<ExhaustLegacySlotCase>,
}

/// 旧 26 字节二进制 `SaveSlot` 单点用例（字段与 `domain::persistence::SaveSlot` 对齐）。
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExhaustLegacySlotCase {
    /// 版本号（v1 = 1）。
    pub version: u32,
    /// 角色索引（1 起）。
    pub character: u32,
    /// 金币。
    pub gold: u32,
    /// 地图索引（0 起）。
    pub map_slot: u32,
    /// 进度。
    pub position: u32,
    /// RNG 已消费步数。
    pub rng_steps: u128,
}

/// 菜单/消息/选择域穷举空间。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustMenu {
    /// 菜单光标测试：循环长度档位。
    pub cursor_counts: Vec<usize>,
    /// 菜单光标测试：起始位置档位。
    pub cursor_starts: Vec<usize>,
    /// 菜单项目标测试：索引档位（含越界值）。
    pub item_target_indexes: Vec<usize>,
    /// 消息侧测试：页码相对 `total` 的偏移档位。
    pub msg_page_offset_levels: Vec<i32>,
    /// 偏移档位说明（与 `msg_page_offset_levels` 等长，仅用于测试输出报告，不参与计算）。
    pub msg_page_offset_labels: Vec<String>,
}

/// 主循环 / 场景状态机域穷举空间（第 5 域，`docs/00_design.md` §1 第 4 条可断言约束
/// 「任意场景 ≤ N 次操作回主菜单」的穷举覆盖；N 读 `balance.ui.max_back_steps`，
/// 状态码 / trigger 枚举档位由本表驱动，加/改档位 = 改 JSON 一行）。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustMainLoop {
    /// 场景状态码枚举（`rules/main_loop.rs` `SceneStack::advance` 转移表覆盖的全部状态码）。
    pub states: Vec<u32>,
    /// 触发事件码枚举（`docs/business/loop_core.md` 状态码/trigger 码表的 trigger 全集）。
    pub triggers: Vec<u32>,
}

/// 进度域穷举空间（第 6 域，`docs/business/progression.md` 地图/卡牌）：
/// 地图几何（格×方向）与卡牌（事件×种子×状态）的**测试枚举维度**由本表驱动，
/// 业务几何（`selection.maps`）与卡牌目录（`balance.cards`）仍读 `balance()`。
#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct ExhaustProgression {
    /// 卡牌域 RNG 种子档位（确定性对账：同 seed + 同事件序列 ⇒ 同结果）。
    pub card_seeds: Vec<u32>,
    /// 卡牌状态档位（hp/gold 初值 + 钳制上限，验证 [`apply`](crate::rules::cards) 的
    /// `0`/`hp_max`/`gold_max` 边界）。
    pub card_state_variants: Vec<ExhaustCardState>,
    /// 卡牌事件 id 上界（`progression.md`：`event_id` 1..=25 全处理路径，`all_event_ids`）。
    pub card_event_id_max: u32,
}

/// 卡牌状态档位（与 `rules/cards.rs` `CardState` 字段对齐，夹具初值 + 钳制上限）。
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExhaustCardState {
    /// 初始 HP。
    pub hp: u32,
    /// 初始金币。
    pub gold: u32,
    /// HP 上限（`Heal` 钳制上界）。
    pub hp_max: u32,
    /// 金币上限（`Gold` 钳制上界）。
    pub gold_max: u32,
}

/// 穷举场景表顶层（`assets/data/exhaust_scenarios.json`）。
#[derive(Debug, Clone, Deserialize)]
pub struct ExhaustScenarios {
    /// 表级 schema 版本（缺省 0，防未来 schema 演进时静默读旧表）。
    #[serde(default)]
    pub schema_version: u32,
    /// 战斗域穷举空间。
    pub combat: ExhaustCombat,
    /// 经济域穷举空间。
    pub trade: ExhaustTrade,
    /// 建造域穷举空间。
    pub build: ExhaustBuild,
    /// 存读档域穷举空间。
    pub saveload: ExhaustSaveload,
    /// 菜单/消息/选择域穷举空间。
    pub menu: ExhaustMenu,
    /// 主循环 / 场景状态机域穷举空间（第 5 域）。
    pub main_loop: ExhaustMainLoop,
    /// 进度域（地图/卡牌）穷举空间（第 6 域）。
    pub progression: ExhaustProgression,
}

static EXHAUST: OnceLock<ExhaustScenarios> = OnceLock::new();

/// 加载穷举场景表（首次解析，之后命中缓存；解析失败 = 硬错误，不 fallback）。
///
/// # Errors
///
/// JSON 语法错误 / 必填段缺失 → [`LoadError`]。
pub fn exhaust() -> Result<&'static ExhaustScenarios, LoadError> {
    if let Some(v) = EXHAUST.get() {
        return Ok(v);
    }
    let val: ExhaustScenarios = parse(EXHAUST_JSON, "exhaust_scenarios.json")?;
    let _ = val.schema_version; // 保留字段，防未来 schema 演进时静默读旧表
    Ok(EXHAUST.get_or_init(|| val))
}

/// 表级自检（供 `tests/data_tables.rs` 校验用：`exhaust_scenarios_parses_and_is_populated` 断言
/// 组合数与 `exhaust()` 实测一致，防 JSON 档位改动后测试静默失配）。
#[must_use]
pub fn exhaust_key_counts() -> BTreeMap<&'static str, usize> {
    let Ok(e) = exhaust() else {
        return BTreeMap::new();
    };
    BTreeMap::from([
        ("combat.seeds", e.combat.seeds.len()),
        ("trade.qty_offset", e.trade.qty_offset_levels.len()),
        ("build.fixture.buildings", e.build.fixture.buildings.len()),
        ("saveload.seeds", e.saveload.seeds.len()),
        ("menu.item_target_indexes", e.menu.item_target_indexes.len()),
        ("main_loop.states", e.main_loop.states.len()),
        ("main_loop.triggers", e.main_loop.triggers.len()),
        ("progression.card_seeds", e.progression.card_seeds.len()),
        ("progression.card_state_variants", e.progression.card_state_variants.len()),
    ])
}
