//! 存读档场景（S9 骨架）：读档/写档判定 + 槽位列表绘制。
use ti_core::oracle::menu::Sfx;
use ti_core::oracle::saveload::SaveSlot;
use ti_core::oracle::selection::MapSlot;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 存档版本号（v2，`rules::saveload` 当前格式）。
const SAVE_VERSION: u32 = 2;

/// 位置字段的无坐标回退值（`GridPos::index` 的网格边长）。
const POSITION_GRID_W: u32 = 20;

/// 存读档场景处理器（`Saveload`）。
pub struct SaveloadScene;

impl Scene for SaveloadScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Saveload
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::LoadSave(slot) => {
                let Some(data) = host.save_slots().get(slot).copied() else {
                    return false;
                };
                host.restore_save(data);
                host.play_sfx(Sfx::Press);
                true
            }
            GameInput::WriteSave(slot) => {
                let character = host.character().map_or(1, |c| c.index() + 1);
                let map_slot = host.map_slot().map_or(0, MapSlot::zero_index);
                let gold = host.gold();
                let position = host.map_pos().map_or(0, |p| p.index(POSITION_GRID_W));
                let data = SaveSlot { version: SAVE_VERSION, character, gold, map_slot, position };
                if host.save_slots().len() > slot {
                    host.write_save_slot(slot, data);
                    host.play_sfx(Sfx::Press);
                }
                false
            }
            GameInput::SaveloadClose | GameInput::Back => {
                host.enter(SceneKind::Index);
                true
            }
            _ => false,
        }
    }

    /// 点有档的槽 = 读档；点空槽 = 存档；下滑 = 关闭。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                let count = host.save_slots().len();
                let hit = (0..count).find(|&i| layout::slot_at(sample.x, sample.y, i));
                let Some(slot) = hit else {
                    return false;
                };
                // 有档（version > 0）→ 读档；空槽 → 存档
                let has = host.save_slots().get(slot).is_some_and(|s| s.version > 0);
                if has {
                    host.advance(GameInput::LoadSave(slot))
                } else {
                    host.advance(GameInput::WriteSave(slot))
                }
            }
            Gesture::SwipeDown => host.advance(GameInput::SaveloadClose),
            _ => false,
        }
    }

    /// 槽位列表 + 每档金/空满标记 + 导航提示。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.saveload.title"), 60, 4);
        let slots: Vec<SaveSlot> = host.save_slots().to_vec();
        for (i, s) in slots.iter().enumerate() {
            let y = 40 + i32::try_from(i).unwrap_or(0) * 40;
            let has = s.version > 0;
            let has_str = if has { i18n::sget("slot.has") } else { i18n::sget("slot.empty") };
            let idx = i + 1;
            let slot_str = i18n::sgetf("slot.label_fmt", &[&idx, &has_str]);
            host.draw_text(&slot_str, 10, y);
            if has {
                let g = i18n::sgetf("hud.gold_fmt", &[&s.gold]);
                host.draw_text(&g, 100, y);
            }
        }
        host.draw_text(i18n::sget("scene.saveload.nav_hint"), 30, 210);
    }
}
