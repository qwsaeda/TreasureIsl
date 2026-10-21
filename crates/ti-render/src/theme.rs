//! 主题系统：所有颜色常量与字号常量的数据驱动来源。
//!
//! 设计原则：零硬编码——所有渲染色值与字号从 [`Theme`] 实例读取，
//! 支持多主题切换（暗色/亮色/高对比度等），不携带任何游戏语义。

/// 主题数据：全部色板 + 字号集中在一个结构体中。
///
/// 颜色格式为 RGB565（`u16`），字号为逻辑像素（`i32`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    // === 色板 ===
    /// 深底（现代深色 UI 背景）。
    pub parch_dark: u16,
    /// 深底亮（背景顶部渐变）。
    pub parch_light: u16,
    /// 卡片描边/标题条（深棕木纹色）。
    pub wood: u16,
    /// 金高光（选中/标题点缀）。
    pub gold: u16,
    /// 航海蓝亮（选中高光）。
    pub gold_glow: u16,
    /// 航海蓝（accent/Logo）。
    pub gold_bright: u16,
    /// 卡片底（菜单项色块）。
    pub menu_bg: u16,
    /// 卡片描边（菜单项边框）。
    pub menu_border: u16,
    /// 进度条底/选中槽底。
    pub hilite: u16,
    /// bevel 凸起（卡片亮）。
    pub bevel_hi: u16,
    /// bevel 凹陷/暗角。
    pub bevel_shadow: u16,
    /// 正文白字（深底上的白文字）。
    pub text_on_parch: u16,
    /// 标题象牙金（压深色标题条）。
    pub text_on_wood: u16,
    /// 危险/BOSS 标记色（血红）。
    pub danger: u16,
    /// 玩家棋子/单位色（青绿）。
    pub player: u16,
    /// 据点标记色（金）。
    pub landmark: u16,
    /// 背景填充色（深底）。
    pub bg: u16,
    /// 标题条主色（卡片描边）。
    pub title_bar: u16,
    /// 选中高亮色（航海蓝）。
    pub selected: u16,
    /// 次文字/提示行色（灰）。
    pub secondary: u16,
    /// 清屏黑。
    pub black: u16,
    /// 暖木色（S18.6 暖色系令牌：Agnes 风格木框；`wood` = 冷蓝旧值保留不动，
    /// 新场景用本令牌 = 换整套木色风格只改数据）。
    pub wood_warm: u16,
    /// 深海水色（主地图海面瓦片底色；S18.9）
    pub sea_deep: u16,
    /// 浅海水色（岛缘浅滩 / 瓦片顶棱高光；S18.9）
    pub sea_shallow: u16,
    /// 礍石色（岛缘礍岩瓦片；S18.9）
    pub reef: u16,
    /// 陆地沙色（岛屿瓦片底色；S18.9）
    pub land_sand: u16,
    /// 陆地浅沙（瓦片顶棱高光；S18.9）
    pub land_sand_light: u16,
    /// 陆地暗沙（瓦片交错变体；S18.9）
    pub land_sand_dark: u16,
    /// 暖金（S18.6 暖色系令牌：Agnes 风格标题金）。
    pub gold_warm: u16,
    /// 夜空深蓝（S18.6 暖色系令牌：Agnes 风格夜空渐变底色 = #1A2942）。
    pub sky_navy: u16,
    /// 夜空地平线色（S18.7：海天交界更亮的蓝 = #16304F）。
    pub sky_horizon: u16,
    /// 月/光晕淡金（S18.7：Agnes 风格月色 = #F2E4C0）。
    pub moon_pale: u16,

    // === 字号 ===
    /// 正文 TTF 字号（px）。
    pub font_size: i32,
    /// 标题档 TTF 字号（px）。
    pub title_font_size: i32,
    /// 提示档 TTF 字号（px）。
    pub hint_font_size: i32,
    /// 位图回退正文 advance（px）。
    pub bitmap_advance: i32,
    /// 位图回退标题 advance（px）。
    pub bitmap_advance_title: i32,
    /// 位图回退提示 advance（px）。
    pub bitmap_advance_hint: i32,
}

impl Theme {
    /// 按**令牌名**取色（S18.6 数据驱动：`scenes.json` 原语的 `color` 字段写令牌名，
    /// 换色 = 改数据 / 改 theme.json 镜像，不动渲染代码）。
    ///
    /// 未知令牌名 = `None`（消费方回退原语默认色，不 panic）。
    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<u16> {
        Some(match name {
            "parch_dark" => self.parch_dark,
            "parch_light" => self.parch_light,
            "wood" => self.wood,
            "gold" => self.gold,
            "gold_glow" => self.gold_glow,
            "gold_bright" => self.gold_bright,
            "menu_bg" => self.menu_bg,
            "menu_border" => self.menu_border,
            "hilite" => self.hilite,
            "bevel_hi" => self.bevel_hi,
            "bevel_shadow" => self.bevel_shadow,
            "text_on_parch" => self.text_on_parch,
            "text_on_wood" => self.text_on_wood,
            "danger" => self.danger,
            "player" => self.player,
            "landmark" => self.landmark,
            "bg" => self.bg,
            "title_bar" => self.title_bar,
            "selected" => self.selected,
            "secondary" => self.secondary,
            "black" => self.black,
            "wood_warm" => self.wood_warm,
            "sea_deep" => self.sea_deep,
            "sea_shallow" => self.sea_shallow,
            "reef" => self.reef,
            "land_sand" => self.land_sand,
            "land_sand_light" => self.land_sand_light,
            "land_sand_dark" => self.land_sand_dark,
            "gold_warm" => self.gold_warm,
            "sky_navy" => self.sky_navy,
            "sky_horizon" => self.sky_horizon,
            "moon_pale" => self.moon_pale,
            _ => return None,
        })
    }

    /// 默认主题单源常量：`chrome::COLOR_*` 兼容重导出与 `theme.json` 镜像都锚定
    /// 此处（F4 单一事实源：改色只改这里，常量/JSON 自动同步）。
    /// S17.1 航海夜·精修 v2（`docs/process/10_ui_redesign.md` §3 令牌表，8-bit→RGB565 脚本口径）。
    pub const DEFAULT: Self = Self {
        // 色板（RGB565）
        parch_dark: 0x0865,
        parch_light: 0x10C9,
        wood: 0x3A17,
        gold: 0xFE55,
        gold_glow: 0x4E3D,
        gold_bright: 0x4E3D,
        menu_bg: 0x190C,
        menu_border: 0x3A17,
        hilite: 0x45B6,
        bevel_hi: 0x2171,
        bevel_shadow: 0x0865,
        text_on_parch: 0xF7BA,
        text_on_wood: 0xFE55,
        danger: 0xFB57,
        player: 0x6EA3,
        landmark: 0xFE55,
        bg: 0x0865,
        title_bar: 0x3A17,
        selected: 0x4E3D,
        secondary: 0x9D30,
        // S18.6 暖色系令牌（Agnes 风格）：暖木 #8B6914 → 0x8B42；暖金 #FFD700 → 0xFEA0；
        // 夜空 #1A2942 → 0x1948。
        wood_warm: 0x8B42,
        sea_deep: 0x108A,
        sea_shallow: 0x1A30,
        reef: 0x4BCD,
        land_sand: 0xFD4D,
        land_sand_light: 0xFE51,
        land_sand_dark: 0xD4CA,
        gold_warm: 0xFEA0,
        sky_navy: 0x1948,
        sky_horizon: 0x198A,
        moon_pale: 0xEF17,
        black: 0x0000,
        // 字号
        font_size: 44,
        title_font_size: 96,
        hint_font_size: 28,
        bitmap_advance: 12,
        bitmap_advance_title: 26,
        bitmap_advance_hint: 8,
    };
}

impl Default for Theme {
    /// 默认主题 = 现代深色 + 航海蓝 UI（与 `08_layout_design.md` §2 一致）。
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Theme {
    /// 创建默认主题（显式构造，语义同 [`Theme::default`]）。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 JSON 文本解析主题（S15.7 数据分离：色板 + 字号来自 `assets/data/theme.json`）。
    ///
    /// 框架**不做任何 IO**：调用方（游戏侧）负责读文件并传入字节，本函数保持框架与
    /// 具体游戏的资源布局解耦（其它游戏可喂自己的主题文件）。
    ///
    /// 缺字段回落 [`Theme::default`] 对应项（不静默改值：只有 JSON 里显式出现的键才生效）；
    /// 语法错误或顶层结构不符则整体回落 `default` 并由调用方决定是否告警。
    #[must_use]
    pub fn from_json_str(text: &str) -> Self {
        let Ok(root) = serde_json::from_str::<ThemeJson>(text) else {
            return Self::default();
        };
        Self::from_json(root)
    }

    /// 从已解析的 JSON 结构合并主题（`from_json_str` 的结构化入口，便于测试）。
    #[must_use]
    pub fn from_json(root: ThemeJson) -> Self {
        use OptHex;
        let d = Self::default();
        let p = root.palette.unwrap_or_default();
        let f = root.fonts.unwrap_or_default();
        Self {
            parch_dark: p.parch_dark.hex().unwrap_or(d.parch_dark),
            parch_light: p.parch_light.hex().unwrap_or(d.parch_light),
            wood: p.wood.hex().unwrap_or(d.wood),
            gold: p.gold.hex().unwrap_or(d.gold),
            gold_glow: p.gold_glow.hex().unwrap_or(d.gold_glow),
            gold_bright: p.gold_bright.hex().unwrap_or(d.gold_bright),
            menu_bg: p.menu_bg.hex().unwrap_or(d.menu_bg),
            menu_border: p.menu_border.hex().unwrap_or(d.menu_border),
            hilite: p.hilite.hex().unwrap_or(d.hilite),
            bevel_hi: p.bevel_hi.hex().unwrap_or(d.bevel_hi),
            bevel_shadow: p.bevel_shadow.hex().unwrap_or(d.bevel_shadow),
            text_on_parch: p.text_on_parch.hex().unwrap_or(d.text_on_parch),
            text_on_wood: p.text_on_wood.hex().unwrap_or(d.text_on_wood),
            danger: p.danger.hex().unwrap_or(d.danger),
            player: p.player.hex().unwrap_or(d.player),
            landmark: p.landmark.hex().unwrap_or(d.landmark),
            bg: p.bg.hex().unwrap_or(d.bg),
            title_bar: p.title_bar.hex().unwrap_or(d.title_bar),
            selected: p.selected.hex().unwrap_or(d.selected),
            secondary: p.secondary.hex().unwrap_or(d.secondary),
            black: p.black.hex().unwrap_or(d.black),
            wood_warm: p.wood_warm.hex().unwrap_or(d.wood_warm),
            sea_deep: p.sea_deep.hex().unwrap_or(d.sea_deep),
            sea_shallow: p.sea_shallow.hex().unwrap_or(d.sea_shallow),
            reef: p.reef.hex().unwrap_or(d.reef),
            land_sand: p.land_sand.hex().unwrap_or(d.land_sand),
            land_sand_light: p.land_sand_light.hex().unwrap_or(d.land_sand_light),
            land_sand_dark: p.land_sand_dark.hex().unwrap_or(d.land_sand_dark),
            gold_warm: p.gold_warm.hex().unwrap_or(d.gold_warm),
            sky_navy: p.sky_navy.hex().unwrap_or(d.sky_navy),
            sky_horizon: p.sky_horizon.hex().unwrap_or(d.sky_horizon),
            moon_pale: p.moon_pale.hex().unwrap_or(d.moon_pale),
            font_size: f.font_size.unwrap_or(d.font_size),
            title_font_size: f.title_font_size.unwrap_or(d.title_font_size),
            hint_font_size: f.hint_font_size.unwrap_or(d.hint_font_size),
            bitmap_advance: f.bitmap_advance.unwrap_or(d.bitmap_advance),
            bitmap_advance_title: f.bitmap_advance_title.unwrap_or(d.bitmap_advance_title),
            bitmap_advance_hint: f.bitmap_advance_hint.unwrap_or(d.bitmap_advance_hint),
        }
    }
}

/// 主题 JSON 顶层结构（`palette` + `fonts` 两组；组内每个键可选）。
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ThemeJson {
    /// 色板组（键名与 [`Theme`] 字段同名）。
    #[serde(default)]
    pub palette: Option<PaletteJson>,
    /// 字号组。
    #[serde(default)]
    pub fonts: Option<FontsJson>,
}

/// 色板组：全部可选（缺失回落 `default`）。色值接受 `"0x10A3"` 字符串或裸数字。
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PaletteJson {
    /// 见 [`Theme::parch_dark`]。
    pub parch_dark: Option<Hex16>,
    /// 见 [`Theme::parch_light`]。
    pub parch_light: Option<Hex16>,
    /// 见 [`Theme::wood`]。
    pub wood: Option<Hex16>,
    /// 见 [`Theme::gold`]。
    pub gold: Option<Hex16>,
    /// 见 [`Theme::gold_glow`]。
    pub gold_glow: Option<Hex16>,
    /// 见 [`Theme::gold_bright`]。
    pub gold_bright: Option<Hex16>,
    /// 见 [`Theme::menu_bg`]。
    pub menu_bg: Option<Hex16>,
    /// 见 [`Theme::menu_border`]。
    pub menu_border: Option<Hex16>,
    /// 见 [`Theme::hilite`]。
    pub hilite: Option<Hex16>,
    /// 见 [`Theme::bevel_hi`]。
    pub bevel_hi: Option<Hex16>,
    /// 见 [`Theme::bevel_shadow`]。
    pub bevel_shadow: Option<Hex16>,
    /// 见 [`Theme::text_on_parch`]。
    pub text_on_parch: Option<Hex16>,
    /// 见 [`Theme::text_on_wood`]。
    pub text_on_wood: Option<Hex16>,
    /// 见 [`Theme::danger`]。
    pub danger: Option<Hex16>,
    /// 见 [`Theme::player`]。
    pub player: Option<Hex16>,
    /// 见 [`Theme::landmark`]。
    pub landmark: Option<Hex16>,
    /// 见 [`Theme::bg`]。
    pub bg: Option<Hex16>,
    /// 见 [`Theme::title_bar`]。
    pub title_bar: Option<Hex16>,
    /// 见 [`Theme::selected`]。
    pub selected: Option<Hex16>,
    /// 见 [`Theme::secondary`]。
    pub secondary: Option<Hex16>,
    /// 见 [`Theme::black`]。
    pub black: Option<Hex16>,
    /// 见 [`Theme::wood_warm`]。
    pub wood_warm: Option<Hex16>,
    /// 深海水色（主地图海面瓦片底色；S18.9）
    pub sea_deep: Option<Hex16>,
    /// 浅海水色（岛缘浅滩 / 瓦片顶棱高光；S18.9）
    pub sea_shallow: Option<Hex16>,
    /// 礍石色（岛缘礍岩瓦片；S18.9）
    pub reef: Option<Hex16>,
    /// 陆地沙色（岛屿瓦片底色；S18.9）
    pub land_sand: Option<Hex16>,
    /// 陆地浅沙（瓦片顶棱高光；S18.9）
    pub land_sand_light: Option<Hex16>,
    /// 陆地暗沙（瓦片交错变体；S18.9）
    pub land_sand_dark: Option<Hex16>,
    /// 见 [`Theme::gold_warm`]。
    pub gold_warm: Option<Hex16>,
    /// 见 [`Theme::sky_navy`]。
    pub sky_navy: Option<Hex16>,
    /// 见 [`Theme::sky_horizon`]。
    pub sky_horizon: Option<Hex16>,
    /// 见 [`Theme::moon_pale`]。
    pub moon_pale: Option<Hex16>,
}

/// 字号组：全部可选（缺失回落 `default`）。
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct FontsJson {
    /// 见 [`Theme::font_size`]。
    pub font_size: Option<i32>,
    /// 见 [`Theme::title_font_size`]。
    pub title_font_size: Option<i32>,
    /// 见 [`Theme::hint_font_size`]。
    pub hint_font_size: Option<i32>,
    /// 见 [`Theme::bitmap_advance`]。
    pub bitmap_advance: Option<i32>,
    /// 见 [`Theme::bitmap_advance_title`]。
    pub bitmap_advance_title: Option<i32>,
    /// 见 [`Theme::bitmap_advance_hint`]。
    pub bitmap_advance_hint: Option<i32>,
}

/// RGB565 色值的 JSON 表示：接受 `"0x10A3"` / `"10A3"` / 裸数字 `4291`。
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum Hex16 {
    /// 带/不带 `0x` 前缀的十六进制字符串。
    Str(String),
    /// 裸十进制数（JSON 无 `0x` 字面量）。
    Num(i64),
}

impl Hex16 {
    /// 解析为 RGB565 `u16`（失败 → `None`，由调用方回落 `default`）。
    #[must_use]
    pub fn to_u16(&self) -> Option<u16> {
        match self {
            Self::Str(s) => {
                let t = s.trim();
                let h = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
                u16::from_str_radix(h, 16).ok()
            }
            Self::Num(n) => u16::try_from(*n).ok(),
        }
    }
}

/// 把 `Option<Hex16>` 组装成 `Option<u16>`（`None`/解析失败均透传 `None`）。
trait OptHex {
    /// 取值（失败为 `None`）。
    fn hex(&self) -> Option<u16>;
}

impl OptHex for Option<Hex16> {
    fn hex(&self) -> Option<u16> {
        self.as_ref().and_then(Hex16::to_u16)
    }
}
