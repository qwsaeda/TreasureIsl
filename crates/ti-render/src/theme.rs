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
    /// 默认主题单源常量：`chrome::COLOR_*` 兼容重导出与 `theme.json` 镜像都锚定
    /// 此处（F4 单一事实源：改色只改这里，常量/JSON 自动同步）。
    pub const DEFAULT: Self = Self {
        // 色板（RGB565）
        parch_dark: 0x10A3,
        parch_light: 0x2106,
        wood: 0x3A0B,
        gold: 0xF5A8,
        gold_glow: 0x453E,
        gold_bright: 0x245C,
        menu_bg: 0x2147,
        menu_border: 0x3A0B,
        hilite: 0x2967,
        bevel_hi: 0x31A9,
        bevel_shadow: 0x0862,
        text_on_parch: 0xFFFF,
        text_on_wood: 0xFF77,
        danger: 0xEAAA,
        player: 0x2D33,
        landmark: 0xF5A8,
        bg: 0x10A3,
        title_bar: 0x3A0B,
        selected: 0x245C,
        secondary: 0x8C93,
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
