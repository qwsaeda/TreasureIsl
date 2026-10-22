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
    pub(crate) cards: RawCards,
    #[serde(default)]
    pub(crate) board: RawBoard,
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
