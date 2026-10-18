//! 主菜单场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件，
//! 宿主只提供状态读写能力（`SceneHost`）。
use ti_core::domain::flow::Sfx;

use crate::game_app::GameInput;
use crate::scenes::chrome;
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

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
                host.set_menu_selected(
                    cur.wrapping_add(layout::MENU_COUNT - 1) % layout::MENU_COUNT,
                );
                false
            }
            GameInput::MoveDown => {
                let cur = host.menu_selected();
                host.set_menu_selected((cur + 1) % layout::MENU_COUNT);
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
        chrome::flash_press();
        host.advance(GameInput::Confirm)
    }

    /// 主菜单渲染（v2 概念图 `evidence/ui_concept/index_v2.png` 风格）：共享外壳
    /// `chrome::scene_shell`（羊皮纸底 / 航海网格 / 木纹外框 / 标题条 / bevel 按钮）+
    /// 菜单列表（QB126 icon 列 + 正文）+ QB150 用户版尾行。坐标全读 `scenes.json` 的
    /// `index` 段；旧开发自检状态行（字形/音效/MOD 计数）已替换为 i18n 尾行
    /// （`scene.index.footer`，开发诊断走 `stats` feature 的 eprintln，不进 UI）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let layout = chrome::scene_layout("index");
        chrome::scene_shell(host, "index", "scene.index.title");
        let labels = crate::shell_ui::menu_labels();
        let sel = host.menu_selected();
        let held = host.press_held();
        let flashing = chrome::press_flash_active();
        // QB126：icon 列（64×64 逻辑盒，三原语 rect 组合；scenes.json `index.icon.x`
        // 缺失/非正 = 不画 icon 列，契约 4 数据驱动）
        let icon_x = layout.extra.get("icon").map(|r| r.x).filter(|x| *x > 0);
        for (i, label) in labels.iter().enumerate() {
            let r = layout.list.item_rect(i);
            chrome::menu_item(host, r.x, r.y, r.w, r.h, i == sel, i == sel && (held || flashing));
            // 文字垂直居中：块高 r.h，字形高 = 正文档（TTF 44px / 位图回退 12px，QA009）
            let ty = r.y + (r.h - host.shell().glyph_px()) / 2;
            chrome::draw_body_text(host, label, layout.list.text_x, ty);
            if let Some(ix) = icon_x {
                let iy = r.y + (r.h - 64) / 2;
                chrome::menu_icon(host, i, ix, iy);
            }
        }
        let status = layout.extra.get("status").copied().unwrap_or_default();
        // QB150：尾行 = 用户版信息（i18n `scene.index.footer`），经 `hint_row` 单查收敛 +
        // 提示档（28px）+ 屏心居中（QB123）
        let footer = crate::i18n::sget("scene.index.footer");
        chrome::hint_row(host, footer, status.x + 2, status.y);
    }
}
