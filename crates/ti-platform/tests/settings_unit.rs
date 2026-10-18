//! S4.1 设置单测：缩放模式、横竖屏、虚拟方向键、手势、按键自定义。

use ti_platform::{
    Gesture, GestureTracker, KeyBindings, Orientation, ScaleMode, Settings, Viewport, VirtualKeyPad,
};

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

// ── 缩放模式 ──────────────────────────────────────────────────────────────────

#[test]
fn coordinate_roundtrip_smooth_scale() {
    let s = Settings::default_settings();
    assert_eq!(s.scale_mode, ScaleMode::Smooth);
    // 呈现坐标系 1080×1920（契约 1）：竖屏逻辑 1080×1920 → 屏幕 2160×3840 正好 2×
    let vp = s.viewport(2160.0, 3840.0);
    assert!(approx_eq(vp.scale(), 2.0));
    for (lx, ly) in [(0.0, 0.0), (540.0, 960.0), (1080.0, 1920.0)] {
        let (sx, sy) = vp.logical_to_screen(lx, ly);
        let (rx, ry) = vp.screen_to_logical(sx, sy);
        assert!(approx_eq(rx, lx), "x: {rx} != {lx}");
        assert!(approx_eq(ry, ly), "y: {ry} != {ly}");
    }
}

#[test]
fn coordinate_roundtrip_integer_scale_mode() {
    let s = Settings { scale_mode: ScaleMode::Integer, ..Settings::default_settings() };
    // 竖屏逻辑 1080×1920（契约 1）：
    // 2160×3840 → raw=min(2160/1080,3840/1920)=2.0 → integer=2.0
    let vp = s.viewport(2160.0, 3840.0);
    assert!(approx_eq(vp.scale(), 2.0));
    // 1620×2880 → raw=min(1.5,1.5)=1.5 → integer=1.0
    let vp = s.viewport(1620.0, 2880.0);
    assert!(approx_eq(vp.scale(), 1.0));
    // 2700×4800 → raw=min(2.5,2.5)=2.5 → integer=2.0
    let vp = s.viewport(2700.0, 4800.0);
    assert!(approx_eq(vp.scale(), 2.0));
    // 540×960 → raw=min(0.5,0.5)=0.5 → integer 最小取 1.0
    let vp = s.viewport(540.0, 960.0);
    assert!(approx_eq(vp.scale(), 1.0));
}

#[test]
fn scale_mode_smooth_keeps_fractional() {
    let s = Settings::default_settings();
    assert_eq!(s.scale_mode, ScaleMode::Smooth);
    // raw=0.7 在 Smooth 模式下保持 0.7
    assert!(approx_eq(s.effective_scale(0.7), 0.7));
}

#[test]
fn scale_mode_integer_floors_to_whole() {
    let s = Settings { scale_mode: ScaleMode::Integer, ..Settings::default_settings() };
    assert!(approx_eq(s.effective_scale(2.7), 2.0));
    assert!(approx_eq(s.effective_scale(0.5), 1.0)); // min 1
    assert!(approx_eq(s.effective_scale(1.0), 1.0));
}

// ── 横竖屏 ────────────────────────────────────────────────────────────────────

#[test]
fn orientation_portrait_logical_size() {
    assert_eq!(Orientation::Portrait.logical_size(), (1080.0, 1920.0));
}

#[test]
fn orientation_landscape_logical_size() {
    assert_eq!(Orientation::Landscape.logical_size(), (1920.0, 1080.0));
}

#[test]
fn orientation_landscape_viewport_fits_wide_screen() {
    let s = Settings { orientation: Orientation::Landscape, ..Settings::default_settings() };
    // 横屏逻辑 1920×1080（契约 1）：800×400 横屏 raw = min(800/1920, 400/1080)
    let vp = s.viewport(800.0, 400.0);
    let expected = (800.0_f32 / 1920.0).min(400.0 / 1080.0);
    assert!(approx_eq(vp.scale(), expected));
}

// ── 虚拟方向键 ────────────────────────────────────────────────────────────────

#[test]
fn virtual_keypad_default_layout_hit_up() {
    let kp = VirtualKeyPad::default_layout();
    assert_eq!(kp.hit(88.0, 164.0), Some("up")); // up rect center
}

#[test]
fn virtual_keypad_default_layout_hit_down() {
    let kp = VirtualKeyPad::default_layout();
    assert_eq!(kp.hit(88.0, 204.0), Some("down")); // down rect center
}

#[test]
fn virtual_keypad_default_layout_hit_left() {
    let kp = VirtualKeyPad::default_layout();
    assert_eq!(kp.hit(40.0, 184.0), Some("left")); // left rect center
}

#[test]
fn virtual_keypad_default_layout_hit_right() {
    let kp = VirtualKeyPad::default_layout();
    assert_eq!(kp.hit(136.0, 184.0), Some("right")); // right rect center
}

#[test]
fn virtual_keypad_miss_returns_none() {
    let kp = VirtualKeyPad::default_layout();
    assert_eq!(kp.hit(10.0, 10.0), None); // top-left, outside all rects
    assert_eq!(kp.hit(88.0, 100.0), None); // center of screen, between pads
}

// ── 手势识别 ──────────────────────────────────────────────────────────────────

#[test]
fn gesture_tap_small_movement() {
    let mut t = GestureTracker::new(20.0);
    assert_eq!(t.feed(50.0, 50.0, true), None);
    assert_eq!(t.feed(53.0, 52.0, false), Some(Gesture::Tap));
}

#[test]
fn gesture_swipe_right() {
    let mut t = GestureTracker::new(20.0);
    assert_eq!(t.feed(30.0, 100.0, true), None);
    assert_eq!(t.feed(80.0, 102.0, false), Some(Gesture::SwipeRight));
}

#[test]
fn gesture_swipe_left() {
    let mut t = GestureTracker::new(20.0);
    assert_eq!(t.feed(120.0, 100.0, true), None);
    assert_eq!(t.feed(70.0, 105.0, false), Some(Gesture::SwipeLeft));
}

#[test]
fn gesture_swipe_up() {
    let mut t = GestureTracker::new(20.0);
    assert_eq!(t.feed(88.0, 160.0, true), None);
    assert_eq!(t.feed(88.0, 80.0, false), Some(Gesture::SwipeUp));
}

#[test]
fn gesture_swipe_down() {
    let mut t = GestureTracker::new(20.0);
    assert_eq!(t.feed(88.0, 60.0, true), None);
    assert_eq!(t.feed(88.0, 140.0, false), Some(Gesture::SwipeDown));
}

#[test]
fn gesture_tracker_no_down_returns_none() {
    let mut t = GestureTracker::new(20.0);
    assert_eq!(t.feed(50.0, 50.0, false), None); // no down yet
}

// ── 按键自定义 ────────────────────────────────────────────────────────────────

#[test]
fn key_bindings_defaults_are_set() {
    let kb = KeyBindings::defaults();
    assert!(!kb.is_empty());
    assert!(kb.get("up").is_some());
    assert!(kb.get("down").is_some());
    assert!(kb.get("left").is_some());
    assert!(kb.get("right").is_some());
    assert!(kb.get("confirm").is_some());
    assert!(kb.get("cancel").is_some());
}

#[test]
fn key_bindings_custom_override() {
    use ti_platform::KeyCode;
    let mut kb = KeyBindings::defaults();
    kb.set("up", KeyCode(0x57)); // W key
    assert_eq!(kb.get("up"), Some(KeyCode(0x57)));
    // 原始 down 未变
    assert_eq!(kb.get("down"), Some(KeyCode(0x28)));
}

#[test]
fn key_bindings_unknown_action_returns_none() {
    let kb = KeyBindings::defaults();
    assert_eq!(kb.get("fly"), None);
}

/// §5.6.6：`From<FilterMode> for miniquad::FilterMode` 零损失映射。
/// `to_miniquad` 薄封装与 `From` 必须逐位一致（宏展开后等价，防薄封装漂移）。
#[test]
fn filter_mode_from_matches_thin_wrapper() {
    use ti_platform::settings::FilterMode;
    for fm in [FilterMode::Nearest, FilterMode::Linear] {
        let via_from: macroquad::miniquad::FilterMode = macroquad::miniquad::FilterMode::from(fm);
        assert_eq!(via_from, fm.to_miniquad(), "From 与薄封装 to_miniquad 应一致：{fm:?}");
    }
}

// ── Settings 综合 ────────────────────────────────────────────────────────────

#[test]
fn settings_default_values() {
    let s = Settings::default_settings();
    assert_eq!(s.scale_mode, ScaleMode::Smooth);
    assert_eq!(s.orientation, Orientation::Portrait);
    assert!(s.virtual_keypad);
    assert!(!s.key_bindings.is_empty());
}

#[test]
fn settings_viewport_coordinate_roundtrip_landscape() {
    let s = Settings { orientation: Orientation::Landscape, ..Settings::default_settings() };
    let vp = s.viewport(800.0, 400.0);
    // 横屏逻辑 1920×1080（契约 1）：取 1080 系逻辑点做往返
    for (lx, ly) in [(0.0, 0.0), (960.0, 540.0), (1920.0, 1080.0)] {
        let (sx, sy) = vp.logical_to_screen(lx, ly);
        let (rx, ry) = vp.screen_to_logical(sx, sy);
        assert!(approx_eq(rx, lx), "landscape x: {rx} != {lx}");
        assert!(approx_eq(ry, ly), "landscape y: {ry} != {ly}");
    }
}

#[test]
fn settings_viewport_integer_mode_roundtrip() {
    let s = Settings { scale_mode: ScaleMode::Integer, ..Settings::default_settings() };
    // 竖屏逻辑 1080×1920：2160×3840 → raw=2.0 → integer=2.0
    let vp = s.viewport(2160.0, 3840.0);
    assert!(approx_eq(vp.scale(), 2.0));
    let (sx, sy) = vp.logical_to_screen(540.0, 960.0);
    let (rx, ry) = vp.screen_to_logical(sx, sy);
    assert!(approx_eq(rx, 540.0));
    assert!(approx_eq(ry, 960.0));
}

// ── P4-2 核心验收（与 parity.md 对应）────────────────────────────────────────

#[test]
fn coordinate_roundtrip_p42_full_sweep() {
    // 整数/非整数缩放 + 任意窗口尺寸 → 命中判定不偏移（呈现坐标系 1080×1920，契约 1）
    let window_sizes: [(f32, f32); 8] = [
        (1080.0, 1920.0),
        (2160.0, 3840.0),
        (3240.0, 5760.0),
        (540.0, 960.0),
        (1620.0, 2880.0),
        (1920.0, 1080.0),
        (2700.0, 4800.0),
        (300.0, 500.0),
    ];
    let logical_points: [(f32, f32); 5] =
        [(0.0, 0.0), (540.0, 960.0), (1080.0, 1920.0), (505.0, 755.0), (1.0, 1.0)];
    for (w, h) in window_sizes {
        for mode in [ScaleMode::Smooth, ScaleMode::Integer] {
            let s = Settings { scale_mode: mode, ..Settings::default_settings() };
            let vp = s.viewport(w, h);
            assert!(vp.scale() > 0.0, "scale must be positive for {w}×{h} {mode:?}");
            for (lx, ly) in logical_points {
                let (sx, sy) = vp.logical_to_screen(lx, ly);
                let (rx, ry) = vp.screen_to_logical(sx, sy);
                assert!(
                    approx_eq(rx, lx),
                    "P4-2 x drift: {rx} != {lx} @ {w}×{h} {mode:?} scale={}",
                    vp.scale()
                );
                assert!(
                    approx_eq(ry, ly),
                    "P4-2 y drift: {ry} != {ly} @ {w}×{h} {mode:?} scale={}",
                    vp.scale()
                );
            }
        }
    }
}

#[test]
fn viewport_from_scale_matches_fit_for_smooth() {
    // Viewport::from_scale(scale, ox, oy, lw, lh) 与 Viewport::fit（用 1080×1920 逻辑）结果一致
    let screen_w = 2160.0_f32;
    let screen_h = 3840.0_f32;
    let vp1 = Viewport::fit(screen_w, screen_h);
    let raw = vp1.scale();
    let vp2 = Viewport::from_scale(
        raw,
        (screen_w - 1080.0 * raw) * 0.5,
        (screen_h - 1920.0 * raw) * 0.5,
        1080.0,
        1920.0,
    );
    for (lx, ly) in [(0.0, 0.0), (540.0, 960.0), (1080.0, 1920.0)] {
        let (x1, y1) = vp1.logical_to_screen(lx, ly);
        let (x2, y2) = vp2.logical_to_screen(lx, ly);
        assert!(approx_eq(x1, x2), "x1: {x1} vs {x2}");
        assert!(approx_eq(y1, y2), "y1: {y1} vs {y2}");
    }
}
