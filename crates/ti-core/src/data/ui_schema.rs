//! S85.3 UI 组件 schema 加载层：`assets/data/ui_schema/*.json` → 类型化结构。
//!
//! 单一事实源 = [`components_v1.json`](../../../../assets/data/ui_schema/components_v1.json)
//! （5 纯 UI 场景的元素声明：type/role/binding/layout/action/state）
//! + [`asset_roles.json`](../../../../assets/data/ui_schema/asset_roles.json)（role→资产路径）。
//!
//! 规范见 `docs/process/33_ui_component_schema.md`。
//!
//! 约定与 `data/mod.rs` 其余表一致：
//! - `include_str!` 进包（wasm 免 IO）；解析结果进 `OnceLock`，每帧不重解析；
//! - **解析失败 = 硬错误**（[`LoadError`]），不 fallback；
//! - 表注入兼容 hook（[`install_injected`]）：首访前可注入覆盖文本（主题/编辑器 Preview 用），
//!   迟注入忽略（与 `table_source` 同口径）；当前无任何调用方 = 纯内嵌默认，0 行为变化。
//!
//! 零游戏语义：本模块只描述「界面元素长什么样 / 绑定到哪个语义字符串」，
//! 不出现 Character/MapSlot/GameApp 等业务类型（业务取数在壳侧 binding resolver）。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::{LoadError, parse};

/// 组件声明表（编译期内联）。
pub(crate) const COMPONENTS_JSON: &str =
    include_str!("../../../../assets/data/ui_schema/components_v1.json");
/// 资产角色注册表（编译期内联）。
pub(crate) const ASSET_ROLES_JSON: &str =
    include_str!("../../../../assets/data/ui_schema/asset_roles.json");

// ---------------------------------------------------------------- 注入 hook

/// 可注入的 schema 文本（首访前生效；缺段 = 内嵌默认）。
#[derive(Debug, Default, Clone)]
pub struct InjectedSchema {
    /// `components_v1.json` 覆盖文本。
    pub components: Option<String>,
    /// `asset_roles.json` 覆盖文本。
    pub asset_roles: Option<String>,
}

static INJECTED: OnceLock<InjectedSchema> = OnceLock::new();

/// 注入 schema 覆盖文本（**首访前**调用；迟调用 = 忽略，保持内嵌默认）。
///
/// 返回 `true` = 注入生效。当前无调用方（主题/编辑器写点在 S85.5/S86 接入），
/// 本函数仅提供与 `table_source::install` 同口径的扩展点。
#[must_use]
pub fn install_injected(schema: InjectedSchema) -> bool {
    INJECTED.set(schema).is_ok()
}

fn injected_components_text() -> Option<String> {
    INJECTED.get().and_then(|s| s.components.clone())
}

fn injected_roles_text() -> Option<String> {
    INJECTED.get().and_then(|s| s.asset_roles.clone())
}

// ---------------------------------------------------------------- 公开类型

/// `components_v1.json` 顶层。
#[derive(Debug, Clone, Deserialize)]
struct UiSchemaFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    scenes: BTreeMap<String, SceneComponents>,
}

/// 单场景的组件列表（`components_v1.json` `scenes.<id>`）。
#[derive(Debug, Clone, Deserialize)]
pub struct SceneComponents {
    /// 场景 id（与键名一致，冗余便于消费方回显）。
    pub scene_id: String,
    /// 元素声明（有序；实际绘制顺序按 `z` 升序，见渲染器）。
    #[serde(default)]
    pub elements: Vec<Component>,
    /// 主题覆盖层（role→资产路径；S85.5 主题写点）。
    #[serde(default)]
    pub overrides: BTreeMap<String, String>,
}

/// 单个 UI 组件声明（零游戏语义的展示描述）。
#[derive(Debug, Clone, Deserialize)]
pub struct Component {
    /// 场景内唯一 id（`^[a-z][a-z0-9_]*$`，校验脚本守）。
    pub id: String,
    /// 组件种类（`image`/`rect`/`text`/`button`/`input`/`list`/`badge`/`grad`/`glow`/`panel`）。
    #[serde(rename = "type")]
    pub kind: String,
    /// 资产角色名（`type∈{image,rect,grad,glow,badge}` 用；须在 `asset_roles.json` 登记）。
    #[serde(default)]
    pub role: String,
    /// 业务绑定名（i18n key 或 binding resolver 名）。
    #[serde(default)]
    pub binding: String,
    /// 布局意图（意图化 / anchored 百分比，禁绝对像素）。
    pub layout: Layout,
    /// 动作名（→ opaque `PickTarget` key，具体映射在壳侧注册表）。
    #[serde(default)]
    pub action: String,
    /// 可见状态集合（`normal`/`selected`/`hover`/`disabled`）。
    #[serde(default)]
    pub state: Vec<String>,
    /// 字面文本（`text`/`button` 用；binding 优先）。
    #[serde(default)]
    pub text: String,
    /// 文本档位（`title`/`body`/`hint`）。
    #[serde(default)]
    pub tier: String,
    /// 图标名（`icon` 用；`assets/ui/icon_<name>.png`）。
    #[serde(default)]
    pub icon: String,
    /// 层序（升序绘制；同层后建者在上）。
    #[serde(default)]
    pub z: i32,
}

/// 布局意图（taffy 自动布局的输入；禁 x/y/w/h 绝对坐标字段）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Layout {
    /// 必填意图：`fill`/`center`/`side_left`/`side_right`/`bottom_bar`/`flex_col`/`flex_row`/`anchored`。
    pub intent: String,
    /// 贴边方位（`side_left`/`side_right` 用；`intent.side` 兼容字段）。
    #[serde(default)]
    pub side: String,
    /// 宽度百分比（贴边/居中面板用）。
    #[serde(default)]
    pub width_pct: Option<f32>,
    /// flex 子项间距（px；`flex_col`/`flex_row` 用）。
    #[serde(default)]
    pub gap: Option<u32>,
    /// 交叉轴对齐（`start`/`center`/`end`/`stretch`）。
    #[serde(default)]
    pub align: String,
    /// 锚点（`anchored` 用：`top_left`/`top_center`/…/`center`）。
    #[serde(default)]
    pub anchor: String,
    /// 锚点百分比偏移。
    #[serde(default)]
    pub offset_pct: Option<OffsetPct>,
    /// 节点百分比尺寸。
    #[serde(default)]
    pub size_pct: Option<SizePct>,
}

/// anchored 的百分比偏移（相对视口；非绝对像素）。
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct OffsetPct {
    /// 水平偏移（视口百分比）。
    pub x: f32,
    /// 垂直偏移（视口百分比）。
    pub y: f32,
}

/// anchored 的百分比尺寸。
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct SizePct {
    /// 宽（视口百分比）。
    pub w: f32,
    /// 高（视口百分比）。
    pub h: f32,
}

/// `asset_roles.json` 顶层（role→资产路径单源）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AssetRoles {
    /// 注册表版本（当前 1）。
    #[serde(default)]
    pub version: u32,
    /// role → 资产路径（`assets/ui/...`）。
    #[serde(default)]
    pub roles: BTreeMap<String, String>,
}

impl AssetRoles {
    /// role → 资产路径（未登记 = `None`）。
    #[must_use]
    pub fn path(&self, role: &str) -> Option<&str> {
        self.roles.get(role).map(String::as_str)
    }
}

// ---------------------------------------------------------------- 缓存入口

static COMPONENTS: OnceLock<BTreeMap<String, SceneComponents>> = OnceLock::new();
static ROLES: OnceLock<AssetRoles> = OnceLock::new();

/// 加载 5 场景组件声明表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 缺必填字段 / `schema_version != 1` 时返回 [`LoadError`]。
pub fn components() -> Result<&'static BTreeMap<String, SceneComponents>, LoadError> {
    if let Some(v) = COMPONENTS.get() {
        return Ok(v);
    }
    let raw = injected_components_text().unwrap_or_else(|| COMPONENTS_JSON.to_owned());
    let file: UiSchemaFile = parse(&raw, "components_v1.json")?;
    if file.schema_version != 1 {
        return Err(LoadError::Schema(format!(
            "components_v1.json: schema_version = {} (expect 1)",
            file.schema_version
        )));
    }
    if file.scenes.is_empty() {
        return Err(LoadError::Schema("components_v1.json: scenes is empty".to_string()));
    }
    Ok(COMPONENTS.get_or_init(|| file.scenes))
}

/// 加载资产角色注册表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误 / 结构不符时返回 [`LoadError`]。
pub fn asset_roles() -> Result<&'static AssetRoles, LoadError> {
    if let Some(v) = ROLES.get() {
        return Ok(v);
    }
    let raw = injected_roles_text().unwrap_or_else(|| ASSET_ROLES_JSON.to_owned());
    let roles: AssetRoles = parse(&raw, "asset_roles.json")?;
    if roles.roles.is_empty() {
        return Err(LoadError::Schema("asset_roles.json: roles is empty".to_string()));
    }
    Ok(ROLES.get_or_init(|| roles))
}
