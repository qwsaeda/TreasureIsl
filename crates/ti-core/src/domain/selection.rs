//! 选择域类型（业务文档：`docs/business/progression.md` §角色与地图选择）。
//!
//! 5 角色 + 3 地图槽 + 选择状态机（选角 → 选图 → 进入主地图）。

/// 可选角色。
///
/// 业务语义（`docs/business/progression.md` §角色）：5 个角色，目录 id/索引
/// （`balance.selection.characters`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    /// 熊猫海盗。
    Panda,
    /// 蒂奇。
    Tichy,
    /// 基德。
    Kidd,
    /// 德雷克。
    Drake,
    /// 凯瑟琳娜。
    Katharina,
}

impl Character {
    /// 全部角色（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [Character; 5] {
        [Self::Panda, Self::Tichy, Self::Kidd, Self::Drake, Self::Katharina]
    }

    /// 角色索引（0..=4；存档字段 `character` = 本值 +1）。
    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::Panda => 0,
            Self::Tichy => 1,
            Self::Kidd => 2,
            Self::Drake => 3,
            Self::Katharina => 4,
        }
    }

    /// 索引 → 角色（越界 `None`）。
    #[must_use]
    pub const fn from_index(i: u32) -> Option<Self> {
        match i {
            0 => Some(Self::Panda),
            1 => Some(Self::Tichy),
            2 => Some(Self::Kidd),
            3 => Some(Self::Drake),
            4 => Some(Self::Katharina),
            _ => None,
        }
    }

    /// 角色标识名（资源寻址 + i18n key 用，正式文案走 `strings.json`）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Panda => "panda",
            Self::Tichy => "tichy",
            Self::Kidd => "kidd",
            Self::Drake => "drake",
            Self::Katharina => "katharina",
        }
    }
}

/// 可选地图槽位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapSlot {
    /// 地图 1。
    M01,
    /// 地图 2。
    M02,
    /// 地图 3。
    M03,
}

impl MapSlot {
    /// 地图 id（与 `balance.json` `selection.maps[].id` / `terrain.json` 键一致；
    /// S18.9 地形查找用）。
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::M01 => "m01",
            Self::M02 => "m02",
            Self::M03 => "m03",
        }
    }

    /// 全部地图槽位（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [MapSlot; 3] {
        [Self::M01, Self::M02, Self::M03]
    }

    /// 地图编号（`1..=3`，存档字段 `map_slot` = 本值）。
    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::M01 => 1,
            Self::M02 => 2,
            Self::M03 => 3,
        }
    }

    /// 0 起索引（遍历用）。
    #[must_use]
    pub const fn zero_index(self) -> u32 {
        match self {
            Self::M01 => 0,
            Self::M02 => 1,
            Self::M03 => 2,
        }
    }

    /// 0 起索引 → 地图（越界 `None`）。
    #[must_use]
    pub const fn from_index(i: u32) -> Option<Self> {
        match i {
            0 => Some(Self::M01),
            1 => Some(Self::M02),
            2 => Some(Self::M03),
            _ => None,
        }
    }

    /// 地图标识名（资源寻址 + i18n key 用）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::M01 => "glmapm01.Map",
            Self::M02 => "glmapm02.Map",
            Self::M03 => "glmapm03.Map",
        }
    }
}

/// 选择状态：角色 + 地图 + 完成标志。
///
/// 业务语义（`docs/business/progression.md` §选择流程）：
/// - 角色与地图均选定 → `completed = true` → 进入主地图；
/// - 未解锁地图 → 选择被拒（`Sfx::Invalid`），状态不变。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionState {
    /// 已选角色（`None` = 未选）。
    pub character: Option<Character>,
    /// 已选地图（`None` = 未选）。
    pub map_slot: Option<MapSlot>,
    /// 选择完成（角色 + 地图均选定）。
    pub completed: bool,
}

impl SelectionState {
    /// 新建空状态。
    #[must_use]
    pub const fn new() -> Self {
        Self { character: None, map_slot: None, completed: false }
    }

    /// 是否已选角色。
    #[must_use]
    pub fn has_character(self) -> bool {
        self.character.is_some()
    }

    /// 是否已选地图。
    #[must_use]
    pub fn has_map(self) -> bool {
        self.map_slot.is_some()
    }

    /// 场景切换复位（`07_ui_flow.md` §1 硬规则 4 + §6 规则 12 `scene_switch_resets_state`）：
    /// 切场景后选中项必须复位，禁止残留（`completed` 同步复位，否则残留「已完成」态）。
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for SelectionState {
    fn default() -> Self {
        Self::new()
    }
}
