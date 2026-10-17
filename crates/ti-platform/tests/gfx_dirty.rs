//! S4.3 脏行带 / 呈现计数单测（置于集成测试目录，豁免门禁的 unwrap/expect 扫描）。
//!
//! `FramePresenter` 的脏帧上传路径需要 GPU 上下文（macroquad），
//! 故本文件只测**无 GPU 路径**：脏行标记逻辑（纯数据结构）与
//! 跳帧计数（`present` 在 `!has_dirty` 时直接累加 skipped 并返回）。

use ti_platform::FB_PIXELS;
use ti_platform::gfx::{FramePresenter, Framebuf, Viewport};

/// 1920 行 × 1080 列，取中间一行做局部置脏（呈现坐标系 1080×1920，契约 1）。
const MIDDLE_ROW: i32 = 960;

#[test]
fn new_framebuf_is_fully_dirty() {
    let fb = Framebuf::new();
    assert!(fb.has_dirty(), "新帧缓冲应全脏（保证首次 present 一定上传）");
    let (f, l) = fb.dirty_row_range();
    assert_eq!((f, l), (0, 1919), "全帧脏行带应覆盖 0..=1919");
}

#[test]
fn clear_marks_full_frame_dirty() {
    let mut fb = Framebuf::new();
    fb.reset_dirty();
    assert!(!fb.has_dirty(), "reset 后应无脏行");
    fb.clear(0xF800);
    assert!(fb.has_dirty(), "clear 应置脏全帧");
    assert_eq!(fb.as_slice(), &vec![0xF800u16; FB_PIXELS][..], "clear 应写满全部像素");
}

#[test]
fn mark_dirty_row_band_semantics() {
    let mut fb = Framebuf::new();
    fb.reset_dirty();
    // 只脏中间一行
    fb.mark_dirty(0, MIDDLE_ROW, 1080, 1);
    assert!(fb.has_dirty());
    let (f, l) = fb.dirty_row_range();
    assert_eq!(f, MIDDLE_ROW, "首脏行应为 960");
    assert_eq!(l, MIDDLE_ROW, "末脏行应为 960");
    // 再脏另一行，band 扩展
    fb.mark_dirty(0, 1500, 1080, 1);
    let (f, l) = fb.dirty_row_range();
    assert_eq!(f, MIDDLE_ROW);
    assert_eq!(l, 1500);
}

#[test]
fn mark_dirty_clips_out_of_bounds() {
    let mut fb = Framebuf::new();
    fb.reset_dirty();
    // 完全在屏外 → no-op（1080×1920 呈现区，y=5000 出界）
    fb.mark_dirty(0, 5000, 100, 50);
    assert!(!fb.has_dirty(), "完全屏外矩形不应置脏");
    // 负坐标 → 从 0 起裁
    fb.mark_dirty(-10, -10, 100, 100);
    assert!(fb.has_dirty());
    let (f, l) = fb.dirty_row_range();
    assert_eq!(f, 0, "负 y 应裁到 0");
    assert!(l < 100, "裁切后末行 < 100");
    // 超出底边 → 裁到 1919
    fb.reset_dirty();
    fb.mark_dirty(0, 1900, 1080, 100);
    let (_, l) = fb.dirty_row_range();
    assert_eq!(l, 1919, "超出底边应裁到 1919");
}

#[test]
fn reset_dirty_clears_all_rows() {
    let mut fb = Framebuf::new();
    fb.clear(0xFFFF);
    assert!(fb.has_dirty());
    fb.reset_dirty();
    assert!(!fb.has_dirty(), "reset 后应无脏行");
    let (f, l) = fb.dirty_row_range();
    assert_eq!((f, l), (0, 0), "无脏行时 dirty_row_range 返回 (0,0)");
}

#[test]
fn presenter_counts_skipped_frames_without_gpu() {
    let mut fb = Framebuf::new();
    fb.reset_dirty(); // 全干净 → present 走跳帧路径（不调 GPU）
    let mut presenter = FramePresenter::new();
    let vp = Viewport::fit(2160.0, 3840.0);
    presenter.present(&mut fb, &vp);
    assert_eq!(presenter.skipped(), 1, "无脏行帧应计入 skipped");
    assert_eq!(presenter.uploads(), 0, "无脏行帧不应触发上传");
    // 再来一次仍跳
    presenter.present(&mut fb, &vp);
    assert_eq!(presenter.skipped(), 2);
    assert_eq!(presenter.uploads(), 0);
}

#[test]
fn fill_rect_like_write_marks_dirty() {
    let mut fb = Framebuf::new();
    fb.reset_dirty();
    let stride = FB_PIXELS / 1920;
    // 模拟 chrome::title_bar 的 fill_rect(12, 12, W-24, 90)：写像素 + mark_dirty
    let px = fb.as_mut_slice();
    for row in 0..40i32 {
        for col in 0..1080i32 {
            let idx =
                usize::try_from(row).unwrap_or(0) * stride + usize::try_from(col).unwrap_or(0);
            px[idx] = 0x01E0;
        }
    }
    fb.mark_dirty(12, 12, 1080, 40);
    assert!(fb.has_dirty());
    let (f, l) = fb.dirty_row_range();
    assert_eq!(f, 12);
    assert_eq!(l, 51, "40 行 fill 的末脏行应为 12+40-1=51");
}
