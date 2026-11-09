//! VS v4 纯 HUD 拳皇对角线（S34.7）：动态构图渲染（用户 2026-11-03 方案 A 拍板）。
//!
//! 独立文件原因：`vs_anim_proto.rs` 已近文件规模上限（900 行），v4 构图拆出
//! 保持单一职责 + 文件规模在基线内（硬规则 8 模块化 + `audit_code` 棘轮）。
//! 调用方 = `vs_anim_proto::render_v2_elem`（`TI_VS_V4=1` 开关，见该文件）。

use crate::i18n;
use crate::scenes::SceneHost;
use crate::scenes::chrome;
use crate::scenes::vs_anim_proto::{anim_ms, tri};
use ti_core::data::CombatAnimCfg;
use ti_core::data::balance;

/// VS v4 纯 HUD 拳皇对角线（S34.7，用户 2026-11-03 方案 A 拍板 + 保留羊皮）。
///
/// 构图 = 羊皮底（`chart_wide` + 暗角）、敌方面板右上 / 我方面板左上（名字 + 兵力双条 +
/// 人手/武器/武装三数）、中央 VS 徽标、底部指令区（跳过/确认/重打）、角色对角线（我方贴左下
/// 近大 1.15 倍、敌方贴右上远小 1.0 倍，斜地线 + 斜冲虚线交叉于 VS，拳皇 VS 对抗感）。
///
/// `result_win`/`result_lose` 结果帧 = 战场让位给书法横幅 + 战报框（败方倒地）。
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub(crate) fn render_v4_hud(
    host: &mut dyn SceneHost,
    phase: &str,
    atk_crew: u32,
    atk_arms: u32,
    def_crew: u32,
    def_arms: u32,
    resolved: bool,
    class_arg: &str,
) {
    let theme = host.theme().clone();
    let c2 = theme.by_name("brass_dark").unwrap_or(0x3942);
    let c3 = theme.by_name("gold_warm").unwrap_or(0xF692);
    let ink = theme.by_name("ink").unwrap_or(0x4183);
    // S49.4 v5 简洁战斗场（用户 2026-11-08「SB设计」→ 参考宝可梦/FGO 简洁场地）：
    // 满屏羊皮海图插画底 → 甲板战场（deck_battle.png 真实海盗船甲板），角色站甲板不叠插画。
    let v5_simple =
        std::env::var("TI_VS_V5").is_ok_and(|v| !v.trim().is_empty() && v.trim() != "0");
    // S49.4 v6 格子场景·水平对位（用户 2026-11-08 三修正，已确认落码）：格子场景底（vs_bg_asset
    // 按 class 分层）+ 水平对位（同一水平线，非对角）+ 血条重做（单条百分比绿黄红）。
    // v6 为**正式默认**（`TI_VS_V6=0` 可强制关回 v5/v4 调试）。
    let v6_grid = !std::env::var("TI_VS_V6").is_ok_and(|v| v.trim() == "0" || v.trim().is_empty());
    // S49.4 动效参数单源（CS-1：`balance.combat.anim`，0 硬编码魔法数字；缺省 = 代码回退）。
    let anim_cfg = balance().map(|b| b.combat.anim.clone()).unwrap_or_default();
    // S49.4 背景（v6 格子场景满屏 / v5 甲板 / v4 羊皮）+ 四周暗角。
    v4_bg(host, &theme, class_arg, v6_grid, v5_simple);

    // S49.4 时间轴（帧率无关累计；`TI_VS_ANIM_T0` 定格相位）+ 阶段分派 + 战后推算 + 血条插值。
    let t = anim_ms(host);
    let tl = v4_timeline(t, &anim_cfg, resolved, atk_crew, atk_arms, def_crew, def_arms);

    // 敌方面板（右上）+ 我方面板（左上）——对角对称，贴角（v6 单条百分比绿黄红）。
    let pw = 400;
    let ph = 200;
    v4_panel(
        host,
        1080 - pw - 40,
        50,
        pw,
        ph,
        tl.disp_def_crew,
        tl.disp_def_arms,
        true,
        ink,
        v6_grid,
        def_crew,
    );
    v4_panel(
        host,
        40,
        50,
        pw,
        ph,
        tl.disp_atk_crew,
        tl.disp_atk_arms,
        false,
        ink,
        v6_grid,
        atk_crew,
    );

    // 中央 VS 徽标（角色对角交叉点；三层圆 = 黄铜深/黄铜/金心）。
    let cx = 540i32;
    let cy = 960i32;
    v4_vs_badge(host, cx, cy, c2, c3);

    // 角色（v6 水平对位 / v5 缩小 / v4 对角）+ 受击红闪 + 道具动效。
    let is_result = matches!(phase, "result_win" | "result_lose");
    v4_chars(host, phase, t, tl.stage, &anim_cfg, v6_grid, v5_simple);

    if is_result {
        v4_result_scene(
            host, phase, t, v6_grid, cx, c2, atk_crew, atk_arms, def_crew, def_arms, resolved,
        );
        return;
    }
    // 指令区（底部三键：跳过/确认/重打；选中 = 确认金环）
    v4_command_zone(host);
}

/// S49.4 背景层：v7 分元素（统一 `vs_bg` 底板全格类共用 + 格类透明底元素叠加）/
/// v6 格子场景整张（按 class 分层 + tint）/ v5 甲板 / v4 羊皮 + 四周暗角。
fn v4_bg(
    host: &mut dyn SceneHost,
    theme: &ti_render::Theme,
    class_arg: &str,
    v6_grid: bool,
    v5_simple: bool,
) {
    if v6_grid {
        let class = if class_arg.is_empty() { "encounter" } else { class_arg };
        // S49.4 v7 分元素（用户 2026-11-08「背景风格不统一」）：风格统一由底板保证。
        // 底板 = 全格类共用 `vs_bg`（金黄甲板），不再按 class 整张换（G1 风格割裂修复）。
        host.blit_image_tint(
            chrome::V2_VS_BG,
            0,
            0,
            1080,
            1920,
            crate::scenes::vs_anim_proto::vs_tint(None),
        );
        // 格类元素 = 透明底单元素叠加（仅格类差异；无元素 = 纯底板）。独立可换
        // （换 AI 生图元素 = 换资产，零引擎改动；G2 元素烘焙死修复）。
        let elem = crate::scenes::vs_anim_proto::vs_class_elem(class);
        if !elem.is_empty() && chrome::asset_present(&elem) {
            // 元素位：中上视觉区（面板下方、角色后方，foot_y=1000 对位不变）。
            host.blit_image(&elem, 190, 200, 700, 656);
        }
    } else if v5_simple && chrome::asset_present(chrome::VS_DECK_BATTLE) {
        host.blit_image(chrome::VS_DECK_BATTLE, 0, 0, 1080, 1920);
    } else if chrome::asset_present(chrome::VS_CHART_WIDE) {
        host.blit_image(chrome::VS_CHART_WIDE, 0, 0, 1080, 1920);
    } else {
        let sand = theme.by_name("land_sand_light").unwrap_or(chrome::COLOR_GOLD_WARM);
        host.fill_rect(0, 0, 1080, 1920, sand);
    }
    // 四周暗角（三层实色描边近似渐变，越外越重）
    for (edge, col) in [(80i32, 0x2A1A), (48, 0x1E12), (24, 0x140C)] {
        host.fill_rect(0, 0, 1080, edge, col);
        host.fill_rect(0, 1920 - edge, 1080, edge, col);
        host.fill_rect(0, 0, edge, 1920, col);
        host.fill_rect(1080 - edge, 0, edge, 1920, col);
    }
}

/// S49.4 时间轴结果（阶段 + 血条显示值，阶段 2 线性插值到战后）。
struct V4Timeline {
    stage: u32,
    disp_atk_crew: u32,
    disp_atk_arms: u32,
    disp_def_crew: u32,
    disp_def_arms: u32,
}

/// S49.4 时间轴计算：阶段分派（`balance.combat.anim.stage_ms`）+ 战后兵力推算
/// （`balance.manpower.loss_pct`）+ 血条插值（整数千分比，防 f32 精度 clippy 红线）。
fn v4_timeline(
    t: u32,
    anim_cfg: &CombatAnimCfg,
    resolved: bool,
    atk_crew: u32,
    atk_arms: u32,
    def_crew: u32,
    def_arms: u32,
) -> V4Timeline {
    // 0 列阵 / 1 冲锋 / 2 必损 / 3 横幅。
    let (s0, s1, s2, _s3) =
        (anim_cfg.stage_ms[0], anim_cfg.stage_ms[1], anim_cfg.stage_ms[2], anim_cfg.stage_ms[3]);
    let stage = if t < s0 {
        0u32
    } else if t < s0 + s1 {
        1
    } else if t < s0 + s1 + s2 {
        2
    } else {
        3
    };
    // 战后兵力（CS-1：`balance.manpower.loss_pct`；resolved=true = 传入即战后真实值）。
    let mpl = balance().map(|b| &b.manpower).cloned().unwrap_or_default();
    let (atk_after_crew, atk_after_arms, def_after_crew, def_after_arms) = if resolved {
        (atk_crew, atk_arms, def_crew, def_arms)
    } else {
        (
            atk_crew * (10000 - mpl.loss_pct.min(10000)) / 10000,
            atk_arms * (10000 - mpl.weapon_loss_pct.min(10000)) / 10000,
            def_crew * (10000 - mpl.loss_pct.min(10000)) / 10000,
            def_arms * (10000 - mpl.weapon_loss_pct.min(10000)) / 10000,
        )
    };
    // 血条插值（阶段 2 从战前线性插到战后；阶段 0/1 = 战前满值，阶段 3 = 战后）。
    let loss_prog_mille: u64 = match stage {
        0 | 1 => 0,
        2 => u64::from(t.saturating_sub(s0 + s1)) * 1000 / u64::from(s2.max(1)),
        _ => 1000,
    }
    .min(1000);
    let lerp = |before: u32, after: u32| -> u32 {
        let b = i64::from(before);
        let a = i64::from(after);
        let v = b + (a - b) * i64::try_from(loss_prog_mille).unwrap_or(0) / 1000;
        u32::try_from(v.max(0)).unwrap_or(0)
    };
    V4Timeline {
        stage,
        disp_atk_crew: lerp(atk_crew, atk_after_crew),
        disp_atk_arms: lerp(atk_arms, atk_after_arms),
        disp_def_crew: lerp(def_crew, def_after_crew),
        disp_def_arms: lerp(def_arms, def_after_arms),
    }
}

/// 中央 VS 徽标（三层圆 = 黄铜深/黄铜/金心；圆近似 = 四段矩形环，引擎无 `fill_circle`）。
fn v4_vs_badge(host: &mut dyn SceneHost, cx: i32, cy: i32, c2: u16, c3: u16) {
    for (r, col) in [(56i32, chrome::COLOR_BRASS_DARK), (50, chrome::COLOR_BRASS), (44, c3)] {
        let d = r * 2;
        host.fill_rect(cx - r / 2, cy - r / 2, d / 2, d / 2, col);
        host.fill_rect(cx, cy - r / 2, d / 2, d / 2, col);
        host.fill_rect(cx - r / 2, cy, d / 2, d / 2, col);
        host.fill_rect(cx, cy, d / 2, d / 2, col);
    }
    let vs = i18n::sget("scene.vs.anim_versus");
    let tw = host.shell().hint_text_width_px(vs);
    host.draw_hint_text(vs, cx - tw / 2, cy - 12, c2);
}

/// S49.4 角色层：v6 水平对位（同一水平线）/ v5 缩小 / v4 对角，受击红闪 + 道具动效。
fn v4_chars(
    host: &mut dyn SceneHost,
    phase: &str,
    t: u32,
    stage: u32,
    anim_cfg: &CombatAnimCfg,
    v6_grid: bool,
    v5_simple: bool,
) {
    // 敌方（右上）：地线 y≈700，立绘高 640（1.0× 基准）
    let mut enemy_cx = 820i32;
    let mut enemy_gy = 700i32;
    let mut enemy_h = 640i32;
    // 我方（左下）：地线 y≈1240，立绘高 736（1.15× 近大）
    let mut player_cx = 260i32;
    let mut player_gy = 1240i32;
    let mut player_h = 736i32;
    // S49.4 v5 简洁战斗场：角色缩至 ~400（v4 640/736 过大 = 插画感），站甲板不叠满屏插画。
    if v5_simple {
        enemy_h = 400;
        player_h = 460;
    }
    // S49.4 v6 格子场景·水平对位（用户 2026-11-08「不要对角，同一水平」）：
    // 拳皇对角线作废 → 宝可梦式上敌下我同一水平线（foot_y 相同 y≈1000），非对角。
    if v6_grid {
        enemy_cx = 830;
        player_cx = 250;
        enemy_gy = 1000;
        player_gy = 1000;
        enemy_h = 420;
        player_h = 480;
    }
    // 斜地线（对角，非平行）：敌方地线 右→中上，我方地线 中下→左下（v6 水平对位去斜线）
    if !v6_grid {
        v4_diag_line(host, 480, 700, 1020, 640, 0x3942, 4);
        v4_diag_line(host, 60, 1300, 520, 1240, 0x3942, 4);
        // 斜冲路线（虚线，交叉于 VS 徽标）
        v4_dash_line(host, player_cx, player_gy, enemy_cx, enemy_gy, 0xF692, 6);
    }
    // 角色 blit（脚下椭圆投影 + 立绘；result 帧败方倒地）
    let is_result = matches!(phase, "result_win" | "result_lose");
    let won = phase == "result_win";
    // S49.4 受击红闪（阶段 1 冲锋：敌受击；阶段 2 必损：双方受击；`balance.combat.anim.hit`）
    let hit_on = (t % anim_cfg.hit.period_ms.max(1)) < anim_cfg.hit.flash_ms;
    let enemy_hit = hit_on && (stage == 1 || stage == 2);
    let hero_hit = hit_on && stage == 2;
    if !is_result {
        v4_char(host, chrome::V2_ENEMY, enemy_cx, enemy_gy, enemy_h, tri(t, 1200, 8), enemy_hit);
        v4_char(host, chrome::V2_HERO, player_cx, player_gy, player_h, tri(t, 900, 10), hero_hit);
    } else if won {
        // 胜：我站立（bob）+ 敌倒地（左下缩小压地）
        v4_char(host, chrome::V2_HERO, player_cx, player_gy, player_h, tri(t, 1000, 8), false);
        if v6_grid {
            // v6 水平对位：敌在水平线原位倒地（缩至 40% 宽，不遮战报框）
            v4_char_down(host, chrome::V2_ENEMY, enemy_cx - 120, enemy_gy - 60, 240);
        } else {
            v4_char_down(host, chrome::V2_ENEMY, 200, 1500, 320);
        }
    } else {
        // 败：敌站立 + 我倒地（左下缩小压地）
        v4_char(host, chrome::V2_ENEMY, enemy_cx, enemy_gy, enemy_h, tri(t, 1300, 8), false);
        if v6_grid {
            // v6 水平对位：我在水平线原位倒地（缩至 40% 宽，不遮战报框）
            v4_char_down(host, chrome::V2_HERO, player_cx - 120, player_gy - 60, 240);
        } else {
            v4_char_down(host, chrome::V2_HERO, 200, 1500, 320);
        }
    }
    // S49.4 道具动效（阶段 1 冲锋 / 2 必损 叠加；`TI_VS_ITEM` 定格演示某道具，无 = 时间轴窗口轮换）
    if !is_result && (stage == 1 || stage == 2) {
        v6_item_effects(host, t, stage, anim_cfg, player_cx, player_gy, enemy_cx, enemy_gy);
    }
}

/// S49.4 结果帧：书法横幅（v6 顶部 / v4 中上）+ 战报框（底部中，v4 已自绘角色不重复画）。
#[allow(clippy::too_many_arguments)]
fn v4_result_scene(
    host: &mut dyn SceneHost,
    phase: &str,
    t: u32,
    v6_grid: bool,
    cx: i32,
    c2: u16,
    atk_crew: u32,
    atk_arms: u32,
    def_crew: u32,
    def_arms: u32,
    resolved: bool,
) {
    let won = phase == "result_win";
    let banner = if won { chrome::V2_BANNER_WIN } else { chrome::V2_BANNER_LOSE };
    if v6_grid {
        // v6：横幅移顶部中央（敌我面板下方 y≈280），带深色半透明底衬（格子场景上黑书法字不清晰）
        let slot = 0x0842u16;
        host.fill_rect(cx - 240, 260, 480, 300, slot);
        crate::scenes::vs_anim_proto::frame(host, cx - 240, 260, 480, 300, chrome::COLOR_BRASS);
        host.blit_image(banner, cx - 200, 280, 400, 260);
    } else {
        host.blit_image(banner, cx - 250, 760, 500, 300);
    }
    crate::scenes::vs_anim_proto::draw_result_scene(
        host,
        phase,
        t,
        0xDE32,
        chrome::COLOR_BRASS,
        c2,
        atk_crew,
        atk_arms,
        def_crew,
        def_arms,
        resolved,
        false, // S49.4：v4 已自绘对角角色，不重复画 v2 角色
    );
}

/// v4 面板（名字 + 兵力双条 + 人手/武器/武装三数）；`def=true` 敌方（红条）。
/// `v6_style=true`（S49.4 v6 格子场景）：血条重做 = 单条（人手基准）+ 百分比 + 绿/黄/红分段，
/// 百分比基准 = `base_crew`（战前满值，固定；`crew` = 当前显示值，阶段 2 插值）。
#[allow(clippy::too_many_arguments)]
fn v4_panel(
    host: &mut dyn SceneHost,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    crew: u32,
    arms: u32,
    def: bool,
    ink: u16,
    v6_style: bool,
    base_crew: u32,
) {
    let theme = host.theme().clone();
    let c2 = theme.by_name("brass_dark").unwrap_or(0x3942);
    let c3 = theme.by_name("gold_warm").unwrap_or(0xF692);
    let parch = theme.by_name("parch_light").unwrap_or(0x10C9);
    let armed = crew.min(arms);
    let maxv = u32::max(crew.max(arms), 1);
    // 面板底（羊皮 + 黄铜框）
    host.fill_rect(x, y, w, h, parch);
    crate::scenes::vs_anim_proto::frame(host, x, y, w, h, chrome::COLOR_BRASS);
    crate::scenes::vs_anim_proto::frame(host, x + 8, y + 8, w - 16, h - 16, c2);
    // 名字（def = 敌方 海盗旗舰 / 我方 木帆船，i18n 占位用 label）
    let name = if def {
        i18n::sget("scene.vs.v2_elm_defender_label")
    } else {
        i18n::sget("scene.vs.v2_elm_attacker_label")
    };
    let nw = host.shell().hint_text_width_px(name);
    host.draw_hint_text(name, x + (w - nw) / 2, y + 20, ink);
    if v6_style {
        // S49.4 v6 血条重做（宝可梦 HP 条）：单条（人手基准）+ 百分比 + 绿/黄/红分段。
        // 满→绿、<60% 黄、<30% 红；条槽深色 + 黄铜框；百分比基准 = base_crew（战前满值）。
        let bx = x + 24;
        let bw = w - 48;
        let bh = 26;
        let bar_y = y + 66;
        // 人手百分比（base_crew 基准，防 0；整数千分比 0..1000，避免 f32 精度红线）
        let base = base_crew.max(1);
        let pct_mille = u64::from(crew) * 1000 / u64::from(base);
        let pct_mille = pct_mille.min(1000) as u32;
        // 健康色分段：满→绿 0x0F80、<60% 黄 0xE8C0、<30% 红 0xF020（千分比 <300/<600）
        let hp_col = if pct_mille < 300 {
            0xF020u16
        } else if pct_mille < 600 {
            0xE8C0u16
        } else {
            0x0F80u16
        };
        // 条槽（深色底 + 黄铜框）
        let slot = 0x0842u16;
        host.fill_rect(bx, bar_y, bw, bh, slot);
        crate::scenes::vs_anim_proto::frame(host, bx, bar_y, bw, bh, c2);
        let fw = i64::from(bw) * i64::from(pct_mille) / 1000;
        if fw > 0 {
            host.fill_rect(
                bx + 3,
                bar_y + 3,
                i32::try_from(fw).unwrap_or(0).saturating_sub(6).max(0),
                bh - 6,
                hp_col,
            );
        }
        // 百分比数字（条右缘内）
        let pct_txt = format!("{}%", pct_mille / 10);
        let tw = host.shell().hint_text_width_px(&pct_txt);
        host.draw_hint_text(&pct_txt, bx + bw - tw - 8, bar_y + 2, 0xFFFF);
        // 武器副条（细，金；百分比 = arms/base_crew）
        let bar_y2 = bar_y + bh + 10;
        host.fill_rect(bx, bar_y2, bw, 10, slot);
        let arms_mille = (u64::from(arms) * 1000 / u64::from(base)).min(1000);
        let fw2 = i64::from(bw) * i64::try_from(arms_mille).unwrap_or(0) / 1000;
        if fw2 > 0 {
            host.fill_rect(
                bx + 3,
                bar_y2 + 2,
                i32::try_from(fw2).unwrap_or(0).saturating_sub(6).max(0),
                6,
                c3,
            );
        }
        crate::scenes::vs_anim_proto::frame(host, bx, bar_y2, bw, 10, c2);
    } else {
        // 兵力双条（人手 = 主条 / 武器 = 副条；条槽 黄铜 + 填充 敌红我蓝）
        let bx = x + 24;
        let bw = w - 48;
        let bh = 22;
        let bar_y1 = y + 70;
        let bar_y2 = bar_y1 + bh + 12;
        host.fill_rect(bx, bar_y1, bw, bh, c2);
        let fw1 = bw * i32::try_from(crew).unwrap_or(0) / i32::try_from(maxv).unwrap_or(1);
        if fw1 > 0 {
            let col = if def { chrome::COLOR_DANGER } else { chrome::COLOR_PLAYER };
            host.fill_rect(bx + 4, bar_y1 + 4, (fw1 - 8).max(0), bh - 8, col);
        }
        host.fill_rect(bx, bar_y2, bw, bh, c2);
        let fw2 = bw * i32::try_from(arms).unwrap_or(0) / i32::try_from(maxv).unwrap_or(1);
        if fw2 > 0 {
            host.fill_rect(bx + 4, bar_y2 + 4, (fw2 - 8).max(0), bh - 8, c3);
        }
        crate::scenes::vs_anim_proto::frame(host, bx, bar_y1, bw, bh, c2);
        crate::scenes::vs_anim_proto::frame(host, bx, bar_y2, bw, bh, c2);
    }
    // 三数（人手/武器/武装）
    let info = i18n::sgetf("scene.vs.v2_elm_force_fmt", &[&crew, &arms, &armed]);
    let iw = host.shell().hint_text_width_px(&info);
    host.draw_hint_text(&info, x + (w - iw) / 2, y + h - 44, ink);
}

/// v4 角色（脚下椭圆投影 + 立绘 blit，`bob_y` = 逐帧 bob）。
/// `hit=true`（S49.4 受击动效）：红闪 tint（RGB565 红 0xF800）+ 后仰 4px（受击退后）。
fn v4_char(
    host: &mut dyn SceneHost,
    asset: &str,
    cx: i32,
    foot_y: i32,
    hgt: i32,
    bob_y: i32,
    hit: bool,
) {
    let w = hgt * 3 / 4;
    let recoil = if hit { 4 } else { 0 };
    let x = cx - w / 2 + recoil;
    let y = foot_y - hgt + bob_y;
    // 脚下椭圆投影（暗斑，贴地感；ink 色 = theme 令牌）
    host.fill_rect(
        cx - w / 3,
        foot_y - 6,
        (2 * w / 3).max(8),
        12,
        host.theme().by_name("ink").unwrap_or(chrome::COLOR_TEXT_ON_PARCH),
    );
    if hit {
        host.blit_image_tint(asset, x, y, w, hgt, 0xF800);
    } else {
        host.blit_image(asset, x, y, w, hgt);
    }
}

/// v4 倒地角色（左下缩小压地，result 帧败方）。
fn v4_char_down(host: &mut dyn SceneHost, asset: &str, x: i32, y: i32, width: i32) {
    host.fill_rect(
        x,
        y + width / 3,
        width,
        12,
        host.theme().by_name("ink").unwrap_or(chrome::COLOR_TEXT_ON_PARCH),
    );
    host.blit_image(asset, x, y, width, width / 3);
}

/// v4 对角线（实线）：`x0,y0 → x1,y1`（`width` px 线宽）。
fn v4_diag_line(
    host: &mut dyn SceneHost,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: u16,
    width: i32,
) {
    let steps = 40i32;
    for i in 0..steps {
        let fx = x0 + (x1 - x0) * i / steps;
        let fy = y0 + (y1 - y0) * i / steps;
        let ex = x0 + (x1 - x0) * (i + 1) / steps;
        let ey = y0 + (y1 - y0) * (i + 1) / steps;
        host.fill_rect(fx, fy, (ex - fx).max(1).max(width), 1, color);
        host.fill_rect(fx, fy, 1, (ey - fy).max(1).max(width), color);
    }
}

/// v4 虚线（隔段画，暗示冲锋路线；`width` px 线宽）。
fn v4_dash_line(
    host: &mut dyn SceneHost,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: u16,
    width: i32,
) {
    let steps = 60i32;
    for i in (0..steps).step_by(2) {
        let fx = x0 + (x1 - x0) * i / steps;
        let fy = y0 + (y1 - y0) * i / steps;
        let ex = x0 + (x1 - x0) * (i + 1) / steps;
        let ey = y0 + (y1 - y0) * (i + 1) / steps;
        host.fill_rect(fx, fy, (ex - fx).max(1).max(width), 1, color);
        host.fill_rect(fx, fy, 1, (ey - fy).max(1).max(width), color);
    }
}

/// v4 指令区（底部三键：跳过/确认/重打；选中 = 确认金环）。
fn v4_command_zone(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    let c2 = theme.by_name("brass_dark").unwrap_or(0x3942);
    let c3 = theme.by_name("gold_warm").unwrap_or(0xF692);
    let parch = theme.by_name("parch_light").unwrap_or(0x10C9);
    let labels = [
        i18n::sget("scene.vs.v4_cmd_skip"),
        i18n::sget("scene.vs.v4_cmd_confirm"),
        i18n::sget("scene.vs.v4_cmd_retry"),
    ];
    let bw = 280i32;
    let bh = 110i32;
    let gap = 40i32;
    let total = bw * 3 + gap * 2;
    let x0 = (1080 - total) / 2;
    let y = 1920 - 180;
    for (i, label) in labels.iter().enumerate() {
        let bx = x0 + i32::try_from(i).unwrap_or(0) * (bw + gap);
        let sel = i == 1;
        let fill = if sel { 0xF0D8 } else { parch };
        host.fill_rect(bx, y, bw, bh, fill);
        crate::scenes::vs_anim_proto::frame(host, bx, y, bw, bh, if sel { c3 } else { c2 });
        if sel {
            crate::scenes::vs_anim_proto::frame(host, bx + 6, y + 6, bw - 12, bh - 12, c3);
        }
        let tw = host.shell().hint_text_width_px(label);
        host.draw_hint_text(label, bx + (bw - tw) / 2, y + bh / 2 - 14, c2);
    }
}

/// S49.4 道具动效层（阶段 1/2 叠加；`TI_VS_ITEM=keg|medkit|rum|smoke` 定格演示某道具，
/// 无 = 时间轴窗口轮换）。道具 = `balance.combat.anim.items[]`（CS-1 参数单源）。
///
/// 视觉：`keg` = 中央爆裂（橙红扩散环 + 火花）、`medkit` = 我方血条绿光脉冲、
/// `rum` = 攻击方刀光增亮（琥珀 tint）、`smoke` = 全屏灰烟（淡入淡出）。
#[allow(clippy::too_many_arguments)]
fn v6_item_effects(
    host: &mut dyn SceneHost,
    t: u32,
    _stage: u32,
    anim: &CombatAnimCfg,
    player_cx: i32,
    player_gy: i32,
    enemy_cx: i32,
    enemy_gy: i32,
) {
    let forced = std::env::var("TI_VS_ITEM").ok();
    // 4 道具按时间轴窗口轮换（每 800ms 换一个；TI_VS_ITEM 强制定格）
    for (i, id) in ["keg", "medkit", "rum", "smoke"].iter().enumerate() {
        let Some(cfg) = anim.items.iter().find(|c| c.id == *id) else { continue };
        let active = match &forced {
            Some(f) => f == *id,
            None => (t / 800) % 4 == u32::try_from(i).unwrap_or(0),
        };
        if !active {
            continue;
        }
        // S49.4：整数千分比 fade（0..1000 进 1 出 1，避免 f32 精度/截断 clippy 红线）
        let dur = cfg.dur_ms.max(1);
        let phase_mille = (t % (dur * 2)) * 1000 / dur; // 0..2000
        let fade_mille = if phase_mille < 1000 { phase_mille } else { 2000 - phase_mille };
        // CS-1 单源：道具动效主色从 `balance.combat.anim.items[].color` 解析（0 硬编码色字面量）
        let col = hex16(&cfg.color);
        match *id {
            "keg" => v6_item_keg(host, fade_mille, col, player_cx, player_gy, enemy_cx, enemy_gy),
            "medkit" => v6_item_medkit(host, fade_mille, col, player_cx, player_gy),
            "rum" => v6_item_rum(host, fade_mille, col, player_cx, player_gy, enemy_cx, enemy_gy),
            "smoke" => v6_item_smoke(host, fade_mille, col),
            _ => {}
        }
    }
}

/// S49.4：解析 `"0xF020"` 风格 RGB565 色字面量字符串（`balance.combat.anim.items[].color`）；
/// 解析失败 = 金暖色回退（`gold_warm` 令牌同值，见 `chrome::COLOR_GOLD_WARM`）。
fn hex16(hex_str: &str) -> u16 {
    let t = hex_str.trim();
    let hex = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
    u16::from_str_radix(hex, 16).unwrap_or(chrome::COLOR_GOLD_WARM)
}

/// keg 爆裂：中央（双方中点）橙红爆裂（中心白热 + 放射火花，`fade_mille` 0..1000 控制扩散/淡出）。
fn v6_item_keg(
    host: &mut dyn SceneHost,
    fade_mille: u32,
    col: u16,
    px: i32,
    py: i32,
    ex: i32,
    ey: i32,
) {
    let cx = (px + ex) / 2;
    let cy = (py + ey) / 2;
    let fm = i32::try_from(fade_mille).unwrap_or(0);
    let white = 0xFFFFu16;
    // 放射火花（8 方向，fade 越强越散，主色 `col`）
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let r = 30 + fm * 90 / 1000;
        let sx = cx + dx * r;
        let sy = cy + dy * r;
        host.fill_rect(sx - 7, sy - 7, 14, 14, col);
    }
    // 中心白热 + 主色核心
    host.fill_rect(cx - 18, cy - 18, 36, 36, white);
    host.fill_rect(cx - 10, cy - 10, 20, 20, col);
}

/// medkit 绿光：我方角色**头顶上方**绿脉冲（治疗上浮），主色 `col`（不遮角色）。
fn v6_item_medkit(host: &mut dyn SceneHost, fade_mille: u32, col: u16, px: i32, py: i32) {
    let fm = i32::try_from(fade_mille).unwrap_or(0);
    // 头顶上方（foot_y 1000 - 角色高 480 - 40 = 480）
    let h = 26 + fm * 30 / 1000;
    let y = py - 560 - fm * 20 / 1000;
    host.fill_rect(px - 50, y, 100, h, col);
    host.fill_rect(px - 16, y - 18, 32, h + 18, col);
}

/// rum 刀光：双方刀光增亮（主色 `col` 斜线沿冲锋线）。
fn v6_item_rum(
    host: &mut dyn SceneHost,
    fade_mille: u32,
    col: u16,
    px: i32,
    py: i32,
    ex: i32,
    ey: i32,
) {
    // 沿双方连线画刀光（宽度随 fade 增强）
    let fm = i32::try_from(fade_mille).unwrap_or(0);
    let w = 3 + fm * 10 / 1000;
    v4_diag_line(host, px, py - 40, ex, ey - 40, col, w);
}

/// smoke 灰烟：全屏灰雾（主色 `col` 细横条纹分布，`fade_mille` 控制条纹浓度；stagger 使画面朦胧）。
fn v6_item_smoke(host: &mut dyn SceneHost, fade_mille: u32, col: u16) {
    if fade_mille < 50 {
        return;
    }
    // 灰雾 = 细横灰条（间隔 70px，高随 fade 增厚；条间留隙 = 半透明朦胧近似，不遮角色轮廓）
    let fm = i32::try_from(fade_mille).unwrap_or(0);
    let th = 4 + fm * 8 / 1000;
    let mut y = 260;
    let mut i = 0;
    while y < 1700 {
        // 每 5 条跳一条（留隙 = 朦胧非实心）
        if i % 5 != 4 {
            let inset = 40 + (i % 4) * 20;
            host.fill_rect(40 + inset, y, 1000 - inset * 2, th, col);
        }
        y += 70;
        i += 1;
    }
}
