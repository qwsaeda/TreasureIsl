//! S17.4 动效帧快照断言（`docs/process/10_ui_redesign.md` §7 验收锚点 ②）：
//! 场景切换交叉淡化 `Framebuf::crossfade_from` 在 t = 0 / 0.5 / 1.0 三帧的
//! 像素单调性 —— 中间帧既不等于旧画面、也不等于新画面、且不越界（RGB565 逐通道
//! 单调）。正反向（黑→白 / 白→黑）各验一遍，防淡化方向写反或 t 映射错位。
//!
//! 静帧基线（`render_snapshot` / `forward` / `visual_regression`）不受影响：它们关掉过渡
//! 走稳态，本测试只锁「过渡中间帧」的行为。

use ti_render::framebuffer::Framebuf;

/// 取某 t 下「old 画面渐变到 new 画面」整帧的像素（全帧应均匀，取首像素 + 全帧一致断言）。
fn frame_at(old: u16, new: u16, t: f32) -> u16 {
    let mut new_fb = Framebuf::new();
    new_fb.clear(new);
    let mut old_fb = Framebuf::new();
    old_fb.clear(old);
    let prev = old_fb.as_slice();
    let mut fb = Framebuf::new();
    fb.clear(new);
    fb.crossfade_from(prev, t);
    let slice = fb.as_slice().to_vec();
    // 全帧均匀：所有像素等于首像素（防止局部未混合）。
    let first = slice[0];
    assert!(slice.iter().all(|&p| p == first), "整帧应均匀混合");
    first
}

fn channels(p: u16) -> (u32, u32, u32) {
    let r = u32::from((p >> 11) & 0x1F);
    let g = u32::from((p >> 5) & 0x3F);
    let b = u32::from(p & 0x1F);
    (r, g, b)
}

/// 黑 → 白：逐通道非减，中间帧严格在两端之间。
#[test]
fn crossfade_black_to_white_monotone() {
    let black = 0x0000u16;
    let white = 0xFFFFu16;
    let f0 = frame_at(black, white, 0.0);
    let fm = frame_at(black, white, 0.5);
    let f1 = frame_at(black, white, 1.0);

    // 端点 = 对应画面本身（t=0 全旧，t=1 全新，不越界）。
    assert_eq!(f0, black, "t=0 应 = 全旧画面（黑）");
    assert_eq!(f1, white, "t=1 应 = 全新画面（白）");

    // 中间帧非全黑、非全白（「非全黑非全白」判据）。
    assert_ne!(fm, black, "t=0.5 中间帧不得全黑");
    assert_ne!(fm, white, "t=0.5 中间帧不得全白");

    // 逐通道单调：黑(0) ≤ 中 ≤ 白(max)，且中间严格在内部。
    let (r0, g0, b0) = channels(f0);
    let (rm, gm, bm) = channels(fm);
    let (r1, g1, b1) = channels(f1);
    assert!(r0 < rm && rm < r1, "R 通道须严格单调黑→白: {r0} {rm} {r1}");
    assert!(g0 < gm && gm < g1, "G 通道须严格单调黑→白: {g0} {gm} {g1}");
    assert!(b0 < bm && bm < b1, "B 通道须严格单调黑→白: {b0} {bm} {b1}");
}

/// 白 → 黑：逐通道非增，中间帧严格在两端之间（反向，防 t 映射写反）。
#[test]
fn crossfade_white_to_black_monotone() {
    let white = 0xFFFFu16;
    let black = 0x0000u16;
    let f0 = frame_at(white, black, 0.0);
    let fm = frame_at(white, black, 0.5);
    let f1 = frame_at(white, black, 1.0);

    assert_eq!(f0, white, "t=0 应 = 全旧画面（白）");
    assert_eq!(f1, black, "t=1 应 = 全新画面（黑）");
    assert_ne!(fm, white, "t=0.5 中间帧不得全白");
    assert_ne!(fm, black, "t=0.5 中间帧不得全黑");

    let (r0, g0, b0) = channels(f0);
    let (rm, gm, bm) = channels(fm);
    let (r1, g1, b1) = channels(f1);
    assert!(rm < r0 && r1 < rm, "R 通道须严格单调白→黑: {r0} {rm} {r1}");
    assert!(gm < g0 && g1 < gm, "G 通道须严格单调白→黑: {g0} {gm} {g1}");
    assert!(bm < b0 && b1 < bm, "B 通道须严格单调白→黑: {b0} {bm} {b1}");
}

/// 非对称端点（深色旧 → 亮色新）：中间帧仍严格介于两端，不塌缩到任一端。
#[test]
fn crossfade_asymmetric_endpoints() {
    // 旧 = 深红 (0x60: r=12)，新 = 青 (0x07E0: g=31,b=7) 之类任意非端点值。
    let old = 0x6000u16; // r=12,g=0,b=0
    let new = 0x07E0u16; // r=0,g=31,b=7
    let f0 = frame_at(old, new, 0.0);
    let fm = frame_at(old, new, 0.5);
    let f1 = frame_at(old, new, 1.0);
    assert_eq!(f0, old);
    assert_eq!(f1, new);
    assert_ne!(fm, old, "非对称端点下中间帧不得塌回旧画面");
    assert_ne!(fm, new, "非对称端点下中间帧不得提前到全新画面");
    // 逐通道在 [min,max] 闭区间内（允许贴边界，因混合可能向某端收敛）。
    for (co, cm, cn) in [
        ((old >> 11) & 0x1F, (fm >> 11) & 0x1F, (new >> 11) & 0x1F),
        ((old >> 5) & 0x3F, (fm >> 5) & 0x3F, (new >> 5) & 0x3F),
        (old & 0x1F, fm & 0x1F, new & 0x1F),
    ] {
        let lo = u16::min(co, cn);
        let hi = u16::max(co, cn);
        assert!(cm >= lo && cm <= hi, "通道须落在两端之间 {co} {cm} {cn}");
    }
}
