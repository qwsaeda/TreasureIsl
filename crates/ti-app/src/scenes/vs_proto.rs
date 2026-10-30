//! S27.7 VS 对战界面改版：5 套候选版式的**原型渲染器**（选型期）。
//!
//! 背景：用户 2026-10-29 要求「对战界面应该有道具特效、现代化回合制的效果，多出几种 VS 界面」，
//! 并要求**引擎真实渲染图**做选型（AI 概念稿不算数）。本模块 = 选型期的 5 套版式原型：
//! `TI_VS_PROTO=a|b|c|d|e` 切换（壳层设环境变量 → `--shot --scene=vs` 出图）。
//!
//! 口径：
//! - 1080×1920 逻辑坐标（与 `08_layout_design.md` §4.4 同一坐标系）；
//! - 素材 = `assets/ui/` 已登记资产（`vs_v1` 批次：`boss_captain` / `hero_back` /
//!   `deck_battle` / `item_*`；既有：`bg_battle_sea` / `bg_night_sea` / `role_*` / `helm` /
//!   `card_panel` / `deck_plaque` / `icon_*`）；
//! - 数值取 `battle_snapshot()`（无快照时用演示值，便于出图对比）。
//!
//! **选定版式后**：本模块按选定版式重写为正式实现（`docs/ui-redesign-v3/11_vs_spec.md`），
//! 未选中的版式删除；道具/动作业务规则 = `docs/business/combat.md` §战斗动作（已定案）。

use std::sync::OnceLock;

use crate::i18n;
use crate::scenes::chrome;
use crate::scenes::chrome::{
    COLOR_BRASS, COLOR_BRASS_DARK, COLOR_BRASS_LIGHT, COLOR_DANGER, COLOR_GOLD, COLOR_GOLD_BRIGHT,
    COLOR_PLAYER, COLOR_TEXT_ON_PARCH, COLOR_WOOD,
};
use crate::scenes::{RenderHost, SceneHost};

/// 环境变量名（选型开关：`a`/`b`/`c`/`d`/`e`）。
pub const PROTO_ENV: &str = "TI_VS_PROTO";

/// 演示用玩家 HP（无战斗快照时出图用）。
const DEMO_PLAYER_HP: u32 = 1_560;
/// 演示用玩家 HP 上限。
const DEMO_PLAYER_HP_MAX: u32 = 2_000;
/// 演示用 BOSS HP。
const DEMO_BOSS_HP: u32 = 1_840;
/// 演示用 BOSS HP 上限。
const DEMO_BOSS_HP_MAX: u32 = 2_400;
/// 演示用回合数。
const DEMO_ROUND: u32 = 7;
/// 演示用伤害数字。
const DEMO_DAMAGE: u32 = 215;

/// 道具槽位（S27.7 批次 `vs_v1` 的 4 件道具 + 演示数量；资产路径单源 = `chrome::VS_*` 常量，
/// 登记 = `assets/ui_manifest.json` 批次 `vs_v1`，缺失图走缺图回退）。
const ITEMS: [(&str, u32); 4] = [
    (chrome::VS_ITEM_KEG, 2),
    (chrome::VS_ITEM_MEDKIT, 1),
    (chrome::VS_ITEM_RUM, 3),
    (chrome::VS_ITEM_SMOKE, 1),
];

/// 选定版式（读环境变量，进程内缓存；未设置 → `None` = 走正式版式）。
#[must_use]
pub fn selected_variant() -> Option<&'static str> {
    static VARIANT: OnceLock<Option<String>> = OnceLock::new();
    VARIANT
        .get_or_init(|| {
            std::env::var(PROTO_ENV)
                .ok()
                .map(|v| v.trim().to_lowercase())
                .filter(|v| matches!(v.as_str(), "a" | "b" | "c" | "d" | "e"))
        })
        .as_deref()
}

/// 出图用战斗数值（有快照取快照，无快照取演示值）。
struct ProtoState {
    /// 玩家当前 HP。
    player_hp: u32,
    /// 玩家 HP 上限。
    player_hp_max: u32,
    /// BOSS 当前 HP。
    boss_hp: u32,
    /// BOSS HP 上限。
    boss_hp_max: u32,
    /// 已战回合数。
    round: u32,
    /// 演示伤害数字。
    damage: u32,
}

impl ProtoState {
    /// 从战斗快照取值（`None` → 演示值）。
    fn from_host(host: &mut dyn SceneHost) -> Self {
        let Some(s) = host.battle_snapshot() else {
            return Self {
                player_hp: DEMO_PLAYER_HP,
                player_hp_max: DEMO_PLAYER_HP_MAX,
                boss_hp: DEMO_BOSS_HP,
                boss_hp_max: DEMO_BOSS_HP_MAX,
                round: DEMO_ROUND,
                damage: DEMO_DAMAGE,
            };
        };
        Self {
            player_hp: s.player_hp,
            player_hp_max: s.player_hp_max.max(1),
            boss_hp: s.boss_hp,
            boss_hp_max: s.boss_hp_max.max(1),
            round: s.rounds,
            damage: DEMO_DAMAGE,
        }
    }

    /// 玩家血条百分比（0..=100）。
    fn player_pct(&self) -> u32 {
        self.player_hp.min(self.player_hp_max) * 100 / self.player_hp_max
    }

    /// BOSS 血条百分比（0..=100）。
    fn boss_pct(&self) -> u32 {
        self.boss_hp.min(self.boss_hp_max) * 100 / self.boss_hp_max
    }
}

/// 主题取色（令牌名 → RGB565；缺失 = 退回 `fallback`，不 panic）。
fn theme_color(host: &dyn SceneHost, name: &str, fallback: u16) -> u16 {
    host.theme().by_name(name).unwrap_or(fallback)
}

/// 圆角面板底（`card_panel` 白蒙版 tint；渲染只走 `RenderHost`，资产路径走本文件单源常量）。
fn plate(host: &mut dyn RenderHost, rx: i32, ry: i32, rw: i32, rh: i32, tint: u16) {
    host.blit_image_tint(chrome::VS_CARD_PANEL, rx, ry, rw, rh, tint);
}

/// 四边描边（黄铜框；`RenderHost` 窄接缝）。
fn frame(host: &mut dyn RenderHost, rx: i32, ry: i32, rw: i32, rh: i32, color: u16, thick: i32) {
    host.fill_rect(rx, ry, rw, thick, color);
    host.fill_rect(rx, ry + rh - thick, rw, thick, color);
    host.fill_rect(rx, ry, thick, rh, color);
    host.fill_rect(rx + rw - thick, ry, thick, rh, color);
}

/// 血条（槽底 + 填充 + 刻度 + 数值 + 上沿标签；文本走 `SceneHost` 文案口，渲染只走 `RenderHost`）。
#[allow(clippy::too_many_arguments)]
fn hp_bar(
    host: &mut dyn SceneHost,
    rx: i32,
    ry: i32,
    rw: i32,
    rh: i32,
    pct: u32,
    fill: u16,
    label: &str,
    value: &str,
    right_align: bool,
) {
    host.fill_rect(rx, ry, rw, rh, COLOR_BRASS_DARK);
    let inner_w = (rw - 8).max(0);
    let inner_h = (rh - 8).max(0);
    let pct_i = i32::try_from(pct.min(100)).unwrap_or(0);
    host.fill_rect(rx + 4, ry + 4, inner_w * pct_i / 100, inner_h, fill);
    // 刻度（4 段）
    for i in 1..4 {
        host.fill_rect(rx + rw * i / 4, ry + 4, 2, inner_h, COLOR_BRASS_DARK);
    }
    frame(host, rx, ry, rw, rh, COLOR_BRASS, 3);
    // 数值（右对齐贴右缘）
    let vw = host.shell().hint_text_width_px(value);
    host.draw_hint_text(
        value,
        rx + rw - 8 - vw,
        ry + (rh - host.shell().hint_glyph_px()) / 2,
        COLOR_TEXT_ON_PARCH,
    );
    // 标签（上沿）
    let lw = host.shell().hint_text_width_px(label);
    let lx = if right_align { rx + rw - lw } else { rx };
    host.draw_hint_text(label, lx, ry - 34, COLOR_TEXT_ON_PARCH);
}

/// 道具槽（面板 + 图标 + 数量角标 + 选中金环；`RenderHost` 渲染 + `SceneHost` 文本口）。
#[allow(clippy::too_many_arguments)]
fn item_slot(
    host: &mut dyn SceneHost,
    rx: i32,
    ry: i32,
    size: i32,
    icon: &str,
    count: u32,
    selected: bool,
) {
    plate(host, rx, ry, size, size, COLOR_BRASS_DARK);
    let pad = size / 6;
    host.blit_image(icon, rx + pad, ry + pad, size - pad * 2, size - pad * 2);
    // 数量角标
    let badge = size / 3;
    host.fill_rect(rx + size - badge - 6, ry + size - badge - 6, badge, badge, COLOR_BRASS);
    let num = count.to_string();
    let nw = host.shell().hint_text_width_px(&num);
    host.draw_hint_text(
        &num,
        rx + size - badge - 6 + (badge - nw) / 2,
        ry + size - badge - 6 + (badge - host.shell().hint_glyph_px()) / 2,
        chrome::COLOR_BLACK,
    );
    if selected {
        frame(host, rx - 4, ry - 4, size + 8, size + 8, COLOR_GOLD_BRIGHT, 4);
    }
}

/// 动作按钮（面板 + 图标 + 文字；`primary` = 强调色；`RenderHost` 渲染 + `SceneHost` 文本口）。
#[allow(clippy::too_many_arguments)]
fn action_button(
    host: &mut dyn SceneHost,
    rx: i32,
    ry: i32,
    rw: i32,
    rh: i32,
    icon: &str,
    label: &str,
    primary: bool,
) {
    plate(host, rx, ry, rw, rh, if primary { COLOR_BRASS } else { COLOR_BRASS_DARK });
    frame(host, rx, ry, rw, rh, if primary { COLOR_GOLD_BRIGHT } else { COLOR_BRASS }, 3);
    let icon_size = (rh / 2).min(rw / 2);
    host.blit_image_tint(
        icon,
        rx + (rw - icon_size) / 2,
        ry + 10,
        icon_size,
        icon_size,
        if primary { COLOR_GOLD_BRIGHT } else { COLOR_BRASS_LIGHT },
    );
    let lw = host.shell().hint_text_width_px(label);
    host.draw_hint_text(
        label,
        rx + (rw - lw) / 2,
        ry + rh - host.shell().hint_glyph_px() - 12,
        if primary { chrome::COLOR_BLACK } else { COLOR_TEXT_ON_PARCH },
    );
}

/// 大号伤害数字（标题档 + 暗色描边阴影；暖金 = 打击感，不用冷调 `gold_bright`）。
fn damage_number(host: &mut dyn SceneHost, cx: i32, y: i32, value: u32) {
    let text = value.to_string();
    let w = host.shell().title_text_width_px(&text);
    let x = cx - w / 2;
    let warm = theme_color(host, "gold_warm", COLOR_GOLD);
    host.draw_title_text(&text, x + 4, y + 4, chrome::COLOR_BLACK);
    host.draw_title_text(&text, x, y, warm);
}

/// 回合徽章（舵轮 + 回合数）。
fn turn_badge(host: &mut dyn SceneHost, cx: i32, cy: i32, size: i32, round: u32) {
    host.blit_image(chrome::VS_HELM, cx - size / 2, cy - size / 2, size, size);
    let text = round.to_string();
    let w = host.shell().hint_text_width_px(&text);
    host.draw_hint_text(&text, cx - w / 2, cy + size / 2 + 6, COLOR_TEXT_ON_PARCH);
}

/// 毫度角 → 正弦/余弦（放大 10000 倍的整数近似；避免浮点误差进坐标）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn sin_cos(mdeg: i32) -> (i32, i32) {
    let rad = f64::from(mdeg) * std::f64::consts::PI / 180_000.0;
    ((rad.sin() * 10_000.0) as i32, (rad.cos() * 10_000.0) as i32)
}

/// 圆形血环（`pct` 决定亮起的段数；细长小片沿圆周切向排布 = 弧形刻度）。
fn hp_ring(host: &mut dyn SceneHost, cx: i32, cy: i32, radius: i32, pct: u32, color: u16) {
    let segments = 24i32;
    let seg_w = 26i32;
    let seg_h = 12i32;
    let pct_i = i32::try_from(pct.min(100)).unwrap_or(0);
    let lit = (segments * pct_i / 100).max(1);
    for i in 0..segments {
        let angle_mdeg = i * 15_000;
        let (sin, cos) = sin_cos(angle_mdeg);
        let sx = cx + radius * cos / 10_000 - seg_w / 2;
        let sy = cy + radius * sin / 10_000 - seg_h / 2;
        let tint = if i < lit { color } else { COLOR_BRASS_DARK };
        host.blit_image_rot(chrome::VS_CARD_PANEL, sx, sy, seg_w, seg_h, tint, angle_mdeg);
    }
    // 环心图标
    host.blit_image_tint(chrome::VS_ICON_ANCHOR, cx - 30, cy - 30, 60, 60, COLOR_BRASS_LIGHT);
}

/// 竖向血槽（自上而下填充 + 刻度；`RenderHost` 窄接缝）。
fn vertical_gauge(
    host: &mut dyn RenderHost,
    rx: i32,
    ry: i32,
    rw: i32,
    rh: i32,
    pct: u32,
    fill: u16,
) {
    host.fill_rect(rx, ry, rw, rh, COLOR_BRASS_DARK);
    let inner_h = (rh - 8).max(0);
    let pct_i = i32::try_from(pct.min(100)).unwrap_or(0);
    host.fill_rect(rx + 4, ry + 4, rw - 8, inner_h * pct_i / 100, fill);
    for i in 1..8 {
        host.fill_rect(rx, ry + rh * i / 8, rw, 3, COLOR_BRASS_DARK);
    }
    frame(host, rx, ry, rw, rh, COLOR_BRASS, 3);
}

/// 渲染入口（`vs.rs` 在 `selected_variant()` = `Some` 时调用）。
pub fn render(host: &mut dyn SceneHost, variant: &str) {
    let st = ProtoState::from_host(host);
    match variant {
        "a" => variant_a(host, &st),
        "b" => variant_b(host, &st),
        "c" => variant_c(host, &st),
        "d" => variant_d(host, &st),
        _ => variant_e(host, &st),
    }
}

/// 版式 A：甲板对峙 + 环形血条 + 底部横向道具腰带。
///
/// 结构：夜间甲板背景；左下玩家背身立绘 / 右上 BOSS 立绘；顶部两侧圆形血环；
/// 底部 4 格黄铜道具腰带（首格选中金环）；中央浮动伤害数字。
fn variant_a(host: &mut dyn SceneHost, st: &ProtoState) {
    host.blit_image(chrome::VS_DECK_BATTLE, 0, 0, 1080, 1920);
    // 角色
    host.blit_image(chrome::VS_HERO_BACK, 60, 700, 380, 570);
    host.blit_image(chrome::VS_BOSS_CAPTAIN, 620, 360, 420, 630);
    // 顶部血环
    hp_ring(host, 150, 210, 88, st.player_pct(), COLOR_PLAYER);
    hp_ring(host, 930, 210, 88, st.boss_pct(), COLOR_DANGER);
    // 中央伤害数字
    damage_number(host, 540, 1080, st.damage);
    // 底部道具腰带（底板先画，槽位后画 ⇒ 槽在板上）
    let slot = 150i32;
    let gap = 28i32;
    let total = slot * 4 + gap * 3;
    let x0 = (1080 - total) / 2;
    let panel = theme_color(host, "panel_wood", COLOR_WOOD);
    plate(host, x0 - 40, 1520, total + 80, 240, panel);
    frame(host, x0 - 40, 1520, total + 80, 240, COLOR_BRASS, 4);
    for (i, (icon, count)) in ITEMS.iter().enumerate() {
        let idx = i32::try_from(i).unwrap_or(0);
        item_slot(host, x0 + idx * (slot + gap), 1560, slot, icon, *count, i == 0);
    }
    // 腰带标题
    let label = i18n::sget("combat.action.item");
    let lw = host.shell().hint_text_width_px(label);
    host.draw_hint_text(label, 540 - lw / 2, 1490, COLOR_TEXT_ON_PARCH);
    // 回合徽章
    turn_badge(host, 540, 1400, 120, st.round);
}

/// 版式 B：海岸 BOSS + 底部扇形手牌 + 能量点。
fn variant_b(host: &mut dyn SceneHost, st: &ProtoState) {
    host.blit_image(chrome::VS_BG_NIGHT_SEA, 0, 0, 1080, 1920);
    // BOSS 立绘（居中偏上）+ 玩家小图
    host.blit_image(chrome::VS_BOSS_CAPTAIN, 330, 240, 420, 630);
    host.blit_image(chrome::VS_HERO_BACK, 60, 940, 280, 420);
    // 顶部细血条（左右分置）
    hp_bar(
        host,
        60,
        120,
        420,
        42,
        st.player_pct(),
        COLOR_PLAYER,
        i18n::sget("combat.side.player"),
        "1560/2000",
        false,
    );
    hp_bar(host, 600, 120, 420, 42, st.boss_pct(), COLOR_DANGER, "BOSS", "1840/2400", true);
    // 伤害数字
    damage_number(host, 540, 900, st.damage);
    // 能量点（5 枚铜珠，前 3 枚点亮）
    for i in 0..5i32 {
        let x = 540 - 5 * 44 / 2 + i * 44;
        let lit = i < 3;
        host.fill_rect(x, 1400, 36, 36, if lit { COLOR_GOLD_BRIGHT } else { COLOR_BRASS_DARK });
        frame(host, x, 1400, 36, 36, COLOR_BRASS, 2);
    }
    // 扇形手牌（4 张道具牌，绕底部中心排布；卡面与图标同角度旋转）
    let card_w = 190i32;
    let card_h = 270i32;
    let panel = theme_color(host, "panel_wood", COLOR_WOOD);
    for (i, (icon, count)) in ITEMS.iter().enumerate() {
        let idx = i32::try_from(i).unwrap_or(0) - 2;
        let rot = idx * 10_000;
        let card_cx = 540 + idx * 160;
        let card_center_y = 1620 - idx.abs() * 14;
        let x = card_cx - card_w / 2;
        let y = card_center_y - card_h / 2;
        host.blit_image_rot(chrome::VS_CARD_PANEL, x, y, card_w, card_h, panel, rot);
        // 图标随卡旋转：图标中心 = 卡心 + 旋转后的局部偏移（上方 30px）
        let (sin, cos) = sin_cos(rot);
        let icon_off_x = 0i32;
        let icon_off_y = -60i32;
        let off_x = (icon_off_x * cos - icon_off_y * sin) / 10_000;
        let off_y = (icon_off_x * sin + icon_off_y * cos) / 10_000;
        host.blit_image_rot(
            icon,
            card_cx + off_x - 44,
            card_center_y + off_y - 44,
            88,
            88,
            0xFFFF,
            rot,
        );
        // 数量角标（卡片右下）
        let num = count.to_string();
        let nw = host.shell().hint_text_width_px(&num);
        host.draw_hint_text(
            &num,
            card_cx + card_w / 2 - 24 - nw,
            card_center_y + card_h / 2 - 44,
            COLOR_TEXT_ON_PARCH,
        );
    }
}

/// 版式 C：海战侧舷对轰 + 两侧竖血槽 + 右下道具转盘。
fn variant_c(host: &mut dyn SceneHost, st: &ProtoState) {
    host.blit_image(chrome::VS_BG_BATTLE_SEA, 0, 0, 1080, 1920);
    // 两侧竖血槽（细长 + 铜框，不抢画面）
    vertical_gauge(host, 72, 320, 26, 1020, st.player_pct(), COLOR_PLAYER);
    vertical_gauge(host, 982, 320, 26, 1020, st.boss_pct(), COLOR_DANGER);
    // 阵营标识
    host.blit_image(chrome::VS_SHIP_PLAYER, 40, 150, 160, 160);
    host.blit_image(chrome::VS_BOSS_CAPTAIN, 880, 120, 180, 270);
    // 伤害数字（中央）
    damage_number(host, 540, 980, st.damage);
    // 右下道具转盘（舵轮 + 4 枚道具，全在屏内）
    host.blit_image(chrome::VS_HELM, 760, 1420, 260, 260);
    for (i, (icon, count)) in ITEMS.iter().enumerate() {
        let angle_mdeg = i32::try_from(i).unwrap_or(0) * 90_000 + 45_000;
        let (sin, cos) = sin_cos(angle_mdeg);
        let cx = 890 + 118 * cos / 10_000;
        let cy = 1550 + 118 * sin / 10_000;
        item_slot(host, cx - 48, cy - 48, 96, icon, *count, i == 0);
    }
}

/// 版式 D：甲板 3/4 透视决斗 + 顶部左右血槽 + 底部弧形道具按钮。
fn variant_d(host: &mut dyn SceneHost, st: &ProtoState) {
    host.blit_image(chrome::VS_DECK_BATTLE, 0, 0, 1080, 1920);
    // 角色（左下背身 / 右上船长）
    host.blit_image(chrome::VS_HERO_BACK, 40, 760, 420, 630);
    host.blit_image(chrome::VS_BOSS_CAPTAIN, 640, 250, 420, 630);
    // 顶部血槽（左我方 / 右 BOSS，左右分置不叠压）
    hp_bar(
        host,
        60,
        110,
        420,
        46,
        st.player_pct(),
        COLOR_PLAYER,
        i18n::sget("combat.side.player"),
        "1560/2000",
        false,
    );
    hp_bar(host, 600, 110, 420, 46, st.boss_pct(), COLOR_DANGER, "BOSS", "1840/2400", true);
    // 伤害数字
    damage_number(host, 540, 1080, st.damage);
    // 底部弧形道具按钮（沿底部弧线排布，弧心在屏幕下方）
    let center_x = 540i32;
    let center_y = 2180i32;
    let radius = 460i32;
    for (i, (icon, count)) in ITEMS.iter().enumerate() {
        let angle_mdeg = 180_000 + i32::try_from(i).unwrap_or(0) * 22_000;
        let (sin, cos) = sin_cos(angle_mdeg.rem_euclid(360_000));
        let cx = center_x + radius * cos / 10_000;
        let cy = center_y + radius * sin / 10_000;
        item_slot(host, cx - 70, cy - 70, 140, icon, *count, i == 0);
    }
}

/// 版式 E：对称回合制 + 顶部分段血条 + 中央舵轮回合徽章 + 底部黄铜指挥台。
fn variant_e(host: &mut dyn SceneHost, st: &ProtoState) {
    host.blit_image(chrome::VS_BG_NIGHT_SEA, 0, 0, 1080, 1920);
    // 对峙双方（左我方半身像 / 右 BOSS）；立绘垫铜牌底 ⇒ 不留硬边矩形
    let panel = theme_color(host, "panel_wood", COLOR_WOOD);
    plate(host, 28, 288, 464, 684, panel);
    frame(host, 28, 288, 464, 684, COLOR_BRASS, 4);
    host.blit_image(chrome::VS_ROLE_KIDD, 40, 300, 440, 660);
    plate(host, 588, 268, 464, 684, panel);
    frame(host, 588, 268, 464, 684, COLOR_BRASS, 4);
    host.blit_image(chrome::VS_BOSS_CAPTAIN, 600, 280, 440, 660);
    // 中缝
    host.fill_rect(539, 240, 2, 900, COLOR_BRASS_DARK);
    // 顶部分段血条
    hp_bar(
        host,
        40,
        120,
        460,
        48,
        st.player_pct(),
        COLOR_PLAYER,
        i18n::sget("combat.side.player"),
        "1560/2000",
        false,
    );
    hp_bar(host, 580, 120, 460, 48, st.boss_pct(), COLOR_DANGER, "BOSS", "1840/2400", true);
    // 中央回合徽章
    turn_badge(host, 540, 200, 140, st.round);
    // 伤害数字（中缝）
    damage_number(host, 540, 1080, st.damage);
    // 底部指挥台
    host.blit_image(chrome::VS_DECK_PLAQUE, 40, 1560, 1000, 300);
    // 动作按钮（图标用美术资产，非几何白蒙版）
    action_button(
        host,
        120,
        1620,
        260,
        170,
        chrome::VS_ICON_CUTLASS,
        i18n::sget("combat.action.attack"),
        true,
    );
    action_button(
        host,
        410,
        1620,
        260,
        170,
        chrome::VS_ITEM_KEG,
        i18n::sget("combat.action.item"),
        false,
    );
    action_button(
        host,
        700,
        1620,
        260,
        170,
        chrome::VS_ICON_SHIELD,
        i18n::sget("combat.action.defend"),
        false,
    );
    // 道具条（指挥台上方）
    let slot = 110i32;
    let gap = 24i32;
    let total = slot * 4 + gap * 3;
    let x0 = (1080 - total) / 2;
    for (i, (icon, count)) in ITEMS.iter().enumerate() {
        let idx = i32::try_from(i).unwrap_or(0);
        item_slot(host, x0 + idx * (slot + gap), 1400, slot, icon, *count, i == 0);
    }
}
