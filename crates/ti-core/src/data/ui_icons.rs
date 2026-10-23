//! UI 图标注册表：`assets/data/ui_icons.json` → [`UiIcons`]（S17.3）。
//!
//! 两层分离（`docs/process/10_ui_redesign.md` §5）：
//! - **图标元数据**（`icons`）：14 枚的 id/尺寸/tint 令牌名，id 锚定 `ti_render::icons::ICON_*`；
//! - **游戏侧映射**（`bindings`）：界面元素 → 图标名（改界面换图标只改 JSON，不改代码）。
//!
//! 与 `scenes.json` 同款解析口径：`include_str!` 内联 + `OnceLock` 缓存 + 语法错误即报错
//! （不静默兜底，`08_render_arch.md` §3 数据纪律）。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{LoadError, UI_ICONS_JSON, parse};

static ICONS: OnceLock<UiIcons> = OnceLock::new();

/// 单枚图标元数据。
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct IconDef {
    /// 图标名（`bindings` 按名引用）。
    pub name: String,
    /// 图标 id（= `ti_render::icons::ICON_*`，0..14）。
    pub id: u32,
    /// 逻辑边长（正方形盒）。
    pub size: i32,
    /// 默认 tint 的 theme 令牌名（`gold` / `text_on_wood` 等；运行时由 `Theme` 解析）。
    pub tint_default: String,
    /// 选中态 tint 的 theme 令牌名。
    pub tint_selected: String,
    /// S21.14 图标图片化：贴图资产路径（相对仓库根，如 `assets/ui/icon_compass.png`）。
    /// `None` = 无贴图（回退 `ti_render::icons` 代码矢量，crash-safe）。blit 时按 `tint` 上色。
    #[serde(default)]
    pub image: Option<String>,
}

/// 界面元素 → 图标名 的绑定（游戏侧语义）。
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct UiIconBindings {
    /// index 主菜单 6 项（按 `menu_index` 升序）。
    #[serde(default)]
    pub index_menu: Vec<IndexMenuIcon>,
    /// `select_map` 行首图标名。
    #[serde(default)]
    pub select_map_row: Option<String>,
    /// `select_role` 行图标名（`monogram` = 圆环 + 数据首字母）。
    #[serde(default)]
    pub select_role_row: Option<String>,
    /// `main_map` HUD 按钮 → 图标名（按 `hud_key`）。
    #[serde(default)]
    pub main_map_hud: Vec<HudIcon>,
    /// `main_map` 据点标记图标名。
    #[serde(default)]
    pub main_map_landmark: Option<String>,
    /// vs BOSS 血条端帽图标名。
    #[serde(default)]
    pub vs_boss_cap: Option<String>,
}

/// index 菜单项图标绑定。
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct IndexMenuIcon {
    /// 菜单项序号（0=新游戏 … 5=离开）。
    pub menu_index: usize,
    /// 图标名。
    pub icon: String,
}

/// HUD 按钮图标绑定。
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct HudIcon {
    /// HUD 矩形键（`hud_trade` / `hud_sell` / `hud_build`）。
    pub hud_key: String,
    /// 图标名。
    pub icon: String,
}
/// 图标缺省边长（JSON 缺 `default_size` 键时的 schema 兜底；须与
/// `ti_render::icons::ICON_SIZE` 恒等，飞轮 R1 ⑥-02 单源对账见 doccheck2）。
const fn default_icon_size() -> i32 {
    48
}

/// 图标注册表全量。
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UiIcons {
    /// 图标原语缺 `size` 参数时的缺省边长（**4 处调用点单源**：`chrome`/Bevy 执行器
    /// 读本值，`icons.rs ICON_SIZE` 为框架侧原生几何常量；飞轮 R1 ⑥-02）。
    #[serde(default = "default_icon_size")]
    pub default_size: i32,
    /// 14 枚图标元数据。
    pub icons: Vec<IconDef>,
    /// 界面元素绑定。
    pub bindings: UiIconBindings,
}

impl UiIcons {
    /// 按名取图标元数据（未登记 = `None`，调用方自行决定是否画占位）。
    #[must_use]
    pub fn icon(&self, name: &str) -> Option<&IconDef> {
        self.icons.iter().find(|i| i.name == name)
    }

    /// index 第 `menu_index` 项的图标元数据（越界序号 = `None`）。
    #[must_use]
    pub fn index_menu_icon(&self, menu_index: usize) -> Option<&IconDef> {
        let bind = self.bindings.index_menu.iter().find(|b| b.menu_index == menu_index)?;
        self.icon(&bind.icon)
    }

    /// HUD 键（`hud_trade` 等）的图标元数据。
    #[must_use]
    pub fn hud_icon(&self, hud_key: &str) -> Option<&IconDef> {
        let bind = self.bindings.main_map_hud.iter().find(|b| b.hud_key == hud_key)?;
        self.icon(&bind.icon)
    }
}

/// 加载图标注册表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 结构不符时返回 [`LoadError`]（不 fallback：图标缺失 = 界面缺图标）。
pub fn ui_icons() -> Result<&'static UiIcons, LoadError> {
    if let Some(v) = ICONS.get() {
        return Ok(v);
    }
    let parsed: UiIcons = parse(UI_ICONS_JSON, "ui_icons.json")?;
    Ok(ICONS.get_or_init(|| parsed))
}

/// name → id 的便利表（BTreeMap，稳定序，便于诊断打印）。
#[must_use]
pub fn icon_ids() -> BTreeMap<String, u32> {
    ui_icons().map(|t| t.icons.iter().map(|i| (i.name.clone(), i.id)).collect()).unwrap_or_default()
}
