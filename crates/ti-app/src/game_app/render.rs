//! 渲染入口 + 动效 + 摄像机（每帧把场景画进绘制列表）。
//!
//! 拆分自 `game_app.rs`（S20.0b）：绘制**只做转发**，场景绘制逻辑在 `scenes/*.rs`
//!（`docs/process/08_render_arch.md` 契约）。

use super::{GameApp, TransitionState, advance_transition_t, approach_ms, transition_ease};
use crate::scenes::{RenderHost, SceneHost, SceneKind, route_render};

impl GameApp {
    /// S18.5：相机平移（壳层把鼠标/触摸拖拽的屏幕 px 步长喂入；世界步 = 屏步 / zoom）。
    pub fn camera_pan(&mut self, step_x: f32, step_y: f32) {
        self.world_cam.pan(step_x, step_y);
    }
    /// S18.5：以 view 空间锚点为中心缩放（滚轮喂入；zoom 钳制 [`crate::world_camera::clamp_zoom`]）。
    pub fn camera_zoom_at(&mut self, anchor_view: [f32; 2], factor: f32, view: [f32; 2]) {
        self.world_cam.zoom_at(anchor_view, factor, view);
    }
    /// S18.5：相机只读视图（Bevy 壳 GPU 侧 zoom/pan 接线）。
    #[must_use]
    pub fn world_cam(&self) -> &crate::world_camera::WorldCam {
        &self.world_cam
    }
    /// S18.5：FX 只读视图（确定性自检 / 视觉验收）。
    #[must_use]
    pub fn fx(&self) -> &crate::particles::ParticleFx {
        &self.fx
    }
    /// 渲染当前场景：场景模块绘制（`scenes/*.rs::render`），纯代码绘制
    /// （`08_render_arch.md` 契约 2：S15.4 起无 legacy 精灵层叠加）。
    ///
    /// 调用方随后调 [`MenuShell::present`] 上屏。
    pub fn render(&mut self) {
        // S18.7a：记录模式每帧先清空绘制列表（否则命令逐帧堆积 →
        // 实体 z 越叠越负 → 超出相机 far 平面 被裁切）。
        if self.record_draw {
            self.draw_list.clear();
        }
        // S17.4：动效推进全部时间制（本帧经过毫秒）——帧率无关（旧按帧计数）。
        crate::scenes::chrome::tick_press_flash(self.frame_delta_ms);
        // S18.5：FX 粒子推进（确定性：同 seed + 同 delta 序列 = 同画面；快照路径先 clear）。
        self.fx.advance(self.frame_delta_ms);
        // HP 条平滑插值（EXP-085）：每帧向实际值推进，步长 = 差值的 25% 或最小 1。
        self.update_displayed_hp();
        // 环岛赛道落格提示条：逐帧衰减（主地图渲染读 `board_toast` 时已过期则不画）。
        if self.board_toast_ticks > 0 {
            self.board_toast_ticks -= 1;
            if self.board_toast_ticks == 0 {
                self.board_toast = None;
            }
        }
        if self.transition.is_none() {
            self.render_scene();
            self.draw_fx();
            return;
        }
        // 取出过渡态（避开与 `render_scene` 的 `&mut self` 借用冲突）。
        let Some(mut trans) = self.transition.take() else {
            self.render_scene();
            self.draw_fx();
            return;
        };
        // S17.4：时间制推进（MOTION_TRANSITION_MS=240ms，帧率无关；delta 下限 1ms 防
        // `frame_delta_ms=0` 卡死过渡）。
        let next_t = advance_transition_t(trans.t, self.frame_delta_ms);
        let done = next_t >= 1.0;
        self.render_scene();
        if done {
            // 终帧：直接显示全新画面（不混合），过渡自然结束。
            self.draw_fx();
            return;
        }
        match trans.prev.take() {
            Some(prev) => {
                // 真交叉淡化：旧画面 → 新画面逐帧插值（EXP-081；S17.9：缓动 t，ease-in-out）。
                self.shell.crossfade_from(&prev, transition_ease(trans.t));
                trans.prev = Some(prev);
            }
            None => {
                // 无旧画面快照（进入前未渲染过）：退回亮度调制（S17.9 同套缓动）。
                self.shell.apply_brightness(transition_ease(trans.t));
            }
        }
        trans.t = next_t;
        self.transition = Some(trans);
        self.draw_fx();
    }
    /// S18.5：FX 粒子渲染（z = [`crate::z_layers::Z_FX`] 层；FX = 强调层 = 场景+chrome 之后绘制，
    /// 调用序 = 层序；`08_render_arch.md` 契约 1）。淡出 = 亮度 4 档量化（CPU Framebuf 无 alpha）。
    fn draw_fx(&mut self) {
        for (x, y, w, h, c) in self.fx.render_rects() {
            // S18.7a: FX 走**绘制目标**（CPU = 帧缓冲 / Bevy = draw list）→ 两后端一致。
            self.draw_target().fill_rect(x, y, w, h, c);
        }
    }
    /// 实际场景渲染（不含过渡逻辑）。
    fn render_scene(&mut self) {
        let kind = SceneKind::from_legacy(self.scene);
        // S15.4：legacy 像素写帧层移除后，场景渲染 = 纯代码绘制（chrome/menu_shell/scenes，
        // 1080×1920，`08_render_arch.md` 契约 2）。不再叠加 `.Map` 精灵层 / 地形瓦片。
        {
            let host: &mut dyn SceneHost = self;
            route_render(host, kind);
        }
        self.rendered_once = true;
    }
    /// HP 条平滑插值（EXP-085 → S17.4 时间制）：按 `MOTION_HP_LERP_MS`=160ms 线性趋近目标，
    /// 帧率无关（旧实现按帧推进差值的 25%）。
    fn update_displayed_hp(&mut self) {
        let Some(b) = self.battle.as_ref() else {
            return;
        };
        let frac = f64::from(self.frame_delta_ms)
            / f64::from(crate::scenes::chrome::MOTION_HP_LERP_MS.max(1));
        let target_p = b.player_hp;
        let target_b = b.boss_hp;
        self.displayed_player_hp = approach_ms(self.displayed_player_hp, target_p, frac);
        self.displayed_boss_hp = approach_ms(self.displayed_boss_hp, target_b, frac);
    }
    /// 场景切换过渡是否进行中（S17.9：主循环 render 门控用——过渡窗口内强制逐帧
    /// `render()`，否则 S4.3 静止帧跳渲染会把交叉淡化冻结在首帧 ~7% 旧画面，切屏残影）。
    #[must_use]
    pub fn transition_active(&self) -> bool {
        self.transition.is_some()
    }
    /// 启动场景切换过渡（EXP-081 真交叉淡化 → S17.4 时间制：`MOTION_TRANSITION_MS`=240ms）。
    ///
    /// 抓取当前帧缓冲作为「旧画面」；`enter()` 在切场景前调用本方法，
    /// 此刻帧缓冲仍持有旧场景像素，故快照有效。
    ///
    /// 两道守卫（否则首帧黑闪）：
    /// - 首帧前（`rendered_once == false`）无旧画面可混合 → 跳过过渡；
    /// - 过渡已在进行时再次切场景 → 沿用原 `prev` 并重置进度（不重抓半混合帧）。
    pub fn start_transition(&mut self) {
        if !self.rendered_once {
            self.transition = None;
            return;
        }
        if let Some(existing) = self.transition.as_mut() {
            existing.t = 0.0;
            return;
        }
        self.transition = Some(TransitionState { prev: Some(self.shell.fb.snapshot()), t: 0.0 });
    }
    /// 视觉回归基准（EXP-072）：渲染当前场景后返回 1080×1920 RGB565 帧缓冲只读切片。
    ///
    /// 测试专用（`tests/visual_regression.rs`）：调本方法拿像素 → 与
    /// `assets/hd_baseline/<场景>.png` 基准图逐像素 diff（RGB565→RGB 量化误差 ≤1）。
    /// 与 [`Self::render`] 同源（场景纯代码绘制，`08_render_arch.md` 契约 2），
    /// 不含 `present` 上屏副作用。
    ///
    /// 快照确定性：先清过渡态 + 重置动画帧计数（微光/翻页正弦），
    /// 使同一场景的快照像素与历史渲染次数无关（EXP-079 动画化回归防护）。
    /// S15.6 QA121：重置后预推 5 帧翻页计数（淡入窗口 0..=4 全过），快照锁实机
    /// **稳态**亮度 1.0——旧实现 `PAGE_FRAME`=0 时亮度 0.6，三套基线锁的是过渡中间态。
    #[must_use]
    pub fn render_to_framebuf(&mut self) -> &[u16] {
        // S18.5 快照口径（EXP-079 稳态）：像素基线 / selftest 不含 FX 粒子（clear 后渲染）；
        // 实玩路径（`render`）粒子照常。
        self.fx.clear();
        self.transition = None;
        crate::scenes::chrome::reset_animation_state();
        // S17.4：翻页亮度推进到稳态（时间制：一次推进满 `MOTION_PAGE_MS`）。
        crate::scenes::chrome::tick_page_frame(crate::scenes::chrome::MOTION_PAGE_MS);
        self.render();
        self.shell.fb.as_slice()
    }
}
