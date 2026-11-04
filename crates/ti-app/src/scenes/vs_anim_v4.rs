//! VS v4 纯 HUD 拳皇对角线（S34.7）：动态构图渲染（用户 2026-11-03 方案 A 拍板）。
//!
//! 独立文件原因：`vs_anim_proto.rs` 已近文件规模上限（900 行），v4 构图拆出
//! 保持单一职责 + 文件规模在基线内（硬规则 8 模块化 + `audit_code` 棘轮）。
//! 调用方 = `vs_anim_proto::render_v2_elem`（`TI_VS_V4=1` 开关，见该文件）。

use crate::i18n;
use crate::scenes::SceneHost;
use crate::scenes::chrome;
use crate::scenes::vs_anim_proto::{anim_ms, tri};

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
    _class: &str,
) {
    let theme = host.theme().clone();
    let c2 = theme.by_name("brass_dark").unwrap_or(0x3942);
    let c3 = theme.by_name("gold_warm").unwrap_or(0xF692);
    let ink = theme.by_name("ink").unwrap_or(0x4183);
    // 羊皮底（保留用户拍板羊皮纹理；暗角压四周，战场中央留亮）
    if chrome::asset_present(chrome::VS_CHART_WIDE) {
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

    // 敌方面板（右上）+ 我方面板（左上）——对角对称，贴角
    let pw = 400;
    let ph = 200;
    v4_panel(host, 1080 - pw - 40, 50, pw, ph, def_crew, def_arms, true, ink);
    v4_panel(host, 40, 50, pw, ph, atk_crew, atk_arms, false, ink);

    // 中央 VS 徽标（角色对角交叉点；三层圆 = 黄铜深/黄铜/金心）
    let cx = 540i32;
    let cy = 960i32;
    for (r, col) in [(56i32, chrome::COLOR_BRASS_DARK), (50, chrome::COLOR_BRASS), (44, c3)] {
        let d = r * 2;
        // 圆近似：四段矩形环（引擎无 fill_circle，用矩形环描边近似；内圆 = 填色矩形）
        host.fill_rect(cx - r / 2, cy - r / 2, d / 2, d / 2, col);
        host.fill_rect(cx, cy - r / 2, d / 2, d / 2, col);
        host.fill_rect(cx - r / 2, cy, d / 2, d / 2, col);
        host.fill_rect(cx, cy, d / 2, d / 2, col);
    }
    let vs = i18n::sget("scene.vs.anim_versus");
    let tw = host.shell().hint_text_width_px(vs);
    host.draw_hint_text(vs, cx - tw / 2, cy - 12, c2);

    // 角色对角线（拳皇式）：我方贴左下（近大 1.15×）、敌方贴右上（1.0×）；
    // 斜地线 + 斜冲虚线交叉于 VS 徽标（暗示冲锋路线）。
    let is_result = matches!(phase, "result_win" | "result_lose");
    let t = anim_ms(host);
    // 敌方（右上）：地线 y≈700，立绘高 640（1.0× 基准）
    let enemy_cx = 820i32;
    let enemy_gy = 700i32;
    let enemy_h = 640i32;
    // 我方（左下）：地线 y≈1240，立绘高 736（1.15× 近大）
    let player_cx = 260i32;
    let player_gy = 1240i32;
    let player_h = 736i32;
    // 斜地线（对角，非平行）：敌方地线 右→中上，我方地线 中下→左下
    v4_diag_line(host, 480, 700, 1020, 640, 0x3942, 4);
    v4_diag_line(host, 60, 1300, 520, 1240, 0x3942, 4);
    // 斜冲路线（虚线，交叉于 VS 徽标）
    v4_dash_line(host, player_cx, player_gy, enemy_cx, enemy_gy, 0xF692, 6);
    // 角色 blit（脚下椭圆投影 + 立绘；result 帧败方倒地）
    let won = phase == "result_win";
    if !is_result {
        v4_char(host, chrome::V2_ENEMY, enemy_cx, enemy_gy, enemy_h, tri(t, 1200, 8));
        v4_char(host, chrome::V2_HERO, player_cx, player_gy, player_h, tri(t, 900, 10));
    } else if won {
        // 胜：我站立（bob）+ 敌倒地（左下缩小压地）
        v4_char(host, chrome::V2_HERO, player_cx, player_gy, player_h, tri(t, 1000, 8));
        v4_char_down(host, chrome::V2_ENEMY, 200, 1500, 320);
    } else {
        // 败：敌站立 + 我倒地（左下缩小压地）
        v4_char(host, chrome::V2_ENEMY, enemy_cx, enemy_gy, enemy_h, tri(t, 1300, 8));
        v4_char_down(host, chrome::V2_HERO, 200, 1500, 320);
    }

    if is_result {
        // 结果帧：书法横幅（中上偏下，不遮角色头）+ 战报框（底部中）
        let banner = if won { chrome::V2_BANNER_WIN } else { chrome::V2_BANNER_LOSE };
        host.blit_image(banner, cx - 250, 760, 500, 300);
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
        );
        return;
    }
    // 指令区（底部三键：跳过/确认/重打；选中 = 确认金环）
    v4_command_zone(host);
}

/// v4 面板（名字 + 兵力双条 + 人手/武器/武装三数）；`def=true` 敌方（红条）。
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
    // 三数（人手/武器/武装）
    let info = i18n::sgetf("scene.vs.v2_elm_force_fmt", &[&crew, &arms, &armed]);
    let iw = host.shell().hint_text_width_px(&info);
    host.draw_hint_text(&info, x + (w - iw) / 2, y + h - 44, ink);
}

/// v4 角色（脚下椭圆投影 + 立绘 blit，`bob_y` = 逐帧 bob）。
fn v4_char(host: &mut dyn SceneHost, asset: &str, cx: i32, foot_y: i32, hgt: i32, bob_y: i32) {
    let w = hgt * 3 / 4;
    let x = cx - w / 2;
    let y = foot_y - hgt + bob_y;
    // 脚下椭圆投影（暗斑，贴地感；ink 色 = theme 令牌）
    host.fill_rect(
        cx - w / 3,
        foot_y - 6,
        (2 * w / 3).max(8),
        12,
        host.theme().by_name("ink").unwrap_or(chrome::COLOR_TEXT_ON_PARCH),
    );
    host.blit_image(asset, x, y, w, hgt);
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
