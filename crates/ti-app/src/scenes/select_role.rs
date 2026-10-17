//! 角色选择场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件。
use ti_core::domain::flow::Sfx;
use ti_core::domain::selection::Character;

use crate::game_app::GameInput;
use crate::scenes::chrome;
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
                // 首次进 SelectRole 时 `character`=None（Index 只 `reset_selection` 不设角色），
                // Confirm 兜底选第 0 个角色（对齐 select_map Confirm 的 map_slot 兜底语义）
                if host.character().is_none() {
                    let Some(c) = Character::from_index(0) else {
                        host.play_sfx(Sfx::Invalid);
                        return false;
                    };
                    host.set_character(Some(c));
                }
                let Some(picked) = host.character() else {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                };
                host.selection_mut().character = Some(picked);
                let done = selection_done(host);
                host.selection_mut().completed = done;
                if done {
                    let ok = host.enter_main_map();
                    if ok {
                        true
                    } else {
                        // FIND-204：数据校验失败 → 停留原场景
                        host.play_sfx(Sfx::Invalid);
                        false
                    }
                } else {
                    host.enter(SceneKind::SelectMap);
                    true
                }
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
                let scene = chrome::scene_layout("select_role");
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
                chrome::flash_press();
                host.advance(GameInput::Confirm)
            }
            Gesture::SwipeDown => host.advance(GameInput::Back),
            _ => false,
        }
    }

    /// 角色列表 + 选中高亮 + 导航提示（v2 羊皮纸/木纹/金色系）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let layout = chrome::scene_layout("select_role");
        chrome::scene_shell(host, "select_role", "scene.select_role.title");
        let current = host.character().map_or(0, |c| c.index() as usize);
        let held = host.press_held();
        let flashing = chrome::press_flash_active();
        for (i, c) in Character::all().iter().enumerate() {
            let r = layout.list.item_rect(i);
            chrome::menu_item(
                host,
                r.x,
                r.y,
                r.w,
                r.h,
                i == current,
                i == current && (held || flashing),
            );
            let ty = r.y + (r.h - host.shell().glyph_px()) / 2; // QA009：字形高按活跃字体（TTF 44px / 位图 12px）
            chrome::draw_body_text(host, c.label(), layout.list.text_x, ty);
            // S15.6 批次6：第二行简介（i18n `role.desc.<label>`，44px 次级灰蓝墨，主行之下 +56）
            let desc_key = format!("role.desc.{}", c.label());
            let desc = crate::i18n::sget(&desc_key);
            chrome::draw_body_text_secondary(host, desc, layout.list.text_x, ty + 56);
        }
        chrome::scene_hint(host, "select_role", "scene.select_role.nav_hint");
    }
}
