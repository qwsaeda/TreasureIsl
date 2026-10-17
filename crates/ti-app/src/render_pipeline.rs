//! 渲染管线契约（`docs/process/08_render_arch.md` 契约 2/3）：越界求交裁剪 + z-order 纯函数层。
//!
//! 职责分离：把「越界裁剪」从像素写帧逻辑提成可单测纯函数，
//! 对齐原版 `sub_5EAA8` 源窗求交为空即不画（`sub_5D9C4` 早退，`evidence/audit_zorder.md` A3）。
//!
//! - [`clip_rect`]：精灵原点 × 尺寸 → 裁剪矩形；越界（原点 ≥ 帧宽/高）返回 `None`（整条跳过）。
//! - [`TRANSPARENT_INDEX`]：PCX 调色板 index 0 = 哨兵透明位（诊断解码用）。
//!
//! **S15.4（架构重构）**：legacy 176×220 `.Map` 像素写帧层已整体移除（`blit_sprite` /
//! `terrain_atlas::blit_*` 逐像素写帧删除），`legacy_scaled` / `scaled_src` 等 176→1080
//! 等比坐标变换随之删除。运行时像素仅由 3 代码绘制原语产生（契约 2）；本文件只保留
//! 纯几何裁剪与诊断解码所需的常量。

/// 逻辑帧宽（现代 UI 1080×1920）。
pub const FB_W: usize = 1080;
/// 逻辑帧高。
pub const FB_H: usize = 1920;

/// PCX 调色板 index 0 = 透明哨兵位（§4 契约）。
///
/// 原版 8-bit blit 例程 `sub_5D9C4` 逐像素 `if (idx != 0) write`，index 0 跳过后透出背景。
/// `ui.pak` 94/120 张 PCX 的调色板 0 槽常被填成洋红 (255,0,255)（`evidence/audit_mag_red_cause.md`）。
/// S15.4 起仅诊断解码（`SpriteAtlas::sprite` / 独立基线）使用本常量，运行时不再有像素写帧。
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
/// - 原点越界（`ox >= FB_W || oy >= FB_H`）→ `None`（整条跳过）。
/// - 原点在屏内 → `Some(ClipRect)`，`w/h` 取「精灵尺寸 × 到帧右缘/下缘剩余」的较小值。
///
/// 纯函数：无副作用、可单测。S15.4 后仅诊断/基线工具使用（运行时已无逐像素写帧）。
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
        // ox=1080（= FB_W）：原点越界 → None 整条跳过
        assert_eq!(clip_rect(1080, 22, 105, 11), None);
        assert_eq!(clip_rect(1100, 44, 105, 11), None);
        assert_eq!(clip_rect(1150, 44, 105, 11), None);
    }

    #[test]
    fn offscreen_y_skipped() {
        // oy=1920（= FB_H）越界 → None
        assert_eq!(clip_rect(10, 1920, 50, 50), None);
        assert_eq!(clip_rect(10, 2000, 50, 50), None);
    }

    #[test]
    fn partial_clip_right_edge() {
        // x=1000 宽 100：右缘 1100>1080，裁到 80 列
        let c = clip_rect(1000, 0, 100, 10);
        assert_eq!(c, Some(ClipRect { x: 1000, y: 0, w: 80, h: 10 }));
    }

    #[test]
    fn boundary_x_1079_last_col() {
        // x=1079 仍是屏内最后一列（部分画 1 列）
        let c = clip_rect(1079, 0, 105, 11);
        assert_eq!(c, Some(ClipRect { x: 1079, y: 0, w: 1, h: 11 }));
    }

    #[test]
    fn zero_size_skipped() {
        assert_eq!(clip_rect(0, 0, 0, 10), None);
        assert_eq!(clip_rect(0, 0, 10, 0), None);
    }
}
