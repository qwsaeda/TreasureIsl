//! 角色选择场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件。
use ti_core::domain::flow::Sfx;
use ti_core::domain::selection::Character;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_SELECTED, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 角色数（`Character::all()` 长度，`spec/menu.md §b`）。
const ROLE_COUNT: u32 = 5;

/// 角色选择场景处理器（`SelectRole`）。
pub struct SelectRoleScene;

/// 读「角色 + 地图是否齐全」两个布尔（`rules::selection` 判据）。
fn selection_done(host: &dyn SceneHost) -> bool {
    let s = host.selection();
    s.has_character() && s.has_map()
}

impl Scene for SelectRoleScene {
    fn kind(&self) -> SceneKind {
        SceneKind::SelectRole
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Prev => {
                let cur = host.character().map_or(0, Character::index);
                let n = if cur == 0 { ROLE_COUNT - 1 } else { cur - 1 };
                let picked = Character::from_index(n);
                host.set_character(picked);
                host.selection_mut().character = picked;
                host.play_sfx(Sfx::Press);
                false
            }
            GameInput::Next => {
                let cur = host.character().map_or(ROLE_COUNT - 1, Character::index);
                let n = (cur + 1) % ROLE_COUNT;
                let picked = Character::from_index(n);
                host.set_character(picked);
                host.selection_mut().character = picked;
                host.play_sfx(Sfx::Press);
                false
            }
            GameInput::Confirm => {
                let picked = host.character();
                host.selection_mut().character = picked;
                let done = selection_done(host);
                host.selection_mut().completed = done;
                if done {
                    host.enter_main_map();
                } else {
                    host.enter(SceneKind::SelectMap);
                }
                true
            }
            GameInput::Back => {
                host.enter(SceneKind::Index);
                true
            }
            _ => false,
        }
    }

    /// 点某行 = 选中该角色并确认；下滑 = 返回。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                let Some(i) = layout::list_index_at(sample.x, sample.y, Character::all().len())
                else {
                    return false;
                };
                let picked = Character::from_index(u32::try_from(i).unwrap_or(0));
                host.set_character(picked);
                host.selection_mut().character = picked;
                host.advance(GameInput::Confirm)
            }
            Gesture::SwipeDown => host.advance(GameInput::Back),
            _ => false,
        }
    }

    /// 角色列表 + 选中高亮 + 导航提示。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.select_role.title"), 40, 4);
        let current = host.character().map_or(0, |c| c.index() as usize);
        for (i, c) in Character::all().iter().enumerate() {
            let y = 50 + i32::try_from(i).unwrap_or(0) * 30;
            if i == current {
                host.fill_rect(20, y - 10, 136, 26, COLOR_SELECTED);
            }
            host.draw_text(c.label(), 60, y);
        }
        host.draw_text(i18n::sget("scene.select_role.nav_hint"), 30, 210);
    }
}
