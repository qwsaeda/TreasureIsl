//! S15 UI 重设计：羊皮纸/木纹/金色系**共享视觉外壳**（7 场景复用）。
//!
//! 绘制函数已抽取到 [`ti_render::shell`] / [`ti_render::primitives`] / [`ti_render::iso`]，
//! 本模块保留：
//! - 状态管理（翻页/按下闪烁动画帧计数）
//! - 场景外壳组合函数（`scene_base` / `scene_frame` 拆分壳 + `centered_title_x` 等；
//!   `scene_shell` 旧组合入口已随原语化退役，飞轮 R1 ⑦-10）
//! - 颜色/字号常量重导出（从 [`ti_render::Theme`] 默认值，兼容旧导入路径）
//!
//! 坐标系：1080×1920 呈现层像素（`menu_shell::LOGICAL_W_I32`/`LOGICAL_H_I32`）。

use std::sync::atomic::{AtomicU32, Ordering};
use ti_core::data;

use ti_render::canvas::Canvas;
use ti_render::framebuffer::LOGICAL_W_I32;
use ti_render::theme::Theme;
// Canvas trait：文本宽度度量（`title_text_width`/`text_width`）在 trait 上，需在作用域内。

use crate::scenes::{RenderHost, SceneHost};
use ti_core::data::{IconDef, Primitive, icon_ids, ui_icons};
use ti_core::rules::board::SpaceKind;

// === 颜色常量重导出（从 Theme 默认值，兼容旧导入路径） ===

/// 深底上的**白文字**色。
pub const COLOR_TEXT_ON_PARCH: u16 = Theme::DEFAULT.text_on_parch;
/// 标题条上的**亮文字**色（象牙金）。
pub const COLOR_TEXT_ON_WOOD: u16 = Theme::DEFAULT.text_on_wood;
/// 危险/BOSS 标记色。
pub const COLOR_DANGER: u16 = Theme::DEFAULT.danger;
/// 玩家棋子/单位色。
pub const COLOR_PLAYER: u16 = Theme::DEFAULT.player;
/// 据点标记色。
pub const COLOR_LANDMARK: u16 = Theme::DEFAULT.landmark;
/// 背景填充色。
pub const COLOR_BG: u16 = Theme::DEFAULT.bg;
/// 标题条主色。
pub const COLOR_TITLE_BAR: u16 = Theme::DEFAULT.title_bar;
/// 选中高亮色。
pub const COLOR_SELECTED: u16 = Theme::DEFAULT.selected;
/// 次文字/提示行色。
pub const COLOR_SECONDARY: u16 = Theme::DEFAULT.secondary;
/// 卡片亮（bevel 凸起）。
pub const COLOR_BEVEL_HI: u16 = Theme::DEFAULT.bevel_hi;
/// 暗角（bevel 凹陷）。
pub const COLOR_BEVEL_SHADOW: u16 = Theme::DEFAULT.bevel_shadow;
/// 卡片底（菜单项色块）。
pub const COLOR_MENU_BG: u16 = Theme::DEFAULT.menu_bg;
/// 卡片描边（菜单项边框）。
pub const COLOR_MENU_BORDER: u16 = Theme::DEFAULT.menu_border;
/// 进度条底。
pub const COLOR_HILITE: u16 = Theme::DEFAULT.hilite;
/// 深底。
pub const COLOR_PARCH_DARK: u16 = Theme::DEFAULT.parch_dark;
/// 深底亮。
pub const COLOR_PARCH_LIGHT: u16 = Theme::DEFAULT.parch_light;
/// 卡片描边/标题条。
pub const COLOR_WOOD: u16 = Theme::DEFAULT.wood;
/// 金高光。
pub const COLOR_GOLD: u16 = Theme::DEFAULT.gold;
/// 航海蓝亮。
pub const COLOR_GOLD_GLOW: u16 = Theme::DEFAULT.gold_glow;
/// 航海蓝。
pub const COLOR_GOLD_BRIGHT: u16 = Theme::DEFAULT.gold_bright;
/// 清屏黑。
pub const COLOR_BLACK: u16 = Theme::DEFAULT.black;
/// S18.6 暖木色（Agnes 风格木框；暖色系令牌）。
pub const COLOR_WOOD_WARM: u16 = Theme::DEFAULT.wood_warm;
/// S18.6 暖金（Agnes 风格标题金；暖色系令牌）。
pub const COLOR_GOLD_WARM: u16 = Theme::DEFAULT.gold_warm;
/// 深暖面板底色（S18.10 中景换色）
pub const COLOR_PANEL_WARM: u16 = Theme::DEFAULT.panel_warm;
/// 黄铜描边/甲板中层（S18.10）
pub const COLOR_BRASS: u16 = Theme::DEFAULT.brass;
/// 黄铜凸边/甲板亮层（S18.10）
pub const COLOR_BRASS_LIGHT: u16 = Theme::DEFAULT.brass_light;
/// 黄铜凹边/槽底/甲板暗层（S18.10）
pub const COLOR_BRASS_DARK: u16 = Theme::DEFAULT.brass_dark;
/// 深海水色（S18.9 地形瓦片）
pub const COLOR_SEA_DEEP: u16 = Theme::DEFAULT.sea_deep;
/// 浅海水色（S18.9 岛缘浅滩）
pub const COLOR_SEA_SHALLOW: u16 = Theme::DEFAULT.sea_shallow;
/// 礍石色（S18.9 岛缘礍岩）
pub const COLOR_REEF: u16 = Theme::DEFAULT.reef;
/// 陆地沙色（S18.9 岛屿）
pub const COLOR_LAND_SAND: u16 = Theme::DEFAULT.land_sand;
/// 陆地浅沙（S18.9 瓦片顶棱）
pub const COLOR_LAND_SAND_LIGHT: u16 = Theme::DEFAULT.land_sand_light;
/// 陆地暗沙（S18.9 瓦片交错）
pub const COLOR_LAND_SAND_DARK: u16 = Theme::DEFAULT.land_sand_dark;
/// S18.6 夜空深蓝（Agnes 风格夜空渐变底色）。
pub const COLOR_SKY_NAVY: u16 = Theme::DEFAULT.sky_navy;

// === 动画状态管理 ===
//
// S17.4 动效参数表（数值源 = `docs/process/10_ui_redesign.md` §7；doccheck2
// `check_ui_redesign_tokens` 断言本表常量 == 文档表）：全部**时间制（ms）**，
// 帧率无关（旧实现按帧计数，30fps 下动效拖长一倍）。

/// 场景切换交叉淡化时长（ms，ease-in-out）。
pub const MOTION_TRANSITION_MS: u32 = 240;
/// 按压闪光时长（ms，线性衰减）。
pub const MOTION_PRESS_FLASH_MS: u32 = 120;
/// 血条数值插值时长（ms，线性趋近目标）。
pub const MOTION_HP_LERP_MS: u32 = 160;
/// 翻页亮度渐入时长（ms，线性）。
pub const MOTION_PAGE_MS: u32 = 120;
/// 翻页亮度**下限**（飞轮 R1 ①-04：动效参数表登记，`gate_data.json ui_motion` 锁定）。
pub const MOTION_PAGE_BRIGHTNESS_MIN: f32 = 0.6;
/// 翻页亮度**幅值**（`下限 + 已历时占比 × 幅值` → 1.0；同上源）。
pub const MOTION_PAGE_BRIGHTNESS_SWING: f32 = 0.4;

static PAGE_MS_LEFT: AtomicU32 = AtomicU32::new(0);
static PRESS_FLASH_MS_LEFT: AtomicU32 = AtomicU32::new(0);

/// 触发按下闪烁（持续 [`MOTION_PRESS_FLASH_MS`]）。
pub fn flash_press() {
    PRESS_FLASH_MS_LEFT.store(MOTION_PRESS_FLASH_MS, Ordering::Relaxed);
}

/// 每帧推进按下闪烁倒计时（`delta_ms` = 本帧经过毫秒）。
pub fn tick_press_flash(delta_ms: u32) {
    let mut cur = PRESS_FLASH_MS_LEFT.load(Ordering::Relaxed);
    while let Err(got) = PRESS_FLASH_MS_LEFT.compare_exchange_weak(
        cur,
        cur.saturating_sub(delta_ms),
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        cur = got;
    }
}

/// 按下闪烁是否处于可见期。
#[must_use]
pub fn press_flash_active() -> bool {
    PRESS_FLASH_MS_LEFT.load(Ordering::Relaxed) > 0
}

/// 重置翻页动画倒计时（起始为最暗）。
pub fn reset_page_frame() {
    PAGE_MS_LEFT.store(MOTION_PAGE_MS, Ordering::Relaxed);
}

/// 重置全部动画状态。
pub fn reset_animation_state() {
    PAGE_MS_LEFT.store(0, Ordering::Relaxed);
    PRESS_FLASH_MS_LEFT.store(0, Ordering::Relaxed);
}

/// 读某场景的版面数据。
#[must_use]
pub fn scene_layout(scene_id: &str) -> data::SceneLayout {
    data::scenes().ok().and_then(|s| s.get(scene_id)).cloned().unwrap_or_default()
}

/// scenes.json ↔ 代码的**版面键契约**（`main_map` 棋盘 / v3 世界画布簇）。
///
/// 单一事实源：`extra`（矩形键）与 `extra_scalar`（点号标量键，见
/// [`ti_core::data`] 的 `format!("{k}.{fk}")` 扁平化）读站点一律引用本模块 const，
/// **勿再写裸字面量**；改 scenes.json 键名只改这里一处。
pub mod keys {
    /// v2 静态棋盘原点 x（`extra_scalar` 点号键 = scenes.json `main_map.grid.x0`）。
    pub const GRID_X0: &str = "grid.x0";
    /// v2 静态棋盘原点 y（= `main_map.grid.y0`）。
    pub const GRID_Y0: &str = "grid.y0";
    /// v2 静态棋盘 tile 半宽（= `main_map.grid.tile_w`）。
    pub const GRID_TILE_W: &str = "grid.tile_w";
    /// v2 静态棋盘 tile 半高（= `main_map.grid.tile_h`）。
    pub const GRID_TILE_H: &str = "grid.tile_h";
    /// v3 世界画布矩形键（extra = `main_map.world_canvas`）。
    pub const WORLD_CANVAS: &str = "world_canvas";
    /// v3 HUD 顶左金币角矩形键（extra = `main_map.hud_gold`）。
    pub const HUD_GOLD: &str = "hud_gold";
    /// v3 HUD 顶右持有量角矩形键（extra = `main_map.hud_player`）。
    pub const HUD_PLAYER: &str = "hud_player";
    /// v3 底部命令盘「贸易」键（extra = `main_map.deck_trade`）。
    pub const DECK_TRADE: &str = "deck_trade";
    /// v3 底部命令盘「出售」键（extra = `main_map.deck_sell`）。
    pub const DECK_SELL: &str = "deck_sell";
    /// v3 底部命令盘「建造」键（extra = `main_map.deck_build`）。
    pub const DECK_BUILD: &str = "deck_build";
    /// v3 「摇骰前进」主操作按钮（extra = `main_map.deck_spin`，`progression.md` §环岛赛道）。
    pub const DECK_SPIN: &str = "deck_spin";
    /// v3 航程条矩形键（extra = `main_map.voyage_bar`，S19.7 航程进度条）。
    pub const VOYAGE_BAR: &str = "voyage_bar";
    /// v5 中央据点岛矩形键（extra = `main_map.island_home`，`10_main_map_v5_spec.md` §3）。
    pub const ISLAND_HOME: &str = "island_home";
    /// v4 赛道节点牌边长键（`extra_scalar` = `main_map.track_node.size`；占瓦片半宽百分比）。
    pub const TRACK_NODE: &str = "track_node";
    /// v4 玩家船标边长键（`extra_scalar` = `main_map.track_ship.size`；占瓦片半宽百分比）。
    pub const TRACK_SHIP: &str = "track_ship";
    /// v4 顶左铜牌（金币）矩形键（extra = `main_map.hud_plaque_l`，S21.6 满屏海图版）。
    pub const HUD_PLAQUE_L: &str = "hud_plaque_l";
    /// v4 顶右铜牌（船/持有量）矩形键（extra = `main_map.hud_plaque_r`，S21.6）。
    pub const HUD_PLAQUE_R: &str = "hud_plaque_r";
    /// 提示行锚点键（extra = `main_map.hint`；S27.4 v5 = 落格提示签条中心 (540,1760)，三键之下）。
    pub const HINT: &str = "hint";
    /// v2 HUD 经济按钮「贸易」矩形键（extra = `main_map.hud_trade`）。
    pub const HUD_TRADE: &str = "hud_trade";
    /// v2 HUD 经济按钮「出售」矩形键（extra = `main_map.hud_sell`）。
    pub const HUD_SELL: &str = "hud_sell";
    /// v2 HUD 经济按钮「建造」矩形键（extra = `main_map.hud_build`）。
    pub const HUD_BUILD: &str = "hud_build";
    /// v3 实体百分比簇前缀（`extra_scalar` = `main_map.canvas_entities.<key>`）。
    pub const CANVAS_ENTITIES: &str = "canvas_entities";
    /// 实体 v2 size 回退键后缀（`extra_scalar` = `<key>.size`）。
    pub const SIZE_SUFFIX: &str = "size";
    // ── v3 场景叙事装饰层（index/vs；各场景一段，键缺失/非正 = 不画） ──
    /// v3 氛围星空层矩形键（extra = 场景 `ambient_art`）。
    pub const AMBIENT_ART: &str = "ambient_art";
    /// v3 仪式感标题艺术面板矩形键（extra = 场景 `title_art`）。
    pub const TITLE_ART: &str = "title_art";
    /// v3 战斗竞技区装饰矩形键（extra = `vs.battle_arena`）。
    pub const BATTLE_ARENA: &str = "battle_arena";
    /// S18.5 FX 战斗命中火花触发点（extra = `vs.fx_hit`；飞轮 R1 ③-01 数据化）。
    pub const FX_HIT: &str = "fx_hit";
    /// S18.5 FX 金币光触发点（extra = `main_map.fx_coin`；飞轮 R1 ③-02 数据化）。
    pub const FX_COIN: &str = "fx_coin";
    /// v3 星空层屏心净空锚点 x（`extra_scalar` = `ambient_art.center_x`；飞轮 R1 ①-32）。
    pub const AMB_CENTER_X: &str = "ambient_art.center_x";
    /// v3 星空层屏心净空锚点 y（= `ambient_art.center_y`；同上）。
    pub const AMB_CENTER_Y: &str = "ambient_art.center_y";
    /// v3 星空层屏心净空半径（百分数，= `ambient_art.exclude`；同上）。
    pub const AMB_EXCLUDE: &str = "ambient_art.exclude";
    /// index 尾行（用户版信息）矩形键（extra = `index.status`；飞轮 R1 ③-16 键治理并入本模块）。
    pub const STATUS: &str = "status";
}

/// 等距投影：`(row, col)` → 呈现层坐标。
#[must_use]
pub fn iso_cell(row: i32, col: i32, layout: &data::SceneLayout) -> (i32, i32) {
    let s = &layout.extra_scalar;
    let ox = s.get(keys::GRID_X0).copied().unwrap_or(0);
    let oy = s.get(keys::GRID_Y0).copied().unwrap_or(0);
    let tw = s.get(keys::GRID_TILE_W).copied().unwrap_or(0);
    let th = s.get(keys::GRID_TILE_H).copied().unwrap_or(0);
    (ox + (col - row) * tw, oy + (row + col) * th)
}

/// 等距 tile 半宽/半高。
#[must_use]
pub fn iso_tile(layout: &data::SceneLayout) -> (i32, i32) {
    let s = &layout.extra_scalar;
    (s.get(keys::GRID_TILE_W).copied().unwrap_or(0), s.get(keys::GRID_TILE_H).copied().unwrap_or(0))
}

// ── v3 场景叙事 · 世界画布棋盘几何（main_map S17.8；G 依赖运行时拟合） ──────

/// v3 世界画布棋盘几何：`world_canvas` 键在位 → 把 `g×g` 等距棋盘（2:1 菱形）
/// **拟合进画布**（宽 `2*g*tw ≤ 画布宽`、`tw` 8pt 向下取整、画布中心对齐）；
/// 缺失/非正 → 回退 v2 静态 `grid.*`（双写：v3 键缺席即 v2 版面不变）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardGeom {
    /// 棋盘菱形左上参考（`iso_cell_fit(0,0)` 原点）。
    pub x0: i32,
    /// 棋盘顶行 y。
    pub y0: i32,
    /// tile 半宽（v3 拟合 / v2 静态）。
    pub tw: i32,
    /// tile 半高（v3 = tw/2 / v2 静态）。
    pub th: i32,
}

/// 场景是否启用 v3 世界画布（`world_canvas` 键 + 正尺寸）。
#[must_use]
pub fn has_world_canvas(layout: &data::SceneLayout) -> bool {
    layout.extra.get(keys::WORLD_CANVAS).is_some_and(|r| r.w > 0 && r.h > 0)
}

/// 棋盘几何（v3 拟合 / v2 回退，见 [`BoardGeom`] 文档）。
///
/// v3：`tw = (画布宽 / (2*g)) / 8 * 8`（≥8，否则回退 v2），`th = tw/2`，
/// 画布中心 `cx/cy` → `x0 = cx - tw`、`y0 = cy - g*th`（菱形中心对齐画布中心）。
#[must_use]
pub fn board_fit(layout: &data::SceneLayout, g: u32) -> BoardGeom {
    let gi = i32::try_from(g).unwrap_or(0);
    if let Some(c) = layout.extra.get(keys::WORLD_CANVAS) {
        if c.w > 0 && c.h > 0 && gi > 0 {
            let tw = c.w / (2 * gi);
            let tw = (tw / 8) * 8;
            if tw >= 8 {
                let th = tw / 2;
                let cx = c.x + c.w / 2;
                let cy = c.y + c.h / 2;
                return BoardGeom { x0: cx - tw, y0: cy - gi * th, tw, th };
            }
        }
    }
    let s = &layout.extra_scalar;
    BoardGeom {
        x0: s.get(keys::GRID_X0).copied().unwrap_or(0),
        y0: s.get(keys::GRID_Y0).copied().unwrap_or(0),
        tw: s.get(keys::GRID_TILE_W).copied().unwrap_or(0),
        th: s.get(keys::GRID_TILE_H).copied().unwrap_or(0),
    }
}

/// v3 等距投影（G 依赖 `board_fit`）：`(row, col)` → 呈现层坐标。
#[must_use]
pub fn iso_cell_fit(row: i32, col: i32, layout: &data::SceneLayout, g: u32) -> (i32, i32) {
    let bg = board_fit(layout, g);
    (bg.x0 + (col - row) * bg.tw, bg.y0 + (row + col) * bg.th)
}

/// v3 tile 半宽/半高（G 依赖 `board_fit`）。
#[must_use]
pub fn iso_tile_fit(layout: &data::SceneLayout, g: u32) -> (i32, i32) {
    let bg = board_fit(layout, g);
    (bg.tw, bg.th)
}

/// v3 实体边长：`canvas_entities.<key>`（`board_fit.tw` 的百分之几）× tw；
/// 缺失/非正 → 回退 v2 `<key>.size`（QA106 语义，`tw/th` 与实体同数据源不越界）。
#[must_use]
pub fn entity_size_fit(layout: &data::SceneLayout, key: &str, g: u32) -> i32 {
    let s = &layout.extra_scalar;
    let ratio = s.get(&format!("{}.{}", keys::CANVAS_ENTITIES, key)).copied().unwrap_or(0);
    if ratio > 0 {
        let bg = board_fit(layout, g);
        return (bg.tw * ratio / 100).max(4);
    }
    s.get(&format!("{key}.{}", keys::SIZE_SUFFIX)).copied().unwrap_or(0)
}

/// v3 世界画布装饰框（`main_map`）：读 `world_canvas` 矩形，缺失/非正 = 不画。
///
/// 调用序（z-order）：`nautical_grid` 之后、等距瓦片之前（垫底框 + 四角罗盘方位块）。
pub fn world_canvas_frame(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(rc) = layout.extra.get(keys::WORLD_CANVAS) else {
        return;
    };
    if rc.w <= 0 || rc.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::world_canvas_frame(c, &theme, rc.x, rc.y, rc.w, rc.h));
}

/// 分区号 → 主题色令牌（9 分区配色，对齐原版地形类别码 1..9；`legacy/docs/facts/map.md` F-308）。
///
/// 单一事实源（S19.7 D2 收敛 + 飞轮 R1 ①-01 数据化）：赛道标记（`main_map.rs`）与航程条
/// （`chrome::voyage_bar`）共用本表；令牌表主源 = `balance.json board_group_tokens`（可改数据），
/// 代码只做 `tokens[group-1]` dispatch；balance 缺失/空 = 硬编码 9 令牌 crash-safe 回退
/// （同 S21.14 图标代码矢量回退口径）。
#[must_use]
pub fn group_color_token(group: u8) -> &'static str {
    const FALLBACK: [&str; 9] = [
        "land_sand",
        "reef",
        "sea_shallow",
        "gold_warm",
        "moon_pale",
        "brass",
        "secondary",
        "hilite",
        "danger",
    ];
    let i = (usize::from(group).saturating_sub(1)).min(FALLBACK.len() - 1);
    if let Ok(b) = data::balance() {
        let tokens = &b.board_group_tokens;
        if !tokens.is_empty() {
            let j = (usize::from(group).saturating_sub(1)).min(tokens.len() - 1);
            if let Some(t) = tokens.get(j) {
                return t.as_str();
            }
        }
    }
    FALLBACK[i]
}

// === v4 环岛赛道：节点牌资产映射 + 航线进度尺（S21.5） ===
//
// 单一事实源 = `docs/ui-redesign-v3/08_main_map_v4_spec.md` §6：
// `SpaceKind` 9 类 ↔ `assets/ui/node_{start,city,port,event,card,battle,treasure,boss,plain}.png`
// 多形状实心黄铜牌（S21.1 已入仓）；玩家 = `ship_player`（1 级顶替口径，按 `boat_level`
// 切 4 张船图 = S21.12）。

/// 贴图源画布边长（px；S21.1 批次 `medallion` 角色 = 512×512；9 张节点牌 + 玩家船同规格）。
pub const ASSET_SOURCE_PX: i32 = 512;

/// 4 级船贴图（`boat_level - 1` 索引：1..=4 → `boat_{1..4}.png` = 小舢板/双桅/三桅/巡防；
/// S21.12 批次生成，规格 `assets/ui_batches/main_map_v4.json`）。
pub const BOAT_ASSETS: [&str; 4] = [
    "assets/ui/boat_1.png",
    "assets/ui/boat_2.png",
    "assets/ui/boat_3.png",
    "assets/ui/boat_4.png",
];

// === S27.7 VS 对战（vs_v1 批次资产）：场景层禁字面量（CS-1），资产路径单源登记 ===

/// VS 场景卡面面板（圆角白蒙版，`vs_proto` 版式 B/E 手牌 + 面板底）。
pub const VS_CARD_PANEL: &str = "assets/ui/card_panel.png";
/// VS 场景舵轮（回合徽章底 + 道具转盘轴心）。
pub const VS_HELM: &str = "assets/ui/helm.png";
/// 道具图标·火药桶（`vs_v1` 批次；`combat.item.keg`）。
pub const VS_ITEM_KEG: &str = "assets/ui/item_keg.png";
/// 道具图标·医疗包（`vs_v1` 批次；`combat.item.medkit`）。
pub const VS_ITEM_MEDKIT: &str = "assets/ui/item_medkit.png";
/// 道具图标·朗姆酒（`vs_v1` 批次；`combat.item.rum`）。
pub const VS_ITEM_RUM: &str = "assets/ui/item_rum.png";
/// 道具图标·烟雾弹（`vs_v1` 批次；`combat.item.smoke`）。
pub const VS_ITEM_SMOKE: &str = "assets/ui/item_smoke.png";
/// VS 动作图标·攻击（刀）。
pub const VS_ICON_CUTLASS: &str = "assets/ui/icon_cutlass.png";
/// VS 动作图标·防御（盾）。
pub const VS_ICON_SHIELD: &str = "assets/ui/icon_shield.png";
/// VS 通用锚点图标（血环环心）。
pub const VS_ICON_ANCHOR: &str = "assets/ui/icon_anchor.png";
/// BOSS 立绘（`vs_v1` 批次）。
pub const VS_BOSS_CAPTAIN: &str = "assets/ui/boss_captain.png";
/// 玩家背身立绘（`vs_v1` 批次）。
pub const VS_HERO_BACK: &str = "assets/ui/hero_back.png";
/// 夜间甲板背景（版式 A/D）。
pub const VS_DECK_BATTLE: &str = "assets/ui/deck_battle.png";
/// 夜间海面背景（版式 B/E）。
pub const VS_BG_NIGHT_SEA: &str = "assets/ui/bg_night_sea.png";
/// 海战海面背景（版式 C）。
pub const VS_BG_BATTLE_SEA: &str = "assets/ui/bg_battle_sea.png";
/// 玩家船（阵营标识，版式 C）。
pub const VS_SHIP_PLAYER: &str = "assets/ui/ship_player.png";
/// 黄铜指挥台底板（版式 E 底部）。
pub const VS_DECK_PLAQUE: &str = "assets/ui/deck_plaque.png";
/// 角色立绘·基德（版式 E 我方半身像）。
pub const VS_ROLE_KIDD: &str = "assets/ui/role_kidd.png";

/// 玩家船通用图（S21.5 入仓；当前级船图缺失时的回退链第二环 ⇒ crash-safe 不空图）。
pub const SHIP_PLAYER_ASSET: &str = "assets/ui/ship_player.png";

/// S27.5：骰子贴图（`ui_manifest.json` `dice` role=board）。
///
/// 用途 = 摇骰 CTA 轴心骰点（用户 2026-10-28 报「没有筛子」= CTA 只有空舵轮，
/// 看不见骰子 ⇒ 不知能否摇）。资产自 S21.1 入仓但此前**零消费**。
pub const DICE_ASSET: &str = "assets/ui/dice.png";

/// S27.5：摇骰 CTA 骰点图边长占 `deck_spin` 矩形边长的百分比（轴心居中）。
///
/// 数值口径 = 264px × 30% ≈ 79px（8pt 栅格取整）。
pub const DICE_EDGE_RATIO: i32 = 30;

/// 按船等级取玩家船贴图（等级越界 = 1 级通用船）。
#[must_use]
pub fn boat_asset(boat_level: u8) -> &'static str {
    let idx = usize::from(boat_level.saturating_sub(1)).min(BOAT_ASSETS.len() - 1);
    BOAT_ASSETS[idx]
}

/// S21.12 建造面板：面板框（1080×1920 居中；768×640 @ (144,640)，8pt 栅格 8 倍数）。
/// 布局 = 标题 64 + 条目 5×96 + 底部提示 96 = 640。
pub const BUILD_PANEL: (i32, i32, i32, i32) = (144, 640, 768, 640);
/// 面板标题条高度。
pub const BUILD_PANEL_HEADER_H: i32 = 64;
/// 单条目的命中/绘制高度（5 条：城堡/投资/银行/要塞/造船）。
pub const BUILD_PANEL_ENTRY_H: i32 = 96;
/// 条目内边距（左右 24；条目命中框 = 面板内缩 24，条内垂直留白 8）。
pub const BUILD_PANEL_PAD: i32 = 24;
/// 面板条目数（4 建筑 + 造船第 5 入口，`economy.md` §船系统 规则 1）。
pub const BUILD_PANEL_ENTRIES: usize = 5;
/// 末条目序号（u8 选中域 0..=4；面板选中状态与键盘钳制用）。
pub const BUILD_LAST_ENTRY: u8 = 4;

/// S21.12 建造面板条目图标（0..3 = 城堡/投资/银行/要塞白蒙版图标；资产路径单一读点，§1 禁硬编码）。
pub const BUILD_ENTRY_ICONS: [&str; 4] = [
    "assets/ui/icon_castle.png",
    "assets/ui/icon_coin.png",
    "assets/ui/icon_chest.png",
    "assets/ui/icon_beacon.png",
];

/// 面板第 `entry` 条目的命中框（8pt 栅格；`entry` ≥ 条目数 = 末条）。
#[must_use]
pub fn build_entry_rect(entry: usize) -> (i32, i32, i32, i32) {
    let (bx, by, bw, _bh) = BUILD_PANEL;
    let clamped = entry.min(BUILD_PANEL_ENTRIES - 1);
    let row_y =
        by + BUILD_PANEL_HEADER_H + BUILD_PANEL_ENTRY_H * i32::try_from(clamped).unwrap_or(0);
    (bx + BUILD_PANEL_PAD, row_y + 8, bw - BUILD_PANEL_PAD * 2, BUILD_PANEL_ENTRY_H - 16)
}

/// 摇骰点数标签（`board_last_roll` > 0 时画在舵轮 CTA 上方；`scene.main_map.toast.roll_fmt`）。
#[must_use]
pub fn roll_label(roll: u32) -> String {
    crate::i18n::sgetf("scene.main_map.toast.roll_fmt", &[&roll])
}

/// 玩家船可见外接框边长（源像素；S21.1 批次抠图后 alpha 包围盒实测）。
pub const SHIP_PLAYER_VISIBLE_PX: i32 = 477;

/// 节点牌视觉边长缺省百分比（占瓦片半宽；`main_map.track_node.size` 缺失时的回退）。
const NODE_EDGE_PCT_FALLBACK: i32 = 100;

/// 玩家船视觉边长缺省百分比（占瓦片半宽；`main_map.track_ship.size` 缺失时的回退）。
const SHIP_EDGE_PCT_FALLBACK: i32 = 150;

/// v4 航线进度尺：左右内边距（px；8pt 栅格）。
const VOYAGE_PAD: i32 = 24;
/// S26.2 航线进度尺：黄铜槽厚度（px；8pt 栅格）。
const VOYAGE_RAIL_H: i32 = 8;
/// S26.2 航线进度尺：每格点边长（px；8pt 栅格）。
const VOYAGE_DOT: i32 = 12;

/// 节点牌资产（kind → 贴图；**唯一事实源** = 规格 §6）。
#[derive(Debug, Clone, Copy)]
pub struct NodeAsset {
    /// 资产路径（相对仓库根；`blit_image` 口径）。
    pub path: &'static str,
    /// 抠图后可见外接框边长（源像素）——等比缩放到统一视觉边长用。
    pub visible_px: i32,
}

/// 节点 kind → 节点牌资产（**唯一事实源**，S21.5；规格 `08_main_map_v4_spec.md` §6）。
///
/// 9 类 kind ↔ 9 张多形状黄铜牌（圆/六边/圆角方/盾/菱/尖星/宝箱/骷髅/铜钉）；
/// 缺资产 = `blit_image` 静默不画（不 panic、不占位）。
#[must_use]
pub fn node_asset(kind: SpaceKind) -> NodeAsset {
    match kind {
        SpaceKind::Start => NodeAsset { path: "assets/ui/node_start.png", visible_px: 488 },
        SpaceKind::City => NodeAsset { path: "assets/ui/node_city.png", visible_px: 352 },
        SpaceKind::Port => NodeAsset { path: "assets/ui/node_port.png", visible_px: 324 },
        SpaceKind::Event => NodeAsset { path: "assets/ui/node_event.png", visible_px: 471 },
        SpaceKind::Card => NodeAsset { path: "assets/ui/node_card.png", visible_px: 314 },
        SpaceKind::Battle => NodeAsset { path: "assets/ui/node_battle.png", visible_px: 468 },
        SpaceKind::Treasure => NodeAsset { path: "assets/ui/node_treasure.png", visible_px: 443 },
        SpaceKind::Boss => NodeAsset { path: "assets/ui/node_boss.png", visible_px: 334 },
        SpaceKind::Plain => NodeAsset { path: "assets/ui/node_plain.png", visible_px: 182 },
        SpaceKind::Territory => NodeAsset { path: "assets/ui/node_plain.png", visible_px: 314 },
    }
}

/// S26.2 岛屿图片化资产（地图 id → 资产路径；`scripts/ui_islands.py` 按 `terrain.json` +
/// `world_canvas` 同几何生成 3 张，`ui_manifest.json` 登记；缺 = 调用方回退代码绘制）。
#[must_use]
pub fn island_asset(map_id: &str) -> Option<&'static str> {
    Some(match map_id {
        "m01" => "assets/ui/island_m01.png",
        "m02" => "assets/ui/island_m02.png",
        "m03" => "assets/ui/island_m03.png",
        _ => return None,
    })
}

// ── S27.4 v5 环形棋盘（`10_main_map_v5_spec.md` §2） ─────────────────────────────
// 几何 = **数据驱动**：`main_map.ring.{center_x,center_y,radius,card_w,card_h}`（scenes.json
// `extra_scalar` 点号键）；**换图 / 调环几何只改数据，不改 Rust**。下述常量仅作数据缺键回退
// （项目「缺键 = 回退」惯例），非单一事实源。

/// 环心缺省（`540, 784`；`main_map.ring.center_x/center_y` 缺失时回退；8pt 栅格，屏心 x=540 豁免）。
pub const RING_CENTER: (i32, i32) = (540, 784);
/// 环半径缺省（`480`；`main_map.ring.radius` 缺失时回退；`2πr/31 ≈ 97` ⇒ 相邻 96 宽卡留 1px 缝）。
pub const RING_RADIUS: i32 = 480;
/// 卡宽缺省（`96`；`main_map.ring.card_w` 缺失时回退）。
pub const RING_CARD_W: i32 = 96;
/// 卡高缺省（`120`；`main_map.ring.card_h` 缺失时回退）。
pub const RING_CARD_H: i32 = 120;

/// v5 环形几何（读 `main_map.ring.{center_x,center_y,radius,card_w,card_h}`，缺键 = 常量回退）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingGeom {
    /// 环心 x（1080 坐标系）。
    pub cx: i32,
    /// 环心 y（1080 坐标系）。
    pub cy: i32,
    /// 环半径（卡中心落此圆；`2πr/格数` 决定相邻卡间距）。
    pub r: i32,
    /// 卡宽（px）。
    pub w: i32,
    /// 卡高（px）。
    pub h: i32,
}

/// 读环形几何（`extra_scalar` 点号键；缺键 = [`RING_CENTER`]/[`RING_RADIUS`]/[`RING_CARD_W`] 回退）。
#[must_use]
pub fn ring_geom(layout: &data::SceneLayout) -> RingGeom {
    let s = &layout.extra_scalar;
    RingGeom {
        cx: s.get("ring.center_x").copied().unwrap_or(RING_CENTER.0),
        cy: s.get("ring.center_y").copied().unwrap_or(RING_CENTER.1),
        r: s.get("ring.radius").copied().filter(|v| *v > 0).unwrap_or(RING_RADIUS),
        w: s.get("ring.card_w").copied().filter(|v| *v > 0).unwrap_or(RING_CARD_W),
        h: s.get("ring.card_h").copied().filter(|v| *v > 0).unwrap_or(RING_CARD_H),
    }
}

/// 环形坐标取整（f64 → i32；钳制到 ±2.1e7 后转换，`rules::battle` 同口径的局部 allow）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn round_coord(value: f64) -> i32 {
    i32::try_from(value.round().clamp(-21_474_836.0, 21_474_836.0) as i64).unwrap_or(0)
}

/// 第 `index` 张卡中心（`θ = -90° + index·(360°/count)`，index=0 = 顶部起点，顺时针；
/// `count` = 赛道格数；几何 = [`ring_geom`]）。
#[must_use]
pub fn ring_card_center(index: usize, count: usize, layout: &data::SceneLayout) -> (i32, i32) {
    let g = ring_geom(layout);
    let count_f = f32::from(u16::try_from(count.max(1)).unwrap_or(1));
    let index_f = f32::from(u16::try_from(index).unwrap_or(0));
    let radius_f = f32::from(i16::try_from(g.r).unwrap_or(0));
    let rad = (-90.0 + index_f * 360.0 / count_f) * std::f32::consts::PI / 180.0;
    let dx = round_coord(f64::from(radius_f) * f64::from(rad.cos()));
    let dy = round_coord(f64::from(radius_f) * f64::from(rad.sin()));
    (g.cx + dx, g.cy + dy)
}

/// 第 `index` 张卡矩形（卡宽高 = [`ring_geom`]，中心 = [`ring_card_center`]）。
#[must_use]
pub fn ring_card_rect(
    index: usize,
    count: usize,
    layout: &data::SceneLayout,
) -> (i32, i32, i32, i32) {
    let g = ring_geom(layout);
    let (cx, cy) = ring_card_center(index, count, layout);
    (cx - g.w / 2, cy - g.h / 2, g.w, g.h)
}

/// 岛型领地卡面资产路径（**按命名约定** `assets/ui/isle_<isle_id>.png`）。
///
/// `isle_id` = `board.json` 的 `isle` 字段（数据）；资产由 S27.2 脚本按同名生成 ⇒
/// **换图 / 加岛型不改 Rust**（缺图由调用方 `asset_present` 判 → 回退功能牌）。
#[must_use]
pub fn isle_card_path(isle: &str) -> String {
    format!("assets/ui/isle_{isle}.png")
}

/// v5 中央据点岛资产路径（**按命名约定** `assets/ui/island_home_<map_id>.png`）。
///
/// `map_id` 运行时来自 `map_slot()`（数据/运行态）⇒ 换图不改 Rust（缺图 = 调用方回退代码圆盘）。
#[must_use]
pub fn central_island_path(map_id: &str) -> String {
    format!("assets/ui/island_home_{map_id}.png")
}

/// S27.5 通用圆角卡板白蒙版（`card_v1` 批次生成；运行时 `blit_image_rot` + tint 上色）。
///
/// 一份蒙版吃三种卡面：领地卡底（parchment 系）、功能卡底（`panel_wood`）、归属色带（`isle_*`）。
pub const CARD_PANEL_ASSET: &str = "assets/ui/card_panel.png";

/// S27.5 第 `index` 张卡的径向旋转（**毫度**，屏幕坐标系顺时针；`0` = 顶部起点朝上）。
///
/// 概念稿变体 B 的环卡语言：卡「上缘」指向环外 ⇒ 旋转角 = 该格在环上的方位角
/// （`index · 360° / count`）。`3/9 点位置` 卡自然转 90°，`底部`转 180°（径向一致）。
#[must_use]
pub fn ring_card_rotation_mdeg(index: usize, count: usize) -> i32 {
    let count_i = i32::try_from(count.max(1)).unwrap_or(1);
    let index_i = i32::try_from(index).unwrap_or(0);
    index_i.saturating_mul(360_000) / count_i
}

/// 岛型 id → 显示名（`isle.<id>` i18n；缺键回退原 id，不 panic）。
#[must_use]
pub fn isle_name(isle: &str) -> String {
    let key = format!("isle.{isle}");
    let name = crate::i18n::sget(&key);
    if name == key { isle.to_owned() } else { name.to_owned() }
}

/// 赛道格类型 → 显示名（`scene.main_map.node.<kind>` i18n；功能格无岛型时用）。
#[must_use]
pub fn space_name(kind: SpaceKind) -> String {
    let key = match kind {
        SpaceKind::Start => "scene.main_map.node.start",
        SpaceKind::City => "scene.main_map.node.city",
        SpaceKind::Port => "scene.main_map.node.port",
        SpaceKind::Event => "scene.main_map.node.event",
        SpaceKind::Card => "scene.main_map.node.card",
        SpaceKind::Battle => "scene.main_map.node.battle",
        SpaceKind::Treasure => "scene.main_map.node.treasure",
        SpaceKind::Boss => "scene.main_map.node.boss",
        SpaceKind::Plain => "scene.main_map.node.plain",
        SpaceKind::Territory => "scene.main_map.node.territory",
    };
    crate::i18n::sget(key).to_owned()
}

/// 局部偏移 → 屏幕坐标（S27.5：旋转卡上的子元素定位）。
///
/// 输入 `(cx, cy)` = 卡心，`(dx, dy)` = 子元素相对卡心的**卡的局部偏移**（+y = 卡的下方），
/// `rot_mdeg` = 卡的顺时针旋转角（毫度，屏幕坐标系 y 向下）。返回旋转后的屏幕坐标：
/// 局部 +y 方向映射为 `(−sinθ, cosθ)`（θ = 90° 时卡的「下方」朝屏幕左侧 = 指向环心）。
#[must_use]
pub fn rotate_local_offset(cx: i32, cy: i32, dx: i32, dy: i32, rot_mdeg: i32) -> (i32, i32) {
    let theta = f64::from(rot_mdeg) * 0.001 * std::f64::consts::PI / 180.0;
    let (sin, cos) = theta.sin_cos();
    let fx = f64::from(dx);
    let fy = f64::from(dy);
    let rx = fx * cos - fy * sin;
    let ry = fx * sin + fy * cos;
    (cx.saturating_add(round_coord(rx)), cy.saturating_add(round_coord(ry)))
}

/// 等比贴图目标矩形（**可见外接框归一化**）：可见边长 = `edge_px`，中心 (cx, cy)，无形变。
///
/// 9 张节点牌的 alpha 包围盒差异大（182~488 px），按整幅 512×512 拉伸会让「铜钉」牌缩成
/// 一小点 ⇒ 统一按可见外接框等比缩放，9 类牌视觉大小一致、形状各异。
#[must_use]
pub fn asset_dest_rect(cx: i32, cy: i32, edge_px: i32, visible_px: i32) -> (i32, i32, i32, i32) {
    let side = (ASSET_SOURCE_PX * edge_px / visible_px.max(1)).max(1);
    (cx - side / 2, cy - side / 2, side, side)
}

/// 贴图是否在位（缺资产 = 回退代码矢量图标的唯一判据；缺资源 = 不画，不 panic）。
#[must_use]
pub fn asset_present(asset: &str) -> bool {
    #[cfg(feature = "hd-8bit")]
    {
        crate::image_cache::get(asset).is_some()
    }
    #[cfg(not(feature = "hd-8bit"))]
    {
        let _ = asset;
        true
    }
}

/// S21.7 节点高亮环（`gold_warm` 2px 外圈，节点牌可见框每侧外扩 3px；预览选中 / 落点脉冲共用）。
pub fn draw_node_ring(
    host: &mut dyn SceneHost,
    cx: i32,
    cy: i32,
    edge: i32,
    visible_px: i32,
    color: u16,
) {
    let (dx, dy, dw, dh) = asset_dest_rect(cx, cy, edge, visible_px);
    let r = 3;
    host.fill_rect(dx - r, dy - r, dw + 2 * r, 2, color);
    host.fill_rect(dx - r, dy + dh + r - 2, dw + 2 * r, 2, color);
    host.fill_rect(dx - r, dy - r, 2, dh + 2 * r, color);
    host.fill_rect(dx + dw + r - 2, dy - r, 2, dh + 2 * r, color);
}

/// 画布相对边长（`extra_scalar` = `<key>.size`，**占瓦片半宽的百分比** × `board_fit.tw`）。
///
/// 百分比口径 = 随棋盘缩放（v3 `world_canvas` tw=24 ⇒ 1× = 24px；v4 tw=48 ⇒ 1× = 48px）；
/// 相邻赛道格中心距 = 1 个瓦片半宽 ⇒ 100% 即「牌与牌刚好相接」。
fn track_edge_px(layout: &data::SceneLayout, g: u32, key: &str, fallback_pct: i32) -> i32 {
    let pct = layout
        .extra_scalar
        .get(&format!("{key}.{}", keys::SIZE_SUFFIX))
        .copied()
        .filter(|v| *v > 0)
        .unwrap_or(fallback_pct);
    let tw = board_fit(layout, g).tw;
    (tw * pct / 100).max(1)
}

/// v4 节点牌可见边长（px）：`main_map.track_node.size`（瓦片半宽百分比）× `board_fit.tw`。
#[must_use]
pub fn node_edge_px(layout: &data::SceneLayout, g: u32) -> i32 {
    track_edge_px(layout, g, keys::TRACK_NODE, NODE_EDGE_PCT_FALLBACK)
}

/// v4 玩家船可见边长（px）：`main_map.track_ship.size`（瓦片半宽百分比）× `board_fit.tw`。
#[must_use]
pub fn ship_edge_px(layout: &data::SceneLayout, g: u32) -> i32 {
    track_edge_px(layout, g, keys::TRACK_SHIP, SHIP_EDGE_PCT_FALLBACK)
}

/// 进度尺槽位中心 x（第 `i` 格）。
fn voyage_slot_center(inner_x: i32, inner_w: i32, count: i32, i: i32) -> i32 {
    inner_x + inner_w * (2 * i + 1) / (2 * count.max(1))
}

/// S21.5/S26.2 v4 航线进度尺（`main_map.voyage_bar`）：把环岛赛道**摊平**为
/// **1 条黄铜槽 + 每格一色点（分区色）+ 玩家船标 + BOSS 终标**（规格 §2；
/// S26.2 起 9 扇区色带 → 每格点，消除「一排裸色块」观感）。
///
/// 业务依据（`docs/business/progression.md` §环岛赛道）：赛道 = 岛屿外环的有序空间序列；
/// 本条 = 「绕岛进度」的可视化载体（分区 = `board.json` 的 `group` 1..9）。
///
/// 画法：底板（`brass_dark`，`panel_warm` 在深海底上隐形）+ 黄铜槽 + 每格点（分区色）
/// + 玩家船贴图 + BOSS 骷髅牌（缺贴图 = 代码矢量图标回退）。
///
/// 无赛道（`board_spaces` 空）/ 键缺失 = 不画。
pub fn voyage_bar(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(rect) = layout.extra.get(keys::VOYAGE_BAR) else {
        return;
    };
    if rect.w <= 0 || rect.h <= 0 {
        return;
    }
    let spaces = host.board_spaces();
    if spaces.len() < 2 {
        return;
    }
    let count = i32::try_from(spaces.len()).unwrap_or(0);
    let player_idx = i32::try_from(host.board_index()).unwrap_or(0).min(count - 1);
    let theme = host.theme().clone();
    // 底板用 `brass_dark`（`panel_warm` 在深海底上≈隐形 —— RGB565 (24,24,16) 与底图同色）
    let panel = theme.by_name("brass_dark").unwrap_or(COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(COLOR_GOLD_WARM);
    let brass_dark = theme.by_name("brass_dark").unwrap_or(COLOR_GOLD_WARM);
    let danger = crate::menu_shell::COLOR_DANGER;

    // 底板 + 黄铜边（8pt 栅格：x/y/w/h 已整除）
    host.fill_rect(rect.x, rect.y, rect.w, rect.h, panel);
    host.fill_rect(rect.x, rect.y, rect.w, 2, brass);
    host.fill_rect(rect.x, rect.y + rect.h - 2, rect.w, 2, brass_dark);
    let track_y = rect.y + rect.h / 2;
    let inner_x = rect.x + VOYAGE_PAD;
    let inner_w = rect.w - VOYAGE_PAD * 2;
    if inner_w <= 0 {
        return;
    }
    // 1 条黄铜槽（环岛摊平 = 水平细槽，8pt 栅格）
    host.fill_rect(inner_x, track_y - VOYAGE_RAIL_H / 2, inner_w, VOYAGE_RAIL_H, brass);

    // S26.2：每格一点（分区色；替代 9 扇区色带 —— 粗色块观感消除）
    for (i, sp) in spaces.iter().enumerate() {
        let c = theme.by_name(group_color_token(sp.group)).unwrap_or(COLOR_GOLD_WARM);
        let cx = voyage_slot_center(inner_x, inner_w, count, i32::try_from(i).unwrap_or(0));
        host.fill_rect(cx - VOYAGE_DOT / 2, track_y - VOYAGE_DOT / 2, VOYAGE_DOT, VOYAGE_DOT, c);
    }

    // 玩家船标（船贴图；缺资产 = 代码 `compass` 图标回退）
    // S21.12：船贴图按 `boat_level` 切 4 张（1 级 = 通用船顶替）。
    let marker = (rect.h / 2).clamp(16, 32);
    let player_x = voyage_slot_center(inner_x, inner_w, count, player_idx);
    let (sx, sy, sw, sh) = asset_dest_rect(player_x, track_y, marker, SHIP_PLAYER_VISIBLE_PX);
    let ship_asset = boat_asset(host.boat_level());
    if asset_present(ship_asset) {
        host.blit_image(ship_asset, sx, sy, sw, sh);
    } else if asset_present(SHIP_PLAYER_ASSET) {
        host.blit_image(SHIP_PLAYER_ASSET, sx, sy, sw, sh);
    } else {
        entity_icon(host, "compass", sx, sy, sw, COLOR_GOLD_WARM);
    }

    // BOSS 终标（骷髅牌；缺资产 = 代码 `sword` 图标回退）
    if let Some(boss_i) = spaces.iter().position(|s| s.kind == SpaceKind::Boss) {
        let boss = node_asset(SpaceKind::Boss);
        let boss_x =
            voyage_slot_center(inner_x, inner_w, count, i32::try_from(boss_i).unwrap_or(0));
        let (bx, by, bw, bh) = asset_dest_rect(boss_x, track_y, marker, boss.visible_px);
        if asset_present(boss.path) {
            host.blit_image(boss.path, bx, by, bw, bh);
        } else {
            entity_icon(host, "sword", bx, by, bw, danger);
        }
    }
}

/// 统一面板 chrome（v4 面板语言，S20.2）：**黄铜细边 + 暖棱**。
///
/// 底色由 `scenes.json` 原语给（`panel_wood` = 深棕木底，替代 `panel_warm` 的近黑）；
/// 本函数只补边与棱（代码侧统一 ⇒ 三个场景的面板语言同源，改一处即全局生效）。
///
/// 画法：黄铜 2px 四边 + 内上 `brass_light` 1px + 内下 `brass_dark` 1px
/// （与 `main_map` HUD 面板同款做法，见 `08_layout_design.md` §版面骨架）。
pub fn wood_panel(host: &mut dyn SceneHost, x: i32, y: i32, w: i32, h: i32) {
    if w <= 4 || h <= 4 {
        return;
    }
    let theme = host.theme().clone();
    let brass = theme.by_name("brass").unwrap_or(COLOR_GOLD_WARM);
    let brass_light = theme.by_name("brass_light").unwrap_or(COLOR_GOLD_WARM);
    let brass_dark = theme.by_name("brass_dark").unwrap_or(COLOR_GOLD_WARM);
    host.fill_rect(x, y, w, 2, brass);
    host.fill_rect(x, y + h - 2, w, 2, brass);
    host.fill_rect(x, y, 2, h, brass);
    host.fill_rect(x + w - 2, y, 2, h, brass);
    host.fill_rect(x + 2, y + 2, w - 4, 1, brass_light);
    host.fill_rect(x + 2, y + h - 3, w - 4, 1, brass_dark);
}

/// 选中态边框（v4 面板语言，S20.2）：**3px 黄铜边**（此前 = 整块金色填充，刺眼且抢戏）。
///
/// S26.3：外圈改用 `gold_warm`（亮金，深面板上可见；旧 `brass` 中调与 `wood_panel` 边框
/// 同色 ⇒ 存读档 5 槽外观全同、选中不可辨）；只画边不填底 ⇒ 面板底色与文字可读性不变。
pub fn selected_border(host: &mut dyn SceneHost, px: i32, py: i32, pw: i32, ph: i32, thick: i32) {
    if pw <= 4 || ph <= 4 {
        return;
    }
    let theme = host.theme().clone();
    let gold_warm = theme.by_name("gold_warm").unwrap_or(COLOR_GOLD_WARM);
    let brass_light = theme.by_name("brass_light").unwrap_or(COLOR_GOLD_WARM);
    let t = thick.max(2);
    host.fill_rect(px - t, py - t, pw + t * 2, t, gold_warm);
    host.fill_rect(px - t, py + ph, pw + t * 2, t, gold_warm);
    host.fill_rect(px - t, py - t, t, ph + t * 2, gold_warm);
    host.fill_rect(px + pw, py - t, t, ph + t * 2, gold_warm);
    host.fill_rect(px - t, py - t, pw + t * 2, 1, brass_light);
}

/// v3 底部命令盘按钮（`main_map`）：读 `<key>` 矩形（`deck_trade`/`deck_sell`/`deck_build`），
/// 面板 + 图标（经 `icon_key` 查 `ui_icons` 注册表，如 `hud_trade`）+ `28px` 标签；缺失/非正 = 不画。
///
/// 图标宽度不足时自动退回纯文字居中（与 `render_hud_button` 同判据）。
pub fn deck_button(
    host: &mut dyn SceneHost,
    scene_id: &str,
    key: &str,
    icon_key: &str,
    label_key: &str,
) {
    let layout = scene_layout(scene_id);
    let Some(r) = layout.extra.get(key) else {
        return;
    };
    if r.w <= 0 || r.h <= 0 {
        return;
    }
    panel(host, r.x, r.y, r.w, r.h);
    let label = crate::i18n::sget(label_key);
    let icon = hud_icon_def(icon_key);
    let rect = *r;
    let (label_x, icon_x) = layout_icon_label(host, &rect, label, icon.as_ref());
    if let Some((id, size, ix)) = icon_x {
        ui_icon(host, id, ix, r.y + (r.h - size) / 2, false);
    }
    let ly = r.y + (r.h - host.shell().hint_glyph_px()) / 2;
    host.draw_hint_text(label, label_x, ly, COLOR_TEXT_ON_PARCH);
}

/// HUD/deck 按钮「图标 + 标签」排布（**单一事实源公式**，飞轮 R1 ⑥-03：`deck_button` 与
/// `main_map::render_hud_button` 双调用点收口）：宽足（`r.w ≥ size + 8`）→ 图标左置
/// （`x + 8`）+ 标签起于图标右缘；不足 → 标签纯文字居中。返回 `(label_x, (id, size, icon_x))`。
#[must_use]
pub fn layout_icon_label(
    host: &mut dyn SceneHost,
    r: &data::Rect,
    label: &str,
    icon: Option<&IconDef>,
) -> (i32, Option<(u32, i32, i32)>) {
    match icon {
        Some(def) if r.w >= def.size + 8 => {
            let ix = r.x + 8;
            (ix + def.size + 8, Some((def.id, def.size, ix)))
        }
        _ => (r.x + (r.w - host.shell().hint_text_width_px(label)) / 2, None),
    }
}

/// v4 面板语言「选中边框」单一厚度（3px 黄铜边，S20.2；飞轮 R1 ⑥-07 单参源：
/// `select_role`/`saveload` 双调用点引用，勿再各抄一份）。
pub const PANEL_SELECTED_BORDER_PX: i32 = 4;

/// 场景外壳**底色层**（纯色 + 垂直渐变 + 暗角）——可被背景类原语覆盖（底色之上、外框之下）。
/// 飞轮 R1 ⑦-10：零生产调用（场景绘制全走原语表），保留为拆分便利组合壳，doc 标注勿当已完成。
pub fn scene_base(host: &mut dyn SceneHost) {
    parchment_bg(host);
}

/// 场景外壳**外框层**（航海网格 + 木外框）——在背景类原语之上。
/// 飞轮 R1 ⑦-10：同 [`scene_base`] 保留口径（零生产调用，拆分便利组合壳）。
pub fn scene_frame(host: &mut dyn SceneHost) {
    nautical_grid(host);
    wood_frame(host);
}

/// 96px 标题屏心居中的左缘 x。
#[must_use]
pub fn centered_title_x(host: &mut dyn SceneHost, title: &str) -> i32 {
    (LOGICAL_W_I32 - host.shell().title_text_width_px(title)) / 2
}

/// 44px 正文屏心居中的左缘 x。
#[must_use]
pub fn centered_body_x(host: &mut dyn SceneHost, text: &str) -> i32 {
    (LOGICAL_W_I32 - host.shell().text_width_px(text)) / 2
}

/// **场景提示行**。
pub fn scene_hint(host: &mut dyn SceneHost, scene_id: &str, hint_key: &str) {
    let layout = scene_layout(scene_id);
    let text = crate::i18n::sget(hint_key);
    hint_row(host, text, LOGICAL_W_I32 / 2, layout.hint.y);
}

/// 推进翻页动画倒计时（`delta_ms` = 本帧经过毫秒）。
pub fn tick_page_frame(delta_ms: u32) {
    let mut cur = PAGE_MS_LEFT.load(Ordering::Relaxed);
    while let Err(got) = PAGE_MS_LEFT.compare_exchange_weak(
        cur,
        cur.saturating_sub(delta_ms),
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        cur = got;
    }
}

/// 获取当前翻页动画亮度因子（[`MOTION_PAGE_BRIGHTNESS_MIN`] → 1.0 线性（幅值
/// [`MOTION_PAGE_BRIGHTNESS_SWING`]），时长 [`MOTION_PAGE_MS`]；倒计时耗尽 = 1.0 稳态）。
#[must_use]
pub fn page_brightness_factor() -> f32 {
    let left = PAGE_MS_LEFT.load(Ordering::Relaxed);
    if left >= MOTION_PAGE_MS {
        return MOTION_PAGE_BRIGHTNESS_MIN;
    }
    let full = f32::from(u16::try_from(MOTION_PAGE_MS).unwrap_or(0));
    let done = f32::from(u16::try_from(MOTION_PAGE_MS - left).unwrap_or(0)) / full;
    MOTION_PAGE_BRIGHTNESS_MIN + done * MOTION_PAGE_BRIGHTNESS_SWING
}

// === 兼容包装函数（旧签名，内部调用 ti-render） ===
//
// S15.7：绘制类 wrapper 一律只收 `&mut MenuShell`（它已 `impl Canvas`）+ 从
// `shell.theme` 内部取主题。这样做有 3 个好处：
//   1. 渲染框架只见 `Canvas`，不见 `SceneHost`/场景/宿主概念 → 框架可独立复用；
//   2. 调用点写 `host.shell_mut()`，避免「`host` 已可变借用 + `host.theme()`
//      还要不可变借用」的 E0502 冲突；
//   3. 无需 trait-object 上转（不依赖 1.86 trait upcasting）。

/// S15.7：把宿主的 [`RenderHost`] 像素面桥接为渲染框架的 [`ti_render::canvas::Canvas`]。
///
/// `ti_render::shell` / `primitives` 只认 `&mut dyn Canvas`，游戏侧统一持 `&mut dyn
/// SceneHost`。本适配器实现 `Canvas` 并把**像素操作**转发到宿主的 `RenderHost` 同名方法
/// （`fill_rect`/`draw_*`/`clear_frame`/`pixel_at`）——**不经由具体 `MenuShell`**——使
/// 测试宿主（`render_intent::VsHost`）能拦截并记录绘制；宽度/字形高是只读几何，走
/// `shell()` 读。渲染框架因此依旧不感知游戏场景概念，且与具体帧缓冲解耦（`PixelSink`）。
struct HostCanvas<'a> {
    host: &'a mut dyn SceneHost,
}

impl ti_render::canvas::Canvas for HostCanvas<'_> {
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        <dyn SceneHost as RenderHost>::fill_rect(self.host, x, y, w, h, color);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        <dyn SceneHost as RenderHost>::draw_text_colored(self.host, text, x, y, color);
    }

    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        <dyn SceneHost as RenderHost>::draw_title_text(self.host, text, x, y, color);
    }

    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, color: u16) {
        <dyn SceneHost as RenderHost>::draw_hint_text(self.host, text, x, y, color);
    }

    fn text_width(&self, text: &str) -> i32 {
        self.host.shell().text_width_px(text)
    }

    fn title_text_width(&self, text: &str) -> i32 {
        self.host.shell().title_text_width_px(text)
    }

    fn hint_text_width(&self, text: &str) -> i32 {
        self.host.shell().hint_text_width_px(text)
    }

    fn glyph_height(&self) -> i32 {
        self.host.shell().glyph_px()
    }

    fn title_glyph_height(&self) -> i32 {
        self.host.shell().title_glyph_px()
    }

    fn hint_glyph_height(&self) -> i32 {
        self.host.shell().hint_glyph_px()
    }

    fn clear_frame(&mut self, color: u16) {
        <dyn SceneHost as RenderHost>::clear_frame(self.host, color);
    }

    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        <dyn SceneHost as RenderHost>::pixel_at(self.host, x, y)
    }

    /// 贴图（S18.8b `image` 原语）：转发到宿主的绘制目标
    /// （CPU 帧缓冲 = 解码混合；绘制列表 = 记录命令交 Bevy 资产服务器）。
    fn blit_image(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32) {
        <dyn SceneHost as RenderHost>::blit_image(self.host, asset, x, y, w, h);
    }

    /// 贴图 + 色调（S21.14 图标图片化跨层 tint）：转发到宿主的绘制目标。
    fn blit_image_tint(&mut self, asset: &str, x: i32, y: i32, w: i32, h: i32, tint: u16) {
        <dyn SceneHost as RenderHost>::blit_image_tint(self.host, asset, x, y, w, h, tint);
    }
}

/// 在 `host` 上构造本地 [`HostCanvas`] 并交给 `f`（消「返回指向临时值的引用」——
/// 桥接体作为函数局部持有，`&mut` 借用随 `f` 作用域结束，不跨函数返回）。
fn with_canvas(host: &mut dyn SceneHost, f: impl FnOnce(&mut HostCanvas<'_>)) {
    let mut bridge = HostCanvas { host };
    f(&mut bridge);
}

/// 背景底：整帧纯色 + 垂直渐变 + 四边暗角。
pub fn parchment_bg(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::primitives::parchment_bg(c, &theme));
}

/// 整屏外框。
pub fn wood_frame(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::wood_frame(c, &theme));
}

/// 标题条。
pub fn title_bar(host: &mut dyn SceneHost, title_h: i32) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::title_bar(c, &theme, title_h));
}

/// 航海图装饰层。
pub fn nautical_grid(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::nautical_grid(c, &theme));
}

/// S18.7 声明式原语 dispatch（**CPU 执行器**）：`scenes.json` 的 `primitives` 表 → 画布绘制。
///
/// 与 Bevy 执行器（`ti_engine::scene_render`）消费**同一份描述** → 双后端同构
/// （同一 `scenes.json` 出同一画面）。**通用原语只有 6 种**（`rect`/`text`/`icon`/
/// `grad`/`glow`/`disc`），渲染层**不认识**"菜单""木框"等场景语义 —— 场景语义 = 数据。
pub fn render_primitives(host: &mut dyn SceneHost, scene_id: &str, selected: i64) {
    let layout = scene_layout(scene_id);
    let theme = host.theme().clone();
    // 层序：z 升序 = 越大越靠上（调用序即层序，`08_render_arch.md` 契约 1）。
    let mut prims: Vec<&Primitive> = layout.primitives.iter().collect();
    prims.sort_by_key(|p| p.z);
    // S29.3（B1）：已迁 bevy_ui 的 chrome 原语跳过（`params["ui_layer"] == "bevy_ui"`）——
    // Bevy 壳由 `ti-engine` 的 bevy_ui 层画这些 chrome（避免 1080 + bevy_ui 双画）；
    // CPU 路径（无 bevy_ui 的后端）照旧画。
    let skip_migrated = host.chrome_in_bevy_ui();
    for p in prims {
        if !primitive_visible(p, selected) {
            continue;
        }
        if skip_migrated && param_str(p, "ui_layer").as_deref() == Some("bevy_ui") {
            continue;
        }
        let color = theme.by_name(&p.color).unwrap_or(theme.text_on_parch);
        let r = p.rect;
        with_canvas(host, |c| match p.kind.as_str() {
            "rect" => ti_render::generic::rect(c, r.x, r.y, r.w, r.h, color),
            "text" => {
                let content = if p.text_key.is_empty() {
                    p.text.clone()
                } else {
                    crate::i18n::sget(&p.text_key).to_owned()
                };
                // S26.3 底部提示底板：tier=hint 且落屏底区（y≥1560）先画 `menu_bg` 条 + 上下 bevel
                // （与 `hint_row` 完全同源：条宽 = 提示文本宽 + 8，条心 = 屏心 540；
                // 消除「白字直压夜空/海图」低对比通病）。
                // S27.5 D3：排除 `hud.` 前缀文本（main_map 三块铜牌标签 y=1608 也 ≥1560，
                // 误画 menu_bg 深蓝底条 = 舵轮下深蓝矩形根因）。
                if p.tier == "hint" && r.y >= 1560 && !p.text_key.starts_with("scene.main_map.hud.")
                {
                    let tw = c.hint_text_width(&content) + 8;
                    let th = c.hint_glyph_height() + 4;
                    let cx = (LOGICAL_W_I32 - tw) / 2;
                    c.fill_rect(cx, r.y - 2, tw, th, theme.menu_bg);
                    c.fill_rect(cx, r.y - 2, tw, 1, theme.bevel_hi);
                    c.fill_rect(cx, r.y - 2 + th - 1, tw, 1, theme.bevel_shadow);
                }
                ti_render::generic::text(c, &content, r.x, r.y, r.w, r.h, &p.tier, color);
            }
            "icon" => {
                let icon_name = param_str(p, "icon");
                let kind = icon_name
                    .as_deref()
                    .and_then(|name| icon_ids().get(name).copied())
                    .unwrap_or(0);
                // 飞轮 R1 ⑥-02：缺省边长读 `ui_icons.json` 注册表单源（`default_size`）；
                // 注册表整体缺失 = 48 crash-safe 兜底（同 ①-07 族口径，框架侧 `icons::ICON_SIZE` 恒等）。
                let default_size = match ui_icons() {
                    Ok(reg) => i64::from(reg.default_size),
                    Err(_) => 48,
                };
                let size =
                    i32::try_from(param_i64(p, "size").unwrap_or(default_size)).unwrap_or(48);
                // S21.14：注册表 `image` 贴图（跨层 tint=`color`）优先；缺 image/缺资产 = 代码矢量回退
                if let Some(def) = icon_name.as_deref().and_then(icon_def) {
                    if let Some(asset) = def.image.as_deref() {
                        let s = def.size.min(size.max(0));
                        c.blit_image_tint(asset, r.x, r.y, s, s, color);
                        return;
                    }
                }
                ti_render::generic::icon(c, kind, r.x, r.y, size, color);
            }
            "grad" => {
                let from =
                    param_str(p, "from_token").and_then(|n| theme.by_name(&n)).unwrap_or(theme.bg);
                let to =
                    param_str(p, "to_token").and_then(|n| theme.by_name(&n)).unwrap_or(theme.bg);
                let steps = i32::try_from(param_i64(p, "steps").unwrap_or(32)).unwrap_or(32);
                ti_render::generic::grad(c, r.x, r.y, r.w, r.h, from, to, steps);
            }
            "glow" => ti_render::generic::glow(c, r.x, r.y, r.w, color),
            "disc" => ti_render::generic::disc(c, r.x, r.y, r.w, color),
            "image" => {
                // S18.8b：贴图原语（资源路径 = 相对 `assets/` 根；缺资源 = 不画）。
                // S21.14：`p.color`（主题令牌）非空 = 贴图 **tint**（白蒙版 × 色）；空 = 白（无色调，
                // 背景/多色图不受影响）⇒ 白蒙版图标经 `color` 上色，非图标贴图保持原样。
                let asset = param_str(p, "asset").unwrap_or_default();
                if !asset.is_empty() {
                    let tint = if p.color.is_empty() {
                        0xFFFF
                    } else {
                        theme.by_name(&p.color).unwrap_or(0xFFFF)
                    };
                    c.blit_image_tint(&asset, r.x, r.y, r.w, r.h, tint);
                }
            }
            _ => {}
        });
    }
}

/// 原语可见性（`when` = `selected`/`normal`；`item` = 菜单项序号）。
fn primitive_visible(p: &Primitive, selected: i64) -> bool {
    match p.when.as_str() {
        "selected" => p.item == selected,
        "normal" => p.item != selected,
        _ => true,
    }
}

/// 原语参数（字符串）。
fn param_str(p: &Primitive, key: &str) -> Option<String> {
    p.params.get(key).and_then(|v| v.as_str()).map(str::to_owned)
}

/// 原语参数（数值）。
fn param_i64(p: &Primitive, key: &str) -> Option<i64> {
    p.params.get(key).and_then(serde_json::Value::as_i64)
}

/// 菜单 icon（兼容出口：委托 `ti_render::icons`，kind 即图标 id）。
pub fn menu_icon(host: &mut dyn SceneHost, kind: usize, x: i32, y: i32) {
    let theme = host.theme().clone();
    let kind_u32 = u32::try_from(kind).unwrap_or(0);
    with_canvas(host, |c| ti_render::shell::menu_icon(c, &theme, kind_u32, x, y));
}

/// S17.3 图标（注册表驱动）：`id` = `assets/data/ui_icons.json` 的 `icons[].id`，
/// `selected` 决定 tint（默认 `theme.gold` / 选中 `theme.text_on_wood`——
/// 选中底是航海蓝，白墨对比更高）。
pub fn ui_icon(host: &mut dyn SceneHost, id: u32, x: i32, y: i32, selected: bool) {
    let theme = host.theme().clone();
    let tint = if selected { theme.text_on_wood } else { theme.gold };
    with_canvas(host, |c| {
        // S21.14：注册表 `image` 贴图（跨层 tint）优先；缺 image/缺资产 = 代码矢量回退
        if let Some(def) = icon_def_by_id(id) {
            if let Some(asset) = def.image.as_deref() {
                let s = def.size.min(ti_render::icons::ICON_SIZE);
                c.blit_image_tint(asset, x, y, s, s, tint);
                return;
            }
        }
        ti_render::icons::icon(c, &theme, id, x, y, tint);
    });
}

/// 实体图标（`ui_icons.json` 注册表名 + 指定色调；S18.12）。
///
/// 图标始终以 **48px 原生尺寸** 绘制（注册表定位），居中于实体盒（`size`）；
/// 注册表查不到 = 不画图标（不 panic、不占位）。
pub fn entity_icon(host: &mut dyn SceneHost, name: &str, x: i32, y: i32, size: i32, tint: u16) {
    let Some(def) = icon_def(name) else {
        return;
    };
    let s = def.size.min(size.max(0));
    let (ox, oy) = (x + (size - s) / 2, y + (size - s) / 2);
    // S21.14：注册表 `image` 贴图（跨层 tint）优先；缺 image/缺资产 = 代码矢量回退
    if let Some(asset) = def.image.as_deref() {
        with_canvas(host, |c| c.blit_image_tint(asset, ox, oy, s, s, tint));
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::icons::icon(c, &theme, def.id, ox, oy, tint));
}

/// 图标元数据取用（`ui_icons.json` 解析失败 = 不画图标，不 panic）。
#[must_use]
pub fn icon_def(name: &str) -> Option<IconDef> {
    ui_icons().ok().and_then(|t| t.icon(name)).cloned()
}

/// 按 id 取图标元数据（`ui_icon` 等按序号取图标的场景；S21.14 图片化用）。
#[must_use]
pub fn icon_def_by_id(id: u32) -> Option<IconDef> {
    ui_icons().ok().and_then(|t| t.icons.iter().find(|i| i.id == id)).cloned()
}

/// 注册表按 id 取用（主菜单等按序号取图标的场景）。
#[must_use]
pub fn index_menu_icon_def(menu_index: usize) -> Option<IconDef> {
    ui_icons().ok().and_then(|t| t.index_menu_icon(menu_index)).cloned()
}

/// 注册表按 HUD 键取用。
#[must_use]
pub fn hud_icon_def(hud_key: &str) -> Option<IconDef> {
    ui_icons().ok().and_then(|t| t.hud_icon(hud_key)).cloned()
}

/// 内容面板。
pub fn panel(host: &mut dyn SceneHost, x: i32, y: i32, w: i32, h: i32) {
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::panel(c, &theme, x, y, w, h));
}

/// 标题条内文字。
pub fn draw_title_text(host: &mut dyn SceneHost, text: &str, x: i32, y: i32, color: u16) {
    <dyn SceneHost as RenderHost>::draw_title_text(host, text, x, y, color);
}

/// 正文档文字。
pub fn draw_body_text(host: &mut dyn SceneHost, text: &str, x: i32, y: i32) {
    host.draw_text_colored(text, x, y, COLOR_TEXT_ON_PARCH);
}

/// 次级文字。
pub fn draw_body_text_secondary(host: &mut dyn SceneHost, text: &str, x: i32, y: i32) {
    host.draw_text_colored(text, x, y, COLOR_SECONDARY);
}

/// 底部提示行。QB123：条**屏心居中**（忽略传入 `x`，与老 chrome 一致；`x` 仅为兼容
/// 旧调用签名保留）。
pub fn hint_row(host: &mut dyn SceneHost, text: &str, _x: i32, y: i32) {
    let theme = host.theme().clone();
    let cx = LOGICAL_W_I32 / 2;
    with_canvas(host, |c| ti_render::shell::hint_row(c, &theme, text, cx, y));
}

// === v3 场景叙事装饰层包装（UI 重设计 v3.1，docs/ui-redesign-v3/02 §8） ===
//
// 数据驱动（契约 4）：矩形全读 `scenes.json` 的 `extra` 键；键缺失 / 非正 = 不画
// （不 panic、不占位色块，与 icon 列 / 实体 size 同判据）。

/// v3 氛围星空层：读 `scenes.json` 该场景的 `ambient_art` 矩形，缺失/非正 = 不画。
/// 飞轮 R1 ④-05/①-32：星位表 + 屏心净空全读 `ambient_art` 数据（`ambient_stars` +
/// `extra_scalar` 的 `ambient_art.center_x/center_y/exclude`），渲染层只遍历；缺 = 不画。
pub fn star_sky(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(art) = layout.extra.get(keys::AMBIENT_ART) else {
        return;
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let stars: Vec<ti_render::shell::Star> = layout
        .ambient_stars
        .iter()
        .map(|a| ti_render::shell::Star { x: a.x, y: a.y, size: a.size, bright: a.bright })
        .collect();
    if stars.is_empty() {
        return;
    }
    let s = &layout.extra_scalar;
    // crash-safe：键缺失 = 无净空（0），与「0 = 关闭」语义同向（①-07 族口径）。
    let cx = s.get(keys::AMB_CENTER_X).copied().unwrap_or(0);
    let cy = s.get(keys::AMB_CENTER_Y).copied().unwrap_or(0);
    let ex = s.get(keys::AMB_EXCLUDE).copied().unwrap_or(0);
    let theme = host.theme().clone();
    with_canvas(host, |c| {
        ti_render::shell::star_sky(c, &theme, art.x, art.y, art.w, art.h, &stars, cx, cy, ex);
    });
}

/// v3 仪式感标题艺术面板：读 `scenes.json` 该场景的 `title_art` 矩形，缺失/非正 = 不画。
///
/// 面板文字（如巨字标题）由调用方按版面叠画（原语零语义）；z-order 在星空层之后。
pub fn title_art_block(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(art) = layout.extra.get(keys::TITLE_ART) else {
        return;
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::title_art_block(c, &theme, art.x, art.y, art.w, art.h));
}

/// v3 战斗竞技区装饰（vs）：读 `scenes.json` 该场景的 `battle_arena` 矩形，缺失/非正 = 不画。
///
/// z-order（调用序）：`scene_shell` 之后、血条/回合文本/提示行之前（装饰垫底，内容叠上）。
pub fn battle_arena(host: &mut dyn SceneHost, scene_id: &str) {
    let layout = scene_layout(scene_id);
    let Some(art) = layout.extra.get(keys::BATTLE_ARENA) else {
        return;
    };
    if art.w <= 0 || art.h <= 0 {
        return;
    }
    let theme = host.theme().clone();
    with_canvas(host, |c| ti_render::shell::battle_arena(c, &theme, art.x, art.y, art.w, art.h));
}
