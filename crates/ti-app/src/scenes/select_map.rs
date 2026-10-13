//! 地图选择场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件。
use ti_core::domain::flow::Sfx;
use ti_core::domain::selection::MapSlot;

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
    #[allow(clippy::cast_precision_loss)]
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                // N3：命中带与渲染 `item_rect` 同公式（scenes.json select_map 段），正逆成对
                let scene = ti_core::data::scenes()
                    .ok()
                    .and_then(|s| s.get("select_map"))
                    .cloned()
                    .unwrap_or_default();
                let bands = layout::scene_list_bands(
                    scene.list.x as f32,
                    scene.list.x as f32 + scene.list.w as f32,
                    scene.list.y as f32,
                    scene.list.row_h as f32,
                    scene.list.item_h as f32,
                    MAP_COUNT as usize,
                );
                let Some(i) = layout::scene_list_index_at(&bands, sample.x, sample.y) else {
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
        // P1-3：坐标/文案硬编码改读 `ti_core::data::scenes()`（scenes.json 字段已建）
        let layout = ti_core::data::scenes()
            .ok()
            .and_then(|s| s.get("select_map"))
            .cloned()
            .unwrap_or_default();
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.select_map.title"), layout.title.x, layout.title.y);
        let current = host.map_slot().map_or(0, |m| m.zero_index() as usize);
        for (i, m) in MapSlot::all().iter().enumerate() {
            let y = layout.list.item_rect(i).y;
            let locked = !host.map_unlocked(*m);
            if i == current && !locked {
                let r = layout.list.item_rect(i);
                host.fill_rect(r.x, r.y, r.w, r.h, COLOR_SELECTED);
            }
            host.draw_text(m.label(), layout.list.text_x, y);
            if locked {
                let text_x = layout.extra.get("text_x").map_or(0, |v| v.x);
                host.draw_text(i18n::sget("scene.select_map.locked"), text_x, y);
            }
        }
        host.draw_text(i18n::sget("scene.select_map.nav_hint"), layout.hint.x, layout.hint.y);
    }
}
