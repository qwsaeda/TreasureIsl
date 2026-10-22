//! 消息/帮助场景（S9 骨架）：翻页判定 + 页面文本绘制。
use ti_core::domain::messages::MsgKind;
use ti_core::rules::messages::page_text;

use crate::game_app::GameInput;
use crate::i18n;
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

    /// S18.7d 声明式原语化：版面（夜空渐变 / 左侧立绘面板 / 右正文面板 / 页码带 / 海平面 /
    /// 木框 / 提示行）= `scenes.json` `message.primitives`（z 序 = 绘制序，`render_primitives`
    /// 统一 dispatch）；运行时内容 = 动态标题（msg 类型 i18n）/ 正文折行（QA111）/
    /// 页点页码（QA144），全部按数据槽（`title`/`body`/`page`）落位，0 版面坐标代码。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let layout = chrome::scene_layout("message");
        chrome::render_primitives(host, "message", -1);
        // S20.2 v4 面板语言：消息卡片 = 深棕木底（scenes.json 原语 `panel_wood`）+ 黄铜细边。
        if let Some(panel) = layout.extra.get("panel") {
            chrome::wood_panel(host, panel.x, panel.y, panel.w, panel.h);
        }
        let kind = host.msg_kind().unwrap_or(MsgKind::Help);
        // 标题（动态 i18n `msg.<kind>`；数据槽 `title`：x = 面板中心点，y = 顶边基线）
        let key = format!("msg.{}", kind.label());
        let title = crate::i18n::sget(&key);
        let t = layout.title;
        let tx = t.x - host.shell().text_width_px(title) / 2;
        host.draw_text_colored(title, tx, t.y, chrome::COLOR_TEXT_ON_WOOD);
        let body = layout.extra.get("body").copied().unwrap_or_default();
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
        // QA144：页点（12×12，间距 24；已翻实心白 / 未翻暗底+框）按 `page` 数据槽
        // 中心点（page.x）排布于页码行之上；中缝分割线已入 JSON 原语（z=7 竖线）。
        let total = host.msg_total();
        let dots_n = i32::try_from(total).unwrap_or(0);
        let dots_w = dots_n * 36 - 24;
        let dots_x = layout.page.x - dots_w.max(0) / 2;
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
        let page_str = i18n::sgetf("scene.message.page", &[&(page + 1), &total]);
        let page_pos = layout.page;
        let page_x = page_pos.x - host.shell().text_width_px(&page_str) / 2;
        chrome::draw_body_text(host, &page_str, page_x, page_pos.y);
        // 提示行 = JSON 原语（tier hint）；页面翻转亮度帧推进（运行时动效，保留代码）。
        let factor = chrome::page_brightness_factor();
        if factor < 1.0 {
            host.shell_mut().apply_brightness(factor);
        }
        chrome::tick_page_frame(host.frame_delta_ms());
    }
}
