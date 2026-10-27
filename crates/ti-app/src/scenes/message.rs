//! 消息/帮助场景（S9 骨架）：翻页判定 + 页面文本绘制。
use ti_core::domain::messages::MsgKind;
use ti_core::rules::messages::page_text;

use crate::game_app::{GameInput, HitData};
use crate::i18n;
use crate::scenes::chrome;
use crate::scenes::{Scene, SceneHost, SceneKind};

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

    /// S29.2：上滑 = 下一页；下滑 = 上一页（`docs/modernization.md` §5 翻页表）。点按（`key=0`
    /// 无元素）兜底 = 下一页；关闭仍走键盘 [`GameInput::MsgClose`]（左右半屏分区命中待
    /// S29.3/S27.4 补 message 分区 `pick_key` 数据）。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        match hit.swipe {
            1 => host.advance(GameInput::MsgNext),
            2 => host.advance(GameInput::MsgPrev),
            _ => {
                if hit.swipe == 0 {
                    host.advance(GameInput::MsgNext)
                } else {
                    false
                }
            }
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
        // S26.3：标题色 `text_on_wood`（金 0xFE55）压在 `wood_warm` 金条上 = 金上金低对比；
        // 改 `COLOR_TEXT_ON_PARCH`（亮奶油 0xF7BA）压中金条 = 暖调高对比（深棕/蓝 出板或闷，弃用）。
        let tx = t.x - host.shell().text_width_px(title) / 2;
        host.draw_text_colored(title, tx, t.y, chrome::COLOR_TEXT_ON_PARCH);
        let body = layout.extra.get("body").copied().unwrap_or_default();
        let page = host.msg_page();
        let key = page_text(kind, page);
        let text = i18n::sget(key.as_str());
        // QA111：正文多行换行——面板内宽（body.w - `edge_pad` 衬边）按 44px 档逐字 advance 折行，
        // 行高 = `body.line_h`（44px 字形 + 8px 行距），行数上限按面板高截断
        // （溢出行不画，不越面板，reserve = `body.clip_reserve`）。飞轮 R1 ②-07：
        // 几何参数组全走 `scenes.json` `message.body` 数据键（x_pad/edge_pad/line_h/clip_reserve）。
        let sc = &layout.extra_scalar;
        let edge_pad = sc.get("body.edge_pad").copied().unwrap_or(0);
        let x_pad = sc.get("body.x_pad").copied().unwrap_or(0);
        let line_h = sc.get("body.line_h").copied().unwrap_or(0);
        let clip_reserve = sc.get("body.clip_reserve").copied().unwrap_or(0);
        let budget = body.w - edge_pad;
        let mut x_cursor = body.x + x_pad;
        let mut y_cursor = body.y + x_pad;
        let mut line_width = 0i32;
        for ch in text.chars() {
            let adv = host.shell().char_advance_px(ch);
            if line_width + adv > budget {
                x_cursor = body.x + x_pad;
                y_cursor += line_h;
                line_width = 0;
            }
            if y_cursor > body.y + body.h - clip_reserve {
                break; // QA111：越面板行不画
            }
            chrome::draw_body_text(host, &ch.to_string(), x_cursor, y_cursor);
            x_cursor += adv;
            line_width += adv;
        }
        // QA144：页点（`page_dots.size` 方点、间距 = `page_dots.gap`；已翻实心白 /
        // 未翻暗底+框）按 `page` 数据槽中心点（page.x）排布于页码行之上
        // （`page_dots.dy`）；中缝分割线已入 JSON 原语（z=7 竖线）。
        // 飞轮 R1 ②-07：页点几何组走 `page_dots.*` 数据键。
        let total = host.msg_total();
        let dots_n = i32::try_from(total).unwrap_or(0);
        let dot_size = sc.get("page_dots.size").copied().unwrap_or(0);
        let dot_gap = sc.get("page_dots.gap").copied().unwrap_or(0);
        let dot_dy = sc.get("page_dots.dy").copied().unwrap_or(0);
        let dot_inner = sc.get("page_dots.inner").copied().unwrap_or(0);
        let dot_inset = (dot_size - dot_inner) / 2;
        let dots_w = dots_n * dot_gap - (dot_gap - dot_size);
        let dots_x = layout.page.x - dots_w.max(0) / 2;
        let dots_y = layout.page.y - dot_dy;
        for i in 0..dots_n {
            let dx = dots_x + i * dot_gap;
            let is_paged = u32::try_from(i).unwrap_or(0) < page; // i ≥ 0（循环变量），cast 无符号丢失
            if is_paged {
                host.fill_rect(dx, dots_y, dot_size, dot_size, chrome::COLOR_TEXT_ON_PARCH);
            } else {
                host.fill_rect(dx, dots_y, dot_size, dot_size, chrome::COLOR_BEVEL_SHADOW);
                host.fill_rect(
                    dx + dot_inset,
                    dots_y + dot_inset,
                    dot_size - dot_inset * 2,
                    dot_size - dot_inset * 2,
                    chrome::COLOR_MENU_BG,
                );
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
