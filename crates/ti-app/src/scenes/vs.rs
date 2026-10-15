//! 战斗场景（S9 骨架）：回合推进 / 战斗面板绘制。
use ti_core::domain::combat::Side;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_DANGER, COLOR_LANDMARK, COLOR_PLAYER, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::TouchSample;
use ti_platform::settings::Gesture;

/// 战斗场景处理器（`Vs`）。
pub struct VsScene;

impl Scene for VsScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Vs
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Attack => {
                host.battle_step();
                false
            }
            GameInput::Back => {
                // 战斗结束/中途退出 → 回主菜单（`spec/battle.md §d`，状态码 25→0）
                host.enter(SceneKind::Index);
                true
            }
            _ => false,
        }
    }

    /// 点任意处 = 攻击一回合；下滑 = 退出战斗。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap | Gesture::SwipeUp | Gesture::SwipeLeft | Gesture::SwipeRight => {
                host.advance(GameInput::Attack)
            }
            Gesture::SwipeDown => host.advance(GameInput::Back),
        }
    }

    /// 战斗面板（双方 HP 进度条 + 回合 + 结果，S14 航海图纸质系）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        let layout =
            ti_core::data::scenes().ok().and_then(|s| s.get("vs")).cloned().unwrap_or_default();
        host.fill_rect(0, 0, 176, layout.title.h, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.vs.title"), layout.title.x, layout.title.y);
        if let Some(b) = host.battle_snapshot() {
            // S14：血条进度条（玩家橙红 / BOSS 海绿，x=16 w=144 h=8，scenes.json 定位置）
            let ph_bar = layout.extra.get("player_hp").copied().unwrap_or_default();
            let bh_bar = layout.extra.get("boss_hp").copied().unwrap_or_default();
            let rd_pos = layout.extra.get("rounds").copied().unwrap_or_default();
            let oc_pos = layout.extra.get("outcome").copied().unwrap_or_default();
            let ph_w = b.player_hp * u32::try_from(ph_bar.w).unwrap_or(0) / b.player_hp_max.max(1);
            let bh_w = b.boss_hp * u32::try_from(bh_bar.w).unwrap_or(0) / b.boss_hp_max.max(1);
            // 玩家血条（橙红 #FF6347）
            host.fill_rect(ph_bar.x, ph_bar.y, ph_bar.w, ph_bar.h, COLOR_DANGER);
            host.fill_rect(
                ph_bar.x,
                ph_bar.y,
                i32::try_from(ph_w).unwrap_or(0),
                ph_bar.h,
                COLOR_PLAYER,
            );
            // BOSS 血条（海绿 #2E8B57）
            host.fill_rect(bh_bar.x, bh_bar.y, bh_bar.w, bh_bar.h, COLOR_LANDMARK);
            host.fill_rect(
                bh_bar.x,
                bh_bar.y,
                i32::try_from(bh_w).unwrap_or(0),
                bh_bar.h,
                COLOR_PLAYER,
            );
            let rd = i18n::sgetf("scene.vs.rounds_fmt", &[&b.rounds]);
            host.draw_text(&rd, rd_pos.x, rd_pos.y);
            match b.outcome {
                Some(Side::Player) => {
                    host.draw_text(i18n::sget("scene.vs.victory"), oc_pos.x, oc_pos.y);
                }
                Some(Side::Enemy) => {
                    host.draw_text(i18n::sget("scene.vs.defeat"), oc_pos.x, oc_pos.y);
                }
                None => {
                    let aw = layout.extra.get("empty").copied().unwrap_or_default();
                    host.draw_text(i18n::sget("scene.vs.awaiting"), aw.x, aw.y);
                }
            }
        } else {
            let em = layout.extra.get("empty").copied().unwrap_or_default();
            host.draw_text(i18n::sget("scene.vs.empty"), em.x, em.y);
        }
        host.draw_text(i18n::sget("scene.vs.hint"), layout.hint.x, layout.hint.y);
    }
}
