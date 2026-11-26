//! S85.3 UI 组件 schema 加载层：`assets/data/ui_schema/*.json` → 类型化结构。
//!
//! 单一事实源 = [`components_v1.json`](../../../../assets/data/ui_schema/components_v1.json)
//! （5 纯 UI 场景的元素声明：type/role/binding/layout/action/state）
//! + [`asset_roles.json`](../../../../assets/data/ui_schema/asset_roles.json)（role→资产路径）
//! + [`world_components.json`](../../../../assets/data/ui_schema/world_components.json)：S87.1 世界层
//!   `vs`/`main_map` 静态元素（`type`∈{Rect,Text,Image} / geometry ref / `world_rect`）。
//!
//! 规范见 `docs/process/33_ui_component_schema.md` + `docs/plan-s87-world-layer.md` §2c。
//! 世界层 schema 当前无消费者（S87.2 渲染器接入），本模块只提供加载与类型，0 行为变化。
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
/// 世界层组件声明表（编译期内联；S87.1，当前无消费者）。
pub(crate) const WORLD_COMPONENTS_JSON: &str =
    include_str!("../../../../assets/data/ui_schema/world_components.json");

// ---------------------------------------------------------------- 注入 hook

/// 可注入的 schema 文本（首访前生效；缺段 = 内嵌默认）。
#[derive(Debug, Default, Clone)]
pub struct InjectedSchema {
    /// `components_v1.json` 覆盖文本。
    pub components: Option<String>,
    /// `asset_roles.json` 覆盖文本。
    pub asset_roles: Option<String>,
    /// S87.1 `world_components.json` 覆盖文本（首访前生效；缺段 = 内嵌默认）。
    pub world_components: Option<String>,
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

fn injected_world_text() -> Option<String> {
    INJECTED.get().and_then(|s| s.world_components.clone())
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

// ---------------------------------------------------------- 世界层 schema（S87.1）

/// `world_components.json` 顶层（未知字段忽略，向前兼容）。
#[derive(Debug, Clone, Deserialize)]
struct WorldSchemaFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    scenes: BTreeMap<String, WorldScene>,
}

/// 单场景的世界层元素列表（`world_components.json` `scenes.<id>`）。
#[derive(Debug, Clone, Deserialize)]
pub struct WorldScene {
    /// 场景 id（与键名一致：仅 `main_map` / `vs`）。
    pub scene_id: String,
    /// 静态元素声明（有序；实际绘制顺序由渲染器按 `z` 升序定，S87.2）。
    #[serde(default)]
    pub elements: Vec<WorldComponent>,
    /// 主题覆盖层（role→资产路径）。
    #[serde(default)]
    pub overrides: BTreeMap<String, String>,
}

/// 单个世界层元素声明（静态可数据化部分；动画/数值保留 Rust 过程式）。
#[derive(Debug, Clone, Deserialize)]
pub struct WorldComponent {
    /// 场景内唯一 id（`^[a-z][a-z0-9_]*$`，校验脚本守）。
    pub id: String,
    /// 元素种类（`Rect`/`Text`/`Image` = CPU `DrawOp` 三型；禁 `grad`/`glow`/`panel`）。
    #[serde(rename = "type")]
    pub kind: String,
    /// 资产角色名（须在 `asset_roles.json` 登记且资产实存）。
    #[serde(default)]
    pub role: String,
    /// 运行时绑定：i18n key 字符串或 `{kind:"host",path,fmt}` 取值路径。
    #[serde(default)]
    pub binding: Option<BindingSpec>,
    /// 几何引用（指向 v6 布局 JSON / `scenes.json`，「不手写像素」单源）。
    #[serde(default)]
    pub geometry: Option<Geometry>,
    /// 世界坐标矩形（仅世界层允许；禁散落裸 x/y/w/h 到元素顶层）。
    #[serde(default)]
    pub world_rect: Option<WorldRect>,
    /// 层序（越大越上）。
    #[serde(default)]
    pub z: i32,
    /// 命中声明（`action` 取 [`crate::domain::ui_action::UiAction`] 闭集 15 项，`index` 为序号）。
    #[serde(default)]
    pub pick: Option<Pick>,
    /// 可见状态集合（`normal`/`selected`/`hover`/`disabled`）。
    #[serde(default)]
    pub state: Vec<String>,
    /// 文本档位（`title`/`body`/`hint`）。
    #[serde(default)]
    pub tier: String,
    /// 运行时可见性条件（白名单 path + op；S87.2 渲染器按此求值）。
    #[serde(default)]
    pub visible_when: Option<VisibleWhen>,
    /// 代码时间轴名（动画引用；动画本体保留 Rust 过程式）。
    #[serde(default)]
    pub anim_ref: String,
    /// 阶段可见性（vs 分 phase 分组：角色基准落点只在指定阶段出现）。
    #[serde(default)]
    pub visible_in_phase: Vec<String>,
}

/// 几何引用：`{source, ref}`——v6 布局 JSON / `scenes.json` 为几何单源，schema 只引用不吸收。
#[derive(Debug, Clone, Deserialize)]
pub struct Geometry {
    /// 源文件名（`main_map_v6_layout_landscape.json` / `scenes.json`）。
    pub source: String,
    /// 源内键路径（如 `tiles[3]` / `main_map.hud_plaque_l`；`ref` 是 Rust 关键字故 rename）。
    #[serde(rename = "ref")]
    pub reference: String,
}

/// 世界坐标矩形（1920×1080 逻辑像素；仅允许出现在 `world_rect` 内）。
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct WorldRect {
    /// 左上 x。
    pub x: i32,
    /// 左上 y。
    pub y: i32,
    /// 宽。
    pub w: i32,
    /// 高。
    pub h: i32,
}

/// 命中声明：`pick: {action, index}`。
#[derive(Debug, Clone, Deserialize)]
pub struct Pick {
    /// 动作名（UiAction 闭集）。
    pub action: String,
    /// 元素序号。
    pub index: u32,
}

/// 元素绑定：i18n key 字符串或宿主运行值路径。
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum BindingSpec {
    /// 静态 i18n key（如 `scene.main_map.title`）。
    I18n(String),
    /// 运行期取值：`{kind:"host",path,fmt}`，path 限 `SceneHost` 白名单。
    Host {
        /// 固定为 `"host"`。
        kind: String,
        /// 白名单访问器路径（如 `EconomyHost.gold`）。
        path: String,
        /// 格式化串（i18n key 或空串）。
        #[serde(default)]
        fmt: String,
    },
}

/// 运行时可见性条件：`{path, op, value?}`。
#[derive(Debug, Clone, Deserialize)]
pub struct VisibleWhen {
    /// 白名单访问器路径（如 `MapHost.handoff_pending`）。
    pub path: String,
    /// 判定操作（`is_none`/`is_some`/`eq`/`ne`）。
    pub op: String,
    /// `eq`/`ne` 的比较值。
    #[serde(default)]
    pub value: Option<String>,
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

static WORLD_COMPONENTS: OnceLock<BTreeMap<String, WorldScene>> = OnceLock::new();

/// 加载世界层组件声明表（首次解析，之后命中缓存；S87.1，当前无消费者）。
///
/// 与 [`components`] 独立 `OnceLock`：世界层只描述 `vs`/`main_map` 静态元素，
/// 几何走 v6 layout.json / `scenes.json` 引用，动画/数值不在表内。
/// 解析失败 = 硬错误（[`LoadError`]），不 fallback。
///
/// # Errors
/// JSON 语法错误 / 缺必填字段 / `schema_version != 1` / `scenes` 为空时返回 [`LoadError`]。
pub fn world_components() -> Result<&'static BTreeMap<String, WorldScene>, LoadError> {
    if let Some(v) = WORLD_COMPONENTS.get() {
        return Ok(v);
    }
    let raw = injected_world_text().unwrap_or_else(|| WORLD_COMPONENTS_JSON.to_owned());
    let file: WorldSchemaFile = parse(&raw, "world_components.json")?;
    if file.schema_version != 1 {
        return Err(LoadError::Schema(format!(
            "world_components.json: schema_version = {} (expect 1)",
            file.schema_version
        )));
    }
    if file.scenes.is_empty() {
        return Err(LoadError::Schema("world_components.json: scenes is empty".to_string()));
    }
    Ok(WORLD_COMPONENTS.get_or_init(|| file.scenes))
}
