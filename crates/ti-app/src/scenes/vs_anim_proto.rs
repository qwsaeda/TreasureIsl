//! VS 战斗动画 v2 概念原型（`TI_VS_ANIM_PROTO=clash|duel|result_win|result_lose`）。
//!
//! 背景：用户 2026-10-30 拍板 VS 全新设计（v2 兵力模型）——
//! 「各带水手对战的动效场景，开打时**兵对兵**（从结果可看出谁人多/人少、有没有兵器）
//! → **将对将** → **玩家赢/败画面 + 嘲讽语**」。
//!
//! **风格裁定（2026-10-30 用户二轮）**：方向 OK，但首版原型「风格不搭（写实海战/星空
//! 背景 vs 游戏羊皮/黄铜手绘）」→ **重设计为游戏统一风**：羊皮纸手绘底 + 黄铜框架 +
//! 手绘水手方阵（弃 `bg_battle_sea`/`bg_night_sea` 写实照片背景）。
//!
//! 本模块 = **设计稿先行**（A 纪律）：`TI_VS_ANIM_PROTO` 环境变量 → 出阶段帧概念图
//! （引擎真实渲染），供用户迭看确认。**确认后**才落终码（vs.rs 正式渲染 +
//! 阶段状态机 `SiegeAnimPhase`）。
//!
//! 口径：
//! - 1080×1920 逻辑坐标（同 `08_layout_design.md` §4.4）；
//! - 素材 = `assets/ui/` 已登记资产（`boss_captain` / `hero_back` / `ship_player`；缺图 = 回退，
//!   不 panic）；背景/框架 = **代码绘制羊皮手绘**（不依赖写实照片资产）；
//! - 兵力 = **演示常量**（攻 30 人 18 武器 vs 守 20 人 12 武器），原型不依赖运行态。
//!
//! 业务规则（单一事实源） = `docs/business/combat.md` §兵力模型（v2）+ `territory.md` §4。

use crate::i18n;
use crate::scenes::SceneHost;
use crate::scenes::chrome;
use crate::scenes::chrome::{COLOR_BRASS, COLOR_DANGER, COLOR_PLAYER, COLOR_TEXT_ON_PARCH};
use ti_core::data::map_pack::{FsPackReader, PackReader, VsTheme, check_pack_manifest};

/// 格类战斗类型环境变量（S34.5：`TI_VS_GRID_CLASS=attack|raid|boss|encounter`）。
pub const GRID_CLASS_ENV: &str = "TI_VS_GRID_CLASS";

/// 环境变量名（阶段帧开关：`clash`/`duel`/`result_win`/`result_lose`）。
pub const ANIM_ENV: &str = "TI_VS_ANIM_PROTO";

/// 元素合成版开关（`TI_VS_V2_ELEM=1` ⇒ 走 `assets/ui/vs_v2/` AI 元素 blit，2026-10-31 重设计）。
pub const V2_ELEM_ENV: &str = "TI_VS_V2_ELEM";

// 演示兵力（攻方舰队 vs 守方驻军+防御；武装 = min(人手, 武器)，combat.md §兵力模型）。
const DEMO_ATK_CREW: u32 = 30;
const DEMO_ATK_ARMS: u32 = 18;
const DEMO_DEF_CREW: u32 = 20;
const DEMO_DEF_ARMS: u32 = 12;
// 演示战损（双方必损：15% 人手 + 20% 武器，combat.md §必损）。
const DEMO_ATK_LOSS: u32 = 5;
const DEMO_DEF_LOSS: u32 = 4;

/// 选定阶段帧（读环境变量；未设置 → `None` = 走正式版式）。
#[must_use]
pub fn selected_phase() -> Option<&'static str> {
    let v = std::env::var(ANIM_ENV).ok()?;
    match v.trim().to_lowercase().as_str() {
        "clash" => Some("clash"),
        "duel" => Some("duel"),
        "result_win" => Some("result_win"),
        "result_lose" => Some("result_lose"),
        _ => None,
    }
}

/// 当前格类战斗类型（S34.5 按格类分层；`TI_VS_GRID_CLASS` 环境变量，默认 `encounter`）。
#[must_use]
pub fn selected_grid_class() -> &'static str {
    match std::env::var(GRID_CLASS_ENV).unwrap_or_default().trim().to_lowercase().as_str() {
        "attack" => "attack",
        "raid" => "raid",
        "boss" => "boss",
        _ => "encounter",
    }
}

/// 当前图 `vs_theme`（S34.5 按地图换主题；`TI_MAP` → 对应 manifest；无 = `None` 回退默认池）。
#[must_use]
pub(crate) fn vs_theme() -> Option<VsTheme> {
    let map_id = std::env::var("TI_MAP")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "m01".to_owned());
    let reader = FsPackReader::new("assets/maps");
    let raw = reader.read_file(&map_id, "manifest.json")?;
    check_pack_manifest(&raw, &map_id).ok()?.vs_theme().cloned()
}

/// 格类 → 背景资产路径（`vs_theme` 优先，缺 = 默认池资产；`CS-1` 资产路径单一读点 = chrome 常量）。
pub(crate) fn vs_bg_asset(class: &str, th: Option<&VsTheme>) -> String {
    let default = match class {
        "attack" => chrome::V2_BG_SIEGE,
        "raid" => chrome::V2_BG_DUNGEON,
        "boss" => chrome::V2_BG_BOSS,
        _ => chrome::V2_VS_BG,
    };
    let Some(th) = th else {
        return default.to_owned();
    };
    let id = match class {
        "attack" => &th.siege,
        "raid" => &th.dungeon,
        "boss" => &th.boss,
        _ => &th.encounter,
    };
    // vs_theme 资产 id（数据面）→ 池路径（前缀 = chrome 目录常量，单一读点）
    format!("{}{id}.png", chrome::V2_VS_BG_DIR)
}

/// 格类 → 主题色调（`vs_theme.tint`；无 = `0xFFFF` 白恒等）。
pub(crate) fn vs_tint(th: Option<&VsTheme>) -> u16 {
    th.map_or(0xFFFF, VsTheme::tint_u16)
}

/// 逐帧时间轴时钟（proto 路径专用）：累计 `frame_delta_ms`（帧率无关，S17.4 唯一时间源）。
static ANIM_ACC: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// `TI_VS_ANIM_T0`：初始相位（ms；proto 离屏出多相位帧证明逐帧用，默认 0）。
static ANIM_T0: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

fn anim_ms(host: &dyn SceneHost) -> u32 {
    use std::sync::atomic::Ordering;
    let t0 = *ANIM_T0.get_or_init(|| {
        std::env::var("TI_VS_ANIM_T0").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0)
    });
    let _ = ANIM_ACC.fetch_add(host.frame_delta_ms(), Ordering::Relaxed);
    ANIM_ACC.load(Ordering::Relaxed).saturating_add(t0)
}

/// 三角波（`0..amp`，`period_ms` 周期）：proto 逐帧 `bob` / 冲锋脉动。
fn tri(ms: u32, period_ms: u32, amp: i32) -> i32 {
    let p = i64::from(period_ms.max(1));
    let phase = i64::from(ms % period_ms.max(1));
    let half = (p / 2).max(1);
    let x = if phase < half { phase } else { p - phase };
    let out = x.saturating_mul(i64::from(amp)) / half;
    i32::try_from(out).unwrap_or(0)
}

/// 渲染入口（`vs.rs` 在 `selected_phase()` = `Some` 时调用）。
pub fn render(host: &mut dyn SceneHost, phase: &str) {
    // 2026-10-31 用户否决旧稿 → 元素合成重设计版：`TI_VS_V2_ELEM=1` ⇒ AI 元素 blit
    // （敌我分列 + 顶/底兵力条 + 结果横幅，与 `scripts/compose_vs_v2.py` 同构图）。
    if std::env::var(V2_ELEM_ENV).is_ok() {
        render_v2_elem(host, phase);
        return;
    }
    match phase {
        "clash" => draw_clash(host),
        "duel" => draw_duel(host),
        "result_win" => draw_result(host, true),
        _ => draw_result(host, false),
    }
}

/// 元素合成重设计版（`TI_VS_V2_ELEM=1`，2026-10-31 用户否决旧稿 → 重设计）。
///
/// 直接 blit `assets/ui/vs_v2` AI 元素（敌我分列 + 顶/底兵力条 + 结果横幅），
/// 与 `scripts/compose_vs_v2.py` 同构图。阶段：`clash`/`duel`/`result_win`/`result_lose`。
#[allow(clippy::too_many_lines)]
fn render_v2_elem(host: &mut dyn SceneHost, phase: &str) {
    let theme = host.theme().clone();
    let ink = theme.by_name("parch_dark").unwrap_or(chrome::COLOR_TEXT_ON_PARCH);
    // 本地渲染色常量（v2 原型专用，不入 theme；走 chrome COLOR_* 有对应令牌时优先令牌）
    let c1 = theme.by_name("ink_dark").unwrap_or(0x28C1);
    let c2 = theme.by_name("brass_dark").unwrap_or(0x3942);
    let c3 = theme.by_name("gold_warm").unwrap_or(0xF692);
    // 背景（S34.5 按格类×地图主题：格类 → vs_theme 背景资产 + 主题 tint；收进 44px 羊皮框）
    let class = selected_grid_class();
    let th = vs_theme();
    let bg = vs_bg_asset(class, th.as_ref());
    host.blit_image_tint(&bg, 44, 44, 1080 - 88, 1920 - 88, vs_tint(th.as_ref()));
    // 羊皮做旧边框
    let c4 = theme.by_name("parch_dark").unwrap_or(0xDDD1);
    let c5 = theme.by_name("brass").unwrap_or(0x7AC6);
    let c6 = theme.by_name("parch_light").unwrap_or(0xDE32);
    for (inset, col) in [(44i32, c6), (30, c5), (18, c5), (8, c4)] {
        let w = 44 - inset + 8;
        host.fill_rect(inset / 2, inset / 2, 1080 - inset, w, col);
        host.fill_rect(inset / 2, 1920 - inset / 2 - w, 1080 - inset, w, col);
        host.fill_rect(inset / 2, inset / 2, w, 1920 - inset, col);
        host.fill_rect(1080 - inset / 2 - w, inset / 2, w, 1920 - inset, col);
    }
    host.fill_rect(0, 0, 1080, 9, c1);
    host.fill_rect(0, 1911, 1080, 9, c1);
    host.fill_rect(0, 0, 9, 1920, c1);
    host.fill_rect(1071, 0, 9, 1920, c1);

    // U20：兵力值 phase 相关（clash/duel 战前，result 战后）+ 满兵力基准 max_crew 对齐兵力条。
    let (atk_crew, atk_arms, def_crew, def_arms) = match phase {
        "result_win" | "result_lose" => (
            DEMO_ATK_CREW - DEMO_ATK_LOSS,
            DEMO_ATK_ARMS * 8 / 10,
            DEMO_DEF_CREW - DEMO_DEF_LOSS,
            DEMO_DEF_ARMS * 8 / 10,
        ),
        _ => (DEMO_ATK_CREW, DEMO_ATK_ARMS, DEMO_DEF_CREW, DEMO_DEF_ARMS),
    };
    // U20：clash/duel 必带 HUD（spec §2.5 顶守/底攻）；result 战报框已含双侧兵力数据 ⇒
    // 隐藏状态框避免与战报框重叠（U20「战报重叠」修复）。
    if !matches!(phase, "result_win" | "result_lose") {
        // 顶部守方状态框（右）+ 底部攻方状态框（左）——黄铜羊皮面板
        draw_status_box(
            host,
            1080 - 470,
            60,
            440,
            130,
            i18n::sget("scene.vs.v2_elm_defender_label"),
            def_crew,
            def_arms,
            DEMO_DEF_CREW,
            true,
        );
        draw_status_box(
            host,
            30,
            1920 - 190,
            440,
            130,
            i18n::sget("scene.vs.v2_elm_attacker_label"),
            atk_crew,
            atk_arms,
            DEMO_ATK_CREW,
            false,
        );
    }

    // 中央 VS 徽标
    host.fill_rect(540 - 70, 890 - 70, 140, 140, c2);
    frame_v2(host, 540 - 70, 890 - 70, 140, 140, chrome::COLOR_GOLD_WARM);
    let tw = host.shell().hint_text_width_px("VS");
    host.draw_hint_text("VS", 540 - tw / 2, 875, c3);

    // #3 逐帧时间轴（proto 路径）：英雄/敌军 bob、冲锋脉动、逼近/晃动（帧率无关）。
    let t = anim_ms(host);

    match phase {
        "clash" => {
            // 兵对兵：左下攻方 crew 冲锋 + hero 领军，右上守方 enemy 督战
            // 2026-10-31 用户「整体重做背景/角色」→ 角色放大（hero/enemy 突出）
            // #3 逐帧：hero/enemy bob + crew 冲锋脉动 + 刀光闪烁。
            let hb = tri(t, 900, 10);
            let eb = -tri(t, 1100, 8);
            let cx = tri(t, 700, 14);
            host.blit_image(chrome::V2_CREW, 430 - 190 + cx, 1130 - 160, 380, 320);
            host.blit_image(chrome::V2_HERO, 215 - 130, 1500 - 190 + hb, 260, 380);
            host.blit_image(chrome::V2_ENEMY, 845 - 130, 400 - 190 + eb, 260, 380);
            // 冲锋刀光（中央，随 t 整体闪烁）
            for i in 0..6 {
                let sx = 300 + i * 80;
                if (t / 120) % 2 == 0 {
                    host.fill_rect(sx, 890 + (i % 2) * 30, 30, 4, c3);
                }
            }
        }
        "duel" => {
            // 将对将：左下 hero vs 右上 enemy 刀光冲突（放大 + 突出上半身）
            // #3 逐帧：双方逼近中央 + 火花脉动。
            let hx = tri(t, 800, 16);
            let ex = -tri(t, 800, 16);
            host.blit_image(chrome::V2_HERO, 240 - 130 + hx, 1360 - 200, 260, 400);
            host.blit_image(chrome::V2_ENEMY, 855 - 130 + ex, 460 - 200, 260, 400);
            // 刀光火花（中央，脉动）
            let burst = (t / 150) % 3 == 0;
            for i in 0..9 {
                let sx = 480 + (i % 3) * 40;
                let sy = 850 + (i / 3) * 50;
                if burst || i < 3 {
                    host.fill_rect(sx, sy, 12, 12, c3);
                }
            }
            host.fill_rect(440, 880, 200, 5, c3);
        }
        "result_win" | "result_lose" => {
            // U20 拆分（render_v2_elem 超函数基线 140 行）：result 帧独立子函数。
            draw_result_scene(host, phase, t, c4, c5, c2);
        }
        _ => {}
    }
    let _ = ink;
}

/// result 帧（胜/败）：角色 + 书法横幅 + 战报框（6 行双侧 16 参数，U20 拆分）。
/// 三段不重叠：横幅顶部（250）+ 角色中下（600 起）+ 战报框底部（1920-500）。
fn draw_result_scene(host: &mut dyn SceneHost, phase: &str, ms: u32, c4: u16, c5: u16, c2: u16) {
    let won = phase == "result_win";
    // 胜方 hero（win）居中偏下；败方（lose）enemy 居中 + hero 倒下左下。
    if won {
        let dy = tri(ms, 1000, 8);
        host.blit_image(chrome::V2_HERO, 540 - 200, 600 + dy, 400, 560);
    } else {
        let sx = tri(ms, 300, 6);
        // A2 修复（飞轮审计）：lose 倒地 hero 原 (60,1360,340,220) 与战报框 (240,1420,600,420)
        // 重叠 160×160；移至左下角框外 (20,1330,200,130)，x 右缘 220 < 框左 240，不重叠。
        host.blit_image(chrome::V2_HERO, 20, 1330, 200, 130);
        host.blit_image(chrome::V2_ENEMY, 540 - 200 + sx, 580, 400, 560);
    }
    // 书法横幅（上中偏上，不遮顶部框与角色头）
    let banner = if won { chrome::V2_BANNER_WIN } else { chrome::V2_BANNER_LOSE };
    host.blit_image(banner, 540 - 250, 250, 500, 300);
    // 战报框（底部中，加高容纳 6 行双侧战报）
    host.fill_rect(240, 1920 - 500, 1080 - 480, 420, c4);
    frame_v2(host, 240, 1920 - 500, 1080 - 480, 420, c5);
    let title = if won {
        &i18n::sget("scene.vs.v2_elm_battle_win")
    } else {
        &i18n::sget("scene.vs.v2_elm_battle_lose")
    };
    host.draw_hint_text(title, 270, 1920 - 470, c2);
    // U20：战报双侧 16 参数（攻守战前→战后 人手/武器/武装/徒手），拆武装/徒手。
    let atk_after_crew = DEMO_ATK_CREW - DEMO_ATK_LOSS;
    let atk_after_arms = DEMO_ATK_ARMS * 8 / 10;
    let atk_before_armed = DEMO_ATK_CREW.min(DEMO_ATK_ARMS);
    let atk_before_unarmed = DEMO_ATK_CREW - atk_before_armed;
    let atk_after_armed = atk_after_crew.min(atk_after_arms);
    let atk_after_unarmed = atk_after_crew - atk_after_armed;
    let def_after_crew = DEMO_DEF_CREW - DEMO_DEF_LOSS;
    let def_after_arms = DEMO_DEF_ARMS * 8 / 10;
    let def_before_armed = DEMO_DEF_CREW.min(DEMO_DEF_ARMS);
    let def_before_unarmed = DEMO_DEF_CREW - def_before_armed;
    let def_after_armed = def_after_crew.min(def_after_arms);
    let def_after_unarmed = def_after_crew - def_after_armed;
    // A7 修复（飞轮审计）：战报第 5 行显示实际损失（我方/敌方 人/器），与数值一致。
    let atk_arm_loss = DEMO_ATK_ARMS - atk_after_arms;
    let def_arm_loss = DEMO_DEF_ARMS - def_after_arms;
    let args: [&dyn std::fmt::Display; 20] = [
        &DEMO_ATK_CREW,
        &DEMO_ATK_ARMS,
        &atk_before_armed,
        &atk_before_unarmed,
        &atk_after_crew,
        &atk_after_arms,
        &atk_after_armed,
        &atk_after_unarmed,
        &DEMO_DEF_CREW,
        &DEMO_DEF_ARMS,
        &def_before_armed,
        &def_before_unarmed,
        &def_after_crew,
        &def_after_arms,
        &def_after_armed,
        &def_after_unarmed,
        &DEMO_ATK_LOSS,
        &atk_arm_loss,
        &DEMO_DEF_LOSS,
        &def_arm_loss,
    ];
    let body = if won {
        i18n::sgetf("scene.vs.v2_elm_battle_report_win", &args)
    } else {
        i18n::sgetf("scene.vs.v2_elm_battle_report_lose", &args)
    };
    // draw_hint_text 单行，分六行画（每侧战前/战后 + 必损 + 嘲讽）
    for (i, line) in body.lines().enumerate() {
        let lw = host.shell().hint_text_width_px(line);
        let iy = i32::try_from(i).unwrap_or(0);
        host.draw_hint_text(line, 540 - lw / 2, 1920 - 440 + iy * 50, c2);
    }
}

/// v2 元素版状态框（顶守/底攻）：名牌 + 人手/武器/武装 + 兵力条。
#[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
fn draw_status_box(
    host: &mut dyn SceneHost,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    name: &str,
    crew: u32,
    arms: u32,
    max_crew: u32,
    def: bool,
) {
    let theme = host.theme().clone();
    let c6 = theme.by_name("parch_light").unwrap_or(0xDE32);
    let c5 = theme.by_name("brass").unwrap_or(0x7AC6);
    let c2 = theme.by_name("brass_dark").unwrap_or(0x3942);
    let c7 = theme.by_name("ink").unwrap_or(0x4183);
    let c8 = theme.by_name("sea_deep").unwrap_or(0x28E2);
    host.fill_rect(x, y, w, h, c6);
    frame_v2(host, x, y, w, h, c5);
    host.draw_hint_text(name, x + 18, y + 12, c2);
    let armed = crew.min(arms);
    let info = i18n::sgetf("scene.vs.v2_elm_force_fmt", &[&crew, &arms, &armed]);
    host.draw_hint_text(&info, x + 18, y + 38, c7);
    // 兵力条（黄铜槽 + 绿守/红攻填充；U20 = 满兵力基准 max_crew，替代硬编码 40）
    let (bx, by, bw, bh) = (x + 18, y + h - 40, w - 36, 22);
    host.fill_rect(bx, by, bw, bh, c8);
    let frac = if max_crew == 0 {
        0.0
    } else {
        // u32→f32 直接 `as` 触发 clippy cast_precision_loss；经 u16 无损转换（兵力 ≤ u16::MAX 常态）。
        let c = u16::try_from(crew).unwrap_or(u16::MAX);
        let m = u16::try_from(max_crew).unwrap_or(1);
        f32::from(c) / f32::from(m)
    };
    // bw ≤ 1080 像素有界，`as` 截断安全（`i32`→`u16` 去符号安全，`f32`→`i32` 截断安全）
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::useless_conversion)]
    let fw = f32::from(u16::try_from((bw - 8).max(0) as u16).unwrap_or(0)) * frac;
    #[allow(clippy::cast_possible_truncation)]
    let fw = fw as i32;
    let col = if def { chrome::COLOR_PLAYER } else { chrome::COLOR_DANGER };
    if fw > 0 {
        host.fill_rect(bx + 4, by + 4, fw, bh - 8, col);
    }
    frame_v2(host, bx, by, bw, bh, chrome::COLOR_BRASS_DARK);
}

/// v2 元素版黄铜框（描边）。
fn frame_v2(host: &mut dyn SceneHost, x: i32, y: i32, width: i32, height: i32, color: u16) {
    host.fill_rect(x, y, width, 3, color);
    host.fill_rect(x, y + height - 3, width, 3, color);
    host.fill_rect(x, y, 3, height, color);
    host.fill_rect(x + width - 3, y, 3, height, color);
}

/// 羊皮手绘底（VS 动画统一风）：**已有泛黄羊皮海图**（`chart_wide`，用户裁定 = 用已有
/// 泛黄羊皮资源，非代码 flat/网格）+ 四周黄铜框 + 四角铆钉。
fn parchment_vs_bg(host: &mut dyn SceneHost) {
    let theme = host.theme().clone();
    let ink = theme.by_name("parch_dark").unwrap_or(chrome::COLOR_TEXT_ON_PARCH);
    // 泛黄羊皮海图拉伸全屏（1024×439 → 1080×1920；羊皮纹理均匀，变形可接受）
    if chrome::asset_present(chrome::VS_CHART_WIDE) {
        host.blit_image(chrome::VS_CHART_WIDE, 0, 0, 1080, 1920);
    } else {
        let sand = theme.by_name("land_sand_light").unwrap_or(chrome::COLOR_GOLD_WARM);
        host.fill_rect(0, 0, 1080, 1920, sand);
    }
    // 四周黄铜框（8px）+ 内衬墨线
    frame(host, 8, 8, 1064, 1904, COLOR_BRASS);
    frame(host, 22, 22, 1036, 1876, ink);
    // 四角铆钉
    for (cx, cy) in [(44, 44), (1036, 44), (44, 1876), (1036, 1876)] {
        host.fill_rect(cx - 10, cy - 10, 20, 20, COLOR_BRASS);
        host.fill_rect(cx - 4, cy - 4, 8, 8, chrome::COLOR_GOLD_WARM);
    }
}

/// 中央纯羊皮竞技面板（用户选型 C：海图底 + 羊皮面板——背景保留海图装饰，前景干净）。
fn arena_panel(host: &mut dyn SceneHost, x: i32, y: i32, width: i32, height: i32) {
    let theme = host.theme().clone();
    let sand = theme.by_name("land_sand_light").unwrap_or(chrome::COLOR_GOLD_WARM);
    let stain = theme.by_name("land_sand_dark").unwrap_or(chrome::COLOR_BRASS_DARK);
    host.fill_rect(x, y, width, height, sand);
    // 边缘做旧（内阴影渐晕）
    for edge in [8i32, 18, 30] {
        host.fill_rect(x, y, width, edge, stain);
        host.fill_rect(x, y + height - edge, width, edge, stain);
        host.fill_rect(x, y, edge, height, stain);
        host.fill_rect(x + width - edge, y, edge, height, stain);
    }
    // 黄铜框
    frame(host, x, y, width, height, COLOR_BRASS);
    frame(host, x + 10, y + 10, width - 20, height - 20, chrome::COLOR_BRASS_DARK);
}

/// 顶部铜牌标题槽（黄铜匾 + 墨线内框）。
fn title_plaque(host: &mut dyn SceneHost, title: &str) {
    let x = 190;
    let y = 60;
    let w = 700;
    let h = 96;
    host.fill_rect(x - 14, y - 14, w + 28, h + 28, chrome::COLOR_BRASS_DARK);
    host.fill_rect(x, y, w, h, chrome::COLOR_BRASS);
    frame(host, x, y, w, h, chrome::COLOR_GOLD_WARM);
    let tw = host.shell().hint_text_width_px(title);
    host.draw_hint_text(
        title,
        540 - tw / 2,
        y + (h - host.shell().hint_glyph_px()) / 2,
        chrome::COLOR_TEXT_ON_PARCH,
    );
}

/// 底部阶段提示铜牌。
fn bottom_plaque(host: &mut dyn SceneHost, note: &str) {
    let x = 190;
    let y = 1760;
    let w = 700;
    let h = 96;
    host.fill_rect(x - 14, y - 14, w + 28, h + 28, chrome::COLOR_BRASS_DARK);
    host.fill_rect(x, y, w, h, chrome::COLOR_BRASS);
    frame(host, x, y, w, h, chrome::COLOR_GOLD_WARM);
    let nw = host.shell().hint_text_width_px(note);
    host.draw_hint_text(
        note,
        540 - nw / 2,
        y + (h - host.shell().hint_glyph_px()) / 2,
        chrome::COLOR_TEXT_ON_PARCH,
    );
}

/// 中央副注（墨线纸面文字）。
fn sub_note(host: &mut dyn SceneHost, text: &str, y: i32) {
    let w = host.shell().hint_text_width_px(text);
    host.draw_hint_text(text, 540 - w / 2, y, COLOR_TEXT_ON_PARCH);
}

/// 手绘水手（方阵单元）：三角帽 + 头 + 身 + 持械弯刀；`armed` = 有兵器（武装）。
fn draw_sailor(host: &mut dyn SceneHost, x: i32, y: i32, armed: bool, color: u16) {
    // 三角帽（上尖 + 帽檐）
    host.fill_rect(x + 1, y, 8, 3, chrome::COLOR_BRASS_DARK);
    host.fill_rect(x, y + 3, 10, 3, chrome::COLOR_BRASS);
    // 头
    host.fill_rect(x + 2, y + 6, 6, 7, color);
    // 身（长袍）
    host.fill_rect(x + 1, y + 13, 8, 20, color);
    host.fill_rect(x, y + 30, 10, 3, chrome::COLOR_BRASS_DARK);
    if armed {
        // 弯刀（斜阶 = 刀）
        host.fill_rect(x + 9, y + 8, 3, 12, chrome::COLOR_GOLD_WARM);
        host.fill_rect(x + 10, y + 8, 2, 14, chrome::COLOR_GOLD_WARM);
    }
}

/// 方阵（`cols × rows` 个水手，`armed_n` 前 N 个持械）。
#[allow(clippy::too_many_arguments)]
fn draw_squad(
    host: &mut dyn SceneHost,
    x0: i32,
    y0: i32,
    cols: usize,
    rows: usize,
    armed_n: u32,
    color: u16,
    gap: i32,
) {
    let mut armed_left = armed_n;
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            let armed = idx < usize::try_from(armed_left).unwrap_or(0);
            let px = x0 + i32::try_from(c).unwrap_or(0) * gap;
            let py = y0 + i32::try_from(r).unwrap_or(0) * gap;
            draw_sailor(host, px, py, armed, color);
            if armed && armed_left > 0 {
                armed_left -= 1;
            }
        }
    }
}

/// 兵力数字铜签（「人手 N / 武器 M」）。
fn force_badge(host: &mut dyn SceneHost, txt: &str, x: i32, y: i32) {
    let w = host.shell().hint_text_width_px(txt) + 32;
    host.fill_rect(x - w / 2, y - 24, w, 40, chrome::COLOR_BRASS);
    frame(host, x - w / 2, y - 24, w, 40, chrome::COLOR_GOLD_WARM);
    let tw = host.shell().hint_text_width_px(txt);
    host.draw_hint_text(txt, x - tw / 2, y - 4, chrome::COLOR_TEXT_ON_PARCH);
}

/// 阶段 1 · 兵对兵：羊皮底 + 攻/守手绘水手方阵对峙 + 中央碰撞火花 + 双方数字跳减。
fn draw_clash(host: &mut dyn SceneHost) {
    parchment_vs_bg(host);
    arena_panel(host, 60, 130, 960, 1660);
    title_plaque(host, i18n::sget("scene.vs.anim_clash"));
    // 顶部守方方阵（守军 20 人，前 12 持械）+ 守军徽（BOSS 缩略，黄铜框内）
    host.fill_rect(388, 160, 304, 220, chrome::COLOR_BRASS_DARK);
    host.blit_image(chrome::VS_BOSS_CAPTAIN, 396, 168, 288, 204);
    draw_squad(host, 170, 420, 5, 4, DEMO_DEF_ARMS, COLOR_DANGER, 48);
    // 底部攻方方阵（舰队 30 人，前 18 持械）+ 玩家船
    draw_squad(host, 120, 1320, 5, 6, DEMO_ATK_ARMS, COLOR_PLAYER, 48);
    host.blit_image(chrome::VS_SHIP_PLAYER, 340, 1600, 400, 260);
    // 中央碰撞线 + 火花 + 跳减数字（铜签）
    for i in 0i32..7 {
        let sx = 300 + i * 80;
        host.fill_rect(sx, 940 + (i % 2) * 30, 30, 4, chrome::COLOR_GOLD_WARM);
        host.fill_rect(sx + 10, 960 + (i % 2) * 30, 12, 12, COLOR_BRASS);
    }
    force_badge(host, &i18n::sgetf("scene.vs.dmg_fmt", &[&DEMO_DEF_LOSS]), 540, 900);
    force_badge(host, &i18n::sgetf("scene.vs.dmg_fmt", &[&DEMO_ATK_LOSS]), 540, 1040);
    sub_note(host, i18n::sget("scene.vs.anim_clash_note"), 1100);
    // 双方兵力数字（阵边铜签）
    force_badge(
        host,
        &i18n::sgetf("scene.vs.siege_force_fmt", &[&DEMO_DEF_CREW, &DEMO_DEF_ARMS]),
        540,
        380,
    );
    force_badge(
        host,
        &i18n::sgetf("scene.vs.siege_force_fmt", &[&DEMO_ATK_CREW, &DEMO_ATK_ARMS]),
        540,
        1290,
    );
    bottom_plaque(host, i18n::sget("scene.vs.anim_clash_note"));
}

/// 阶段 2 · 将对将：羊皮底 + 双首领单挑（VS + 挥砍弧线 + 火花）。
fn draw_duel(host: &mut dyn SceneHost) {
    parchment_vs_bg(host);
    arena_panel(host, 60, 130, 960, 1660);
    title_plaque(host, i18n::sget("scene.vs.anim_duel"));
    // 双首领（黄铜框内立绘）
    host.fill_rect(70, 640, 430, 640, chrome::COLOR_BRASS_DARK);
    host.blit_image(chrome::VS_HERO_BACK, 80, 650, 410, 620);
    host.fill_rect(580, 380, 430, 640, chrome::COLOR_BRASS_DARK);
    host.blit_image(chrome::VS_BOSS_CAPTAIN, 590, 390, 410, 620);
    // 挥砍弧线 + 火花（中央）
    for i in 0i32..6 {
        let sx = 440 + i * 40;
        host.fill_rect(sx, 980 - i * 24, 36, 5, chrome::COLOR_GOLD_WARM);
    }
    for i in 0..9 {
        let sx = 480 + (i % 3) * 30;
        let sy = 900 + (i / 3) * 36;
        host.fill_rect(sx, sy, 10, 10, COLOR_BRASS);
    }
    // VS 铜章
    force_badge(host, i18n::sget("scene.vs.anim_versus"), 540, 1060);
    sub_note(host, i18n::sget("scene.vs.anim_duel_note"), 1500);
    bottom_plaque(host, i18n::sget("scene.vs.anim_duel_note"));
}

/// 阶段 3 · 胜负 + 嘲讽：羊皮底 + 中央大横幅（胜/败）+ 嘲讽语 + 战报（兵力条 + 战损）。
fn draw_result(host: &mut dyn SceneHost, won: bool) {
    parchment_vs_bg(host);
    arena_panel(host, 60, 130, 960, 1660);
    let banner = if won {
        i18n::sget("scene.vs.anim_win_banner")
    } else {
        i18n::sget("scene.vs.anim_lose_banner")
    };
    // 中央大横幅（黄铜大字 + 双翼饰线）
    let bw = 640;
    let bx = 540 - bw / 2;
    let by = 640;
    host.fill_rect(bx - 16, by - 16, bw + 32, 150, chrome::COLOR_BRASS_DARK);
    host.fill_rect(bx, by, bw, 118, chrome::COLOR_BRASS);
    frame(host, bx, by, bw, 118, chrome::COLOR_GOLD_WARM);
    let tw = host.shell().hint_text_width_px(banner);
    host.draw_hint_text(banner, 540 - tw / 2, by + 40, chrome::COLOR_GOLD_WARM);
    // 嘲讽语（纸面墨字）
    let taunt =
        if won { i18n::sget("scene.vs.taunt_win") } else { i18n::sget("scene.vs.taunt_lose") };
    sub_note(host, taunt, 860);
    // 战报：攻方/守方兵力条（黄铜槽，剩余 = 填充；战损 = 右端空）
    let (ax, ay, aw, ah) = (150, 1060, 780, 44);
    let def_crew_left = DEMO_DEF_CREW.saturating_sub(DEMO_DEF_LOSS);
    let atk_crew_left = DEMO_ATK_CREW.saturating_sub(DEMO_ATK_LOSS);
    // 守方（血条，满 = 原兵力 20；填充 = 剩余 16）
    host.fill_rect(ax, ay, aw, ah, chrome::COLOR_BRASS_DARK);
    host.fill_rect(
        ax,
        ay,
        aw * i32::try_from(def_crew_left).unwrap_or(0)
            / i32::try_from(DEMO_DEF_CREW.max(1)).unwrap_or(1),
        ah,
        COLOR_DANGER,
    );
    frame(host, ax, ay, aw, ah, COLOR_BRASS);
    let def_txt = i18n::sgetf("scene.vs.siege_force_fmt", &[&def_crew_left, &DEMO_DEF_ARMS]);
    force_badge(host, &def_txt, 540, ay - 8);
    // 攻方（青绿条）
    host.fill_rect(ax, ay + 120, aw, ah, chrome::COLOR_BRASS_DARK);
    host.fill_rect(
        ax,
        ay + 120,
        aw * i32::try_from(atk_crew_left).unwrap_or(0)
            / i32::try_from(DEMO_ATK_CREW.max(1)).unwrap_or(1),
        ah,
        COLOR_PLAYER,
    );
    frame(host, ax, ay + 120, aw, ah, COLOR_BRASS);
    let atk_txt = i18n::sgetf("scene.vs.siege_force_fmt", &[&atk_crew_left, &DEMO_ATK_ARMS]);
    force_badge(host, &atk_txt, 540, ay + 112);
    // 战损行 + 结束提示
    let loss = if won {
        i18n::sgetf("scene.vs.anim_loss_fmt", &[&DEMO_DEF_LOSS])
    } else {
        i18n::sgetf("scene.vs.anim_loss_fmt", &[&DEMO_ATK_LOSS])
    };
    sub_note(host, &loss, ay + 230);
    bottom_plaque(host, i18n::sget("scene.vs.anim_result_note"));
}

/// 黄铜框（4 边 2px；`x,y,width,height` = 面板矩形，`color` = 描边色）。
fn frame(host: &mut dyn SceneHost, x: i32, y: i32, width: i32, height: i32, color: u16) {
    host.fill_rect(x, y, width, 2, color);
    host.fill_rect(x, y + height - 2, width, 2, color);
    host.fill_rect(x, y, 2, height, color);
    host.fill_rect(x + width - 2, y, 2, height, color);
}
