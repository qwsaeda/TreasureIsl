//! 战斗场景（S9 骨架）：回合推进 / 战斗面板绘制。
use ti_core::domain::combat::Side;

use crate::game_app::{GameInput, HitData, SiegeSnapshot};
use crate::i18n;
use crate::scenes::chrome;
use crate::scenes::chrome::{COLOR_DANGER, COLOR_PLAYER};
use crate::scenes::{Scene, SceneHost, SceneKind};

/// 战斗场景处理器（`Vs`）。
pub struct VsScene;

impl Scene for VsScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Vs
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Attack => {
                // S33.5：攻占战报（`siege_view` 在位）= 一次性确定性结算（`assault`），
                // 非 HP 回合制；已结算再 Attack = no-op（结果保留供 Back 退出）。
                if host.siege_snapshot().is_some() {
                    host.settle_siege();
                } else {
                    host.battle_step();
                }
                false
            }
            GameInput::Back => {
                // S33.5：攻占战报 Back = 清战报回主地图（结果已写回账本）。
                if host.siege_snapshot().is_some() {
                    host.enter(SceneKind::MainMap);
                // S28.6：副本战结束/中途退出 = 回主地图（落格续局，territory.md §4）；
                // BOSS 战 = 回主菜单（`spec/battle.md §d`，状态码 25→0）
                } else if host.in_dungeon_battle() {
                    host.enter(SceneKind::MainMap);
                } else {
                    host.enter(SceneKind::Index);
                }
                true
            }
            _ => false,
        }
    }

    /// S29.2：点任意处（`key=0`）/ 上·左·右滑 = 攻击一回合；下滑 = 退出战斗。
    /// S49.4 交互契约（§2.6.1）：`pick_key` 2=跳过（快进横幅）/ 3=确认（结束回主地图）/
    /// 4=重打（重置演出）；无命中 = 现有 swipe/点按语义。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        match hit.key {
            2 => {
                crate::scenes::vs_anim_proto::skip_anim_to_banner();
                false
            }
            3 => host.advance(GameInput::Back),
            4 => {
                crate::scenes::vs_anim_proto::reset_anim();
                false
            }
            _ => match hit.swipe {
                2 => host.advance(GameInput::Back),
                _ => host.advance(GameInput::Attack),
            },
        }
    }

    /// 战斗面板（双方 HP 进度条 + 回合 + 结果，v2 羊皮纸/木纹/金色系）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        // S33.5：攻占战报（`siege_view` 在位）= 兵力战报视图（攻方舰队 vs 守方驻军+建筑防御），
        // 与 HP 回合制战斗分叉渲染（`assault` 一次性确定性，Attack 结算出结果）。
        if let Some(s) = host.siege_snapshot() {
            render_siege(host, &s);
            return;
        }
        // S27.7 选型期：环境变量 `TI_VS_PROTO=a|b|c|d|e` ⇒ 走 5 套候选版式原型渲染
        // （`scenes::vs_proto`）；未设置 = 正式版式（旧布局不变）。
        // 2026-10-30 用户拍板 VS v2（兵对兵→将对将→胜负+嘲讽）：5 套原型全作废，
        // `TI_VS_ANIM_PROTO=clash|duel|result_win|result_lose` ⇒ 战斗动画概念帧（设计稿先行）。
        if let Some(phase) = crate::scenes::vs_anim_proto::selected_phase() {
            crate::scenes::vs_anim_proto::render(host, phase);
            return;
        }
        if let Some(variant) = crate::scenes::vs_proto::selected_variant() {
            crate::scenes::vs_proto::render(host, variant);
            return;
        }
        let layout = chrome::scene_layout("vs");
        // S34.6 D3（用户 2026-11-03「之前设计的很好，你又瞎JB的乱设计」）：vs.rs 正式战斗
        // **完整复用 `vs_anim_proto::render_v2_elem` 好设计构图**（甲板插画 + crew/hero/enemy
        // 分列 + 顶/底兵力状态框 + 中央 VS 徽标 + 羊皮框），不再画 vs_theme 背景 +
        // scenes.json 海域图原语 + 叠 hero/enemy 小人（用户「屎中屡」根因 = 正式渲染不像
        // 已批的 vs_anim 好设计）。phase 由 battle 结果推导（None→clash 兵对兵，
        // Player→result_win，Boss→result_lose）；非战斗 = 原空态。
        if let Some(b) = host.battle_snapshot() {
            let phase = match b.outcome {
                Some(Side::Player) => "result_win",
                Some(Side::Enemy) => "result_lose",
                None => "clash",
            };
            // S34.6 D4（spec §2.5 数据绑定）：兵力 = 运行态（`BattleSnapshot` 席位舰队/守方），
            // 替换演示常量（用户 2026-11-03「SB设计，阅读设计规范」= 构图对但兵力假）。
            // S34.6 D5（兵力模型迁移）：`resolved` = 已结算 → result 帧用 `after_crew/arms`
            // （真实战后，不再按 loss_pct 推）；未结算 → clash 战前兵力。
            // S34.6 D6（格类分层 spec §2.4）：格类 = 副本战（dungeon）→ raid / BOSS → boss；
            // 背景按格类（`bg_dungeon`/`bg_boss`），修复「BOSS 战显示甲板非炮台」（用户屎根因）。
            let grid_class = if b.is_dungeon { "raid" } else { "boss" };
            crate::scenes::vs_anim_proto::render_v2_elem(
                host,
                phase,
                if b.resolved { b.after_crew } else { b.attacker_crew },
                if b.resolved { b.after_arms } else { b.attacker_arms },
                b.defender_crew,
                b.defender_arms,
                b.resolved,
                grid_class,
            );
            return;
        }
        // 非战斗（空态）：vs_theme 背景 + 原语 + 空文案。
        let class = crate::scenes::vs_anim_proto::selected_grid_class();
        let th = crate::scenes::vs_anim_proto::vs_theme();
        let bg = crate::scenes::vs_anim_proto::vs_bg_asset(class, th.as_ref());
        // S38 方案 A：横屏 = 画布 1920×1080（空态背景铺满横屏）；竖屏 = 原 1080×1920。
        let landscape =
            host.shell().settings.orientation == ti_platform::settings::Orientation::Landscape;
        let (bw, bh) = if landscape { (1920, 1080) } else { (1080, 1920) };
        host.blit_image_tint(&bg, 0, 0, bw, bh, crate::scenes::vs_anim_proto::vs_tint(th.as_ref()));
        chrome::render_primitives_filtered(host, "vs", -1, 4);
        {
            // S38：横屏 empty 文案锚点走 `landscape.extra.empty`（回退竖屏 `extra.empty`）。
            let em = layout.extra_for(landscape).get("empty").copied().unwrap_or_default();
            let text = i18n::sget("scene.vs.empty");
            let text_x = chrome::centered_body_x(host, text);
            chrome::draw_body_text(host, text, text_x, em.y);
        }
    }
}

/// S34.6 D3（用户 2026-11-03）：正式战斗复用 `vs_anim_proto::render_v2_elem` 好设计构图，
/// 原 `render_battle_bars`（HP 双槽 + 回合/结果文本）已退役——好设计自带顶/底兵力状态框 +
/// VS 徽标 + 甲板插画 + crew/hero/enemy 分列，无需另画 HP 槽（海域图 + 叠小人的「屎」版废除）。
/// S33.5 攻占战报渲染：双方兵力条（攻方舰队 vs 守方驻军+建筑防御）+ 结果行。
///
/// 与 HP 回合制战斗分叉：`assault` 是**一次性确定性结算**（`territory.md` §4），
/// 无回合推进，`Attack` 出结果、`Back` 回主地图。格类标题 = 攻占（船队攻打建筑）。
#[allow(clippy::too_many_lines)]
fn render_siege(host: &mut dyn SceneHost, snap: &SiegeSnapshot) {
    let layout = chrome::scene_layout("vs");
    // S34.5 延伸：vs_theme 攻占战背景（格类=attack）替换夜空底（bg_night z=3）。
    let class = "attack";
    let th = crate::scenes::vs_anim_proto::vs_theme();
    let bg = crate::scenes::vs_anim_proto::vs_bg_asset(class, th.as_ref());
    // S38 方案 A：横屏 = 画布 1920×1080（攻占战报背景铺满横屏）；竖屏 = 原 1080×1920。
    let landscape =
        host.shell().settings.orientation == ti_platform::settings::Orientation::Landscape;
    let (bw, bh) = if landscape { (1920, 1080) } else { (1080, 1920) };
    host.blit_image_tint(&bg, 0, 0, bw, bh, crate::scenes::vs_anim_proto::vs_tint(th.as_ref()));
    chrome::render_primitives_filtered(host, "vs", -1, 4);
    let theme = host.theme().clone();
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
    let isle_key = format!("isle.{}", snap.isle);
    let isle_name = crate::i18n::sget(&isle_key);
    // 标题：攻占（船队攻打建筑）
    let title = crate::i18n::sgetf("scene.vs.siege_title", &[&isle_name]);
    let title_x = chrome::centered_body_x(host, &title);
    chrome::draw_body_text(host, &title, title_x, 64);

    // 攻方/守方兵力条（攻方舰队 / 守方驻军+防御）—— S38：横屏走 `landscape.extra`。
    let ph_bar = layout.extra_for(landscape).get("player_hp").copied().unwrap_or_default();
    let bh_bar = layout.extra_for(landscape).get("boss_hp").copied().unwrap_or_default();
    let atk_label = i18n::sget("scene.vs.siege_attacker");
    let def_label = i18n::sget("scene.vs.siege_defender");
    // 攻方条（青绿）
    host.fill_rect(ph_bar.x, ph_bar.y, ph_bar.w, ph_bar.h, COLOR_PLAYER);
    host.fill_rect(ph_bar.x, ph_bar.y, ph_bar.w, 2, brass);
    host.fill_rect(ph_bar.x, ph_bar.y + ph_bar.h - 2, ph_bar.w, 2, brass);
    host.fill_rect(ph_bar.x, ph_bar.y, 2, ph_bar.h, brass);
    host.fill_rect(ph_bar.x + ph_bar.w - 2, ph_bar.y, 2, ph_bar.h, brass);
    host.draw_hint_text(atk_label, ph_bar.x, ph_bar.y - 32, chrome::COLOR_TEXT_ON_PARCH);
    let atk_num =
        i18n::sgetf("scene.vs.siege_force_fmt", &[&snap.attacker_crew, &snap.attacker_arms]);
    host.draw_hint_text(
        &atk_num,
        ph_bar.x + ph_bar.w - 8 - host.shell().hint_text_width_px(&atk_num),
        ph_bar.y + (ph_bar.h - host.shell().hint_glyph_px()) / 2,
        chrome::COLOR_TEXT_ON_PARCH,
    );
    // 守方条（血红）
    host.fill_rect(bh_bar.x, bh_bar.y, bh_bar.w, bh_bar.h, COLOR_DANGER);
    host.fill_rect(bh_bar.x, bh_bar.y, bh_bar.w, 2, brass);
    host.fill_rect(bh_bar.x, bh_bar.y + bh_bar.h - 2, bh_bar.w, 2, brass);
    host.fill_rect(bh_bar.x, bh_bar.y, 2, bh_bar.h, brass);
    host.fill_rect(bh_bar.x + bh_bar.w - 2, bh_bar.y, 2, bh_bar.h, brass);
    host.draw_hint_text(
        def_label,
        bh_bar.x + bh_bar.w - host.shell().hint_text_width_px(def_label),
        bh_bar.y - 32,
        chrome::COLOR_TEXT_ON_PARCH,
    );
    let def_num =
        i18n::sgetf("scene.vs.siege_force_fmt", &[&snap.defender_crew, &snap.defender_arms]);
    host.draw_hint_text(
        &def_num,
        bh_bar.x + bh_bar.w - 8 - host.shell().hint_text_width_px(&def_num),
        bh_bar.y + (bh_bar.h - host.shell().hint_glyph_px()) / 2,
        chrome::COLOR_TEXT_ON_PARCH,
    );

    // 结果行（`outcome` 未结算 = 提示按 Attack；已结算 = 胜/败）—— S38：横屏走 `landscape.extra`。
    let rd_pos = layout.extra_for(landscape).get("rounds").copied().unwrap_or_default();
    let oc_pos = layout.extra_for(landscape).get("outcome").copied().unwrap_or_default();
    match snap.won {
        Some(true) => {
            let text = i18n::sget("scene.vs.siege_win");
            let text_x = chrome::centered_body_x(host, text);
            chrome::draw_body_text(host, text, text_x, oc_pos.y);
        }
        Some(false) => {
            let text = i18n::sget("scene.vs.siege_loss");
            let text_x = chrome::centered_body_x(host, text);
            chrome::draw_body_text(host, text, text_x, oc_pos.y);
        }
        None => {
            let text = i18n::sget("scene.vs.siege_prompt");
            let text_x = chrome::centered_body_x(host, text);
            chrome::draw_body_text(host, text, text_x, rd_pos.y);
        }
    }
}
