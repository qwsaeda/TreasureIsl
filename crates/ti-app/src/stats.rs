//! S4.3 性能诊断：每秒一次的平均 FPS 打印（`--features stats`）。
//!
//! 桌面 / wasm 壳主循环共用：每帧调 [`FpsMeter::tick`]，
//! 满 1 秒窗口时返回该窗口的 (平均 fps, 上传帧数, 跳帧数)，
//! 主循环自行决定打印格式。不依赖任何窗口 API，可在无头测试中驱动。

/// 帧率 + 呈现器计数观测器（S4.3：桌面 60fps 验收 + 脏行带跳帧率观测）。
///
/// 用法：主循环每帧取 [`ti_platform::gfx::FramePresenter`] 的
/// `uploads()` / `skipped()` 与单调毫秒时钟一起传入 [`tick`]；
/// 返回 `Some` 时打印一次窗口统计并继续累计。
#[derive(Debug)]
pub struct FpsMeter {
    frames: u64,
    prev_frames: u64,
    prev_uploads_total: u64,
    prev_skipped_total: u64,
    uploads_total: u64,
    skipped_total: u64,
    last_uploads: u64,
    last_skipped: u64,
    window_start_ms: u128,
}

impl FpsMeter {
    /// 新建观测器（首帧调用时建立窗口起点）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: 0,
            prev_frames: 0,
            prev_uploads_total: 0,
            prev_skipped_total: 0,
            uploads_total: 0,
            skipped_total: 0,
            last_uploads: 0,
            last_skipped: 0,
            window_start_ms: 0,
        }
    }

    /// 记录一帧。
    ///
    /// `now_ms`：单调毫秒时钟；`uploads` / `skipped`：呈现器**累计**计数
    /// （内部差分出本帧增量）。满 1 秒窗口时返回
    /// `Some((窗口平均 fps, 窗口上传帧数, 窗口跳帧数))` 并滚动窗口，
    /// 否则返回 `None`。
    #[must_use]
    pub fn tick(&mut self, now_ms: u128, uploads: u64, skipped: u64) -> Option<(f32, u64, u64)> {
        self.frames = self.frames.saturating_add(1);
        let up_delta = uploads.saturating_sub(self.last_uploads);
        let sk_delta = skipped.saturating_sub(self.last_skipped);
        self.uploads_total = self.uploads_total.saturating_add(up_delta);
        self.skipped_total = self.skipped_total.saturating_add(sk_delta);
        if self.frames == 1 {
            self.window_start_ms = now_ms;
        }
        self.last_uploads = uploads;
        self.last_skipped = skipped;
        let elapsed = now_ms.saturating_sub(self.window_start_ms);
        if elapsed < 1000 {
            return None;
        }
        let window_frames = self.frames.saturating_sub(self.prev_frames);
        let window_uploads = self.uploads_total.saturating_sub(self.prev_uploads_total);
        let window_skipped = self.skipped_total.saturating_sub(self.prev_skipped_total);
        let elapsed_ms = u16::try_from(elapsed).unwrap_or(u16::MAX);
        let secs = f32::from(elapsed_ms) / 1000.0;
        let frames_u16 = u16::try_from(window_frames).unwrap_or(u16::MAX);
        let avg = f32::from(frames_u16) / secs.max(1.0);
        self.prev_frames = self.frames;
        self.prev_uploads_total = self.uploads_total;
        self.prev_skipped_total = self.skipped_total;
        self.window_start_ms = now_ms;
        Some((avg, window_uploads, window_skipped))
    }

    /// 记录一帧（f64 秒时钟版，供 macroquad `get_time()` 直接驱动）。
    ///
    /// 内部把秒转换为饱和 u128 毫秒（溢出时钳位），其余语义与
    /// [`tick`](Self::tick) 相同。
    #[must_use]
    pub fn tick_seconds(
        &mut self,
        now_secs: f64,
        uploads: u64,
        skipped: u64,
    ) -> Option<(f32, u64, u64)> {
        // get_time() 在 2^31 秒（~68 年）内可精确表示整数秒；
        // 超出时钳位，不影响 1 秒窗口的 fps 统计。
        // 钳位到 u32::MAX 后 f64→u32 转换不丢值（值域已限定），
        // 允许 pedantic 的 cast 告警（见 docs/style.md §5.10 惯例）。
        let whole = if now_secs.is_finite() {
            let upper = f64::from(u32::MAX);
            let clamped = now_secs.max(0.0).min(upper);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let secs_u32 = clamped as u32;
            secs_u32
        } else {
            0
        };
        let now_ms = u128::from(u64::from(whole)) * 1000;
        self.tick(now_ms, uploads, skipped)
    }

    /// 累计上传帧数（全时段，供退出时汇总）。
    #[must_use]
    pub fn total_uploads(&self) -> u64 {
        self.uploads_total
    }

    /// 累计跳帧数（全时段，供退出时汇总）。
    #[must_use]
    pub fn total_skipped(&self) -> u64 {
        self.skipped_total
    }
}

impl Default for FpsMeter {
    fn default() -> Self {
        Self::new()
    }
}
