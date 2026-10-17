//! 输入处理：屏幕坐标→逻辑坐标转换 + 虚拟按键命中判定。
//!
//! 原版触摸事件直接在 176×220 逻辑坐标系处理；本模块把后端送来的
//! 屏幕坐标事件经 [`Viewport`] 反变换为逻辑坐标（P4-2），
//! 并提供触摸屏虚拟按键的命中判定。

use crate::backend::{Event, Input, Rect};
use crate::gfx::Viewport;

/// 触摸屏上的虚拟按键（逻辑坐标矩形 + 名称）。
pub struct VirtualButton {
    /// 按键名（如 `"ok"`、`"cancel"`）。
    pub name: &'static str,
    /// 命中区域（176×220 逻辑坐标）。
    pub rect: Rect,
}

impl VirtualButton {
    /// 判定逻辑坐标点是否落在本按键区域内。
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> bool {
        let rx = self.rect.x as f32;
        let ry = self.rect.y as f32;
        let rw = self.rect.w as f32;
        let rh = self.rect.h as f32;
        x >= rx && x < rx + rw && y >= ry && y < ry + rh
    }
}

/// 把一个屏幕坐标事件转换为逻辑坐标事件（坐标经 [`Viewport`] 反变换）。
///
/// 按键事件（`KeyDown`/`KeyUp`/`Quit`）原样透传。
#[must_use]
pub fn translate_event(screen_event: Event, vp: &Viewport) -> Event {
    match screen_event {
        Event::PointerDown { x, y } => {
            let (lx, ly) = vp.screen_to_logical(x, y);
            Event::PointerDown { x: lx, y: ly }
        }
        Event::PointerUp { x, y } => {
            let (lx, ly) = vp.screen_to_logical(x, y);
            Event::PointerUp { x: lx, y: ly }
        }
        Event::PointerMove { x, y } => {
            let (lx, ly) = vp.screen_to_logical(x, y);
            Event::PointerMove { x: lx, y: ly }
        }
        other => other,
    }
}

/// 测试用输入后端：按预设序列喂事件，`poll` 后清空。
pub struct HeadlessInput {
    events: Vec<Event>,
}

impl HeadlessInput {
    /// 新建空事件队列。
    #[must_use]
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// 追加一个事件到队列尾部。
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }
}

impl Default for HeadlessInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for HeadlessInput {
    fn poll(&mut self, out: &mut Vec<Event>) {
        out.append(&mut self.events);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Rect;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn translate_screen_pointer_down_to_logical() {
        // 呈现坐标系 1080×1920：2× 整数缩放下屏幕中心 (1080,1920) ↔ 逻辑中心 (540,960)
        let vp = Viewport::fit(2160.0, 3840.0);
        let screen_ev = Event::PointerDown { x: 1080.0, y: 1920.0 };
        let logical_ev = translate_event(screen_ev, &vp);
        assert!(matches!(logical_ev, Event::PointerDown { .. }));
        if let Event::PointerDown { x, y } = logical_ev {
            assert!(approx_eq(x, 540.0), "x={x}");
            assert!(approx_eq(y, 960.0), "y={y}");
        }
    }

    #[test]
    fn translate_roundtrips_through_viewport() {
        let vp = Viewport::fit(1440.0, 2560.0);
        for (sx, sy) in [(400.0, 640.0), (725.5, 1105.5)] {
            let logical_ev = translate_event(Event::PointerDown { x: sx, y: sy }, &vp);
            assert!(matches!(logical_ev, Event::PointerDown { .. }));
            if let Event::PointerDown { x: lx, y: ly } = logical_ev {
                let (rx, ry) = vp.logical_to_screen(lx, ly);
                assert!(approx_eq(rx, sx), "rx={rx} sx={sx}");
                assert!(approx_eq(ry, sy), "ry={ry} sy={sy}");
            }
        }
    }

    #[test]
    fn key_events_pass_through_unchanged() {
        let vp = Viewport::fit(352.0, 440.0);
        let ev = Event::KeyDown(crate::backend::KeyCode(0x26));
        let translated = translate_event(ev, &vp);
        assert!(matches!(translated, Event::KeyDown(_)));
        if let Event::KeyDown(k) = translated {
            assert_eq!(k, crate::backend::KeyCode(0x26));
        }
    }

    #[test]
    fn virtual_button_hit_test() {
        let btn = VirtualButton { name: "ok", rect: Rect { x: 60, y: 180, w: 56, h: 30 } };
        assert!(btn.hit(88.0, 195.0), "center should hit");
        assert!(!btn.hit(10.0, 10.0), "far outside should miss");
        assert!(!btn.hit(60.0, 210.0), "just below bottom edge should miss");
        assert!(btn.hit(60.0, 180.0), "top-left corner should hit");
    }

    #[test]
    fn headless_input_poll_drains_queue() {
        let mut input = HeadlessInput::new();
        input.push(Event::Quit);
        input.push(Event::PointerDown { x: 1.0, y: 2.0 });
        let mut out = Vec::new();
        input.poll(&mut out);
        assert_eq!(out.len(), 2);
        let mut out2 = Vec::new();
        input.poll(&mut out2);
        assert!(out2.is_empty(), "second poll must be empty");
    }
}
