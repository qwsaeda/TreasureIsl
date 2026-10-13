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
    #[allow(clippy::cast_precision_loss)]
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                // N3：命中带与渲染 `item_rect` 同公式（scenes.json select_role 段），正逆成对
                let scene = ti_core::data::scenes()
                    .ok()
                    .and_then(|s| s.get("select_role"))
                    .cloned()
                    .unwrap_or_default();
                let bands = layout::scene_list_bands(
                    scene.list.x as f32,
                    scene.list.x as f32 + scene.list.w as f32,
                    scene.list.y as f32,
                    scene.list.row_h as f32,
                    scene.list.item_h as f32,
                    scene.list.highlight_offset as f32,
                    ROLE_COUNT as usize,
                );
                let Some(i) = layout::scene_list_index_at(&bands, sample.x, sample.y) else {
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
        // P1-3：坐标/文案硬编码改读 `ti_core::data::scenes()`（scenes.json 字段已建）
        let layout = ti_core::data::scenes()
            .ok()
            .and_then(|s| s.get("select_role"))
            .cloned()
            .unwrap_or_default();
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.select_role.title"), layout.title.x, layout.title.y);
        let current = host.character().map_or(0, |c| c.index() as usize);
        for (i, c) in Character::all().iter().enumerate() {
            let y = layout.list.item_text_pos(i).1;
            if i == current {
                let r = layout.list.item_rect(i);
                host.fill_rect(r.x, r.y, r.w, r.h, COLOR_SELECTED);
            }
            host.draw_text(c.label(), layout.list.text_x, y);
        }
        host.draw_text(i18n::sget("scene.select_role.nav_hint"), layout.hint.x, layout.hint.y);
    }
}
