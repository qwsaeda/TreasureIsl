//! 消息/帮助场景（S9 骨架）：翻页判定 + 页面文本绘制。
use ti_core::domain::messages::MsgKind;
use ti_core::rules::messages::page_text;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::LOGICAL_W_I32;
use crate::scenes::chrome;
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
                chrome::reset_page_frame();
                false
            }
            GameInput::MsgPrev => {
                host.msg_prev_page();
                chrome::reset_page_frame();
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

    /// 标题 + 当前页文本（QA111 换行）+ 页码（QA124）+ 导航提示（v2 羊皮纸/木纹/金色系）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let layout = chrome::scene_layout("message");
        let kind = host.msg_kind().unwrap_or(MsgKind::Help);
        // 文案 key = `msg.<kind>`（`strings.json` 存中文；直接 `sget(kind.label())`
        // 会落到裸 key，界面显示英文 "help"，见 evidence/ui_concept/review_all.txt）
        chrome::scene_shell(host, "message", &format!("msg.{}", kind.label()));
        let body = layout.extra.get("body").copied().unwrap_or_default();
        chrome::panel(host, body.x, body.y, body.w.max(1), body.h.max(1));
        let page = host.msg_page();
        let key = page_text(kind, page);
        let text = i18n::sget(key.as_str());
        // QA111：正文多行换行——面板内宽（body.w - 8 衬边）按 44px 档逐字 advance 折行，
        // 行高 52px（44px 字形 + 8px 行距），行数上限按面板高截断（溢出行不画，不越面板）。
        let budget = body.w - 8;
        let mut x_cursor = body.x + 4;
        let mut y_cursor = body.y + 4;
        let mut line_width = 0i32;
        for ch in text.chars() {
            let adv = host.shell().char_advance_px(ch);
            if line_width + adv > budget {
                x_cursor = body.x + 4;
                y_cursor += 52;
                line_width = 0;
            }
            if y_cursor > body.y + body.h - 48 {
                break; // QA111：越面板行不画
            }
            chrome::draw_body_text(host, &ch.to_string(), x_cursor, y_cursor);
            x_cursor += adv;
            line_width += adv;
        }
        // QA144：页点（12×12，间距 24；已翻实心白 / 未翻暗底+框）在页码行之上屏心排布；
        // 中缝分割线 = 触摸左右屏分界（`MSG_SPLIT_X` = 屏中线）的视觉锚（2px 竖线，面板内）。
        let total = host.msg_total();
        let dots_n = i32::try_from(total).unwrap_or(0);
        let dots_w = dots_n * 36 - 24;
        let dots_x = (LOGICAL_W_I32 - dots_w.max(0)) / 2;
        let dots_y = layout.page.y - 48;
        for i in 0..dots_n {
            let dx = dots_x + i * 36;
            let is_paged = u32::try_from(i).unwrap_or(0) < page; // i ≥ 0（循环变量），cast 无符号丢失
            if is_paged {
                host.fill_rect(dx, dots_y, 12, 12, chrome::COLOR_TEXT_ON_PARCH);
            } else {
                host.fill_rect(dx, dots_y, 12, 12, chrome::COLOR_BEVEL_SHADOW);
                host.fill_rect(dx + 2, dots_y + 2, 8, 8, chrome::COLOR_MENU_BG);
            }
        }
        host.fill_rect(
            LOGICAL_W_I32 / 2,
            body.y + 8,
            2,
            (body.h - 16).max(0),
            chrome::COLOR_MENU_BORDER,
        );
        let page_str = i18n::sgetf("scene.message.page", &[&(page + 1), &total]);
        let page_pos = layout.page;
        let page_x = chrome::centered_body_x(host, &page_str);
        chrome::draw_body_text(host, &page_str, page_x, page_pos.y);
        chrome::scene_hint(host, "message", "scene.message.nav_hint");
        let factor = chrome::page_brightness_factor();
        if factor < 1.0 {
            host.shell_mut().apply_brightness(factor);
        }
        chrome::tick_page_frame(host.frame_delta_ms());
    }
}
