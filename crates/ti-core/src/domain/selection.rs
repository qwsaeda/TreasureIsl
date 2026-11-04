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

    /// 角色总数（= `all()` 长度；表现层循环/导航计数的单一事实源，B11：
    /// 域层加角色 → 本出口自动 +1，消费方不再各自硬编码镜像计数）。
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn count() -> u32 {
        Self::all().len() as u32
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

/// 可选地图槽位（S37：20 图 = m01~m20；推进链 20 图，m20 = 通关图）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapSlot {
    /// 地图 1。
    M01,
    /// 地图 2。
    M02,
    /// 地图 3。
    M03,
    /// 地图 4（冰霜极地）。
    M04,
    /// 地图 5（热带乐园）。
    M05,
    /// 地图 6（沙漠绿洲）。
    M06,
    /// 地图 7（迷雾海藻）。
    M07,
    /// 地图 8（怒海断崖）。
    M08,
    /// 地图 9（落日浅滩）。
    M09,
    /// 地图 10（深渊海沟）。
    M10,
    /// 地图 11（双岩骷髅）。
    M11,
    /// 地图 12（宝藏之岛）。
    M12,
    /// 地图 13（红树林湾）。
    M13,
    /// 地图 14（幽灵浅礁）。
    M14,
    /// 地图 15（熔岩海岸）。
    M15,
    /// 地图 16（月夜海湾）。
    M16,
    /// 地图 17（水晶港湾）。
    M17,
    /// 地图 18（风暴残骸）。
    M18,
    /// 地图 19（珊瑚环礁）。
    M19,
    /// 地图 20（火山之眼；通关图）。
    M20,
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
            Self::M04 => "m04",
            Self::M05 => "m05",
            Self::M06 => "m06",
            Self::M07 => "m07",
            Self::M08 => "m08",
            Self::M09 => "m09",
            Self::M10 => "m10",
            Self::M11 => "m11",
            Self::M12 => "m12",
            Self::M13 => "m13",
            Self::M14 => "m14",
            Self::M15 => "m15",
            Self::M16 => "m16",
            Self::M17 => "m17",
            Self::M18 => "m18",
            Self::M19 => "m19",
            Self::M20 => "m20",
        }
    }

    /// 全部地图槽位（按枚举声明顺序，遍历用）。
    #[must_use]
    pub const fn all() -> [MapSlot; 20] {
        [
            Self::M01,
            Self::M02,
            Self::M03,
            Self::M04,
            Self::M05,
            Self::M06,
            Self::M07,
            Self::M08,
            Self::M09,
            Self::M10,
            Self::M11,
            Self::M12,
            Self::M13,
            Self::M14,
            Self::M15,
            Self::M16,
            Self::M17,
            Self::M18,
            Self::M19,
            Self::M20,
        ]
    }

    /// 地图槽总数（= `all()` 长度；表现层循环/导航计数的单一事实源，B11：
    /// 域层加图 → 本出口自动 +1，消费方不再各自硬编码镜像计数）。
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn count() -> u32 {
        Self::all().len() as u32
    }

    /// 地图编号（`1..=20`，存档字段 `map_slot` = 本值）。
    #[must_use]
    pub const fn index(self) -> u32 {
        self.zero_index() + 1
    }

    /// 0 起索引（遍历用）。
    #[must_use]
    pub const fn zero_index(self) -> u32 {
        match self {
            Self::M01 => 0,
            Self::M02 => 1,
            Self::M03 => 2,
            Self::M04 => 3,
            Self::M05 => 4,
            Self::M06 => 5,
            Self::M07 => 6,
            Self::M08 => 7,
            Self::M09 => 8,
            Self::M10 => 9,
            Self::M11 => 10,
            Self::M12 => 11,
            Self::M13 => 12,
            Self::M14 => 13,
            Self::M15 => 14,
            Self::M16 => 15,
            Self::M17 => 16,
            Self::M18 => 17,
            Self::M19 => 18,
            Self::M20 => 19,
        }
    }

    /// 0 起索引 → 地图（越界 `None`）。
    #[must_use]
    pub const fn from_index(i: u32) -> Option<Self> {
        match i {
            0 => Some(Self::M01),
            1 => Some(Self::M02),
            2 => Some(Self::M03),
            3 => Some(Self::M04),
            4 => Some(Self::M05),
            5 => Some(Self::M06),
            6 => Some(Self::M07),
            7 => Some(Self::M08),
            8 => Some(Self::M09),
            9 => Some(Self::M10),
            10 => Some(Self::M11),
            11 => Some(Self::M12),
            12 => Some(Self::M13),
            13 => Some(Self::M14),
            14 => Some(Self::M15),
            15 => Some(Self::M16),
            16 => Some(Self::M17),
            17 => Some(Self::M18),
            18 => Some(Self::M19),
            19 => Some(Self::M20),
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
            Self::M04 => "glmapm04.Map",
            Self::M05 => "glmapm05.Map",
            Self::M06 => "glmapm06.Map",
            Self::M07 => "glmapm07.Map",
            Self::M08 => "glmapm08.Map",
            Self::M09 => "glmapm09.Map",
            Self::M10 => "glmapm10.Map",
            Self::M11 => "glmapm11.Map",
            Self::M12 => "glmapm12.Map",
            Self::M13 => "glmapm13.Map",
            Self::M14 => "glmapm14.Map",
            Self::M15 => "glmapm15.Map",
            Self::M16 => "glmapm16.Map",
            Self::M17 => "glmapm17.Map",
            Self::M18 => "glmapm18.Map",
            Self::M19 => "glmapm19.Map",
            Self::M20 => "glmapm20.Map",
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
