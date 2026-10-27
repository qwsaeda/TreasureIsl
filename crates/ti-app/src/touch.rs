//! 触屏输入层：**手势识别**（点按 / 四向滑动）+ 屏幕→1080 逻辑反变换。
//!
//! **S29.2（B1）：命中几何已退役** —— 「点哪 = 哪个元素」的判定（徽章 / 列表行 / 槽位带 /
//! 网格格点逆变换等坐标几何一族）原在此 `layout` 模块，与渲染 `item_rect`/`iso_cell` 正逆成对
//! （EXP-067/071）。B1 把命中/几何全移进
//! **Bevy 拾取层**（`ti_engine::pick`：`PickTarget` + 指针事件 → `PickEvent`，`scenes.json`
//! `pick_key`/`pick_index` 数据驱动）；逻辑核心消费归一 `ti_app::game_app::HitData`（`route_hit`）。
//! 本模块只留**手势识别**（[`TouchPad`] + [`Gesture`]），供非 Bevy 壳（Android macroquad，
//! S18.8d 退役）与 headless 手势测试；Bevy 壳走 `poll_hit`（拾取命中，非 `poll_touch`）。
//!
//! 坐标约定：`TouchPad` 产出 **1080×1920 呈现层坐标**（`menu_shell::LOGICAL_W_I32/H_I32`
//! 单一权威呈现坐标系，`08_render_arch.md` 契约 1）；屏幕坐标由调用方用
//! [`Viewport::screen_to_logical`] 反变换后传入（见 [`TouchPad::feed_screen`]）。

use ti_platform::gfx::Viewport;
use ti_platform::settings::{Gesture, GestureTracker};

/// 一次完成的触摸：手势类型 + 释放点的逻辑坐标。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchSample {
    /// 识别出的手势（点按/四向滑动）。
    pub gesture: Gesture,
    /// 释放点逻辑 x（用于命中判定）。
    pub x: f32,
    /// 释放点逻辑 y（用于命中判定）。
    pub y: f32,
}

/// 触屏输入垫：把每帧指针采样转成逻辑坐标手势。
///
/// 后端只需每帧给出「当前指针位置 + 是否按下」，本垫负责反变换、手势识别与
/// 启用开关，产出 [`TouchSample`] 交给 `GameApp::poll_touch`。
#[derive(Debug, Clone)]
pub struct TouchPad {
    /// 手势识别器（滑动阈值 = 20 逻辑像素）。
    tracker: GestureTracker,
    /// 最近一次采样点的逻辑 x。
    x: f32,
    /// 最近一次采样点的逻辑 y。
    y: f32,
    /// 是否启用（设置项可关；关闭时 `feed*` 恒返回 `None`，不产生任何输入）。
    enabled: bool,
}

impl TouchPad {
    /// 新建触屏垫（默认阈值 20 逻辑像素，默认启用）。
    #[must_use]
    pub fn new() -> Self {
        Self { tracker: GestureTracker::default_threshold(), x: 0.0, y: 0.0, enabled: true }
    }

    /// 指定滑动阈值（逻辑像素）新建。
    #[must_use]
    pub fn with_threshold(swipe_threshold: f32) -> Self {
        Self { tracker: GestureTracker::new(swipe_threshold), x: 0.0, y: 0.0, enabled: true }
    }

    /// 启用/禁用触屏（禁用时鼠标点击不产生触摸手势）。
    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
    }

    /// 当前是否启用。
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 最近一次采样点的逻辑坐标。
    #[must_use]
    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    /// 喂一帧采样（**已是逻辑坐标**）；手势完成时返回 [`TouchSample`]。
    ///
    /// `is_down = true` 记录起点并返回 `None`；`is_down = false` 判定手势
    /// （点按/滑动）并返回带释放点坐标的样本。未启用时恒返回 `None`。
    pub fn feed(&mut self, lx: f32, ly: f32, is_down: bool) -> Option<TouchSample> {
        if !self.enabled {
            return None;
        }
        self.x = lx;
        self.y = ly;
        self.tracker.feed(lx, ly, is_down).map(|gesture| TouchSample { gesture, x: lx, y: ly })
    }

    /// 喂一帧采样（**屏幕坐标**）：内部按视口反变换到逻辑坐标（`docs/parity.md` P4-2）。
    ///
    /// 未启用时不做反变换直接返回 `None`（省一次运算）。
    pub fn feed_screen(
        &mut self,
        vp: &Viewport,
        sx: f32,
        sy: f32,
        is_down: bool,
    ) -> Option<TouchSample> {
        if !self.enabled {
            return None;
        }
        let (lx, ly) = vp.screen_to_logical(sx, sy);
        self.feed(lx, ly, is_down)
    }
}

impl Default for TouchPad {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Gesture, TouchPad, TouchSample};
    use ti_platform::gfx::Viewport;

    /// 浮点近似相等（避免 `float_cmp` lint，容差 1e-3）。
    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn tap_recognized_within_threshold() {
        let mut pad = TouchPad::new();
        assert!(pad.feed(50.0, 90.0, true).is_none(), "按下时手势未完成");
        let got = pad.feed(52.0, 92.0, false);
        assert!(
            matches!(got, Some(TouchSample { gesture: Gesture::Tap, x, y })
                if near(x, 52.0) && near(y, 92.0)),
            "小幅移动应识别为点按 (52,92)，实际 {got:?}"
        );
    }

    #[test]
    fn swipe_direction_recognized() {
        let cases = [
            (50.0_f32, 90.0_f32, 50.0_f32, 30.0_f32, Gesture::SwipeUp),
            (50.0, 90.0, 50.0, 160.0, Gesture::SwipeDown),
            (50.0, 90.0, 10.0, 90.0, Gesture::SwipeLeft),
            (50.0, 90.0, 100.0, 90.0, Gesture::SwipeRight),
        ];
        for (sx, sy, ex, ey, want) in cases {
            let mut pad = TouchPad::new();
            let _ = pad.feed(sx, sy, true);
            let got = pad.feed(ex, ey, false);
            assert!(
                matches!(got, Some(TouchSample { gesture, .. }) if gesture == want),
                "({sx},{sy})→({ex},{ey}) 应识别为 {want:?}，实际 {got:?}"
            );
        }
    }

    #[test]
    fn disabled_pad_never_produces_input() {
        let mut pad = TouchPad::new();
        pad.set_enabled(false);
        assert!(!pad.is_enabled());
        assert!(pad.feed(10.0, 10.0, true).is_none());
        assert!(pad.feed(10.0, 10.0, false).is_none(), "禁用后不应有手势");
    }

    #[test]
    fn feed_screen_inverts_viewport() {
        // 视口 2x：屏幕 (1080,1920) → 逻辑 (540,960)
        let vp = Viewport::fit(2160.0, 3840.0);
        let mut pad = TouchPad::new();
        assert!(pad.feed_screen(&vp, 1080.0, 1920.0, true).is_none());
        let got = pad.feed_screen(&vp, 1080.0, 1920.0, false);
        assert!(
            matches!(got, Some(TouchSample { x, y, .. }) if near(x, 540.0) && near(y, 960.0)),
            "屏幕 (1080,1920) 应反变换为逻辑 (540,960)，实际 {got:?}"
        );
        let (px, py) = pad.pos();
        assert!(near(px, 540.0) && near(py, 960.0), "pad.pos() 应记住最近采样点");
    }
}
