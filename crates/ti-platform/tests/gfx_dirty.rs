//! S4.3 脏行带 / 呈现计数单测（置于集成测试目录，豁免门禁的 unwrap/expect 扫描）。
//!
//! S15.10：脏行跟踪从**帧缓冲**迁到 [`FramePresenter`]（帧缓冲只管像素，在渲染框架
//! crate `ti_render::framebuffer`；脏行带是**上传带宽优化**，属呈现层职责）。
//! 本文件因此改测 presenter 的脏行状态机；`present` 在 `!has_dirty` 时累加 skipped
//! 并返回（无 GPU 路径）。
//!
//! `present` 的脏帧上传路径需要 GPU 上下文（macroquad），故只测**无 GPU 路径**。

use ti_platform::FB_PIXELS;
use ti_platform::gfx::{FramePresenter, Viewport};
use ti_render::framebuffer::Framebuf;

/// 1920 行 × 1080 列，取中间一行做局部置脏（呈现坐标系 1080×1920，契约 1）。
const MIDDLE_ROW: i32 = 960;

#[test]
fn new_presenter_is_fully_dirty() {
    let p = FramePresenter::new();
    assert!(p.has_dirty(), "新呈现器应全脏（保证首次 present 一定上传）");
    let (f, l) = p.dirty_row_range();
    assert_eq!((f, l), (0, 1919), "全帧脏行带应覆盖 0..=1919");
}

#[test]
fn clear_writes_pixels_but_presenter_owns_dirty() {
    // S15.10：帧缓冲 `clear` 只写像素，不再置脏（脏行在 presenter）。
    let mut fb = Framebuf::new();
    let mut p = FramePresenter::new();
    p.reset_dirty();
    assert!(!p.has_dirty(), "reset 后应无脏行");
    fb.clear(0xF800);
    assert_eq!(fb.as_slice(), &vec![0xF800u16; FB_PIXELS][..], "clear 应写满全部像素");
    p.mark_all_dirty(); // 调用方负责标脏
    assert!(p.has_dirty(), "整帧像素操作后标全帧脏应生效");
}

#[test]
fn mark_dirty_row_band_semantics() {
    let mut p = FramePresenter::new();
    p.reset_dirty();
    // 只脏中间一行
    p.mark_dirty(0, MIDDLE_ROW, 1080, 1);
    assert!(p.has_dirty());
    let (f, l) = p.dirty_row_range();
    assert_eq!(f, MIDDLE_ROW, "首脏行应为 960");
    assert_eq!(l, MIDDLE_ROW, "末脏行应为 960");
    // 再脏另一行，band 扩展
    p.mark_dirty(0, 1500, 1080, 1);
    let (f, l) = p.dirty_row_range();
    assert_eq!(f, MIDDLE_ROW);
    assert_eq!(l, 1500);
}

#[test]
fn mark_dirty_clips_out_of_bounds() {
    let mut p = FramePresenter::new();
    p.reset_dirty();
    // 完全在屏外 → no-op（1080×1920 呈现区，y=5000 出界）
    p.mark_dirty(0, 5000, 100, 50);
    assert!(!p.has_dirty(), "完全屏外矩形不应置脏");
    // 负坐标 → 从 0 起裁
    p.mark_dirty(-10, -10, 100, 100);
    assert!(p.has_dirty());
    let (f, l) = p.dirty_row_range();
    assert_eq!(f, 0, "负 y 应裁到 0");
    assert!(l < 100, "裁切后末行 < 100");
    // 超出底边 → 裁到 1919
    p.reset_dirty();
    p.mark_dirty(0, 1900, 1080, 100);
    let (_, l) = p.dirty_row_range();
    assert_eq!(l, 1919, "超出底边应裁到 1919");
}

#[test]
fn reset_dirty_clears_all_rows() {
    let mut fb = Framebuf::new();
    fb.clear(0xFFFF);
    let mut p = FramePresenter::new();
    p.mark_all_dirty();
    assert!(p.has_dirty());
    p.reset_dirty();
    assert!(!p.has_dirty(), "reset 后应无脏行");
    let (f, l) = p.dirty_row_range();
    assert_eq!((f, l), (0, 0), "无脏行时 dirty_row_range 返回 (0,0)");
}

#[test]
fn presenter_counts_skipped_frames_without_gpu() {
    let fb = Framebuf::new();
    let mut presenter = FramePresenter::new();
    presenter.reset_dirty(); // 全干净 → present 走跳帧路径（不调 GPU）
    let vp = Viewport::fit(2160.0, 3840.0);
    presenter.present(&fb, &vp);
    assert_eq!(presenter.skipped(), 1, "无脏行帧应计入 skipped");
    assert_eq!(presenter.uploads(), 0, "无脏行帧不应触发上传");
    // 再来一次仍跳
    presenter.present(&fb, &vp);
    assert_eq!(presenter.skipped(), 2);
    assert_eq!(presenter.uploads(), 0);
}

#[test]
fn fill_rect_marks_dirty_through_presenter() {
    // 模拟 chrome::title_bar 的 fill_rect(12, 12, W-24, 90)：渲染框架帧缓冲写像素 + presenter 标脏
    let mut fb = Framebuf::new();
    let mut p = FramePresenter::new();
    p.reset_dirty();
    fb.fill_rect(12, 12, 1080, 40, 0x01E0);
    p.mark_dirty(12, 12, 1080, 40);
    assert!(p.has_dirty());
    let (f, l) = p.dirty_row_range();
    assert_eq!(f, 12);
    assert_eq!(l, 51, "40 行 fill 的末脏行应为 12+40-1=51");
    // 像素确由渲染框架写入（单源验证：不再经 ti-app 的手写循环）
    let stride = FB_PIXELS / 1920;
    assert_eq!(fb.as_slice()[12 * stride + 12], 0x01E0, "fill_rect 应写入 (12,12) 像素");
    assert_eq!(fb.as_slice()[12 * stride + 11], 0, "fill_rect 左边界外不应写入");
}
