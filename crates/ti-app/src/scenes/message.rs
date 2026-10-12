//! 消息/帮助场景（S9 骨架）：翻页判定 + 页面文本绘制。
use ti_core::domain::messages::MsgKind;
use ti_core::rules::messages::page_text;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 消息/帮助场景处理器（`Message`）。
pub struct MessageScene;

impl Scene for MessageScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Message
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::MsgNext => {
                host.msg_next_page();
                false
            }
            GameInput::MsgPrev => {
                host.msg_prev_page();
                false
            }
            GameInput::MsgClose | GameInput::Back => {
                host.close_message();
                true
            }
            _ => false,
        }
    }

    /// 左半屏 = 关闭；右半屏 = 下一页；上滑 = 下一页；下滑 = 上一页
    /// （`docs/modernization.md` §5 翻页表）。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                if sample.x < layout::MSG_SPLIT_X {
                    host.advance(GameInput::MsgClose)
                } else {
                    host.advance(GameInput::MsgNext)
                }
            }
            Gesture::SwipeUp => host.advance(GameInput::MsgNext),
            Gesture::SwipeDown => host.advance(GameInput::MsgPrev),
            _ => false,
        }
    }

    /// 标题 + 当前页文本 + 页码 + 导航提示。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        let kind = host.msg_kind().unwrap_or(MsgKind::Help);
        host.draw_text(i18n::sget(kind.label()), 60, 4);
        // 当前页文本（`rules::messages::page_text` 分页钳位，文本经 `i18n::sget` 查表）
        let page = host.msg_page();
        let key = page_text(kind, page);
        let text = i18n::sget(key.as_str());
        host.draw_text(text, 10, 40);
        let total = host.msg_total();
        let page_str = format!("{}/{}", page + 1, total);
        host.draw_text(&page_str, 60, 200);
        host.draw_text(i18n::sget("scene.message.nav_hint"), 40, 210);
    }
}
