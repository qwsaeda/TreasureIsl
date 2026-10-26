//! S18.5：主地图相机 zoom/pan 纯函数模型（`world_canvas` 镜头）。
//!
//! 模型（1080×1920 逻辑 view，`08_render_arch.md` 单一坐标系）：
//! - 世界 = 网格像素空间（网格原点 = 左上，`y` 向下）；
//! - 相机中心 = 世界 px（[`WorldCam::cam`]）；`zoom` = 缩放因子（[`ZOOM_MIN`]..=[`ZOOM_MAX`]）；
//! - 投影：`screen = (world − cam) × zoom + view/2`（逆式即 `screen_to_world`）；
//! - `pan` 屏幕步长反 zoom（同屏步，zoom 大 → 世界步小）；
//! - `zoom_at` 锚点不变（锚点下世界点在 zoom 前后停在同一点）；
//! - 确定性：无 RNG / 无时间 → 相机单测可复现（DoD 判据）。
//!
//! 接线（S18.5）：Bevy 壳 = GPU 侧 zoom（`OrthographicProjection.scale` = 世界单位/像素 ⇒
//! 与 `zoom` **反比**：放大 ⇒ 可见世界收缩）+ 平移（sprite 位移）；
//! `grid_offset` 预留 CPU terrain 相机（`terrain_atlas::camera`）接线，
//! 落地挂 S18.6 像素基线重建（`08_render_arch` 回写同轮）。

/// zoom 下限（缩小端）。
pub const ZOOM_MIN: f32 = 0.5;

/// zoom 上限（放大端）。
pub const ZOOM_MAX: f32 = 2.0;

/// 相机状态（确定性：无 RNG / 无时间 / 无输入）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCam {
    /// 相机中心（世界 px；原点 = 网格左上）。
    pub cam: [f32; 2],
    /// 缩放因子（[`clamp_zoom`] 钳制）。
    pub zoom: f32,
}

impl Default for WorldCam {
    fn default() -> Self {
        // 中心对齐 1080×1920 逻辑 view 中心（默认 view）。
        Self { cam: [540.0, 960.0], zoom: 1.0 }
    }
}

/// 把 zoom 因子钳制到 [`ZOOM_MIN`]..=[`ZOOM_MAX`]（端点幂等）。
#[must_use]
pub fn clamp_zoom(zoom: f32) -> f32 {
    zoom.clamp(ZOOM_MIN, ZOOM_MAX)
}

impl WorldCam {
    /// 以 `anchor_s`（view 空间锚点，左上原点）为中心缩放 `factor` 倍。
    /// 锚点不变式：`screen_to_world(anchor)` 在 zoom 前后落在同一世界点
    /// （`factor` 使 zoom 未越界时精确成立；越界 = 钳制后端点保持）。
    pub fn zoom_at(&mut self, anchor_s: [f32; 2], factor: f32, view: [f32; 2]) {
        let before = self.screen_to_world(anchor_s, view);
        self.zoom = clamp_zoom(self.zoom * factor);
        self.cam = [
            before[0] - (anchor_s[0] - view[0] / 2.0) / self.zoom,
            before[1] - (anchor_s[1] - view[1] / 2.0) / self.zoom,
        ];
    }

    /// 平移：`step_x/step_y` = 屏幕 px 步长（世界步 = 屏步 / zoom，反 zoom）。
    pub fn pan(&mut self, step_x: f32, step_y: f32) {
        self.cam[0] += step_x / self.zoom;
        self.cam[1] += step_y / self.zoom;
    }

    /// view 空间点 → 世界 px。
    #[must_use]
    pub fn screen_to_world(&self, s: [f32; 2], view: [f32; 2]) -> [f32; 2] {
        [
            self.cam[0] + (s[0] - view[0] / 2.0) / self.zoom,
            self.cam[1] + (s[1] - view[1] / 2.0) / self.zoom,
        ]
    }

    /// 世界 px → view 空间点。
    #[must_use]
    pub fn world_to_screen(&self, w: [f32; 2], view: [f32; 2]) -> [f32; 2] {
        [
            (w[0] - self.cam[0]) * self.zoom + view[0] / 2.0,
            (w[1] - self.cam[1]) * self.zoom + view[1] / 2.0,
        ]
    }

    /// CPU terrain 相机（`terrain_atlas::camera` 的 `(i64, i64)` 网格偏移）接线：
    /// 相机中心 → 网格偏移（`cell_base` = 单格基准 px，向下取整）。
    /// 落地挂 S18.6（像素基线重建同轮回写 `08_render_arch` 降级口径）。
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn grid_offset(&self, cell_base: f32) -> [i64; 2] {
        let c = cell_base.max(1.0);
        // f32 floor → i32（`as` 截断；floor 后值域内安全）→ i64。
        [i64::from((self.cam[0] / c).floor() as i32), i64::from((self.cam[1] / c).floor() as i32)]
    }

    /// 纯缩放（view 中心锚点）。
    pub fn zoom_center(&mut self, factor: f32, view: [f32; 2]) {
        self.zoom_at([view[0] / 2.0, view[1] / 2.0], factor, view);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: [f32; 2] = [1080.0, 1920.0];

    #[test]
    fn default_center_maps_to_view_center() {
        let c = WorldCam::default();
        let s = c.world_to_screen([540.0, 960.0], VIEW);
        assert!((s[0] - 540.0).abs() < 1e-3 && (s[1] - 960.0).abs() < 1e-3);
    }

    #[test]
    fn world_screen_roundtrip() {
        let mut c = WorldCam { zoom: 1.5, ..WorldCam::default() };
        c.pan(-120.0, 340.0);
        let w = [731.0, 402.0];
        let s = c.world_to_screen(w, VIEW);
        let back = c.screen_to_world(s, VIEW);
        assert!(
            (back[0] - w[0]).abs() < 1e-2 && (back[1] - w[1]).abs() < 1e-2,
            "roundtrip 漂移：{back:?} vs {w:?}"
        );
    }

    #[test]
    fn zoom_anchor_invariant() {
        let mut c = WorldCam::default();
        let anchor: [f32; 2] = [300.0, 700.0];
        let world_before = c.screen_to_world(anchor, VIEW);
        c.zoom_at(anchor, 1.6, VIEW);
        let world_after = c.screen_to_world(anchor, VIEW);
        assert!(
            (world_after[0] - world_before[0]).abs() < 1e-3
                && (world_after[1] - world_before[1]).abs() < 1e-3,
            "锚点 zoom 必须不变（{world_before:?} → {world_after:?}）"
        );
    }

    #[test]
    fn pan_step_inverse_zoom() {
        let c = WorldCam { zoom: 1.0, ..WorldCam::default() };
        let c2 = WorldCam { zoom: 2.0, ..WorldCam::default() };
        let cam0 = c.cam;
        let cam2_0 = c2.cam;
        let mut c = c;
        let mut c2 = c2;
        c.pan(100.0, 100.0);
        c2.pan(100.0, 100.0);
        let d1 = c.cam[0] - cam0[0];
        let d2 = c2.cam[0] - cam2_0[0];
        assert!((d1 / d2 - 2.0).abs() < 1e-4, "同屏步：z=1 世界步须 = z=2 的 2 倍（{d1} vs {d2}）");
    }

    #[test]
    fn zoom_clamp_endpoints_idempotent() {
        let mut c = WorldCam::default();
        for _ in 0..32 {
            c.zoom_center(2.0, VIEW);
        }
        assert!((c.zoom - ZOOM_MAX).abs() < 1e-6, "连乘须收敛到 ZOOM_MAX：{}", c.zoom);
        for _ in 0..32 {
            c.zoom_center(0.5, VIEW);
        }
        assert!((c.zoom - ZOOM_MIN).abs() < 1e-6, "连除须收敛到 ZOOM_MIN：{}", c.zoom);
        let z_before = c.zoom;
        c.zoom_center(0.25, VIEW);
        assert_eq!(c.zoom, z_before, "越界 factor 钳制后须幂等");
    }

    #[test]
    fn grid_offset_matches_cam_div_cell() {
        let mut c = WorldCam::default();
        c.pan(-360.0, 512.0);
        // 默认 cam=(540,960) zoom=1：pan 后 cam=(180,1472) → cell=48 → (3,30)。
        assert_eq!(c.grid_offset(48.0), [3, 30], "grid 偏移 = floor(cam/cell)（{c:?}）");
        assert!(c.cam[0] > 0.0 && c.cam[1] > 0.0, "pan 后相机中心仍须在世界坐标内");
    }
}
