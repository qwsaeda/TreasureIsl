//! 战斗场景（S9 骨架）：回合推进 / 战斗面板绘制。
use ti_core::oracle::battle::Side;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_TITLE_BAR};
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

    /// 战斗面板（双方 HP + 回合 + 结果）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.vs.title"), 64, 4);
        match host.battle_snapshot() {
            Some(b) => {
                let ph = i18n::sgetf("scene.vs.player_hp_fmt", &[&b.player_hp]);
                let bh = i18n::sgetf("scene.vs.boss_hp_fmt", &[&b.boss_hp]);
                let rd = i18n::sgetf("scene.vs.rounds_fmt", &[&b.rounds]);
                host.draw_text(&ph, 10, 50);
                host.draw_text(&bh, 10, 80);
                host.draw_text(&rd, 10, 110);
                match b.outcome {
                    Some(Side::Player) => {
                        host.draw_text(i18n::sget("scene.vs.victory"), 60, 150);
                    }
                    Some(Side::Enemy) => {
                        host.draw_text(i18n::sget("scene.vs.defeat"), 60, 150);
                    }
                    None => host.draw_text(i18n::sget("scene.vs.awaiting"), 50, 150),
                }
            }
            None => {
                host.draw_text(i18n::sget("scene.vs.empty"), 60, 50);
            }
        }
        host.draw_text(i18n::sget("scene.vs.hint"), 60, 210);
    }
}
