//! 主菜单场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件，
//! 宿主只提供状态读写能力（`SceneHost`）。
use ti_core::oracle::menu::Sfx;

use crate::game_app::GameInput;
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

    /// 复用 `MenuShell::render` 菜单 + 标题 + 状态行机制。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let sel = host.menu_selected();
        host.shell_mut().selected = sel;
        host.shell_mut().render();
    }
}
