//! 渲染管线契约（`docs/process/07_render_pipeline.md`）：越界求交裁剪 + z-order 纯函数层。
//!
//! 职责分离：把「越界裁剪」从 `sprite_atlas.rs` 的 blit 内联逻辑提成可单测纯函数，
//! 对齐原版 `sub_5EAA8` 源窗求交为空即不画（`sub_5D9C4` 早退，`evidence/audit_zorder.md` A3）。
//!
//! - [`clip_rect`]：精灵原点 × 尺寸 → 裁剪矩形；越界（原点 ≥ 帧宽/高）返回 `None`（整条跳过）。
//! - [`TRANSPARENT_INDEX`]：PCX 调色板 index 0 = 哨兵透明位（§4 契约，4 处同口径）。

/// 逻辑帧宽（`docs/design/ui_controls.md`，原版场景 176×220）。
pub const FB_W: usize = 176;
/// 逻辑帧高。
pub const FB_H: usize = 220;

/// PCX 调色板 index 0 = 透明哨兵位（§4 契约）。
///
/// 原版 8-bit blit 例程 `sub_5D9C4` 逐像素 `if (idx != 0) write`，index 0 跳过后透出背景。
/// `ui.pak` 94/120 张 PCX 的调色板 0 槽常被填成洋红 (255,0,255)（`evidence/audit_mag_red_cause.md`）。
pub const TRANSPARENT_INDEX: usize = 0;

/// 裁剪矩形（源精灵左上角 + 画入帧缓冲的宽高）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRect {
    /// 帧缓冲内左上角 x（已求交，0 ≤ x < `FB_W`）。
    pub x: usize,
    /// 帧缓冲内左上角 y（已求交，0 ≤ y < `FB_H`）。
    pub y: usize,
    /// 实际写入宽度（已裁剪，> 0）。
    pub w: usize,
    /// 实际写入高度（已裁剪，> 0）。
    pub h: usize,
}

/// 越界求交裁剪（契约 §3，对齐原版 `sub_5EAA8` 求交为空即不画）。
///
/// - 原点越界（`ox >= FB_W || oy >= FB_H`）→ `None`（整条跳过，不写像素、不 `mark_dirty`）。
/// - 原点在屏内 → `Some(ClipRect)`，`w/h` 取「精灵尺寸 × 到帧右缘/下缘剩余」的较小值。
///
/// 纯函数：无副作用、可单测。`blit_sprite` / `blit_sprite_rgba` 开头调用本函数，
/// `None` 直接早退（修掉原 `ox = px_x.min(FB_W-1)` 把越界精灵拉回右缘画 1 列竖条的 bug）。
#[must_use]
pub fn clip_rect(ox: usize, oy: usize, src_w: usize, src_h: usize) -> Option<ClipRect> {
    // 契约 §3：原点 ≥ 帧宽/高 → 原版源窗求交为空 → 整条不画（高置信，audit_zorder.md A3）
    if ox >= FB_W || oy >= FB_H || src_w == 0 || src_h == 0 {
        return None;
    }
    // 原点在屏内：裁剪右缘/下缘超出部分（与原版一致，部分越界裁剪）
    let w = src_w.min(FB_W - ox);
    let h = src_h.min(FB_H - oy);
    Some(ClipRect { x: ox, y: oy, w, h })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_inside_no_clip() {
        let c = clip_rect(6, 12, 60, 58);
        assert_eq!(c, Some(ClipRect { x: 6, y: 12, w: 60, h: 58 }));
    }

    #[test]
    fn offscreen_x_skipped() {
        // x=185（f1=111 越界条，audit_zorder.md B4）：原 ox=min(185,175)=175 拉回右缘，
        // 契约修正后 ox>=FB_W → None 整条跳过
        assert_eq!(clip_rect(185, 22, 105, 11), None);
        assert_eq!(clip_rect(191, 44, 105, 11), None);
        assert_eq!(clip_rect(207, 44, 105, 11), None);
    }

    #[test]
    fn offscreen_y_skipped() {
        assert_eq!(clip_rect(34, 205, 47, 15), Some(ClipRect { x: 34, y: 205, w: 47, h: 15 }));
        // y=230 越界
        assert_eq!(clip_rect(10, 230, 50, 50), None);
    }

    #[test]
    fn partial_clip_right_edge() {
        // x=150 宽 100：右缘 250>176，裁到 26 列
        let c = clip_rect(150, 0, 100, 10);
        assert_eq!(c, Some(ClipRect { x: 150, y: 0, w: 26, h: 10 }));
    }

    #[test]
    fn boundary_x_175_last_col() {
        // x=175 仍是屏内最后一列（部分画 1 列）
        let c = clip_rect(175, 0, 105, 11);
        assert_eq!(c, Some(ClipRect { x: 175, y: 0, w: 1, h: 11 }));
    }

    #[test]
    fn zero_size_skipped() {
        assert_eq!(clip_rect(0, 0, 0, 10), None);
        assert_eq!(clip_rect(0, 0, 10, 0), None);
    }
}
