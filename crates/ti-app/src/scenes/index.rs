//! 主菜单场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件，
//! 宿主只提供状态读写能力（`SceneHost`）。
use ti_core::domain::flow::Sfx;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_SELECTED, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 主菜单项数（`spec/menu.md §a` case 0..=5）。
const MENU_ITEMS: usize = 6;

/// 主菜单场景处理器（`Index`）。
pub struct IndexScene;

impl Scene for IndexScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Index
    }

    /// 菜单项 0..5（`spec/menu.md §a` case 0..=5）：
    /// - 0 新游戏 → `SelectRole`；1 继续 → `Saveload`；
    /// - 2/3/4 选项/说明 → `Message`（Help kind）；5 退出 → 置 `quit`。
    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::MoveUp => {
                let cur = host.menu_selected();
                host.set_menu_selected(cur.wrapping_add(MENU_ITEMS - 1) % MENU_ITEMS);
                false
            }
            GameInput::MoveDown => {
                let cur = host.menu_selected();
                host.set_menu_selected((cur + 1) % MENU_ITEMS);
                false
            }
            GameInput::Confirm => {
                host.play_sfx(Sfx::Press);
                match host.menu_selected() {
                    0 => {
                        host.reset_selection();
                        host.enter(SceneKind::SelectRole);
                        true
                    }
                    1 => {
                        host.enter(SceneKind::Saveload);
                        true
                    }
                    2..=4 => {
                        host.open_help();
                        true
                    }
                    5 => {
                        host.request_quit();
                        host.enter(SceneKind::Exit);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// 点菜单行 = 选中该行并立即执行（`docs/modernization.md` §5）。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        if sample.gesture != Gesture::Tap {
            return false;
        }
        let Some(idx) = layout::menu_row_at(sample.x, sample.y) else {
            return false;
        };
        host.set_menu_selected(idx);
        host.advance(GameInput::Confirm)
    }

    /// 主菜单渲染：坐标全读 `scenes.json` 的 `index` 段（`list.y`=48 / `list.row_h`=32 等）。
    /// 旧版 `menu_shell` 的 `MENU_TOP`=96 / `ROW_H`=20 死坐标路径已删除（FIND-101）。
    #[allow(
        clippy::needless_range_loop,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        let layout =
            ti_core::data::scenes().ok().and_then(|s| s.get("index")).cloned().unwrap_or_default();
        let title_h = layout.title.h.max(1);
        host.fill_rect(0, 0, 176, title_h, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.index.title"), layout.title.x, layout.title.y);
        let labels = crate::shell_ui::menu_labels();
        let sel = host.menu_selected();
        for i in 0..labels.len() {
            let r = layout.list.item_rect(i);
            if i == sel {
                host.fill_rect(r.x, r.y, r.w, r.h, COLOR_SELECTED);
            }
            host.draw_text(labels[i], layout.list.text_x, layout.list.item_text_pos(i).1);
        }
        let [glyphs, sounds, modules] = crate::shell_ui::status_texts(
            host.shell().font_glyphs,
            host.shell().wav_count,
            host.shell().mod_count,
        );
        let status = layout.extra.get("status").copied().unwrap_or_default();
        let cursor = status.x;
        host.draw_text(&glyphs, cursor, status.y);
        let cursor2 = status.x + i32::try_from(sounds.chars().count()).unwrap_or(0) * 12;
        host.draw_text(&sounds, cursor2, status.y);
        let cursor3 = cursor2 + i32::try_from(modules.chars().count()).unwrap_or(0) * 12;
        host.draw_text(&modules, cursor3, status.y);
    }
}
