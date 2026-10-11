//! 地图选择场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件。
use ti_core::oracle::menu::Sfx;
use ti_core::oracle::selection::MapSlot;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_SELECTED, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 地图槽数（`MapSlot::all()` 长度，`spec/menu.md §c`）。
const MAP_COUNT: u32 = 3;

/// 地图选择场景处理器（`SelectMap`）。
pub struct SelectMapScene;

/// 读「角色 + 地图是否齐全」两个布尔（`rules::selection` 判据）。
fn selection_done(host: &dyn SceneHost) -> bool {
    let s = host.selection();
    s.has_character() && s.has_map()
}

impl Scene for SelectMapScene {
    fn kind(&self) -> SceneKind {
        SceneKind::SelectMap
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Prev => {
                let cur = host.map_slot().map_or(0, MapSlot::zero_index);
                let n = if cur == 0 { MAP_COUNT - 1 } else { cur - 1 };
                host.set_map_slot(MapSlot::from_index(n));
                host.play_sfx(Sfx::Press);
                false
            }
            GameInput::Next => {
                let cur = host.map_slot().map_or(MAP_COUNT - 1, MapSlot::zero_index);
                let n = (cur + 1) % MAP_COUNT;
                host.set_map_slot(MapSlot::from_index(n));
                host.play_sfx(Sfx::Press);
                false
            }
            GameInput::Confirm => {
                let Some(slot) = host.map_slot() else {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                };
                if !host.map_unlocked(slot) {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                }
                host.selection_mut().map_slot = Some(slot);
                let done = selection_done(host);
                host.selection_mut().completed = done;
                if done {
                    host.enter_main_map();
                    true
                } else {
                    host.play_sfx(Sfx::Press);
                    false
                }
            }
            GameInput::Back => {
                host.enter(SceneKind::SelectRole);
                true
            }
            _ => false,
        }
    }

    /// 点某行 = 选中该地图并确认；下滑 = 返回。
    ///
    /// 未解锁地图点按由 `on_input` 的 `Confirm` 分支判 `Sfx::Invalid`。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                let Some(i) = layout::list_index_at(sample.x, sample.y, MapSlot::all().len())
                else {
                    return false;
                };
                host.set_map_slot(MapSlot::from_index(u32::try_from(i).unwrap_or(0)));
                host.advance(GameInput::Confirm)
            }
            Gesture::SwipeDown => host.advance(GameInput::Back),
            _ => false,
        }
    }

    /// 地图列表 + 选中高亮 + 锁定标记 + 导航提示。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.select_map.title"), 40, 4);
        let current = host.map_slot().map_or(0, |m| m.zero_index() as usize);
        for (i, m) in MapSlot::all().iter().enumerate() {
            let y = 50 + i32::try_from(i).unwrap_or(0) * 30;
            let locked = !host.map_unlocked(*m);
            if i == current && !locked {
                host.fill_rect(20, y - 10, 136, 26, COLOR_SELECTED);
            }
            host.draw_text(m.label(), 60, y);
            if locked {
                host.draw_text(i18n::sget("scene.select_map.locked"), 130, y);
            }
        }
        host.draw_text(i18n::sget("scene.select_map.nav_hint"), 30, 210);
    }
}
