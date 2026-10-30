//! 经济/战斗/贸易/建造/卡牌 配置：`Raw*` serde 中间层 + `balance()` 缓存入口。
//! 拆分自 `data.rs`（S11）；顶层 re-export `balance`，外部 API 不变。
//!
//! `convert` 留在 `data/mod.rs`（引用同文件 cfg struct，不跨模块）。

use serde::Deserialize;

use std::collections::BTreeMap;
use std::sync::OnceLock;

use super::{BALANCE_JSON, Balance, LoadError, convert, parse};

#[derive(Deserialize)]
pub(crate) struct RawBalance {
    pub(crate) economy: RawEconomy,
    pub(crate) combat: RawCombat,
    pub(crate) trade: RawTrade,
    pub(crate) build: RawBuild,
    #[serde(default)]
    pub(crate) boat: RawBoat,
    pub(crate) cards: RawCards,
    #[serde(default)]
    pub(crate) board: RawBoard,
    /// 赛道 9 分区 → 主题色令牌表（飞轮 R1 ①-01 数据化：`chrome::group_color_token` 主源；
    /// 缺省/空 = 代码 9 令牌 crash-safe 回退）。
    #[serde(default)]
    pub(crate) board_group_tokens: Vec<String>,
    /// 领地系统参数（S28.3：`docs/business/territory.md` §3/§4.1）。
    #[serde(default)]
    pub(crate) territory: RawTerritory,
    #[serde(default)]
    pub(crate) messages: RawMessages,
    #[serde(default)]
    pub(crate) selection: RawSelection,
    pub(crate) save: RawSave,
    pub(crate) ui: RawUi,
}

/// 消息界面 serde 形状（`balance.messages`，缺省 0 页 → 运行时回退 3）。
#[derive(Deserialize, Default)]
pub(crate) struct RawMessages {
    #[serde(default)]
    pub(crate) pages: BTreeMap<String, u32>,
}

/// 选择界面 serde 形状（`balance.selection`，缺省 5 角色 / 3 地图）。
///
/// `balance.selection` 真实形状是 `{characters:[...], maps:[...]}`（目录），
/// 这里读 `character_count`/`map_count` 数字字段（缺省兜底）；真实角色/地图目录由
/// `selection.rs` 直接读 `balance.selection`（S8.3.e 落地），本处只兜底计数。
#[derive(Deserialize, Default)]
pub(crate) struct RawSelection {
    #[serde(default = "default_char_count")]
    pub(crate) character_count: u32,
    #[serde(default = "default_map_count")]
    pub(crate) map_count: u32,
    /// 角色目录（`id`/`index`）。
    #[serde(default)]
    pub(crate) characters: Vec<RawCharacter>,
    /// 地图目录（`id`/`index`/`grid_w`/`grid_h`/`boss_tier`/`player_start`/`landmarks`）。
    #[serde(default)]
    pub(crate) maps: Vec<RawMapEntry>,
}

/// 角色目录条目（`balance.selection.characters[]`）。
#[derive(Deserialize, Default, Clone)]
pub(crate) struct RawCharacter {
    pub(crate) id: String,
    pub(crate) index: u32,
}

/// 地图目录条目（`balance.selection.maps[]`）。
#[derive(Deserialize, Clone)]
pub(crate) struct RawMapEntry {
    pub(crate) id: String,
    pub(crate) index: u32,
    pub(crate) grid_w: u32,
    pub(crate) grid_h: u32,
    pub(crate) boss_tier: u32,
    pub(crate) player_start: [u32; 2],
    pub(crate) landmarks: Vec<[u32; 2]>,
    /// BOSS 格 `(row, col)`（缺省 = 无 BOSS 格，`progression.md` 落格结算用）。
    #[serde(default)]
    pub(crate) boss_tile: Option<[u32; 2]>,
}

fn default_char_count() -> u32 {
    5
}

fn default_map_count() -> u32 {
    3
}

#[derive(Deserialize)]
pub(crate) struct RawEconomy {
    pub(crate) gold_max: u32,
    pub(crate) bank_rate: f32,
    pub(crate) initial_gold: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawCombat {
    pub(crate) player_first: bool,
    pub(crate) max_rounds: u32,
    #[serde(default)]
    pub(crate) player: Option<RawPlayerUnit>,
    pub(crate) damage: RawDamage,
    pub(crate) boss_tiers: Vec<RawBoss>,
    /// 防御动作的反击减伤百分比（S27.7；缺省 50）。
    #[serde(default = "default_defend_reduction")]
    pub(crate) defend_reduction_pct: u32,
    /// 战斗道具表（S27.7；缺省空表 = 无道具可用）。
    #[serde(default)]
    pub(crate) items: Vec<RawCombatItem>,
}

/// `defend_reduction_pct` 缺省值（与 `docs/business/combat.md` §战斗动作一致）。
pub(crate) fn default_defend_reduction() -> u32 {
    50
}

/// 战斗道具条目（`balance.combat.items[]`，S27.7）。
#[derive(Deserialize)]
pub(crate) struct RawCombatItem {
    pub(crate) id: String,
    pub(crate) name_key: String,
    /// 携带上限（件）。
    #[serde(default)]
    pub(crate) carry: u32,
    pub(crate) effect: RawItemEffect,
}

/// 道具效果 serde 形状（`kind` 决定用哪个字段，S27.7）。
#[derive(Deserialize)]
pub(crate) struct RawItemEffect {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) value: u32,
    #[serde(default)]
    pub(crate) rounds: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawPlayerUnit {
    pub(crate) hp: u32,
    pub(crate) atk: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawDamage {
    #[allow(dead_code)]
    pub(crate) model: String,
    pub(crate) flat: u32,
    pub(crate) atk_coef: f32,
    pub(crate) rng_span: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawBoss {
    pub(crate) tier: u32,
    pub(crate) hp: u32,
    pub(crate) atk: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawTrade {
    pub(crate) markup: f32,
    pub(crate) goods: Vec<RawGoods>,
}

#[derive(Deserialize)]
pub(crate) struct RawGoods {
    pub(crate) id: u32,
    pub(crate) name_key: String,
    pub(crate) buy: u32,
    pub(crate) qty_max: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawBuild {
    pub(crate) max_level: u32,
    pub(crate) buildings: Vec<RawBuilding>,
}

#[derive(Deserialize)]
pub(crate) struct RawBuilding {
    pub(crate) id: String,
    pub(crate) name_key: String,
    pub(crate) base_cost: u32,
    pub(crate) gold_max: u32,
    /// 每级战斗 atk 加成（成长机制，`docs/business/combat.md` §建造加成；缺省 0）。
    #[serde(default)]
    pub(crate) atk_bonus: u32,
    /// 每级战斗 hp 加成（成长机制，缺省 0）。
    #[serde(default)]
    pub(crate) hp_bonus: u32,
}

/// 船系统 serde 形状（`balance.boat`；规格 = `docs/business/economy.md` §船系统）。
#[derive(Deserialize, Default)]
pub(crate) struct RawBoat {
    #[serde(default)]
    pub(crate) levels: Vec<RawBoatLevel>,
}

/// 船等级条目 serde 形状（`balance.boat.levels[]`）。
#[derive(Deserialize)]
pub(crate) struct RawBoatLevel {
    pub(crate) level: u32,
    pub(crate) name_key: String,
    /// 升到本级的造价（1 级 = 0，开局默认）。
    pub(crate) cost: u32,
    /// 货舱容量（= 单次贸易数量上限）。
    pub(crate) cargo: u32,
    /// 战斗 atk 加成（累计值，直接加）。
    pub(crate) atk: u32,
    /// 战斗 hp 加成（累计值，直接加）。
    pub(crate) hp: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawCards {
    pub(crate) event_chance: f32,
    pub(crate) hand_size: u32,
    pub(crate) deck: Vec<RawCard>,
}

/// 赛道参数 serde 形状（`balance.board`）。
#[derive(Deserialize, Default)]
pub(crate) struct RawBoard {
    #[serde(default = "default_dice_sides")]
    pub(crate) dice_sides: u32,
    #[serde(default = "default_dice_sides")]
    pub(crate) max_steps: u32,
    #[serde(default)]
    pub(crate) lap_bonus_gold: u32,
    #[serde(default)]
    pub(crate) battle_damage: u32,
    #[serde(default)]
    pub(crate) battle_gold_loss: u32,
    #[serde(default)]
    pub(crate) treasure_gold: u32,
    #[serde(default)]
    pub(crate) city_gold: u32,
    /// BOSS 现身所需绕岛圈数（S19.3；缺省 1 = 绕满 1 圈才现身）。
    #[serde(default = "default_boss_min_laps")]
    pub(crate) boss_min_laps: u32,
}

fn default_dice_sides() -> u32 {
    6
}

fn default_boss_min_laps() -> u32 {
    1
}

#[derive(Deserialize)]
pub(crate) struct RawCard {
    pub(crate) id: u32,
    pub(crate) name_key: String,
    pub(crate) weight: u32,
    pub(crate) effect: RawEffect,
}

#[derive(Deserialize)]
pub(crate) struct RawEffect {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) value: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawSave {
    pub(crate) slot_count: u32,
    pub(crate) magic: u32,
    pub(crate) v2_size: u32,
}

#[derive(Deserialize)]
pub(crate) struct RawUi {
    pub(crate) logical_width: u32,
    pub(crate) logical_height: u32,
    pub(crate) glyph_advance: u32,
    pub(crate) max_back_steps: u32,
}

// ------------------------------------------------------------------- 转换

static BALANCE: OnceLock<Balance> = OnceLock::new();

/// 加载平衡数值表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 必填段缺失 / 数值越界时返回 [`LoadError`]（不 fallback 默认值）。
pub fn balance() -> Result<&'static Balance, LoadError> {
    if let Some(v) = BALANCE.get() {
        return Ok(v);
    }
    let raw: RawBalance = parse(BALANCE_JSON, "balance.json")?;
    let val = convert(raw)?;
    Ok(BALANCE.get_or_init(|| val))
}

/// 领地系统 serde 形状（`balance.territory`，S28.3；缺省 = 代码回退值）。
#[derive(Deserialize, Default, Clone)]
pub(crate) struct RawTerritory {
    /// 加固等级全局上限（§3 表按原型可降）。
    #[serde(default = "default_max_level")]
    pub(crate) max_level: u32,
    /// 过路费每级系数（`toll(T,L) = toll_base * (1 + toll_per_level * L)`）。
    #[serde(default)]
    pub(crate) toll_per_level: f32,
    /// 加固成本每级系数。
    #[serde(default)]
    pub(crate) upgrade_cost_per_level: f32,
    /// 产出每级系数。
    #[serde(default)]
    pub(crate) production_per_level: f32,
    /// 守军每级系数。
    #[serde(default)]
    pub(crate) defense_per_level: f32,
    /// 首占奖励（金）。
    #[serde(default)]
    pub(crate) first_capture_bonus: u32,
    /// 副本守军 atk 系数（`atk = defense × 系数`，S28.6 副本战斗；缺省 0.5）。
    #[serde(default = "default_dungeon_attack_coef")]
    pub(crate) dungeon_attack_coef: f32,
    /// 据点岛升级参数（与领地加固同公式族）。
    #[serde(default)]
    pub(crate) home_upgrade: RawHomeUpgrade,
    /// 10 类岛 → 主题色令牌（棋盘分区色带 / 副本氛围 / 据点 HUD 三处同源）。
    #[serde(default)]
    pub(crate) isle_tokens: Vec<String>,
    /// AI 席位策略参数（§6）。
    #[serde(default)]
    pub(crate) ai: RawTerritoryAi,
    /// 10 类岛原型（§3 表）。
    #[serde(default)]
    pub(crate) prototypes: Vec<RawTerritoryPrototype>,
}

/// 据点岛升级 serde 形状（`balance.territory.home_upgrade`）。
#[derive(Deserialize, Default, Clone)]
pub(crate) struct RawHomeUpgrade {
    /// 基础成本（金）。
    #[serde(default)]
    pub(crate) base_cost: u32,
    /// 每级成本系数。
    #[serde(default)]
    pub(crate) per_level: f32,
    /// 等级上限。
    #[serde(default)]
    pub(crate) max_level: u32,
}

/// AI 策略参数 serde 形状（`balance.territory.ai`）。
#[derive(Deserialize, Default, Clone)]
pub(crate) struct RawTerritoryAi {
    /// 攻占阈值：对方领地防守强度低于此值 ⇒ 选择攻占（否则交钱）。
    #[serde(default)]
    pub(crate) attack_threshold: f32,
    /// 交钱偏好（0..1，越高越倾向交钱）。
    #[serde(default)]
    pub(crate) toll_aversion: f32,
    /// 升级优先阈值：自己领地收益低于此值 ⇒ 优先升级。
    #[serde(default)]
    pub(crate) upgrade_priority: f32,
}

/// 单个岛原型 serde 形状（`balance.territory.prototypes[]`）。
#[derive(Deserialize, Default, Clone)]
pub(crate) struct RawTerritoryPrototype {
    /// 原型 id（`harbour` … `boss_fort`）。
    pub(crate) id: String,
    /// 岛名 i18n 键（`isle.<id>`）。
    #[serde(default)]
    pub(crate) name_key: String,
    /// 专属副本场景 id（`raid_beach` … `fort_boss`）。
    #[serde(default)]
    pub(crate) dungeon_id: String,
    /// 副本名 i18n 键（`dungeon.<id>`）。
    #[serde(default)]
    pub(crate) dungeon_name_key: String,
    /// 守军名（显示用）。
    #[serde(default)]
    pub(crate) guard: String,
    /// 占领后产出语义（显示用）。
    #[serde(default)]
    pub(crate) benefit: String,
    /// 加固等级上限（§3 表）。
    #[serde(default)]
    pub(crate) max_level: u32,
    /// 过路费基值。
    #[serde(default)]
    pub(crate) toll_base: u32,
    /// 加固成本基值。
    #[serde(default)]
    pub(crate) upgrade_cost_base: u32,
    /// 每圈产出基值。
    #[serde(default)]
    pub(crate) production_base: u32,
    /// 守军基值（战斗 HP 基线）。
    #[serde(default)]
    pub(crate) defense_base: u32,
}

fn default_max_level() -> u32 {
    3
}

fn default_dungeon_attack_coef() -> f32 {
    0.5
}
