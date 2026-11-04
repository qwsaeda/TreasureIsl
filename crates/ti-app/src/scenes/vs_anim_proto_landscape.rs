//! S38 方案 A：VS 战斗横屏构图（1920×1080）。
//!
//! 用户 2026-11-04 已过目 `vs_landscape` 概念图（`evidence/ui_concept/vs_landscape.png`）并确认。
//! 横屏构图 = 羊皮框（inset 44）+ 顶双 HP 状态框（左攻 / 右守）+ VS 徽标屏心 + 角色对角 +
//! 指令区；result 帧 = 横幅顶部 + 角色对角 + 战报框底部横屏。
//!
//! 竖屏路径 = `vs_anim_proto::render_v2_elem` 原式（0 行为变化，本文件仅横屏分支）。
//!
//! 单一事实源 = `docs/ui-redesign-v3/12_main_map_v6_vs_v2_spec.md` §2（VS v2 构图）。

use crate::i18n;
use crate::scenes::SceneHost;
use crate::scenes::chrome;
use crate::scenes::vs_anim_proto::{
    anim_ms, draw_status_box, frame_v2, tri, vs_bg_asset, vs_theme, vs_tint,
};

/// S38 方案 A：横屏 vs 构图（1920×1080，`vs_landscape` 概念图：羊皮框 + 顶双 HP 状态框 +
/// VS 徽标屏心 + 角色对角 + 指令区）。竖屏路径 = `render_v2_elem` 原式（0 行为变化）。
/// 用户 2026-11-04 已过目 `vs_landscape` 概念图。
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_v2_elem_landscape(
    host: &mut dyn SceneHost,
    phase: &str,
    atk_crew: u32,
    atk_arms: u32,
    def_crew: u32,
    def_arms: u32,
    resolved: bool,
    grid_class: &str,
) {
    let theme = host.theme().clone();
    let c1 = theme.by_name("ink_dark").unwrap_or(0x28C1);
    let c2 = theme.by_name("brass_dark").unwrap_or(0x3942);
    let c3 = theme.by_name("gold_warm").unwrap_or(0xF692);
    let c4 = theme.by_name("parch_dark").unwrap_or(0xDDD1);
    let c5 = theme.by_name("brass").unwrap_or(0x7AC6);
    let c6 = theme.by_name("parch_light").unwrap_or(0xDE32);
    // 背景（格类 → vs_theme 背景资产 + tint；横屏铺满羊皮框内）
    let th = vs_theme();
    let bg = vs_bg_asset(grid_class, th.as_ref());
    host.blit_image_tint(&bg, 44, 44, 1920 - 88, 1080 - 88, vs_tint(th.as_ref()));
    // 羊皮做旧边框（横屏 1920×1080，inset 44）
    for (inset, col) in [(44i32, c6), (30, c5), (18, c5), (8, c4)] {
        let w = 44 - inset + 8;
        host.fill_rect(inset / 2, inset / 2, 1920 - inset, w, col);
        host.fill_rect(inset / 2, 1080 - inset / 2 - w, 1920 - inset, w, col);
        host.fill_rect(inset / 2, inset / 2, w, 1080 - inset, col);
        host.fill_rect(1920 - inset / 2 - w, inset / 2, w, 1080 - inset, col);
    }
    host.fill_rect(0, 0, 1920, 9, c1);
    host.fill_rect(0, 1071, 1920, 9, c1);
    host.fill_rect(0, 0, 9, 1080, c1);
    host.fill_rect(1911, 0, 9, 1080, c1);
    // 顶双状态框（左攻 attacker / 右守 defender）—— 概念图双 HP 条顶；非 result 帧。
    // S38：x=64/976 对齐 8pt 栅格（10_ui_redesign §4），对称屏心 960（64+880=944, 976 距 960 16）。
    if !matches!(phase, "result_win" | "result_lose") {
        draw_status_box(
            host,
            64,
            40,
            880,
            110,
            i18n::sget("scene.vs.v2_elm_attacker_label"),
            atk_crew,
            atk_arms,
            atk_crew.max(1),
            false,
        );
        draw_status_box(
            host,
            976,
            40,
            880,
            110,
            i18n::sget("scene.vs.v2_elm_defender_label"),
            def_crew,
            def_arms,
            def_crew.max(1),
            true,
        );
    }
    // 中央 VS 徽标（屏心 960,540）
    host.fill_rect(960 - 70, 540 - 70, 140, 140, c2);
    frame_v2(host, 960 - 70, 540 - 70, 140, 140, chrome::COLOR_GOLD_WARM);
    let tw = host.shell().hint_text_width_px("VS");
    host.draw_hint_text("VS", 960 - tw / 2, 525, c3);
    let t = anim_ms(host);
    match phase {
        "clash" => {
            // 兵对兵：左中 crew 冲锋 + 左下 hero + 右上 enemy（对角）
            let hb = tri(t, 900, 10);
            let eb = -tri(t, 1100, 8);
            let cx = tri(t, 700, 14);
            host.blit_image(chrome::V2_CREW, 120 + cx, 420, 380, 320);
            host.blit_image(chrome::V2_HERO, 60, 600 + hb, 300, 440);
            host.blit_image(chrome::V2_ENEMY, 1480, 100 + eb, 300, 440);
            for i in 0..6 {
                let sx = 700 + i * 80;
                if (t / 120) % 2 == 0 {
                    host.fill_rect(sx, 500 + (i % 2) * 30, 30, 4, c3);
                }
            }
        }
        "duel" => {
            // 将对将：左下 hero vs 右上 enemy 逼近中央（对角）
            let hx = tri(t, 800, 16);
            let ex = -tri(t, 800, 16);
            host.blit_image(chrome::V2_HERO, 120 + hx, 500, 300, 440);
            host.blit_image(chrome::V2_ENEMY, 1500 + ex, 90, 300, 440);
            let burst = (t / 150) % 3 == 0;
            for i in 0..9 {
                let sx = 820 + (i % 3) * 40;
                let sy = 450 + (i / 3) * 50;
                if burst || i < 3 {
                    host.fill_rect(sx, sy, 12, 12, c3);
                }
            }
            host.fill_rect(780, 480, 360, 5, c3);
        }
        "result_win" | "result_lose" => {
            draw_result_scene_landscape(
                host, phase, t, c4, c5, c2, atk_crew, atk_arms, def_crew, def_arms, resolved,
            );
        }
        _ => {}
    }
}

/// S38 方案 A：横屏 result 帧（胜/败）：横幅顶部 + 角色对角 + 战报框底部横屏。
/// 三段不重叠：横幅顶部（180）+ 角色中（300 起）+ 战报框底部（1080-360）。
#[allow(clippy::too_many_arguments)]
fn draw_result_scene_landscape(
    host: &mut dyn SceneHost,
    phase: &str,
    ms: u32,
    c4: u16,
    c5: u16,
    c2: u16,
    atk_crew: u32,
    atk_arms: u32,
    def_crew: u32,
    def_arms: u32,
    resolved: bool,
) {
    let won = phase == "result_win";
    // 胜方 hero 居中偏下；败方 enemy 居中 + hero 倒地左下（对角）
    if won {
        let dy = tri(ms, 1000, 8);
        host.blit_image(chrome::V2_HERO, 960 - 200, 200 + dy, 400, 560);
    } else {
        let sx = tri(ms, 300, 6);
        host.blit_image(chrome::V2_HERO, 60, 780, 200, 130);
        host.blit_image(chrome::V2_ENEMY, 960 - 200 + sx, 180, 400, 560);
    }
    // 书法横幅（上中，不遮顶部与角色头）
    let banner = if won { chrome::V2_BANNER_WIN } else { chrome::V2_BANNER_LOSE };
    host.blit_image(banner, 960 - 250, 80, 500, 300);
    // 战报框（底部中）
    host.fill_rect(240, 1080 - 360, 1920 - 480, 300, c4);
    frame_v2(host, 240, 1080 - 360, 1920 - 480, 300, c5);
    let title = if won {
        &i18n::sget("scene.vs.v2_elm_battle_win")
    } else {
        &i18n::sget("scene.vs.v2_elm_battle_lose")
    };
    host.draw_hint_text(title, 270, 1080 - 330, c2);
    // 战报兵力（同 draw_result_scene 推算口径）
    let (atk_after_crew, atk_after_arms, def_after_crew, def_after_arms) = if resolved {
        (atk_crew, atk_arms, def_crew, def_arms)
    } else {
        (atk_crew * 85 / 100, atk_arms * 80 / 100, def_crew * 85 / 100, def_arms * 80 / 100)
    };
    let atk_after_armed = atk_after_crew.min(atk_after_arms);
    let atk_after_unarmed = atk_after_crew - atk_after_armed;
    let def_after_armed = def_after_crew.min(def_after_arms);
    let def_after_unarmed = def_after_crew - def_after_armed;
    let atk_arm_loss = atk_arms - atk_after_arms;
    let def_arm_loss = def_arms - def_after_arms;
    let atk_crew_loss = atk_after_crew - atk_crew;
    let def_crew_loss = def_after_crew - def_crew;
    let args: [&dyn std::fmt::Display; 20] = [
        &atk_crew,
        &atk_arms,
        &atk_crew.min(atk_arms),
        &(atk_crew - atk_crew.min(atk_arms)),
        &atk_after_crew,
        &atk_after_arms,
        &atk_after_armed,
        &atk_after_unarmed,
        &atk_crew_loss,
        &atk_arm_loss,
        &def_crew,
        &def_arms,
        &def_crew.min(def_arms),
        &(def_crew - def_crew.min(def_arms)),
        &def_after_crew,
        &def_after_arms,
        &def_after_armed,
        &def_after_unarmed,
        &def_crew_loss,
        &def_arm_loss,
    ];
    let body = if won {
        i18n::sgetf("scene.vs.v2_elm_battle_report_win", &args)
    } else {
        i18n::sgetf("scene.vs.v2_elm_battle_report_lose", &args)
    };
    for (i, line) in body.lines().enumerate() {
        let lw = host.shell().hint_text_width_px(line);
        let iy = i32::try_from(i).unwrap_or(0);
        host.draw_hint_text(line, 960 - lw / 2, 1080 - 300 + iy * 44, c2);
    }
}
