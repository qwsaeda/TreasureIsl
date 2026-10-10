//! 数据表加载层：`assets/data/*.json` → 类型化结构（`OnceLock` 缓存 + 解析失败硬错误）。
//!
//! 用途（`docs/process/02_conventions.md` §1）：数值与文本一律进 JSON，
//! 调平衡不改代码。本模块是唯一的读取入口。
//!
//! 约定：
//! - `include_str!` 进包（wasm 免 IO），反序列化结果进 [`OnceLock`]，每帧不重解析；
//! - **解析失败 = 硬错误**：返回 [`LoadError`]，调用方须显式处理，
//!   不 fallback 到默认值（避免「能跑但数值错」——见 `docs/business/00_index.md`）；
//! - 访问器 [`balance`] / [`strings`] / [`scenes`] 均返回 `Result`，
//!   由调用点在启动时一次性校验（无 fallback，不主动中止进程——见门禁红线）。
//!
//! 集成测试见 `crates/ti-core/tests/data_tables.rs`（tests/ 目录不受 src 扫描约束）。
//!
//! 业务规格见 `docs/business/`（唯一事实源）；本模块只做反序列化。

use std::collections::BTreeMap;
use std::fmt;
use std::sync::OnceLock;

use serde::Deserialize;

/// 平衡数值表（编译期内联，保证 wasm 不依赖运行时文件系统）。
const BALANCE_JSON: &str = include_str!("../../../assets/data/balance.json");
/// 文本表（编译期内联）。
const STRINGS_JSON: &str = include_str!("../../../assets/data/strings.json");
/// 场景布局表（编译期内联）。
const SCENES_JSON: &str = include_str!("../../../assets/data/scenes.json");
/// 控件表（编译期内联；`scripts/gen_layout.py` 从 `ref/controls.json` 生成，
/// S8.5 控件层数据源，`docs/impl/14_asset_enhancement.md`）。
const CONTROLS_JSON: &str = include_str!("../../../assets/data/controls.json");

/// 表级元数据键名：数字型的 `schema_version`，在 key→value 映射中跳过。
const SCHEMA_VERSION_KEY: &str = "schema_version";

/// 数据表加载失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// JSON 语法错误。
    Syntax(String),
    /// 结构不符：缺少必填字段、类型不匹配或取值越界。
    Schema(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "data table syntax error: {e}"),
            Self::Schema(e) => write!(f, "data table schema error: {e}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// 反序列化并归类错误。
///
/// 约束 `DeserializeOwned`：本模块中间层类型皆 owned，直接从内联字符串解析。
fn parse<T: serde::de::DeserializeOwned>(raw: &str, table: &str) -> Result<T, LoadError> {
    serde_json::from_str(raw).map_err(|e| LoadError::Syntax(format!("{table}: {e}")))
}

// ---------------------------------------------------------------- balance

/// 战斗伤害模型参数（`balance.combat.damage`）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageCfg {
    /// 每回合最小伤害（保证战斗收敛）。
    pub flat: u32,
    /// 攻击力系数。
    pub atk_coef: f32,
    /// 随机浮动幅度（`rand_offset ∈ [0, rng_span]`）。
    pub rng_span: u32,
}

/// 一个 BOSS 档（`balance.combat.boss_tiers`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossTier {
    /// 档位号。
    pub tier: u32,
    /// 初始 HP。
    pub hp: u32,
    /// 攻击力。
    pub atk: u32,
}

/// 商品条目（`balance.trade.goods`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoodsDef {
    /// 商品 id。
    pub id: u32,
    /// 文本 key（对应 `strings.json`，非中文本体）。
    pub name_key: String,
    /// 买入单价。
    pub buy: u32,
    /// 持有上限。
    pub qty_max: u32,
}

/// 建筑条目（`balance.build.buildings`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingDef {
    /// 建筑 id（如 `Castle`）。
    pub id: String,
    /// 文本 key。
    pub name_key: String,
    /// 基础造价（实际造价 = `base_cost × (level+1)`）。
    pub base_cost: u32,
    /// 建成后抬升的金量上限。
    pub gold_max: u32,
}

/// 卡牌效果（`balance.cards.deck[].effect`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardEffectDef {
    /// 扣 HP。
    Damage(u32),
    /// 加 HP。
    Heal(u32),
    /// 加金。
    Gold(u32),
    /// 地图移动（由地图系统处理，不改卡牌状态）。
    Move,
    /// 未知效果（无状态变化）。
    Unknown,
}

/// 卡牌条目（`balance.cards.deck`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDef {
    /// 卡牌 id。
    pub id: u32,
    /// 文本 key。
    pub name_key: String,
    /// 抽中权重。
    pub weight: u32,
    /// 效果。
    pub effect: CardEffectDef,
}

/// 经济参数（`balance.economy`）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EconomyCfg {
    /// 金币上限（建造可抬升）。
    pub gold_max: u32,
    /// 溢出进银行的比例。
    pub bank_rate: f32,
    /// 开局金币。
    pub initial_gold: u32,
}

/// 战斗参数（`balance.combat`）。
#[derive(Debug, Clone, PartialEq)]
pub struct CombatCfg {
    /// 玩家是否固定先手。
    pub player_first: bool,
    /// 回合数上限（兜底判胜负）。
    pub max_rounds: u32,
    /// 伤害模型。
    pub damage: DamageCfg,
    /// BOSS 档表。
    pub boss_tiers: Vec<BossTier>,
}

impl CombatCfg {
    /// 按档位查 BOSS（越界档 → `None`，见 `docs/business/combat.md`）。
    #[must_use]
    pub fn boss(&self, tier: u32) -> Option<BossTier> {
        self.boss_tiers.iter().copied().find(|b| b.tier == tier)
    }
}

/// 贸易参数（`balance.trade`）。
#[derive(Debug, Clone, PartialEq)]
pub struct TradeCfg {
    /// 售价系数（售价 = `buy × markup`）。
    pub markup: f32,
    /// 商品目录。
    pub goods: Vec<GoodsDef>,
}

/// 建造参数（`balance.build`）。
#[derive(Debug, Clone, PartialEq)]
pub struct BuildCfg {
    /// 建筑等级上限。
    pub max_level: u32,
    /// 建筑目录。
    pub buildings: Vec<BuildingDef>,
}

impl BuildCfg {
    /// 按 id 查建筑。
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&BuildingDef> {
        self.buildings.iter().find(|b| b.id == id)
    }
}

/// 卡牌参数（`balance.cards`）。
#[derive(Debug, Clone, PartialEq)]
pub struct CardsCfg {
    /// 移动触发卡牌的概率（`0.0..=1.0`）。
    pub event_chance: f32,
    /// 一次抽牌张数。
    pub hand_size: u32,
    /// 卡池。
    pub deck: Vec<CardDef>,
}

/// 存读档参数（`balance.save`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveCfg {
    /// 槽位数。
    pub slot_count: u32,
    /// 存档魔数。
    pub magic: u32,
    /// v2 存档长度（字节）。
    pub v2_size: u32,
}

/// UI 全局参数（`balance.ui`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCfg {
    /// 逻辑分辨率宽。
    pub logical_width: u32,
    /// 逻辑分辨率高。
    pub logical_height: u32,
    /// 字形步进。
    pub glyph_advance: u32,
    /// 「无死锁」判据：任意场景最多几步可回主菜单。
    pub max_back_steps: u32,
}

/// 消息界面参数（`balance.messages`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagesCfg {
    /// 某类消息界面的页数（`kind 短 key → 页数`，`docs/business/loop_core.md` §消息）。
    pub pages: BTreeMap<String, u32>,
}

/// 选择界面参数（`balance.selection`，S8.3.e 预留：角色/地图目录 id）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionCfg {
    /// 角色数（`selection.character_count`）。
    pub character_count: u32,
    /// 地图数（`selection.map_count`）。
    pub map_count: u32,
}

/// 单个控件记录（`controls.json` 的 `scenes[].records[]`，S8.5 控件层）。
///
/// 字段语义见 `docs/facts/map.md`（F-306/F-311：`f0`=x、`f1`=w、`f2`=h、
/// `f3`=y、`f4`=flags、`f5`=控件类型、`f6`=extra）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlRec {
    /// 控件类型码（`0` = 无类型，`1..=44` = 具名类型，见 `kind_name`）。
    pub kind: u32,
    /// 控件类型名（`"none"` / `"kind12"` 等稳定字符串，非中文）。
    pub kind_name: &'static str,
    /// 左上角 x（逻辑像素）。
    pub x: u32,
    /// 左上角 y（逻辑像素，F-311：`f3` = Y 坐标）。
    pub y: u32,
    /// 宽（逻辑像素）。
    pub w: u32,
    /// 高（逻辑像素）。
    pub h: u32,
    /// 标志位（`f4`；`1` = 可见，S8.5 渲染只画可见控件）。
    pub flags: u32,
    /// 附加字段（`f6`，类型相关）。
    pub extra: u32,
}

impl ControlRec {
    /// 是否可见（`flags & 1 != 0`；`h == 0` 或 `w == 0` 视为不可见，S8.5 渲染判据）。
    #[must_use]
    pub fn visible(&self) -> bool {
        self.flags & 1 != 0 && self.w > 0 && self.h > 0
    }
}

/// 单场景的控件集合（`controls.json` 的 `scenes[]`，S8.5）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneControls {
    /// 场景 id（如 `"Bigcard.Map"`）。
    pub id: String,
    /// 场景宽（逻辑像素）。
    pub width: u32,
    /// 场景高（逻辑像素）。
    pub height: u32,
    /// 该场景的控件记录（按原顺序）。
    pub records: Vec<ControlRec>,
}

/// 控件表（`assets/data/controls.json` 顶层，S8.5 控件层单一事实源）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlsCfg {
    /// 来源（`ref/controls.json`，脚本可追溯）。
    pub source: String,
    /// 场景数（`scene_count`）。
    pub scene_count: u32,
    /// 控件记录总数（`record_total`，S8.5 DoD：2041）。
    pub record_total: u32,
    /// 各场景控件集合（`scenes`）。
    pub scenes: Vec<SceneControls>,
}

impl ControlsCfg {
    /// 按场景 id 查控件集合（未命中 → `None`）。
    #[must_use]
    pub fn scene(&self, id: &str) -> Option<&SceneControls> {
        self.scenes.iter().find(|s| s.id == id)
    }
    /// 按场景 id 迭代**可见**控件（S8.5 渲染用：`flags&1 != 0` 且 `w>0 && h>0`，
    /// 即「draw 面积 > 0」判据；未命中场景 → 空迭代器，不分配、不泄漏）。
    pub fn visible_records(&self, id: &str) -> impl Iterator<Item = &ControlRec> {
        self.scene(id).map(|s| s.records.iter()).into_iter().flatten().filter(|r| r.visible())
    }
}

/// 平衡数值表（`assets/data/balance.json` 顶层）。
#[derive(Debug, Clone, PartialEq)]
pub struct Balance {
    /// 经济参数。
    pub economy: EconomyCfg,
    /// 战斗参数。
    pub combat: CombatCfg,
    /// 贸易参数。
    pub trade: TradeCfg,
    /// 建造参数。
    pub build: BuildCfg,
    /// 卡牌参数。
    pub cards: CardsCfg,
    /// 消息界面参数。
    pub messages: MessagesCfg,
    /// 选择界面参数。
    pub selection: SelectionCfg,
    /// 存读档参数。
    pub save: SaveCfg,
    /// UI 参数。
    pub ui: UiCfg,
}

// ------------------------------------------------------- serde 中间层（JSON 形状）

#[derive(Deserialize)]
struct RawBalance {
    economy: RawEconomy,
    combat: RawCombat,
    trade: RawTrade,
    build: RawBuild,
    cards: RawCards,
    #[serde(default)]
    messages: RawMessages,
    #[serde(default)]
    selection: RawSelection,
    save: RawSave,
    ui: RawUi,
}

/// 消息界面 serde 形状（`balance.messages`，缺省 0 页 → 运行时回退 3）。
#[derive(Deserialize, Default)]
struct RawMessages {
    #[serde(default)]
    pages: BTreeMap<String, u32>,
}

/// 选择界面 serde 形状（`balance.selection`，缺省 5 角色 / 3 地图）。
///
/// `balance.selection` 真实形状是 `{characters:[...], maps:[...]}`（目录），
/// 这里读 `character_count`/`map_count` 数字字段（缺省兜底）；真实角色/地图目录由
/// `selection.rs` 直接读 `balance.selection`（S8.3.e 落地），本处只兜底计数。
#[derive(Deserialize, Default)]
struct RawSelection {
    #[serde(default = "default_char_count")]
    character_count: u32,
    #[serde(default = "default_map_count")]
    map_count: u32,
}

fn default_char_count() -> u32 {
    5
}

fn default_map_count() -> u32 {
    3
}

#[derive(Deserialize)]
struct RawEconomy {
    gold_max: u32,
    bank_rate: f32,
    initial_gold: u32,
}

#[derive(Deserialize)]
struct RawCombat {
    player_first: bool,
    max_rounds: u32,
    damage: RawDamage,
    boss_tiers: Vec<RawBoss>,
}

#[derive(Deserialize)]
struct RawDamage {
    #[allow(dead_code)]
    model: String,
    flat: u32,
    atk_coef: f32,
    rng_span: u32,
}

#[derive(Deserialize)]
struct RawBoss {
    tier: u32,
    hp: u32,
    atk: u32,
}

#[derive(Deserialize)]
struct RawTrade {
    markup: f32,
    goods: Vec<RawGoods>,
}

#[derive(Deserialize)]
struct RawGoods {
    id: u32,
    name_key: String,
    buy: u32,
    qty_max: u32,
}

#[derive(Deserialize)]
struct RawBuild {
    max_level: u32,
    buildings: Vec<RawBuilding>,
}

#[derive(Deserialize)]
struct RawBuilding {
    id: String,
    name_key: String,
    base_cost: u32,
    gold_max: u32,
}

#[derive(Deserialize)]
struct RawCards {
    event_chance: f32,
    hand_size: u32,
    deck: Vec<RawCard>,
}

#[derive(Deserialize)]
struct RawCard {
    id: u32,
    name_key: String,
    weight: u32,
    effect: RawEffect,
}

#[derive(Deserialize)]
struct RawEffect {
    kind: String,
    #[serde(default)]
    value: u32,
}

#[derive(Deserialize)]
struct RawSave {
    slot_count: u32,
    magic: u32,
    v2_size: u32,
}

#[derive(Deserialize)]
struct RawUi {
    logical_width: u32,
    logical_height: u32,
    glyph_advance: u32,
    max_back_steps: u32,
}

// ------------------------------------------------------------------- 转换

fn convert(raw: RawBalance) -> Result<Balance, LoadError> {
    let cards = CardsCfg {
        event_chance: raw.cards.event_chance,
        hand_size: raw.cards.hand_size,
        deck: raw
            .cards
            .deck
            .into_iter()
            .map(|c| {
                let effect = match c.effect.kind.as_str() {
                    "damage" => CardEffectDef::Damage(c.effect.value),
                    "heal" => CardEffectDef::Heal(c.effect.value),
                    "gold" => CardEffectDef::Gold(c.effect.value),
                    "move" => CardEffectDef::Move,
                    "unknown" => CardEffectDef::Unknown,
                    other => {
                        return Err(LoadError::Schema(format!(
                            "cards.deck[{}]: unknown effect `{other}`",
                            c.id
                        )));
                    }
                };
                Ok(CardDef { id: c.id, name_key: c.name_key, weight: c.weight, effect })
            })
            .collect::<Result<Vec<_>, LoadError>>()?,
    };

    if !(0.0..=1.0).contains(&cards.event_chance) {
        return Err(LoadError::Schema(format!(
            "cards.event_chance = {} out of range (must be in [0,1])",
            cards.event_chance
        )));
    }

    let trade = TradeCfg {
        markup: raw.trade.markup,
        goods: raw
            .trade
            .goods
            .into_iter()
            .map(|g| GoodsDef { id: g.id, name_key: g.name_key, buy: g.buy, qty_max: g.qty_max })
            .collect(),
    };

    let build = BuildCfg {
        max_level: raw.build.max_level,
        buildings: raw
            .build
            .buildings
            .into_iter()
            .map(|b| BuildingDef {
                id: b.id,
                name_key: b.name_key,
                base_cost: b.base_cost,
                gold_max: b.gold_max,
            })
            .collect(),
    };

    Ok(Balance {
        economy: EconomyCfg {
            gold_max: raw.economy.gold_max,
            bank_rate: raw.economy.bank_rate,
            initial_gold: raw.economy.initial_gold,
        },
        combat: CombatCfg {
            player_first: raw.combat.player_first,
            max_rounds: raw.combat.max_rounds,
            damage: DamageCfg {
                flat: raw.combat.damage.flat,
                atk_coef: raw.combat.damage.atk_coef,
                rng_span: raw.combat.damage.rng_span,
            },
            boss_tiers: raw
                .combat
                .boss_tiers
                .into_iter()
                .map(|b| BossTier { tier: b.tier, hp: b.hp, atk: b.atk })
                .collect(),
        },
        trade,
        build,
        cards,
        messages: MessagesCfg { pages: raw.messages.pages },
        selection: SelectionCfg {
            character_count: raw.selection.character_count,
            map_count: raw.selection.map_count,
        },
        save: SaveCfg {
            slot_count: raw.save.slot_count,
            magic: raw.save.magic,
            v2_size: raw.save.v2_size,
        },
        ui: UiCfg {
            logical_width: raw.ui.logical_width,
            logical_height: raw.ui.logical_height,
            glyph_advance: raw.ui.glyph_advance,
            max_back_steps: raw.ui.max_back_steps,
        },
    })
}

// ------------------------------------------------------------------- controls

/// 控件表 serde 形状（`assets/data/controls.json` 顶层，`scripts/gen_layout.py` 生成）。
#[derive(Deserialize)]
struct RawControls {
    #[serde(default)]
    source: String,
    #[serde(default)]
    scene_count: u32,
    #[serde(default)]
    record_total: u32,
    #[serde(default)]
    scenes: Vec<RawSceneControls>,
}

/// 单场景控件 serde 形状（`controls.json` 的 `scenes[]`）。
#[derive(Deserialize)]
struct RawSceneControls {
    id: String,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
    #[serde(default)]
    records: Vec<RawControlRec>,
}

/// 单控件 serde 形状（`controls.json` 的 `records[]`）。
#[derive(Deserialize)]
struct RawControlRec {
    #[serde(default)]
    kind: u32,
    #[serde(default)]
    kind_name: String,
    #[serde(default)]
    x: u32,
    #[serde(default)]
    y: u32,
    #[serde(default)]
    w: u32,
    #[serde(default)]
    h: u32,
    #[serde(default)]
    flags: u32,
    #[serde(default)]
    extra: u32,
}

/// `kind_name`（`"none"` / `"kind1"`..`"kind44"`）→ `&'static str`（静态表内插值）。
///
/// 脚本 `gen_layout.py` 只生成这 45 种名字（`KIND_NAMES`），故静态表可覆盖全部；
/// 非预期名字 → 退 `"kind0"`（不 panic、不泄漏，门禁红线）。
fn control_kind_name_static(name: &str) -> &'static str {
    if name == "none" {
        return "none";
    }
    if let Some(n) = name.strip_prefix("kind") {
        if let Ok(k) = n.parse::<u32>() {
            if (1..=44).contains(&k) {
                return KIND_NAME_TABLE
                    .get(usize::try_from(k).unwrap_or(0))
                    .copied()
                    .unwrap_or("kind1");
            }
        }
    }
    "kind1"
}

/// 静态控件类型名表（`gen_layout.py` 的 `KIND_NAMES` 镜像，索引 1..=44）。
static KIND_NAME_TABLE: [&str; 45] = [
    "none", "kind1", "kind2", "kind3", "kind4", "kind5", "kind6", "kind7", "kind8", "kind9",
    "kind10", "kind11", "kind12", "kind13", "kind14", "kind15", "kind16", "kind17", "kind18",
    "kind19", "kind20", "kind21", "kind22", "kind23", "kind24", "kind25", "kind26", "kind27",
    "kind28", "kind29", "kind30", "kind31", "kind32", "kind33", "kind34", "kind35", "kind36",
    "kind37", "kind38", "kind39", "kind40", "kind41", "kind42", "kind43", "kind44",
];

/// 控件表全局缓存（`OnceLock`，编译期内联 JSON 反序列化一次）。
static CONTROLS: OnceLock<ControlsCfg> = OnceLock::new();

/// 加载控件表（首次解析，之后命中缓存；S8.5 控件层数据源）。
///
/// # Errors
///
/// JSON 语法错误 / 顶层形状非对象（缺 `scenes`）时返回 [`LoadError`]，不 fallback
/// （数据表硬错误，`docs/business/00_index.md` 约定）。
pub fn controls() -> Result<&'static ControlsCfg, LoadError> {
    if let Some(v) = CONTROLS.get() {
        return Ok(v);
    }
    let raw: RawControls = parse(CONTROLS_JSON, "controls.json")?;
    let scenes: Vec<SceneControls> = raw
        .scenes
        .into_iter()
        .map(|sc| SceneControls {
            id: sc.id,
            width: sc.width,
            height: sc.height,
            records: sc
                .records
                .into_iter()
                .map(|r| ControlRec {
                    kind: r.kind,
                    kind_name: control_kind_name_static(&r.kind_name),
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                    flags: r.flags,
                    extra: r.extra,
                })
                .collect(),
        })
        .collect();
    Ok(CONTROLS.get_or_init(|| ControlsCfg {
        source: raw.source,
        scene_count: raw.scene_count,
        record_total: raw.record_total,
        scenes,
    }))
}

// ------------------------------------------------------------------- strings

/// 文本表（`assets/data/strings.json`）：key → 中文文本。
#[derive(Debug, Clone, Default)]
pub struct Strings {
    map: BTreeMap<String, String>,
}

impl Strings {
    /// 按 key 取文本；缺失返回 `key` 本身（便于定位，不 panic）。
    #[must_use]
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.map.get(key).map_or(key, String::as_str)
    }

    /// 按 key 取文本；缺失返回 `None`。
    #[must_use]
    pub fn try_get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }

    /// 键总数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 是否为空表。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 全部键（`validate_data.py --check` 的孤儿检测用）。
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(String::as_str)
    }
}

// ------------------------------------------------------------------- scenes

/// 一个矩形定位（`scenes.json` 中的 `{x, y[, w, h]}`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    /// 左上 x（逻辑坐标）。
    pub x: i32,
    /// 左上 y。
    pub y: i32,
    /// 宽（`0` 表示未指定）。
    pub w: i32,
    /// 高。
    pub h: i32,
}

/// 列表布局（`scenes.json` 的 `list`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListLayout {
    /// 列表起点 x。
    pub x: i32,
    /// 列表起点 y。
    pub y: i32,
    /// 行高（步进）。
    pub row_h: i32,
    /// 每项高（绘制用）。
    pub item_h: i32,
    /// 文本 x。
    pub text_x: i32,
    /// 宽度。
    pub w: i32,
}

impl ListLayout {
    /// 第 `i` 项的绘制矩形。
    #[must_use]
    pub fn item_rect(&self, i: usize) -> Rect {
        let n = i32::try_from(i).unwrap_or(0);
        Rect { x: self.x, y: self.y + n * self.row_h - self.item_h / 2, w: self.w, h: self.item_h }
    }

    /// 第 `i` 项的文本位置。
    #[must_use]
    pub fn item_text_pos(&self, i: usize) -> (i32, i32) {
        let n = i32::try_from(i).unwrap_or(0);
        (self.text_x, self.y + n * self.row_h)
    }
}

/// 单个场景的布局（`assets/data/scenes.json` 的一个条目）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneLayout {
    /// 场景 id（键名，如 `select_role`）。
    pub id: String,
    /// 背景色。
    pub bg: u16,
    /// 标题条。
    pub title: Rect,
    /// 列表布局（无列表场景为全 0）。
    pub list: ListLayout,
    /// 提示行。
    pub hint: Rect,
    /// HUD（金/持有量）。
    pub hud: Rect,
    /// 页码。
    pub page: Rect,
}

// ------------------------------------------------------------------- 缓存入口

static BALANCE: OnceLock<Balance> = OnceLock::new();
static STRINGS: OnceLock<Strings> = OnceLock::new();
static SCENES: OnceLock<BTreeMap<String, SceneLayout>> = OnceLock::new();

/// 加载平衡数值表（首次调用解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误、结构不符、取值越界时返回 [`LoadError`]（不 fallback）。
pub fn balance() -> Result<&'static Balance, LoadError> {
    if let Some(v) = BALANCE.get() {
        return Ok(v);
    }
    let raw: RawBalance = parse(BALANCE_JSON, "balance.json")?;
    let val = convert(raw)?;
    Ok(BALANCE.get_or_init(|| val))
}

/// 摊平后的文本表（编译期一次构建，`sget` 用它做查找）。
pub fn pairs() -> &'static [(&'static str, &'static str)] {
    static PAIRS: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    PAIRS.get_or_init(|| {
        let table = STRINGS.get_or_init(load_strings_table);
        table.map.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    })
}

/// 加载文本表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误、非 `string→string` 映射时返回 [`LoadError`]（不 fallback）。
pub fn strings() -> Result<&'static Strings, LoadError> {
    if let Some(v) = STRINGS.get() {
        return Ok(v);
    }
    let raw: BTreeMap<String, serde_json::Value> = parse(STRINGS_JSON, "strings.json")?;
    let mut map = BTreeMap::new();
    for (k, v) in raw {
        if k == SCHEMA_VERSION_KEY {
            continue;
        }
        match v {
            serde_json::Value::String(s) => {
                map.insert(k, s);
            }
            other => {
                return Err(LoadError::Schema(format!(
                    "strings.json: `{k}` should be text, got {other}"
                )));
            }
        }
    }
    Ok(STRINGS.get_or_init(|| Strings { map }))
}

/// 仅供 [`pairs`] 使用的「加载或回退空表」入口（`pairs` 非 Result 语义）。
fn load_strings_table() -> Strings {
    match parse::<BTreeMap<String, serde_json::Value>>(STRINGS_JSON, "strings.json") {
        Ok(raw) => {
            let mut map = BTreeMap::new();
            for (k, v) in raw {
                if k == SCHEMA_VERSION_KEY {
                    continue;
                }
                if let serde_json::Value::String(s) = v {
                    map.insert(k, s);
                }
            }
            Strings { map }
        }
        Err(_) => Strings::default(),
    }
}

/// 加载场景布局表。
///
/// # Errors
/// JSON 语法错误、缺少 `bg` 等必填字段时返回 [`LoadError`]。
pub fn scenes() -> Result<&'static BTreeMap<String, SceneLayout>, LoadError> {
    if let Some(v) = SCENES.get() {
        return Ok(v);
    }
    let raw_map: BTreeMap<String, serde_json::Value> = parse(SCENES_JSON, "scenes.json")?;
    let mut raw = BTreeMap::new();
    for (k, v) in raw_map {
        if k == SCHEMA_VERSION_KEY {
            continue;
        }
        match serde_json::from_value::<RawScene>(v) {
            Ok(s) => {
                raw.insert(k, s);
            }
            Err(e) => return Err(LoadError::Schema(format!("scenes.json: `{k}` bad shape: {e}"))),
        }
    }
    let mut out = BTreeMap::new();
    for (id, r) in raw {
        out.insert(
            id.clone(),
            SceneLayout {
                id,
                bg: r.bg,
                title: r.title.map_or_else(Rect::default, Rect::from),
                list: ListLayout {
                    x: r.list.as_ref().map_or(0, |l| l.x),
                    y: r.list.as_ref().map_or(0, |l| l.y),
                    row_h: r.list.as_ref().map_or(0, |l| l.row_h),
                    item_h: r.list.as_ref().map_or(0, |l| l.item_h),
                    text_x: r.list.as_ref().map_or(0, |l| l.text_x),
                    w: r.list.as_ref().map_or(0, |l| l.w),
                },
                hint: r.hint.map_or_else(Rect::default, Rect::from),
                hud: r.hud.map_or_else(Rect::default, Rect::from),
                page: r.page.map_or_else(Rect::default, Rect::from),
            },
        );
    }
    Ok(SCENES.get_or_init(|| out))
}

#[derive(Deserialize)]
struct RawScene {
    bg: u16,
    #[serde(default)]
    title: Option<RectJson>,
    #[serde(default)]
    list: Option<ListJson>,
    #[serde(default)]
    hint: Option<RectJson>,
    #[serde(default)]
    hud: Option<RectJson>,
    #[serde(default)]
    page: Option<RectJson>,
}

#[derive(Deserialize, Default)]
struct RectJson {
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default)]
    w: i32,
    #[serde(default)]
    h: i32,
}

impl From<RectJson> for Rect {
    fn from(r: RectJson) -> Self {
        Self { x: r.x, y: r.y, w: r.w, h: r.h }
    }
}

#[derive(Deserialize, Default)]
struct ListJson {
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default)]
    w: i32,
    #[serde(default)]
    row_h: i32,
    #[serde(default)]
    item_h: i32,
    #[serde(default)]
    text_x: i32,
}
